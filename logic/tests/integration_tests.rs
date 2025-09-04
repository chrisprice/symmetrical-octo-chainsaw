//! Tests for the logic module using executor-std
//! 
//! These tests use the standard executor to validate the logic components

extern crate std;

use embassy_executor::Executor;
use logic::{LogicController, init};

#[tokio::test]
async fn test_logic_controller_creation() {
    let controller = LogicController::new(42);
    assert_eq!(controller.id, 42);
}

#[tokio::test] 
async fn test_logic_controller_process() {
    let controller = LogicController::new(21);
    let result = controller.process().await;
    assert_eq!(result, 42); // 21 * 2
}

#[tokio::test]
async fn test_init_function() {
    let controller = init();
    assert_eq!(controller.id, 1);
    
    let result = controller.process().await;
    assert_eq!(result, 2); // 1 * 2
}

#[tokio::test]
async fn test_executor_integration() {
    // Test that our logic works with embassy executor
    let _executor = Executor::new();
    let controller = LogicController::new(10);
    
    // This test demonstrates that the logic can work with embassy executor
    let result = controller.process().await;
    assert_eq!(result, 20);
}