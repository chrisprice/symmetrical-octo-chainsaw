use defmt::*;
use edge_http::io::server::DefaultServer;
use edge_nal::TcpBind;
use edge_nal_embassy::TcpBuffers;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

pub mod ws;

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
        if server
            .run(None, acceptor, ws::WsHandler)
            .await
            .is_err()
        {
            warn!("Server error, restarting");
            Timer::after(Duration::from_secs(1)).await;
        }
    }
}
