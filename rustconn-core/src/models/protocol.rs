//! Protocol configuration types for SSH, RDP, and VNC connections.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Protocol type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    /// SSH protocol
    Ssh,
    /// RDP protocol
    Rdp,
    /// VNC protocol
    Vnc,
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ssh => write!(f, "SSH"),
            Self::Rdp => write!(f, "RDP"),
            Self::Vnc => write!(f, "VNC"),
        }
    }
}

/// Protocol-specific configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolConfig {
    /// SSH protocol configuration
    Ssh(SshConfig),
    /// RDP protocol configuration
    Rdp(RdpConfig),
    /// VNC protocol configuration
    Vnc(VncConfig),
}

impl ProtocolConfig {
    /// Returns the protocol type for this configuration
    #[must_use]
    pub const fn protocol_type(&self) -> ProtocolType {
        match self {
            Self::Ssh(_) => ProtocolType::Ssh,
            Self::Rdp(_) => ProtocolType::Rdp,
            Self::Vnc(_) => ProtocolType::Vnc,
        }
    }
}


/// SSH authentication method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SshAuthMethod {
    /// Password authentication
    #[default]
    Password,
    /// Public key authentication
    PublicKey,
    /// Keyboard-interactive authentication
    KeyboardInteractive,
    /// SSH agent authentication
    Agent,
}

/// SSH protocol configuration
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshConfig {
    /// Authentication method
    #[serde(default)]
    pub auth_method: SshAuthMethod,
    /// Path to SSH private key file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_path: Option<PathBuf>,
    /// `ProxyJump` configuration (host or user@host)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_jump: Option<String>,
    /// Enable SSH `ControlMaster` for connection multiplexing
    #[serde(default)]
    pub use_control_master: bool,
    /// Custom SSH options (key-value pairs)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_options: HashMap<String, String>,
    /// Command to execute on connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_command: Option<String>,
}

/// RDP client type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RdpClient {
    /// `FreeRDP` (xfreerdp)
    #[default]
    FreeRdp,
    /// Custom RDP client binary
    Custom(PathBuf),
}

/// Screen resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl Resolution {
    /// Creates a new resolution
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// RDP gateway configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RdpGateway {
    /// Gateway hostname
    pub hostname: String,
    /// Gateway port (default: 443)
    #[serde(default = "default_gateway_port")]
    pub port: u16,
    /// Gateway username (if different from connection username)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

const fn default_gateway_port() -> u16 {
    443
}

/// RDP protocol configuration
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RdpConfig {
    /// RDP client to use
    #[serde(default)]
    pub client: RdpClient,
    /// Screen resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    /// Color depth (8, 15, 16, 24, or 32)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_depth: Option<u8>,
    /// Enable audio redirection
    #[serde(default)]
    pub audio_redirect: bool,
    /// RDP gateway configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<RdpGateway>,
    /// Custom command-line arguments
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_args: Vec<String>,
}

/// VNC client type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VncClient {
    /// `TightVNC` viewer
    #[default]
    TightVnc,
    /// `TigerVNC` viewer
    TigerVnc,
    /// Custom VNC client binary
    Custom(PathBuf),
}

/// VNC protocol configuration
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VncConfig {
    /// VNC client to use
    #[serde(default)]
    pub client: VncClient,
    /// Preferred encoding (e.g., "tight", "zrle", "hextile")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    /// Compression level (0-9)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<u8>,
    /// Quality level (0-9)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
    /// Custom command-line arguments
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_args: Vec<String>,
}
