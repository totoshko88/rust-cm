---
inclusion: always
---

# RustConn Project Structure

## Workspace Layout

Three-crate Cargo workspace:

| Crate | Type | Purpose |
|-------|------|---------|
| `rustconn/` | Binary | GTK4 GUI application |
| `rustconn-core/` | Library | Business logic, models, protocols (GUI-free) |
| `rustconn-cli/` | Binary | CLI for headless operations |

## Crate Boundaries (STRICT)

- `rustconn-core` MUST NOT import `gtk4`, `vte4`, `adw`, or any GUI crate
- Business logic, data models, protocol implementations → `rustconn-core`
- GUI code (widgets, dialogs, rendering) → `rustconn`
- Both binaries depend on `rustconn-core`

## Key Directories

### rustconn/ (GUI)

| Path | Purpose |
|------|---------|
| `src/app.rs` | GTK Application, actions, shortcuts |
| `src/window.rs` | Main window, header bar |
| `src/sidebar.rs` | Connection tree view |
| `src/terminal.rs` | VTE terminal notebook (SSH) |
| `src/state.rs` | `SharedAppState` = `Rc<RefCell<AppState>>` |
| `src/dialogs/` | Modal dialogs |
| `src/session/` | Protocol session widgets |

### rustconn-core/src/

| Path | Purpose |
|------|---------|
| `lib.rs` | Public API exports |
| `error.rs` | Error types (`thiserror`) |
| `models/` | Connection, Group, Protocol, Snippet, Template |
| `config/` | Settings persistence |
| `connection/` | Connection CRUD |
| `protocol/` | Protocol trait + implementations |
| `import/` | Import formats |
| `export/` | Export formats |
| `secret/` | Credential backends |
| `session/` | Session state, logging |
| `automation/` | Expect scripts, key sequences |
| `cluster/` | Multi-host commands |
| `variables/` | Variable substitution |
| `search/` | Connection filtering |
| `wol/` | Wake-on-LAN |

### rustconn-core/tests/

| Path | Purpose |
|------|---------|
| `properties/` | Property-based tests (`proptest`) |
| `integration/` | Integration tests |
| `fixtures/` | Test data files |

## Extension Traits

| Feature | Trait | Location |
|---------|-------|----------|
| Protocol | `Protocol` | `rustconn-core/src/protocol/` |
| Import format | `ImportSource` | `rustconn-core/src/import/` |
| Export format | `ExportTarget` | `rustconn-core/src/export/` |
| Credential backend | `SecretBackend` | `rustconn-core/src/secret/` |

## State Management

- GUI: `SharedAppState` = `Rc<RefCell<AppState>>` (interior mutability)
- Persistence: Manager structs own data and handle I/O

## Module Conventions

- Feature directories use `mod.rs` for organization
- Public types re-exported through `lib.rs`
- Test modules mirror source structure

## File Placement

| Adding | Location | Steps |
|--------|----------|-------|
| Data model | `rustconn-core/src/models/` | Re-export in `models.rs` |
| Protocol | `rustconn-core/src/protocol/` | Implement `Protocol` trait |
| Import format | `rustconn-core/src/import/` | Implement `ImportSource` trait |
| Export format | `rustconn-core/src/export/` | Implement `ExportTarget` trait |
| Dialog | `rustconn/src/dialogs/` | Register in `mod.rs` |
| Property tests | `rustconn-core/tests/properties/` | Add module to `mod.rs` |
