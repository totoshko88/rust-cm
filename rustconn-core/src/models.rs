//! Core data models for `RustConn`
//!
//! This module defines the primary data structures used throughout `RustConn`,
//! including connections, groups, credentials, and snippets.

mod connection;
mod credentials;
mod group;
mod protocol;
mod snippet;

pub use connection::{Connection, PasswordSource};
pub use credentials::Credentials;
pub use group::ConnectionGroup;
pub use protocol::{
    ProtocolConfig, ProtocolType, RdpConfig, RdpGateway, Resolution, SharedFolder,
    SpiceConfig, SpiceImageCompression, SshAuthMethod, SshConfig, VncConfig,
};
pub use snippet::{Snippet, SnippetVariable};
