#![no_std]
#![no_main]

mod automation_2040w;
mod mcp23017;
mod pac_man_ball;
mod rats_nest;

use crate::{
    pac_man_ball::{Io, Outputs},
    rats_nest::RatsNest,
};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{self};
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self};
use embassy_rp::peripherals::{I2C0, PIO0};
use embassy_rp::pio::{self};
use embassy_time::Timer;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let board = automation_2040w::Automation2040W::new(p, Irqs);

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
