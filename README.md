# Symmetrical Octo Chainsaw

A Rust workspace for the Automation 2040 W board featuring Embassy async framework.

## Projects

### Logic (`logic/`)
Core logic components using embassy-executor, designed to be `no_std` compatible for embedded use while supporting `std` for testing.

**Dependencies:**
- `embassy-executor` - Core dependency
- Tests use `executor-std` for comprehensive testing

**Features:**
- Reusable logic controllers
- Async task management
- Comprehensive test suite

### Automation 2040 W BSP (`automation-2040w-bsp/`)
Board Support Package for the Pimoroni Automation 2040 W board (RP2040-based).

**Dependencies:**
- `embassy-rp` - Hardware abstraction for RP2040
- `logic` - Uses the logic project for controllers

**Features:**
- Relay control (8 relays)
- Digital input reading (4 inputs)
- Board initialization and setup

### Main Application (`main/`)
Example application demonstrating integration between BSP and logic components.

**Dependencies:**
- Both `automation-2040w-bsp` and `logic` projects
- `embassy-executor` with Cortex-M features
- `embassy-rp` for RP2040 support

## Building

```bash
# Test the logic components (uses std executor)
cargo test -p logic

# Check BSP project
cargo check -p automation-2040w-bsp

# Build for embedded target (main application)
cd main && cargo check --target thumbv6m-none-eabi

# Check non-embedded projects
cargo check --workspace --exclude main
```

## Architecture

The workspace follows Embassy async patterns with proper dependency relationships:
- BSP depends on Logic project ✓
- Logic project uses embassy-executor ✓ 
- BSP uses embassy-rp ✓
- Tests execute against executor-std ✓