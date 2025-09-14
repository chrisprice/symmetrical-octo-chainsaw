use defmt::*;
use edge_http::io::server::{Connection, DefaultServer, Handler};
use edge_http::io::Error;
use edge_http::ws::MAX_BASE64_KEY_RESPONSE_LEN;
use edge_http::Method;
use edge_nal::TcpBind;
use edge_nal_embassy::TcpBuffers;
use edge_ws::{FrameHeader, FrameType};
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;

#[embassy_executor::task]
pub async fn http_task(stack: Stack<'static>) -> ! {
    let addr = "0.0.0.0:80".parse().expect("invalid address");

    static BUFFERS: StaticCell<TcpBuffers<1, 512, 512>> = StaticCell::new();
    let buffers = BUFFERS.init(TcpBuffers::new());

    let tcp = edge_nal_embassy::Tcp::new(stack, buffers);
    loop {
        let Ok(acceptor) = tcp.bind(addr).await else {
            warn!("Failed to bind to {}, retrying", addr);
            Timer::after(Duration::from_secs(1)).await;
            continue;
        };

        info!("Listening on {}", addr);

        let mut server = DefaultServer::new();
        if server.run(None, acceptor, WsHandler).await.is_err() {
            warn!("Server error, restarting");
            Timer::after(Duration::from_secs(1)).await;
        }
    }
}

#[derive(Debug)]
enum WsHandlerError<C, W> {
    Connection(C),
    Ws(W),
}

impl<C, W> From<C> for WsHandlerError<C, W> {
    fn from(e: C) -> Self {
        Self::Connection(e)
    }
}

struct WsHandler;

impl Handler for WsHandler {
    type Error<E>
        = WsHandlerError<Error<E>, edge_ws::Error<E>>
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

            info!("Connection upgraded to WS, starting a simple WS echo server now");

            // Now we have the TCP socket in a state where it can be operated as a WS connection
            // Run a simple WS echo server here

            let mut socket = conn.unbind()?;

            let mut buf = [0_u8; 8192];

            loop {
                let mut header = FrameHeader::recv(&mut socket)
                    .await
                    .map_err(WsHandlerError::Ws)?;
                let payload = header
                    .recv_payload(&mut socket, &mut buf)
                    .await
                    .map_err(WsHandlerError::Ws)?;

                match header.frame_type {
                    FrameType::Text(_) => {
                        info!(
                            "Got {}, with payload \"{}\"",
                            header,
                            core::str::from_utf8(payload).unwrap()
                        );
                    }
                    FrameType::Binary(_) => {
                        info!("Got {}, with payload {:?}", header, payload);
                    }
                    FrameType::Close => {
                        info!("Got {}, client closed the connection cleanly", header);
                        break;
                    }
                    _ => {
                        info!("Got {}", header);
                    }
                }

                // Echo it back now

                header.mask_key = None; // Servers never mask the payload

                if matches!(header.frame_type, FrameType::Ping) {
                    header.frame_type = FrameType::Pong;
                }

                info!("Echoing back as {}", header);

                header.send(&mut socket).await.map_err(WsHandlerError::Ws)?;
                header
                    .send_payload(&mut socket, payload)
                    .await
                    .map_err(WsHandlerError::Ws)?;
            }
        }

        Ok(())
    }
}
