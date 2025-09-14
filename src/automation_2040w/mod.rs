//! Pin mappings for the Pimoroni Automation 2040 W board.

use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use embassy_rp::Peripherals;
use embassy_rp::adc::{self, Adc, Channel};
use embassy_rp::gpio::{Flex, Input, Level, Output, Pull};
use embassy_rp::i2c::{self, I2c};
use embassy_rp::interrupt::typelevel::{ADC_IRQ_FIFO, Binding, I2C0_IRQ, PIO0_IRQ_0};
use embassy_rp::peripherals::{DMA_CH0, I2C0, PIO0};
use embassy_rp::pio::Pio;
use embassy_rp::pio::{self};

#[allow(dead_code)]
pub struct Automation2040W<'d> {
    pub gp0: Flex<'d>,
    pub gp1: Flex<'d>,
    pub gp2: Flex<'d>,
    pub adc_0: Channel<'d>,
    pub adc_1: Channel<'d>,
    pub adc_2: Channel<'d>,
    pub i2c: I2c<'d, I2C0, i2c::Async>,
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
    pub pwr: Output<'d>,
    pub spi: PioSpi<'d, PIO0, 0, DMA_CH0>,
    pub adc: Adc<'d, adc::Async>,
}

impl<'d> Automation2040W<'d> {
    pub fn new(
        p: Peripherals,
        irqs: impl Binding<I2C0_IRQ, i2c::InterruptHandler<I2C0>>
        + Binding<PIO0_IRQ_0, pio::InterruptHandler<PIO0>>
        + Binding<ADC_IRQ_FIFO, adc::InterruptHandler>
        + Copy,
    ) -> Self {
        let gp0 = Flex::new(p.PIN_0);
        let gp1 = Flex::new(p.PIN_1);
        let gp2 = Flex::new(p.PIN_2);

        let adc = Adc::new(p.ADC, irqs, Default::default());
        let adc_0 = Channel::new_pin(p.PIN_26, Pull::None);
        let adc_1 = Channel::new_pin(p.PIN_27, Pull::None);
        let adc_2 = Channel::new_pin(p.PIN_28, Pull::None);

        let i2c = i2c::I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, irqs, Default::default());

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

        let pwr = Output::new(p.PIN_23, Level::Low);
        let cs = Output::new(p.PIN_25, Level::High);
        let mut pio = Pio::new(p.PIO0, irqs);
        let spi = PioSpi::new(
            &mut pio.common,
            pio.sm0,
            DEFAULT_CLOCK_DIVIDER,
            pio.irq0,
            cs,
            p.PIN_24,
            p.PIN_29,
            p.DMA_CH0,
        );

        Self {
            gp0,
            gp1,
            gp2,
            adc_0,
            adc_1,
            adc_2,
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
            pwr,
            spi,
            adc,
        }
    }
}
