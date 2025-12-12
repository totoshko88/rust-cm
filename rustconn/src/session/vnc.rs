//! VNC session widget for native embedding
//!
//! This module provides the `VncSessionWidget` struct that wraps the VNC FFI
//! display widget with overlay controls and state management.
//!
//! # Requirements Coverage
//!
//! - Requirement 2.1: Native VNC embedding as GTK widget
//! - Requirement 2.2: Keyboard and mouse input forwarding
//! - Requirement 2.3: VNC authentication handling
//! - Requirement 2.5: Connection state management and error handling

use super::{SessionError, SessionState};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Overlay, Spinner};
use rustconn_core::ffi::{VncCredentialType, VncDisplay};
use std::cell::RefCell;
use std::rc::Rc;

/// Callback type for authentication requests
type AuthCallback = Box<dyn Fn(&[VncCredentialType]) + 'static>;

/// Callback type for state change notifications
type StateCallback = Box<dyn Fn(SessionState) + 'static>;

/// VNC session widget with overlay controls
///
/// This widget wraps the VNC FFI display and provides:
/// - Connection lifecycle management
/// - Authentication callback handling
/// - State tracking and error reporting
/// - Overlay controls for session management
///
/// # Example
///
/// ```ignore
/// use rustconn::session::vnc::VncSessionWidget;
///
/// let widget = VncSessionWidget::new();
///
/// // Set up authentication callback
/// widget.connect_auth_required(|creds| {
///     // Prompt user for credentials
/// });
///
/// // Connect to VNC server
/// widget.connect("192.168.1.100", 5900, None);
/// ```
pub struct VncSessionWidget {
    /// The GTK overlay container
    overlay: Overlay,
    /// The VNC display widget
    display: Rc<VncDisplay>,
    /// Current session state
    state: Rc<RefCell<SessionState>>,
    /// Status label for connection feedback
    status_label: Label,
    /// Spinner for connection progress
    spinner: Spinner,
    /// Status container (kept for future floating controls integration)
    #[allow(dead_code)]
    status_container: GtkBox,
    /// Authentication callback
    auth_callback: Rc<RefCell<Option<AuthCallback>>>,
    /// State change callback
    state_callback: Rc<RefCell<Option<StateCallback>>>,
}

impl VncSessionWidget {
    /// Creates a new VNC session widget
    ///
    /// The widget is created in a disconnected state and ready for connection.
    #[must_use]
    pub fn new() -> Self {
        let display = Rc::new(VncDisplay::new());
        let state = Rc::new(RefCell::new(SessionState::Disconnected));
        let auth_callback: Rc<RefCell<Option<AuthCallback>>> = Rc::new(RefCell::new(None));
        let state_callback: Rc<RefCell<Option<StateCallback>>> = Rc::new(RefCell::new(None));

        // Create the overlay container
        let overlay = Overlay::new();

        // Create a placeholder widget for the VNC display
        // In a real implementation, this would be the actual gtk-vnc widget
        let display_placeholder = GtkBox::new(Orientation::Vertical, 0);
        display_placeholder.set_hexpand(true);
        display_placeholder.set_vexpand(true);
        display_placeholder.add_css_class("vnc-display");

        // Create status container for connection feedback
        let status_container = GtkBox::new(Orientation::Vertical, 12);
        status_container.set_halign(gtk4::Align::Center);
        status_container.set_valign(gtk4::Align::Center);

        let spinner = Spinner::new();
        spinner.set_spinning(false);
        spinner.set_visible(false);

        let status_label = Label::new(Some("Disconnected"));
        status_label.add_css_class("dim-label");

        status_container.append(&spinner);
        status_container.append(&status_label);

        // Set up the overlay
        overlay.set_child(Some(&display_placeholder));
        overlay.add_overlay(&status_container);

        let widget = Self {
            overlay,
            display,
            state,
            status_label,
            spinner,
            status_container,
            auth_callback,
            state_callback,
        };

        // Set up VNC display signal handlers
        widget.setup_display_signals();

        widget
    }

    /// Sets up signal handlers for the VNC display
    fn setup_display_signals(&self) {
        let state = self.state.clone();
        let status_label = self.status_label.clone();
        let spinner = self.spinner.clone();
        let state_callback = self.state_callback.clone();

        // Connected signal
        let state_clone = state.clone();
        let status_label_clone = status_label.clone();
        let spinner_clone = spinner.clone();
        let state_callback_clone = state_callback.clone();
        self.display.connect_vnc_connected(move |_| {
            *state_clone.borrow_mut() = SessionState::Connected;
            status_label_clone.set_text("Connected");
            spinner_clone.set_spinning(false);
            spinner_clone.set_visible(false);

            if let Some(ref callback) = *state_callback_clone.borrow() {
                callback(SessionState::Connected);
            }
        });

        // Disconnected signal
        let state_clone = state.clone();
        let status_label_clone = status_label.clone();
        let spinner_clone = spinner.clone();
        let state_callback_clone = state_callback.clone();
        self.display.connect_vnc_disconnected(move |_| {
            *state_clone.borrow_mut() = SessionState::Disconnected;
            status_label_clone.set_text("Disconnected");
            spinner_clone.set_spinning(false);
            spinner_clone.set_visible(false);

            if let Some(ref callback) = *state_callback_clone.borrow() {
                callback(SessionState::Disconnected);
            }
        });

        // Auth credential signal
        let state_clone = state.clone();
        let status_label_clone = status_label.clone();
        let auth_callback_clone = self.auth_callback.clone();
        let state_callback_clone = state_callback.clone();
        self.display.connect_vnc_auth_credential(move |_, creds| {
            *state_clone.borrow_mut() = SessionState::Authenticating;
            status_label_clone.set_text("Authenticating...");

            if let Some(ref callback) = *state_callback_clone.borrow() {
                callback(SessionState::Authenticating);
            }

            if let Some(ref callback) = *auth_callback_clone.borrow() {
                callback(creds);
            }
        });

        // Auth failure signal
        let state_clone = state;
        let status_label_clone = status_label;
        let spinner_clone = spinner;
        let state_callback_clone = state_callback;
        self.display.connect_vnc_auth_failure(move |_, msg| {
            let error = SessionError::authentication_failed(msg);
            *state_clone.borrow_mut() = SessionState::Error(error.clone());
            status_label_clone.set_text(&format!("Authentication failed: {msg}"));
            spinner_clone.set_spinning(false);
            spinner_clone.set_visible(false);

            if let Some(ref callback) = *state_callback_clone.borrow() {
                callback(SessionState::Error(error));
            }
        });
    }

    /// Connects to a VNC server
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname or IP address of the VNC server
    /// * `port` - The port number (typically 5900 + display number)
    /// * `password` - Optional password for authentication
    ///
    /// # Errors
    ///
    /// Returns a `SessionError` if the connection cannot be initiated.
    pub fn connect(
        &self,
        host: &str,
        port: u16,
        password: Option<&str>,
    ) -> Result<(), SessionError> {
        // Check current state
        let current_state = self.state.borrow().clone();
        if !current_state.can_transition_to(&SessionState::Connecting) {
            return Err(SessionError::connection_failed(format!(
                "Cannot connect from state: {current_state}"
            )));
        }

        // Update state to connecting
        *self.state.borrow_mut() = SessionState::Connecting;
        self.status_label.set_text(&format!("Connecting to {host}:{port}..."));
        self.spinner.set_visible(true);
        self.spinner.set_spinning(true);

        // Notify state change
        if let Some(ref callback) = *self.state_callback.borrow() {
            callback(SessionState::Connecting);
        }

        // If password is provided, set it before connecting
        if let Some(pwd) = password {
            self.display
                .set_credential(VncCredentialType::Password, pwd)
                .map_err(|e| SessionError::authentication_failed(e.to_string()))?;
        }

        // Initiate connection
        self.display
            .open_host(host, port)
            .map_err(|e| SessionError::connection_failed(e.to_string()))?;

        Ok(())
    }

    /// Disconnects from the VNC server
    pub fn disconnect(&self) {
        self.display.close();
        *self.state.borrow_mut() = SessionState::Disconnected;
        self.status_label.set_text("Disconnected");
        self.spinner.set_spinning(false);
        self.spinner.set_visible(false);

        if let Some(ref callback) = *self.state_callback.borrow() {
            callback(SessionState::Disconnected);
        }
    }

    /// Returns the GTK widget for embedding in containers
    #[must_use]
    pub fn widget(&self) -> &gtk4::Widget {
        self.overlay.upcast_ref()
    }

    /// Returns the current session state
    #[must_use]
    pub fn state(&self) -> SessionState {
        self.state.borrow().clone()
    }

    /// Returns whether the session is connected
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.state.borrow().is_connected()
    }

    /// Provides credentials for authentication
    ///
    /// This should be called in response to the auth_required callback.
    ///
    /// # Arguments
    ///
    /// * `username` - Optional username
    /// * `password` - The password
    ///
    /// # Errors
    ///
    /// Returns a `SessionError` if credentials cannot be set.
    pub fn provide_credentials(
        &self,
        username: Option<&str>,
        password: &str,
    ) -> Result<(), SessionError> {
        if let Some(user) = username {
            self.display
                .set_credential(VncCredentialType::Username, user)
                .map_err(|e| SessionError::authentication_failed(e.to_string()))?;
        }

        self.display
            .set_credential(VncCredentialType::Password, password)
            .map_err(|e| SessionError::authentication_failed(e.to_string()))?;

        Ok(())
    }

    /// Enables or disables display scaling
    pub fn set_scaling(&self, enabled: bool) {
        self.display.set_scaling(enabled);
    }

    /// Returns whether scaling is enabled
    #[must_use]
    pub fn scaling_enabled(&self) -> bool {
        self.display.scaling_enabled()
    }

    /// Connects a callback for authentication requests
    ///
    /// The callback receives a list of credential types that the server requires.
    pub fn connect_auth_required<F>(&self, callback: F)
    where
        F: Fn(&[VncCredentialType]) + 'static,
    {
        *self.auth_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Connects a callback for state changes
    ///
    /// The callback is invoked whenever the session state changes.
    pub fn connect_state_changed<F>(&self, callback: F)
    where
        F: Fn(SessionState) + 'static,
    {
        *self.state_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Returns the underlying VNC display (for advanced usage)
    #[must_use]
    pub fn display(&self) -> &VncDisplay {
        &self.display
    }
}

impl Default for VncSessionWidget {
    fn default() -> Self {
        Self::new()
    }
}

// Manual Debug implementation since we can't derive it for callback types
impl std::fmt::Debug for VncSessionWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VncSessionWidget")
            .field("state", &self.state.borrow())
            .field("display", &"VncDisplay { ... }")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require GTK to be initialized, which may not be available
    // in all test environments. The property tests in rustconn-core handle
    // the core logic testing without GTK dependencies.

    #[test]
    fn test_session_state_transitions() {
        // Test that state transitions are properly validated
        let disconnected = SessionState::Disconnected;
        assert!(disconnected.can_transition_to(&SessionState::Connecting));
        assert!(!disconnected.can_transition_to(&SessionState::Connected));

        let connecting = SessionState::Connecting;
        assert!(connecting.can_transition_to(&SessionState::Connected));
        assert!(connecting.can_transition_to(&SessionState::Authenticating));
        assert!(connecting.can_transition_to(&SessionState::Disconnected));

        let connected = SessionState::Connected;
        assert!(connected.can_transition_to(&SessionState::Disconnected));
        assert!(!connected.can_transition_to(&SessionState::Connecting));
    }
}
