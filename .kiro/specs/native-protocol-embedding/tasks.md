# Implementation Plan

## Phase 1: Code Cleanup and Preparation

- [x] 1. Fix clippy warnings in rustconn crate
  - [x] 1.1 Fix `too_many_lines` warnings by refactoring large functions
    - Refactor `ConnectionDialog::new()` (235 lines) into smaller methods
    - Refactor `create_basic_tab()`, `create_rdp_options()` into components
    - Refactor `SettingsDialog::create_secrets_tab()` (157 lines)
    - Refactor `MainWindow::setup_actions()` (334 lines)
    - Refactor `MainWindow::connect_signals()`, `start_connection()`, `delete_selected_connections()`
    - Refactor `show_quick_connect_dialog()`, `show_snippets_manager()`, `show_sessions_manager()`
    - _Requirements: 7.2_

  - [x] 1.2 Fix numeric cast warnings with safe conversions
    - Replace `as u16`, `as u32`, `as i32` with `TryFrom` or checked casts
    - Add proper error handling for conversion failures
    - Files: `import.rs` (cast_precision_loss), `progress.rs` (cast_precision_loss), `settings.rs` (cast_possible_truncation, cast_sign_loss)
    - _Requirements: 7.3_

  - [x] 1.3 Fix `match_same_arms` warnings
    - Consolidate identical match arms in `rustconn-core/src/models/connection.rs` (VNC/SPICE default ports)
    - _Requirements: 7.4_

  - [x] 1.4 Fix documentation and style warnings
    - Fix `assigning_clones` warning in `import.rs` (use `clone_from()`)
    - Fix `needless_pass_by_value` warnings in `window.rs` (use references for `PathBuf`, `SharedAppState`)
    - Fix `map_or` suggestion in `window.rs` for session logger
    - Allow `struct_excessive_bools` for `SpiceConfig` (4 bools is acceptable for config struct)
    - _Requirements: 7.5_

- [x] 2. Checkpoint - Ensure all clippy warnings are resolved
  - Ensure all tests pass, ask the user if questions arise.

## Phase 2: Remove Legacy X11/External Process Code

- [x] 3. Remove X11 and external process code
  - [x] 3.1 Delete `rustconn/src/embedded.rs` file
    - Remove entire file containing `DisplayServer`, `EmbeddingError`, `SessionControls`, `EmbeddedSessionTab`, `RdpLauncher`, `VncLauncher`
    - _Requirements: 1.1, 1.3, 1.4_

  - [x] 3.2 Clean up `rustconn/src/terminal.rs`
    - Remove `create_external_tab()` method
    - Remove `create_embedded_tab()` method
    - Remove `create_embedded_tab_with_widget()` method
    - Remove import of `crate::embedded::EmbeddedSessionTab`
    - Keep SSH terminal functionality intact
    - _Requirements: 1.2_

  - [x] 3.3 Update `rustconn-core/src/protocol/rdp.rs`
    - Remove `find_freerdp_binary()` function
    - Remove `get_client_binary()` function
    - Remove `build_command()` implementation
    - Keep `validate_connection()` logic
    - _Requirements: 1.2_

  - [x] 3.4 Update `rustconn-core/src/protocol/vnc.rs`
    - Remove `get_client_binary()` function
    - Remove `is_tigervnc()` function
    - Remove `build_command()` implementation
    - Keep `validate_connection()` logic
    - _Requirements: 1.2_

  - [x] 3.5 Update `rustconn-core/src/models/protocol.rs`
    - Remove `RdpClient` enum (FreeRdp, Custom variants)
    - Remove `VncClient` enum (TightVnc, TigerVnc, Custom variants)
    - Remove `client` field from `RdpConfig` and `VncConfig`
    - _Requirements: 1.2_

  - [x] 3.6 Update Protocol trait in `rustconn-core/src/protocol/mod.rs`
    - Remove `uses_embedded_terminal()` method from trait
    - Remove `build_command()` method from trait
    - Add placeholder `create_session_widget()` method signature
    - _Requirements: 1.2_

- [x] 4. Checkpoint - Verify code compiles after removal
  - Ensure all tests pass, ask the user if questions arise.

## Phase 3: Add SPICE Protocol Support

- [x] 5. Add SPICE protocol configuration model
  - [x] 5.1 Add `SpiceConfig` struct to `rustconn-core/src/models/protocol.rs`
    - Add fields: `tls_enabled`, `ca_cert_path`, `skip_cert_verify`, `usb_redirection`, `shared_folders`, `clipboard_enabled`, `image_compression`
    - Add `SpiceImageCompression` enum
    - _Requirements: 6.1_

  - [x] 5.2 Extend `ProtocolType` and `ProtocolConfig` enums
    - Add `Spice` variant to `ProtocolType`
    - Add `Spice(SpiceConfig)` variant to `ProtocolConfig`
    - Update `protocol_type()` method
    - _Requirements: 4.1_

  - [x] 5.3 Write property test for SPICE config serialization round-trip
    - **Property 4: Protocol configuration round-trip serialization**
    - **Validates: Requirements 6.2**

  - [x] 5.4 Write property test for SPICE config validation
    - **Property 5: Protocol configuration validation rejects invalid inputs**
    - **Validates: Requirements 6.3**

  - [x] 5.5 Write property test for default SPICE config validity
    - **Property 6: Default configurations are valid**
    - **Validates: Requirements 6.4**

- [x] 6. Add SPICE protocol handler
  - [x] 6.1 Create `rustconn-core/src/protocol/spice.rs`
    - Implement `SpiceProtocol` struct
    - Implement `Protocol` trait for SPICE
    - Add validation for SPICE-specific fields
    - _Requirements: 4.1, 6.3_

  - [x] 6.2 Register SPICE protocol in registry
    - Update `rustconn-core/src/protocol/registry.rs`
    - Add SPICE to available protocols
    - _Requirements: 4.1_

- [x] 7. Checkpoint - Verify SPICE model and protocol work
  - Ensure all tests pass, ask the user if questions arise.

## Phase 4: FFI Bindings Foundation

- [x] 8. Create FFI bindings infrastructure
  - [x] 8.1 Create `rustconn-core/src/ffi/mod.rs` module
    - Set up module structure for FFI bindings
    - Add common FFI utilities and error types
    - _Requirements: 8.1_

  - [x] 8.2 Create VNC FFI bindings (`rustconn-core/src/ffi/vnc.rs`)
    - Create safe wrapper `VncDisplay` struct
    - Implement `new()`, `open_host()`, `close()`, `is_open()`
    - Implement `set_credential()`, `set_scaling()`
    - Implement signal connections for vnc-connected, vnc-disconnected, vnc-auth-credential
    - Implement `widget()` to return GTK widget
    - _Requirements: 2.1, 8.1, 8.2_

  - [x] 8.3 Write property test for VNC widget GTK integration
    - **Property 10: FFI widgets integrate with GTK4 hierarchy**
    - **Validates: Requirements 8.2**

- [x] 9. Checkpoint - Verify VNC FFI bindings compile
  - Ensure all tests pass, ask the user if questions arise.

## Phase 5: VNC Native Embedding

- [x] 10. Implement VNC session widget
  - [x] 10.1 Create `rustconn/src/session/mod.rs` module
    - Define `SessionWidget` enum (Ssh, Vnc, Rdp, Spice)
    - Define `SessionState` enum (Disconnected, Connecting, Authenticating, Connected, Error)
    - _Requirements: 2.1, 2.5_

  - [x] 10.2 Create `rustconn/src/session/vnc.rs`
    - Implement `VncSessionWidget` struct with overlay and controls
    - Implement `connect()`, `disconnect()`, `widget()`, `state()` methods
    - Handle VNC authentication callbacks
    - Handle connection state changes
    - _Requirements: 2.1, 2.2, 2.3, 2.5_

  - [x] 10.3 Write property test for VNC widget creation
    - **Property 1: Protocol widget creation returns valid GTK widget**
    - **Validates: Requirements 1.2, 2.1**

  - [x] 10.4 Write property test for session state transitions
    - **Property 2: Session state transitions are valid**
    - **Validates: Requirements 2.5**

- [x] 11. Implement floating controls component
  - [x] 11.1 Create `rustconn/src/floating_controls.rs`
    - Implement `FloatingControls` struct
    - Add disconnect, fullscreen, settings buttons
    - Implement `show()`, `hide()` with Revealer animation
    - Implement auto-hide timeout logic
    - _Requirements: 5.1, 5.2, 5.6_

  - [x] 11.2 Add CSS styling for floating controls
    - Create semi-transparent background
    - Add hover effects and transitions
    - _Requirements: 5.2_

  - [x] 11.3 Write property test for overlay control presence
    - **Property 8: Session overlay contains required controls**
    - **Validates: Requirements 5.1**

  - [x] 11.4 Write property test for fullscreen toggle idempotence
    - **Property 7: Fullscreen state toggle is idempotent pair**
    - **Validates: Requirements 5.4**

- [x] 12. Integrate VNC into terminal notebook
  - [x] 12.1 Update `TerminalNotebook` to support VNC sessions
    - Add method to create VNC session tab
    - Integrate `VncSessionWidget` with notebook tabs
    - Handle tab switching for VNC sessions
    - _Requirements: 2.1, 2.6_

  - [x] 12.2 Update connection flow in `MainWindow`
    - Modify `start_connection()` to use native VNC widget
    - Handle VNC authentication prompts
    - _Requirements: 2.1, 2.3_

  - [x] 12.3 Write property test for session isolation
    - **Property 3: Multiple sessions maintain isolation**
    - **Validates: Requirements 2.6**

- [x] 13. Checkpoint - Verify VNC embedding works end-to-end
  - Ensure all tests pass, ask the user if questions arise.

## Phase 6: RDP Native Embedding

- [x] 14. Create RDP FFI bindings
  - [x] 14.1 Create RDP FFI bindings (`rustconn-core/src/ffi/rdp.rs`)
    - Create safe wrapper `RdpDisplay` struct based on gtk-frdp
    - Implement `new()`, `open()`, `close()`, `state()`
    - Implement `set_credentials()`, `set_clipboard_enabled()`
    - Implement signal connections for connection state changes
    - Implement `widget()` to return GTK widget
    - _Requirements: 3.1, 8.1, 8.2_

- [x] 15. Implement RDP session widget
  - [x] 15.1 Create `rustconn/src/session/rdp.rs`
    - Implement `RdpSessionWidget` struct with overlay and controls
    - Implement `connect()`, `disconnect()`, `widget()`, `state()` methods
    - Handle NLA authentication
    - Handle gateway configuration
    - _Requirements: 3.1, 3.3, 3.4_

  - [x] 15.2 Write property test for RDP widget creation
    - **Property 1: Protocol widget creation returns valid GTK widget**
    - **Validates: Requirements 1.2, 3.1**

- [x] 16. Integrate RDP into terminal notebook
  - [x] 16.1 Update `TerminalNotebook` to support RDP sessions
    - Add method to create RDP session tab
    - Integrate `RdpSessionWidget` with notebook tabs
    - _Requirements: 3.1_

  - [x] 16.2 Update connection flow for RDP
    - Modify `start_connection()` to use native RDP widget
    - Handle RDP authentication and gateway
    - _Requirements: 3.1, 3.3, 3.4_

- [x] 17. Checkpoint - Verify RDP embedding works
  - Ensure all tests pass, ask the user if questions arise.

## Phase 7: SPICE Native Embedding

- [x] 18. Create SPICE FFI bindings
  - [x] 18.1 Create SPICE FFI bindings (`rustconn-core/src/ffi/spice.rs`)
    - Create safe wrapper `SpiceDisplay` struct
    - Implement `new()`, `open()`, `close()`, `is_connected()`
    - Implement `set_usb_redirection()`, `add_shared_folder()`
    - Implement TLS certificate handling
    - Implement `widget()` to return GTK widget
    - _Requirements: 4.2, 8.1, 8.2_

  - [x] 18.2 Write property test for SPICE TLS validation
    - **Property 9: TLS certificate validation respects configuration**
    - **Validates: Requirements 4.6**

- [x] 19. Implement SPICE session widget
  - [x] 19.1 Create `rustconn/src/session/spice.rs`
    - Implement `SpiceSessionWidget` struct with overlay and controls
    - Implement `connect()`, `disconnect()`, `widget()`, `state()` methods
    - Handle SPICE agent features (clipboard, resize)
    - _Requirements: 4.2, 4.5_

  - [x] 19.2 Write property test for SPICE widget creation
    - **Property 1: Protocol widget creation returns valid GTK widget**
    - **Validates: Requirements 1.2, 4.2**

- [x] 20. Integrate SPICE into terminal notebook
  - [x] 20.1 Update `TerminalNotebook` to support SPICE sessions
    - Add method to create SPICE session tab
    - Integrate `SpiceSessionWidget` with notebook tabs
    - _Requirements: 4.2_

  - [x] 20.2 Update connection flow for SPICE
    - Modify `start_connection()` to use native SPICE widget
    - Handle SPICE-specific features
    - _Requirements: 4.2, 4.3, 4.4, 4.5_

- [x] 21. Checkpoint - Verify SPICE embedding works
  - Ensure all tests pass, ask the user if questions arise.

## Phase 8: UI Updates

- [x] 22. Update connection dialog for new protocols
  - [x] 22.1 Add SPICE protocol option to connection dialog
    - Add SPICE to protocol dropdown
    - Create SPICE-specific options tab
    - Add TLS, USB redirection, shared folders configuration
    - _Requirements: 4.1_

  - [x] 22.2 Update VNC options tab
    - Remove external client selection (TightVNC, TigerVNC, Custom)
    - Update options for native embedding (scaling, clipboard, view-only)
    - _Requirements: 2.1_

  - [x] 22.3 Update RDP options tab
    - Remove external client selection (FreeRDP, Custom)
    - Keep resolution, color depth, audio, gateway, shared folders options
    - _Requirements: 3.1_

- [x] 23. Update settings dialog
  - [x] 23.1 Remove external client path settings
    - Remove VNC client path configuration
    - Remove RDP client path configuration
    - Note: No client path configuration existed - Clients tab is informational only (auto-detection)
    - _Requirements: 1.2_

- [x] 24. Final Checkpoint - Full integration testing
  - All tests pass: 212 unit tests, 235 property tests, 4 doc tests
  - Clippy passes with no new warnings
  - Phase 8 (UI Updates) complete
