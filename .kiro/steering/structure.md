---
inclusion: always
---

# RustConn Project Structure

Three-crate Cargo workspace with strict GUI/logic separation.

## Crate Boundaries (CRITICAL)

| Crate | Type | Rule |
|-------|------|------|
| `rustconn/` | Binary | GUI only — imports `gtk4`, `vte4`, `adw` |
| `rustconn-core/` | Library | GUI-free — MUST NOT import `gtk4`, `vte4`, `adw` |
| `rustconn-cli/` | Binary | CLI interface — depends on `rustconn-core` only |

Decision rule: "Does this need GTK?" → No: `rustconn-core` / Yes: `rustconn`

## File Placement

| Adding | Location | Action |
|--------|----------|--------|
| Data model | `rustconn-core/src/models/` | Re-export in `models.rs` |
| Protocol | `rustconn-core/src/protocol/` | Implement `Protocol` trait |
| Import format | `rustconn-core/src/import/` | Implement `ImportSource` trait |
| Export format | `rustconn-core/src/export/` | Implement `ExportTarget` trait |
| Secret backend | `rustconn-core/src/secret/` | Implement `SecretBackend` trait |
| Dialog | `rustconn/src/dialogs/` | Register in `dialogs/mod.rs` |
| Property tests | `rustconn-core/tests/properties/` | Register in `properties/mod.rs` |

## GUI Crate Layout (`rustconn/src/`)

| File(s) | Purpose |
|---------|---------|
| `app.rs` | GTK Application, global actions, keyboard shortcuts |
| `window.rs` | Main window layout, header bar |
| `window_*.rs` | Window functionality by domain (sessions, protocols, dialogs, etc.) |
| `sidebar.rs` + `sidebar_ui.rs` + `sidebar_types.rs` | Connection tree view (logic/widgets/types split) |
| `state.rs` | `SharedAppState` = `Rc<RefCell<AppState>>` |
| `dialogs/` | Modal dialog implementations |
| `embedded_*.rs` | Embedded protocol viewers (RDP, VNC, SPICE) |
| `terminal/` | VTE terminal notebook for SSH |

## Core Library Layout (`rustconn-core/src/`)

| Directory | Purpose |
|-----------|---------|
| `models/` | Connection, Group, Protocol, Snippet, Template structs |
| `config/` | Settings persistence (`manager.rs`, `settings.rs`) |
| `connection/` | Connection CRUD, lazy loading, virtual scroll |
| `protocol/` | Protocol trait + SSH, RDP, VNC, SPICE implementations |
| `import/` | Remmina, Asbru, SSH config, Ansible, Royal TS importers |
| `export/` | Export format implementations |
| `secret/` | Credential backends (libsecret, KeePassXC, KDBX) |
| `session/` | Session state, logging |
| `automation/` | Expect scripts, key sequences |
| `search/` | Connection filtering with caching |

## State Management

GUI state uses single-threaded interior mutability:

```rust
pub type SharedAppState = Rc<RefCell<AppState>>;
```

- Pass `&SharedAppState` to functions needing mutable access
- Manager structs (`ConnectionManager`, `ConfigManager`, etc.) own data and handle I/O
- Async operations use thread-local tokio runtime via `with_runtime()`

## Module Conventions

- Feature directories use `mod.rs` for organization
- Public types re-exported through `lib.rs`
- Large files split by suffix: `*_types.rs`, `*_ui.rs`
- Test modules in `tests/properties/` mirror source structure

## Tests

| Type | Location |
|------|----------|
| Property tests | `rustconn-core/tests/properties/` |
| Integration tests | `rustconn-core/tests/integration/` |
| Fixtures | `rustconn-core/tests/fixtures/` |

Register new property test modules in `tests/properties/mod.rs`.
