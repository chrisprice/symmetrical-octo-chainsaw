use defmt::*;

#[derive(Format, PartialEq, Eq, Clone, Debug, Default)]
pub struct Inputs {
    checker_0_sensor: bool,
    checker_1_sensor: bool,
    checker_2_sensor: bool,
    checker_3_sensor: bool,
    checker_4_sensor: bool,
    checker_5_sensor: bool,
    checker_6_sensor: bool,
    tilt_switch: bool,
    left_in_sensor_1: bool,
    left_in_sensor_2: bool,
    right_in_sensor_1: bool,
    right_in_sensor_2: bool,
    hopper_left_sensor: bool,
    hopper_right_sensor: bool,
    hopper_out_sensor: bool,
    table_sensor: bool,
    left_divider_sensor: bool,
    right_divider_sensor: bool,
    test_switch: bool,
    select_switch_up: bool,
    select_switch_down: bool,
    enter_switch: bool,
}

#[derive(Format, PartialEq, Eq, Clone, Debug, Default)]
pub struct Outputs {
    checker_0_led: bool,
    checker_1_led: bool,
    checker_2_led: bool,
    checker_3_led: bool,
    checker_4_led: bool,
    checker_5_led: bool,
    checker_6_led: bool,
    table_motor: bool,
    left_hopper: bool,
    right_hopper: bool,
    lockout_solenoid_left: bool,
    lockout_solenoid_right: bool,
    out_hopper: bool,
    payout_solenoid: bool,
    divider_solenoid_left: bool,
    divider_solenoid_right: bool,
}

