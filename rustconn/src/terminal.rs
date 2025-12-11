//! Terminal notebook area
//!
//! This module provides the tabbed terminal interface using VTE4
//! for SSH sessions and placeholder tabs for RDP/VNC connections.

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, Box as GtkBox, Button, Label, Notebook, Orientation};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use uuid::Uuid;
use vte4::prelude::*;
use vte4::{PtyFlags, Terminal};

/// Terminal session information
#[derive(Debug, Clone)]
pub struct TerminalSession {
    /// Session UUID
    pub id: Uuid,
    /// Connection ID this session is for
    pub connection_id: Uuid,
    /// Connection name for display
    pub name: String,
    /// Protocol type (ssh, rdp, vnc)
    pub protocol: String,
    /// Whether this is an embedded terminal or external window
    pub is_embedded: bool,
    /// Log file path if logging is enabled
    pub log_file: Option<PathBuf>,
}

/// Terminal notebook widget for managing multiple terminal sessions
pub struct TerminalNotebook {
    notebook: Notebook,
    /// Map of session IDs to their tab indices
    sessions: Rc<RefCell<HashMap<Uuid, u32>>>,
    /// Map of session IDs to terminal widgets (for embedded sessions)
    terminals: Rc<RefCell<HashMap<Uuid, Terminal>>>,
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
            session_info: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Creates the welcome tab content
    fn create_welcome_tab() -> GtkBox {
        let container = GtkBox::new(Orientation::Vertical, 16);
        container.set_halign(gtk4::Align::Center);
        container.set_valign(gtk4::Align::Center);
        container.set_margin_start(32);
        container.set_margin_end(32);
        container.set_margin_top(32);
        container.set_margin_bottom(32);

        let title = Label::new(Some("Welcome to RustConn"));
        title.add_css_class("title-1");
        container.append(&title);

        let subtitle = Label::new(Some(
            "Select a connection from the sidebar to get started,\nor create a new connection.",
        ));
        subtitle.add_css_class("dim-label");
        subtitle.set_justify(gtk4::Justification::Center);
        container.append(&subtitle);

        container
    }

    /// Creates a new terminal tab for an SSH session
    ///
    /// This creates the terminal widget and prepares it for spawning a command.
    /// Returns the session UUID for tracking.
    pub fn create_terminal_tab(
        &self,
        connection_id: Uuid,
        title: &str,
        protocol: &str,
    ) -> Uuid {
        let session_id = Uuid::new_v4();

        // Create VTE terminal
        let terminal = Terminal::new();
        terminal.set_hexpand(true);
        terminal.set_vexpand(true);

        // Configure terminal appearance
        Self::configure_terminal(&terminal);

        // Create scrolled container for terminal
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&terminal)
            .build();

        // Create tab label with close button
        let tab_label = Self::create_tab_label(title, session_id, &self.notebook, &self.sessions);

        // Add the page
        let page_num = self.notebook.append_page(&scrolled, Some(&tab_label));

        // Store session mapping
        self.sessions.borrow_mut().insert(session_id, page_num as u32);
        self.terminals.borrow_mut().insert(session_id, terminal);
        
        // Store session info
        self.session_info.borrow_mut().insert(session_id, TerminalSession {
            id: session_id,
            connection_id,
            name: title.to_string(),
            protocol: protocol.to_string(),
            is_embedded: true,
            log_file: None,
        });

        // Switch to the new tab
        self.notebook.set_current_page(Some(page_num as u32));

        // Make the tab reorderable
        self.notebook.set_tab_reorderable(&scrolled, true);

        session_id
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
            gdk::RGBA::new(0.0, 0.0, 0.0, 1.0),       // Black
            gdk::RGBA::new(0.8, 0.0, 0.0, 1.0),       // Red
            gdk::RGBA::new(0.0, 0.8, 0.0, 1.0),       // Green
            gdk::RGBA::new(0.8, 0.8, 0.0, 1.0),       // Yellow
            gdk::RGBA::new(0.0, 0.0, 0.8, 1.0),       // Blue
            gdk::RGBA::new(0.8, 0.0, 0.8, 1.0),       // Magenta
            gdk::RGBA::new(0.0, 0.8, 0.8, 1.0),       // Cyan
            gdk::RGBA::new(0.8, 0.8, 0.8, 1.0),       // White
            gdk::RGBA::new(0.4, 0.4, 0.4, 1.0),       // Bright Black
            gdk::RGBA::new(1.0, 0.0, 0.0, 1.0),       // Bright Red
            gdk::RGBA::new(0.0, 1.0, 0.0, 1.0),       // Bright Green
            gdk::RGBA::new(1.0, 1.0, 0.0, 1.0),       // Bright Yellow
            gdk::RGBA::new(0.0, 0.0, 1.0, 1.0),       // Bright Blue
            gdk::RGBA::new(1.0, 0.0, 1.0, 1.0),       // Bright Magenta
            gdk::RGBA::new(0.0, 1.0, 1.0, 1.0),       // Bright Cyan
            gdk::RGBA::new(1.0, 1.0, 1.0, 1.0),       // Bright White
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
        let argv_refs: Vec<&str> = argv_gstr.iter().map(|s| s.as_str()).collect();

        // Convert envv if provided
        let envv_gstr: Option<Vec<glib::GString>> = envv.map(|e| {
            e.iter().map(|s| glib::GString::from(*s)).collect()
        });
        let envv_refs: Option<Vec<&str>> = envv_gstr.as_ref().map(|e| {
            e.iter().map(|s| s.as_str()).collect()
        });

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
                        eprintln!("Failed to spawn command: {}", e);
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
            format!("{}@{}", user, host)
        } else {
            host.to_string()
        };
        argv.push(&destination);

        self.spawn_command(session_id, &argv, None, None)
    }

    /// Creates a tab label with title and close button
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

        // Connect close button
        let notebook_weak = notebook.downgrade();
        let sessions_clone = sessions.clone();
        close_button.connect_clicked(move |_| {
            if let Some(notebook) = notebook_weak.upgrade() {
                let sessions = sessions_clone.borrow();
                if let Some(&page_num) = sessions.get(&session_id) {
                    notebook.remove_page(Some(page_num));
                }
            }
        });

        tab_box.append(&close_button);
        tab_box
    }

    /// Creates a placeholder tab for external connections (RDP/VNC)
    pub fn create_external_tab(
        &self,
        connection_id: Uuid,
        title: &str,
        protocol: &str,
    ) -> Uuid {
        let session_id = Uuid::new_v4();

        let container = GtkBox::new(Orientation::Vertical, 16);
        container.set_halign(gtk4::Align::Center);
        container.set_valign(gtk4::Align::Center);

        let icon_name = match protocol {
            "rdp" => "computer-symbolic",
            "vnc" => "video-display-symbolic",
            _ => "network-server-symbolic",
        };

        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(64);
        icon.add_css_class("dim-label");
        container.append(&icon);

        let label = Label::new(Some(&format!(
            "{} session running in external window",
            protocol.to_uppercase()
        )));
        label.add_css_class("dim-label");
        container.append(&label);

        let title_label = Label::new(Some(title));
        title_label.add_css_class("title-3");
        container.append(&title_label);

        // Create tab label with close button
        let tab_label = Self::create_tab_label(title, session_id, &self.notebook, &self.sessions);

        let page_num = self.notebook.append_page(&container, Some(&tab_label));
        self.sessions.borrow_mut().insert(session_id, page_num as u32);
        
        // Store session info
        self.session_info.borrow_mut().insert(session_id, TerminalSession {
            id: session_id,
            connection_id,
            name: title.to_string(),
            protocol: protocol.to_string(),
            is_embedded: false,
            log_file: None,
        });
        
        self.notebook.set_current_page(Some(page_num as u32));

        session_id
    }

    /// Closes a terminal tab by session ID
    pub fn close_tab(&self, session_id: Uuid) {
        let mut sessions = self.sessions.borrow_mut();
        if let Some(page_num) = sessions.remove(&session_id) {
            self.notebook.remove_page(Some(page_num));

            // Update page numbers for remaining sessions
            // (pages after the removed one shift down by 1)
            for (_, num) in sessions.iter_mut() {
                if *num > page_num {
                    *num -= 1;
                }
            }
        }
        
        // Clean up terminal and session info
        self.terminals.borrow_mut().remove(&session_id);
        self.session_info.borrow_mut().remove(&session_id);
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
        let sessions = self.sessions.borrow();
        
        for (session_id, &page_num) in sessions.iter() {
            if page_num == current_page {
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
    pub fn widget(&self) -> &Notebook {
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

    /// Creates an embedded session tab for RDP/VNC connections
    ///
    /// This creates an `EmbeddedSessionTab` widget and adds it to the notebook.
    /// On X11, the session will be embedded within the tab.
    /// On Wayland, a placeholder is shown and the session runs in an external window.
    ///
    /// # Arguments
    /// * `connection_id` - The connection UUID
    /// * `title` - Display name for the connection
    /// * `protocol` - Protocol type ("rdp" or "vnc")
    ///
    /// # Returns
    /// A tuple of (session_id, is_embedded) where is_embedded indicates if
    /// the session is embedded (X11) or external (Wayland)
    pub fn create_embedded_tab(
        &self,
        connection_id: Uuid,
        title: &str,
        protocol: &str,
    ) -> (Uuid, bool) {
        use crate::embedded::EmbeddedSessionTab;

        let (tab, is_embedded) = EmbeddedSessionTab::new(connection_id, title, protocol);
        let session_id = tab.id();

        // Create tab label with close button
        let tab_label = Self::create_tab_label(title, session_id, &self.notebook, &self.sessions);

        // Add the page
        let page_num = self.notebook.append_page(tab.widget(), Some(&tab_label));

        // Store session mapping
        self.sessions.borrow_mut().insert(session_id, page_num as u32);

        // Store session info
        self.session_info.borrow_mut().insert(
            session_id,
            TerminalSession {
                id: session_id,
                connection_id,
                name: title.to_string(),
                protocol: protocol.to_string(),
                is_embedded,
                log_file: None,
            },
        );

        // Switch to the new tab
        self.notebook.set_current_page(Some(page_num as u32));

        // Make the tab reorderable
        self.notebook.set_tab_reorderable(tab.widget(), true);

        (session_id, is_embedded)
    }

    /// Creates an embedded session tab and returns the tab widget for further configuration
    ///
    /// This is similar to `create_embedded_tab` but returns the `EmbeddedSessionTab`
    /// so the caller can configure it (e.g., start the session, set up callbacks).
    ///
    /// # Arguments
    /// * `connection_id` - The connection UUID
    /// * `title` - Display name for the connection
    /// * `protocol` - Protocol type ("rdp" or "vnc")
    ///
    /// # Returns
    /// A tuple of (EmbeddedSessionTab, is_embedded)
    pub fn create_embedded_tab_with_widget(
        &self,
        connection_id: Uuid,
        title: &str,
        protocol: &str,
    ) -> (crate::embedded::EmbeddedSessionTab, bool) {
        use crate::embedded::EmbeddedSessionTab;

        let (tab, is_embedded) = EmbeddedSessionTab::new(connection_id, title, protocol);
        let session_id = tab.id();

        // Create tab label with close button
        let tab_label = Self::create_tab_label(title, session_id, &self.notebook, &self.sessions);

        // Add the page
        let page_num = self.notebook.append_page(tab.widget(), Some(&tab_label));

        // Store session mapping
        self.sessions.borrow_mut().insert(session_id, page_num as u32);

        // Store session info
        self.session_info.borrow_mut().insert(
            session_id,
            TerminalSession {
                id: session_id,
                connection_id,
                name: title.to_string(),
                protocol: protocol.to_string(),
                is_embedded,
                log_file: None,
            },
        );

        // Switch to the new tab
        self.notebook.set_current_page(Some(page_num as u32));

        // Make the tab reorderable
        self.notebook.set_tab_reorderable(tab.widget(), true);

        (tab, is_embedded)
    }
}

impl Default for TerminalNotebook {
    fn default() -> Self {
        Self::new()
    }
}
