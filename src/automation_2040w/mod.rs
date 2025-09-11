//! Pin mappings for the Pimoroni Automation 2040 W board.

use embassy_rp::Peripherals;
use embassy_rp::gpio::{Flex, Input, Level, Output, Pull};
use embassy_rp::i2c::{self, Async, Config, I2c, InterruptHandler};
use embassy_rp::interrupt::typelevel::{Binding, I2C0_IRQ};
use embassy_rp::peripherals::I2C0;

#[allow(dead_code)]
pub struct Automation2040W<'d> {
    pub gp0: Flex<'d>,
    pub gp1: Flex<'d>,
    pub gp2: Flex<'d>,
    pub i2c: I2c<'d, I2C0, Async>,
    pub conn_led: Output<'d>,
    pub adc_led_1: Output<'d>,
    pub adc_led_2: Output<'d>,
    pub adc_led_3: Output<'d>,
    pub relay_1: Output<'d>,
    pub relay_2: Output<'d>,
    pub relay_3: Output<'d>,
    pub user_switch_a: Input<'d>,
    pub user_switch_b: Input<'d>,
    pub user_led_1: Output<'d>,
    pub user_led_2: Output<'d>,
    pub output_1: Output<'d>,
    pub output_2: Output<'d>,
    pub output_3: Output<'d>,
    pub in_buffered_1: Input<'d>,
    pub in_buffered_2: Input<'d>,
    pub in_buffered_3: Input<'d>,
    pub in_buffered_4: Input<'d>,
}

impl<'d> Automation2040W<'d> {
    pub fn new(p: Peripherals, irq: impl Binding<I2C0_IRQ, InterruptHandler<I2C0>>) -> Self {
        let gp0 = Flex::new(p.PIN_0);
        let gp1 = Flex::new(p.PIN_1);
        let gp2 = Flex::new(p.PIN_2);

        let i2c = i2c::I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, irq, Config::default());

        let conn_led = Output::new(p.PIN_3, Level::Low);

        let adc_led_1 = Output::new(p.PIN_6, Level::Low);
        let adc_led_2 = Output::new(p.PIN_7, Level::Low);
        let adc_led_3 = Output::new(p.PIN_8, Level::Low);

        let relay_1 = Output::new(p.PIN_9, Level::Low);
        let relay_2 = Output::new(p.PIN_10, Level::Low);
        let relay_3 = Output::new(p.PIN_11, Level::Low);

        let user_switch_a = Input::new(p.PIN_12, Pull::Up);
        let user_switch_b = Input::new(p.PIN_13, Pull::Up);

        let user_led_1 = Output::new(p.PIN_14, Level::Low);
        let user_led_2 = Output::new(p.PIN_15, Level::Low);

        let output_1 = Output::new(p.PIN_16, Level::Low);
        let output_2 = Output::new(p.PIN_17, Level::Low);
        let output_3 = Output::new(p.PIN_18, Level::Low);

        let in_buffered_1 = Input::new(p.PIN_19, Pull::Down);
        let in_buffered_2 = Input::new(p.PIN_20, Pull::Down);
        let in_buffered_3 = Input::new(p.PIN_21, Pull::Down);
        let in_buffered_4 = Input::new(p.PIN_22, Pull::Down);

        Self {
            gp0,
            gp1,
            gp2,
            i2c,
            conn_led,
            adc_led_1,
            adc_led_2,
            adc_led_3,
            relay_1,
            relay_2,
            relay_3,
            user_switch_a,
            user_switch_b,
            user_led_1,
            user_led_2,
            output_1,
            output_2,
            output_3,
            in_buffered_1,
            in_buffered_2,
            in_buffered_3,
            in_buffered_4,
        }
    }
}
