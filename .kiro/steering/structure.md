---
inclusion: always
---

# RustConn Project Structure

Three-crate Cargo workspace with strict separation between GUI and business logic.

## Crate Boundaries (ENFORCED)

| Crate | Type | Depends On | Rule |
|-------|------|------------|------|
| `rustconn/` | Binary | `rustconn-core` | GUI only — widgets, dialogs, rendering |
| `rustconn-core/` | Library | — | GUI-free — MUST NOT import `gtk4`, `vte4`, `adw` |
| `rustconn-cli/` | Binary | `rustconn-core` | CLI for headless operations |

When adding code, ask: "Does this need GTK?" If no → `rustconn-core`. If yes → `rustconn`.

## File Placement Rules

| Adding | Location | Required Steps |
|--------|----------|----------------|
| Data model | `rustconn-core/src/models/` | Re-export in `models.rs` |
| Protocol | `rustconn-core/src/protocol/` | Implement `Protocol` trait |
| Import format | `rustconn-core/src/import/` | Implement `ImportSource` trait |
| Export format | `rustconn-core/src/export/` | Implement `ExportTarget` trait |
| Secret backend | `rustconn-core/src/secret/` | Implement `SecretBackend` trait |
| Dialog | `rustconn/src/dialogs/` | Register in `mod.rs` |
| Property tests | `rustconn-core/tests/properties/` | Add module to `mod.rs` |

## Key Files (rustconn/)

| File | Responsibility |
|------|----------------|
| `app.rs` | GTK Application, global actions, keyboard shortcuts |
| `window.rs` | Main window layout, header bar |
| `window_*.rs` | Window functionality split by domain (sessions, protocols, dialogs) |
| `sidebar.rs` | Connection tree view logic |
| `sidebar_ui.rs` | Sidebar widget construction |
| `sidebar_types.rs` | Sidebar data types |
| `terminal.rs` | VTE terminal notebook for SSH |
| `state.rs` | `SharedAppState` = `Rc<RefCell<AppState>>` |
| `dialogs/` | Modal dialog implementations |
| `embedded_*.rs` | Embedded protocol viewers (RDP, VNC, SPICE) |

## Key Directories (rustconn-core/src/)

| Directory | Purpose |
|-----------|---------|
| `models/` | Connection, Group, Protocol, Snippet, Template structs |
| `config/` | Settings persistence (`manager.rs`, `settings.rs`) |
| `connection/` | Connection CRUD, lazy loading, virtual scroll |
| `protocol/` | Protocol trait + SSH, RDP, VNC, SPICE implementations |
| `import/` | Remmina, Asbru, SSH config, Ansible importers |
| `export/` | Export format implementations |
| `secret/` | Credential backends (libsecret, KeePassXC, KDBX) |
| `session/` | Session state, logging |
| `automation/` | Expect scripts, key sequences |
| `search/` | Connection filtering with caching |

## Extension Traits

Implement these traits to extend functionality:

```rust
// New protocol
impl Protocol for MyProtocol { ... }  // in rustconn-core/src/protocol/

// New import format  
impl ImportSource for MyImporter { ... }  // in rustconn-core/src/import/

// New export format
impl ExportTarget for MyExporter { ... }  // in rustconn-core/src/export/

// New credential backend
impl SecretBackend for MyBackend { ... }  // in rustconn-core/src/secret/
```

## State Management

- GUI state: `SharedAppState` = `Rc<RefCell<AppState>>` (single-threaded interior mutability)
- Persistence: Manager structs (`ConnectionManager`, `ConfigManager`, etc.) own data and handle I/O
- Pass `&SharedAppState` to functions needing mutable access

## Module Conventions

- Feature directories use `mod.rs` for organization
- Public types re-exported through `lib.rs`
- Large files split by suffix: `*_types.rs`, `*_ui.rs` (e.g., `sidebar.rs` → `sidebar_types.rs`, `sidebar_ui.rs`)
- Test modules in `tests/properties/` mirror source structure

## Tests Location

| Test Type | Location |
|-----------|----------|
| Property tests | `rustconn-core/tests/properties/` |
| Integration tests | `rustconn-core/tests/integration/` |
| Test fixtures | `rustconn-core/tests/fixtures/` |
