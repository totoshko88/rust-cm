//! Session manager for RustConn
//!
//! This module provides the SessionManager which handles the lifecycle
//! of active connection sessions, including starting, terminating,
//! and tracking sessions.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use uuid::Uuid;

use crate::error::SessionError;
use crate::models::{Connection, Credentials};
use crate::protocol::ProtocolRegistry;

use super::logger::SessionLogger;
use super::session::{Session, SessionState, SessionType};

/// Result type for session operations
pub type SessionResult<T> = Result<T, SessionError>;

/// Manages active connection sessions
///
/// The SessionManager is responsible for:
/// - Starting new sessions for connections
/// - Tracking active sessions
/// - Terminating sessions
/// - Managing session logging
pub struct SessionManager {
    /// Active sessions indexed by session ID
    sessions: HashMap<Uuid, Session>,
    /// Protocol registry for building commands
    protocol_registry: ProtocolRegistry,
    /// Session logger for recording terminal output
    logger: Option<SessionLogger>,
    /// Whether logging is enabled
    logging_enabled: bool,
}

impl SessionManager {
    /// Creates a new SessionManager
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            protocol_registry: ProtocolRegistry::new(),
            logger: None,
            logging_enabled: false,
        }
    }

    /// Creates a new SessionManager with logging enabled
    ///
    /// # Errors
    /// Returns an error if the logger cannot be initialized
    pub fn with_logging(log_dir: PathBuf) -> SessionResult<Self> {
        let logger = SessionLogger::new(log_dir)?;
        Ok(Self {
            sessions: HashMap::new(),
            protocol_registry: ProtocolRegistry::new(),
            logger: Some(logger),
            logging_enabled: true,
        })
    }

    /// Enables or disables session logging
    pub fn set_logging_enabled(&mut self, enabled: bool) {
        self.logging_enabled = enabled;
    }

    /// Sets the session logger
    pub fn set_logger(&mut self, logger: SessionLogger) {
        self.logger = Some(logger);
    }

    /// Starts a new session for a connection
    ///
    /// For SSH connections, this prepares the session but does not spawn
    /// the process - that should be done by the terminal widget.
    /// For RDP/VNC connections, this spawns the external client process.
    ///
    /// # Errors
    /// Returns an error if the session cannot be started
    pub fn start_session(
        &mut self,
        connection: &Connection,
        credentials: Option<&Credentials>,
    ) -> SessionResult<Uuid> {
        // Get the protocol handler
        let protocol = self
            .protocol_registry
            .get(&connection.protocol.to_string())
            .ok_or_else(|| {
                SessionError::StartFailed(format!(
                    "Unknown protocol: {}",
                    connection.protocol
                ))
            })?;

        // Validate the connection
        protocol.validate_connection(connection).map_err(|e| {
            SessionError::StartFailed(format!("Invalid connection configuration: {}", e))
        })?;

        // Determine session type
        let session_type = if protocol.uses_embedded_terminal() {
            SessionType::Embedded
        } else {
            SessionType::External
        };

        // Create the session
        let mut session = Session::new(
            connection.id,
            connection.name.clone(),
            protocol.protocol_id().to_string(),
            session_type,
        );

        // Set up logging if enabled
        if self.logging_enabled {
            if let Some(ref logger) = self.logger {
                match logger.create_log_file(connection.id, &connection.name) {
                    Ok(log_path) => session.set_log_file(log_path),
                    Err(e) => {
                        // Log error but don't fail the session
                        eprintln!("Warning: Failed to create log file: {}", e);
                    }
                }
            }
        }

        // For external sessions (RDP/VNC), spawn the process immediately
        if session_type == SessionType::External {
            let command = protocol.build_command(connection, credentials).map_err(|e| {
                SessionError::StartFailed(format!("Failed to build command: {}", e))
            })?;

            let child = spawn_external_process(command)?;
            session.set_process(child);
        }

        let session_id = session.id;
        self.sessions.insert(session_id, session);

        Ok(session_id)
    }

    /// Builds the command for an embedded terminal session
    ///
    /// This is used by the terminal widget to get the command to execute.
    ///
    /// # Errors
    /// Returns an error if the command cannot be built
    pub fn build_embedded_command(
        &self,
        connection: &Connection,
        credentials: Option<&Credentials>,
    ) -> SessionResult<Command> {
        let protocol = self
            .protocol_registry
            .get(&connection.protocol.to_string())
            .ok_or_else(|| {
                SessionError::StartFailed(format!(
                    "Unknown protocol: {}",
                    connection.protocol
                ))
            })?;

        protocol.build_command(connection, credentials).map_err(|e| {
            SessionError::StartFailed(format!("Failed to build command: {}", e))
        })
    }

    /// Sets the process handle for an embedded session
    ///
    /// This is called by the terminal widget after spawning the process.
    ///
    /// # Errors
    /// Returns an error if the session is not found
    pub fn set_session_process(
        &mut self,
        session_id: Uuid,
        process: std::process::Child,
    ) -> SessionResult<()> {
        let session = self.sessions.get_mut(&session_id).ok_or_else(|| {
            SessionError::NotFound(session_id.to_string())
        })?;

        session.set_process(process);
        Ok(())
    }

    /// Terminates a session
    ///
    /// # Errors
    /// Returns an error if the session cannot be terminated
    pub fn terminate_session(&mut self, session_id: Uuid) -> SessionResult<()> {
        let session = self.sessions.get_mut(&session_id).ok_or_else(|| {
            SessionError::NotFound(session_id.to_string())
        })?;

        // Terminate the process
        session.terminate().map_err(|e| {
            SessionError::TerminateFailed(format!("Failed to terminate process: {}", e))
        })?;

        // Finalize the log file
        if let Some(ref log_path) = session.log_file {
            if let Err(e) = SessionLogger::finalize_log(log_path) {
                eprintln!("Warning: Failed to finalize log file: {}", e);
            }
        }

        Ok(())
    }

    /// Force kills a session
    ///
    /// # Errors
    /// Returns an error if the session cannot be killed
    pub fn kill_session(&mut self, session_id: Uuid) -> SessionResult<()> {
        let session = self.sessions.get_mut(&session_id).ok_or_else(|| {
            SessionError::NotFound(session_id.to_string())
        })?;

        session.kill().map_err(|e| {
            SessionError::TerminateFailed(format!("Failed to kill process: {}", e))
        })?;

        // Finalize the log file
        if let Some(ref log_path) = session.log_file {
            if let Err(e) = SessionLogger::finalize_log(log_path) {
                eprintln!("Warning: Failed to finalize log file: {}", e);
            }
        }

        Ok(())
    }

    /// Removes a terminated session from tracking
    pub fn remove_session(&mut self, session_id: Uuid) -> Option<Session> {
        self.sessions.remove(&session_id)
    }

    /// Gets a reference to a session
    #[must_use]
    pub fn get_session(&self, session_id: Uuid) -> Option<&Session> {
        self.sessions.get(&session_id)
    }

    /// Gets a mutable reference to a session
    pub fn get_session_mut(&mut self, session_id: Uuid) -> Option<&mut Session> {
        self.sessions.get_mut(&session_id)
    }

    /// Returns all active sessions
    #[must_use]
    pub fn active_sessions(&self) -> Vec<&Session> {
        self.sessions
            .values()
            .filter(|s| s.state == SessionState::Active || s.state == SessionState::Starting)
            .collect()
    }

    /// Returns all sessions for a specific connection
    #[must_use]
    pub fn sessions_for_connection(&self, connection_id: Uuid) -> Vec<&Session> {
        self.sessions
            .values()
            .filter(|s| s.connection_id == connection_id)
            .collect()
    }

    /// Returns the number of active sessions
    #[must_use]
    pub fn active_session_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| s.state == SessionState::Active || s.state == SessionState::Starting)
            .count()
    }

    /// Checks and updates the state of all sessions
    ///
    /// This should be called periodically to detect terminated processes.
    pub fn refresh_session_states(&mut self) {
        for session in self.sessions.values_mut() {
            if session.state == SessionState::Active {
                let _ = session.is_running();
            }
        }
    }

    /// Cleans up terminated sessions
    ///
    /// Removes sessions that have been terminated from tracking.
    pub fn cleanup_terminated_sessions(&mut self) {
        self.sessions.retain(|_, session| {
            session.state != SessionState::Terminated && session.state != SessionState::Error
        });
    }

    /// Terminates all active sessions
    ///
    /// # Errors
    /// Returns the first error encountered, but attempts to terminate all sessions
    pub fn terminate_all(&mut self) -> SessionResult<()> {
        let session_ids: Vec<Uuid> = self.sessions.keys().copied().collect();
        let mut first_error: Option<SessionError> = None;

        for session_id in session_ids {
            if let Err(e) = self.terminate_session(session_id) {
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }

        if let Some(e) = first_error {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Returns a reference to the session logger
    #[must_use]
    pub fn logger(&self) -> Option<&SessionLogger> {
        self.logger.as_ref()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawns an external process for RDP/VNC connections
fn spawn_external_process(mut command: Command) -> SessionResult<std::process::Child> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| SessionError::StartFailed(format!("Failed to spawn process: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.active_session_count(), 0);
    }

    #[test]
    fn test_session_not_found() {
        let mut manager = SessionManager::new();
        let result = manager.terminate_session(Uuid::new_v4());
        assert!(result.is_err());
    }
}
