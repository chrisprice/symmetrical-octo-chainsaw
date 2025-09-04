# Automation 2040 W BSP

Board Support Package for the Pimoroni Automation 2040 W board.

## Features

- RP2040 microcontroller support via embassy-rp
- 8 relay outputs control
- 4 digital inputs reading
- Integration with logic components

## Hardware

The Automation 2040 W is an RP2040-based board featuring:
- 8 relay outputs for automation control
- 4 buffered digital inputs
- Industrial-grade design
- WiFi connectivity

## Usage

```rust
use automation_2040w_bsp::init;

// Initialize board
let mut board = init();

// Start background tasks
board.start(&spawner).await?;

// Control relays
board.set_relay(0, true);  // Turn on relay 0

// Read inputs
if let Some(state) = board.read_input(0) {
    // Handle input state
}
```

## Dependencies

- `embassy-rp` - RP2040 hardware abstraction
- `logic` - Reusable logic components