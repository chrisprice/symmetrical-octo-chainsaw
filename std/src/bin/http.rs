use anyhow::Error;
use edge_nal::TcpBind;
use edge_nal_std::Stack;
use embassy_time::{Duration, Timer};
use futures_lite::future::{block_on, or};
use log::info;
use symmetrical_octo_chainsaw_shared::{
    http::{run_server, ws::WsHandler},
    pac_man_ball::Inputs,
};

fn main() {
    block_on(or(run(), or(fake_inputs(), print_outputs())));
}

pub async fn fake_inputs() -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        WsHandler::signal_inputs(Inputs {
            checker_0_sensor: rand::random_bool(0.1),
            checker_1_sensor: rand::random_bool(0.1),
            checker_2_sensor: rand::random_bool(0.1),
            checker_3_sensor: rand::random_bool(0.1),
            checker_4_sensor: rand::random_bool(0.1),
            checker_5_sensor: rand::random_bool(0.1),
            checker_6_sensor: rand::random_bool(0.1),
            tilt_switch: rand::random_bool(0.1),
            left_in_sensor_1: rand::random_bool(0.1),
            left_in_sensor_2: rand::random_bool(0.1),
            right_in_sensor_1: rand::random_bool(0.1),
            right_in_sensor_2: rand::random_bool(0.1),
            hopper_left_sensor: rand::random_bool(0.1),
            hopper_right_sensor: rand::random_bool(0.1),
            hopper_out_sensor: rand::random_bool(0.1),
            table_sensor: rand::random_bool(0.1),
            left_divider_sensor: rand::random_bool(0.1),
            right_divider_sensor: rand::random_bool(0.1),
            test_switch: rand::random_bool(0.1),
            select_switch_up: rand::random_bool(0.1),
            select_switch_down: rand::random_bool(0.1),
            enter_switch: rand::random_bool(0.1),
        });
    }
}

pub async fn print_outputs() -> ! {
    loop {
        let outputs = WsHandler::wait_for_outputs().await;
        info!("Outputs: {outputs:?}");
    }
}

pub async fn run() -> ! {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let addr = "0.0.0.0:8881";

    info!("Running HTTP server on {addr}");

    run_server(|| async move {
        let acceptor = Stack::new().bind(addr.parse().unwrap()).await?;
        Ok::<_, Error>(acceptor)
    })
    .await
}
