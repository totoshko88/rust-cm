//! Core data models for `RustConn`
//!
//! This module defines the primary data structures used throughout `RustConn`,
//! including connections, groups, credentials, and snippets.

mod connection;
mod credentials;
mod group;
mod protocol;
mod snippet;

pub use connection::Connection;
pub use credentials::Credentials;
pub use group::ConnectionGroup;
pub use protocol::{
    ProtocolConfig, ProtocolType, RdpClient, RdpConfig, RdpGateway, Resolution, SshAuthMethod,
    SshConfig, VncClient, VncConfig,
};
pub use snippet::{Snippet, SnippetVariable};
