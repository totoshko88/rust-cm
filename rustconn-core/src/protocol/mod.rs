//! Protocol layer for `RustConn`
//!
//! This module provides the Protocol trait and implementations for SSH, RDP, and VNC protocols.
//! Each protocol handler is responsible for building the appropriate command to establish
//! a connection.

mod registry;
mod rdp;
mod ssh;
mod vnc;

pub use registry::ProtocolRegistry;
pub use rdp::RdpProtocol;
pub use ssh::SshProtocol;
pub use vnc::VncProtocol;

use std::process::Command;

use crate::error::ProtocolError;
use crate::models::{Connection, Credentials};

/// Result type for protocol operations
pub type ProtocolResult<T> = Result<T, ProtocolError>;

/// Core trait for all connection protocols
///
/// This trait defines the interface that all protocol handlers must implement.
/// It provides methods for building commands, validation, and protocol metadata.
pub trait Protocol: Send + Sync {
    /// Returns the protocol identifier (e.g., "ssh", "rdp", "vnc")
    fn protocol_id(&self) -> &'static str;

    /// Returns human-readable protocol name
    fn display_name(&self) -> &'static str;

    /// Returns default port for this protocol
    fn default_port(&self) -> u16;

    /// Whether this protocol uses embedded terminal (SSH) or external window (RDP/VNC)
    fn uses_embedded_terminal(&self) -> bool;

    /// Builds command line arguments for the connection
    ///
    /// # Arguments
    /// * `connection` - The connection configuration
    /// * `credentials` - Optional credentials for authentication
    ///
    /// # Returns
    /// A `Command` ready to be executed, or a `ProtocolError` if the command cannot be built
    ///
    /// # Errors
    /// Returns `ProtocolError` if the command cannot be built due to invalid configuration
    fn build_command(
        &self,
        connection: &Connection,
        credentials: Option<&Credentials>,
    ) -> ProtocolResult<Command>;

    /// Validates connection configuration for this protocol
    ///
    /// # Arguments
    /// * `connection` - The connection to validate
    ///
    /// # Returns
    /// `Ok(())` if valid, or a `ProtocolError` describing the validation failure
    ///
    /// # Errors
    /// Returns `ProtocolError` if the connection configuration is invalid
    fn validate_connection(&self, connection: &Connection) -> ProtocolResult<()>;
}
