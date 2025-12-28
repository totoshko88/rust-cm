---
inclusion: always
---

# RustConn Tech Stack

Rust 2021 edition, MSRV 1.87, three-crate Cargo workspace.

## Crate Overview

| Crate | Purpose | Key Dependencies |
|-------|---------|------------------|
| `rustconn` | GTK4 GUI | `gtk4` 0.10 (`v4_14`), `vte4` 0.9, optional `ksni`+`resvg` |
| `rustconn-core` | Business logic (GUI-free) | `tokio` 1.48, `serde`/`serde_json`/`serde_yaml`/`toml`, `uuid`, `chrono`, `thiserror`, `secrecy`, `ring`+`argon2`, `regex` |
| `rustconn-cli` | CLI interface | `clap` 4.5 (derive) |

## Enforced Code Style

- `unsafe_code = "forbid"` — no unsafe code
- Clippy lints: `all`, `pedantic`, `nursery`
- Line width: 100 chars max
- Indentation: 4 spaces
- Line endings: LF only

## Required Code Patterns

When writing code, use these patterns:

**Errors** — always `thiserror`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("description: {0}")]
    Variant(String),
}
```

**Secrets** — always `SecretString`:
```rust
use secrecy::SecretString;
let password: SecretString = SecretString::new(value.into());
```

**IDs** — UUID v4:
```rust
let id = uuid::Uuid::new_v4();
```

**Timestamps** — chrono UTC:
```rust
let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
```

**Async traits** — `async-trait` macro:
```rust
#[async_trait::async_trait]
impl MyTrait for MyStruct {
    async fn method(&self) -> Result<(), Error> { /* ... */ }
}
```

## Do / Don't Rules

| DO | DON'T |
|----|-------|
| Return `Result<T, Error>` from fallible functions | Use `unwrap()`/`expect()` except for provably impossible states |
| Use `thiserror` for all error types | Define errors without `#[derive(thiserror::Error)]` |
| Wrap credentials in `SecretString` | Store passwords/keys as plain `String` |
| Use `tokio` for all async code | Mix async runtimes |
| Keep `rustconn-core` GUI-free | Import `gtk4`/`vte4`/`adw` in `rustconn-core` |
| Prefer `adw::` widgets over `gtk::` equivalents | Use deprecated GTK patterns |

## Testing

- Property tests: `rustconn-core/tests/properties/` using `proptest`
- Temp files: use `tempfile` crate
- Register new property test modules in `tests/properties/mod.rs`

## Commands

```bash
cargo build                    # Build all crates
cargo build --release          # Release build
cargo run -p rustconn          # Run GUI
cargo run -p rustconn-cli      # Run CLI
cargo test                     # Run all tests
cargo test -p rustconn-core --test property_tests  # Property tests only
cargo clippy --all-targets     # Lint check
cargo fmt --check              # Format check
```
