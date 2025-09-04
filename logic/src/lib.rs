//! Logic module for embassy-based applications
//! 
//! This module provides reusable logic components that can be used
//! across different embassy applications and BSPs.

#![no_std]

use embassy_executor::Spawner;
use embassy_futures::yield_now;

/// A simple logic component that demonstrates embassy task usage
pub struct LogicController {
    pub id: u32,
}

impl LogicController {
    /// Create a new logic controller
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    /// Process some logic asynchronously
    pub async fn process(&self) -> u32 {
        // Simulate some async work
        yield_now().await;
        self.id * 2
    }

    /// Spawn a background task
    pub fn spawn_background_task(&self, spawner: &Spawner) -> Result<(), embassy_executor::SpawnError> {
        spawner.spawn(background_task(self.id))
    }
}

/// A background task that can be spawned
#[embassy_executor::task]
async fn background_task(_id: u32) {
    loop {
        // Do some background work
        yield_now().await;
        // In real implementation, this would do actual work
    }
}

/// Initialize the logic system
pub fn init() -> LogicController {
    LogicController::new(1)
}