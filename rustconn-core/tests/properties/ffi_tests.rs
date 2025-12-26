//! Property-based tests for FFI bindings
//!
//! These tests validate the correctness properties for FFI wrappers
//! as defined in the design document for native protocol embedding.
//!
//! Note: RDP and SPICE protocols use native Rust implementations:
//! - RDP: `rustconn/src/embedded_rdp.rs` with `ironrdp` crate
//! - SPICE: `rustconn/src/embedded_spice.rs` with `spice-client` crate

use proptest::prelude::*;
use rustconn_core::ffi::{ConnectionState, FfiDisplay, VncCredentialType, VncDisplay};

// ============================================================================
// Generators for VNC configurations
// ============================================================================

/// Strategy for generating valid hostnames
fn arb_host() -> impl Strategy<Value = String> {
    "[a-z0-9]([a-z0-9-]{0,15}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,15}[a-z0-9])?)*"
}

/// Strategy for generating valid ports (non-zero)
fn arb_port() -> impl Strategy<Value = u16> {
    1u16..=65535u16
}

/// Strategy for generating VNC credential types
fn arb_credential_type() -> impl Strategy<Value = VncCredentialType> {
    prop_oneof![
        Just(VncCredentialType::Password),
        Just(VncCredentialType::Username),
        Just(VncCredentialType::ClientName),
    ]
}

/// Strategy for generating non-empty credential values
fn arb_credential_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9!@#$%^&*()_+-=]{1,64}"
}

/// Strategy for generating connection states
fn arb_connection_state() -> impl Strategy<Value = ConnectionState> {
    prop_oneof![
        Just(ConnectionState::Disconnected),
        Just(ConnectionState::Connecting),
        Just(ConnectionState::Authenticating),
        Just(ConnectionState::Connected),
        Just(ConnectionState::Error),
    ]
}

// ============================================================================
// Property Tests for VNC Display
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_vnc_display_initial_state_is_disconnected(_seed in any::<u64>()) {
        let display = VncDisplay::new();

        prop_assert_eq!(
            display.connection_state(),
            ConnectionState::Disconnected,
            "New VncDisplay should start in Disconnected state"
        );

        prop_assert!(!display.is_open(), "New VncDisplay should not be open");
        prop_assert!(display.host().is_none(), "New VncDisplay should have no host");
        prop_assert!(display.port().is_none(), "New VncDisplay should have no port");
    }

    #[test]
    fn prop_vnc_display_open_host_transitions_to_connecting(
        host in arb_host(),
        port in arb_port()
    ) {
        let display = VncDisplay::new();

        let result = display.open_host(&host, port);
        prop_assert!(result.is_ok(), "open_host should succeed with valid host and port");

        prop_assert_eq!(
            display.connection_state(),
            ConnectionState::Connecting,
            "State should be Connecting after open_host"
        );

        prop_assert_eq!(display.host(), Some(host), "Host should be set after open_host");
        prop_assert_eq!(display.port(), Some(port), "Port should be set after open_host");
    }

    #[test]
    fn prop_vnc_display_close_returns_to_disconnected(
        host in arb_host(),
        port in arb_port()
    ) {
        let display = VncDisplay::new();

        display.open_host(&host, port).unwrap();
        display.close();

        prop_assert_eq!(
            display.connection_state(),
            ConnectionState::Disconnected,
            "State should be Disconnected after close"
        );

        prop_assert!(display.host().is_none(), "Host should be None after close");
        prop_assert!(display.port().is_none(), "Port should be None after close");
    }

    #[test]
    fn prop_vnc_display_rejects_empty_host(port in arb_port()) {
        let display = VncDisplay::new();

        let result = display.open_host("", port);
        prop_assert!(result.is_err(), "open_host should reject empty host");

        prop_assert_eq!(
            display.connection_state(),
            ConnectionState::Disconnected,
            "State should remain Disconnected after rejected open_host"
        );
    }

    #[test]
    fn prop_vnc_display_rejects_zero_port(host in arb_host()) {
        let display = VncDisplay::new();

        let result = display.open_host(&host, 0);
        prop_assert!(result.is_err(), "open_host should reject zero port");

        prop_assert_eq!(
            display.connection_state(),
            ConnectionState::Disconnected,
            "State should remain Disconnected after rejected open_host"
        );
    }

    #[test]
    fn prop_vnc_display_rejects_duplicate_connection(
        host in arb_host(),
        port in arb_port()
    ) {
        let display = VncDisplay::new();

        let result1 = display.open_host(&host, port);
        prop_assert!(result1.is_ok(), "First open_host should succeed");

        let result2 = display.open_host(&host, port);
        prop_assert!(result2.is_err(), "Second open_host should fail while connecting");
    }

    #[test]
    fn prop_vnc_display_set_credential_accepts_valid_values(
        cred_type in arb_credential_type(),
        value in arb_credential_value()
    ) {
        let display = VncDisplay::new();

        let result = display.set_credential(cred_type, &value);
        prop_assert!(result.is_ok(), "set_credential should accept non-empty value");
    }

    #[test]
    fn prop_vnc_display_set_credential_rejects_empty_values(
        cred_type in arb_credential_type()
    ) {
        let display = VncDisplay::new();

        let result = display.set_credential(cred_type, "");
        prop_assert!(result.is_err(), "set_credential should reject empty value");
    }

    #[test]
    fn prop_vnc_display_scaling_toggle_is_consistent(enabled in any::<bool>()) {
        let display = VncDisplay::new();

        display.set_scaling(enabled);
        prop_assert_eq!(
            display.scaling_enabled(),
            enabled,
            "scaling_enabled should match what was set"
        );
    }

    #[test]
    fn prop_vnc_display_ffi_display_trait_consistency(
        host in arb_host(),
        port in arb_port()
    ) {
        let display = VncDisplay::new();

        prop_assert_eq!(
            FfiDisplay::state(&display),
            display.connection_state(),
            "FfiDisplay::state should match connection_state"
        );

        prop_assert_eq!(
            FfiDisplay::is_connected(&display),
            display.is_open(),
            "FfiDisplay::is_connected should match is_open"
        );

        display.open_host(&host, port).unwrap();
        FfiDisplay::close(&display);

        prop_assert_eq!(
            display.connection_state(),
            ConnectionState::Disconnected,
            "FfiDisplay::close should disconnect"
        );
    }
}

// ============================================================================
// Property Tests for Connection State
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_connection_state_display_is_non_empty(state in arb_connection_state()) {
        let display_str = state.to_string();
        prop_assert!(!display_str.is_empty(), "ConnectionState display should not be empty");
    }

    #[test]
    fn prop_connection_state_default_is_disconnected(_seed in any::<u64>()) {
        let state: ConnectionState = Default::default();
        prop_assert_eq!(
            state,
            ConnectionState::Disconnected,
            "Default ConnectionState should be Disconnected"
        );
    }

    #[test]
    fn prop_connection_state_equality_is_reflexive(state in arb_connection_state()) {
        prop_assert_eq!(state, state, "ConnectionState equality should be reflexive");
    }
}

// ============================================================================
// Property Tests for VNC Credential Type
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_vnc_credential_type_display_is_non_empty(cred_type in arb_credential_type()) {
        let display_str = cred_type.to_string();
        prop_assert!(!display_str.is_empty(), "VncCredentialType display should not be empty");
    }

    #[test]
    fn prop_vnc_credential_type_equality_is_reflexive(cred_type in arb_credential_type()) {
        prop_assert_eq!(cred_type, cred_type, "VncCredentialType equality should be reflexive");
    }
}

// ============================================================================
// Unit Tests for Edge Cases
// ============================================================================

#[test]
fn test_vnc_display_lifecycle() {
    let display = VncDisplay::new();
    assert_eq!(display.connection_state(), ConnectionState::Disconnected);

    display.open_host("localhost", 5900).unwrap();
    assert_eq!(display.connection_state(), ConnectionState::Connecting);

    display
        .set_credential(VncCredentialType::Password, "test")
        .unwrap();
    display
        .set_credential(VncCredentialType::Username, "user")
        .unwrap();

    display.set_scaling(true);
    assert!(display.scaling_enabled());

    display.close();
    assert_eq!(display.connection_state(), ConnectionState::Disconnected);
}

#[test]
fn test_vnc_display_drop_cleanup() {
    {
        let display = VncDisplay::new();
        display.open_host("localhost", 5900).unwrap();
    }
    // If we get here without panicking, cleanup worked
}

#[test]
fn test_vnc_display_signal_callbacks() {
    use std::cell::Cell;
    use std::rc::Rc;

    let display = VncDisplay::new();

    let connected_called = Rc::new(Cell::new(false));
    let disconnected_called = Rc::new(Cell::new(false));
    let auth_called = Rc::new(Cell::new(false));
    let auth_failure_called = Rc::new(Cell::new(false));

    let cc = connected_called.clone();
    display.connect_vnc_connected(move |_| cc.set(true));

    let dc = disconnected_called.clone();
    display.connect_vnc_disconnected(move |_| dc.set(true));

    let ac = auth_called.clone();
    display.connect_vnc_auth_credential(move |_, _| ac.set(true));

    let afc = auth_failure_called.clone();
    display.connect_vnc_auth_failure(move |_, _| afc.set(true));

    assert!(!connected_called.get());
    assert!(!disconnected_called.get());
    assert!(!auth_called.get());
    assert!(!auth_failure_called.get());
}
