#![allow(dead_code)]

use bitflags::bitflags;
use embassy_rp::i2c::{Async, I2c};
use embedded_hal_async::i2c::I2c as _;

pub const ADDR: u8 = 0x20; // default addr

macro_rules! mcpregs {
        ($($name:ident : $val:expr),* $(,)?) => {
            $(
                pub const $name: u8 = $val;
            )*

            pub fn regname(reg: u8) -> &'static str {
                match reg {
                    $(
                        $val => stringify!($name),
                    )*
                    _ => panic!("bad reg"),
                }
            }
        }
    }

// These are correct for IOCON.BANK=0
mcpregs! {
    IODIRA: 0x00,
    IPOLA: 0x02,
    GPINTENA: 0x04,
    DEFVALA: 0x06,
    INTCONA: 0x08,
    IOCONA: 0x0A,
    GPPUA: 0x0C,
    INTFA: 0x0E,
    INTCAPA: 0x10,
    GPIOA: 0x12,
    OLATA: 0x14,
    IODIRB: 0x01,
    IPOLB: 0x03,
    GPINTENB: 0x05,
    DEFVALB: 0x07,
    INTCONB: 0x09,
    IOCONB: 0x0B,
    GPPUB: 0x0D,
    INTFB: 0x0F,
    INTCAPB: 0x11,
    GPIOB: 0x13,
    OLATB: 0x15,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[repr(transparent)]
    pub struct Direction: u8 {
        const P0_IN = 1 << 0;
        const P1_IN = 1 << 1;
        const P2_IN = 1 << 2;
        const P3_IN = 1 << 3;
        const P4_IN = 1 << 4;
        const P5_IN = 1 << 5;
        const P6_IN = 1 << 6;
        const P7_IN = 1 << 7;
        const ALL_IN = 0xff;
        const ALL_OUT = 0x00;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[repr(transparent)]
    pub struct PullUp: u8 {
        const P0 = 1 << 0;
        const P1 = 1 << 1;
        const P2 = 1 << 2;
        const P3 = 1 << 3;
        const P4 = 1 << 4;
        const P5 = 1 << 5;
        const P6 = 1 << 6;
        const P7 = 1 << 7;
        const ALL = 0xff;
        const NONE = 0x00;
    }
}

pub struct Mcp23017<'a, 'd, I2C: embassy_rp::i2c::Instance> {
    i2c: &'a mut I2c<'d, I2C, Async>,
    address: u8,
}

impl<'a, 'd, I2C: embassy_rp::i2c::Instance> Mcp23017<'a, 'd, I2C> {
    pub fn new(i2c: &'a mut I2c<'d, I2C, Async>, address: u8) -> Self {
        Self { i2c, address }
    }

    pub async fn read_gpiob(&mut self) -> Result<u8, embassy_rp::i2c::Error> {
        let mut buffer = [0];
        self.i2c
            .write_read(self.address, &[GPIOB], &mut buffer)
            .await?;
        Ok(buffer[0])
    }

    pub async fn write_gpioa(&mut self, value: u8) -> Result<(), embassy_rp::i2c::Error> {
        self.i2c.write(self.address, &[GPIOA, value]).await?;
        Ok(())
    }

    pub async fn set_iodira(&mut self, directions: Direction) -> Result<(), embassy_rp::i2c::Error> {
        self.i2c
            .write(self.address, &[IODIRA, directions.bits()])
            .await?;
        Ok(())
    }

    pub async fn set_iodirb(&mut self, directions: Direction) -> Result<(), embassy_rp::i2c::Error> {
        self.i2c
            .write(self.address, &[IODIRB, directions.bits()])
            .await?;
        Ok(())
    }

    pub async fn set_gppub(&mut self, pull_ups: PullUp) -> Result<(), embassy_rp::i2c::Error> {
        self.i2c
            .write(self.address, &[GPPUB, pull_ups.bits()])
            .await?;
        Ok(())
    }
}
