//! Secret backend trait definition
//!
//! This module defines the `SecretBackend` trait that all secret storage
//! implementations must implement.

use async_trait::async_trait;

use crate::error::SecretResult;
use crate::models::Credentials;

/// Abstraction over secret storage backends
///
/// This trait defines the interface for storing, retrieving, and deleting
/// credentials from various secret storage backends like `KeePassXC` or libsecret.
#[async_trait]
pub trait SecretBackend: Send + Sync {
    /// Store credentials for a connection
    ///
    /// # Arguments
    /// * `connection_id` - Unique identifier for the connection
    /// * `credentials` - The credentials to store
    ///
    /// # Errors
    /// Returns `SecretError` if the storage operation fails
    async fn store(&self, connection_id: &str, credentials: &Credentials) -> SecretResult<()>;

    /// Retrieve credentials for a connection
    ///
    /// # Arguments
    /// * `connection_id` - Unique identifier for the connection
    ///
    /// # Returns
    /// `Some(Credentials)` if found, `None` if not found
    ///
    /// # Errors
    /// Returns `SecretError` if the retrieval operation fails
    async fn retrieve(&self, connection_id: &str) -> SecretResult<Option<Credentials>>;

    /// Delete credentials for a connection
    ///
    /// # Arguments
    /// * `connection_id` - Unique identifier for the connection
    ///
    /// # Errors
    /// Returns `SecretError` if the deletion operation fails
    async fn delete(&self, connection_id: &str) -> SecretResult<()>;

    /// Check if the backend is available and operational
    ///
    /// # Returns
    /// `true` if the backend is available, `false` otherwise
    async fn is_available(&self) -> bool;

    /// Returns the backend identifier
    ///
    /// # Returns
    /// A static string identifying this backend (e.g., "keepassxc", "libsecret")
    fn backend_id(&self) -> &'static str;

    /// Returns a human-readable name for this backend
    ///
    /// # Returns
    /// A static string with the display name (e.g., "`KeePassXC`", "GNOME Keyring")
    fn display_name(&self) -> &'static str;
}
