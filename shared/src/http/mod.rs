use edge_http::io::server::DefaultServer;
use edge_nal::TcpAccept;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};

use crate::{
    http::ws::WsHandler,
    pac_man_ball::{Inputs, Outputs},
};

pub mod ws;

pub async fn run_server<F, Fut, A, E>(
    mut acceptor_fn: F,
    ingress_signal: &Signal<CriticalSectionRawMutex, Outputs>,
    egress_signal: &Signal<CriticalSectionRawMutex, Inputs>,
) -> !
where
    F: FnMut() -> Fut,
    Fut: core::future::Future<Output = Result<A, E>>,
    A: TcpAccept,
{
    loop {
        let acceptor = match acceptor_fn().await {
            Ok(acceptor) => acceptor,
            Err(_) => {
                warn!("Failed to bind, retrying");
                Timer::after(Duration::from_secs(1)).await;
                continue;
            }
        };

        info!("Server running");

        let mut server = DefaultServer::new();
        let handler = WsHandler::new(ingress_signal, egress_signal);
        if server.run(None, acceptor, handler).await.is_err() {
            warn!("Server error, restarting");
            Timer::after(Duration::from_secs(1)).await;
        }
    }
}
