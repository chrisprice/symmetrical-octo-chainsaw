//! Main application for Automation 2040 W
//! 
//! This application demonstrates using the BSP and logic components together.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp as _;
use embassy_futures::yield_now;
use automation_2040w_bsp::init;
use panic_halt as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the board
    let mut board = init();
    
    // Start background tasks
    if let Err(_) = board.start(&spawner).await {
        // Handle spawn error if needed
    }

    // Main application loop
    loop {
        // Process logic
        let result = board.logic.process().await;
        
        // Example: Use result to control relays
        // This is just a demonstration - in a real app you'd have actual logic
        board.set_relay(0, result > 10);
        
        // Yield control
        yield_now().await;
    }
}