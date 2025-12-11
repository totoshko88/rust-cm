//! `RustConn` Core Library
//!
//! This crate provides the core functionality for the `RustConn` connection manager,
//! including connection management, protocol handling, configuration, and import capabilities.

pub mod config;
pub mod connection;
pub mod dialog_utils;
pub mod error;
pub mod import;
pub mod models;
pub mod protocol;
pub mod secret;
pub mod session;
pub mod snippet;

pub use config::{AppSettings, ConfigManager};
pub use connection::ConnectionManager;
pub use error::{
    ConfigError, ConfigResult, ImportError, ProtocolError, RustConnError, SecretError,
    SessionError,
};
pub use import::{
    AnsibleInventoryImporter, AsbruImporter, ImportResult, ImportSource, RemminaImporter,
    SkippedEntry, SshConfigImporter,
};
pub use models::{
    Connection, ConnectionGroup, Credentials, ProtocolConfig, RdpClient, RdpConfig, RdpGateway,
    Resolution, Snippet, SnippetVariable, SshAuthMethod, SshConfig, VncClient, VncConfig,
};
pub use protocol::{Protocol, ProtocolRegistry, RdpProtocol, SshProtocol, VncProtocol};
pub use secret::{KdbxExporter, KeePassXcBackend, LibSecretBackend, SecretBackend, SecretManager};
pub use session::{Session, SessionLogger, SessionManager, SessionState, SessionType};
pub use snippet::SnippetManager;
