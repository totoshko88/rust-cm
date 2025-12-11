//! Property-based tests for protocol command builders
//!
//! These tests validate the correctness properties for SSH, RDP, and VNC
//! command builders as defined in the design document.

use proptest::prelude::*;
use std::collections::HashMap;

use rustconn_core::models::{
    Connection, ProtocolConfig, RdpClient, RdpConfig, RdpGateway, Resolution, SshAuthMethod,
    SshConfig, VncClient, VncConfig,
};
use rustconn_core::protocol::{Protocol, RdpProtocol, SshProtocol, VncProtocol};

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
    prop::collection::hash_map(
        "[A-Za-z][A-Za-z0-9]{0,20}",
        "[a-zA-Z0-9_.-]{1,30}",
        0..5,
    )
}

fn arb_ssh_config() -> impl Strategy<Value = SshConfig> {
    (
        arb_ssh_auth_method(),
        prop::option::of("[a-z0-9.-]{1,30}"),  // proxy_jump
        any::<bool>(),                          // use_control_master
        arb_ssh_custom_options(),
        prop::option::of("[a-z0-9 -]{1,50}"),  // startup_command
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
        "[a-zA-Z][a-zA-Z0-9_-]{0,30}",           // name
        "[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?)*", // host
        1u16..65535,                             // port
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

fn arb_rdp_client() -> impl Strategy<Value = RdpClient> {
    // Only test FreeRdp since Custom requires existing file path
    Just(RdpClient::FreeRdp)
}

fn arb_resolution() -> impl Strategy<Value = Resolution> {
    (640u32..3840, 480u32..2160).prop_map(|(w, h)| Resolution::new(w, h))
}

fn arb_color_depth() -> impl Strategy<Value = u8> {
    prop_oneof![Just(8u8), Just(15u8), Just(16u8), Just(24u8), Just(32u8)]
}

fn arb_rdp_gateway() -> impl Strategy<Value = RdpGateway> {
    (
        "[a-z0-9.-]{1,30}",                       // hostname
        443u16..65535,                            // port
        prop::option::of("[a-z][a-z0-9_-]{0,20}"), // username
    )
        .prop_map(|(hostname, port, username)| RdpGateway {
            hostname,
            port,
            username,
        })
}

fn arb_rdp_config() -> impl Strategy<Value = RdpConfig> {
    (
        arb_rdp_client(),
        prop::option::of(arb_resolution()),
        prop::option::of(arb_color_depth()),
        any::<bool>(),                            // audio_redirect
        prop::option::of(arb_rdp_gateway()),
        prop::collection::vec("/[a-z-]{1,20}", 0..3), // custom_args
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

fn arb_rdp_connection() -> impl Strategy<Value = Connection> {
    (
        "[a-zA-Z][a-zA-Z0-9_-]{0,30}",           // name
        "[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?)*", // host
        1u16..65535,                             // port
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

fn arb_vnc_client() -> impl Strategy<Value = VncClient> {
    prop_oneof![Just(VncClient::TightVnc), Just(VncClient::TigerVnc),]
}

fn arb_vnc_config() -> impl Strategy<Value = VncConfig> {
    (
        arb_vnc_client(),
        prop::option::of("(tight|zrle|hextile|raw)"), // encoding
        prop::option::of(0u8..=9),                     // compression
        prop::option::of(0u8..=9),                     // quality
        prop::collection::vec("-[a-z]{1,15}", 0..3),  // custom_args
    )
        .prop_map(|(client, encoding, compression, quality, custom_args)| VncConfig {
            client,
            encoding,
            compression,
            quality,
            custom_args,
        })
}

fn arb_vnc_connection() -> impl Strategy<Value = Connection> {
    (
        "[a-zA-Z][a-zA-Z0-9_-]{0,30}",           // name
        "[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,30}[a-z0-9])?)*", // host
        5900u16..6000,                           // port (VNC display range)
        arb_vnc_config(),
    )
        .prop_map(|(name, host, port, vnc_config)| {
            Connection::new(name, host, port, ProtocolConfig::Vnc(vnc_config))
        })
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: rustconn, Property 3: SSH Command Builder Correctness**
    // **Validates: Requirements 2.2, 2.3, 2.4, 2.5**
    //
    // For any valid SSH connection configuration (including auth method, proxy jump,
    // control master, and custom options), the built command must include all
    // specified parameters in the correct SSH command-line format.

    #[test]
    fn prop_ssh_command_includes_host(conn in arb_ssh_connection()) {
        let protocol = SshProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        // The host (or user@host) must appear in the arguments
        let has_host = args.iter().any(|a| a.contains(&conn.host));
        prop_assert!(has_host, "Command must contain host: {:?}", args);
    }

    #[test]
    fn prop_ssh_command_includes_custom_port(conn in arb_ssh_connection()) {
        let protocol = SshProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        // If port is not default (22), it must be specified with -p
        if conn.port != 22 {
            let has_port_flag = args.iter().any(|a| a == "-p");
            let has_port_value = args.iter().any(|a| a == &conn.port.to_string());
            prop_assert!(has_port_flag && has_port_value,
                "Non-default port {} must be specified with -p: {:?}", conn.port, args);
        }
    }

    #[test]
    fn prop_ssh_command_includes_proxy_jump(conn in arb_ssh_connection()) {
        let protocol = SshProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Ssh(ssh_config) = &conn.protocol_config {
            if let Some(proxy) = &ssh_config.proxy_jump {
                let has_jump_flag = args.iter().any(|a| a == "-J");
                let has_proxy = args.iter().any(|a| a == proxy);
                prop_assert!(has_jump_flag && has_proxy,
                    "ProxyJump {} must be specified with -J: {:?}", proxy, args);
            }
        }
    }

    #[test]
    fn prop_ssh_command_includes_control_master(conn in arb_ssh_connection()) {
        let protocol = SshProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Ssh(ssh_config) = &conn.protocol_config {
            if ssh_config.use_control_master {
                let has_control_master = args.iter().any(|a| a.contains("ControlMaster="));
                let has_control_persist = args.iter().any(|a| a.contains("ControlPersist="));
                let has_control_path = args.iter().any(|a| a.contains("ControlPath="));
                prop_assert!(has_control_master && has_control_persist && has_control_path,
                    "ControlMaster options must be present: {:?}", args);
            }
        }
    }

    #[test]
    fn prop_ssh_command_includes_custom_options(conn in arb_ssh_connection()) {
        let protocol = SshProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Ssh(ssh_config) = &conn.protocol_config {
            for (key, value) in &ssh_config.custom_options {
                let expected = format!("{key}={value}");
                let has_option = args.iter().any(|a| a == &expected);
                prop_assert!(has_option,
                    "Custom option {} must be present: {:?}", expected, args);
            }
        }
    }

    #[test]
    fn prop_ssh_command_includes_username(conn in arb_ssh_connection()) {
        let protocol = SshProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let Some(username) = &conn.username {
            let expected = format!("{username}@{}", conn.host);
            let has_user_host = args.iter().any(|a| a == &expected);
            prop_assert!(has_user_host,
                "Username must be in user@host format: {:?}", args);
        }
    }

    // **Feature: rustconn, Property 4: RDP Command Builder Correctness**
    // **Validates: Requirements 3.1, 3.2, 3.3, 3.5**
    //
    // For any valid RDP connection configuration (including resolution, color depth,
    // audio redirect, gateway, and custom client), the built command must use the
    // correct client binary and include all specified parameters.

    #[test]
    fn prop_rdp_command_includes_server(conn in arb_rdp_connection()) {
        let protocol = RdpProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        // Server must be specified with /v:
        let has_server = args.iter().any(|a| a.starts_with("/v:") && a.contains(&conn.host));
        prop_assert!(has_server, "Command must contain /v:host: {:?}", args);
    }

    #[test]
    fn prop_rdp_command_includes_resolution(conn in arb_rdp_connection()) {
        let protocol = RdpProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Rdp(rdp_config) = &conn.protocol_config {
            if let Some(res) = &rdp_config.resolution {
                let has_width = args.iter().any(|a| a == &format!("/w:{}", res.width));
                let has_height = args.iter().any(|a| a == &format!("/h:{}", res.height));
                prop_assert!(has_width && has_height,
                    "Resolution {}x{} must be specified: {:?}", res.width, res.height, args);
            }
        }
    }

    #[test]
    fn prop_rdp_command_includes_color_depth(conn in arb_rdp_connection()) {
        let protocol = RdpProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Rdp(rdp_config) = &conn.protocol_config {
            if let Some(depth) = rdp_config.color_depth {
                let has_depth = args.iter().any(|a| a == &format!("/bpp:{depth}"));
                prop_assert!(has_depth,
                    "Color depth {} must be specified: {:?}", depth, args);
            }
        }
    }

    #[test]
    fn prop_rdp_command_includes_audio(conn in arb_rdp_connection()) {
        let protocol = RdpProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Rdp(rdp_config) = &conn.protocol_config {
            if rdp_config.audio_redirect {
                let has_sound = args.iter().any(|a| a == "/sound");
                prop_assert!(has_sound, "Audio redirect must include /sound: {:?}", args);
            }
        }
    }

    #[test]
    fn prop_rdp_command_includes_gateway(conn in arb_rdp_connection()) {
        let protocol = RdpProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Rdp(rdp_config) = &conn.protocol_config {
            if let Some(gw) = &rdp_config.gateway {
                let has_gateway = args.iter().any(|a| a == &format!("/g:{}", gw.hostname));
                prop_assert!(has_gateway,
                    "Gateway {} must be specified: {:?}", gw.hostname, args);

                if let Some(gw_user) = &gw.username {
                    let has_gw_user = args.iter().any(|a| a == &format!("/gu:{gw_user}"));
                    prop_assert!(has_gw_user,
                        "Gateway username {} must be specified: {:?}", gw_user, args);
                }
            }
        }
    }

    #[test]
    fn prop_rdp_command_includes_custom_args(conn in arb_rdp_connection()) {
        let protocol = RdpProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Rdp(rdp_config) = &conn.protocol_config {
            for custom_arg in &rdp_config.custom_args {
                let has_arg = args.iter().any(|a| a == custom_arg);
                prop_assert!(has_arg,
                    "Custom arg {} must be present: {:?}", custom_arg, args);
            }
        }
    }

    // **Feature: rustconn, Property 5: VNC Command Builder Correctness**
    // **Validates: Requirements 4.1, 4.2, 4.3**
    //
    // For any valid VNC connection configuration (including client preference,
    // encoding, compression, and quality), the built command must use the correct
    // client binary and include all specified parameters.

    #[test]
    fn prop_vnc_command_includes_server(conn in arb_vnc_connection()) {
        let protocol = VncProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        // Server must contain the host
        let has_host = args.iter().any(|a| a.contains(&conn.host));
        prop_assert!(has_host, "Command must contain host: {:?}", args);
    }

    #[test]
    fn prop_vnc_command_includes_encoding(conn in arb_vnc_connection()) {
        let protocol = VncProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Vnc(vnc_config) = &conn.protocol_config {
            if let Some(encoding) = &vnc_config.encoding {
                // Check for either TightVNC or TigerVNC style
                let has_encoding = args.iter().any(|a| a == encoding);
                prop_assert!(has_encoding,
                    "Encoding {} must be present: {:?}", encoding, args);
            }
        }
    }

    #[test]
    fn prop_vnc_command_includes_compression(conn in arb_vnc_connection()) {
        let protocol = VncProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Vnc(vnc_config) = &conn.protocol_config {
            if let Some(compression) = vnc_config.compression {
                let has_compression = args.iter().any(|a| a == &compression.to_string());
                prop_assert!(has_compression,
                    "Compression {} must be present: {:?}", compression, args);
            }
        }
    }

    #[test]
    fn prop_vnc_command_includes_quality(conn in arb_vnc_connection()) {
        let protocol = VncProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Vnc(vnc_config) = &conn.protocol_config {
            if let Some(quality) = vnc_config.quality {
                let has_quality = args.iter().any(|a| a == &quality.to_string());
                prop_assert!(has_quality,
                    "Quality {} must be present: {:?}", quality, args);
            }
        }
    }

    #[test]
    fn prop_vnc_command_includes_custom_args(conn in arb_vnc_connection()) {
        let protocol = VncProtocol::new();
        let cmd = protocol.build_command(&conn, None).unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        if let ProtocolConfig::Vnc(vnc_config) = &conn.protocol_config {
            for custom_arg in &vnc_config.custom_args {
                let has_arg = args.iter().any(|a| a == custom_arg);
                prop_assert!(has_arg,
                    "Custom arg {} must be present: {:?}", custom_arg, args);
            }
        }
    }
}
