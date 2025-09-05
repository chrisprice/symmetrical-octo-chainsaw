#![no_std]
#![no_main]

mod automation_2040w;
mod mcp23017;
mod pac_man_ball;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::InterruptHandler;
use embassy_rp::peripherals::I2C0;
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use {defmt_rtt as _, panic_probe as _};


bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let board = automation_2040w::Automation2040W::new(p, Irqs);

    let mut led = board.user_led_1;
    let button = board.user_switch_a;

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }

    let mut i2c = board.i2c;

    use mcp23017::*;

    info!("init mcp23017 config for IxpandO");
    // init - a outputs, b inputs
    i2c.write(ADDR, &[IODIRA, 0x00]).await.unwrap();
    i2c.write(ADDR, &[IODIRB, 0xff]).await.unwrap();
    i2c.write(ADDR, &[GPPUB, 0xff]).await.unwrap(); // pullups

    let mut val = 1;
    loop {
        let mut portb = [0];

        i2c.write_read(mcp23017::ADDR, &[GPIOB], &mut portb)
            .await
            .unwrap();
        info!("portb = {:02x}", portb[0]);
        i2c.write(mcp23017::ADDR, &[GPIOA, val | portb[0]])
            .await
            .unwrap();
        val = val.rotate_left(1);

        // get a register dump
        info!("getting register dump");
        let mut regs = [0; 22];
        i2c.write_read(ADDR, &[0], &mut regs).await.unwrap();
        // always get the regdump but only display it if portb'0 is set
        if portb[0] & 1 != 0 {
            for (idx, reg) in regs.into_iter().enumerate() {
                info!("{} => {:02x}", regname(idx as u8), reg);
            }
        }

        Timer::after_millis(100).await;
    }
}
