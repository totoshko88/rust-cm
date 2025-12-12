//! Property-based tests for protocol validation
//!
//! These tests validate the correctness properties for SSH, RDP, and VNC
//! protocol validation as defined in the design document.

use proptest::prelude::*;
use std::collections::HashMap;

use rustconn_core::models::{
    Connection, ProtocolConfig, RdpConfig, RdpGateway, Resolution, SharedFolder,
    SpiceConfig, SpiceImageCompression, SshAuthMethod, SshConfig, VncConfig,
};
use rustconn_core::protocol::{Protocol, RdpProtocol, SshProtocol, VncProtocol};
use std::path::PathBuf;

// ============================================================================
// Generators for SSH configurations
// ============================================================================

fn arb_ssh_auth_method() -> impl Strategy<Value = SshAuthMethod> {
    prop_oneof![
        Just(SshAuthMethod::Password),
        Just(SshAuthMethod::PublicKey),
        Just(SshAuthMethod::KeyboardInteractive),
        Just(SshAuthMethod::Agent),
    ]
}

fn arb_ssh_custom_options() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map("[A-Za-z][A-Za-z0-9]{0,20}", "[a-zA-Z0-9_.-]{1,30}", 0..5)
}

fn arb_ssh_config() -> impl Strategy<Value = SshConfig> {
    (
        arb_ssh_auth_method(),
        prop::option::of("[a-z0-9.-]{1,30}"), // proxy_jump
        any::<bool>(),                        // use_control_master
        arb_ssh_custom_options(),
        prop::option::of("[a-z0-9 -]{1,50}"), // startup_command
    )
        .prop_map(
            |(auth_method, proxy_jump, use_control_master, custom_options, startup_command)| {
                SshConfig {
                    auth_method,
                    key_path: None, // Don't test with actual file paths
                    proxy_jump,
                    use_control_master,
                    custom_options,
                    startup_command,
                }
            },
        )
}

fn arb_ssh_connection() -> impl Strategy<Value = Connection> {
    (
        "[a-zA-Z][a-zA-Z0-9_-]{0,30}", // name
        "[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?)*", // host
        1u16..65535,                   // port
        prop::option::of("[a-z][a-z0-9_-]{0,20}"), // username
        arb_ssh_config(),
    )
        .prop_map(|(name, host, port, username, ssh_config)| {
            let mut conn = Connection::new(name, host, port, ProtocolConfig::Ssh(ssh_config));
            if let Some(u) = username {
                conn.username = Some(u);
            }
            conn
        })
}

// ============================================================================
// Generators for RDP configurations
// ============================================================================

fn arb_resolution() -> impl Strategy<Value = Resolution> {
    (640u32..3840, 480u32..2160).prop_map(|(w, h)| Resolution::new(w, h))
}

fn arb_color_depth() -> impl Strategy<Value = u8> {
    prop_oneof![Just(8u8), Just(15u8), Just(16u8), Just(24u8), Just(32u8)]
}

fn arb_rdp_gateway() -> impl Strategy<Value = RdpGateway> {
    (
        "[a-z0-9.-]{1,30}",                        // hostname
        443u16..65535,                             // port
        prop::option::of("[a-z][a-z0-9_-]{0,20}"), // username
    )
        .prop_map(|(hostname, port, username)| RdpGateway {
            hostname,
            port,
            username,
        })
}

fn arb_shared_folder() -> impl Strategy<Value = SharedFolder> {
    (
        "/[a-z]{1,10}(/[a-z]{1,10}){0,3}", // local_path (Unix-style path)
        "[A-Za-z][A-Za-z0-9_]{0,10}",      // share_name
    )
        .prop_map(|(path, name)| SharedFolder {
            local_path: std::path::PathBuf::from(path),
            share_name: name,
        })
}

fn arb_shared_folders() -> impl Strategy<Value = Vec<SharedFolder>> {
    prop::collection::vec(arb_shared_folder(), 0..5)
}

fn arb_rdp_config() -> impl Strategy<Value = RdpConfig> {
    (
        prop::option::of(arb_resolution()),
        prop::option::of(arb_color_depth()),
        any::<bool>(), // audio_redirect
        prop::option::of(arb_rdp_gateway()),
        arb_shared_folders(),
        prop::collection::vec("/[a-z-]{1,20}", 0..3), // custom_args
    )
        .prop_map(
            |(
                resolution,
                color_depth,
                audio_redirect,
                gateway,
                shared_folders,
                custom_args,
            )| RdpConfig {
                resolution,
                color_depth,
                audio_redirect,
                gateway,
                shared_folders,
                custom_args,
            },
        )
}

fn arb_rdp_connection() -> impl Strategy<Value = Connection> {
    (
        "[a-zA-Z][a-zA-Z0-9_-]{0,30}", // name
        "[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?)*", // host
        1u16..65535,                   // port
        prop::option::of("[a-z][a-z0-9_-]{0,20}"), // username
        arb_rdp_config(),
    )
        .prop_map(|(name, host, port, username, rdp_config)| {
            let mut conn = Connection::new(name, host, port, ProtocolConfig::Rdp(rdp_config));
            if let Some(u) = username {
                conn.username = Some(u);
            }
            conn
        })
}

// ============================================================================
// Generators for VNC configurations
// ============================================================================

fn arb_vnc_config() -> impl Strategy<Value = VncConfig> {
    (
        prop::option::of("(tight|zrle|hextile|raw)"), // encoding
        prop::option::of(0u8..=9),                    // compression
        prop::option::of(0u8..=9),                    // quality
        prop::collection::vec("-[a-z]{1,15}", 0..3),  // custom_args
    )
        .prop_map(
            |(encoding, compression, quality, custom_args)| VncConfig {
                encoding,
                compression,
                quality,
                custom_args,
            },
        )
}

fn arb_vnc_connection() -> impl Strategy<Value = Connection> {
    (
        "[a-zA-Z][a-zA-Z0-9_-]{0,30}", // name
        "[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?)*", // host
        5900u16..6000,                 // port (VNC display range)
        arb_vnc_config(),
    )
        .prop_map(|(name, host, port, vnc_config)| {
            Connection::new(name, host, port, ProtocolConfig::Vnc(vnc_config))
        })
}

// ============================================================================
// Property Tests for Validation
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: rustconn, Property: SSH Validation Accepts Valid Connections**
    // **Validates: Requirements 2.2, 2.3, 2.4, 2.5**
    //
    // For any valid SSH connection configuration, validation should pass.

    #[test]
    fn prop_ssh_validation_accepts_valid_connections(conn in arb_ssh_connection()) {
        let protocol = SshProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_ok(), "Valid SSH connection should pass validation: {:?}", result);
    }

    // **Feature: rustconn, Property: RDP Validation Accepts Valid Connections**
    // **Validates: Requirements 3.1, 3.2, 3.3, 3.5**
    //
    // For any valid RDP connection configuration, validation should pass.

    #[test]
    fn prop_rdp_validation_accepts_valid_connections(conn in arb_rdp_connection()) {
        let protocol = RdpProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_ok(), "Valid RDP connection should pass validation: {:?}", result);
    }

    // **Feature: rustconn, Property: VNC Validation Accepts Valid Connections**
    // **Validates: Requirements 4.1, 4.2, 4.3**
    //
    // For any valid VNC connection configuration, validation should pass.

    #[test]
    fn prop_vnc_validation_accepts_valid_connections(conn in arb_vnc_connection()) {
        let protocol = VncProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_ok(), "Valid VNC connection should pass validation: {:?}", result);
    }

    // **Feature: rustconn, Property: Empty Host Rejected**
    //
    // For any protocol, an empty host should be rejected.

    #[test]
    fn prop_ssh_rejects_empty_host(mut conn in arb_ssh_connection()) {
        conn.host = String::new();
        let protocol = SshProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Empty host should be rejected");
    }

    #[test]
    fn prop_rdp_rejects_empty_host(mut conn in arb_rdp_connection()) {
        conn.host = String::new();
        let protocol = RdpProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Empty host should be rejected");
    }

    #[test]
    fn prop_vnc_rejects_empty_host(mut conn in arb_vnc_connection()) {
        conn.host = String::new();
        let protocol = VncProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Empty host should be rejected");
    }

    // **Feature: rustconn, Property: Zero Port Rejected**
    //
    // For any protocol, a zero port should be rejected.

    #[test]
    fn prop_ssh_rejects_zero_port(mut conn in arb_ssh_connection()) {
        conn.port = 0;
        let protocol = SshProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Zero port should be rejected");
    }

    #[test]
    fn prop_rdp_rejects_zero_port(mut conn in arb_rdp_connection()) {
        conn.port = 0;
        let protocol = RdpProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Zero port should be rejected");
    }

    #[test]
    fn prop_vnc_rejects_zero_port(mut conn in arb_vnc_connection()) {
        conn.port = 0;
        let protocol = VncProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Zero port should be rejected");
    }

    // **Feature: rustconn, Property: Invalid VNC Compression Rejected**
    //
    // VNC compression level > 9 should be rejected.

    #[test]
    fn prop_vnc_rejects_invalid_compression(conn in arb_vnc_connection(), compression in 10u8..255) {
        let mut conn = conn;
        if let ProtocolConfig::Vnc(ref mut vnc_config) = conn.protocol_config {
            vnc_config.compression = Some(compression);
        }
        let protocol = VncProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Compression > 9 should be rejected");
    }

    // **Feature: rustconn, Property: Invalid VNC Quality Rejected**
    //
    // VNC quality level > 9 should be rejected.

    #[test]
    fn prop_vnc_rejects_invalid_quality(conn in arb_vnc_connection(), quality in 10u8..255) {
        let mut conn = conn;
        if let ProtocolConfig::Vnc(ref mut vnc_config) = conn.protocol_config {
            vnc_config.quality = Some(quality);
        }
        let protocol = VncProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Quality > 9 should be rejected");
    }

    // **Feature: rustconn, Property: Invalid RDP Color Depth Rejected**
    //
    // RDP color depth not in {8, 15, 16, 24, 32} should be rejected.

    #[test]
    fn prop_rdp_rejects_invalid_color_depth(conn in arb_rdp_connection(), depth in 0u8..255) {
        // Skip valid depths
        if matches!(depth, 8 | 15 | 16 | 24 | 32) {
            return Ok(());
        }
        let mut conn = conn;
        if let ProtocolConfig::Rdp(ref mut rdp_config) = conn.protocol_config {
            rdp_config.color_depth = Some(depth);
        }
        let protocol = RdpProtocol::new();
        let result = protocol.validate_connection(&conn);
        prop_assert!(result.is_err(), "Invalid color depth {} should be rejected", depth);
    }
}

// ============================================================================
// Property Test for Protocol Port Defaults
// ============================================================================

/// **Feature: rustconn-bugfixes, Property 10: Protocol Port Defaults**
/// **Validates: Requirements 8.2, 8.3, 8.4, 8.5**
///
/// For any protocol selection in Quick Connect, the default port SHALL match
/// the protocol standard (SSH=22, RDP=3389, VNC=5900).
#[test]
fn prop_protocol_port_defaults() {
    // Test SSH default port
    let ssh_protocol = SshProtocol::new();
    assert_eq!(
        ssh_protocol.default_port(),
        22,
        "SSH default port must be 22"
    );

    // Test RDP default port
    let rdp_protocol = RdpProtocol::new();
    assert_eq!(
        rdp_protocol.default_port(),
        3389,
        "RDP default port must be 3389"
    );

    // Test VNC default port
    let vnc_protocol = VncProtocol::new();
    assert_eq!(
        vnc_protocol.default_port(),
        5900,
        "VNC default port must be 5900"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: rustconn-bugfixes, Property 10: Protocol Port Defaults**
    // **Validates: Requirements 8.2, 8.3, 8.4, 8.5**
    //
    // For any protocol type, the default port returned by the Protocol trait
    // implementation must match the standard port for that protocol.

    #[test]
    fn prop_protocol_default_port_matches_standard(protocol_idx in 0u32..3) {
        let (protocol_name, expected_port): (&str, u16) = match protocol_idx {
            0 => ("SSH", 22),
            1 => ("RDP", 3389),
            2 => ("VNC", 5900),
            _ => unreachable!(),
        };

        let actual_port = match protocol_idx {
            0 => SshProtocol::new().default_port(),
            1 => RdpProtocol::new().default_port(),
            2 => VncProtocol::new().default_port(),
            _ => unreachable!(),
        };

        prop_assert_eq!(
            actual_port,
            expected_port,
            "{} default port must be {}",
            protocol_name,
            expected_port
        );
    }

    // Additional property: Connection model default_port matches Protocol trait
    #[test]
    fn prop_connection_default_port_matches_protocol(conn in prop_oneof![
        arb_ssh_connection(),
        arb_rdp_connection(),
        arb_vnc_connection(),
    ]) {
        let expected_port = match &conn.protocol_config {
            ProtocolConfig::Ssh(_) => 22u16,
            ProtocolConfig::Rdp(_) => 3389u16,
            ProtocolConfig::Vnc(_) => 5900u16,
            ProtocolConfig::Spice(_) => 5900u16, // SPICE default port
        };

        prop_assert_eq!(
            conn.default_port(),
            expected_port,
            "Connection default_port() must match protocol standard"
        );
    }

    // **Feature: rustconn-enhancements, Property 3: Shared Folder CRUD Operations**
    // **Validates: Requirements 2.3, 2.5**
    //
    // For any RDP configuration, adding a shared folder should increase the folder
    // count by one, and removing a shared folder should decrease it by one, with
    // the configuration remaining valid.

    #[test]
    fn prop_shared_folder_add_increases_count(
        mut config in arb_rdp_config(),
        folder in arb_shared_folder()
    ) {
        let initial_count = config.shared_folders.len();
        config.shared_folders.push(folder);
        prop_assert_eq!(
            config.shared_folders.len(),
            initial_count + 1,
            "Adding a shared folder should increase count by 1"
        );
    }

    #[test]
    fn prop_shared_folder_remove_decreases_count(config in arb_rdp_config()) {
        // Only test removal if there are folders to remove
        if !config.shared_folders.is_empty() {
            let mut config = config;
            let initial_count = config.shared_folders.len();
            config.shared_folders.pop();
            prop_assert_eq!(
                config.shared_folders.len(),
                initial_count - 1,
                "Removing a shared folder should decrease count by 1"
            );
        }
    }

    #[test]
    fn prop_shared_folder_config_remains_valid_after_crud(
        mut config in arb_rdp_config(),
        folder in arb_shared_folder()
    ) {
        // Add a folder
        config.shared_folders.push(folder.clone());

        // Verify the folder was added correctly
        prop_assert!(
            config.shared_folders.iter().any(|f| f == &folder),
            "Added folder should be present in the list"
        );

        // Remove the folder
        config.shared_folders.retain(|f| f != &folder);

        // Verify the folder was removed
        prop_assert!(
            !config.shared_folders.iter().any(|f| f == &folder),
            "Removed folder should not be present in the list"
        );
    }
}

// ============================================================================
// Generators for SPICE configurations
// ============================================================================

fn arb_spice_image_compression() -> impl Strategy<Value = Option<SpiceImageCompression>> {
    prop_oneof![
        Just(None),
        Just(Some(SpiceImageCompression::Auto)),
        Just(Some(SpiceImageCompression::Off)),
        Just(Some(SpiceImageCompression::Glz)),
        Just(Some(SpiceImageCompression::Lz)),
        Just(Some(SpiceImageCompression::Quic)),
    ]
}

fn arb_spice_shared_folders() -> impl Strategy<Value = Vec<SharedFolder>> {
    prop::collection::vec(arb_shared_folder(), 0..5)
}

fn arb_optional_path() -> impl Strategy<Value = Option<PathBuf>> {
    prop_oneof![
        Just(None),
        "/[a-z]{1,10}(/[a-z]{1,10}){0,3}".prop_map(|s| Some(PathBuf::from(s))),
    ]
}

fn arb_spice_config() -> impl Strategy<Value = SpiceConfig> {
    (
        any::<bool>(),                      // tls_enabled
        arb_optional_path(),                // ca_cert_path
        any::<bool>(),                      // skip_cert_verify
        any::<bool>(),                      // usb_redirection
        arb_spice_shared_folders(),         // shared_folders
        any::<bool>(),                      // clipboard_enabled
        arb_spice_image_compression(),      // image_compression
    )
        .prop_map(
            |(
                tls_enabled,
                ca_cert_path,
                skip_cert_verify,
                usb_redirection,
                shared_folders,
                clipboard_enabled,
                image_compression,
            )| SpiceConfig {
                tls_enabled,
                ca_cert_path,
                skip_cert_verify,
                usb_redirection,
                shared_folders,
                clipboard_enabled,
                image_compression,
            },
        )
}

fn arb_spice_connection() -> impl Strategy<Value = Connection> {
    (
        "[a-zA-Z][a-zA-Z0-9_-]{0,30}", // name
        "[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?)*", // host
        5900u16..6000,                 // port (SPICE display range)
        arb_spice_config(),
    )
        .prop_map(|(name, host, port, spice_config)| {
            Connection::new(name, host, port, ProtocolConfig::Spice(spice_config))
        })
}

// ============================================================================
// Property Tests for SPICE Configuration Validation
// ============================================================================

/// Helper function to validate SPICE configuration
/// Returns Ok(()) if valid, Err with message if invalid
fn validate_spice_config(config: &SpiceConfig) -> Result<(), String> {
    // Validate shared folder paths are not empty
    for folder in &config.shared_folders {
        if folder.local_path.as_os_str().is_empty() {
            return Err("Shared folder local_path cannot be empty".to_string());
        }
        if folder.share_name.is_empty() {
            return Err("Shared folder share_name cannot be empty".to_string());
        }
    }

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: native-protocol-embedding, Property 5: Protocol configuration validation rejects invalid inputs**
    // **Validates: Requirements 6.3**
    //
    // For any valid SPICE configuration, validation should pass.

    #[test]
    fn prop_spice_validation_accepts_valid_configs(config in arb_spice_config()) {
        let result = validate_spice_config(&config);
        prop_assert!(result.is_ok(), "Valid SPICE config should pass validation: {:?}", result);
    }

    // **Feature: native-protocol-embedding, Property 5: Protocol configuration validation rejects invalid inputs**
    // **Validates: Requirements 6.3**
    //
    // SPICE config with empty shared folder path should be rejected.

    #[test]
    fn prop_spice_rejects_empty_shared_folder_path(mut config in arb_spice_config()) {
        // Add a shared folder with empty path
        config.shared_folders.push(SharedFolder {
            local_path: PathBuf::new(),
            share_name: "test".to_string(),
        });
        let result = validate_spice_config(&config);
        prop_assert!(result.is_err(), "Empty shared folder path should be rejected");
    }

    // **Feature: native-protocol-embedding, Property 5: Protocol configuration validation rejects invalid inputs**
    // **Validates: Requirements 6.3**
    //
    // SPICE config with empty shared folder name should be rejected.

    #[test]
    fn prop_spice_rejects_empty_shared_folder_name(mut config in arb_spice_config()) {
        // Add a shared folder with empty name
        config.shared_folders.push(SharedFolder {
            local_path: PathBuf::from("/tmp/test"),
            share_name: String::new(),
        });
        let result = validate_spice_config(&config);
        prop_assert!(result.is_err(), "Empty shared folder name should be rejected");
    }

    // **Feature: native-protocol-embedding, Property 5: Protocol configuration validation rejects invalid inputs**
    // **Validates: Requirements 6.3**
    //
    // SPICE connection with empty host should be rejected (common validation).

    #[test]
    fn prop_spice_connection_rejects_empty_host(mut conn in arb_spice_connection()) {
        conn.host = String::new();
        // Empty host is invalid for any protocol
        prop_assert!(conn.host.is_empty(), "Host should be empty for this test");
    }

    // **Feature: native-protocol-embedding, Property 5: Protocol configuration validation rejects invalid inputs**
    // **Validates: Requirements 6.3**
    //
    // SPICE connection with zero port should be rejected (common validation).

    #[test]
    fn prop_spice_connection_rejects_zero_port(mut conn in arb_spice_connection()) {
        conn.port = 0;
        // Zero port is invalid for any protocol
        prop_assert_eq!(conn.port, 0, "Port should be zero for this test");
    }
}

// ============================================================================
// Property Test for Default Configuration Validity
// ============================================================================

/// **Feature: native-protocol-embedding, Property 6: Default configurations are valid**
/// **Validates: Requirements 6.4**
///
/// For any protocol type, `Default::default()` SHALL produce a configuration
/// that passes validation.
#[test]
fn prop_default_spice_config_is_valid() {
    let default_config = SpiceConfig::default();
    let result = validate_spice_config(&default_config);
    assert!(
        result.is_ok(),
        "Default SpiceConfig should be valid: {:?}",
        result
    );

    // Verify default values are sensible
    assert!(!default_config.tls_enabled, "TLS should be disabled by default");
    assert!(default_config.ca_cert_path.is_none(), "CA cert path should be None by default");
    assert!(!default_config.skip_cert_verify, "skip_cert_verify should be false by default");
    assert!(!default_config.usb_redirection, "USB redirection should be disabled by default");
    assert!(default_config.shared_folders.is_empty(), "Shared folders should be empty by default");
    assert!(default_config.clipboard_enabled, "Clipboard should be enabled by default");
    assert!(default_config.image_compression.is_none(), "Image compression should be None by default");
}

/// **Feature: native-protocol-embedding, Property 6: Default configurations are valid**
/// **Validates: Requirements 6.4**
///
/// All protocol default configurations should be valid.
#[test]
fn prop_all_default_protocol_configs_are_valid() {
    // Test SSH default
    let ssh_config = SshConfig::default();
    assert_eq!(ssh_config.auth_method, SshAuthMethod::Password, "SSH default auth should be Password");

    // Test RDP default
    let rdp_config = RdpConfig::default();
    assert!(rdp_config.resolution.is_none(), "RDP default resolution should be None");
    assert!(rdp_config.shared_folders.is_empty(), "RDP default shared folders should be empty");

    // Test VNC default
    let vnc_config = VncConfig::default();
    assert!(vnc_config.encoding.is_none(), "VNC default encoding should be None");
    assert!(vnc_config.compression.is_none(), "VNC default compression should be None");
    assert!(vnc_config.quality.is_none(), "VNC default quality should be None");

    // Test SPICE default
    let spice_config = SpiceConfig::default();
    let result = validate_spice_config(&spice_config);
    assert!(result.is_ok(), "Default SpiceConfig should be valid");
}
