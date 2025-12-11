//! RDP protocol handler

use std::path::PathBuf;
use std::process::Command;

use crate::error::ProtocolError;
use crate::models::{Connection, Credentials, ProtocolConfig, RdpClient, RdpConfig};

use super::{Protocol, ProtocolResult};

/// RDP protocol handler
///
/// Implements the Protocol trait for RDP connections, building `FreeRDP` (xfreerdp)
/// commands with support for resolution, color depth, audio redirection, gateway,
/// and custom client binaries.
pub struct RdpProtocol;

impl RdpProtocol {
    /// Creates a new RDP protocol handler
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extracts RDP config from a connection, returning an error if not RDP
    fn get_rdp_config(connection: &Connection) -> ProtocolResult<&RdpConfig> {
        match &connection.protocol_config {
            ProtocolConfig::Rdp(config) => Ok(config),
            _ => Err(ProtocolError::InvalidConfig(
                "Connection is not an RDP connection".to_string(),
            )),
        }
    }

    /// Gets the client binary path based on configuration
    fn get_client_binary(client: &RdpClient) -> PathBuf {
        match client {
            RdpClient::FreeRdp => PathBuf::from("xfreerdp"),
            RdpClient::Custom(path) => path.clone(),
        }
    }
}

impl Default for RdpProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl Protocol for RdpProtocol {
    fn protocol_id(&self) -> &'static str {
        "rdp"
    }

    fn display_name(&self) -> &'static str {
        "RDP"
    }

    fn default_port(&self) -> u16 {
        3389
    }

    fn uses_embedded_terminal(&self) -> bool {
        false
    }

    fn build_command(
        &self,
        connection: &Connection,
        credentials: Option<&Credentials>,
    ) -> ProtocolResult<Command> {
        let rdp_config = Self::get_rdp_config(connection)?;

        let client_binary = Self::get_client_binary(&rdp_config.client);
        let mut cmd = Command::new(&client_binary);

        // Add server address with port
        let server = if connection.port == self.default_port() {
            format!("/v:{}", connection.host)
        } else {
            format!("/v:{}:{}", connection.host, connection.port)
        };
        cmd.arg(&server);

        // Add username
        if let Some(username) = &connection.username {
            cmd.arg(format!("/u:{username}"));
        } else if let Some(creds) = credentials {
            if let Some(username) = &creds.username {
                cmd.arg(format!("/u:{username}"));
            }
        }

        // Add resolution if specified
        if let Some(resolution) = &rdp_config.resolution {
            cmd.arg(format!("/w:{}", resolution.width));
            cmd.arg(format!("/h:{}", resolution.height));
        }

        // Add color depth if specified
        if let Some(depth) = rdp_config.color_depth {
            cmd.arg(format!("/bpp:{depth}"));
        }

        // Add audio redirection
        if rdp_config.audio_redirect {
            cmd.arg("/sound");
        }

        // Add gateway configuration
        if let Some(gateway) = &rdp_config.gateway {
            cmd.arg(format!("/g:{}", gateway.hostname));
            if gateway.port != 443 {
                cmd.arg(format!("/gp:{}", gateway.port));
            }
            if let Some(gw_username) = &gateway.username {
                cmd.arg(format!("/gu:{gw_username}"));
            }
        }

        // Add custom arguments
        for arg in &rdp_config.custom_args {
            cmd.arg(arg);
        }

        Ok(cmd)
    }

    fn validate_connection(&self, connection: &Connection) -> ProtocolResult<()> {
        let rdp_config = Self::get_rdp_config(connection)?;

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

        // Validate color depth if specified
        if let Some(depth) = rdp_config.color_depth {
            if !matches!(depth, 8 | 15 | 16 | 24 | 32) {
                return Err(ProtocolError::InvalidConfig(format!(
                    "Invalid color depth: {depth}. Must be 8, 15, 16, 24, or 32"
                )));
            }
        }

        // Validate custom client path exists if specified
        if let RdpClient::Custom(path) = &rdp_config.client {
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
    use crate::models::{ProtocolConfig, RdpGateway, Resolution};

    fn create_rdp_connection(config: RdpConfig) -> Connection {
        Connection::new(
            "Test RDP".to_string(),
            "windows.example.com".to_string(),
            3389,
            ProtocolConfig::Rdp(config),
        )
    }

    #[test]
    fn test_rdp_protocol_metadata() {
        let protocol = RdpProtocol::new();
        assert_eq!(protocol.protocol_id(), "rdp");
        assert_eq!(protocol.display_name(), "RDP");
        assert_eq!(protocol.default_port(), 3389);
        assert!(!protocol.uses_embedded_terminal());
    }

    #[test]
    fn test_build_basic_rdp_command() {
        let protocol = RdpProtocol::new();
        let connection = create_rdp_connection(RdpConfig::default());

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain the server
        assert!(args.iter().any(|a| a.to_str().unwrap().contains("/v:windows.example.com")));
    }

    #[test]
    fn test_build_rdp_command_with_custom_port() {
        let protocol = RdpProtocol::new();
        let mut connection = create_rdp_connection(RdpConfig::default());
        connection.port = 3390;

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain server:port
        assert!(args.iter().any(|a| a.to_str().unwrap().contains("/v:windows.example.com:3390")));
    }

    #[test]
    fn test_build_rdp_command_with_username() {
        let protocol = RdpProtocol::new();
        let connection = create_rdp_connection(RdpConfig::default())
            .with_username("administrator");

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain username
        assert!(args.iter().any(|a| a.to_str().unwrap().contains("/u:administrator")));
    }

    #[test]
    fn test_build_rdp_command_with_resolution() {
        let protocol = RdpProtocol::new();
        let config = RdpConfig {
            resolution: Some(Resolution::new(1920, 1080)),
            ..Default::default()
        };
        let connection = create_rdp_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        assert!(args_str.iter().any(|a| a.contains("/w:1920")));
        assert!(args_str.iter().any(|a| a.contains("/h:1080")));
    }

    #[test]
    fn test_build_rdp_command_with_color_depth() {
        let protocol = RdpProtocol::new();
        let config = RdpConfig {
            color_depth: Some(24),
            ..Default::default()
        };
        let connection = create_rdp_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        assert!(args.iter().any(|a| a.to_str().unwrap().contains("/bpp:24")));
    }

    #[test]
    fn test_build_rdp_command_with_audio() {
        let protocol = RdpProtocol::new();
        let config = RdpConfig {
            audio_redirect: true,
            ..Default::default()
        };
        let connection = create_rdp_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        assert!(args.iter().any(|a| a.to_str() == Some("/sound")));
    }

    #[test]
    fn test_build_rdp_command_with_gateway() {
        let protocol = RdpProtocol::new();
        let config = RdpConfig {
            gateway: Some(RdpGateway {
                hostname: "gateway.example.com".to_string(),
                port: 443,
                username: Some("gwuser".to_string()),
            }),
            ..Default::default()
        };
        let connection = create_rdp_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        assert!(args_str.iter().any(|a| a.contains("/g:gateway.example.com")));
        assert!(args_str.iter().any(|a| a.contains("/gu:gwuser")));
    }

    #[test]
    fn test_build_rdp_command_with_custom_args() {
        let protocol = RdpProtocol::new();
        let config = RdpConfig {
            custom_args: vec!["/cert-ignore".to_string(), "/clipboard".to_string()],
            ..Default::default()
        };
        let connection = create_rdp_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        assert!(args.iter().any(|a| a.to_str() == Some("/cert-ignore")));
        assert!(args.iter().any(|a| a.to_str() == Some("/clipboard")));
    }
}
