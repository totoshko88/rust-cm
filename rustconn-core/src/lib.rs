//! `RustConn` Core Library
//!
//! This crate provides the core functionality for the `RustConn` connection manager,
//! including connection management, protocol handling, configuration, and import capabilities.

pub mod config;
pub mod connection;
pub mod dialog_utils;
pub mod error;
pub mod ffi;
pub mod import;
pub mod models;
pub mod progress;
pub mod protocol;
pub mod secret;
pub mod session;
pub mod snippet;
pub mod split_view;

pub use config::{AppSettings, ConfigManager};
pub use connection::ConnectionManager;
pub use error::{
    ConfigError, ConfigResult, ImportError, ProtocolError, RustConnError, SecretError, SessionError,
};
pub use ffi::{ConnectionState, FfiDisplay, FfiError, FfiResult, VncCredentialType, VncDisplay, VncError};
pub use import::{
    AnsibleInventoryImporter, AsbruImporter, ImportResult, ImportSource, RemminaImporter,
    SkippedEntry, SshConfigImporter,
};
pub use models::{
    Connection, ConnectionGroup, Credentials, ProtocolConfig, RdpConfig, RdpGateway,
    Resolution, Snippet, SnippetVariable, SpiceConfig, SpiceImageCompression, SshAuthMethod,
    SshConfig, VncConfig,
};
pub use progress::{
    CallbackProgressReporter, CancelHandle, LocalProgressReporter, NoOpProgressReporter,
    ProgressReporter,
};
pub use protocol::{
    detect_rdp_client, detect_ssh_client, detect_vnc_client, ClientDetectionResult, ClientInfo,
    Protocol, ProtocolRegistry, RdpProtocol, SshProtocol, VncProtocol,
};
pub use secret::{
    parse_keepassxc_version, CredentialResolver, KdbxExporter, KeePassStatus, KeePassXcBackend,
    LibSecretBackend, SecretBackend, SecretManager,
};
pub use session::{Session, SessionLogger, SessionManager, SessionState, SessionType};
pub use snippet::SnippetManager;
pub use split_view::{PaneModel, SessionInfo, SplitDirection, SplitViewModel};
