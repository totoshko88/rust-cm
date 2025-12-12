//! Terminal notebook area
//!
//! This module provides the tabbed terminal interface using VTE4
//! for SSH sessions and native GTK widgets for VNC/RDP/SPICE connections.
//!
//! # Requirements Coverage
//!
//! - Requirement 2.1: Native VNC embedding as GTK widget
//! - Requirement 2.6: Multiple VNC sessions in separate tabs with proper isolation

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, Box as GtkBox, Button, Label, Notebook, Orientation, Widget};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use uuid::Uuid;
use vte4::prelude::*;
use vte4::{PtyFlags, Terminal};

use crate::session::{
    RdpSessionWidget, SessionState, SessionWidget, SpiceSessionWidget, VncSessionWidget,
};

/// Terminal session information
#[derive(Debug, Clone)]
pub struct TerminalSession {
    /// Session UUID (stored for future session management features)
    #[allow(dead_code)]
    pub id: Uuid,
    /// Connection ID this session is for
    pub connection_id: Uuid,
    /// Connection name for display
    pub name: String,
    /// Protocol type (ssh, rdp, vnc, spice)
    pub protocol: String,
    /// Whether this is an embedded terminal or external window
    pub is_embedded: bool,
    /// Log file path if logging is enabled
    pub log_file: Option<PathBuf>,
}

/// Session widget storage for non-SSH sessions
enum SessionWidgetStorage {
    /// VNC session widget
    Vnc(Rc<VncSessionWidget>),
    /// RDP session widget
    Rdp(Rc<RdpSessionWidget>),
    /// SPICE session widget
    Spice(Rc<SpiceSessionWidget>),
}

/// Terminal notebook widget for managing multiple terminal sessions
pub struct TerminalNotebook {
    notebook: Notebook,
    /// Map of session IDs to their tab indices
    sessions: Rc<RefCell<HashMap<Uuid, u32>>>,
    /// Map of session IDs to terminal widgets (for SSH sessions)
    terminals: Rc<RefCell<HashMap<Uuid, Terminal>>>,
    /// Map of session IDs to session widgets (for VNC/RDP/SPICE sessions)
    session_widgets: Rc<RefCell<HashMap<Uuid, SessionWidgetStorage>>>,
    /// Session metadata
    session_info: Rc<RefCell<HashMap<Uuid, TerminalSession>>>,
}

impl TerminalNotebook {
    /// Creates a new terminal notebook
    #[must_use]
    pub fn new() -> Self {
        let notebook = Notebook::new();
        notebook.set_scrollable(true);
        notebook.set_show_border(false);
        notebook.set_tab_pos(gtk4::PositionType::Top);

        // Add a welcome tab
        let welcome = Self::create_welcome_tab();
        let welcome_label = Label::new(Some("Welcome"));
        notebook.append_page(&welcome, Some(&welcome_label));

        Self {
            notebook,
            sessions: Rc::new(RefCell::new(HashMap::new())),
            terminals: Rc::new(RefCell::new(HashMap::new())),
            session_widgets: Rc::new(RefCell::new(HashMap::new())),
            session_info: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Creates the welcome tab content (minimal - actual content shown in split view)
    fn create_welcome_tab() -> GtkBox {
        // Create empty container - actual welcome content is in split view
        let container = GtkBox::new(Orientation::Vertical, 0);
        // Add an invisible spacer to ensure the page has some content
        let spacer = gtk4::DrawingArea::new();
        spacer.set_content_width(1);
        spacer.set_content_height(1);
        container.append(&spacer);
        container
    }

    /// Creates a new terminal tab for an SSH session
    ///
    /// This creates the terminal widget and prepares it for spawning a command.
    /// Returns the session UUID for tracking.
    pub fn create_terminal_tab(&self, connection_id: Uuid, title: &str, protocol: &str) -> Uuid {
        let session_id = Uuid::new_v4();
        let is_first_session = self.sessions.borrow().is_empty();

        // Remove Welcome tab if this is the first session
        // Welcome tab is page 0 and has no session_id
        if is_first_session && self.notebook.n_pages() > 0 {
            self.notebook.remove_page(Some(0));
        }

        // Create VTE terminal
        let terminal = Terminal::new();
        terminal.set_hexpand(true);
        terminal.set_vexpand(true);

        // Configure terminal appearance
        Self::configure_terminal(&terminal);

        // Create empty placeholder for notebook page (terminal shown in split view)
        let placeholder = GtkBox::new(Orientation::Vertical, 0);
        // Add an invisible spacer to ensure the page has some content
        let spacer = gtk4::DrawingArea::new();
        spacer.set_content_width(1);
        spacer.set_content_height(1);
        placeholder.append(&spacer);

        // Create tab label with close button
        let tab_label = Self::create_tab_label(title, session_id, &self.notebook, &self.sessions);

        // Add the page with empty placeholder (terminal is NOT added to notebook)
        let page_num = self.notebook.append_page(&placeholder, Some(&tab_label));

        // Store session mapping BEFORE switching page
        // This ensures switch_page handler can find the session
        self.sessions
            .borrow_mut()
            .insert(session_id, page_num);
        // Store terminal separately - it will be shown in split view
        self.terminals.borrow_mut().insert(session_id, terminal);

        // Store session info
        self.session_info.borrow_mut().insert(
            session_id,
            TerminalSession {
                id: session_id,
                connection_id,
                name: title.to_string(),
                protocol: protocol.to_string(),
                is_embedded: true,
                log_file: None,
            },
        );

        // Make the tab reorderable
        self.notebook.set_tab_reorderable(&placeholder, true);

        // Switch to the new tab AFTER all data is stored
        // This triggers switch_page signal which will show the session in split view
        self.notebook.set_current_page(Some(page_num));

        session_id
    }

    /// Creates a new VNC session tab
    ///
    /// This creates a VNC session widget and prepares it for connection.
    /// Returns the session UUID for tracking.
    ///
    /// # Requirements Coverage
    ///
    /// - Requirement 2.1: Native VNC embedding as GTK widget
    /// - Requirement 2.6: Multiple VNC sessions in separate tabs
    pub fn create_vnc_session_tab(&self, connection_id: Uuid, title: &str) -> Uuid {
        let session_id = Uuid::new_v4();
        let is_first_session = self.sessions.borrow().is_empty();

        // Remove Welcome tab if this is the first session
        if is_first_session && self.notebook.n_pages() > 0 {
            self.notebook.remove_page(Some(0));
        }

        // Create VNC session widget
        let vnc_widget = Rc::new(VncSessionWidget::new());

        // Create empty placeholder for notebook page (VNC widget shown in split view)
        let placeholder = GtkBox::new(Orientation::Vertical, 0);
        let spacer = gtk4::DrawingArea::new();
        spacer.set_content_width(1);
        spacer.set_content_height(1);
        placeholder.append(&spacer);

        // Create tab label with close button
        let tab_label = Self::create_tab_label(title, session_id, &self.notebook, &self.sessions);

        // Add the page with empty placeholder
        let page_num = self.notebook.append_page(&placeholder, Some(&tab_label));

        // Store session mapping
        self.sessions.borrow_mut().insert(session_id, page_num);

        // Store VNC widget
        self.session_widgets
            .borrow_mut()
            .insert(session_id, SessionWidgetStorage::Vnc(vnc_widget));

        // Store session info
        self.session_info.borrow_mut().insert(
            session_id,
            TerminalSession {
                id: session_id,
                connection_id,
                name: title.to_string(),
                protocol: "vnc".to_string(),
                is_embedded: true,
                log_file: None,
            },
        );

        // Make the tab reorderable
        self.notebook.set_tab_reorderable(&placeholder, true);

        // Switch to the new tab
        self.notebook.set_current_page(Some(page_num));

        session_id
    }

    /// Creates a new RDP session tab
    ///
    /// This creates an RDP session widget and prepares it for connection.
    /// Returns the session UUID for tracking.
    ///
    /// # Requirements Coverage
    ///
    /// - Requirement 3.1: Native RDP embedding as GTK widget
    pub fn create_rdp_session_tab(&self, connection_id: Uuid, title: &str) -> Uuid {
        let session_id = Uuid::new_v4();
        let is_first_session = self.sessions.borrow().is_empty();

        // Remove Welcome tab if this is the first session
        if is_first_session && self.notebook.n_pages() > 0 {
            self.notebook.remove_page(Some(0));
        }

        // Create RDP session widget
        let rdp_widget = Rc::new(RdpSessionWidget::new());

        // Create empty placeholder for notebook page (RDP widget shown in split view)
        let placeholder = GtkBox::new(Orientation::Vertical, 0);
        let spacer = gtk4::DrawingArea::new();
        spacer.set_content_width(1);
        spacer.set_content_height(1);
        placeholder.append(&spacer);

        // Create tab label with close button
        let tab_label = Self::create_tab_label(title, session_id, &self.notebook, &self.sessions);

        // Add the page with empty placeholder
        let page_num = self.notebook.append_page(&placeholder, Some(&tab_label));

        // Store session mapping
        self.sessions.borrow_mut().insert(session_id, page_num);

        // Store RDP widget
        self.session_widgets
            .borrow_mut()
            .insert(session_id, SessionWidgetStorage::Rdp(rdp_widget));

        // Store session info
        self.session_info.borrow_mut().insert(
            session_id,
            TerminalSession {
                id: session_id,
                connection_id,
                name: title.to_string(),
                protocol: "rdp".to_string(),
                is_embedded: true,
                log_file: None,
            },
        );

        // Make the tab reorderable
        self.notebook.set_tab_reorderable(&placeholder, true);

        // Switch to the new tab
        self.notebook.set_current_page(Some(page_num));

        session_id
    }

    /// Creates a new SPICE session tab
    ///
    /// This creates a SPICE session widget and prepares it for connection.
    /// Returns the session UUID for tracking.
    ///
    /// # Requirements Coverage
    ///
    /// - Requirement 4.2: Native SPICE embedding as GTK widget
    pub fn create_spice_session_tab(&self, connection_id: Uuid, title: &str) -> Uuid {
        let session_id = Uuid::new_v4();
        let is_first_session = self.sessions.borrow().is_empty();

        // Remove Welcome tab if this is the first session
        if is_first_session && self.notebook.n_pages() > 0 {
            self.notebook.remove_page(Some(0));
        }

        // Create SPICE session widget
        let spice_widget = Rc::new(SpiceSessionWidget::new());

        // Create empty placeholder for notebook page (SPICE widget shown in split view)
        let placeholder = GtkBox::new(Orientation::Vertical, 0);
        let spacer = gtk4::DrawingArea::new();
        spacer.set_content_width(1);
        spacer.set_content_height(1);
        placeholder.append(&spacer);

        // Create tab label with close button
        let tab_label = Self::create_tab_label(title, session_id, &self.notebook, &self.sessions);

        // Add the page with empty placeholder
        let page_num = self.notebook.append_page(&placeholder, Some(&tab_label));

        // Store session mapping
        self.sessions.borrow_mut().insert(session_id, page_num);

        // Store SPICE widget
        self.session_widgets
            .borrow_mut()
            .insert(session_id, SessionWidgetStorage::Spice(spice_widget));

        // Store session info
        self.session_info.borrow_mut().insert(
            session_id,
            TerminalSession {
                id: session_id,
                connection_id,
                name: title.to_string(),
                protocol: "spice".to_string(),
                is_embedded: true,
                log_file: None,
            },
        );

        // Make the tab reorderable
        self.notebook.set_tab_reorderable(&placeholder, true);

        // Switch to the new tab
        self.notebook.set_current_page(Some(page_num));

        session_id
    }

    /// Gets the VNC session widget for a session
    #[must_use]
    pub fn get_vnc_widget(&self, session_id: Uuid) -> Option<Rc<VncSessionWidget>> {
        let widgets = self.session_widgets.borrow();
        match widgets.get(&session_id) {
            Some(SessionWidgetStorage::Vnc(widget)) => Some(widget.clone()),
            _ => None,
        }
    }

    /// Gets the RDP session widget for a session
    #[must_use]
    pub fn get_rdp_widget(&self, session_id: Uuid) -> Option<Rc<RdpSessionWidget>> {
        let widgets = self.session_widgets.borrow();
        match widgets.get(&session_id) {
            Some(SessionWidgetStorage::Rdp(widget)) => Some(widget.clone()),
            _ => None,
        }
    }

    /// Gets the SPICE session widget for a session
    #[must_use]
    pub fn get_spice_widget(&self, session_id: Uuid) -> Option<Rc<SpiceSessionWidget>> {
        let widgets = self.session_widgets.borrow();
        match widgets.get(&session_id) {
            Some(SessionWidgetStorage::Spice(widget)) => Some(widget.clone()),
            _ => None,
        }
    }

    /// Gets the session widget (VNC/RDP/SPICE) for a session
    #[must_use]
    pub fn get_session_widget(&self, session_id: Uuid) -> Option<SessionWidget> {
        let widgets = self.session_widgets.borrow();
        match widgets.get(&session_id) {
            Some(SessionWidgetStorage::Vnc(_)) => {
                // Return a new VncSessionWidget wrapper
                // Note: The actual widget is stored separately and accessed via get_vnc_widget
                Some(SessionWidget::Vnc(VncSessionWidget::new()))
            }
            Some(SessionWidgetStorage::Rdp(_)) => {
                // Return a new RdpSessionWidget wrapper
                // Note: The actual widget is stored separately and accessed via get_rdp_widget
                Some(SessionWidget::Rdp(RdpSessionWidget::new()))
            }
            Some(SessionWidgetStorage::Spice(_)) => {
                // Return a new SpiceSessionWidget wrapper
                // Note: The actual widget is stored separately and accessed via get_spice_widget
                Some(SessionWidget::Spice(SpiceSessionWidget::new()))
            }
            _ => {
                drop(widgets);
                // Check if it's an SSH terminal
                if let Some(terminal) = self.terminals.borrow().get(&session_id) {
                    Some(SessionWidget::Ssh(terminal.clone()))
                } else {
                    None
                }
            }
        }
    }

    /// Gets the GTK widget for a session (for display in split view)
    ///
    /// Returns the appropriate widget based on session type:
    /// - SSH: VTE Terminal widget
    /// - VNC: VncSessionWidget overlay
    /// - RDP: RdpSessionWidget overlay
    /// - SPICE: SpiceSessionWidget overlay
    #[must_use]
    pub fn get_session_display_widget(&self, session_id: Uuid) -> Option<Widget> {
        // Check for VNC/RDP/SPICE session widgets first
        let widgets = self.session_widgets.borrow();
        if let Some(storage) = widgets.get(&session_id) {
            return match storage {
                SessionWidgetStorage::Vnc(widget) => Some(widget.widget().clone()),
                SessionWidgetStorage::Rdp(widget) => Some(widget.widget().clone()),
                SessionWidgetStorage::Spice(widget) => Some(widget.widget().clone()),
            };
        }
        drop(widgets);

        // Fall back to SSH terminal
        self.terminals
            .borrow()
            .get(&session_id)
            .map(|t| t.clone().upcast())
    }

    /// Gets the session state for a VNC/RDP/SPICE session
    #[must_use]
    pub fn get_session_state(&self, session_id: Uuid) -> Option<SessionState> {
        let widgets = self.session_widgets.borrow();
        match widgets.get(&session_id) {
            Some(SessionWidgetStorage::Vnc(widget)) => Some(widget.state()),
            Some(SessionWidgetStorage::Rdp(widget)) => Some(widget.state()),
            Some(SessionWidgetStorage::Spice(widget)) => Some(widget.state()),
            _ => None,
        }
    }

    /// Configures terminal appearance and behavior
    fn configure_terminal(terminal: &Terminal) {
        // Cursor settings
        terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
        terminal.set_cursor_shape(vte4::CursorShape::Block);

        // Scrolling behavior
        terminal.set_scroll_on_output(false);
        terminal.set_scroll_on_keystroke(true);
        terminal.set_scrollback_lines(10000);

        // Input handling
        terminal.set_input_enabled(true);
        terminal.set_allow_hyperlink(true);
        terminal.set_mouse_autohide(true);

        // Set up terminal colors (dark theme)
        let bg_color = gdk::RGBA::new(0.1, 0.1, 0.1, 1.0);
        let fg_color = gdk::RGBA::new(0.9, 0.9, 0.9, 1.0);
        terminal.set_color_background(&bg_color);
        terminal.set_color_foreground(&fg_color);

        // Set up palette colors (standard 16-color palette)
        let palette: [gdk::RGBA; 16] = [
            gdk::RGBA::new(0.0, 0.0, 0.0, 1.0), // Black
            gdk::RGBA::new(0.8, 0.0, 0.0, 1.0), // Red
            gdk::RGBA::new(0.0, 0.8, 0.0, 1.0), // Green
            gdk::RGBA::new(0.8, 0.8, 0.0, 1.0), // Yellow
            gdk::RGBA::new(0.0, 0.0, 0.8, 1.0), // Blue
            gdk::RGBA::new(0.8, 0.0, 0.8, 1.0), // Magenta
            gdk::RGBA::new(0.0, 0.8, 0.8, 1.0), // Cyan
            gdk::RGBA::new(0.8, 0.8, 0.8, 1.0), // White
            gdk::RGBA::new(0.4, 0.4, 0.4, 1.0), // Bright Black
            gdk::RGBA::new(1.0, 0.0, 0.0, 1.0), // Bright Red
            gdk::RGBA::new(0.0, 1.0, 0.0, 1.0), // Bright Green
            gdk::RGBA::new(1.0, 1.0, 0.0, 1.0), // Bright Yellow
            gdk::RGBA::new(0.0, 0.0, 1.0, 1.0), // Bright Blue
            gdk::RGBA::new(1.0, 0.0, 1.0, 1.0), // Bright Magenta
            gdk::RGBA::new(0.0, 1.0, 1.0, 1.0), // Bright Cyan
            gdk::RGBA::new(1.0, 1.0, 1.0, 1.0), // Bright White
        ];
        let palette_refs: Vec<&gdk::RGBA> = palette.iter().collect();
        terminal.set_colors(Some(&fg_color), Some(&bg_color), &palette_refs);

        // Font settings
        let font_desc = gtk4::pango::FontDescription::from_string("Monospace 11");
        terminal.set_font(Some(&font_desc));
    }

    /// Spawns a command in the terminal
    ///
    /// # Arguments
    /// * `session_id` - The session ID to spawn the command in
    /// * `argv` - The command and arguments to execute
    /// * `envv` - Optional environment variables
    /// * `working_directory` - Optional working directory
    ///
    /// # Returns
    /// `true` if the command was spawned successfully
    pub fn spawn_command(
        &self,
        session_id: Uuid,
        argv: &[&str],
        envv: Option<&[&str]>,
        working_directory: Option<&str>,
    ) -> bool {
        let terminals = self.terminals.borrow();
        let Some(terminal) = terminals.get(&session_id) else {
            return false;
        };

        // Convert argv to the format VTE expects
        let argv_gstr: Vec<glib::GString> = argv.iter().map(|s| glib::GString::from(*s)).collect();
        let argv_refs: Vec<&str> = argv_gstr.iter().map(gtk4::glib::GString::as_str).collect();

        // Convert envv if provided
        let envv_gstr: Option<Vec<glib::GString>> =
            envv.map(|e| e.iter().map(|s| glib::GString::from(*s)).collect());
        let envv_refs: Option<Vec<&str>> = envv_gstr
            .as_ref()
            .map(|e| e.iter().map(gtk4::glib::GString::as_str).collect());

        // Spawn the command asynchronously
        terminal.spawn_async(
            PtyFlags::DEFAULT,
            working_directory,
            &argv_refs,
            envv_refs.as_deref().unwrap_or(&[]),
            glib::SpawnFlags::DEFAULT,
            || {},
            -1,
            gio::Cancellable::NONE,
            |result| {
                match result {
                    Ok(_pid) => {
                        // Command spawned successfully
                    }
                    Err(e) => {
                        eprintln!("Failed to spawn command: {e}");
                    }
                }
            },
        );

        true
    }

    /// Spawns an SSH command in the terminal
    ///
    /// This is a convenience method for spawning SSH connections.
    pub fn spawn_ssh(
        &self,
        session_id: Uuid,
        host: &str,
        port: u16,
        username: Option<&str>,
        identity_file: Option<&str>,
        extra_args: &[&str],
    ) -> bool {
        let mut argv = vec!["ssh"];

        // Add port if not default
        let port_str;
        if port != 22 {
            port_str = port.to_string();
            argv.push("-p");
            argv.push(&port_str);
        }

        // Add identity file if specified
        if let Some(key) = identity_file {
            argv.push("-i");
            argv.push(key);
        }

        // Add extra arguments
        argv.extend(extra_args);

        // Add destination
        let destination = if let Some(user) = username {
            format!("{user}@{host}")
        } else {
            host.to_string()
        };
        argv.push(&destination);

        self.spawn_command(session_id, &argv, None, None)
    }

    /// Creates a tab label with title, close button, and drag source
    fn create_tab_label(
        title: &str,
        session_id: Uuid,
        notebook: &Notebook,
        sessions: &Rc<RefCell<HashMap<Uuid, u32>>>,
    ) -> GtkBox {
        let tab_box = GtkBox::new(Orientation::Horizontal, 4);

        let label = Label::new(Some(title));
        label.set_hexpand(true);
        tab_box.append(&label);

        let close_button = Button::from_icon_name("window-close-symbolic");
        close_button.add_css_class("flat");
        close_button.add_css_class("circular");
        close_button.set_tooltip_text(Some("Close tab"));

        // Connect close button - switch to this tab first, then trigger close-tab action
        let notebook_weak = notebook.downgrade();
        let sessions_clone = sessions.clone();
        close_button.connect_clicked(move |button| {
            if let Some(notebook) = notebook_weak.upgrade() {
                let sessions = sessions_clone.borrow();
                if let Some(&page_num) = sessions.get(&session_id) {
                    // Switch to this tab first so close-tab action closes the right one
                    notebook.set_current_page(Some(page_num));
                    drop(sessions);
                    // Trigger the close-tab action
                    if let Some(root) = button.root() {
                        if let Some(window) = root.downcast_ref::<gtk4::ApplicationWindow>() {
                            gtk4::prelude::ActionGroupExt::activate_action(window, "close-tab", None);
                        }
                    }
                }
            }
        });

        tab_box.append(&close_button);

        // Add drag source for dragging sessions to split panes
        let drag_source = gtk4::DragSource::new();
        drag_source.set_actions(gdk::DragAction::MOVE);

        // Provide the session ID as drag data
        let session_id_str = session_id.to_string();
        drag_source.connect_prepare(move |_source, _x, _y| {
            let value = glib::Value::from(&session_id_str);
            let content = gdk::ContentProvider::for_value(&value);
            Some(content)
        });

        tab_box.add_controller(drag_source);
        tab_box.set_tooltip_text(Some("Drag to split pane"));

        tab_box
    }

    /// Closes a terminal tab by session ID
    pub fn close_tab(&self, session_id: Uuid) {
        // Get page_num and remove from sessions map first
        let page_num = self.sessions.borrow_mut().remove(&session_id);

        // Clean up terminal and session info before removing page
        // (to avoid issues with switch_page signal)
        self.terminals.borrow_mut().remove(&session_id);
        self.session_widgets.borrow_mut().remove(&session_id);
        self.session_info.borrow_mut().remove(&session_id);

        // Now remove the page (this may trigger switch_page signal)
        if let Some(page_num) = page_num {
            self.notebook.remove_page(Some(page_num));

            // Update page numbers for remaining sessions
            // (pages after the removed one shift down by 1)
            let mut sessions = self.sessions.borrow_mut();
            for (_, num) in sessions.iter_mut() {
                if *num > page_num {
                    *num -= 1;
                }
            }
            let is_empty = sessions.is_empty();
            drop(sessions);

            // Restore Welcome tab if no sessions remain
            if is_empty {
                let welcome = Self::create_welcome_tab();
                let welcome_label = Label::new(Some("Welcome"));
                self.notebook.append_page(&welcome, Some(&welcome_label));
                self.notebook.set_current_page(Some(0));
            }
        }
    }

    /// Gets the terminal widget for a session
    #[must_use]
    pub fn get_terminal(&self, session_id: Uuid) -> Option<Terminal> {
        self.terminals.borrow().get(&session_id).cloned()
    }

    /// Gets session info for a session
    #[must_use]
    pub fn get_session_info(&self, session_id: Uuid) -> Option<TerminalSession> {
        self.session_info.borrow().get(&session_id).cloned()
    }

    /// Sets the log file path for a session
    pub fn set_log_file(&self, session_id: Uuid, log_file: PathBuf) {
        if let Some(info) = self.session_info.borrow_mut().get_mut(&session_id) {
            info.log_file = Some(log_file);
        }
    }

    /// Copies selected text from the active terminal to clipboard
    pub fn copy_to_clipboard(&self) {
        if let Some(terminal) = self.get_active_terminal() {
            terminal.copy_clipboard_format(vte4::Format::Text);
        }
    }

    /// Pastes text from clipboard to the active terminal
    pub fn paste_from_clipboard(&self) {
        if let Some(terminal) = self.get_active_terminal() {
            terminal.paste_clipboard();
        }
    }

    /// Gets the terminal for the currently active tab
    #[must_use]
    pub fn get_active_terminal(&self) -> Option<Terminal> {
        let current_page = self.notebook.current_page()?;
        let sessions = self.sessions.borrow();

        // Find the session ID for the current page
        for (session_id, &page_num) in sessions.iter() {
            if page_num == current_page {
                return self.terminals.borrow().get(session_id).cloned();
            }
        }
        None
    }

    /// Gets the session ID for the currently active tab
    #[must_use]
    pub fn get_active_session_id(&self) -> Option<Uuid> {
        let current_page = self.notebook.current_page()?;
        self.get_session_id_for_page(current_page)
    }

    /// Gets the session ID for a specific page number
    #[must_use]
    pub fn get_session_id_for_page(&self, page_num: u32) -> Option<Uuid> {
        let sessions = self.sessions.borrow();
        for (session_id, &stored_page) in sessions.iter() {
            if stored_page == page_num {
                return Some(*session_id);
            }
        }
        None
    }

    /// Sends text to the active terminal
    pub fn send_text(&self, text: &str) {
        if let Some(terminal) = self.get_active_terminal() {
            terminal.feed_child(text.as_bytes());
        }
    }

    /// Sends text to a specific terminal session
    pub fn send_text_to_session(&self, session_id: Uuid, text: &str) {
        if let Some(terminal) = self.get_terminal(session_id) {
            terminal.feed_child(text.as_bytes());
        }
    }

    /// Returns the main widget for this notebook
    #[must_use]
    pub const fn widget(&self) -> &Notebook {
        &self.notebook
    }

    /// Returns the number of open tabs
    #[must_use]
    pub fn tab_count(&self) -> u32 {
        self.notebook.n_pages()
    }

    /// Switches to a specific tab by session ID
    pub fn switch_to_tab(&self, session_id: Uuid) {
        let sessions = self.sessions.borrow();
        if let Some(&page_num) = sessions.get(&session_id) {
            self.notebook.set_current_page(Some(page_num));
        }
    }

    /// Returns all session IDs
    #[must_use]
    pub fn session_ids(&self) -> Vec<Uuid> {
        self.sessions.borrow().keys().copied().collect()
    }

    /// Connects a callback for when a terminal child exits
    pub fn connect_child_exited<F>(&self, session_id: Uuid, callback: F)
    where
        F: Fn(i32) + 'static,
    {
        if let Some(terminal) = self.get_terminal(session_id) {
            terminal.connect_child_exited(move |_terminal, status| {
                callback(status);
            });
        }
    }

    /// Connects a callback for terminal output (for logging)
    pub fn connect_contents_changed<F>(&self, session_id: Uuid, callback: F)
    where
        F: Fn() + 'static,
    {
        if let Some(terminal) = self.get_terminal(session_id) {
            terminal.connect_contents_changed(move |_terminal| {
                callback();
            });
        }
    }
}

impl Default for TerminalNotebook {
    fn default() -> Self {
        Self::new()
    }
}
