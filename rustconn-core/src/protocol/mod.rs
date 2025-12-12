//! Protocol layer for `RustConn`
//!
//! This module provides the Protocol trait and implementations for SSH, RDP, VNC, and SPICE protocols.
//! Each protocol handler is responsible for validation and protocol metadata.
//! Native session widgets will be implemented in later phases.

mod detection;
mod rdp;
mod registry;
mod spice;
mod ssh;
mod vnc;

pub use detection::{
    detect_rdp_client, detect_ssh_client, detect_vnc_client, ClientDetectionResult, ClientInfo,
};
pub use rdp::RdpProtocol;
pub use registry::ProtocolRegistry;
pub use spice::SpiceProtocol;
pub use ssh::SshProtocol;
pub use vnc::VncProtocol;

use crate::error::ProtocolError;
use crate::models::Connection;

/// Result type for protocol operations
pub type ProtocolResult<T> = Result<T, ProtocolError>;

/// Core trait for all connection protocols
///
/// This trait defines the interface that all protocol handlers must implement.
/// It provides methods for validation and protocol metadata.
/// 
/// Note: Native session widget creation will be added in Phase 5-7.
pub trait Protocol: Send + Sync {
    /// Returns the protocol identifier (e.g., "ssh", "rdp", "vnc")
    fn protocol_id(&self) -> &'static str;

    /// Returns human-readable protocol name
    fn display_name(&self) -> &'static str;

    /// Returns default port for this protocol
    fn default_port(&self) -> u16;

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
