use defmt::*;
use serde::{Deserialize, Serialize};

pub trait Io {
    type Error;
    async fn inputs(&mut self) -> Result<Inputs, Self::Error>;
    async fn set_outputs(&mut self, outputs: Outputs) -> Result<(), Self::Error>;
}

#[derive(Serialize, Deserialize, Format, PartialEq, Eq, Clone, Debug, Default)]
pub struct Inputs {
    pub checker_0_sensor: bool,
    pub checker_1_sensor: bool,
    pub checker_2_sensor: bool,
    pub checker_3_sensor: bool,
    pub checker_4_sensor: bool,
    pub checker_5_sensor: bool,
    pub checker_6_sensor: bool,
    pub tilt_switch: bool,
    pub left_in_sensor_1: bool,
    pub left_in_sensor_2: bool,
    pub right_in_sensor_1: bool,
    pub right_in_sensor_2: bool,
    pub hopper_left_sensor: bool,
    pub hopper_right_sensor: bool,
    pub hopper_out_sensor: bool,
    pub table_sensor: bool,
    pub left_divider_sensor: bool,
    pub right_divider_sensor: bool,
    pub test_switch: bool,
    pub select_switch_up: bool,
    pub select_switch_down: bool,
    pub enter_switch: bool,
}

#[derive(Serialize, Deserialize, Format, PartialEq, Eq, Clone, Debug, Default)]
pub struct Outputs {
    pub checker_0_led: bool,
    pub checker_1_led: bool,
    pub checker_2_led: bool,
    pub checker_3_led: bool,
    pub checker_4_led: bool,
    pub checker_5_led: bool,
    pub checker_6_led: bool,
    pub table_motor: bool,
    pub left_hopper: bool,
    pub right_hopper: bool,
    pub lockout_solenoid_left: bool,
    pub lockout_solenoid_right: bool,
    pub out_hopper: bool,
    pub payout_solenoid: bool,
    pub divider_solenoid_left: bool,
    pub divider_solenoid_right: bool,
    pub ray_lamp: bool,
}
