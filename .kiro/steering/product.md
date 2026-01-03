---
inclusion: always
---

# RustConn Product Context

Linux connection manager for SSH, RDP, VNC, and SPICE protocols. GTK4/libadwaita GUI targeting Wayland-first environments.

## Protocol Architecture

| Protocol | Backend | Session Type |
|----------|---------|--------------|
| SSH | VTE terminal | Embedded tab (primary use case) |
| RDP | FreeRDP (`xfreerdp`) | External window |
| VNC | TigerVNC (`vncviewer`) | External window |
| SPICE | `remote-viewer` | External window |

## Core Features

- Connection organization: groups, tags
- Import/export: Remmina, Asbru-CM, SSH config, Ansible inventory, Royal TS
- Credential backends: libsecret (default), KeePassXC (optional)
- Session logging, command snippets, cluster commands, Wake-on-LAN

## Mandatory Constraints

### Credentials — CRITICAL

```rust
// ALWAYS use SecretString for passwords/keys
use secrecy::SecretString;
let password: SecretString = SecretString::new(value.into());

// ALWAYS persist via SecretBackend trait
impl SecretBackend for MyBackend { ... }
```

NEVER store passwords as plain `String`.

### Crate Boundaries — ENFORCED

| Code Type | Crate | Rule |
|-----------|-------|------|
| Business logic | `rustconn-core` | MUST NOT import `gtk4`, `vte4`, `adw` |
| GUI/widgets | `rustconn` | Depends on `rustconn-core` |
| CLI | `rustconn-cli` | Depends on `rustconn-core` |

### Display Server

- Wayland-first design
- AVOID X11-specific APIs
- Test on Wayland before X11

### Graceful Degradation

Optional features (KeePassXC, tray icon) MUST NOT break core functionality when unavailable. Check feature availability at runtime.

## Extensibility Traits

| Adding | Implement | Location |
|--------|-----------|----------|
| Protocol | `Protocol` | `rustconn-core/src/protocol/` |
| Import format | `ImportSource` | `rustconn-core/src/import/` |
| Export format | `ExportTarget` | `rustconn-core/src/export/` |
| Secret backend | `SecretBackend` | `rustconn-core/src/secret/` |

## UI Implementation (`rustconn/`)

| Pattern | Implementation |
|---------|----------------|
| Widget preference | `adw::` over `gtk::` equivalents |
| Transient messages | `adw::ToastOverlay` |
| Modal dialogs | `adw::Dialog` or `gtk::Window` with `set_modal(true)` |
| Main layout | Sidebar `gtk::TreeView` + `gtk::Notebook` tabs |
| Spacing | 12px margins, 6px between related elements (GNOME HIG) |

## Error Handling

### In `rustconn-core`

```rust
#[derive(Debug, thiserror::Error)]
pub enum FeatureError {
    #[error("description: {0}")]
    Variant(String),
}
```

- Return `Result<T, E>` from all fallible functions
- NO panics; `unwrap()`/`expect()` only for provably impossible states

### In `rustconn` (GUI)

- Display user-friendly toast/dialog for errors
- Log technical details via `tracing`
- Never expose internal error messages to users

## Pre-Implementation Checklist

Before writing code:

1. Crate placement — Business logic → `rustconn-core`; UI → `rustconn`
2. Secrets — Wrapped in `SecretString`, persisted via `SecretBackend`
3. Degradation — Feature works when optional dependencies missing
4. Error handling — All fallible functions return `Result<T, E>`
5. UI patterns — Follows libadwaita patterns and GNOME HIG
