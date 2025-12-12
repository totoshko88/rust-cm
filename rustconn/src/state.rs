//! Application state management
//!
//! This module provides the central application state that holds all managers
//! and provides thread-safe access to core functionality.

use rustconn_core::{
    AppSettings, ConfigManager, Connection, ConnectionGroup, ConnectionManager, CredentialResolver,
    Credentials, ImportResult, SecretManager, Session, SessionManager, Snippet, SnippetManager,
};
use rustconn_core::models::PasswordSource;
use secrecy::SecretString;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use uuid::Uuid;

/// Cached credentials for a connection (session-only, not persisted)
#[derive(Clone)]
pub struct CachedCredentials {
    /// Username
    pub username: String,
    /// Password (stored securely in memory)
    pub password: SecretString,
    /// Domain for Windows authentication
    pub domain: String,
}

/// Application state holding all managers
///
/// This struct provides centralized access to all core functionality
/// and is shared across the application using Rc<`RefCell`<>>.
pub struct AppState {
    /// Connection manager for CRUD operations
    connection_manager: ConnectionManager,
    /// Session manager for active connections
    session_manager: SessionManager,
    /// Snippet manager for command snippets
    snippet_manager: SnippetManager,
    /// Secret manager for credentials (infrastructure for future keyring integration)
    #[allow(dead_code)]
    secret_manager: SecretManager,
    /// Configuration manager for persistence
    config_manager: ConfigManager,
    /// Application settings
    settings: AppSettings,
    /// Session-level password cache (cleared on app exit)
    password_cache: HashMap<Uuid, CachedCredentials>,
}

impl AppState {
    /// Creates a new application state
    ///
    /// Initializes all managers and loads configuration from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub fn new() -> Result<Self, String> {
        // Initialize config manager
        let config_manager = ConfigManager::new()
            .map_err(|e| format!("Failed to initialize config manager: {e}"))?;

        // Load settings
        let settings = config_manager
            .load_settings()
            .unwrap_or_else(|_| AppSettings::default());

        // Initialize connection manager
        let connection_manager = ConnectionManager::new(config_manager.clone())
            .map_err(|e| format!("Failed to initialize connection manager: {e}"))?;

        // Initialize session manager with logging if enabled
        let session_manager = if settings.logging.enabled {
            let log_dir = if settings.logging.log_directory.is_absolute() {
                settings.logging.log_directory.clone()
            } else {
                config_manager
                    .config_dir()
                    .join(&settings.logging.log_directory)
            };
            SessionManager::with_logging(log_dir).unwrap_or_else(|_| SessionManager::new())
        } else {
            SessionManager::new()
        };

        // Initialize snippet manager
        let snippet_manager = SnippetManager::new(config_manager.clone())
            .map_err(|e| format!("Failed to initialize snippet manager: {e}"))?;

        // Initialize secret manager (empty for now, backends added later)
        let secret_manager = SecretManager::empty();

        Ok(Self {
            connection_manager,
            session_manager,
            snippet_manager,
            secret_manager,
            config_manager,
            settings,
            password_cache: HashMap::new(),
        })
    }

    // ========== Password Cache Operations ==========

    /// Caches credentials for a connection (session-only)
    pub fn cache_credentials(
        &mut self,
        connection_id: Uuid,
        username: &str,
        password: &str,
        domain: &str,
    ) {
        self.password_cache.insert(
            connection_id,
            CachedCredentials {
                username: username.to_string(),
                password: SecretString::new(password.to_string()),
                domain: domain.to_string(),
            },
        );
    }

    /// Gets cached credentials for a connection
    pub fn get_cached_credentials(&self, connection_id: Uuid) -> Option<&CachedCredentials> {
        self.password_cache.get(&connection_id)
    }

    /// Checks if credentials are cached for a connection
    #[allow(dead_code)]
    pub fn has_cached_credentials(&self, connection_id: Uuid) -> bool {
        self.password_cache.contains_key(&connection_id)
    }

    /// Clears cached credentials for a connection
    #[allow(dead_code)]
    pub fn clear_cached_credentials(&mut self, connection_id: Uuid) {
        self.password_cache.remove(&connection_id);
    }

    /// Clears all cached credentials
    #[allow(dead_code)]
    pub fn clear_all_cached_credentials(&mut self) {
        self.password_cache.clear();
    }

    // ========== Connection Operations ==========

    /// Creates a new connection
    pub fn create_connection(&mut self, connection: Connection) -> Result<Uuid, String> {
        // Check for duplicate name
        if self.connection_exists_by_name(&connection.name) {
            return Err(format!(
                "Connection with name '{}' already exists",
                connection.name
            ));
        }

        self.connection_manager
            .create_connection_from(connection)
            .map_err(|e| format!("Failed to create connection: {e}"))
    }

    /// Checks if a connection with the given name exists
    pub fn connection_exists_by_name(&self, name: &str) -> bool {
        self.connection_manager
            .list_connections()
            .iter()
            .any(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// Checks if a group with the given name exists
    pub fn group_exists_by_name(&self, name: &str) -> bool {
        self.connection_manager
            .list_groups()
            .iter()
            .any(|g| g.name.eq_ignore_ascii_case(name))
    }

    /// Generates a unique name by appending a number if needed
    pub fn generate_unique_connection_name(&self, base_name: &str) -> String {
        if !self.connection_exists_by_name(base_name) {
            return base_name.to_string();
        }

        let mut counter = 1;
        loop {
            let new_name = format!("{base_name} ({counter})");
            if !self.connection_exists_by_name(&new_name) {
                return new_name;
            }
            counter += 1;
        }
    }

    /// Generates a unique group name by appending a number if needed
    pub fn generate_unique_group_name(&self, base_name: &str) -> String {
        if !self.group_exists_by_name(base_name) {
            return base_name.to_string();
        }

        let mut counter = 1;
        loop {
            let new_name = format!("{base_name} ({counter})");
            if !self.group_exists_by_name(&new_name) {
                return new_name;
            }
            counter += 1;
        }
    }

    /// Updates an existing connection
    pub fn update_connection(&mut self, id: Uuid, connection: Connection) -> Result<(), String> {
        self.connection_manager
            .update_connection(id, connection)
            .map_err(|e| format!("Failed to update connection: {e}"))
    }

    /// Deletes a connection
    pub fn delete_connection(&mut self, id: Uuid) -> Result<(), String> {
        self.connection_manager
            .delete_connection(id)
            .map_err(|e| format!("Failed to delete connection: {e}"))
    }

    /// Gets a connection by ID
    pub fn get_connection(&self, id: Uuid) -> Option<&Connection> {
        self.connection_manager.get_connection(id)
    }

    /// Lists all connections
    pub fn list_connections(&self) -> Vec<&Connection> {
        self.connection_manager.list_connections()
    }

    /// Searches connections
    pub fn search_connections(&self, query: &str) -> Vec<&Connection> {
        self.connection_manager.search(query)
    }

    /// Gets connections by group
    pub fn get_connections_by_group(&self, group_id: Uuid) -> Vec<&Connection> {
        self.connection_manager.get_by_group(group_id)
    }

    /// Gets ungrouped connections
    pub fn get_ungrouped_connections(&self) -> Vec<&Connection> {
        self.connection_manager.get_ungrouped()
    }

    // ========== Group Operations ==========

    /// Creates a new group
    pub fn create_group(&mut self, name: String) -> Result<Uuid, String> {
        // Check for duplicate name
        if self.group_exists_by_name(&name) {
            return Err(format!("Group with name '{name}' already exists"));
        }

        self.connection_manager
            .create_group(name)
            .map_err(|e| format!("Failed to create group: {e}"))
    }

    /// Creates a group with a parent
    pub fn create_group_with_parent(
        &mut self,
        name: String,
        parent_id: Uuid,
    ) -> Result<Uuid, String> {
        self.connection_manager
            .create_group_with_parent(name, parent_id)
            .map_err(|e| format!("Failed to create group: {e}"))
    }

    /// Deletes a group (connections become ungrouped)
    pub fn delete_group(&mut self, id: Uuid) -> Result<(), String> {
        self.connection_manager
            .delete_group(id)
            .map_err(|e| format!("Failed to delete group: {e}"))
    }

    /// Deletes a group and all connections within it (cascade delete)
    pub fn delete_group_cascade(&mut self, id: Uuid) -> Result<(), String> {
        self.connection_manager
            .delete_group_cascade(id)
            .map_err(|e| format!("Failed to delete group: {e}"))
    }

    /// Counts connections in a group (including child groups)
    pub fn count_connections_in_group(&self, group_id: Uuid) -> usize {
        self.connection_manager.count_connections_in_group(group_id)
    }

    /// Gets a group by ID
    pub fn get_group(&self, id: Uuid) -> Option<&ConnectionGroup> {
        self.connection_manager.get_group(id)
    }

    /// Lists all groups
    pub fn list_groups(&self) -> Vec<&ConnectionGroup> {
        self.connection_manager.list_groups()
    }

    /// Gets root-level groups
    pub fn get_root_groups(&self) -> Vec<&ConnectionGroup> {
        self.connection_manager.get_root_groups()
    }

    /// Gets child groups
    pub fn get_child_groups(&self, parent_id: Uuid) -> Vec<&ConnectionGroup> {
        self.connection_manager.get_child_groups(parent_id)
    }

    /// Moves a connection to a group
    pub fn move_connection_to_group(
        &mut self,
        connection_id: Uuid,
        group_id: Option<Uuid>,
    ) -> Result<(), String> {
        self.connection_manager
            .move_connection_to_group(connection_id, group_id)
            .map_err(|e| format!("Failed to move connection: {e}"))
    }

    /// Gets the group path
    pub fn get_group_path(&self, group_id: Uuid) -> Option<String> {
        self.connection_manager.get_group_path(group_id)
    }

    /// Updates the sort order of a connection
    #[allow(dead_code)]
    pub fn update_connection_sort_order(
        &mut self,
        connection_id: Uuid,
        sort_order: i32,
    ) -> Result<(), String> {
        if let Some(conn) = self.connection_manager.get_connection(connection_id) {
            let mut updated = conn.clone();
            updated.sort_order = sort_order;
            self.connection_manager
                .update_connection(connection_id, updated)
                .map_err(|e| format!("Failed to update sort order: {e}"))
        } else {
            Err(format!("Connection not found: {connection_id}"))
        }
    }

    /// Updates the sort order of a group
    #[allow(dead_code)]
    pub fn update_group_sort_order(
        &mut self,
        group_id: Uuid,
        sort_order: i32,
    ) -> Result<(), String> {
        if let Some(group) = self.connection_manager.get_group(group_id) {
            let mut updated = group.clone();
            updated.sort_order = sort_order;
            self.connection_manager
                .update_group(group_id, updated)
                .map_err(|e| format!("Failed to update sort order: {e}"))
        } else {
            Err(format!("Group not found: {group_id}"))
        }
    }

    /// Reorders connections by updating their `sort_order` values
    #[allow(dead_code)]
    pub fn reorder_connections(&mut self, connection_ids: &[Uuid]) -> Result<(), String> {
        for (index, &id) in connection_ids.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            self.update_connection_sort_order(id, index as i32)?;
        }
        Ok(())
    }

    /// Sorts connections within a specific group alphabetically
    pub fn sort_group(&mut self, group_id: Uuid) -> Result<(), String> {
        self.connection_manager
            .sort_group(group_id)
            .map_err(|e| format!("Failed to sort group: {e}"))
    }

    /// Sorts all groups and connections alphabetically
    pub fn sort_all(&mut self) -> Result<(), String> {
        self.connection_manager
            .sort_all()
            .map_err(|e| format!("Failed to sort all: {e}"))
    }

    /// Reorders a connection to be positioned after another connection
    pub fn reorder_connection(
        &mut self,
        connection_id: Uuid,
        target_id: Uuid,
    ) -> Result<(), String> {
        self.connection_manager
            .reorder_connection(connection_id, target_id)
            .map_err(|e| format!("Failed to reorder connection: {e}"))
    }

    /// Reorders a group to be positioned after another group
    pub fn reorder_group(&mut self, group_id: Uuid, target_id: Uuid) -> Result<(), String> {
        self.connection_manager
            .reorder_group(group_id, target_id)
            .map_err(|e| format!("Failed to reorder group: {e}"))
    }

    /// Updates the `last_connected` timestamp for a connection
    pub fn update_last_connected(&mut self, connection_id: Uuid) -> Result<(), String> {
        self.connection_manager
            .update_last_connected(connection_id)
            .map_err(|e| format!("Failed to update last connected: {e}"))
    }

    /// Sorts all connections by `last_connected` timestamp (most recent first)
    pub fn sort_by_recent(&mut self) -> Result<(), String> {
        self.connection_manager
            .sort_by_recent()
            .map_err(|e| format!("Failed to sort by recent: {e}"))
    }

    // ========== Session Operations ==========

    /// Starts a session for a connection
    #[allow(dead_code)]
    pub fn start_session(
        &mut self,
        connection_id: Uuid,
        _credentials: Option<&Credentials>,
    ) -> Result<Uuid, String> {
        let connection = self
            .connection_manager
            .get_connection(connection_id)
            .ok_or_else(|| format!("Connection not found: {connection_id}"))?
            .clone();

        self.session_manager
            .start_session(&connection)
            .map_err(|e| format!("Failed to start session: {e}"))
    }

    /// Terminates a session
    pub fn terminate_session(&mut self, session_id: Uuid) -> Result<(), String> {
        self.session_manager
            .terminate_session(session_id)
            .map_err(|e| format!("Failed to terminate session: {e}"))
    }

    /// Gets a session by ID
    #[allow(dead_code)]
    pub fn get_session(&self, session_id: Uuid) -> Option<&Session> {
        self.session_manager.get_session(session_id)
    }

    /// Gets active sessions
    pub fn active_sessions(&self) -> Vec<&Session> {
        self.session_manager.active_sessions()
    }

    /// Gets the session manager (for building commands)
    pub const fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Gets mutable session manager
    #[allow(dead_code)]
    pub fn session_manager_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }

    // ========== Snippet Operations ==========

    /// Creates a new snippet
    pub fn create_snippet(&mut self, snippet: Snippet) -> Result<Uuid, String> {
        self.snippet_manager
            .create_snippet_from(snippet)
            .map_err(|e| format!("Failed to create snippet: {e}"))
    }

    /// Updates a snippet
    pub fn update_snippet(&mut self, id: Uuid, snippet: Snippet) -> Result<(), String> {
        self.snippet_manager
            .update_snippet(id, snippet)
            .map_err(|e| format!("Failed to update snippet: {e}"))
    }

    /// Deletes a snippet
    pub fn delete_snippet(&mut self, id: Uuid) -> Result<(), String> {
        self.snippet_manager
            .delete_snippet(id)
            .map_err(|e| format!("Failed to delete snippet: {e}"))
    }

    /// Gets a snippet by ID
    pub fn get_snippet(&self, id: Uuid) -> Option<&Snippet> {
        self.snippet_manager.get_snippet(id)
    }

    /// Lists all snippets
    pub fn list_snippets(&self) -> Vec<&Snippet> {
        self.snippet_manager.list_snippets()
    }

    /// Searches snippets
    pub fn search_snippets(&self, query: &str) -> Vec<&Snippet> {
        self.snippet_manager.search(query)
    }

    // ========== Secret/Credential Operations ==========

    /// Gets a reference to the secret manager
    #[allow(dead_code)]
    pub const fn secret_manager(&self) -> &SecretManager {
        &self.secret_manager
    }

    /// Gets a mutable reference to the secret manager
    #[allow(dead_code)]
    pub fn secret_manager_mut(&mut self) -> &mut SecretManager {
        &mut self.secret_manager
    }

    /// Stores credentials for a connection (blocking wrapper for async operation)
    ///
    /// Note: This uses tokio's `block_on` to run the async operation synchronously.
    /// For better performance in async contexts, use `secret_manager()` directly.
    #[allow(dead_code)]
    pub fn store_credentials(
        &self,
        connection_id: Uuid,
        credentials: &Credentials,
    ) -> Result<(), String> {
        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {e}"))?;

        rt.block_on(async {
            self.secret_manager
                .store(&connection_id.to_string(), credentials)
                .await
                .map_err(|e| format!("Failed to store credentials: {e}"))
        })
    }

    /// Retrieves credentials for a connection (blocking wrapper for async operation)
    ///
    /// Note: This uses tokio's `block_on` to run the async operation synchronously.
    /// For better performance in async contexts, use `secret_manager()` directly.
    #[allow(dead_code)]
    pub fn retrieve_credentials(&self, connection_id: Uuid) -> Result<Option<Credentials>, String> {
        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {e}"))?;

        rt.block_on(async {
            self.secret_manager
                .retrieve(&connection_id.to_string())
                .await
                .map_err(|e| format!("Failed to retrieve credentials: {e}"))
        })
    }

    /// Deletes credentials for a connection (blocking wrapper for async operation)
    ///
    /// Note: This uses tokio's `block_on` to run the async operation synchronously.
    /// For better performance in async contexts, use `secret_manager()` directly.
    #[allow(dead_code)]
    pub fn delete_credentials(&self, connection_id: Uuid) -> Result<(), String> {
        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {e}"))?;

        rt.block_on(async {
            self.secret_manager
                .delete(&connection_id.to_string())
                .await
                .map_err(|e| format!("Failed to delete credentials: {e}"))
        })
    }

    /// Checks if any secret backend is available (blocking wrapper)
    #[allow(dead_code)]
    pub fn has_secret_backend(&self) -> bool {
        let Ok(rt) = tokio::runtime::Runtime::new() else {
            return false;
        };

        rt.block_on(async { self.secret_manager.is_available().await })
    }

    /// Resolves credentials for a connection using the credential resolution chain
    ///
    /// This method implements the credential resolution flow based on the connection's
    /// `password_source` setting:
    /// - `PasswordSource::KeePass` - Try `KeePass` first, fallback if enabled
    /// - `PasswordSource::Keyring` - Try system keyring (libsecret)
    /// - `PasswordSource::Stored` - Return None (caller uses stored password)
    /// - `PasswordSource::Prompt` - Return None (caller prompts user)
    /// - `PasswordSource::None` - Try fallback chain if enabled
    ///
    /// # Returns
    /// - `Ok(Some(Credentials))` - Credentials found from a backend
    /// - `Ok(None)` - No credentials found, caller should prompt user or use stored
    /// - `Err(String)` - Error during resolution
    pub fn resolve_credentials(&self, connection: &Connection) -> Result<Option<Credentials>, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {e}"))?;

        let resolver = CredentialResolver::new(
            Arc::new(SecretManager::empty()), // Use empty manager for now
            self.settings.secrets.clone(),
        );

        rt.block_on(async {
            resolver
                .resolve(connection)
                .await
                .map_err(|e| format!("Failed to resolve credentials: {e}"))
        })
    }

    /// Resolves credentials for a connection by ID
    ///
    /// Convenience method that looks up the connection and resolves credentials.
    pub fn resolve_credentials_for_connection(&self, connection_id: Uuid) -> Result<Option<Credentials>, String> {
        let connection = self
            .get_connection(connection_id)
            .ok_or_else(|| format!("Connection not found: {connection_id}"))?
            .clone();

        self.resolve_credentials(&connection)
    }

    /// Determines if credentials should be prompted for a connection
    ///
    /// Returns `true` if the connection's password source requires user input
    /// and no credentials are available from other sources.
    pub fn should_prompt_for_credentials(&self, connection: &Connection) -> bool {
        match connection.password_source {
            PasswordSource::Prompt => true,
            PasswordSource::Stored => false, // Use stored password
            PasswordSource::None => {
                // Check if fallback is enabled and backends are available
                !self.settings.secrets.enable_fallback || !self.has_secret_backend()
            }
            PasswordSource::KeePass => {
                // Prompt if KeePass is not enabled
                !self.settings.secrets.kdbx_enabled
            }
            PasswordSource::Keyring => {
                // Prompt if no backend available
                !self.has_secret_backend()
            }
        }
    }

    /// Checks if `KeePass` integration is currently active
    #[allow(dead_code)]
    pub const fn is_keepass_active(&self) -> bool {
        self.settings.secrets.kdbx_enabled && self.settings.secrets.kdbx_path.is_some()
    }

    // ========== Settings Operations ==========

    /// Gets the current settings
    pub const fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Updates and saves settings
    pub fn update_settings(&mut self, settings: AppSettings) -> Result<(), String> {
        self.config_manager
            .save_settings(&settings)
            .map_err(|e| format!("Failed to save settings: {e}"))?;

        // Update session manager logging
        if settings.logging.enabled != self.settings.logging.enabled {
            self.session_manager
                .set_logging_enabled(settings.logging.enabled);
        }

        self.settings = settings;
        Ok(())
    }

    /// Gets the config manager
    #[allow(dead_code)]
    pub const fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }

    /// Gets the connection manager
    pub fn connection_manager(&mut self) -> &mut ConnectionManager {
        &mut self.connection_manager
    }

    // ========== Import Operations ==========

    /// Imports connections from an import result with automatic group creation
    ///
    /// Creates a parent group for the import source (e.g., "Remmina Import", "SSH Config Import")
    /// and organizes connections into subgroups based on their original grouping.
    pub fn import_connections_with_source(
        &mut self,
        result: &ImportResult,
        source_name: &str,
    ) -> Result<usize, String> {
        let mut imported = 0;

        // Create parent group for this import source
        // Use generate_unique_group_name to handle duplicate names
        let base_group_name = format!("{source_name} Import");
        let parent_group_name = self.generate_unique_group_name(&base_group_name);
        let parent_group_id = match self.connection_manager.create_group(parent_group_name) {
            Ok(id) => Some(id),
            Err(_) => {
                // Group might already exist, try to find it
                self.connection_manager
                    .list_groups()
                    .iter()
                    .find(|g| g.name == base_group_name)
                    .map(|g| g.id)
            }
        };

        // Create a map for subgroups - maps OLD group UUID to NEW group UUID
        let mut group_uuid_map: std::collections::HashMap<Uuid, Uuid> =
            std::collections::HashMap::new();
        // Also keep name-based map for Remmina groups
        let mut subgroup_map: std::collections::HashMap<String, Uuid> =
            std::collections::HashMap::new();

        // Import groups from result as subgroups
        for group in &result.groups {
            let new_group_id = if let Some(parent_id) = parent_group_id {
                match self
                    .connection_manager
                    .create_group_with_parent(group.name.clone(), parent_id)
                {
                    Ok(id) => Some(id),
                    Err(_) => {
                        // Try to find existing
                        self.connection_manager
                            .get_child_groups(parent_id)
                            .iter()
                            .find(|g| g.name == group.name)
                            .map(|g| g.id)
                    }
                }
            } else {
                self.connection_manager.create_group(group.name.clone()).ok()
            };

            if let Some(new_id) = new_group_id {
                // Map old group UUID to new group UUID
                group_uuid_map.insert(group.id, new_id);
                subgroup_map.insert(group.name.clone(), new_id);
            }
        }

        // Import connections with automatic conflict resolution
        for conn in &result.connections {
            let mut connection = conn.clone();

            // Check for Remmina group tag (format: "remmina:group_name")
            let remmina_group = connection
                .tags
                .iter()
                .find(|t| t.starts_with("remmina:"))
                .map(|t| t.strip_prefix("remmina:").unwrap_or("").to_string());

            // Remove the remmina group tag from tags
            connection.tags.retain(|t| !t.starts_with("remmina:"));

            // Determine target group
            let target_group_id = if let Some(group_name) = remmina_group {
                // Create subgroup for Remmina group if not exists
                if !subgroup_map.contains_key(&group_name) {
                    if let Some(parent_id) = parent_group_id {
                        if let Ok(id) = self
                            .connection_manager
                            .create_group_with_parent(group_name.clone(), parent_id)
                        {
                            subgroup_map.insert(group_name.clone(), id);
                        }
                    }
                }
                subgroup_map.get(&group_name).copied()
            } else if let Some(existing_group_id) = connection.group_id {
                // Connection has a group from import - map to new UUID
                group_uuid_map
                    .get(&existing_group_id)
                    .copied()
                    .or(parent_group_id)
            } else {
                // Use parent import group
                parent_group_id
            };

            // Set the group
            connection.group_id = target_group_id;

            // Auto-resolve name conflicts
            if self.connection_exists_by_name(&connection.name) {
                connection.name = self.generate_unique_connection_name(&connection.name);
            }

            match self.connection_manager.create_connection_from(connection) {
                Ok(_) => imported += 1,
                Err(e) => eprintln!("Warning: Failed to import connection {}: {}", conn.name, e),
            }
        }

        Ok(imported)
    }

    /// Imports connections from an import result (legacy method)
    #[allow(dead_code)]
    pub fn import_connections(&mut self, result: &ImportResult) -> Result<usize, String> {
        self.import_connections_with_source(result, "Unknown")
    }
}

/// Shared application state type
pub type SharedAppState = Rc<RefCell<AppState>>;

/// Creates a new shared application state
pub fn create_shared_state() -> Result<SharedAppState, String> {
    AppState::new().map(|state| Rc::new(RefCell::new(state)))
}
