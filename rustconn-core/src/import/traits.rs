//! Import source trait and result types.
//!
//! This module defines the core abstractions for the import engine,
//! allowing different import sources to be implemented uniformly.

use crate::error::ImportError;
use crate::models::{Connection, ConnectionGroup};

/// Result of an import operation containing successful imports and any issues encountered.
#[derive(Debug, Default)]
pub struct ImportResult {
    /// Successfully imported connections
    pub connections: Vec<Connection>,
    /// Successfully imported or created groups
    pub groups: Vec<ConnectionGroup>,
    /// Entries that were skipped (invalid but non-fatal)
    pub skipped: Vec<SkippedEntry>,
    /// Errors encountered during import
    pub errors: Vec<ImportError>,
}

impl ImportResult {
    /// Creates a new empty import result
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of entries processed
    #[must_use]
    pub fn total_processed(&self) -> usize {
        self.connections.len() + self.skipped.len() + self.errors.len()
    }

    /// Returns true if the import had any errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns true if any entries were skipped
    #[must_use]
    pub fn has_skipped(&self) -> bool {
        !self.skipped.is_empty()
    }

    /// Returns a summary string of the import result
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Imported: {}, Groups: {}, Skipped: {}, Errors: {}",
            self.connections.len(),
            self.groups.len(),
            self.skipped.len(),
            self.errors.len()
        )
    }

    /// Adds a connection to the result
    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    /// Adds a group to the result
    pub fn add_group(&mut self, group: ConnectionGroup) {
        self.groups.push(group);
    }

    /// Adds a skipped entry to the result
    pub fn add_skipped(&mut self, entry: SkippedEntry) {
        self.skipped.push(entry);
    }

    /// Adds an error to the result
    pub fn add_error(&mut self, error: ImportError) {
        self.errors.push(error);
    }

    /// Merges another import result into this one
    pub fn merge(&mut self, other: ImportResult) {
        self.connections.extend(other.connections);
        self.groups.extend(other.groups);
        self.skipped.extend(other.skipped);
        self.errors.extend(other.errors);
    }
}

/// An entry that was skipped during import
#[derive(Debug, Clone)]
pub struct SkippedEntry {
    /// Identifier or name of the skipped entry
    pub identifier: String,
    /// Reason why the entry was skipped
    pub reason: String,
    /// Source location (file path, line number, etc.)
    pub location: Option<String>,
}

impl SkippedEntry {
    /// Creates a new skipped entry
    #[must_use]
    pub fn new(identifier: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            reason: reason.into(),
            location: None,
        }
    }

    /// Creates a new skipped entry with location information
    #[must_use]
    pub fn with_location(
        identifier: impl Into<String>,
        reason: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            reason: reason.into(),
            location: Some(location.into()),
        }
    }
}

/// Trait for import source implementations.
///
/// Each import source (SSH config, Asbru-CM, Remmina, Ansible) implements
/// this trait to provide a uniform interface for importing connections.
pub trait ImportSource: Send + Sync {
    /// Returns the unique identifier for this import source
    fn source_id(&self) -> &'static str;

    /// Returns a human-readable name for this import source
    fn display_name(&self) -> &'static str;

    /// Checks if this import source is available (e.g., config files exist)
    fn is_available(&self) -> bool;

    /// Returns the default paths where this source looks for configuration
    fn default_paths(&self) -> Vec<std::path::PathBuf>;

    /// Imports connections from the source
    ///
    /// # Errors
    ///
    /// Returns an error if the import fails completely (e.g., file not found).
    /// Partial failures (invalid entries) are recorded in the `ImportResult`.
    fn import(&self) -> Result<ImportResult, ImportError>;

    /// Imports connections from a specific path
    ///
    /// # Errors
    ///
    /// Returns an error if the import fails completely.
    fn import_from_path(&self, path: &std::path::Path) -> Result<ImportResult, ImportError>;
}
