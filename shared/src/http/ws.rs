use edge_http::io::server::{Connection, Handler};
use edge_http::ws::MAX_BASE64_KEY_RESPONSE_LEN;
use edge_http::Method;
use edge_ws::{FrameHeader, FrameType};
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embedded_io_async::{Read, Write};
use serde_json_core::heapless::{self, String};

use crate::pac_man_ball::{Inputs, Outputs};

static INPUTS: Signal<CriticalSectionRawMutex, Inputs> = Signal::new();
static OUTPUTS: Signal<CriticalSectionRawMutex, Outputs> = Signal::new();

fn is_allowed_origin(origin: &str) -> bool {
    // The `Origin` header is `scheme "://" host [ ":" port ]`.
    // We are interested in the host part.
    if let Some(host_part) = origin.split("://").nth(1) {
        let host = host_part.split(':').next().unwrap_or("");
        host == "chrisprice.dev" || host == "localhost"
    } else {
        false
    }
}

#[derive(Default)]
pub struct WsHandler;

impl WsHandler {
    pub fn signal_inputs(&self, inputs: Inputs) {
        INPUTS.signal(inputs);
    }
    pub async fn wait_for_outputs(&self) -> Outputs {
        OUTPUTS.wait().await
    }
}

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

        let origin: Option<String<256>> = headers
            .headers
            .get("Origin")
            .map(String::<256>::try_from)
            .map(|r| r.unwrap_or_default());
        let is_allowed = origin
            .as_ref()
            .map(|origin| is_allowed_origin(&origin))
            .unwrap_or(false);

        if headers.method == Method::Options {
            if is_allowed {
                let request_headers: String<256> = String::try_from(
                    headers
                        .headers
                        .get("Access-Control-Request-Headers")
                        .unwrap_or(""),
                )
                .unwrap_or_default();
                conn.initiate_response(
                    204,
                    None,
                    &[
                        ("Access-Control-Allow-Origin", &origin.unwrap()),
                        ("Access-Control-Allow-Methods", "GET, OPTIONS"),
                        ("Access-Control-Allow-Headers", &request_headers),
                        ("Access-Control-Max-Age", "86400"),
                    ],
                )
                .await?;
            } else {
                // Or a 403 Forbidden, but this is simpler
                conn.initiate_response(204, None, &[]).await?;
            }
        } else if headers.method != Method::Get {
            conn.initiate_response(405, Some("Method Not Allowed"), &[])
                .await?;
        } else if headers.path != "/" {
            let cors_headers: &[(&str, &str)] = if is_allowed {
                &[("Access-Control-Allow-Origin", &origin.unwrap())]
            } else {
                &[]
            };
            conn.initiate_response(404, Some("Not Found"), cors_headers)
                .await?;
        } else if !conn.is_ws_upgrade_request()? {
            let mut response_headers = [("Content-Type", "text/plain"), ("", "")];
            let mut headers_len = 1;
            if is_allowed {
                if let Some(origin) = origin.as_ref() {
                    response_headers[1] = ("Access-Control-Allow-Origin", &origin);
                    headers_len = 2;
                }
            }

            conn.initiate_response(200, Some("OK"), &response_headers[..headers_len])
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
                match select(FrameHeader::recv(&mut socket), INPUTS.wait()).await {
                    Either::First(header) => {
                        let header = header.map_err(Error::Ws)?;
                        let payload = header
                            .recv_payload(&mut socket, &mut buf)
                            .await
                            .map_err(Error::Ws)?;
                        match header.frame_type {
                            FrameType::Text(fragmented) => {
                                assert!(!fragmented, "Fragmented frames not supported");
                                let (outputs, length): (Outputs, _) =
                                    serde_json_core::from_slice(payload)?;
                                assert_eq!(length, payload.len(), "Did not consume full payload");
                                info!("Got {}, with payload \"{:?}\"", header, outputs);
                                OUTPUTS.signal(outputs);
                            }
                            FrameType::Close => {
                                info!("Got {}, client closed the connection cleanly", header);
                                break;
                            }
                            _ => {
                                warn!("Unexpected {}, closing", header);
                                break;
                            }
                        }
                    }
                    Either::Second(payload) => {
                        let payload_len = serde_json_core::to_slice(&payload, &mut buf)?
                            .try_into()
                            .expect("buffer.len() << u64::MAX");
                        let header = FrameHeader {
                            frame_type: FrameType::Text(false),
                            payload_len,
                            mask_key: None,
                        };
                        header.send(&mut socket).await.map_err(Error::Ws)?;
                        header
                            .send_payload(&mut socket, &buf)
                            .await
                            .map_err(Error::Ws)?;
                    }
                };
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
