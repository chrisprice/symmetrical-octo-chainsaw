use core::marker::PhantomData;

use defmt::*;
use edge_http::Method;
use edge_http::io::server::{Connection, Handler};
use edge_http::ws::MAX_BASE64_KEY_RESPONSE_LEN;
use edge_ws::{FrameHeader, FrameType};
use embedded_io_async::{Read, Write};

use crate::pac_man_ball::{Inputs, Outputs};

#[derive(Default)]
pub struct WsHandler;

impl Handler for WsHandler {
    type Error<E>
        = Error<E>
    where
        E: core::fmt::Debug;

    async fn handle<T, const N: usize>(
        &self,
        _task_id: impl core::fmt::Display + Clone,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        let headers = conn.headers()?;

        if headers.method != Method::Get {
            conn.initiate_response(405, Some("Method Not Allowed"), &[])
                .await?;
        } else if headers.path != "/" {
            conn.initiate_response(404, Some("Not Found"), &[]).await?;
        } else if !conn.is_ws_upgrade_request()? {
            conn.initiate_response(200, Some("OK"), &[("Content-Type", "text/plain")])
                .await?;

            conn.write_all(b"Initiate WS Upgrade request to switch this connection to WS")
                .await?;
        } else {
            let mut buf = [0_u8; MAX_BASE64_KEY_RESPONSE_LEN];
            conn.initiate_ws_upgrade_response(&mut buf).await?;

            conn.complete().await?;

            info!("Connection upgraded to WS");

            let mut socket = conn.unbind()?;

            let mut buf = [0_u8; 8192];

            loop {
                let mut header = FrameHeader::recv(&mut socket).await.map_err(Error::Ws)?;
                let payload = header
                    .recv_payload(&mut socket, &mut buf)
                    .await
                    .map_err(Error::Ws)?;

                match header.frame_type {
                    FrameType::Text(fragmented) => {
                        defmt::assert!(!fragmented, "Fragmented frames not supported");
                        let (outputs, length): (Outputs, _) = serde_json_core::from_slice(payload)?;
                        defmt::assert_eq!(length, payload.len(), "Did not consume full payload");
                        info!("Got {}, with payload \"{}\"", header, outputs);
                        let inputs = Inputs::default();
                        let size = serde_json_core::to_slice(&inputs, &mut buf)?;
                        header.mask_key = None;
                        header
                            .send_payload(&mut socket, &buf[..size])
                            .await
                            .map_err(Error::Ws)?;
                    }
                    FrameType::Close => {
                        info!("Got {}, client closed the connection cleanly", header);
                        break;
                    }
                    FrameType::Ping => {
                        info!("Got {}, will respond with Pong", header);
                        header.mask_key = None;
                        header.frame_type = FrameType::Pong;
                        header.send(&mut socket).await.map_err(Error::Ws)?;
                        continue;
                    }
                    _ => {
                        warn!("Unexpected {}, closing", header);
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error<E> {
    Connection(edge_http::io::Error<E>),
    Ws(edge_ws::Error<E>),
    JsonDe(serde_json_core::de::Error),
    JsonSer(serde_json_core::ser::Error),
}

impl<E> From<edge_http::io::Error<E>> for Error<E> {
    fn from(e: edge_http::io::Error<E>) -> Self {
        Self::Connection(e)
    }
}

impl<E> From<serde_json_core::de::Error> for Error<E> {
    fn from(e: serde_json_core::de::Error) -> Self {
        Self::JsonDe(e)
    }
}

impl<E> From<serde_json_core::ser::Error> for Error<E> {
    fn from(e: serde_json_core::ser::Error) -> Self {
        Self::JsonSer(e)
    }
}
