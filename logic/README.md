# Logic

Core logic components for embassy-based applications.

## Features

- `no_std` compatible for embedded use
- Embassy async task support
- Comprehensive test suite using `executor-std`

## Usage

```rust
use logic::{LogicController, init};

// Initialize logic system
let controller = init();

// Process logic asynchronously  
let result = controller.process().await;

// Spawn background tasks
controller.spawn_background_task(&spawner)?;
```

## Testing

Tests use the standard executor to validate components:

```bash
cargo test
```

All tests run against `executor-std` as specified in the requirements.