#![no_std]
#![no_main]

mod automation_2040w;
mod mcp23017;
mod net;
mod rats_nest;

use crate::rats_nest::RatsNest;
use defmt::*;
use edge_nal::TcpBind;
use edge_nal_embassy::TcpBuffers;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_rp::adc::{self};
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self};
use embassy_rp::peripherals::{I2C0, PIO0};
use embassy_rp::pio::{self};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use static_cell::StaticCell;
use symmetrical_octo_chainsaw_shared::http::run_server;
use symmetrical_octo_chainsaw_shared::pac_man_ball::{Inputs, Io, Outputs};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
pub async fn http_task(
    stack: Stack<'static>,
    ingress_signal: &'static Signal<CriticalSectionRawMutex, Outputs>,
    egress_signal: &'static Signal<CriticalSectionRawMutex, Inputs>,
) -> ! {
    let addr = "0.0.0.0:80".parse().expect("invalid address");

    static BUFFERS: StaticCell<TcpBuffers<4, 512, 512>> = StaticCell::new();
    let buffers = BUFFERS.init(TcpBuffers::new());

    let tcp = edge_nal_embassy::Tcp::new(stack, buffers);

    run_server(
        || async {
            info!("Binding to {}", addr);
            tcp.bind(addr).await
        },
        ingress_signal,
        egress_signal,
    )
    .await
}

#[embassy_executor::task]
async fn pipe_task(
    rats_nest: &'static mut RatsNest<'static, I2C0>,
    ingress_signal: &'static Signal<CriticalSectionRawMutex, Outputs>,
    egress_signal: &'static Signal<CriticalSectionRawMutex, Inputs>,
) -> ! {
    loop {
        let inputs = unwrap!(rats_nest.inputs().await);
        egress_signal.signal(inputs);

        if let Some(outputs) = ingress_signal.try_take() {
            unwrap!(rats_nest.set_outputs(outputs).await);
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let board = automation_2040w::Automation2040W::new(p, Irqs);

    let stack = net::init(
        spawner,
        option_env!("WIFI_SSID").unwrap_or("symmetrical-octo-chainsaw"),
        option_env!("WIFI_PASSPHRASE").unwrap_or("default_password"),
        board.conn_led,
        board.pwr,
        board.spi,
    )
    .await;

    static INGRESS_SIGNAL: StaticCell<Signal<CriticalSectionRawMutex, Outputs>> = StaticCell::new();
    let ingress_signal = INGRESS_SIGNAL.init(Signal::new());
    static EGRESS_SIGNAL: StaticCell<Signal<CriticalSectionRawMutex, Inputs>> = StaticCell::new();
    let egress_signal = EGRESS_SIGNAL.init(Signal::new());

    unwrap!(spawner.spawn(http_task(stack, ingress_signal, egress_signal)));

    let mut led = board.user_led_1;
    let i2c = board.i2c;

    static RATS_NEST: StaticCell<RatsNest<I2C0>> = StaticCell::new();
    let rats_nest = RATS_NEST.init(unwrap!(RatsNest::new(i2c).await));

    unwrap!(spawner.spawn(pipe_task(rats_nest, ingress_signal, egress_signal)));

    loop {
        Timer::after_secs(1).await;
        led.toggle();
    }
}
