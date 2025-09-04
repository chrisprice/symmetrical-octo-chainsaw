# Main Application

Example application for the Automation 2040 W board using Embassy framework.

## Features

- Demonstrates BSP and logic integration
- Embassy async main loop
- Embedded target configuration for RP2040

## Building

This project targets embedded hardware:

```bash
# Check for embedded target
cargo check --target thumbv6m-none-eabi

# Build for RP2040 (requires probe-rs for flashing)
cargo build --release --target thumbv6m-none-eabi
```

## Configuration

- Target: `thumbv6m-none-eabi` (Cortex-M0+)  
- Linker: Uses memory.x for RP2040 memory layout
- Embassy executor with Cortex-M architecture

## Dependencies

- `automation-2040w-bsp` - Board support package
- `logic` - Logic components
- `embassy-executor` with Cortex-M features
- `embassy-rp` for RP2040 support
- `panic-halt` for panic handling