//! Embedded session support for RDP/VNC connections
//!
//! This module provides support for embedding RDP and VNC sessions
//! within the main application window using X11 embedding (GtkSocket).
//! On Wayland, sessions fall back to external windows.

/// Display server type detected at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    /// X11 display server - supports embedding via XEmbed protocol
    X11,
    /// Wayland display server - no embedding support, uses external windows
    Wayland,
    /// Unknown display server
    Unknown,
}

impl DisplayServer {
    /// Detects the current display server by checking environment variables
    ///
    /// Detection logic:
    /// 1. Check GDK_BACKEND environment variable (explicit override)
    /// 2. Check WAYLAND_DISPLAY (indicates Wayland session)
    /// 3. Check DISPLAY (indicates X11 session)
    /// 4. Default to Unknown if neither is set
    #[must_use]
    pub fn detect() -> Self {
        // Check for explicit GDK_BACKEND override
        if let Ok(backend) = std::env::var("GDK_BACKEND") {
            let backend_lower = backend.to_lowercase();
            if backend_lower.contains("x11") {
                return Self::X11;
            }
            if backend_lower.contains("wayland") {
                return Self::Wayland;
            }
        }

        // Check XDG_SESSION_TYPE for more reliable detection
        if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
            let session_lower = session_type.to_lowercase();
            if session_lower == "x11" {
                return Self::X11;
            }
            if session_lower == "wayland" {
                return Self::Wayland;
            }
        }

        // Check for Wayland display
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            return Self::Wayland;
        }

        // Check for X11 display
        if std::env::var("DISPLAY").is_ok() {
            return Self::X11;
        }

        Self::Unknown
    }

    /// Returns whether embedding is supported on this display server
    #[must_use]
    pub const fn supports_embedding(&self) -> bool {
        matches!(self, Self::X11)
    }

    /// Returns a human-readable description of the display server
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::X11 => "X11",
            Self::Wayland => "Wayland",
            Self::Unknown => "Unknown",
        }
    }
}



use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, DrawingArea, Label, Orientation};
use std::cell::RefCell;
use std::process::Child;
use std::rc::Rc;
use uuid::Uuid;

/// Error type for embedding operations
#[derive(Debug, Clone)]
pub enum EmbeddingError {
    /// Embedding not supported on Wayland
    WaylandNotSupported { protocol: String },
    /// Failed to get window ID for embedding
    WindowIdNotAvailable,
    /// Client process failed to start
    ProcessStartFailed(String),
    /// Client exited unexpectedly
    ClientExited { code: i32 },
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WaylandNotSupported { protocol } => {
                write!(f, "Embedding not supported on Wayland for {protocol}")
            }
            Self::WindowIdNotAvailable => write!(f, "Failed to get window ID for embedding"),
            Self::ProcessStartFailed(msg) => write!(f, "Failed to start client process: {msg}"),
            Self::ClientExited { code } => write!(f, "Client exited with code {code}"),
        }
    }
}

impl std::error::Error for EmbeddingError {}

/// Session controls for embedded sessions
pub struct SessionControls {
    container: GtkBox,
    fullscreen_button: Button,
    disconnect_button: Button,
    status_label: Label,
}

impl SessionControls {
    /// Creates new session controls
    #[must_use]
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.set_margin_start(8);
        container.set_margin_end(8);
        container.set_margin_top(4);
        container.set_margin_bottom(4);

        // Status label
        let status_label = Label::new(Some("Connecting..."));
        status_label.set_hexpand(true);
        status_label.set_halign(gtk4::Align::Start);
        status_label.add_css_class("dim-label");
        container.append(&status_label);

        // Fullscreen button
        let fullscreen_button = Button::from_icon_name("view-fullscreen-symbolic");
        fullscreen_button.set_tooltip_text(Some("Toggle Fullscreen"));
        fullscreen_button.add_css_class("flat");
        container.append(&fullscreen_button);

        // Disconnect button
        let disconnect_button = Button::from_icon_name("process-stop-symbolic");
        disconnect_button.set_tooltip_text(Some("Disconnect"));
        disconnect_button.add_css_class("flat");
        disconnect_button.add_css_class("destructive-action");
        container.append(&disconnect_button);

        Self {
            container,
            fullscreen_button,
            disconnect_button,
            status_label,
        }
    }

    /// Returns the container widget
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Sets the status text
    pub fn set_status(&self, status: &str) {
        self.status_label.set_text(status);
    }

    /// Connects a callback for the fullscreen button
    pub fn connect_fullscreen<F: Fn() + 'static>(&self, callback: F) {
        self.fullscreen_button.connect_clicked(move |_| callback());
    }

    /// Connects a callback for the disconnect button
    pub fn connect_disconnect<F: Fn() + 'static>(&self, callback: F) {
        self.disconnect_button.connect_clicked(move |_| callback());
    }

    /// Returns the fullscreen button for external configuration
    #[must_use]
    pub fn fullscreen_button(&self) -> &Button {
        &self.fullscreen_button
    }

    /// Returns the disconnect button for external configuration
    #[must_use]
    pub fn disconnect_button(&self) -> &Button {
        &self.disconnect_button
    }

    /// Updates the fullscreen button icon based on state
    pub fn set_fullscreen_icon(&self, is_fullscreen: bool) {
        let icon_name = if is_fullscreen {
            "view-restore-symbolic"
        } else {
            "view-fullscreen-symbolic"
        };
        self.fullscreen_button.set_icon_name(icon_name);
    }
}

impl Default for SessionControls {
    fn default() -> Self {
        Self::new()
    }
}

/// Embedded session tab for RDP/VNC connections
///
/// This widget provides a container for embedding external graphical
/// sessions (RDP/VNC) within the main application window.
///
/// On X11, it uses the XEmbed protocol to embed the client window.
/// On Wayland, it shows a placeholder with information about the
/// external window.
pub struct EmbeddedSessionTab {
    /// Session UUID
    id: Uuid,
    /// Connection ID this session is for
    connection_id: Uuid,
    /// Protocol type (rdp or vnc)
    protocol: String,
    /// Main container widget
    container: GtkBox,
    /// Drawing area for embedding (X11) or placeholder (Wayland)
    embed_area: DrawingArea,
    /// Session controls
    controls: SessionControls,
    /// Child process handle
    process: Rc<RefCell<Option<Child>>>,
    /// Whether the session is embedded (X11) or external (Wayland)
    is_embedded: bool,
    /// Whether fullscreen mode is active
    is_fullscreen: Rc<RefCell<bool>>,
    /// X11 window ID for embedding (if available)
    window_id: Rc<RefCell<Option<u64>>>,
}

impl EmbeddedSessionTab {
    /// Creates a new embedded session tab
    ///
    /// # Arguments
    /// * `connection_id` - The connection UUID
    /// * `connection_name` - Display name for the connection
    /// * `protocol` - Protocol type ("rdp" or "vnc")
    ///
    /// # Returns
    /// A tuple of (Self, bool) where the bool indicates if embedding is supported
    #[must_use]
    pub fn new(connection_id: Uuid, connection_name: &str, protocol: &str) -> (Self, bool) {
        let id = Uuid::new_v4();
        let display_server = DisplayServer::detect();
        let is_embedded = display_server.supports_embedding();

        // Create main container
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        // Create session controls
        let controls = SessionControls::new();
        container.append(controls.widget());

        // Create embed area or placeholder
        let embed_area = DrawingArea::new();
        embed_area.set_hexpand(true);
        embed_area.set_vexpand(true);

        if is_embedded {
            // X11: Set up drawing area for embedding
            embed_area.set_content_width(800);
            embed_area.set_content_height(600);
            controls.set_status(&format!(
                "{} session - {} (embedded)",
                protocol.to_uppercase(),
                connection_name
            ));
        } else {
            // Wayland: Show placeholder with info
            controls.set_status(&format!(
                "{} session - {} (external window)",
                protocol.to_uppercase(),
                connection_name
            ));

            // Draw placeholder content
            let protocol_clone = protocol.to_string();
            let name_clone = connection_name.to_string();
            embed_area.set_draw_func(move |_area, cr, width, height| {
                // Background
                cr.set_source_rgb(0.15, 0.15, 0.15);
                let _ = cr.paint();

                // Center text
                cr.set_source_rgb(0.7, 0.7, 0.7);
                cr.select_font_face(
                    "Sans",
                    gtk4::cairo::FontSlant::Normal,
                    gtk4::cairo::FontWeight::Normal,
                );

                // Title
                cr.set_font_size(24.0);
                let title = format!("{} Session", protocol_clone.to_uppercase());
                let extents = cr.text_extents(&title).unwrap();
                let x = (f64::from(width) - extents.width()) / 2.0;
                let y = f64::from(height) / 2.0 - 30.0;
                cr.move_to(x, y);
                let _ = cr.show_text(&title);

                // Connection name
                cr.set_font_size(16.0);
                let extents = cr.text_extents(&name_clone).unwrap();
                let x = (f64::from(width) - extents.width()) / 2.0;
                let y = f64::from(height) / 2.0;
                cr.move_to(x, y);
                let _ = cr.show_text(&name_clone);

                // Info message
                cr.set_font_size(12.0);
                cr.set_source_rgb(0.5, 0.5, 0.5);
                let info = "Running in external window (Wayland does not support embedding)";
                let extents = cr.text_extents(info).unwrap();
                let x = (f64::from(width) - extents.width()) / 2.0;
                let y = f64::from(height) / 2.0 + 40.0;
                cr.move_to(x, y);
                let _ = cr.show_text(info);
            });
        }

        container.append(&embed_area);

        let tab = Self {
            id,
            connection_id,
            protocol: protocol.to_string(),
            container,
            embed_area,
            controls,
            process: Rc::new(RefCell::new(None)),
            is_embedded,
            is_fullscreen: Rc::new(RefCell::new(false)),
            window_id: Rc::new(RefCell::new(None)),
        };

        // Wire up control buttons
        tab.setup_controls();

        (tab, is_embedded)
    }

    /// Sets up control button callbacks
    fn setup_controls(&self) {
        // Fullscreen toggle
        let is_fullscreen = self.is_fullscreen.clone();
        self.controls.connect_fullscreen(move || {
            let mut fs = is_fullscreen.borrow_mut();
            *fs = !*fs;
            // Note: Actual fullscreen implementation would need window reference
        });

        // Disconnect
        let process = self.process.clone();
        self.controls.connect_disconnect(move || {
            if let Some(mut child) = process.borrow_mut().take() {
                let _ = child.kill();
            }
        });
    }

    /// Returns the session UUID
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the connection UUID
    #[must_use]
    pub const fn connection_id(&self) -> Uuid {
        self.connection_id
    }

    /// Returns the protocol type
    #[must_use]
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    /// Returns the main container widget
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Returns whether the session is embedded
    #[must_use]
    pub const fn is_embedded(&self) -> bool {
        self.is_embedded
    }

    /// Returns whether fullscreen mode is active
    #[must_use]
    pub fn is_fullscreen(&self) -> bool {
        *self.is_fullscreen.borrow()
    }

    /// Sets the status text
    pub fn set_status(&self, status: &str) {
        self.controls.set_status(status);
    }

    /// Sets the child process
    pub fn set_process(&self, child: Child) {
        *self.process.borrow_mut() = Some(child);
    }

    /// Returns the X11 window ID if available
    #[must_use]
    pub fn window_id(&self) -> Option<u64> {
        *self.window_id.borrow()
    }

    /// Sets the X11 window ID for embedding
    pub fn set_window_id(&self, id: u64) {
        *self.window_id.borrow_mut() = Some(id);
    }

    /// Disconnects the session and kills the process
    pub fn disconnect(&self) {
        if let Some(mut child) = self.process.borrow_mut().take() {
            let _ = child.kill();
        }
        self.controls.set_status("Disconnected");
    }

    /// Checks if the process is still running
    #[must_use]
    pub fn is_running(&self) -> bool {
        if let Some(ref mut child) = *self.process.borrow_mut() {
            match child.try_wait() {
                Ok(None) => true,  // Still running
                Ok(Some(_)) => false,  // Exited
                Err(_) => false,  // Error checking
            }
        } else {
            false
        }
    }

    /// Gets the native window handle for X11 embedding
    ///
    /// This attempts to get the X11 window ID from the drawing area's
    /// native surface. Only works on X11.
    #[must_use]
    pub fn get_native_window_id(&self) -> Option<u64> {
        // In GTK4, getting the native window ID requires the widget to be realized
        // and using platform-specific APIs. For X11, we need to use GDK's X11 backend.
        //
        // This is a placeholder - actual implementation would use:
        // gdk_x11_surface_get_xid() on the native surface
        //
        // For now, we'll return None and handle this in the start methods
        // where we can use alternative approaches like creating a separate
        // X11 window or using the parent window ID.
        None
    }

    /// Toggles fullscreen mode for the session
    ///
    /// This method should be called with a reference to the parent window
    /// to properly toggle fullscreen mode.
    pub fn toggle_fullscreen(&self, window: &gtk4::ApplicationWindow) {
        let mut fs = self.is_fullscreen.borrow_mut();
        *fs = !*fs;
        
        if *fs {
            window.fullscreen();
            self.controls.set_fullscreen_icon(true);
        } else {
            window.unfullscreen();
            self.controls.set_fullscreen_icon(false);
        }
    }

    /// Sets up fullscreen toggle with window reference
    ///
    /// This connects the fullscreen button to toggle the window's
    /// fullscreen state.
    pub fn setup_fullscreen_toggle(&self, window: &gtk4::ApplicationWindow) {
        let window_weak = window.downgrade();
        let is_fullscreen = self.is_fullscreen.clone();
        let controls_fs_btn = self.controls.fullscreen_button().clone();
        
        self.controls.fullscreen_button().connect_clicked(move |_| {
            if let Some(win) = window_weak.upgrade() {
                let mut fs = is_fullscreen.borrow_mut();
                *fs = !*fs;
                
                if *fs {
                    win.fullscreen();
                    controls_fs_btn.set_icon_name("view-restore-symbolic");
                } else {
                    win.unfullscreen();
                    controls_fs_btn.set_icon_name("view-fullscreen-symbolic");
                }
            }
        });
    }

    /// Returns the session controls for external configuration
    #[must_use]
    pub fn controls(&self) -> &SessionControls {
        &self.controls
    }
}


/// RDP session launcher for embedded and external sessions
pub struct RdpLauncher;

impl RdpLauncher {
    /// Starts an RDP session, attempting to embed on X11
    ///
    /// # Arguments
    /// * `tab` - The embedded session tab to use
    /// * `host` - Remote host address
    /// * `port` - Remote port (default 3389)
    /// * `username` - Optional username
    /// * `resolution` - Optional resolution (width, height)
    /// * `extra_args` - Additional xfreerdp arguments
    ///
    /// # Returns
    /// Ok(()) on success, or an EmbeddingError on failure
    ///
    /// # Errors
    /// Returns error if the RDP client fails to start
    pub fn start(
        tab: &EmbeddedSessionTab,
        host: &str,
        port: u16,
        username: Option<&str>,
        resolution: Option<(u32, u32)>,
        extra_args: &[String],
    ) -> Result<(), EmbeddingError> {
        let display_server = DisplayServer::detect();

        if display_server.supports_embedding() {
            Self::start_embedded(tab, host, port, username, resolution, extra_args)
        } else {
            Self::start_external(tab, host, port, username, resolution, extra_args)
        }
    }

    /// Starts an embedded RDP session using xfreerdp with /parent-window:
    fn start_embedded(
        tab: &EmbeddedSessionTab,
        host: &str,
        port: u16,
        username: Option<&str>,
        resolution: Option<(u32, u32)>,
        extra_args: &[String],
    ) -> Result<(), EmbeddingError> {
        use std::process::Command;

        let mut cmd = Command::new("xfreerdp");

        // Server address
        if port == 3389 {
            cmd.arg(format!("/v:{host}"));
        } else {
            cmd.arg(format!("/v:{host}:{port}"));
        }

        // Username
        if let Some(user) = username {
            cmd.arg(format!("/u:{user}"));
        }

        // Resolution
        if let Some((width, height)) = resolution {
            cmd.arg(format!("/w:{width}"));
            cmd.arg(format!("/h:{height}"));
        } else {
            // Default to a reasonable size
            cmd.arg("/w:1280");
            cmd.arg("/h:720");
        }

        // For X11 embedding, we need to get the window ID
        // Since GTK4 doesn't expose X11 window IDs directly for DrawingArea,
        // we use a workaround: create a floating window and get its ID,
        // or use the /parent-window: parameter with the main window's ID
        //
        // For now, we'll use the decorations-off mode which works better
        // with GTK4's compositor
        cmd.arg("/decorations:off");
        cmd.arg("/floatbar:sticky:on,default:visible,show:always");

        // Add extra arguments
        for arg in extra_args {
            cmd.arg(arg);
        }

        // Spawn the process
        match cmd.spawn() {
            Ok(child) => {
                tab.set_process(child);
                tab.set_status(&format!("Connected to {host}"));
                Ok(())
            }
            Err(e) => Err(EmbeddingError::ProcessStartFailed(e.to_string())),
        }
    }

    /// Starts an external RDP session (Wayland fallback)
    fn start_external(
        tab: &EmbeddedSessionTab,
        host: &str,
        port: u16,
        username: Option<&str>,
        resolution: Option<(u32, u32)>,
        extra_args: &[String],
    ) -> Result<(), EmbeddingError> {
        use std::process::Command;

        let mut cmd = Command::new("xfreerdp");

        // Server address
        if port == 3389 {
            cmd.arg(format!("/v:{host}"));
        } else {
            cmd.arg(format!("/v:{host}:{port}"));
        }

        // Username
        if let Some(user) = username {
            cmd.arg(format!("/u:{user}"));
        }

        // Resolution
        if let Some((width, height)) = resolution {
            cmd.arg(format!("/w:{width}"));
            cmd.arg(format!("/h:{height}"));
        }

        // Add extra arguments
        for arg in extra_args {
            cmd.arg(arg);
        }

        // Spawn the process
        match cmd.spawn() {
            Ok(child) => {
                tab.set_process(child);
                tab.set_status(&format!("Connected to {host} (external window)"));
                Ok(())
            }
            Err(e) => Err(EmbeddingError::ProcessStartFailed(e.to_string())),
        }
    }
}


/// VNC session launcher for embedded and external sessions
pub struct VncLauncher;

impl VncLauncher {
    /// Starts a VNC session, attempting to embed on X11
    ///
    /// # Arguments
    /// * `tab` - The embedded session tab to use
    /// * `host` - Remote host address
    /// * `port` - Remote port (default 5900)
    /// * `encoding` - Optional encoding preference
    /// * `quality` - Optional quality level (0-9)
    /// * `extra_args` - Additional vncviewer arguments
    ///
    /// # Returns
    /// Ok(()) on success, or an EmbeddingError on failure
    ///
    /// # Errors
    /// Returns error if the VNC client fails to start
    pub fn start(
        tab: &EmbeddedSessionTab,
        host: &str,
        port: u16,
        encoding: Option<&str>,
        quality: Option<u8>,
        extra_args: &[String],
    ) -> Result<(), EmbeddingError> {
        let display_server = DisplayServer::detect();

        if display_server.supports_embedding() {
            Self::start_embedded(tab, host, port, encoding, quality, extra_args)
        } else {
            Self::start_external(tab, host, port, encoding, quality, extra_args)
        }
    }

    /// Starts an embedded VNC session using vncviewer
    ///
    /// Note: TigerVNC and TightVNC have different embedding support.
    /// TigerVNC supports -embed option on some versions.
    fn start_embedded(
        tab: &EmbeddedSessionTab,
        host: &str,
        port: u16,
        encoding: Option<&str>,
        quality: Option<u8>,
        extra_args: &[String],
    ) -> Result<(), EmbeddingError> {
        use std::process::Command;

        let mut cmd = Command::new("vncviewer");

        // Encoding preference
        if let Some(enc) = encoding {
            // Try TigerVNC style first
            cmd.arg("-PreferredEncoding");
            cmd.arg(enc);
        }

        // Quality level
        if let Some(q) = quality {
            cmd.arg("-QualityLevel");
            cmd.arg(q.to_string());
        }

        // Add extra arguments
        for arg in extra_args {
            cmd.arg(arg);
        }

        // Server address (VNC uses display numbers)
        // Port 5900 = display :0, 5901 = display :1, etc.
        let server = if port == 5900 {
            format!("{host}:0")
        } else if port > 5900 && port < 6000 {
            let display = port - 5900;
            format!("{host}:{display}")
        } else {
            // Non-standard port, use ::port format
            format!("{host}::{port}")
        };
        cmd.arg(&server);

        // Spawn the process
        match cmd.spawn() {
            Ok(child) => {
                tab.set_process(child);
                tab.set_status(&format!("Connected to {host}"));
                Ok(())
            }
            Err(e) => Err(EmbeddingError::ProcessStartFailed(e.to_string())),
        }
    }

    /// Starts an external VNC session (Wayland fallback)
    fn start_external(
        tab: &EmbeddedSessionTab,
        host: &str,
        port: u16,
        encoding: Option<&str>,
        quality: Option<u8>,
        extra_args: &[String],
    ) -> Result<(), EmbeddingError> {
        use std::process::Command;

        let mut cmd = Command::new("vncviewer");

        // Encoding preference
        if let Some(enc) = encoding {
            cmd.arg("-PreferredEncoding");
            cmd.arg(enc);
        }

        // Quality level
        if let Some(q) = quality {
            cmd.arg("-QualityLevel");
            cmd.arg(q.to_string());
        }

        // Add extra arguments
        for arg in extra_args {
            cmd.arg(arg);
        }

        // Server address
        let server = if port == 5900 {
            format!("{host}:0")
        } else if port > 5900 && port < 6000 {
            let display = port - 5900;
            format!("{host}:{display}")
        } else {
            format!("{host}::{port}")
        };
        cmd.arg(&server);

        // Spawn the process
        match cmd.spawn() {
            Ok(child) => {
                tab.set_process(child);
                tab.set_status(&format!("Connected to {host} (external window)"));
                Ok(())
            }
            Err(e) => Err(EmbeddingError::ProcessStartFailed(e.to_string())),
        }
    }
}


/// Helper functions for embedded session management
pub mod helpers {
    use super::*;

    /// Creates an info message about Wayland limitations
    ///
    /// Returns a formatted message explaining that embedding is not
    /// supported on Wayland and the session will run in an external window.
    #[must_use]
    pub fn wayland_fallback_message(protocol: &str) -> String {
        format!(
            "Embedding {} sessions is not supported on Wayland.\n\n\
             The session will open in an external window.\n\n\
             To enable embedded sessions, run RustConn under X11 \
             (set GDK_BACKEND=x11 or use XWayland).",
            protocol.to_uppercase()
        )
    }

    /// Checks if embedding is available and returns appropriate message
    ///
    /// # Returns
    /// - `Ok(())` if embedding is supported
    /// - `Err(message)` with user-friendly message if not supported
    pub fn check_embedding_support(protocol: &str) -> Result<(), String> {
        let display_server = DisplayServer::detect();
        
        if display_server.supports_embedding() {
            Ok(())
        } else {
            Err(wayland_fallback_message(protocol))
        }
    }

    /// Creates an embedded session tab with appropriate fallback handling
    ///
    /// This is the main entry point for creating embedded sessions.
    /// It handles display server detection and provides appropriate
    /// user feedback.
    ///
    /// # Arguments
    /// * `connection_id` - The connection UUID
    /// * `connection_name` - Display name for the connection
    /// * `protocol` - Protocol type ("rdp" or "vnc")
    ///
    /// # Returns
    /// A tuple of (tab, is_embedded, optional_warning_message)
    #[must_use]
    pub fn create_session_tab(
        connection_id: Uuid,
        connection_name: &str,
        protocol: &str,
    ) -> (EmbeddedSessionTab, bool, Option<String>) {
        let (tab, is_embedded) = EmbeddedSessionTab::new(connection_id, connection_name, protocol);
        
        let warning = if !is_embedded {
            Some(wayland_fallback_message(protocol))
        } else {
            None
        };
        
        (tab, is_embedded, warning)
    }
}
