//! Property-based tests for serialization round-trip
//!
//! **Feature: rustconn, Property 6: Connection Serialization Round-Trip**
//! **Validates: Requirements 10.5, 10.6**

use proptest::prelude::*;
use rustconn_core::{
    Connection, ProtocolConfig, RdpClient, RdpConfig, RdpGateway, Resolution, SshAuthMethod,
    SshConfig, VncClient, VncConfig,
};
use std::collections::HashMap;
use std::path::PathBuf;

// Strategy for generating valid connection names
fn arb_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,31}".prop_map(|s| s)
}

// Strategy for generating valid hostnames
fn arb_host() -> impl Strategy<Value = String> {
    "[a-z0-9]([a-z0-9-]{0,15}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,15}[a-z0-9])?)*"
        .prop_map(|s| s)
}

// Strategy for generating valid ports
fn arb_port() -> impl Strategy<Value = u16> {
    1u16..=65535u16
}

// Strategy for generating optional usernames
fn arb_username() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[a-z][a-z0-9_]{0,15}".prop_map(Some),
    ]
}

// Strategy for generating tags
fn arb_tags() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-z]{1,10}", 0..5)
}

// Strategy for SSH auth method
fn arb_ssh_auth_method() -> impl Strategy<Value = SshAuthMethod> {
    prop_oneof![
        Just(SshAuthMethod::Password),
        Just(SshAuthMethod::PublicKey),
        Just(SshAuthMethod::KeyboardInteractive),
        Just(SshAuthMethod::Agent),
    ]
}

// Strategy for optional PathBuf
fn arb_optional_path() -> impl Strategy<Value = Option<PathBuf>> {
    prop_oneof![
        Just(None),
        "[a-z]{1,10}(/[a-z]{1,10}){0,3}".prop_map(|s| Some(PathBuf::from(s))),
    ]
}

// Strategy for optional string
fn arb_optional_string() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[a-zA-Z0-9_-]{1,20}".prop_map(Some),
    ]
}

// Strategy for custom SSH options
fn arb_custom_options() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map("[A-Za-z]{1,20}", "[a-zA-Z0-9]{1,10}", 0..3)
}

// Strategy for SSH config
fn arb_ssh_config() -> impl Strategy<Value = SshConfig> {
    (
        arb_ssh_auth_method(),
        arb_optional_path(),
        arb_optional_string(),
        any::<bool>(),
        arb_custom_options(),
        arb_optional_string(),
    )
        .prop_map(
            |(auth_method, key_path, proxy_jump, use_control_master, custom_options, startup_command)| {
                SshConfig {
                    auth_method,
                    key_path,
                    proxy_jump,
                    use_control_master,
                    custom_options,
                    startup_command,
                }
            },
        )
}


// Strategy for RDP client
fn arb_rdp_client() -> impl Strategy<Value = RdpClient> {
    prop_oneof![
        Just(RdpClient::FreeRdp),
        "[a-z]{1,10}(/[a-z]{1,10}){0,2}".prop_map(|s| RdpClient::Custom(PathBuf::from(s))),
    ]
}

// Strategy for optional resolution
fn arb_optional_resolution() -> impl Strategy<Value = Option<Resolution>> {
    prop_oneof![
        Just(None),
        (640u32..4096u32, 480u32..2160u32).prop_map(|(w, h)| Some(Resolution::new(w, h))),
    ]
}

// Strategy for optional color depth
fn arb_optional_color_depth() -> impl Strategy<Value = Option<u8>> {
    prop_oneof![
        Just(None),
        prop_oneof![Just(8u8), Just(15u8), Just(16u8), Just(24u8), Just(32u8)].prop_map(Some),
    ]
}

// Strategy for optional RDP gateway
fn arb_optional_gateway() -> impl Strategy<Value = Option<RdpGateway>> {
    prop_oneof![
        Just(None),
        (arb_host(), 1u16..65535u16, arb_optional_string()).prop_map(|(hostname, port, username)| {
            Some(RdpGateway {
                hostname,
                port,
                username,
            })
        }),
    ]
}

// Strategy for custom args
fn arb_custom_args() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-zA-Z0-9_=-]{1,20}", 0..3)
}

// Strategy for RDP config
fn arb_rdp_config() -> impl Strategy<Value = RdpConfig> {
    (
        arb_rdp_client(),
        arb_optional_resolution(),
        arb_optional_color_depth(),
        any::<bool>(),
        arb_optional_gateway(),
        arb_custom_args(),
    )
        .prop_map(
            |(client, resolution, color_depth, audio_redirect, gateway, custom_args)| RdpConfig {
                client,
                resolution,
                color_depth,
                audio_redirect,
                gateway,
                custom_args,
            },
        )
}

// Strategy for VNC client
fn arb_vnc_client() -> impl Strategy<Value = VncClient> {
    prop_oneof![
        Just(VncClient::TightVnc),
        Just(VncClient::TigerVnc),
        "[a-z]{1,10}(/[a-z]{1,10}){0,2}".prop_map(|s| VncClient::Custom(PathBuf::from(s))),
    ]
}

// Strategy for optional encoding
fn arb_optional_encoding() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        prop_oneof![
            Just("tight".to_string()),
            Just("zrle".to_string()),
            Just("hextile".to_string()),
        ]
        .prop_map(Some),
    ]
}

// Strategy for optional compression/quality (0-9)
fn arb_optional_level() -> impl Strategy<Value = Option<u8>> {
    prop_oneof![
        Just(None),
        (0u8..=9u8).prop_map(Some),
    ]
}

// Strategy for VNC config
fn arb_vnc_config() -> impl Strategy<Value = VncConfig> {
    (
        arb_vnc_client(),
        arb_optional_encoding(),
        arb_optional_level(),
        arb_optional_level(),
        arb_custom_args(),
    )
        .prop_map(|(client, encoding, compression, quality, custom_args)| VncConfig {
            client,
            encoding,
            compression,
            quality,
            custom_args,
        })
}

// Strategy for protocol config
fn arb_protocol_config() -> impl Strategy<Value = ProtocolConfig> {
    prop_oneof![
        arb_ssh_config().prop_map(ProtocolConfig::Ssh),
        arb_rdp_config().prop_map(ProtocolConfig::Rdp),
        arb_vnc_config().prop_map(ProtocolConfig::Vnc),
    ]
}


// Strategy for generating a complete Connection
fn arb_connection() -> impl Strategy<Value = Connection> {
    (
        arb_name(),
        arb_host(),
        arb_port(),
        arb_protocol_config(),
        arb_username(),
        arb_tags(),
    )
        .prop_map(|(name, host, port, protocol_config, username, tags)| {
            let mut conn = Connection::new(name, host, port, protocol_config);
            if let Some(u) = username {
                conn = conn.with_username(u);
            }
            if !tags.is_empty() {
                conn = conn.with_tags(tags);
            }
            conn
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: rustconn, Property 6: Connection Serialization Round-Trip**
    /// **Validates: Requirements 10.5, 10.6**
    ///
    /// For any valid Connection object, serializing to TOML and then deserializing
    /// should produce an equivalent Connection object with all fields preserved.
    #[test]
    fn connection_toml_round_trip(conn in arb_connection()) {
        // Serialize to TOML
        let toml_str = toml::to_string(&conn)
            .expect("Connection should serialize to TOML");

        // Deserialize back from TOML
        let deserialized: Connection = toml::from_str(&toml_str)
            .expect("TOML should deserialize back to Connection");

        // Verify all fields are preserved
        prop_assert_eq!(conn.id, deserialized.id, "ID should be preserved");
        prop_assert_eq!(conn.name, deserialized.name, "Name should be preserved");
        prop_assert_eq!(conn.protocol, deserialized.protocol, "Protocol type should be preserved");
        prop_assert_eq!(conn.host, deserialized.host, "Host should be preserved");
        prop_assert_eq!(conn.port, deserialized.port, "Port should be preserved");
        prop_assert_eq!(conn.username, deserialized.username, "Username should be preserved");
        prop_assert_eq!(conn.group_id, deserialized.group_id, "Group ID should be preserved");
        prop_assert_eq!(conn.tags, deserialized.tags, "Tags should be preserved");
        prop_assert_eq!(conn.protocol_config, deserialized.protocol_config, "Protocol config should be preserved");

        // Timestamps may have nanosecond precision loss in TOML, so compare at second precision
        prop_assert_eq!(
            conn.created_at.timestamp(),
            deserialized.created_at.timestamp(),
            "Created timestamp should be preserved (second precision)"
        );
        prop_assert_eq!(
            conn.updated_at.timestamp(),
            deserialized.updated_at.timestamp(),
            "Updated timestamp should be preserved (second precision)"
        );
    }

    /// Additional test: Connection JSON round-trip for completeness
    /// This validates that the serde implementation works across formats
    #[test]
    fn connection_json_round_trip(conn in arb_connection()) {
        // Serialize to JSON
        let json_str = serde_json::to_string(&conn)
            .expect("Connection should serialize to JSON");

        // Deserialize back from JSON
        let deserialized: Connection = serde_json::from_str(&json_str)
            .expect("JSON should deserialize back to Connection");

        // Verify equality (JSON preserves nanosecond precision)
        prop_assert_eq!(conn, deserialized, "Connection should round-trip through JSON");
    }
}
