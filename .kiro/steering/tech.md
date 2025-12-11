# RustConn Tech Stack

## Language & Toolchain

- Rust 2021 edition (MSRV: 1.75)
- Cargo workspace with two crates

## Key Dependencies

### GUI (rustconn crate)
- `gtk4` (0.9) - GTK4 bindings with v4_12 features
- `vte4` (0.8) - Terminal emulator widget for embedded SSH

### Core Library (rustconn-core crate)
- `tokio` - Async runtime (full features)
- `serde` / `serde_json` / `serde_yaml` / `toml` - Serialization
- `uuid` - Connection/group identifiers
- `chrono` - Timestamps
- `thiserror` - Error definitions
- `async-trait` - Async trait support
- `secrecy` - Secure string handling
- `regex` - Pattern matching for imports

### Testing
- `proptest` - Property-based testing
- `tempfile` - Temporary file handling in tests

## Linting & Formatting

Strict linting enabled at workspace level:
- `unsafe_code = "forbid"` - No unsafe code allowed
- Clippy: `all`, `pedantic`, `nursery` warnings enabled

Clippy thresholds (`.clippy.toml`):
- Cognitive complexity: 25
- Max arguments: 7
- Type complexity: 250

Formatting (`rustfmt.toml`):
- Max width: 100 characters
- 4-space indentation
- Unix line endings
- Imports and modules reordered

## Common Commands

```bash
# Build all crates
cargo build

# Build release
cargo build --release

# Run the application
cargo run -p rustconn

# Run all tests
cargo test

# Run property tests only
cargo test -p rustconn-core --test property_tests

# Check lints
cargo clippy --all-targets

# Format code
cargo fmt

# Check formatting
cargo fmt --check
```
