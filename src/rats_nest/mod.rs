use embassy_rp::i2c::{Async, I2c};
use embedded_hal_async::i2c::I2c as _;

use crate::{
    mcp23017,
    pac_man_ball::{Inputs, Io, Outputs},
};

const PULL_UP_ENABLED: u8 = 0xFF;
const DIRECTION_OUTPUT: u8 = 0x00;
const DIRECTION_INPUT: u8 = 0xFF;

const ADDRESSES: [u8; 3] = [
    mcp23017::ADDR + 0x01,
    mcp23017::ADDR + 0x02,
    mcp23017::ADDR + 0x04,
];

pub struct RatsNest<'d, I2C: embassy_rp::i2c::Instance> {
    i2c: I2c<'d, I2C, Async>,
}

impl<'d, I2C: embassy_rp::i2c::Instance> RatsNest<'d, I2C> {
    pub async fn new(i2c: I2c<'d, I2C, Async>) -> Result<Self, embassy_rp::i2c::Error> {
        let mut ha = Self { i2c };
        for address in ADDRESSES {
            ha.configure(address).await?;
        }
        Ok(ha)
    }

    async fn configure(&mut self, address: u8) -> Result<(), embassy_rp::i2c::Error> {
        self.i2c
            .write_async(address, [mcp23017::IODIRA, DIRECTION_OUTPUT])
            .await?;
        self.i2c
            .write_async(address, [mcp23017::IODIRB, DIRECTION_INPUT])
            .await?;
        self.i2c
            .write_async(address, [mcp23017::GPPUB, PULL_UP_ENABLED])
            .await?;
        Ok(())
    }

    async fn read_portb(&mut self, address: u8) -> Result<u8, embassy_rp::i2c::Error> {
        let mut buffer = [0];
        self.i2c
            .write_read(address, &[mcp23017::GPIOB], &mut buffer)
            .await?;
        Ok(buffer[0])
    }

    async fn write_porta(&mut self, address: u8, value: u8) -> Result<(), embassy_rp::i2c::Error> {
        self.i2c
            .write_async(address, [mcp23017::GPIOA, value])
            .await?;
        Ok(())
    }
}

impl<'d, I2C: embassy_rp::i2c::Instance> Io for RatsNest<'d, I2C> {
    type Error = embassy_rp::i2c::Error;

    async fn inputs(&mut self) -> Result<Inputs, Self::Error> {
        let values = [
            self.read_portb(ADDRESSES[0]).await?,
            self.read_portb(ADDRESSES[1]).await?,
            self.read_portb(ADDRESSES[2]).await?,
        ];
        Ok(Inputs {
            checker_0_sensor: values[2] & 0x01 == 0,
            checker_1_sensor: values[2] & 0x02 == 0,
            checker_2_sensor: values[2] & 0x04 == 0,
            checker_3_sensor: values[2] & 0x08 == 0,
            checker_4_sensor: values[2] & 0x10 == 0,
            checker_5_sensor: values[2] & 0x20 == 0,
            checker_6_sensor: values[2] & 0x40 == 0,
            tilt_switch: values[2] & 0x80 == 0,
            left_in_sensor_1: values[1] & 0x01 == 0,
            left_in_sensor_2: values[1] & 0x02 == 0,
            right_in_sensor_1: values[1] & 0x04 == 0,
            right_in_sensor_2: values[1] & 0x08 == 0,
            hopper_left_sensor: values[1] & 0x10 == 0,
            hopper_right_sensor: values[1] & 0x20 == 0,
            hopper_out_sensor: values[1] & 0x40 == 0,
            table_sensor: values[1] & 0x80 == 0,
            left_divider_sensor: values[0] & 0x01 == 0,
            right_divider_sensor: values[0] & 0x02 == 0,
            test_switch: values[0] & 0x10 == 0,
            select_switch_up: values[0] & 0x20 == 0,
            select_switch_down: values[0] & 0x40 == 0,
            enter_switch: values[0] & 0x80 == 0,
        })
    }

    async fn set_outputs(&mut self, outputs: Outputs) -> Result<(), Self::Error> {
        let mut values = [0u8; 3];
        values[0] |= if outputs.checker_0_led { 0x01 } else { 0x00 };
        values[0] |= if outputs.checker_1_led { 0x02 } else { 0x00 };
        values[0] |= if outputs.checker_2_led { 0x04 } else { 0x00 };
        values[0] |= if outputs.checker_3_led { 0x08 } else { 0x00 };
        values[0] |= if outputs.checker_4_led { 0x10 } else { 0x00 };
        values[0] |= if outputs.checker_5_led { 0x20 } else { 0x00 };
        values[0] |= if outputs.checker_6_led { 0x40 } else { 0x00 };
        values[0] |= if outputs.table_motor { 0x80 } else { 0x00 };
        values[1] |= if outputs.left_hopper { 0x01 } else { 0x00 };
        values[1] |= if outputs.right_hopper { 0x02 } else { 0x00 };
        values[1] |= if outputs.lockout_solenoid_left { 0x04 } else { 0x00 };
        values[1] |= if outputs.lockout_solenoid_right { 0x08 } else { 0x00 };
        values[1] |= if outputs.out_hopper { 0x10 } else { 0x00 };
        values[1] |= if outputs.payout_solenoid { 0x20 } else { 0x00 };
        values[1] |= if outputs.divider_solenoid_left { 0x40 } else { 0x00 };
        values[1] |= if outputs.divider_solenoid_right { 0x80 } else { 0x00 };
        values[2] |= if outputs.ray_lamp { 0x04 } else { 0x00 };
        for (address, value) in ADDRESSES.iter().zip(values.iter()) {
            self.write_porta(*address, *value).await?;
        }
        Ok(())
    }
}
