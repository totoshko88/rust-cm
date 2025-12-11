//! SSH protocol handler

use std::process::Command;

use crate::error::ProtocolError;
use crate::models::{Connection, Credentials, ProtocolConfig, SshAuthMethod, SshConfig};

use super::{Protocol, ProtocolResult};

/// SSH protocol handler
///
/// Implements the Protocol trait for SSH connections, building ssh commands
/// with support for various authentication methods, proxy jump, control master,
/// and custom options.
pub struct SshProtocol;

impl SshProtocol {
    /// Creates a new SSH protocol handler
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extracts SSH config from a connection, returning an error if not SSH
    fn get_ssh_config(connection: &Connection) -> ProtocolResult<&SshConfig> {
        match &connection.protocol_config {
            ProtocolConfig::Ssh(config) => Ok(config),
            _ => Err(ProtocolError::InvalidConfig(
                "Connection is not an SSH connection".to_string(),
            )),
        }
    }
}

impl Default for SshProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl Protocol for SshProtocol {
    fn protocol_id(&self) -> &'static str {
        "ssh"
    }

    fn display_name(&self) -> &'static str {
        "SSH"
    }

    fn default_port(&self) -> u16 {
        22
    }

    fn uses_embedded_terminal(&self) -> bool {
        true
    }

    fn build_command(
        &self,
        connection: &Connection,
        credentials: Option<&Credentials>,
    ) -> ProtocolResult<Command> {
        let ssh_config = Self::get_ssh_config(connection)?;

        let mut cmd = Command::new("ssh");

        // Add port if not default
        if connection.port != self.default_port() {
            cmd.arg("-p").arg(connection.port.to_string());
        }

        // Add authentication method specific options
        match &ssh_config.auth_method {
            SshAuthMethod::PublicKey => {
                if let Some(key_path) = &ssh_config.key_path {
                    cmd.arg("-i").arg(key_path);
                }
                // Disable password auth when using public key
                cmd.arg("-o").arg("PasswordAuthentication=no");
            }
            SshAuthMethod::Password => {
                // Prefer keyboard-interactive for password prompts
                cmd.arg("-o").arg("PreferredAuthentications=password,keyboard-interactive");
            }
            SshAuthMethod::KeyboardInteractive => {
                cmd.arg("-o").arg("PreferredAuthentications=keyboard-interactive");
            }
            SshAuthMethod::Agent => {
                // Use SSH agent - no special options needed, but disable password
                cmd.arg("-o").arg("PasswordAuthentication=no");
            }
        }

        // Add proxy jump if configured
        if let Some(proxy_jump) = &ssh_config.proxy_jump {
            cmd.arg("-J").arg(proxy_jump);
        }

        // Add control master options if enabled
        if ssh_config.use_control_master {
            cmd.arg("-o").arg("ControlMaster=auto");
            cmd.arg("-o").arg("ControlPersist=600");
            // Use a socket path based on connection details
            let socket_path = "/tmp/ssh-rustconn-%r@%h:%p".to_string();
            cmd.arg("-o").arg(format!("ControlPath={socket_path}"));
        }

        // Add custom options
        for (key, value) in &ssh_config.custom_options {
            cmd.arg("-o").arg(format!("{key}={value}"));
        }

        // Build the destination
        let destination = connection.username.as_ref().map_or_else(
            || {
                credentials
                    .and_then(|creds| creds.username.as_ref())
                    .map_or_else(
                        || connection.host.clone(),
                        |username| format!("{username}@{}", connection.host),
                    )
            },
            |username| format!("{username}@{}", connection.host),
        );

        cmd.arg(&destination);

        // Add startup command if configured
        if let Some(startup_cmd) = &ssh_config.startup_command {
            cmd.arg(startup_cmd);
        }

        Ok(cmd)
    }

    fn validate_connection(&self, connection: &Connection) -> ProtocolResult<()> {
        let ssh_config = Self::get_ssh_config(connection)?;

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

        // Validate key path exists if using public key auth
        if matches!(ssh_config.auth_method, SshAuthMethod::PublicKey) {
            if let Some(key_path) = &ssh_config.key_path {
                if !key_path.as_os_str().is_empty() && !key_path.exists() {
                    return Err(ProtocolError::InvalidConfig(format!(
                        "SSH key file not found: {}",
                        key_path.display()
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProtocolConfig;
    use std::collections::HashMap;

    fn create_ssh_connection(config: SshConfig) -> Connection {
        Connection::new(
            "Test SSH".to_string(),
            "example.com".to_string(),
            22,
            ProtocolConfig::Ssh(config),
        )
    }

    #[test]
    fn test_ssh_protocol_metadata() {
        let protocol = SshProtocol::new();
        assert_eq!(protocol.protocol_id(), "ssh");
        assert_eq!(protocol.display_name(), "SSH");
        assert_eq!(protocol.default_port(), 22);
        assert!(protocol.uses_embedded_terminal());
    }

    #[test]
    fn test_build_basic_ssh_command() {
        let protocol = SshProtocol::new();
        let connection = create_ssh_connection(SshConfig::default());

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain the host
        assert!(args.iter().any(|a| a.to_str() == Some("example.com")));
    }

    #[test]
    fn test_build_ssh_command_with_custom_port() {
        let protocol = SshProtocol::new();
        let mut connection = create_ssh_connection(SshConfig::default());
        connection.port = 2222;

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain -p 2222
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();
        assert!(args_str.contains(&"-p"));
        assert!(args_str.contains(&"2222"));
    }

    #[test]
    fn test_build_ssh_command_with_username() {
        let protocol = SshProtocol::new();
        let connection = create_ssh_connection(SshConfig::default())
            .with_username("admin");

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();

        // Should contain user@host
        assert!(args.iter().any(|a| a.to_str() == Some("admin@example.com")));
    }

    #[test]
    fn test_build_ssh_command_with_proxy_jump() {
        let protocol = SshProtocol::new();
        let config = SshConfig {
            proxy_jump: Some("bastion.example.com".to_string()),
            ..Default::default()
        };
        let connection = create_ssh_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        assert!(args_str.contains(&"-J"));
        assert!(args_str.contains(&"bastion.example.com"));
    }

    #[test]
    fn test_build_ssh_command_with_control_master() {
        let protocol = SshProtocol::new();
        let config = SshConfig {
            use_control_master: true,
            ..Default::default()
        };
        let connection = create_ssh_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        // Should contain ControlMaster options
        assert!(args_str.iter().any(|a| a.contains("ControlMaster=auto")));
        assert!(args_str.iter().any(|a| a.contains("ControlPersist=")));
        assert!(args_str.iter().any(|a| a.contains("ControlPath=")));
    }

    #[test]
    fn test_build_ssh_command_with_custom_options() {
        let protocol = SshProtocol::new();
        let mut custom_options = HashMap::new();
        custom_options.insert("ServerAliveInterval".to_string(), "60".to_string());
        custom_options.insert("StrictHostKeyChecking".to_string(), "no".to_string());

        let config = SshConfig {
            custom_options,
            ..Default::default()
        };
        let connection = create_ssh_connection(config);

        let cmd = protocol.build_command(&connection, None).unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        let args_str: Vec<_> = args.iter().map(|a| a.to_str().unwrap()).collect();

        // Should contain custom options
        assert!(args_str.iter().any(|a| a.contains("ServerAliveInterval=60")));
        assert!(args_str.iter().any(|a| a.contains("StrictHostKeyChecking=no")));
    }
}
