//! Credential resolution chain for connections
//!
//! This module provides the `CredentialResolver` which resolves credentials
//! for connections based on their configuration and available backends.

use std::sync::Arc;

use crate::config::SecretSettings;
use crate::error::SecretResult;
use crate::models::{Connection, Credentials, PasswordSource};

use super::manager::SecretManager;

/// Resolves credentials for a connection based on configuration
///
/// The resolver implements a fallback chain that tries multiple credential
/// sources in order based on the connection's `password_source` setting
/// and the application's secret settings.
pub struct CredentialResolver {
    /// Secret manager for backend access
    secret_manager: Arc<SecretManager>,
    /// Secret settings for configuration
    settings: SecretSettings,
}

impl CredentialResolver {
    /// Creates a new `CredentialResolver`
    ///
    /// # Arguments
    /// * `secret_manager` - The secret manager with configured backends
    /// * `settings` - Secret settings for configuration
    #[must_use]
    pub const fn new(secret_manager: Arc<SecretManager>, settings: SecretSettings) -> Self {
        Self {
            secret_manager,
            settings,
        }
    }

    /// Generates a lookup key for `KeePass` entry retrieval
    ///
    /// The key format is: `rustconn/{name}` where name is the connection name.
    /// If the connection name is empty, falls back to using the host.
    ///
    /// # Arguments
    /// * `connection` - The connection to generate a key for
    ///
    /// # Returns
    /// A string key suitable for `KeePass` entry lookup
    #[must_use]
    pub fn generate_lookup_key(connection: &Connection) -> String {
        let identifier = if connection.name.trim().is_empty() {
            &connection.host
        } else {
            &connection.name
        };
        format!("rustconn/{identifier}")
    }

    /// Resolves credentials for a connection
    ///
    /// Resolution order based on `password_source`:
    /// 1. If `PasswordSource::KeePass` and `KeePass` integration active -> `KeePass` lookup
    /// 2. If `PasswordSource::Keyring` -> libsecret lookup
    /// 3. If `PasswordSource::Stored` -> return None (caller should use stored password)
    /// 4. If `PasswordSource::Prompt` -> return None (caller should prompt user)
    /// 5. If `PasswordSource::None` -> try fallback chain if enabled
    ///
    /// When the primary source fails and fallback is enabled, tries the next
    /// available source in the chain.
    ///
    /// # Arguments
    /// * `connection` - The connection to resolve credentials for
    ///
    /// # Returns
    /// `Some(Credentials)` if found from any source, `None` if not found
    ///
    /// # Errors
    /// Returns `SecretError` if backend operations fail
    pub async fn resolve(&self, connection: &Connection) -> SecretResult<Option<Credentials>> {
        match connection.password_source {
            PasswordSource::KeePass => {
                self.resolve_from_keepass(connection).await
            }
            PasswordSource::Keyring => {
                self.resolve_from_keyring(connection).await
            }
            PasswordSource::Stored | PasswordSource::Prompt => {
                // Caller handles these cases
                Ok(None)
            }
            PasswordSource::None => {
                // Try fallback chain if enabled
                if self.settings.enable_fallback {
                    self.resolve_with_fallback(connection).await
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Resolves credentials from `KeePass`
    async fn resolve_from_keepass(&self, connection: &Connection) -> SecretResult<Option<Credentials>> {
        if !self.settings.kdbx_enabled {
            // `KeePass` not enabled, try fallback if allowed
            if self.settings.enable_fallback {
                return self.resolve_from_keyring(connection).await;
            }
            return Ok(None);
        }

        let lookup_key = Self::generate_lookup_key(connection);
        let result = self.secret_manager.retrieve(&lookup_key).await?;

        if result.is_some() {
            return Ok(result);
        }

        // `KeePass` lookup failed, try fallback if enabled
        if self.settings.enable_fallback {
            self.resolve_from_keyring(connection).await
        } else {
            Ok(None)
        }
    }

    /// Resolves credentials from system keyring (libsecret)
    async fn resolve_from_keyring(&self, connection: &Connection) -> SecretResult<Option<Credentials>> {
        let connection_id = connection.id.to_string();
        self.secret_manager.retrieve(&connection_id).await
    }

    /// Resolves credentials using the fallback chain
    ///
    /// Tries sources in order: `KeePass` (if enabled) -> Keyring
    async fn resolve_with_fallback(&self, connection: &Connection) -> SecretResult<Option<Credentials>> {
        // Try `KeePass` first if enabled
        if self.settings.kdbx_enabled {
            let lookup_key = Self::generate_lookup_key(connection);
            if let Some(creds) = self.secret_manager.retrieve(&lookup_key).await? {
                return Ok(Some(creds));
            }
        }

        // Fall back to keyring
        self.resolve_from_keyring(connection).await
    }

    /// Checks if `KeePass` integration is currently active
    #[must_use]
    pub const fn is_keepass_active(&self) -> bool {
        self.settings.kdbx_enabled && self.settings.kdbx_path.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ProtocolConfig, ProtocolType, SshConfig};
    use uuid::Uuid;

    fn create_test_connection(name: &str, host: &str) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            name: name.to_string(),
            host: host.to_string(),
            port: 22,
            protocol: ProtocolType::Ssh,
            username: None,
            group_id: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            protocol_config: ProtocolConfig::Ssh(SshConfig::default()),
            sort_order: 0,
            last_connected: None,
            password_source: PasswordSource::None,
            domain: None,
        }
    }

    #[test]
    fn test_generate_lookup_key_with_name() {
        let conn = create_test_connection("My Server", "192.168.1.1");
        let key = CredentialResolver::generate_lookup_key(&conn);
        assert_eq!(key, "rustconn/My Server");
    }

    #[test]
    fn test_generate_lookup_key_with_empty_name() {
        let conn = create_test_connection("", "192.168.1.1");
        let key = CredentialResolver::generate_lookup_key(&conn);
        assert_eq!(key, "rustconn/192.168.1.1");
    }

    #[test]
    fn test_generate_lookup_key_with_whitespace_name() {
        let conn = create_test_connection("   ", "example.com");
        let key = CredentialResolver::generate_lookup_key(&conn);
        assert_eq!(key, "rustconn/example.com");
    }

    #[test]
    fn test_generate_lookup_key_contains_identifier() {
        let conn = create_test_connection("Production DB", "db.example.com");
        let key = CredentialResolver::generate_lookup_key(&conn);
        // Key should contain either name or host
        assert!(key.contains("Production DB") || key.contains("db.example.com"));
    }
}
