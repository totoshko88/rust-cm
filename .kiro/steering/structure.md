# RustConn Project Structure

## Workspace Layout

```
rustconn/                    # Cargo workspace root
├── rustconn/                # GUI application crate (binary)
│   └── src/
│       ├── main.rs          # Entry point
│       ├── app.rs           # GTK Application setup, actions, shortcuts
│       ├── window.rs        # Main window, header bar, layout
│       ├── sidebar.rs       # Connection list/tree sidebar
│       ├── terminal.rs      # Terminal notebook for SSH sessions
│       ├── state.rs         # Application state management
│       └── dialogs/         # Modal dialogs
│           ├── connection.rs  # New/edit connection
│           ├── import.rs      # Import wizard
│           ├── settings.rs    # App settings
│           └── snippet.rs     # Snippet editor
│
└── rustconn-core/           # Core library crate
    ├── src/
    │   ├── lib.rs           # Public API exports
    │   ├── error.rs         # Error types (thiserror)
    │   ├── models.rs        # Re-exports from models/
    │   ├── models/          # Data structures
    │   │   ├── connection.rs  # Connection model
    │   │   ├── credentials.rs # Credential types
    │   │   ├── group.rs       # Connection groups
    │   │   ├── protocol.rs    # Protocol configs (SSH/RDP/VNC)
    │   │   └── snippet.rs     # Command snippets
    │   ├── config/          # Configuration management
    │   │   ├── manager.rs     # Config file I/O
    │   │   └── settings.rs    # Settings structures
    │   ├── connection/      # Connection management
    │   │   └── manager.rs     # CRUD operations
    │   ├── protocol/        # Protocol implementations
    │   │   ├── mod.rs         # Protocol trait
    │   │   ├── registry.rs    # Protocol registry
    │   │   ├── ssh.rs         # SSH handler
    │   │   ├── rdp.rs         # RDP handler
    │   │   └── vnc.rs         # VNC handler
    │   ├── import/          # Import sources
    │   │   ├── traits.rs      # ImportSource trait
    │   │   ├── ssh_config.rs  # ~/.ssh/config
    │   │   ├── remmina.rs     # Remmina profiles
    │   │   ├── asbru.rs       # Asbru-CM
    │   │   └── ansible.rs     # Ansible inventory
    │   ├── secret/          # Credential storage
    │   │   ├── backend.rs     # SecretBackend trait
    │   │   ├── manager.rs     # Secret manager
    │   │   ├── libsecret.rs   # GNOME Keyring
    │   │   ├── keepassxc.rs   # KeePassXC integration
    │   │   └── kdbx.rs        # KDBX file export
    │   ├── session/         # Session management
    │   │   ├── manager.rs     # Active sessions
    │   │   ├── session.rs     # Session state
    │   │   └── logger.rs      # Session logging
    │   └── snippet/         # Snippet management
    │       └── manager.rs     # Snippet CRUD
    └── tests/
        └── properties/      # Property-based tests
```

## Architecture Patterns

### Separation of Concerns
- `rustconn-core`: Pure Rust, no GUI dependencies, all business logic
- `rustconn`: GTK4 GUI, depends on core, handles presentation

### Trait-Based Extensibility
- `Protocol` trait: Add new protocols by implementing trait
- `ImportSource` trait: Add new import formats
- `SecretBackend` trait: Add new credential storage backends

### Error Handling
- Domain-specific error types in `error.rs`
- Type aliases for Result types (e.g., `ConfigResult<T>`, `ProtocolResult<T>`)
- Uses `thiserror` for derive macros

### State Management
- `SharedAppState` (Rc<RefCell<AppState>>) for GUI state
- Managers handle persistence (ConfigManager, ConnectionManager, etc.)
