use anyhow::Error;
use edge_nal::TcpBind;
use edge_nal_std::Stack;
use futures_lite::future::block_on;
use symmetrical_octo_chainsaw_shared::http::run_server;

fn main() {
    block_on(run());
}

pub async fn run() -> ! {
    let addr = "0.0.0.0:8881";

    println!("Running HTTP server on {addr}");

    run_server(|| async move {
        let acceptor = Stack::new().bind(addr.parse().unwrap()).await?;
        Ok::<_, Error>(acceptor)
    })
    .await
}
