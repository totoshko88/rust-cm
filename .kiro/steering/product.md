---
inclusion: always
---

# RustConn Product Context

Linux connection manager for SSH, RDP, VNC, and SPICE protocols. GTK4/libadwaita GUI targeting Wayland-first environments.

## Protocol Overview

| Protocol | Backend | Session Type |
|----------|---------|--------------|
| SSH | VTE terminal | Embedded tab |
| RDP | FreeRDP (`xfreerdp`) | External window |
| VNC | TigerVNC (`vncviewer`) | External window |
| SPICE | `remote-viewer` | External window |

## Core Features

- Connection organization: groups and tags
- Import/export: Remmina, Asbru-CM, SSH config, Ansible inventory
- Credential backends: libsecret, KeePassXC
- Session logging, command snippets, cluster commands, Wake-on-LAN

## Mandatory Constraints

Apply these rules to ALL code changes:

| Constraint | Requirement |
|------------|-------------|
| Credentials | MUST wrap in `secrecy::SecretString`; persist via `SecretBackend` trait only |
| Display server | Wayland-first; AVOID X11-specific APIs |
| Crate boundary | `rustconn-core` MUST NOT import `gtk4`, `vte4`, or `adw` |
| Extensibility | New protocols → `Protocol` trait; formats → `ImportSource`/`ExportTarget`; secrets → `SecretBackend` |
| Graceful degradation | Optional features (KeePassXC, tray) MUST NOT break core when unavailable |

## UI Implementation Rules

When writing GUI code in `rustconn/`:

- PREFER `adw::` widgets over `gtk::` equivalents
- Transient messages → `adw::ToastOverlay`
- Modal dialogs → `adw::Dialog` or `gtk::Window` with `set_modal(true)`
- Layout → sidebar `gtk::TreeView` + `gtk::Notebook` session tabs
- Spacing → 12px margins, 6px between related elements (GNOME HIG)

## Error Handling Pattern

Define errors in `rustconn-core` using `thiserror`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum FeatureError {
    #[error("description: {0}")]
    Variant(String),
}
```

Error handling by layer:
- Library (`rustconn-core`): Return `Result<T, E>`; NO panics; `unwrap()`/`expect()` only for impossible states
- GUI (`rustconn`): Display user-friendly toast or dialog; log technical details via `tracing`

## Pre-Implementation Checklist

Before writing code, verify:

1. Crate placement correct? Business logic → `rustconn-core`; UI → `rustconn`
2. Secrets wrapped in `SecretString` and persisted via `SecretBackend`?
3. Feature degrades gracefully when dependencies missing?
4. All fallible functions return `Result<T, E>`?
5. UI follows libadwaita patterns and GNOME HIG?
