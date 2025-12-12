//! Session logging functionality
//!
//! This module provides session logging capabilities for recording
//! terminal output to timestamped log files.

use chrono::Utc;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::SessionError;

/// Session logger for writing terminal output to files
pub struct SessionLogger {
    /// Base directory for log files
    log_dir: PathBuf,
    /// Maximum number of log files to retain per connection
    max_logs_per_connection: usize,
    /// Maximum age of log files in days (0 = no limit)
    max_log_age_days: u32,
}

impl SessionLogger {
    /// Creates a new session logger with the specified log directory
    ///
    /// # Errors
    /// Returns an error if the log directory cannot be created
    pub fn new(log_dir: PathBuf) -> Result<Self, SessionError> {
        // Ensure log directory exists
        fs::create_dir_all(&log_dir).map_err(|e| {
            SessionError::LoggingError(format!("Failed to create log directory: {e}"))
        })?;

        Ok(Self {
            log_dir,
            max_logs_per_connection: 10,
            max_log_age_days: 30,
        })
    }

    /// Sets the maximum number of log files to retain per connection
    #[must_use]
    pub const fn with_max_logs(mut self, max_logs: usize) -> Self {
        self.max_logs_per_connection = max_logs;
        self
    }

    /// Sets the maximum age of log files in days
    #[must_use]
    pub const fn with_max_age_days(mut self, days: u32) -> Self {
        self.max_log_age_days = days;
        self
    }

    /// Creates a new log file for a session
    ///
    /// Returns the path to the created log file.
    ///
    /// # Errors
    /// Returns an error if the log file cannot be created
    pub fn create_log_file(
        &self,
        connection_id: Uuid,
        connection_name: &str,
    ) -> Result<PathBuf, SessionError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let safe_name = sanitize_filename(connection_name);
        let filename = format!("{safe_name}_{timestamp}.log");

        // Create connection-specific subdirectory
        let conn_dir = self.log_dir.join(connection_id.to_string());
        fs::create_dir_all(&conn_dir).map_err(|e| {
            SessionError::LoggingError(format!("Failed to create connection log directory: {e}"))
        })?;

        let log_path = conn_dir.join(filename);

        // Create the log file with header
        let mut file = File::create(&log_path)
            .map_err(|e| SessionError::LoggingError(format!("Failed to create log file: {e}")))?;

        // Write log header
        writeln!(file, "# RustConn Session Log").map_err(|e| {
            SessionError::LoggingError(format!("Failed to write log header: {e}"))
        })?;
        writeln!(
            file,
            "# Connection: {connection_name} ({connection_id})"
        )
        .map_err(|e| SessionError::LoggingError(format!("Failed to write log header: {e}")))?;
        writeln!(file, "# Started: {}", Utc::now().to_rfc3339()).map_err(|e| {
            SessionError::LoggingError(format!("Failed to write log header: {e}"))
        })?;
        writeln!(file, "#").map_err(|e| {
            SessionError::LoggingError(format!("Failed to write log header: {e}"))
        })?;
        writeln!(file).map_err(|e| {
            SessionError::LoggingError(format!("Failed to write log header: {e}"))
        })?;

        // Clean up old logs
        self.cleanup_old_logs(&conn_dir)?;

        Ok(log_path)
    }

    /// Opens an existing log file for appending
    ///
    /// # Errors
    /// Returns an error if the log file cannot be opened
    pub fn open_log_file(path: &Path) -> Result<BufWriter<File>, SessionError> {
        let file = OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| SessionError::LoggingError(format!("Failed to open log file: {e}")))?;

        Ok(BufWriter::new(file))
    }

    /// Writes data to a log file
    ///
    /// # Errors
    /// Returns an error if writing fails
    pub fn write_to_log(writer: &mut BufWriter<File>, data: &[u8]) -> Result<(), SessionError> {
        writer
            .write_all(data)
            .map_err(|e| SessionError::LoggingError(format!("Failed to write to log: {e}")))?;
        writer
            .flush()
            .map_err(|e| SessionError::LoggingError(format!("Failed to flush log: {e}")))?;
        Ok(())
    }

    /// Finalizes a log file with an end marker
    ///
    /// # Errors
    /// Returns an error if writing fails
    pub fn finalize_log(path: &Path) -> Result<(), SessionError> {
        let mut file = OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| SessionError::LoggingError(format!("Failed to open log file: {e}")))?;

        writeln!(file).map_err(|e| {
            SessionError::LoggingError(format!("Failed to write log footer: {e}"))
        })?;
        writeln!(file, "#").map_err(|e| {
            SessionError::LoggingError(format!("Failed to write log footer: {e}"))
        })?;
        writeln!(file, "# Session ended: {}", Utc::now().to_rfc3339()).map_err(|e| {
            SessionError::LoggingError(format!("Failed to write log footer: {e}"))
        })?;

        Ok(())
    }

    /// Cleans up old log files based on retention settings
    fn cleanup_old_logs(&self, conn_dir: &Path) -> Result<(), SessionError> {
        let entries: Vec<_> = fs::read_dir(conn_dir)
            .map_err(|e| {
                SessionError::LoggingError(format!("Failed to read log directory: {e}"))
            })?
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
            .collect();

        // Sort by modification time (newest first)
        let mut log_files: Vec<_> = entries
            .iter()
            .filter_map(|e| {
                let metadata = e.metadata().ok()?;
                let modified = metadata.modified().ok()?;
                Some((e.path(), modified))
            })
            .collect();

        log_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess logs
        if self.max_logs_per_connection > 0 && log_files.len() > self.max_logs_per_connection {
            for (path, _) in log_files.iter().skip(self.max_logs_per_connection) {
                let _ = fs::remove_file(path);
            }
        }

        // Remove old logs by age
        if self.max_log_age_days > 0 {
            let cutoff = std::time::SystemTime::now()
                - std::time::Duration::from_secs(u64::from(self.max_log_age_days) * 24 * 60 * 60);

            for (path, modified) in &log_files {
                if *modified < cutoff {
                    let _ = fs::remove_file(path);
                }
            }
        }

        Ok(())
    }

    /// Returns the log directory path
    #[must_use]
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    /// Lists all log files for a connection
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read
    pub fn list_logs(&self, connection_id: Uuid) -> Result<Vec<PathBuf>, SessionError> {
        let conn_dir = self.log_dir.join(connection_id.to_string());

        if !conn_dir.exists() {
            return Ok(Vec::new());
        }

        let mut logs: Vec<_> = fs::read_dir(&conn_dir)
            .map_err(|e| {
                SessionError::LoggingError(format!("Failed to read log directory: {e}"))
            })?
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
            .map(|e| e.path())
            .collect();

        logs.sort_by(|a, b| b.cmp(a)); // Newest first
        Ok(logs)
    }
}

impl Default for SessionLogger {
    fn default() -> Self {
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustconn")
            .join("logs");

        Self::new(log_dir).unwrap_or_else(|_| Self {
            log_dir: PathBuf::from("./logs"),
            max_logs_per_connection: 10,
            max_log_age_days: 30,
        })
    }
}

/// Sanitizes a filename by removing or replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .chars()
        .take(64) // Limit length
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test-server"), "test-server");
        assert_eq!(sanitize_filename("test server"), "test_server");
        assert_eq!(sanitize_filename("test/server"), "test_server");
        assert_eq!(sanitize_filename("test:server"), "test_server");
    }
}
