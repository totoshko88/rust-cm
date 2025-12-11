//! VNC protocol handler

use std::path::PathBuf;
use std::process::Command;

use crate::error::ProtocolError;
use crate::models::{Connection, Credentials, ProtocolConfig, VncClient, VncConfig};

use super::{Protocol, ProtocolResult};

/// VNC protocol handler
///
/// Implements the Protocol trait for VNC connections, building commands for
/// `TightVNC`, `TigerVNC`, or custom VNC clients with support for encoding,
/// compression, and quality options.
pub struct VncProtocol;

impl VncProtocol {
    /// Creates a new VNC protocol handler
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extracts VNC config from a connection, returning an error if not VNC
    fn get_vnc_config(connection: &Connection) -> ProtocolResult<&VncConfig> {
        match &connection.protocol_config {
            ProtocolConfig::Vnc(config) => Ok(config),
            _ => Err(ProtocolError::InvalidConfig(
                "Connection is not a VNC connection".to_string(),
            )),
        }
    }

    /// Gets the client binary path based on configuration
    fn get_client_binary(client: &VncClient) -> PathBuf {
        match client {
            VncClient::TightVnc | VncClient::TigerVnc => PathBuf::from("vncviewer"),
            VncClient::Custom(path) => path.clone(),
        }
    }

    /// Checks if the client is `TigerVNC` (uses different argument format)
    const fn is_tigervnc(client: &VncClient) -> bool {
        matches!(client, VncClient::TigerVnc)
    }
}

impl Default for VncProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl Protocol for VncProtocol {
    fn protocol_id(&self) -> &'static str {
        "vnc"
    }

    fn display_name(&self) -> &'static str {
        "VNC"
    }

    fn default_port(&self) -> u16 {
        5900
    }

    fn uses_embedded_terminal(&self) -> bool {
        false
    }

    fn build_command(
        &self,
        connection: &Connection,
        _credentials: Option<&Credentials>,
    ) -> ProtocolResult<Command> {
        let vnc_config = Self::get_vnc_config(connection)?;

        let client_binary = Self::get_client_binary(&vnc_config.client);
        let mut cmd = Command::new(&client_binary);

        let is_tiger = Self::is_tigervnc(&vnc_config.client);

        // Add encoding if specified
        if let Some(encoding) = &vnc_config.encoding {
            if is_tiger {
                cmd.arg("-PreferredEncoding").arg(encoding);
            } else {
                // TightVNC style
                cmd.arg("-encoding").arg(encoding);
            }
        }

        // Add compression level if specified (0-9)
        if let Some(compression) = vnc_config.compression {
            if is_tiger {
                cmd.arg("-CompressLevel").arg(compression.to_string());
            } else {
                cmd.arg("-compresslevel").arg(compression.to_string());
            }
        }

        // Add quality level if specified (0-9)
        if let Some(quality) = vnc_config.quality {
            if is_tiger {
                cmd.arg("-QualityLevel").arg(quality.to_string());
            } else {
                cmd.arg("-quality").arg(quality.to_string());
            }
        }

        // Add custom arguments
        for arg in &vnc_config.custom_args {
            cmd.arg(arg);
        }

        // Add server address (host:display or host::port)
        // VNC uses display numbers (port = 5900 + display)
        // If port is 5900, use display 0; otherwise calculate display or use ::port format
        let server = if connection.port == self.default_port() {
            format!("{}:0", connection.host)
        } else if connection.port > 5900 && connection.port < 6000 {
            // Standard VNC display range
            let display = connection.port - 5900;
            format!("{}:{display}", connection.host)
        } else {
            // Non-standard port, use ::port format
            format!("{}::{}", connection.host, connection.port)
        };
        cmd.arg(&server);

        Ok(cmd)
    }

    fn validate_connection(&self, connection: &Connection) -> ProtocolResult<()> {
        let vnc_config = Self::get_vnc_config(connection)?;

        // Validate host is not empty
        if connection.host.is_empty() {
            return Err(ProtocolError::InvalidConfig(
                "Host cannot be empty".to_string(),
            ));
        }

        // Validate port is in valid range
        if connection.port == 0 {
            return Err(ProtocolError::InvalidConfig(
                "Port cannot be 0".to_string(),
            ));
        }

        // Validate compression level if specified (0-9)
        if let Some(compression) = vnc_config.compression {
            if compression > 9 {
                return Err(ProtocolError::InvalidConfig(format!(
                    "Invalid compression level: {compression}. Must be 0-9"
                )));
            }
        }

        // Validate quality level if specified (0-9)
        if let Some(quality) = vnc_config.quality {
            if quality > 9 {
                return Err(ProtocolError::InvalidConfig(format!(
                    "Invalid quality level: {quality}. Must be 0-9"
                )));
            }
        }

        // Validate custom client path exists if specified
        if let VncClient::Custom(path) = &vnc_config.client {
            if !path.as_os_str().is_empty() && !path.exists() {
                return Err(ProtocolError::ClientNotFound(path.clone()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProtocolConfig;

    fn create_vnc_connection(config: VncConfig) -> Connection {
        Connection::new(
            "Test VNC".to_string(),
            "vnc.example.com".to_string(),
            5900,
            ProtocolConfig::Vnc(config),
        )
    }

    #[test]
    fn test_vnc_protocol_metadata() {
        let protocol = VncProtocol::new();
        assert_eq!(protocol.protocol_id(), "vnc");
        assert_eq!(protocol.display_name(), "VNC");
        assert_eq!(protocol.default_port(), 5900);
        assert!(!protocol.uses_embedded_terminal());
    }

    #[test]
    fn test_build_basic_vnc_command() {
        let protocol = VncProtocol::new();
        let connection = create_vnc_connection(VncConfig::default());

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain host:0 for default port
        assert!(args.iter().any(|a| a.to_str() == Some("vnc.example.com:0")));
    }

    #[test]
    fn test_build_vnc_command_with_display() {
        let protocol = VncProtocol::new();
        let mut connection = create_vnc_connection(VncConfig::default());
        connection.port = 5901; // Display :1

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain host:1
        assert!(args.iter().any(|a| a.to_str() == Some("vnc.example.com:1")));
    }

    #[test]
    fn test_build_vnc_command_with_nonstandard_port() {
        let protocol = VncProtocol::new();
        let mut connection = create_vnc_connection(VncConfig::default());
        connection.port = 9999;

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain host::port format
        assert!(args.iter().any(|a| a.to_str() == Some("vnc.example.com::9999")));
    }

    #[test]
    fn test_build_vnc_command_with_encoding_tightvnc() {
        let protocol = VncProtocol::new();
        let config = VncConfig {
            client: VncClient::TightVnc,
            encoding: Some("tight".to_string()),
            ..Default::default()
        };
        let connection = create_vnc_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        assert!(args_str.contains(&"-encoding"));
        assert!(args_str.contains(&"tight"));
    }

    #[test]
    fn test_build_vnc_command_with_encoding_tigervnc() {
        let protocol = VncProtocol::new();
        let config = VncConfig {
            client: VncClient::TigerVnc,
            encoding: Some("zrle".to_string()),
            ..Default::default()
        };
        let connection = create_vnc_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        assert!(args_str.contains(&"-PreferredEncoding"));
        assert!(args_str.contains(&"zrle"));
    }

    #[test]
    fn test_build_vnc_command_with_compression() {
        let protocol = VncProtocol::new();
        let config = VncConfig {
            compression: Some(6),
            ..Default::default()
        };
        let connection = create_vnc_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        assert!(args_str.contains(&"-compresslevel"));
        assert!(args_str.contains(&"6"));
    }

    #[test]
    fn test_build_vnc_command_with_quality() {
        let protocol = VncProtocol::new();
        let config = VncConfig {
            quality: Some(8),
            ..Default::default()
        };
        let connection = create_vnc_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        assert!(args_str.contains(&"-quality"));
        assert!(args_str.contains(&"8"));
    }

    #[test]
    fn test_build_vnc_command_with_custom_args() {
        let protocol = VncProtocol::new();
        let config = VncConfig {
            custom_args: vec!["-fullscreen".to_string(), "-viewonly".to_string()],
            ..Default::default()
        };
        let connection = create_vnc_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        assert!(args.iter().any(|a| a.to_str() == Some("-fullscreen")));
        assert!(args.iter().any(|a| a.to_str() == Some("-viewonly")));
    }

    #[test]
    fn test_validate_invalid_compression() {
        let protocol = VncProtocol::new();
        let config = VncConfig {
            compression: Some(15), // Invalid: > 9
            ..Default::default()
        };
        let connection = create_vnc_connection(config);

        let result = protocol.validate_connection(&connection);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_quality() {
        let protocol = VncProtocol::new();
        let config = VncConfig {
            quality: Some(10), // Invalid: > 9
            ..Default::default()
        };
        let connection = create_vnc_connection(config);

        let result = protocol.validate_connection(&connection);
        assert!(result.is_err());
    }
}
