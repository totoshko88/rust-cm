# Requirements Document

## Introduction

This document specifies requirements for a major refactoring of RustConn's protocol handling to achieve native Wayland-first embedding for VNC, RDP, and SPICE protocols. The current implementation relies on external processes (xfreerdp, vncviewer) which open separate windows. The goal is to embed all remote desktop sessions as native GTK4 widgets within the main application window, similar to how SSH sessions are already embedded via VTE4.

Additionally, this refactoring includes cleanup of legacy X11 code and resolution of all clippy warnings to improve code quality.

## Glossary

- **RustConn**: The connection manager application being developed
- **VTE4**: Virtual Terminal Emulator library for GTK4, used for SSH terminal embedding
- **gtk-vnc**: GTK widget library for VNC client functionality
- **gtk-frdp**: GNOME's GTK widget wrapper around FreeRDP library for RDP embedding
- **spice-gtk**: GTK widget library for SPICE protocol client
- **GtkOverlay**: GTK4 widget that allows overlaying widgets on top of other widgets
- **FFI**: Foreign Function Interface for calling C libraries from Rust
- **Wayland**: Modern Linux display server protocol (replacement for X11)
- **X11/XEmbed**: Legacy display server and its window embedding protocol
- **FreeRDP**: Open-source RDP client library
- **SPICE**: Simple Protocol for Independent Computing Environments, used for VM display

## Requirements

### Requirement 1: Remove X11 Legacy Code

**User Story:** As a developer, I want to remove X11-specific embedding code, so that the codebase is cleaner and focused on Wayland-native solutions.

#### Acceptance Criteria

1. WHEN the application starts THEN the RustConn system SHALL NOT check for X11 display server or attempt X11-specific embedding
2. WHEN a user connects via RDP or VNC THEN the RustConn system SHALL use native GTK4 widgets instead of external process windows
3. WHEN the `DisplayServer` enum is used THEN the RustConn system SHALL remove X11 variant and related detection logic
4. WHEN the `embedded.rs` module is refactored THEN the RustConn system SHALL remove all XEmbed-related code and comments

### Requirement 2: Native VNC Embedding

**User Story:** As a user, I want VNC sessions to appear as embedded tabs within RustConn, so that I can manage all my remote sessions in one window.

#### Acceptance Criteria

1. WHEN a user initiates a VNC connection THEN the RustConn system SHALL display the VNC session as an embedded GTK widget
2. WHEN the VNC session is active THEN the RustConn system SHALL forward keyboard and mouse input to the remote host
3. WHEN the VNC server requires authentication THEN the RustConn system SHALL prompt for credentials and handle VNC authentication protocols
4. WHEN the VNC session window is resized THEN the RustConn system SHALL scale or adjust the remote display accordingly
5. WHEN the VNC connection is lost THEN the RustConn system SHALL display an error state and offer reconnection options
6. WHEN multiple VNC sessions are open THEN the RustConn system SHALL display each in a separate tab with proper isolation

### Requirement 3: Native RDP Embedding

**User Story:** As a user, I want RDP sessions to appear as embedded tabs within RustConn, so that I can access Windows machines without separate windows.

#### Acceptance Criteria

1. WHEN a user initiates an RDP connection THEN the RustConn system SHALL display the RDP session as an embedded GTK widget using gtk-frdp
2. WHEN the RDP session is active THEN the RustConn system SHALL render the remote desktop framebuffer to a GdkTexture
3. WHEN the RDP server requires NLA authentication THEN the RustConn system SHALL handle Network Level Authentication properly
4. WHEN RDP gateway is configured THEN the RustConn system SHALL route the connection through the specified gateway
5. WHEN clipboard operations occur THEN the RustConn system SHALL synchronize clipboard content between local and remote systems
6. WHEN audio redirection is enabled THEN the RustConn system SHALL play remote audio through local audio system
7. WHEN the RDP session window is resized THEN the RustConn system SHALL negotiate new resolution with the remote host

### Requirement 4: SPICE Protocol Support

**User Story:** As a user managing virtual machines, I want SPICE protocol support, so that I can connect to libvirt/QEMU VMs with optimal performance.

#### Acceptance Criteria

1. WHEN a user creates a new connection THEN the RustConn system SHALL offer SPICE as a protocol option alongside SSH, RDP, and VNC
2. WHEN a user initiates a SPICE connection THEN the RustConn system SHALL display the VM console as an embedded GTK widget
3. WHEN the SPICE session is active THEN the RustConn system SHALL support USB redirection if configured
4. WHEN the SPICE session is active THEN the RustConn system SHALL support shared folders if configured
5. WHEN the SPICE agent is available THEN the RustConn system SHALL enable clipboard sharing and display auto-resize
6. WHEN the SPICE connection uses TLS THEN the RustConn system SHALL validate certificates according to user preferences

### Requirement 5: Floating Session Controls

**User Story:** As a user, I want floating control buttons over my remote sessions, so that I can quickly disconnect or toggle fullscreen without leaving the session view.

#### Acceptance Criteria

1. WHEN a remote session (VNC/RDP/SPICE) is displayed THEN the RustConn system SHALL overlay floating control buttons using GtkOverlay
2. WHEN the user hovers over the session area THEN the RustConn system SHALL show the floating controls with fade-in animation
3. WHEN the user clicks the disconnect button THEN the RustConn system SHALL terminate the session and show disconnected state
4. WHEN the user clicks the fullscreen button THEN the RustConn system SHALL toggle the session between fullscreen and windowed mode
5. WHEN the user clicks the settings button THEN the RustConn system SHALL display session-specific settings (resolution, quality, etc.)
6. WHEN the floating controls are idle THEN the RustConn system SHALL auto-hide them after a configurable timeout

### Requirement 6: Protocol Configuration Models

**User Story:** As a developer, I want well-defined configuration models for each protocol, so that connection settings are type-safe and validated.

#### Acceptance Criteria

1. WHEN a SPICE connection is configured THEN the RustConn system SHALL store SPICE-specific settings (TLS, USB redirection, shared folders)
2. WHEN protocol configuration is serialized THEN the RustConn system SHALL produce valid TOML/JSON that can be deserialized back
3. WHEN protocol configuration is loaded THEN the RustConn system SHALL validate all fields and report specific validation errors
4. WHEN default values are needed THEN the RustConn system SHALL provide sensible defaults for each protocol type

### Requirement 7: Code Quality Improvements

**User Story:** As a developer, I want all clippy warnings resolved, so that the codebase follows Rust best practices and is maintainable.

#### Acceptance Criteria

1. WHEN `cargo clippy -p rustconn` is run THEN the RustConn system SHALL produce zero warnings
2. WHEN functions exceed 100 lines THEN the RustConn system SHALL refactor them into smaller, focused functions
3. WHEN numeric casts are performed THEN the RustConn system SHALL use safe conversion methods (TryFrom, checked casts)
4. WHEN match arms have identical bodies THEN the RustConn system SHALL consolidate them or use wildcards appropriately
5. WHEN documentation references technical terms THEN the RustConn system SHALL use backticks for proper formatting

### Requirement 8: FFI Integration Architecture

**User Story:** As a developer, I want a clean FFI architecture for C library integration, so that gtk-vnc, gtk-frdp, and spice-gtk can be used safely from Rust.

#### Acceptance Criteria

1. WHEN FFI bindings are created THEN the RustConn system SHALL use safe Rust wrappers around unsafe C calls
2. WHEN GTK widgets from C libraries are used THEN the RustConn system SHALL integrate them properly with GTK4-rs widget hierarchy
3. WHEN memory is allocated by C libraries THEN the RustConn system SHALL ensure proper cleanup through Drop implementations
4. WHEN callbacks are passed to C libraries THEN the RustConn system SHALL handle Rust closure lifetimes correctly
