#![no_std]
#![no_main]

mod automation_2040w;
mod http;
mod mcp23017;
mod pac_man_ball;
mod rats_nest;

use crate::{
    http::http_task,
    pac_man_ball::{Io, Outputs},
    rats_nest::RatsNest,
};
use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_rp::i2c::{self};
use embassy_rp::peripherals::{I2C0, PIO0};
use embassy_rp::pio::{self};
use embassy_rp::{
    adc::{self},
    gpio::Output,
    peripherals::DMA_CH0,
};
use embassy_rp::{bind_interrupts, clocks::RoscRng};
use embassy_time::Duration;
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn conn_led_task(mut led: Output<'static>, stack: Stack<'static>) -> ! {
    loop {
        // Slow blink until config up
        while !stack.is_config_up() {
            led.set_high();
            Timer::after(Duration::from_millis(500)).await;
            led.set_low();
            Timer::after(Duration::from_millis(500)).await;
        }

        // Fast blink until link up
        while !stack.is_link_up() {
            led.set_high();
            Timer::after(Duration::from_millis(100)).await;
            led.set_low();
            Timer::after(Duration::from_millis(100)).await;
        }

        // Solid on while link is up
        led.set_high();
        stack.wait_link_down().await;

        // If link goes down, start over (loop restarts)
        led.set_low();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let board = automation_2040w::Automation2040W::new(p, Irqs);

    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download cyw43-firmware/43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, board.pwr, board.spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(embassy_net::Ipv4Address::new(169, 254, 1, 1), 16),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(net_device, config, RESOURCES.init(StackResources::new()), {
            let mut rng = RoscRng;
            rng.next_u64()
        });

    unwrap!(spawner.spawn(net_task(runner)));
    unwrap!(spawner.spawn(conn_led_task(board.user_led_2, stack)));

    control
        .start_ap_wpa2(
            "symmetrical-octo-chainsaw",
            option_env!("WIFI_PASSWORD").unwrap_or("default_password"),
            5,
        )
        .await;

    unwrap!(spawner.spawn(http_task(stack)));

    let mut led = board.user_led_1;
    let button = board.user_switch_a;
    let i2c = board.i2c;

    let mut ha = RatsNest::new(i2c).await.unwrap();
    let rats_nest = &mut ha;

    loop {
        let inputs = rats_nest.inputs().await.unwrap();
        info!("inputs: {:?}", inputs);

        let mut outputs = Outputs::default();
        if button.is_high() {
            led.set_high();
            outputs.checker_0_led = true;
        } else {
            led.set_low();
            outputs.checker_0_led = false;
        }
        info!("outputs: {:?}", outputs);
        rats_nest.set_outputs(outputs).await.unwrap();

        Timer::after_millis(1000).await;
    }
}
