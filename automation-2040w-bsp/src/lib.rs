//! Board Support Package for Automation 2040 W
//! 
//! This BSP provides hardware abstraction for the Pimoroni Automation 2040 W board,
//! which is based on the Raspberry Pi RP2040 microcontroller.

#![no_std]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Output};
use logic::LogicController;

/// Automation 2040 W board configuration and peripherals
pub struct Automation2040W {
    /// Logic controller instance
    pub logic: LogicController,
    /// Relay outputs (8 relays)
    pub relays: [Option<Output<'static>>; 8],
    /// Digital inputs (4 inputs)  
    pub inputs: [Option<Input<'static>>; 4],
}

impl Automation2040W {
    /// Initialize the Automation 2040 W board
    pub fn new() -> Self {
        Self {
            logic: logic::init(),
            relays: [None, None, None, None, None, None, None, None],
            inputs: [None, None, None, None],
        }
    }

    /// Start the board's background tasks
    pub async fn start(&self, spawner: &Spawner) -> Result<(), embassy_executor::SpawnError> {
        self.logic.spawn_background_task(spawner)
    }

    /// Control a relay by index (0-7)
    pub fn set_relay(&mut self, index: usize, state: bool) {
        if let Some(ref mut relay) = self.relays.get_mut(index).and_then(|r| r.as_mut()) {
            if state {
                relay.set_high();
            } else {
                relay.set_low();
            }
        }
    }

    /// Read a digital input by index (0-3)
    pub fn read_input(&self, index: usize) -> Option<bool> {
        self.inputs.get(index)
            .and_then(|input| input.as_ref())
            .map(|input| input.is_low())
    }
}

/// Initialize the board with all peripherals
pub fn init() -> Automation2040W {
    Automation2040W::new()
}