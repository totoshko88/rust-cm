//! Connection model representing a saved remote access configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::protocol::{ProtocolConfig, ProtocolType};

/// Source of password/credentials for a connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PasswordSource {
    /// No password stored
    #[default]
    None,
    /// Password stored in connection config (encrypted)
    Stored,
    /// Password retrieved from `KeePass` database
    KeePass,
    /// Password retrieved from system keyring (libsecret)
    Keyring,
    /// Prompt user for password on each connection
    Prompt,
}

/// A saved remote connection configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    /// Unique identifier for the connection
    pub id: Uuid,
    /// Human-readable name for the connection
    pub name: String,
    /// Protocol type (SSH, RDP, VNC)
    pub protocol: ProtocolType,
    /// Remote host address (hostname or IP)
    pub host: String,
    /// Remote port number
    pub port: u16,
    /// Username for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Group this connection belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<Uuid>,
    /// Tags for organization and filtering
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Timestamp when the connection was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the connection was last modified
    pub updated_at: DateTime<Utc>,
    /// Protocol-specific configuration
    pub protocol_config: ProtocolConfig,
    /// Sort order for manual ordering (lower values appear first)
    #[serde(default)]
    pub sort_order: i32,
    /// Timestamp when the connection was last used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_connected: Option<DateTime<Utc>>,
    /// Source of password for this connection
    #[serde(default)]
    pub password_source: PasswordSource,
    /// Domain for RDP/Windows authentication
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

impl Connection {
    /// Creates a new connection with the given parameters
    #[must_use]
    pub fn new(name: String, host: String, port: u16, protocol_config: ProtocolConfig) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            protocol: protocol_config.protocol_type(),
            host,
            port,
            username: None,
            group_id: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            protocol_config,
            sort_order: 0,
            last_connected: None,
            password_source: PasswordSource::None,
            domain: None,
        }
    }

    /// Creates a new SSH connection with default configuration
    #[must_use]
    pub fn new_ssh(name: String, host: String, port: u16) -> Self {
        Self::new(
            name,
            host,
            port,
            ProtocolConfig::Ssh(super::protocol::SshConfig::default()),
        )
    }

    /// Creates a new RDP connection with default configuration
    #[must_use]
    pub fn new_rdp(name: String, host: String, port: u16) -> Self {
        Self::new(
            name,
            host,
            port,
            ProtocolConfig::Rdp(super::protocol::RdpConfig::default()),
        )
    }

    /// Creates a new VNC connection with default configuration
    #[must_use]
    pub fn new_vnc(name: String, host: String, port: u16) -> Self {
        Self::new(
            name,
            host,
            port,
            ProtocolConfig::Vnc(super::protocol::VncConfig::default()),
        )
    }

    /// Creates a new SPICE connection with default configuration
    #[must_use]
    pub fn new_spice(name: String, host: String, port: u16) -> Self {
        Self::new(
            name,
            host,
            port,
            ProtocolConfig::Spice(super::protocol::SpiceConfig::default()),
        )
    }

    /// Sets the username for this connection
    #[must_use]
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the group for this connection
    #[must_use]
    pub const fn with_group(mut self, group_id: Uuid) -> Self {
        self.group_id = Some(group_id);
        self
    }

    /// Adds tags to this connection
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Updates the `updated_at` timestamp to now
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Returns the default port for this connection's protocol
    #[must_use]
    pub const fn default_port(&self) -> u16 {
        match self.protocol {
            ProtocolType::Ssh => 22,
            ProtocolType::Rdp => 3389,
            ProtocolType::Vnc | ProtocolType::Spice => 5900,
        }
    }
}
