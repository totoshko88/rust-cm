# Design Document: Native Protocol Embedding

## Overview

This design document describes the architecture for native Wayland-first protocol embedding in RustConn. The implementation replaces external process-based connections (xfreerdp, vncviewer) with native GTK4 widgets using gtk-vnc, gtk-frdp (FreeRDP), and spice-gtk libraries through FFI bindings.

The design follows a layered architecture:
1. **FFI Layer**: Safe Rust wrappers around C libraries
2. **Widget Layer**: GTK4-compatible session widgets
3. **Session Layer**: Connection lifecycle management
4. **UI Layer**: Floating controls and tab integration

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        RustConn GUI                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    TerminalNotebook                      │    │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │    │
│  │  │SSH Tab  │  │VNC Tab  │  │RDP Tab  │  │SPICE Tab│    │    │
│  │  │(VTE4)   │  │(gtk-vnc)│  │(gtk-frdp)│ │(spice)  │    │    │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│  ┌───────────────────────────┴───────────────────────────────┐  │
│  │                    SessionOverlay                          │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │ GtkOverlay                                          │  │  │
│  │  │  ┌─────────────────────────────────────────────┐   │  │  │
│  │  │  │ Protocol Widget (VncDisplay/RdpWidget/Spice)│   │  │  │
│  │  │  └─────────────────────────────────────────────┘   │  │  │
│  │  │  ┌─────────────────────────────────────────────┐   │  │  │
│  │  │  │ FloatingControls (disconnect, fullscreen)   │   │  │  │
│  │  │  └─────────────────────────────────────────────┘   │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      rustconn-core                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ProtocolConfig│ │SessionManager│ │ProtocolWidgetFactory    │  │
│  │(SSH/RDP/VNC/ │ │              │ │                         │  │
│  │ SPICE)       │ │              │ │ create_vnc_widget()     │  │
│  └─────────────┘  └─────────────┘  │ create_rdp_widget()     │  │
│                                     │ create_spice_widget()   │  │
│                                     └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      FFI Bindings Layer                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ vnc-sys     │  │ frdp-sys    │  │ spice-sys               │  │
│  │ (gtk-vnc)   │  │ (gtk-frdp)  │  │ (spice-gtk)             │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. FFI Bindings Crates

#### vnc-sys (gtk-vnc bindings)

```rust
// rustconn-core/src/ffi/vnc.rs

/// Safe wrapper around GVncDisplay widget
pub struct VncDisplay {
    inner: *mut ffi::GtkVncDisplay,
}

impl VncDisplay {
    /// Creates a new VNC display widget
    pub fn new() -> Self;
    
    /// Opens connection to VNC server
    pub fn open_host(&self, host: &str, port: u16) -> Result<(), VncError>;
    
    /// Closes the connection
    pub fn close(&self);
    
    /// Returns whether connected
    pub fn is_open(&self) -> bool;
    
    /// Sets credential for authentication
    pub fn set_credential(&self, cred_type: VncCredentialType, value: &str);
    
    /// Enables/disables scaling
    pub fn set_scaling(&self, enabled: bool);
    
    /// Gets the underlying GTK widget
    pub fn widget(&self) -> &gtk4::Widget;
}

// Signal connections
impl VncDisplay {
    pub fn connect_vnc_connected<F: Fn(&Self) + 'static>(&self, f: F);
    pub fn connect_vnc_disconnected<F: Fn(&Self) + 'static>(&self, f: F);
    pub fn connect_vnc_auth_credential<F: Fn(&Self, &[VncCredentialType]) + 'static>(&self, f: F);
    pub fn connect_vnc_auth_failure<F: Fn(&Self, &str) + 'static>(&self, f: F);
}
```

#### frdp-sys (gtk-frdp bindings)

```rust
// rustconn-core/src/ffi/rdp.rs

/// Safe wrapper around FrdpDisplay widget (from GNOME Connections)
pub struct RdpDisplay {
    inner: *mut ffi::FrdpDisplay,
}

impl RdpDisplay {
    /// Creates a new RDP display widget
    pub fn new() -> Self;
    
    /// Opens connection to RDP server
    pub fn open(&self, config: &RdpConnectionConfig) -> Result<(), RdpError>;
    
    /// Closes the connection
    pub fn close(&self);
    
    /// Returns connection state
    pub fn state(&self) -> RdpConnectionState;
    
    /// Sets credentials
    pub fn set_credentials(&self, username: &str, password: &str, domain: Option<&str>);
    
    /// Enables clipboard sharing
    pub fn set_clipboard_enabled(&self, enabled: bool);
    
    /// Gets the underlying GTK widget
    pub fn widget(&self) -> &gtk4::Widget;
}

pub struct RdpConnectionConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub resolution: Option<Resolution>,
    pub gateway: Option<RdpGatewayConfig>,
}
```

#### spice-sys (spice-gtk bindings)

```rust
// rustconn-core/src/ffi/spice.rs

/// Safe wrapper around SpiceDisplay widget
pub struct SpiceDisplay {
    session: *mut ffi::SpiceSession,
    display: *mut ffi::SpiceDisplay,
}

impl SpiceDisplay {
    /// Creates a new SPICE display widget
    pub fn new() -> Self;
    
    /// Opens connection to SPICE server
    pub fn open(&self, uri: &str) -> Result<(), SpiceError>;
    
    /// Closes the connection
    pub fn close(&self);
    
    /// Returns connection state
    pub fn is_connected(&self) -> bool;
    
    /// Enables USB redirection
    pub fn set_usb_redirection(&self, enabled: bool);
    
    /// Enables shared folders
    pub fn add_shared_folder(&self, path: &Path, name: &str);
    
    /// Gets the underlying GTK widget
    pub fn widget(&self) -> &gtk4::Widget;
}
```

### 2. Session Widget Layer

```rust
// rustconn/src/session_widget.rs

/// Unified session widget that wraps protocol-specific displays
pub enum SessionWidget {
    Ssh(vte4::Terminal),
    Vnc(VncSessionWidget),
    Rdp(RdpSessionWidget),
    Spice(SpiceSessionWidget),
}

/// VNC session widget with overlay controls
pub struct VncSessionWidget {
    overlay: gtk4::Overlay,
    display: VncDisplay,
    controls: FloatingControls,
    state: Rc<RefCell<SessionState>>,
}

impl VncSessionWidget {
    pub fn new() -> Self;
    pub fn connect(&self, host: &str, port: u16, credentials: Option<&Credentials>);
    pub fn disconnect(&self);
    pub fn widget(&self) -> &gtk4::Widget;
    pub fn state(&self) -> SessionState;
}

/// Session state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
    Error(SessionError),
}
```

### 3. Floating Controls Component

```rust
// rustconn/src/floating_controls.rs

/// Floating control bar for remote sessions
pub struct FloatingControls {
    container: gtk4::Box,
    disconnect_btn: gtk4::Button,
    fullscreen_btn: gtk4::Button,
    settings_btn: gtk4::Button,
    revealer: gtk4::Revealer,
    auto_hide_timeout: Rc<RefCell<Option<glib::SourceId>>>,
}

impl FloatingControls {
    pub fn new() -> Self;
    
    /// Shows controls with animation
    pub fn show(&self);
    
    /// Hides controls with animation
    pub fn hide(&self);
    
    /// Sets auto-hide timeout in milliseconds
    pub fn set_auto_hide_timeout(&self, ms: u32);
    
    /// Connects disconnect button callback
    pub fn connect_disconnect<F: Fn() + 'static>(&self, f: F);
    
    /// Connects fullscreen button callback
    pub fn connect_fullscreen<F: Fn() + 'static>(&self, f: F);
    
    /// Connects settings button callback
    pub fn connect_settings<F: Fn() + 'static>(&self, f: F);
    
    /// Updates fullscreen button icon
    pub fn set_fullscreen_active(&self, active: bool);
    
    /// Returns the widget for overlay
    pub fn widget(&self) -> &gtk4::Widget;
}
```

### 4. Protocol Configuration Extension

```rust
// rustconn-core/src/models/protocol.rs (additions)

/// Protocol type identifier (extended)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    Ssh,
    Rdp,
    Vnc,
    Spice,  // NEW
}

/// Protocol-specific configuration (extended)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolConfig {
    Ssh(SshConfig),
    Rdp(RdpConfig),
    Vnc(VncConfig),
    Spice(SpiceConfig),  // NEW
}

/// SPICE protocol configuration
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpiceConfig {
    /// Enable TLS encryption
    #[serde(default)]
    pub tls_enabled: bool,
    
    /// CA certificate path for TLS verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert_path: Option<PathBuf>,
    
    /// Skip certificate verification (insecure)
    #[serde(default)]
    pub skip_cert_verify: bool,
    
    /// Enable USB redirection
    #[serde(default)]
    pub usb_redirection: bool,
    
    /// Shared folders
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_folders: Vec<SharedFolder>,
    
    /// Enable clipboard sharing
    #[serde(default = "default_true")]
    pub clipboard_enabled: bool,
    
    /// Preferred image compression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_compression: Option<SpiceImageCompression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpiceImageCompression {
    Auto,
    Off,
    Glz,
    Lz,
    Quic,
}

fn default_true() -> bool { true }
```

## Data Models

### Session State Model

```rust
/// Active session information
pub struct ActiveSession {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub protocol: ProtocolType,
    pub state: SessionState,
    pub started_at: DateTime<Utc>,
    pub widget: SessionWidget,
}

/// Session error types
#[derive(Debug, Clone)]
pub enum SessionError {
    ConnectionFailed(String),
    AuthenticationFailed(String),
    Disconnected(String),
    ProtocolError(String),
}
```

### VNC Configuration Model (Updated)

```rust
/// VNC protocol configuration (updated for native embedding)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VncConfig {
    /// Preferred encoding (tight, zrle, hextile, raw)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    
    /// Enable lossy compression for better performance
    #[serde(default)]
    pub lossy_compression: bool,
    
    /// Quality level for lossy compression (0-9)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
    
    /// Enable local cursor rendering
    #[serde(default = "default_true")]
    pub local_cursor: bool,
    
    /// Enable scaling to fit window
    #[serde(default)]
    pub scaling: bool,
    
    /// Enable clipboard sharing
    #[serde(default = "default_true")]
    pub clipboard_enabled: bool,
    
    /// Read-only mode (view only)
    #[serde(default)]
    pub view_only: bool,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Protocol widget creation returns valid GTK widget

*For any* valid protocol configuration (VNC, RDP, or SPICE), creating a session widget SHALL return a widget that implements `IsA<gtk4::Widget>` and can be added to a GTK container.

**Validates: Requirements 1.2, 2.1, 3.1, 4.2**

### Property 2: Session state transitions are valid

*For any* session, state transitions SHALL only follow valid paths: Disconnected → Connecting → (Authenticating →)? Connected, and any state → Disconnected or Error.

**Validates: Requirements 2.5, 5.3**

### Property 3: Multiple sessions maintain isolation

*For any* set of N sessions created, each session SHALL have a unique ID, and operations on one session SHALL NOT affect the state of other sessions.

**Validates: Requirements 2.6**

### Property 4: Protocol configuration round-trip serialization

*For any* valid ProtocolConfig (including SpiceConfig), serializing to TOML/JSON and deserializing back SHALL produce an equivalent configuration.

**Validates: Requirements 6.2**

### Property 5: Protocol configuration validation rejects invalid inputs

*For any* protocol configuration with invalid field values (e.g., quality > 9, empty host), validation SHALL return a specific error describing the invalid field.

**Validates: Requirements 6.3**

### Property 6: Default configurations are valid

*For any* protocol type, `Default::default()` SHALL produce a configuration that passes validation.

**Validates: Requirements 6.4**

### Property 7: Fullscreen state toggle is idempotent pair

*For any* session, toggling fullscreen twice SHALL return to the original state.

**Validates: Requirements 5.4**

### Property 8: Session overlay contains required controls

*For any* remote session widget (VNC/RDP/SPICE), the overlay SHALL contain disconnect, fullscreen, and settings buttons.

**Validates: Requirements 5.1**

### Property 9: TLS certificate validation respects configuration

*For any* SPICE connection with TLS enabled, if `skip_cert_verify` is false, invalid certificates SHALL cause connection failure; if true, connection SHALL proceed.

**Validates: Requirements 4.6**

### Property 10: FFI widgets integrate with GTK4 hierarchy

*For any* FFI-wrapped widget (VncDisplay, RdpDisplay, SpiceDisplay), the widget SHALL be usable as a child of standard GTK4 containers (Box, Overlay, Notebook).

**Validates: Requirements 8.2**

## Code Removal Plan

### Files to Remove

```
rustconn/src/embedded.rs          # Entire file - X11/external process logic
```

### Code to Remove from Existing Files

#### rustconn-core/src/protocol/rdp.rs
- Remove `find_freerdp_binary()` function
- Remove `build_command()` implementation (external process)
- Remove `RdpClient` enum (FreeRdp, Custom variants)
- Keep validation logic, adapt for native embedding

#### rustconn-core/src/protocol/vnc.rs
- Remove `get_client_binary()` function
- Remove `build_command()` implementation (external process)
- Remove `VncClient` enum (TightVnc, TigerVnc, Custom variants)
- Keep validation logic, adapt for native embedding

#### rustconn/src/terminal.rs
- Remove `create_external_tab()` method
- Remove `create_embedded_tab()` and `create_embedded_tab_with_widget()` methods
- Remove references to `EmbeddedSessionTab`
- Simplify to only handle SSH (VTE4) sessions

### Structs/Enums to Remove

```rust
// From embedded.rs - entire module removed
pub enum DisplayServer { X11, Wayland, Unknown }
pub enum EmbeddingError { ... }
pub struct SessionControls { ... }
pub struct EmbeddedSessionTab { ... }
pub struct RdpLauncher { ... }
pub struct VncLauncher { ... }
pub mod helpers { ... }

// From protocol models
pub enum RdpClient { FreeRdp, Custom(PathBuf) }
pub enum VncClient { TightVnc, TigerVnc, Custom(PathBuf) }
```

### Dependencies to Remove from Cargo.toml

No external dependencies to remove - the current implementation uses `std::process::Command` which is part of std.

### New Dependencies to Add

```toml
# rustconn/Cargo.toml
[dependencies]
# FFI bindings (to be created or found)
# gtk-vnc-sys = "0.1"  # Or custom bindings
# frdp-sys = "0.1"     # Custom bindings from GNOME Connections
# spice-gtk-sys = "0.1" # Or custom bindings

[build-dependencies]
# For generating FFI bindings
# gir = "0.19"  # If using gir for binding generation
```

### Protocol Trait Changes

```rust
// rustconn-core/src/protocol/mod.rs

pub trait Protocol {
    fn protocol_id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn default_port(&self) -> u16;
    
    // REMOVE: fn uses_embedded_terminal(&self) -> bool;
    // REMOVE: fn build_command(...) -> ProtocolResult<Command>;
    
    // ADD: Native widget creation
    fn create_session_widget(&self, config: &ProtocolConfig) -> Result<SessionWidget, SessionError>;
    
    fn validate_connection(&self, connection: &Connection) -> ProtocolResult<()>;
}
```

## Error Handling

### Error Types

```rust
// rustconn-core/src/error.rs (additions)

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Session disconnected: {0}")]
    Disconnected(String),
    
    #[error("FFI error: {0}")]
    FfiError(String),
    
    #[error("Widget creation failed: {0}")]
    WidgetCreationFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum VncError {
    #[error("VNC connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("VNC authentication failed")]
    AuthenticationFailed,
    
    #[error("VNC server closed connection")]
    ServerDisconnected,
}

#[derive(Debug, thiserror::Error)]
pub enum RdpError {
    #[error("RDP connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("RDP NLA authentication failed")]
    NlaAuthenticationFailed,
    
    #[error("RDP gateway error: {0}")]
    GatewayError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SpiceError {
    #[error("SPICE connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("SPICE TLS certificate validation failed")]
    CertificateValidationFailed,
    
    #[error("SPICE channel error: {0}")]
    ChannelError(String),
}
```

### Error Recovery Strategy

1. **Connection Failures**: Display error in session tab with "Retry" button
2. **Authentication Failures**: Re-prompt for credentials with error message
3. **Disconnections**: Show disconnected state with "Reconnect" option
4. **FFI Errors**: Log detailed error, show user-friendly message

## Testing Strategy

### Dual Testing Approach

This implementation uses both unit tests and property-based tests:

- **Unit tests**: Verify specific examples, edge cases, and error conditions
- **Property-based tests**: Verify universal properties across all valid inputs

### Property-Based Testing Framework

- **Library**: `proptest` (already in workspace dependencies)
- **Minimum iterations**: 100 per property
- **Test annotation format**: `// **Feature: native-protocol-embedding, Property N: description**`

### Unit Test Categories

1. **Configuration Tests**
   - Serialization/deserialization of all protocol configs
   - Validation of field constraints
   - Default value correctness

2. **State Machine Tests**
   - Valid state transitions
   - Invalid transition rejection
   - State persistence across operations

3. **Widget Integration Tests**
   - Widget creation for each protocol
   - Widget hierarchy integration
   - Signal connection verification

### Property Test Categories

1. **Round-trip Properties**
   - Config serialization round-trip (Property 4)
   - State toggle idempotence (Property 7)

2. **Invariant Properties**
   - Session isolation (Property 3)
   - Overlay control presence (Property 8)
   - Default validity (Property 6)

3. **Validation Properties**
   - Invalid input rejection (Property 5)
   - Certificate validation (Property 9)

### Test File Structure

```
rustconn-core/tests/
├── properties/
│   ├── protocol_config_tests.rs  # Properties 4, 5, 6
│   ├── session_state_tests.rs    # Properties 2, 3, 7
│   └── widget_tests.rs           # Properties 1, 8, 10
└── property_tests.rs             # Test runner
```
