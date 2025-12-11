//! Main application window
//!
//! This module provides the main window implementation for RustConn,
//! including the header bar, sidebar, terminal area, and action handling.

use gtk4::prelude::*;
use gtk4::{
    gio, glib, Application, ApplicationWindow, Button, HeaderBar, Label,
    MenuButton, Orientation, Paned,
};
use std::rc::Rc;
use uuid::Uuid;
use chrono;

use crate::dialogs::{ConnectionDialog, ImportDialog, SettingsDialog, SnippetDialog};
use crate::sidebar::{ConnectionItem, ConnectionSidebar};
use crate::state::SharedAppState;
use crate::terminal::TerminalNotebook;

/// Shared sidebar type
type SharedSidebar = Rc<ConnectionSidebar>;
/// Shared terminal notebook type
type SharedNotebook = Rc<TerminalNotebook>;

/// Main application window wrapper
///
/// Provides access to the main window and its components.
pub struct MainWindow {
    window: ApplicationWindow,
    sidebar: SharedSidebar,
    terminal_notebook: SharedNotebook,
    state: SharedAppState,
    paned: Paned,
}

impl MainWindow {
    /// Creates a new main window for the application
    #[must_use]
    pub fn new(app: &Application, state: SharedAppState) -> Self {
        // Register custom icon from assets before creating window
        Self::register_app_icon();
        
        // Create the main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("RustConn")
            .default_width(1200)
            .default_height(800)
            .icon_name("org.rustconn.RustConn")
            .build();

        // Apply saved window geometry if available
        {
            let state_ref = state.borrow();
            let settings = state_ref.settings();
            if settings.ui.remember_window_geometry {
                if let (Some(width), Some(height)) = (settings.ui.window_width, settings.ui.window_height) {
                    if width > 0 && height > 0 {
                        window.set_default_size(width, height);
                    }
                }
            }
        }

        // Create header bar
        let header_bar = Self::create_header_bar();
        window.set_titlebar(Some(&header_bar));

        // Create the main layout with paned container
        let paned = Paned::new(Orientation::Horizontal);
        
        // Apply saved sidebar width
        {
            let state_ref = state.borrow();
            let settings = state_ref.settings();
            let sidebar_width = settings.ui.sidebar_width.unwrap_or(280);
            paned.set_position(sidebar_width);
        }
        
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);

        // Create sidebar
        let sidebar = Rc::new(ConnectionSidebar::new());
        paned.set_start_child(Some(sidebar.widget()));

        // Create terminal notebook area
        let terminal_notebook = Rc::new(TerminalNotebook::new());
        paned.set_end_child(Some(terminal_notebook.widget()));

        window.set_child(Some(&paned));

        let main_window = Self {
            window,
            sidebar,
            terminal_notebook,
            state,
            paned,
        };

        // Set up window actions
        main_window.setup_actions();
        
        // Load initial data
        main_window.load_connections();
        
        // Connect signals
        main_window.connect_signals();

        main_window
    }

    /// Creates the header bar with title and controls
    fn create_header_bar() -> HeaderBar {
        let header_bar = HeaderBar::new();
        header_bar.set_show_title_buttons(true);

        // Add title
        let title = Label::new(Some("RustConn"));
        title.add_css_class("title");
        header_bar.set_title_widget(Some(&title));

        // Local shell button (leftmost)
        let local_shell_button = Button::from_icon_name("utilities-terminal-symbolic");
        local_shell_button.set_tooltip_text(Some("Local Shell"));
        local_shell_button.set_action_name(Some("win.local-shell"));
        header_bar.pack_start(&local_shell_button);

        // Add connection button
        let add_button = Button::from_icon_name("list-add-symbolic");
        add_button.set_tooltip_text(Some("New Connection (Ctrl+N)"));
        add_button.set_action_name(Some("win.new-connection"));
        header_bar.pack_start(&add_button);

        // Add import button
        let import_button = Button::from_icon_name("document-open-symbolic");
        import_button.set_tooltip_text(Some("Import Connections (Ctrl+I)"));
        import_button.set_action_name(Some("win.import"));
        header_bar.pack_start(&import_button);

        // Add menu button
        let menu_button = MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menu")
            .build();
        
        let menu = Self::create_app_menu();
        menu_button.set_menu_model(Some(&menu));
        header_bar.pack_end(&menu_button);

        // Add settings button
        let settings_button = Button::from_icon_name("emblem-system-symbolic");
        settings_button.set_tooltip_text(Some("Settings (Ctrl+,)"));
        settings_button.set_action_name(Some("win.settings"));
        header_bar.pack_end(&settings_button);

        header_bar
    }

    /// Creates the application menu
    fn create_app_menu() -> gio::Menu {
        let menu = gio::Menu::new();
        
        // Connection section
        let conn_section = gio::Menu::new();
        conn_section.append(Some("New Connection"), Some("win.new-connection"));
        conn_section.append(Some("New Group"), Some("win.new-group"));
        conn_section.append(Some("Quick Connect"), Some("win.quick-connect"));
        conn_section.append(Some("Local Shell"), Some("win.local-shell"));
        menu.append_section(None, &conn_section);
        
        // Snippet section
        let snippet_section = gio::Menu::new();
        snippet_section.append(Some("New Snippet"), Some("win.new-snippet"));
        snippet_section.append(Some("Manage Snippets"), Some("win.manage-snippets"));
        menu.append_section(None, &snippet_section);
        
        // Session section
        let session_section = gio::Menu::new();
        session_section.append(Some("Active Sessions"), Some("win.show-sessions"));
        menu.append_section(None, &session_section);
        
        // Import/Export section
        let io_section = gio::Menu::new();
        io_section.append(Some("Import..."), Some("win.import"));
        io_section.append(Some("Export..."), Some("win.export"));
        menu.append_section(None, &io_section);
        
        // Edit section
        let edit_section = gio::Menu::new();
        edit_section.append(Some("Copy"), Some("win.copy"));
        edit_section.append(Some("Paste"), Some("win.paste"));
        menu.append_section(None, &edit_section);
        
        // App section
        let app_section = gio::Menu::new();
        app_section.append(Some("Settings"), Some("win.settings"));
        app_section.append(Some("About"), Some("app.about"));
        app_section.append(Some("Quit"), Some("app.quit"));
        menu.append_section(None, &app_section);
        
        menu
    }

    /// Sets up window actions
    fn setup_actions(&self) {
        let window = &self.window;
        let state = self.state.clone();
        let sidebar = self.sidebar.clone();
        let terminal_notebook = self.terminal_notebook.clone();

        // New connection action
        let new_conn_action = gio::SimpleAction::new("new-connection", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        new_conn_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_new_connection_dialog(&win, state_clone.clone(), sidebar_clone.clone());
            }
        });
        window.add_action(&new_conn_action);

        // New group action
        let new_group_action = gio::SimpleAction::new("new-group", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        new_group_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_new_group_dialog(&win, state_clone.clone(), sidebar_clone.clone());
            }
        });
        window.add_action(&new_group_action);

        // Import action
        let import_action = gio::SimpleAction::new("import", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        import_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_import_dialog(&win, state_clone.clone(), sidebar_clone.clone());
            }
        });
        window.add_action(&import_action);

        // Settings action
        let settings_action = gio::SimpleAction::new("settings", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        settings_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_settings_dialog(&win, state_clone.clone());
            }
        });
        window.add_action(&settings_action);

        // Search action
        let search_action = gio::SimpleAction::new("search", None);
        let sidebar_clone = sidebar.clone();
        search_action.connect_activate(move |_, _| {
            sidebar_clone.search_entry().grab_focus();
        });
        window.add_action(&search_action);

        // Copy action
        let copy_action = gio::SimpleAction::new("copy", None);
        let notebook_clone = terminal_notebook.clone();
        copy_action.connect_activate(move |_, _| {
            notebook_clone.copy_to_clipboard();
        });
        window.add_action(&copy_action);

        // Paste action
        let paste_action = gio::SimpleAction::new("paste", None);
        let notebook_clone = terminal_notebook.clone();
        paste_action.connect_activate(move |_, _| {
            notebook_clone.paste_from_clipboard();
        });
        window.add_action(&paste_action);

        // Close tab action
        let close_tab_action = gio::SimpleAction::new("close-tab", None);
        let notebook_clone = terminal_notebook.clone();
        close_tab_action.connect_activate(move |_, _| {
            if let Some(session_id) = notebook_clone.get_active_session_id() {
                notebook_clone.close_tab(session_id);
            }
        });
        window.add_action(&close_tab_action);

        // Connect action (for connecting to selected connection)
        let connect_action = gio::SimpleAction::new("connect", None);
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        let notebook_clone = terminal_notebook.clone();
        connect_action.connect_activate(move |_, _| {
            Self::connect_selected(&state_clone, &sidebar_clone, &notebook_clone);
        });
        window.add_action(&connect_action);

        // Edit connection action
        let edit_action = gio::SimpleAction::new("edit-connection", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        edit_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::edit_selected_connection(&win, &state_clone, &sidebar_clone);
            }
        });
        window.add_action(&edit_action);

        // Delete connection action
        let delete_action = gio::SimpleAction::new("delete-connection", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        delete_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::delete_selected_connection(&win, &state_clone, &sidebar_clone);
            }
        });
        window.add_action(&delete_action);

        // Duplicate connection action
        let duplicate_action = gio::SimpleAction::new("duplicate-connection", None);
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        duplicate_action.connect_activate(move |_, _| {
            Self::duplicate_selected_connection(&state_clone, &sidebar_clone);
        });
        window.add_action(&duplicate_action);

        // Move to group action
        let move_to_group_action = gio::SimpleAction::new("move-to-group", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        move_to_group_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_move_to_group_dialog(&win, &state_clone, &sidebar_clone);
            }
        });
        window.add_action(&move_to_group_action);

        // Focus sidebar action
        let focus_sidebar_action = gio::SimpleAction::new("focus-sidebar", None);
        let sidebar_clone = sidebar.clone();
        focus_sidebar_action.connect_activate(move |_, _| {
            sidebar_clone.list_view().grab_focus();
        });
        window.add_action(&focus_sidebar_action);

        // Focus terminal action
        let focus_terminal_action = gio::SimpleAction::new("focus-terminal", None);
        let notebook_clone = terminal_notebook.clone();
        focus_terminal_action.connect_activate(move |_, _| {
            if let Some(terminal) = notebook_clone.get_active_terminal() {
                terminal.grab_focus();
            }
        });
        window.add_action(&focus_terminal_action);

        // Next tab action
        let next_tab_action = gio::SimpleAction::new("next-tab", None);
        let notebook_clone = terminal_notebook.clone();
        next_tab_action.connect_activate(move |_, _| {
            let widget = notebook_clone.widget();
            let current = widget.current_page().unwrap_or(0);
            let total = widget.n_pages();
            if total > 0 {
                let next = (current + 1) % total;
                widget.set_current_page(Some(next));
            }
        });
        window.add_action(&next_tab_action);

        // Previous tab action
        let prev_tab_action = gio::SimpleAction::new("prev-tab", None);
        let notebook_clone = terminal_notebook.clone();
        prev_tab_action.connect_activate(move |_, _| {
            let widget = notebook_clone.widget();
            let current = widget.current_page().unwrap_or(0);
            let total = widget.n_pages();
            if total > 0 {
                let prev = if current == 0 { total - 1 } else { current - 1 };
                widget.set_current_page(Some(prev));
            }
        });
        window.add_action(&prev_tab_action);

        // Local shell action
        let local_shell_action = gio::SimpleAction::new("local-shell", None);
        let notebook_clone = terminal_notebook.clone();
        local_shell_action.connect_activate(move |_, _| {
            Self::open_local_shell(&notebook_clone);
        });
        window.add_action(&local_shell_action);

        // Quick connect action
        let quick_connect_action = gio::SimpleAction::new("quick-connect", None);
        let window_weak = window.downgrade();
        let notebook_clone = terminal_notebook.clone();
        quick_connect_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_quick_connect_dialog(&win, notebook_clone.clone());
            }
        });
        window.add_action(&quick_connect_action);

        // Group operations action (toggle mode)
        let group_ops_action = gio::SimpleAction::new_stateful(
            "group-operations",
            None,
            &false.to_variant(),
        );
        let sidebar_clone = sidebar.clone();
        group_ops_action.connect_activate(move |action, _| {
            let current = action.state().and_then(|v| v.get::<bool>()).unwrap_or(false);
            action.set_state(&(!current).to_variant());
            Self::toggle_group_operations_mode(&sidebar_clone, !current);
        });
        window.add_action(&group_ops_action);

        // Select all action (for group operations mode)
        let select_all_action = gio::SimpleAction::new("select-all", None);
        let sidebar_clone = sidebar.clone();
        select_all_action.connect_activate(move |_, _| {
            if sidebar_clone.is_group_operations_mode() {
                sidebar_clone.select_all();
            }
        });
        window.add_action(&select_all_action);

        // Clear selection action (for group operations mode)
        let clear_selection_action = gio::SimpleAction::new("clear-selection", None);
        let sidebar_clone = sidebar.clone();
        clear_selection_action.connect_activate(move |_, _| {
            sidebar_clone.clear_selection();
        });
        window.add_action(&clear_selection_action);

        // Delete selected action (bulk delete)
        let delete_selected_action = gio::SimpleAction::new("delete-selected", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        delete_selected_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::delete_selected_connections(&win, &state_clone, &sidebar_clone);
            }
        });
        window.add_action(&delete_selected_action);

        // Move selected to group action
        let move_selected_action = gio::SimpleAction::new("move-selected-to-group", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        move_selected_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_move_selected_to_group_dialog(&win, &state_clone, &sidebar_clone);
            }
        });
        window.add_action(&move_selected_action);

        // Sort connections action
        let sort_action = gio::SimpleAction::new("sort-connections", None);
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        sort_action.connect_activate(move |_, _| {
            Self::sort_connections(&state_clone, &sidebar_clone);
        });
        window.add_action(&sort_action);

        // Export action
        let export_action = gio::SimpleAction::new("export", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        export_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_export_dialog(&win, state_clone.clone());
            }
        });
        window.add_action(&export_action);

        // New snippet action
        let new_snippet_action = gio::SimpleAction::new("new-snippet", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        new_snippet_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_new_snippet_dialog(&win, state_clone.clone());
            }
        });
        window.add_action(&new_snippet_action);

        // Manage snippets action
        let manage_snippets_action = gio::SimpleAction::new("manage-snippets", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let notebook_clone = terminal_notebook.clone();
        manage_snippets_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_snippets_manager(&win, state_clone.clone(), notebook_clone.clone());
            }
        });
        window.add_action(&manage_snippets_action);

        // Execute snippet action (for running a snippet in active terminal)
        let execute_snippet_action = gio::SimpleAction::new("execute-snippet", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let notebook_clone = terminal_notebook.clone();
        execute_snippet_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_snippet_picker(&win, state_clone.clone(), notebook_clone.clone());
            }
        });
        window.add_action(&execute_snippet_action);

        // Show sessions action
        let show_sessions_action = gio::SimpleAction::new("show-sessions", None);
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let notebook_clone = terminal_notebook.clone();
        show_sessions_action.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                Self::show_sessions_manager(&win, state_clone.clone(), notebook_clone.clone());
            }
        });
        window.add_action(&show_sessions_action);
    }

    /// Connects UI signals
    fn connect_signals(&self) {
        let state = self.state.clone();
        let sidebar = self.sidebar.clone();
        let terminal_notebook = self.terminal_notebook.clone();
        let paned = self.paned.clone();
        let window = self.window.clone();

        // Connect sidebar search
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        sidebar.search_entry().connect_search_changed(move |entry| {
            let query = entry.text();
            Self::filter_connections(&state_clone, &sidebar_clone, &query);
        });

        // Connect sidebar double-click to connect
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        let notebook_clone = terminal_notebook.clone();
        sidebar.list_view().connect_activate(move |_, position| {
            Self::connect_at_position(&state_clone, &sidebar_clone, &notebook_clone, position);
        });

        // Save window state on close
        let state_clone = state.clone();
        let paned_clone = paned.clone();
        window.connect_close_request(move |win| {
            // Save window geometry
            let (width, height) = win.default_size();
            let sidebar_width = paned_clone.position();
            
            if let Ok(mut state) = state_clone.try_borrow_mut() {
                let mut settings = state.settings().clone();
                if settings.ui.remember_window_geometry {
                    settings.ui.window_width = Some(width);
                    settings.ui.window_height = Some(height);
                    settings.ui.sidebar_width = Some(sidebar_width);
                    let _ = state.update_settings(settings);
                }
            }
            
            glib::Propagation::Proceed
        });
    }

    /// Loads connections into the sidebar
    fn load_connections(&self) {
        let state = self.state.borrow();
        let store = self.sidebar.store();
        
        // Clear existing items
        store.remove_all();
        
        // Add root groups
        for group in state.get_root_groups() {
            let group_item = ConnectionItem::new_group(&group.id.to_string(), &group.name);
            self.add_group_children(&state, &group_item, group.id);
            store.append(&group_item);
        }
        
        // Add ungrouped connections
        for conn in state.get_ungrouped_connections() {
            let protocol = match &conn.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
            };
            let item = ConnectionItem::new_connection(
                &conn.id.to_string(),
                &conn.name,
                protocol,
                &conn.host,
            );
            store.append(&item);
        }
    }

    /// Recursively adds group children
    fn add_group_children(&self, state: &std::cell::Ref<crate::state::AppState>, parent_item: &ConnectionItem, group_id: Uuid) {
        // Add child groups
        for child_group in state.get_child_groups(group_id) {
            let child_item = ConnectionItem::new_group(&child_group.id.to_string(), &child_group.name);
            self.add_group_children(state, &child_item, child_group.id);
            parent_item.add_child(&child_item);
        }
        
        // Add connections in this group
        for conn in state.get_connections_by_group(group_id) {
            let protocol = match &conn.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
            };
            let item = ConnectionItem::new_connection(
                &conn.id.to_string(),
                &conn.name,
                protocol,
                &conn.host,
            );
            parent_item.add_child(&item);
        }
    }

    /// Filters connections based on search query
    fn filter_connections(state: &SharedAppState, sidebar: &SharedSidebar, query: &str) {
        let store = sidebar.store();
        store.remove_all();
        
        let state_ref = state.borrow();
        
        if query.is_empty() {
            // Show all connections in hierarchy
            drop(state_ref);
            // Re-load full hierarchy - need to call load_connections differently
            // For now, just show all connections flat
            let state_ref = state.borrow();
            for conn in state_ref.list_connections() {
                let protocol = match &conn.protocol_config {
                    rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                    rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                    rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
                };
                let item = ConnectionItem::new_connection(
                    &conn.id.to_string(),
                    &conn.name,
                    protocol,
                    &conn.host,
                );
                store.append(&item);
            }
        } else {
            // Show filtered results
            for conn in state_ref.search_connections(query) {
                let protocol = match &conn.protocol_config {
                    rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                    rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                    rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
                };
                let item = ConnectionItem::new_connection(
                    &conn.id.to_string(),
                    &conn.name,
                    protocol,
                    &conn.host,
                );
                store.append(&item);
            }
        }
    }

    /// Connects to the selected connection
    fn connect_selected(_state: &SharedAppState, _sidebar: &SharedSidebar, _notebook: &SharedNotebook) {
        // Get selected item from sidebar
        // This is a simplified version - in practice you'd get the selection from the list view
        // For now, we'll implement the connection logic
    }

    /// Connects to a connection at a specific position
    fn connect_at_position(state: &SharedAppState, sidebar: &SharedSidebar, notebook: &SharedNotebook, position: u32) {
        // Get the item at position and connect
        let store = sidebar.store();
        if let Some(item) = store.item(position) {
            if let Some(conn_item) = item.downcast_ref::<ConnectionItem>() {
                if !conn_item.is_group() {
                    let id_str = conn_item.id();
                    if let Ok(conn_id) = Uuid::parse_str(&id_str) {
                        Self::start_connection(state, notebook, conn_id);
                    }
                }
            }
        }
    }

    /// Starts a connection
    fn start_connection(state: &SharedAppState, notebook: &SharedNotebook, connection_id: Uuid) {
        let state_ref = state.borrow();
        
        if let Some(conn) = state_ref.get_connection(connection_id) {
            let protocol = match &conn.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
            };
            
            // Check if logging is enabled
            let logging_enabled = state_ref.settings().logging.enabled;
            let conn_name = conn.name.clone();
            
            if protocol == "ssh" {
                // Create terminal tab for SSH
                let session_id = notebook.create_terminal_tab(
                    connection_id,
                    &conn.name,
                    protocol,
                );
                
                // Build and spawn SSH command
                let port = conn.port;
                let host = conn.host.clone();
                let username = conn.username.clone();
                
                // Get SSH-specific options
                let (identity_file, extra_args) = if let rustconn_core::ProtocolConfig::Ssh(ssh_config) = &conn.protocol_config {
                    let key = ssh_config.key_path.as_ref().map(|p| p.to_string_lossy().to_string());
                    let mut args = Vec::new();
                    
                    if let Some(proxy) = &ssh_config.proxy_jump {
                        args.push("-J".to_string());
                        args.push(proxy.clone());
                    }
                    
                    if ssh_config.use_control_master {
                        args.push("-o".to_string());
                        args.push("ControlMaster=auto".to_string());
                    }
                    
                    for (k, v) in &ssh_config.custom_options {
                        args.push("-o".to_string());
                        args.push(format!("{}={}", k, v));
                    }
                    
                    (key, args)
                } else {
                    (None, Vec::new())
                };
                
                drop(state_ref);
                
                // Set up session logging if enabled
                if logging_enabled {
                    Self::setup_session_logging(state, notebook, session_id, connection_id, &conn_name);
                }
                
                // Wire up child exited callback for session cleanup
                Self::setup_child_exited_handler(state, notebook, session_id);
                
                // Spawn SSH
                let extra_refs: Vec<&str> = extra_args.iter().map(|s| s.as_str()).collect();
                notebook.spawn_ssh(
                    session_id,
                    &host,
                    port,
                    username.as_deref(),
                    identity_file.as_deref(),
                    &extra_refs,
                );
            } else {
                // Create embedded tab for RDP/VNC
                let conn_name = conn.name.clone();
                let host = conn.host.clone();
                let port = conn.port;
                let username = conn.username.clone();
                
                // Get protocol-specific configuration
                let (rdp_config, vnc_config) = match &conn.protocol_config {
                    rustconn_core::ProtocolConfig::Rdp(rdp) => (Some(rdp.clone()), None),
                    rustconn_core::ProtocolConfig::Vnc(vnc) => (None, Some(vnc.clone())),
                    _ => (None, None),
                };
                
                drop(state_ref);
                
                // Create embedded session tab
                let (tab, is_embedded) = notebook.create_embedded_tab_with_widget(
                    connection_id,
                    &conn_name,
                    protocol,
                );
                
                // Start the session based on protocol
                let result = if protocol == "rdp" {
                    Self::start_rdp_session(&tab, &host, port, username.as_deref(), rdp_config.as_ref())
                } else {
                    Self::start_vnc_session(&tab, &host, port, vnc_config.as_ref())
                };
                
                // Handle embedding failures gracefully
                match result {
                    Ok(()) => {
                        // Session started successfully
                        if is_embedded {
                            tab.set_status(&format!("Connected to {}", conn_name));
                        } else {
                            // Wayland fallback - session running in external window
                            tab.set_status(&format!("External window - {}", conn_name));
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to start {} session: {}", protocol.to_uppercase(), e);
                        
                        // Update tab status with error
                        tab.set_status(&format!("Error: {}", e));
                        
                        // Show user notification about the failure
                        // The tab already shows the error status, and on Wayland
                        // the placeholder explains the external window situation
                        if !is_embedded {
                            // On Wayland, the session may still be running in external window
                            // even if we got an error (e.g., process spawn succeeded but
                            // embedding wasn't possible)
                            tab.set_status(&format!("External window - {} (check external window)", conn_name));
                        }
                    }
                }
                
                // Register session with session manager
                if let Ok(mut state_mut) = state.try_borrow_mut() {
                    if let Err(e) = state_mut.start_session(connection_id, None) {
                        eprintln!("Failed to register session: {}", e);
                    }
                }
            }
        }
    }
    
    /// Sets up session logging for a terminal session
    fn setup_session_logging(
        state: &SharedAppState,
        notebook: &SharedNotebook,
        session_id: Uuid,
        connection_id: Uuid,
        connection_name: &str,
    ) {
        // Get the session logger from state and create a log file
        let log_path = {
            let state_ref = state.borrow();
            if let Some(logger) = state_ref.session_manager().logger() {
                match logger.create_log_file(connection_id, connection_name) {
                    Ok(path) => Some(path),
                    Err(e) => {
                        eprintln!("Warning: Failed to create log file: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        };
        
        // Set the log file path on the terminal session
        if let Some(path) = log_path {
            notebook.set_log_file(session_id, path.clone());
            
            // Wire up contents changed callback for logging
            Self::setup_contents_changed_handler(notebook, session_id, path);
        }
    }
    
    /// Sets up the child exited handler for session cleanup
    fn setup_child_exited_handler(
        state: &SharedAppState,
        notebook: &SharedNotebook,
        session_id: Uuid,
    ) {
        let state_clone = state.clone();
        let notebook_clone = notebook.clone();
        
        notebook.connect_child_exited(session_id, move |exit_status| {
            // Update session status in state manager
            if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
                // Terminate the session in the session manager
                let _ = state_mut.terminate_session(session_id);
            }
            
            // Finalize the log file if logging was enabled
            if let Some(info) = notebook_clone.get_session_info(session_id) {
                if let Some(ref log_path) = info.log_file {
                    if let Err(e) = rustconn_core::SessionLogger::finalize_log(log_path) {
                        eprintln!("Warning: Failed to finalize log file: {}", e);
                    }
                }
            }
            
            // Log the exit status for debugging
            if exit_status != 0 {
                eprintln!("Session {} exited with status: {}", session_id, exit_status);
            }
        });
    }
    
    /// Sets up the contents changed handler for session logging
    /// 
    /// Note: VTE4's Rust bindings have limited support for extracting terminal text.
    /// This handler logs activity timestamps when terminal content changes.
    /// For full session logging, consider using the script command or terminal recording.
    fn setup_contents_changed_handler(
        notebook: &SharedNotebook,
        session_id: Uuid,
        log_path: std::path::PathBuf,
    ) {
        use std::cell::RefCell;
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::rc::Rc;
        
        // Create a shared writer for the log file
        let log_writer: Rc<RefCell<Option<std::io::BufWriter<std::fs::File>>>> = 
            Rc::new(RefCell::new(None));
        
        // Debounce counter to avoid excessive logging
        let change_counter: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let last_log_time: Rc<RefCell<std::time::Instant>> = 
            Rc::new(RefCell::new(std::time::Instant::now()));
        
        // Open the log file for appending
        if let Ok(file) = OpenOptions::new().append(true).open(&log_path) {
            *log_writer.borrow_mut() = Some(std::io::BufWriter::new(file));
        }
        
        let log_writer_clone = log_writer.clone();
        let counter_clone = change_counter.clone();
        let last_time_clone = last_log_time.clone();
        
        notebook.connect_contents_changed(session_id, move || {
            // Increment change counter
            let mut counter = counter_clone.borrow_mut();
            *counter += 1;
            
            // Debounce: only log every 100 changes or every 5 seconds
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(*last_time_clone.borrow());
            
            if *counter >= 100 || elapsed.as_secs() >= 5 {
                if let Ok(mut writer_opt) = log_writer_clone.try_borrow_mut() {
                    if let Some(ref mut writer) = *writer_opt {
                        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                        let _ = writeln!(writer, "[{}] Terminal activity ({} changes)", timestamp, *counter);
                        let _ = writer.flush();
                    }
                }
                
                // Reset counter and time
                *counter = 0;
                *last_time_clone.borrow_mut() = now;
            }
        });
    }

    /// Shows the new connection dialog
    fn show_new_connection_dialog(window: &ApplicationWindow, state: SharedAppState, sidebar: SharedSidebar) {
        let dialog = ConnectionDialog::new(Some(&window.clone().upcast()));
        dialog.setup_key_file_chooser(Some(&window.clone().upcast()));
        
        let window_clone = window.clone();
        dialog.run(move |result| {
            if let Some(conn) = result {
                if let Ok(mut state_mut) = state.try_borrow_mut() {
                    match state_mut.create_connection(conn) {
                        Ok(_) => {
                            // Reload sidebar
                            drop(state_mut);
                            Self::reload_sidebar(&state, &sidebar);
                        }
                        Err(e) => {
                            // Show error in UI dialog with proper transient parent
                            let alert = gtk4::AlertDialog::builder()
                                .message("Error Creating Connection")
                                .detail(&e)
                                .modal(true)
                                .build();
                            alert.show(Some(&window_clone));
                        }
                    }
                }
            }
        });
    }

    /// Shows the new group dialog with optional parent selection
    fn show_new_group_dialog(window: &ApplicationWindow, state: SharedAppState, sidebar: SharedSidebar) {
        Self::show_new_group_dialog_with_parent(window, state, sidebar, None);
    }

    /// Shows the new group dialog with parent group selection
    fn show_new_group_dialog_with_parent(
        window: &ApplicationWindow,
        state: SharedAppState,
        sidebar: SharedSidebar,
        preselected_parent: Option<Uuid>,
    ) {
        // Using Window instead of deprecated Dialog
        let entry = gtk4::Entry::new();
        entry.set_placeholder_text(Some("Group name"));
        
        let group_window = gtk4::Window::builder()
            .title("New Group")
            .transient_for(window)
            .modal(true)
            .default_width(350)
            .build();
        
        // Create header bar with Cancel/Create buttons
        let header = gtk4::HeaderBar::new();
        let cancel_btn = gtk4::Button::builder().label("Cancel").build();
        let create_btn = gtk4::Button::builder().label("Create").css_classes(["suggested-action"]).build();
        header.pack_start(&cancel_btn);
        header.pack_end(&create_btn);
        group_window.set_titlebar(Some(&header));
        
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        
        // Group name
        let name_label = Label::new(Some("Group name:"));
        name_label.set_halign(gtk4::Align::Start);
        content.append(&name_label);
        content.append(&entry);
        
        // Parent group dropdown
        let parent_label = Label::new(Some("Parent group (optional):"));
        parent_label.set_halign(gtk4::Align::Start);
        parent_label.set_margin_top(8);
        content.append(&parent_label);
        
        let parent_dropdown = gtk4::DropDown::from_strings(&["(None - Root Level)"]);
        
        // Populate parent dropdown with existing groups
        let state_ref = state.borrow();
        let groups: Vec<_> = state_ref.list_groups().iter().map(|g| (*g).clone()).collect();
        drop(state_ref);
        
        let mut group_ids: Vec<Option<Uuid>> = vec![None];
        let mut strings: Vec<String> = vec!["(None - Root Level)".to_string()];
        let mut preselected_index = 0u32;
        
        for group in &groups {
            let state_ref = state.borrow();
            let path = state_ref.get_group_path(group.id).unwrap_or_else(|| group.name.clone());
            drop(state_ref);
            
            strings.push(path);
            group_ids.push(Some(group.id));
            
            if preselected_parent == Some(group.id) {
                preselected_index = (group_ids.len() - 1) as u32;
            }
        }
        
        let string_list = gtk4::StringList::new(&strings.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        parent_dropdown.set_model(Some(&string_list));
        parent_dropdown.set_selected(preselected_index);
        
        content.append(&parent_dropdown);
        group_window.set_child(Some(&content));
        
        // Connect cancel button
        let window_clone = group_window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_clone.close();
        });
        
        // Connect create button
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        let window_clone = group_window.clone();
        let entry_clone = entry.clone();
        let dropdown_clone = parent_dropdown.clone();
        create_btn.connect_clicked(move |_| {
            let name = entry_clone.text().to_string();
            if name.trim().is_empty() {
                let alert = gtk4::AlertDialog::builder()
                    .message("Validation Error")
                    .detail("Group name cannot be empty")
                    .modal(true)
                    .build();
                alert.show(Some(&window_clone));
                return;
            }
            
            let selected_idx = dropdown_clone.selected() as usize;
            let parent_id = if selected_idx < group_ids.len() {
                group_ids[selected_idx]
            } else {
                None
            };
            
            if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
                let result = if let Some(pid) = parent_id {
                    state_mut.create_group_with_parent(name, pid)
                } else {
                    state_mut.create_group(name)
                };
                
                match result {
                    Ok(_) => {
                        drop(state_mut);
                        Self::reload_sidebar(&state_clone, &sidebar_clone);
                        window_clone.close();
                    }
                    Err(e) => {
                        let alert = gtk4::AlertDialog::builder()
                            .message("Error")
                            .detail(&e)
                            .modal(true)
                            .build();
                        alert.show(Some(&window_clone));
                    }
                }
            }
        });

        group_window.present();
    }

    /// Shows the import dialog
    fn show_import_dialog(window: &ApplicationWindow, state: SharedAppState, sidebar: SharedSidebar) {
        let dialog = ImportDialog::new(Some(&window.clone().upcast()));

        let window_clone = window.clone();
        dialog.run_with_source(move |result, source_name| {
            if let Some(import_result) = result {
                if let Ok(mut state_mut) = state.try_borrow_mut() {
                    match state_mut.import_connections_with_source(&import_result, &source_name) {
                        Ok(count) => {
                            drop(state_mut);
                            Self::reload_sidebar(&state, &sidebar);
                            // Show success message with proper transient parent
                            let alert = gtk4::AlertDialog::builder()
                                .message("Import Successful")
                                .detail(&format!(
                                    "Imported {} connections to '{}' group",
                                    count, source_name
                                ))
                                .modal(true)
                                .build();
                            alert.show(Some(&window_clone));
                        }
                        Err(e) => {
                            let alert = gtk4::AlertDialog::builder()
                                .message("Import Failed")
                                .detail(&e)
                                .modal(true)
                                .build();
                            alert.show(Some(&window_clone));
                        }
                    }
                }
            }
        });
    }

    /// Shows the settings dialog
    fn show_settings_dialog(window: &ApplicationWindow, state: SharedAppState) {
        let dialog = SettingsDialog::new(Some(&window.clone().upcast()));
        
        // Load current settings
        {
            let state_ref = state.borrow();
            dialog.set_settings(state_ref.settings());
        }
        
        dialog.run(move |result| {
            if let Some(settings) = result {
                if let Ok(mut state_mut) = state.try_borrow_mut() {
                    if let Err(e) = state_mut.update_settings(settings) {
                        eprintln!("Failed to save settings: {}", e);
                    }
                }
            }
        });
    }

    /// Edits the selected connection or group
    fn edit_selected_connection(window: &ApplicationWindow, state: &SharedAppState, sidebar: &SharedSidebar) {
        // Get selected item using sidebar's method (works in both single and multi-selection modes)
        let Some(conn_item) = sidebar.get_selected_item() else { return };
        
        let id_str = conn_item.id();
        let Ok(id) = Uuid::parse_str(&id_str) else { return };
        
        if conn_item.is_group() {
            // Edit group - show simple rename dialog
            Self::show_edit_group_dialog(window, state.clone(), sidebar.clone(), id);
        } else {
            // Edit connection
            let state_ref = state.borrow();
            let Some(conn) = state_ref.get_connection(id).cloned() else { return };
            drop(state_ref);
            
            let dialog = ConnectionDialog::new(Some(&window.clone().upcast()));
            dialog.setup_key_file_chooser(Some(&window.clone().upcast()));
            dialog.set_connection(&conn);
            
            let state_clone = state.clone();
            let sidebar_clone = sidebar.clone();
            let window_clone = window.clone();
            dialog.run(move |result| {
                if let Some(updated_conn) = result {
                    if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
                        match state_mut.update_connection(id, updated_conn) {
                            Ok(()) => {
                                drop(state_mut);
                                Self::reload_sidebar(&state_clone, &sidebar_clone);
                            }
                            Err(e) => {
                                let alert = gtk4::AlertDialog::builder()
                                    .message("Error Updating Connection")
                                    .detail(&e)
                                    .modal(true)
                                    .build();
                                alert.show(Some(&window_clone));
                            }
                        }
                    }
                }
            });
        }
    }

    /// Shows dialog to edit a group name
    fn show_edit_group_dialog(window: &ApplicationWindow, state: SharedAppState, sidebar: SharedSidebar, group_id: Uuid) {
        let state_ref = state.borrow();
        let Some(group) = state_ref.get_group(group_id).cloned() else { return };
        drop(state_ref);
        
        let entry = gtk4::Entry::new();
        entry.set_text(&group.name);
        
        let group_window = gtk4::Window::builder()
            .title("Edit Group")
            .transient_for(window)
            .modal(true)
            .default_width(300)
            .build();
        
        let header = gtk4::HeaderBar::new();
        let cancel_btn = gtk4::Button::builder().label("Cancel").build();
        let save_btn = gtk4::Button::builder().label("Save").css_classes(["suggested-action"]).build();
        header.pack_start(&cancel_btn);
        header.pack_end(&save_btn);
        group_window.set_titlebar(Some(&header));
        
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        
        let label = Label::new(Some("Group name:"));
        content.append(&label);
        content.append(&entry);
        group_window.set_child(Some(&content));
        
        let window_clone = group_window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_clone.close();
        });
        
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        let window_clone = group_window.clone();
        let entry_clone = entry.clone();
        let old_name = group.name.clone();
        save_btn.connect_clicked(move |_| {
            let new_name = entry_clone.text().to_string();
            if new_name.trim().is_empty() {
                let alert = gtk4::AlertDialog::builder()
                    .message("Validation Error")
                    .detail("Group name cannot be empty")
                    .modal(true)
                    .build();
                alert.show(Some(&window_clone));
                return;
            }
            
            // Check for duplicate name (but allow keeping same name)
            if new_name != old_name {
                let state_ref = state_clone.borrow();
                if state_ref.group_exists_by_name(&new_name) {
                    drop(state_ref);
                    let alert = gtk4::AlertDialog::builder()
                        .message("Validation Error")
                        .detail(&format!("Group with name '{}' already exists", new_name))
                        .modal(true)
                        .build();
                    alert.show(Some(&window_clone));
                    return;
                }
                drop(state_ref);
            }
            
            if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
                if let Some(existing) = state_mut.get_group(group_id).cloned() {
                    let mut updated = existing;
                    updated.name = new_name;
                    if let Err(e) = state_mut.connection_manager().update_group(group_id, updated) {
                        let alert = gtk4::AlertDialog::builder()
                            .message("Error")
                            .detail(&format!("{}", e))
                            .modal(true)
                            .build();
                        alert.show(Some(&window_clone));
                        return;
                    }
                }
                drop(state_mut);
                Self::reload_sidebar(&state_clone, &sidebar_clone);
                window_clone.close();
            }
        });
        
        group_window.present();
    }

    /// Deletes the selected connection or group
    fn delete_selected_connection(window: &ApplicationWindow, state: &SharedAppState, sidebar: &SharedSidebar) {
        // Get selected item using sidebar's method (works in both single and multi-selection modes)
        let Some(conn_item) = sidebar.get_selected_item() else { return };
        
        let id_str = conn_item.id();
        let Ok(id) = Uuid::parse_str(&id_str) else { return };
        let name = conn_item.name();
        let is_group = conn_item.is_group();
        
        // Show confirmation dialog
        let item_type = if is_group { "group" } else { "connection" };
        let detail = if is_group {
            format!("Are you sure you want to delete the group '{}'?\n\nAll connections in this group will become ungrouped.", name)
        } else {
            format!("Are you sure you want to delete the connection '{}'?", name)
        };
        
        let alert = gtk4::AlertDialog::builder()
            .message(&format!("Delete {}?", item_type))
            .detail(&detail)
            .buttons(["Cancel", "Delete"])
            .default_button(0)
            .cancel_button(0)
            .modal(true)
            .build();
        
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        let window_clone = window.clone();
        alert.choose(Some(window), gio::Cancellable::NONE, move |result| {
            if result == Ok(1) { // "Delete" button
                if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
                    let delete_result = if is_group {
                        state_mut.delete_group(id)
                    } else {
                        state_mut.delete_connection(id)
                    };
                    
                    match delete_result {
                        Ok(()) => {
                            drop(state_mut);
                            Self::reload_sidebar(&state_clone, &sidebar_clone);
                        }
                        Err(e) => {
                            let error_alert = gtk4::AlertDialog::builder()
                                .message("Error Deleting")
                                .detail(&e)
                                .modal(true)
                                .build();
                            error_alert.show(Some(&window_clone));
                        }
                    }
                }
            }
        });
    }

    /// Deletes all selected connections (bulk delete for group operations mode)
    fn delete_selected_connections(window: &ApplicationWindow, state: &SharedAppState, sidebar: &SharedSidebar) {
        let selected_ids = sidebar.get_selected_ids();
        
        if selected_ids.is_empty() {
            let alert = gtk4::AlertDialog::builder()
                .message("No Selection")
                .detail("Please select one or more items to delete.")
                .modal(true)
                .build();
            alert.show(Some(window));
            return;
        }
        
        // Build list of items to delete for confirmation
        let state_ref = state.borrow();
        let mut item_names: Vec<String> = Vec::new();
        let mut connection_count = 0;
        let mut group_count = 0;
        
        for id in &selected_ids {
            if let Some(conn) = state_ref.get_connection(*id) {
                item_names.push(format!("• {} (connection)", conn.name));
                connection_count += 1;
            } else if let Some(group) = state_ref.get_group(*id) {
                item_names.push(format!("• {} (group)", group.name));
                group_count += 1;
            }
        }
        drop(state_ref);
        
        let summary = match (connection_count, group_count) {
            (c, 0) => format!("{} connection(s)", c),
            (0, g) => format!("{} group(s)", g),
            (c, g) => format!("{} connection(s) and {} group(s)", c, g),
        };
        
        // Create custom dialog with scrolling for large lists
        let dialog = gtk4::Window::builder()
            .title("Delete Selected Items?")
            .transient_for(window)
            .modal(true)
            .default_width(400)
            .default_height(if item_names.len() > 10 { 400 } else { 250 })
            .build();
        
        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let delete_btn = Button::builder()
            .label("Delete All")
            .css_classes(["destructive-action"])
            .build();
        header.pack_start(&cancel_btn);
        header.pack_end(&delete_btn);
        dialog.set_titlebar(Some(&header));
        
        let content = gtk4::Box::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        
        // Summary label
        let summary_label = Label::builder()
            .label(&format!("Are you sure you want to delete {}?", summary))
            .halign(gtk4::Align::Start)
            .wrap(true)
            .build();
        content.append(&summary_label);
        
        // Scrolled list of items
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_height(100)
            .max_content_height(250)
            .vexpand(true)
            .build();
        
        let items_label = Label::builder()
            .label(&item_names.join("\n"))
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Start)
            .wrap(true)
            .selectable(true)
            .build();
        scrolled.set_child(Some(&items_label));
        content.append(&scrolled);
        
        // Warning label
        let warning_label = Label::builder()
            .label("Connections in deleted groups will become ungrouped.")
            .halign(gtk4::Align::Start)
            .wrap(true)
            .css_classes(["dim-label"])
            .build();
        content.append(&warning_label);
        
        dialog.set_child(Some(&content));
        
        // Connect cancel button
        let dialog_weak = dialog.downgrade();
        cancel_btn.connect_clicked(move |_| {
            if let Some(d) = dialog_weak.upgrade() {
                d.close();
            }
        });
        
        // Connect delete button
        let dialog_weak = dialog.downgrade();
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        let window_clone = window.clone();
        delete_btn.connect_clicked(move |_| {
            if let Some(d) = dialog_weak.upgrade() {
                d.close();
            }
            
            let mut success_count = 0;
            let mut failures: Vec<String> = Vec::new();
            
            if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
                for id in &selected_ids {
                    // Try to delete as connection first, then as group
                    let delete_result = state_mut.delete_connection(*id)
                        .or_else(|_| state_mut.delete_group(*id));
                    
                    match delete_result {
                        Ok(()) => success_count += 1,
                        Err(e) => failures.push(format!("{}: {}", id, e)),
                    }
                }
            }
            
            // Reload sidebar
            Self::reload_sidebar(&state_clone, &sidebar_clone);
            
            // Show results
            if failures.is_empty() {
                let success_alert = gtk4::AlertDialog::builder()
                    .message("Deletion Complete")
                    .detail(&format!("Successfully deleted {} item(s).", success_count))
                    .modal(true)
                    .build();
                success_alert.show(Some(&window_clone));
            } else {
                let error_alert = gtk4::AlertDialog::builder()
                    .message("Deletion Partially Complete")
                    .detail(&format!(
                        "Deleted {} item(s).\n\nFailed to delete {} item(s):\n{}",
                        success_count,
                        failures.len(),
                        failures.join("\n")
                    ))
                    .modal(true)
                    .build();
                error_alert.show(Some(&window_clone));
            }
        });
        
        dialog.present();
    }

    /// Shows dialog to move selected items to a group
    fn show_move_selected_to_group_dialog(window: &ApplicationWindow, state: &SharedAppState, sidebar: &SharedSidebar) {
        let selected_ids = sidebar.get_selected_ids();
        
        if selected_ids.is_empty() {
            let alert = gtk4::AlertDialog::builder()
                .message("No Selection")
                .detail("Please select one or more connections to move.")
                .modal(true)
                .build();
            alert.show(Some(window));
            return;
        }
        
        // Filter to only connections (not groups)
        let state_ref = state.borrow();
        let connection_ids: Vec<Uuid> = selected_ids.iter()
            .filter(|id| state_ref.get_connection(**id).is_some())
            .copied()
            .collect();
        drop(state_ref);
        
        if connection_ids.is_empty() {
            let alert = gtk4::AlertDialog::builder()
                .message("No Connections Selected")
                .detail("Only connections can be moved to groups. Please select at least one connection.")
                .modal(true)
                .build();
            alert.show(Some(window));
            return;
        }
        
        // Build group selection dialog
        let state_ref = state.borrow();
        let groups = state_ref.list_groups();
        let mut group_names: Vec<String> = vec!["(No Group)".to_string()];
        let mut group_ids: Vec<Option<Uuid>> = vec![None];
        
        for group in groups {
            group_names.push(group.name.clone());
            group_ids.push(Some(group.id));
        }
        drop(state_ref);
        
        let alert = gtk4::AlertDialog::builder()
            .message("Move to Group")
            .detail(&format!("Select a group for {} connection(s):", connection_ids.len()))
            .buttons(group_names)
            .default_button(0)
            .cancel_button(-1)
            .modal(true)
            .build();
        
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        let window_clone = window.clone();
        alert.choose(Some(window), gio::Cancellable::NONE, move |result| {
            if let Ok(choice) = result {
                let choice_idx = choice as usize;
                if choice_idx < group_ids.len() {
                    let target_group = group_ids[choice_idx];
                    let mut success_count = 0;
                    let mut failures: Vec<String> = Vec::new();
                    
                    if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
                        for conn_id in &connection_ids {
                            match state_mut.move_connection_to_group(*conn_id, target_group) {
                                Ok(()) => success_count += 1,
                                Err(e) => failures.push(format!("{}: {}", conn_id, e)),
                            }
                        }
                    }
                    
                    // Reload sidebar
                    Self::reload_sidebar(&state_clone, &sidebar_clone);
                    
                    // Show results if there were failures
                    if !failures.is_empty() {
                        let error_alert = gtk4::AlertDialog::builder()
                            .message("Move Partially Complete")
                            .detail(&format!(
                                "Moved {} connection(s).\n\nFailed to move {} connection(s):\n{}",
                                success_count,
                                failures.len(),
                                failures.join("\n")
                            ))
                            .modal(true)
                            .build();
                        error_alert.show(Some(&window_clone));
                    }
                }
            }
        });
    }

    /// Duplicates the selected connection
    fn duplicate_selected_connection(state: &SharedAppState, sidebar: &SharedSidebar) {
        // Get selected item using sidebar's method (works in both single and multi-selection modes)
        let Some(conn_item) = sidebar.get_selected_item() else { return };
        
        // Can only duplicate connections, not groups
        if conn_item.is_group() {
            return;
        }
        
        let id_str = conn_item.id();
        let Ok(id) = Uuid::parse_str(&id_str) else { return };
        
        let state_ref = state.borrow();
        let Some(conn) = state_ref.get_connection(id).cloned() else { return };
        
        // Generate unique name for duplicate
        let new_name = state_ref.generate_unique_connection_name(&format!("{} (copy)", conn.name));
        drop(state_ref);
        
        // Create duplicate with new ID and name
        let mut duplicate = conn;
        duplicate.id = Uuid::new_v4();
        duplicate.name = new_name;
        duplicate.created_at = chrono::Utc::now();
        duplicate.updated_at = chrono::Utc::now();
        
        if let Ok(mut state_mut) = state.try_borrow_mut() {
            match state_mut.connection_manager().create_connection_from(duplicate) {
                Ok(_) => {
                    drop(state_mut);
                    Self::reload_sidebar(state, sidebar);
                }
                Err(e) => {
                    eprintln!("Failed to duplicate connection: {}", e);
                }
            }
        }
    }

    /// Reloads the sidebar with current data (preserving hierarchy)
    fn reload_sidebar(state: &SharedAppState, sidebar: &SharedSidebar) {
        let store = sidebar.store();
        store.remove_all();
        
        let state_ref = state.borrow();
        
        // Add root groups with their children
        for group in state_ref.get_root_groups() {
            let group_item = ConnectionItem::new_group(&group.id.to_string(), &group.name);
            Self::add_group_children_static(&state_ref, &group_item, group.id);
            store.append(&group_item);
        }
        
        // Add ungrouped connections
        for conn in state_ref.get_ungrouped_connections() {
            let protocol = match &conn.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
            };
            let item = ConnectionItem::new_connection(
                &conn.id.to_string(),
                &conn.name,
                protocol,
                &conn.host,
            );
            store.append(&item);
        }
    }

    /// Recursively adds group children (static version for reload)
    fn add_group_children_static(
        state: &std::cell::Ref<crate::state::AppState>,
        parent_item: &ConnectionItem,
        group_id: Uuid,
    ) {
        // Add child groups
        for child_group in state.get_child_groups(group_id) {
            let child_item = ConnectionItem::new_group(&child_group.id.to_string(), &child_group.name);
            Self::add_group_children_static(state, &child_item, child_group.id);
            parent_item.add_child(&child_item);
        }
        
        // Add connections in this group
        for conn in state.get_connections_by_group(group_id) {
            let protocol = match &conn.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
            };
            let item = ConnectionItem::new_connection(
                &conn.id.to_string(),
                &conn.name,
                protocol,
                &conn.host,
            );
            parent_item.add_child(&item);
        }
    }

    /// Presents the window to the user
    pub fn present(&self) {
        self.window.present();
    }

    /// Returns a reference to the underlying GTK window
    #[must_use]
    pub fn gtk_window(&self) -> &ApplicationWindow {
        &self.window
    }

    /// Registers the application icon in the icon theme
    fn register_app_icon() {
        if let Some(display) = gtk4::gdk::Display::default() {
            let icon_theme = gtk4::IconTheme::for_display(&display);
            
            // Add the icons directory to the icon search path (hicolor structure)
            let icons_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons");
            icon_theme.add_search_path(icons_path);
        }
    }

    /// Returns a reference to the connection sidebar
    #[must_use]
    pub fn sidebar(&self) -> &ConnectionSidebar {
        &self.sidebar
    }

    /// Returns a reference to the terminal notebook
    #[must_use]
    pub fn terminal_notebook(&self) -> &TerminalNotebook {
        &self.terminal_notebook
    }

    /// Opens a local shell terminal
    fn open_local_shell(notebook: &SharedNotebook) {
        let session_id = notebook.create_terminal_tab(
            Uuid::nil(),
            "Local Shell",
            "local",
        );
        
        // Get user's default shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        notebook.spawn_command(session_id, &[&shell], None, None);
    }

    /// Starts an RDP session using the embedded session tab
    ///
    /// This method launches xfreerdp with appropriate parameters based on
    /// the connection configuration. On X11, it attempts to embed the session.
    /// On Wayland, it falls back to an external window.
    fn start_rdp_session(
        tab: &crate::embedded::EmbeddedSessionTab,
        host: &str,
        port: u16,
        username: Option<&str>,
        rdp_config: Option<&rustconn_core::RdpConfig>,
    ) -> Result<(), crate::embedded::EmbeddingError> {
        use crate::embedded::RdpLauncher;

        // Extract resolution from config
        let resolution = rdp_config.and_then(|cfg| {
            cfg.resolution.as_ref().map(|r| (r.width, r.height))
        });

        // Build extra arguments from config
        let mut extra_args = Vec::new();
        if let Some(cfg) = rdp_config {
            // Add custom arguments
            for arg in &cfg.custom_args {
                extra_args.push(arg.clone());
            }

            // Add color depth if specified
            if let Some(depth) = cfg.color_depth {
                extra_args.push(format!("/bpp:{depth}"));
            }

            // Add audio redirection if enabled
            if cfg.audio_redirect {
                extra_args.push("/sound".to_string());
            }
        }

        RdpLauncher::start(tab, host, port, username, resolution, &extra_args)
    }

    /// Starts a VNC session using the embedded session tab
    ///
    /// This method launches vncviewer with appropriate parameters based on
    /// the connection configuration. On X11, it attempts to embed the session.
    /// On Wayland, it falls back to an external window.
    fn start_vnc_session(
        tab: &crate::embedded::EmbeddedSessionTab,
        host: &str,
        port: u16,
        vnc_config: Option<&rustconn_core::VncConfig>,
    ) -> Result<(), crate::embedded::EmbeddingError> {
        use crate::embedded::VncLauncher;

        // Extract encoding preference from config
        let encoding = vnc_config.and_then(|cfg| cfg.encoding.as_deref());

        // Extract quality level from config
        let quality = vnc_config.and_then(|cfg| cfg.quality);

        // Build extra arguments from config
        let mut extra_args = Vec::new();
        if let Some(cfg) = vnc_config {
            // Add custom arguments
            for arg in &cfg.custom_args {
                extra_args.push(arg.clone());
            }

            // Add compression level if specified
            if let Some(comp) = cfg.compression {
                extra_args.push("-CompressLevel".to_string());
                extra_args.push(comp.to_string());
            }
        }

        VncLauncher::start(tab, host, port, encoding, quality, &extra_args)
    }

    /// Shows a notification about embedding failure
    ///
    /// This displays an alert dialog informing the user that embedding
    /// failed and the session is running in an external window.
    #[allow(dead_code)]
    fn show_embedding_failure_notification(
        window: &ApplicationWindow,
        protocol: &str,
        connection_name: &str,
        error: &crate::embedded::EmbeddingError,
    ) {
        use crate::embedded::DisplayServer;

        let display_server = DisplayServer::detect();
        let is_wayland = matches!(display_server, DisplayServer::Wayland);

        let title = format!("{} Session", protocol.to_uppercase());
        let message = if is_wayland {
            format!(
                "The {} session for '{}' is running in an external window.\n\n\
                 Embedding is not supported on Wayland. To enable embedded sessions, \
                 run RustConn under X11 (set GDK_BACKEND=x11).",
                protocol.to_uppercase(),
                connection_name
            )
        } else {
            format!(
                "Failed to embed {} session for '{}'.\n\n\
                 Error: {}\n\n\
                 The session may be running in an external window.",
                protocol.to_uppercase(),
                connection_name,
                error
            )
        };

        let dialog = gtk4::AlertDialog::builder()
            .message(&title)
            .detail(&message)
            .modal(true)
            .build();

        dialog.show(Some(window));
    }

    /// Shows the quick connect dialog
    fn show_quick_connect_dialog(window: &ApplicationWindow, notebook: SharedNotebook) {
        // Create a quick connect window
        let quick_window = gtk4::Window::builder()
            .title("Quick Connect")
            .transient_for(window)
            .modal(true)
            .default_width(550)
            .default_height(550)
            .build();
        
        // Create header bar with Cancel/Quick Connect buttons
        let header = gtk4::HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let connect_btn = Button::builder()
            .label("Quick Connect")
            .css_classes(["suggested-action"])
            .build();
        header.pack_start(&cancel_btn);
        header.pack_end(&connect_btn);
        quick_window.set_titlebar(Some(&header));
        
        // Create content
        let content = gtk4::Box::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        
        // Info label
        let info_label = Label::new(Some("⚠ This connection will not be saved"));
        info_label.add_css_class("dim-label");
        content.append(&info_label);
        
        // Basic fields
        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .build();
        
        let host_label = Label::builder().label("Host:").halign(gtk4::Align::End).build();
        let host_entry = gtk4::Entry::builder().hexpand(true).placeholder_text("hostname or IP").build();
        grid.attach(&host_label, 0, 0, 1, 1);
        grid.attach(&host_entry, 1, 0, 2, 1);
        
        let port_label = Label::builder().label("Port:").halign(gtk4::Align::End).build();
        let port_adj = gtk4::Adjustment::new(22.0, 1.0, 65535.0, 1.0, 10.0, 0.0);
        let port_spin = gtk4::SpinButton::builder().adjustment(&port_adj).climb_rate(1.0).digits(0).build();
        grid.attach(&port_label, 0, 1, 1, 1);
        grid.attach(&port_spin, 1, 1, 1, 1);
        
        let user_label = Label::builder().label("Username:").halign(gtk4::Align::End).build();
        let user_entry = gtk4::Entry::builder().hexpand(true).placeholder_text("(optional)").build();
        grid.attach(&user_label, 0, 2, 1, 1);
        grid.attach(&user_entry, 1, 2, 2, 1);
        
        content.append(&grid);
        quick_window.set_child(Some(&content));
        
        // Connect cancel
        let window_clone = quick_window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_clone.close();
        });
        
        // Connect quick connect button
        let window_clone = quick_window.clone();
        let host_clone = host_entry.clone();
        let port_clone = port_spin.clone();
        let user_clone = user_entry.clone();
        connect_btn.connect_clicked(move |_| {
            let host = host_clone.text().to_string();
            if host.trim().is_empty() {
                return;
            }
            
            let port = port_clone.value() as u16;
            let username = {
                let text = user_clone.text();
                if text.trim().is_empty() { None } else { Some(text.to_string()) }
            };
            
            let session_id = notebook.create_terminal_tab(
                Uuid::nil(),
                &format!("Quick: {}", host),
                "ssh",
            );
            
            notebook.spawn_ssh(
                session_id,
                &host,
                port,
                username.as_deref(),
                None,
                &[],
            );
            
            window_clone.close();
        });
        
        quick_window.present();
    }

    /// Toggles group operations mode for multi-select
    fn toggle_group_operations_mode(sidebar: &SharedSidebar, enabled: bool) {
        sidebar.set_group_operations_mode(enabled);
    }

    /// Sorts connections alphabetically and updates sort_order
    fn sort_connections(state: &SharedAppState, sidebar: &SharedSidebar) {
        let store = sidebar.store();
        let state_ref = state.borrow();
        
        // Get and sort groups
        let mut groups: Vec<_> = state_ref.get_root_groups().iter().map(|g| (*g).clone()).collect();
        groups.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        // Get and sort ungrouped connections
        let mut ungrouped: Vec<_> = state_ref.get_ungrouped_connections().iter().map(|c| (*c).clone()).collect();
        ungrouped.sort_by(|a, b| {
            // Sort by protocol first, then alphabetically
            let proto_a = match &a.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => 0,
                rustconn_core::ProtocolConfig::Rdp(_) => 1,
                rustconn_core::ProtocolConfig::Vnc(_) => 2,
            };
            let proto_b = match &b.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => 0,
                rustconn_core::ProtocolConfig::Rdp(_) => 1,
                rustconn_core::ProtocolConfig::Vnc(_) => 2,
            };
            match proto_a.cmp(&proto_b) {
                std::cmp::Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                other => other,
            }
        });
        
        drop(state_ref);
        
        // Update sort_order for ungrouped connections
        let connection_ids: Vec<Uuid> = ungrouped.iter().map(|c| c.id).collect();
        if let Ok(mut state_mut) = state.try_borrow_mut() {
            let _ = state_mut.reorder_connections(&connection_ids);
        }
        
        // Rebuild store with sorted items (groups first, then ungrouped)
        store.remove_all();
        
        let state_ref = state.borrow();
        
        // Add sorted groups with their sorted children
        for group in &groups {
            let group_item = ConnectionItem::new_group(&group.id.to_string(), &group.name);
            Self::add_sorted_group_children(&state_ref, &group_item, group.id);
            store.append(&group_item);
        }
        
        // Add sorted ungrouped connections
        for conn in &ungrouped {
            let protocol = match &conn.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
            };
            let item = ConnectionItem::new_connection(
                &conn.id.to_string(),
                &conn.name,
                protocol,
                &conn.host,
            );
            store.append(&item);
        }
    }

    /// Recursively adds sorted group children
    fn add_sorted_group_children(
        state: &std::cell::Ref<crate::state::AppState>,
        parent_item: &ConnectionItem,
        group_id: Uuid,
    ) {
        // Get and sort child groups
        let mut child_groups: Vec<_> = state.get_child_groups(group_id).iter().map(|g| (*g).clone()).collect();
        child_groups.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        for child_group in &child_groups {
            let child_item = ConnectionItem::new_group(&child_group.id.to_string(), &child_group.name);
            Self::add_sorted_group_children(state, &child_item, child_group.id);
            parent_item.add_child(&child_item);
        }
        
        // Get and sort connections in this group
        let mut connections: Vec<_> = state.get_connections_by_group(group_id).iter().map(|c| (*c).clone()).collect();
        connections.sort_by(|a, b| {
            let proto_a = match &a.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => 0,
                rustconn_core::ProtocolConfig::Rdp(_) => 1,
                rustconn_core::ProtocolConfig::Vnc(_) => 2,
            };
            let proto_b = match &b.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => 0,
                rustconn_core::ProtocolConfig::Rdp(_) => 1,
                rustconn_core::ProtocolConfig::Vnc(_) => 2,
            };
            match proto_a.cmp(&proto_b) {
                std::cmp::Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                other => other,
            }
        });
        
        for conn in &connections {
            let protocol = match &conn.protocol_config {
                rustconn_core::ProtocolConfig::Ssh(_) => "ssh",
                rustconn_core::ProtocolConfig::Rdp(_) => "rdp",
                rustconn_core::ProtocolConfig::Vnc(_) => "vnc",
            };
            let item = ConnectionItem::new_connection(
                &conn.id.to_string(),
                &conn.name,
                protocol,
                &conn.host,
            );
            parent_item.add_child(&item);
        }
    }

    /// Shows the export dialog
    fn show_export_dialog(window: &ApplicationWindow, state: SharedAppState) {
        let file_dialog = gtk4::FileDialog::builder()
            .title("Export Configuration")
            .modal(true)
            .build();
        
        // Set default filename
        let default_name = format!("rustconn-export-{}.json", chrono::Local::now().format("%Y%m%d"));
        file_dialog.set_initial_name(Some(&default_name));
        
        let state_clone = state.clone();
        let window_clone = window.clone();
        file_dialog.save(Some(window), gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    Self::export_config(&window_clone, &state_clone, &path);
                }
            }
        });
    }

    /// Exports configuration to file
    fn export_config(window: &ApplicationWindow, state: &SharedAppState, path: &std::path::Path) {
        let state_ref = state.borrow();
        
        // Build export data (without secrets)
        let mut export_data = serde_json::json!({
            "version": "1.0",
            "exported_at": chrono::Local::now().to_rfc3339(),
            "connections": [],
            "groups": []
        });
        
        // Export connections (without passwords)
        let connections: Vec<serde_json::Value> = state_ref.list_connections()
            .iter()
            .map(|conn| {
                serde_json::json!({
                    "id": conn.id.to_string(),
                    "name": conn.name,
                    "host": conn.host,
                    "port": conn.port,
                    "username": conn.username,
                    "group_id": conn.group_id.map(|id| id.to_string()),
                    "tags": conn.tags,
                    "protocol": match &conn.protocol_config {
                        rustconn_core::ProtocolConfig::Ssh(ssh) => serde_json::json!({
                            "type": "ssh",
                            "key_path": ssh.key_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                            "proxy_jump": ssh.proxy_jump,
                            "use_control_master": ssh.use_control_master,
                        }),
                        rustconn_core::ProtocolConfig::Rdp(rdp) => serde_json::json!({
                            "type": "rdp",
                            "resolution": rdp.resolution.as_ref().map(|r| format!("{}x{}", r.width, r.height)),
                            "color_depth": rdp.color_depth,
                        }),
                        rustconn_core::ProtocolConfig::Vnc(vnc) => serde_json::json!({
                            "type": "vnc",
                            "encoding": vnc.encoding,
                            "compression": vnc.compression,
                            "quality": vnc.quality,
                        }),
                    }
                })
            })
            .collect();
        
        export_data["connections"] = serde_json::Value::Array(connections);
        
        // Export groups
        let groups: Vec<serde_json::Value> = state_ref.list_groups()
            .iter()
            .map(|group| {
                serde_json::json!({
                    "id": group.id.to_string(),
                    "name": group.name,
                    "parent_id": group.parent_id.map(|id| id.to_string()),
                })
            })
            .collect();
        
        export_data["groups"] = serde_json::Value::Array(groups);
        
        drop(state_ref);
        
        // Write to file
        match std::fs::write(path, serde_json::to_string_pretty(&export_data).unwrap_or_default()) {
            Ok(()) => {
                // Show success message with warning about secrets
                let alert = gtk4::AlertDialog::builder()
                    .message("Export Successful")
                    .detail("Configuration exported successfully.\n\n⚠ Note: Passwords were NOT exported for security reasons. SSH key paths have been preserved.")
                    .modal(true)
                    .build();
                alert.show(Some(window));
            }
            Err(e) => {
                let alert = gtk4::AlertDialog::builder()
                    .message("Export Failed")
                    .detail(&format!("Failed to export configuration: {}", e))
                    .modal(true)
                    .build();
                alert.show(Some(window));
            }
        }
    }

    // ========== Snippet Management Methods ==========

    /// Shows the new snippet dialog
    fn show_new_snippet_dialog(window: &ApplicationWindow, state: SharedAppState) {
        let dialog = SnippetDialog::new(Some(&window.clone().upcast()));
        
        let window_clone = window.clone();
        dialog.run(move |result| {
            if let Some(snippet) = result {
                if let Ok(mut state_mut) = state.try_borrow_mut() {
                    match state_mut.create_snippet(snippet) {
                        Ok(_) => {
                            let alert = gtk4::AlertDialog::builder()
                                .message("Snippet Created")
                                .detail("Snippet has been saved successfully.")
                                .modal(true)
                                .build();
                            alert.show(Some(&window_clone));
                        }
                        Err(e) => {
                            let alert = gtk4::AlertDialog::builder()
                                .message("Error Creating Snippet")
                                .detail(&e)
                                .modal(true)
                                .build();
                            alert.show(Some(&window_clone));
                        }
                    }
                }
            }
        });
    }

    /// Shows the snippets manager window
    fn show_snippets_manager(window: &ApplicationWindow, state: SharedAppState, notebook: SharedNotebook) {
        let manager_window = gtk4::Window::builder()
            .title("Manage Snippets")
            .transient_for(window)
            .modal(true)
            .default_width(600)
            .default_height(500)
            .build();

        // Create header bar
        let header = HeaderBar::new();
        let close_btn = Button::builder().label("Close").build();
        let new_btn = Button::builder()
            .label("New Snippet")
            .css_classes(["suggested-action"])
            .build();
        header.pack_start(&close_btn);
        header.pack_end(&new_btn);
        manager_window.set_titlebar(Some(&header));

        // Create main content
        let content = gtk4::Box::new(Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Search entry
        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search snippets..."));
        content.append(&search_entry);

        // Snippets list
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let snippets_list = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();
        scrolled.set_child(Some(&snippets_list));
        content.append(&scrolled);

        // Action buttons
        let button_box = gtk4::Box::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::End);
        
        let edit_btn = Button::builder().label("Edit").sensitive(false).build();
        let delete_btn = Button::builder().label("Delete").sensitive(false).build();
        let execute_btn = Button::builder()
            .label("Execute")
            .sensitive(false)
            .css_classes(["suggested-action"])
            .build();
        
        button_box.append(&edit_btn);
        button_box.append(&delete_btn);
        button_box.append(&execute_btn);
        content.append(&button_box);

        manager_window.set_child(Some(&content));

        // Populate snippets list
        Self::populate_snippets_list(&state, &snippets_list, "");

        // Connect search
        let state_clone = state.clone();
        let list_clone = snippets_list.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            Self::populate_snippets_list(&state_clone, &list_clone, &query);
        });

        // Connect selection changed
        let edit_clone = edit_btn.clone();
        let delete_clone = delete_btn.clone();
        let execute_clone = execute_btn.clone();
        snippets_list.connect_row_selected(move |_, row| {
            let has_selection = row.is_some();
            edit_clone.set_sensitive(has_selection);
            delete_clone.set_sensitive(has_selection);
            execute_clone.set_sensitive(has_selection);
        });

        // Connect close button
        let window_clone = manager_window.clone();
        close_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Connect new button
        let state_clone = state.clone();
        let list_clone = snippets_list.clone();
        let manager_clone = manager_window.clone();
        new_btn.connect_clicked(move |_| {
            let dialog = SnippetDialog::new(Some(&manager_clone.clone().upcast()));
            let state_inner = state_clone.clone();
            let list_inner = list_clone.clone();
            dialog.run(move |result| {
                if let Some(snippet) = result {
                    if let Ok(mut state_mut) = state_inner.try_borrow_mut() {
                        let _ = state_mut.create_snippet(snippet);
                        drop(state_mut);
                        Self::populate_snippets_list(&state_inner, &list_inner, "");
                    }
                }
            });
        });

        // Connect edit button
        let state_clone = state.clone();
        let list_clone = snippets_list.clone();
        let manager_clone = manager_window.clone();
        edit_btn.connect_clicked(move |_| {
            if let Some(row) = list_clone.selected_row() {
                if let Some(id_str) = row.widget_name().as_str().strip_prefix("snippet-") {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        let state_ref = state_clone.borrow();
                        if let Some(snippet) = state_ref.get_snippet(id).cloned() {
                            drop(state_ref);
                            let dialog = SnippetDialog::new(Some(&manager_clone.clone().upcast()));
                            dialog.set_snippet(&snippet);
                            let state_inner = state_clone.clone();
                            let list_inner = list_clone.clone();
                            dialog.run(move |result| {
                                if let Some(updated) = result {
                                    if let Ok(mut state_mut) = state_inner.try_borrow_mut() {
                                        let _ = state_mut.update_snippet(id, updated);
                                        drop(state_mut);
                                        Self::populate_snippets_list(&state_inner, &list_inner, "");
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });

        // Connect delete button
        let state_clone = state.clone();
        let list_clone = snippets_list.clone();
        let manager_clone = manager_window.clone();
        delete_btn.connect_clicked(move |_| {
            if let Some(row) = list_clone.selected_row() {
                if let Some(id_str) = row.widget_name().as_str().strip_prefix("snippet-") {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        let alert = gtk4::AlertDialog::builder()
                            .message("Delete Snippet?")
                            .detail("Are you sure you want to delete this snippet?")
                            .buttons(["Cancel", "Delete"])
                            .default_button(0)
                            .cancel_button(0)
                            .modal(true)
                            .build();
                        
                        let state_inner = state_clone.clone();
                        let list_inner = list_clone.clone();
                        alert.choose(Some(&manager_clone), gio::Cancellable::NONE, move |result| {
                            if result == Ok(1) {
                                if let Ok(mut state_mut) = state_inner.try_borrow_mut() {
                                    let _ = state_mut.delete_snippet(id);
                                    drop(state_mut);
                                    Self::populate_snippets_list(&state_inner, &list_inner, "");
                                }
                            }
                        });
                    }
                }
            }
        });

        // Connect execute button
        let state_clone = state.clone();
        let list_clone = snippets_list.clone();
        let notebook_clone = notebook.clone();
        let manager_clone = manager_window.clone();
        execute_btn.connect_clicked(move |_| {
            if let Some(row) = list_clone.selected_row() {
                if let Some(id_str) = row.widget_name().as_str().strip_prefix("snippet-") {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        let state_ref = state_clone.borrow();
                        if let Some(snippet) = state_ref.get_snippet(id).cloned() {
                            drop(state_ref);
                            Self::execute_snippet(&manager_clone, &notebook_clone, &snippet);
                        }
                    }
                }
            }
        });

        manager_window.present();
    }

    /// Populates the snippets list with filtered results
    fn populate_snippets_list(state: &SharedAppState, list: &gtk4::ListBox, query: &str) {
        // Clear existing rows
        while let Some(row) = list.row_at_index(0) {
            list.remove(&row);
        }

        let state_ref = state.borrow();
        let snippets = if query.is_empty() {
            state_ref.list_snippets()
        } else {
            state_ref.search_snippets(query)
        };

        for snippet in snippets {
            let row = gtk4::ListBoxRow::new();
            row.set_widget_name(&format!("snippet-{}", snippet.id));

            let hbox = gtk4::Box::new(Orientation::Horizontal, 12);
            hbox.set_margin_top(8);
            hbox.set_margin_bottom(8);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);

            let vbox = gtk4::Box::new(Orientation::Vertical, 4);
            vbox.set_hexpand(true);

            let name_label = Label::builder()
                .label(&snippet.name)
                .halign(gtk4::Align::Start)
                .css_classes(["heading"])
                .build();
            vbox.append(&name_label);

            let cmd_preview = if snippet.command.len() > 50 {
                format!("{}...", &snippet.command[..50])
            } else {
                snippet.command.clone()
            };
            let cmd_label = Label::builder()
                .label(&cmd_preview)
                .halign(gtk4::Align::Start)
                .css_classes(["dim-label", "monospace"])
                .build();
            vbox.append(&cmd_label);

            if let Some(ref cat) = snippet.category {
                let cat_label = Label::builder()
                    .label(cat)
                    .halign(gtk4::Align::Start)
                    .css_classes(["dim-label"])
                    .build();
                vbox.append(&cat_label);
            }

            hbox.append(&vbox);
            row.set_child(Some(&hbox));
            list.append(&row);
        }
    }

    /// Shows a snippet picker for quick execution
    fn show_snippet_picker(window: &ApplicationWindow, state: SharedAppState, notebook: SharedNotebook) {
        let picker_window = gtk4::Window::builder()
            .title("Execute Snippet")
            .transient_for(window)
            .modal(true)
            .default_width(400)
            .default_height(400)
            .build();

        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        header.pack_start(&cancel_btn);
        picker_window.set_titlebar(Some(&header));

        let content = gtk4::Box::new(Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search snippets..."));
        content.append(&search_entry);

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let snippets_list = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();
        scrolled.set_child(Some(&snippets_list));
        content.append(&scrolled);

        picker_window.set_child(Some(&content));

        Self::populate_snippets_list(&state, &snippets_list, "");

        // Connect search
        let state_clone = state.clone();
        let list_clone = snippets_list.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            Self::populate_snippets_list(&state_clone, &list_clone, &query);
        });

        // Connect cancel
        let window_clone = picker_window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Connect row activation (double-click or Enter)
        let state_clone = state.clone();
        let notebook_clone = notebook.clone();
        let window_clone = picker_window.clone();
        snippets_list.connect_row_activated(move |_, row| {
            if let Some(id_str) = row.widget_name().as_str().strip_prefix("snippet-") {
                if let Ok(id) = Uuid::parse_str(id_str) {
                    let state_ref = state_clone.borrow();
                    if let Some(snippet) = state_ref.get_snippet(id).cloned() {
                        drop(state_ref);
                        Self::execute_snippet(&window_clone, &notebook_clone, &snippet);
                        window_clone.close();
                    }
                }
            }
        });

        picker_window.present();
    }

    /// Executes a snippet in the active terminal
    fn execute_snippet(parent: &impl IsA<gtk4::Window>, notebook: &SharedNotebook, snippet: &rustconn_core::Snippet) {
        // Check if there's an active terminal
        if notebook.get_active_terminal().is_none() {
            let alert = gtk4::AlertDialog::builder()
                .message("No Active Terminal")
                .detail("Please open a terminal session first before executing a snippet.")
                .modal(true)
                .build();
            alert.show(Some(parent));
            return;
        }

        // Check if snippet has variables that need values
        let variables = rustconn_core::SnippetManager::extract_variables(&snippet.command);
        
        if variables.is_empty() {
            // No variables, execute directly
            notebook.send_text(&format!("{}\n", snippet.command));
        } else {
            // Show variable input dialog
            Self::show_variable_input_dialog(parent, notebook, snippet);
        }
    }

    /// Shows a dialog to input variable values for a snippet
    fn show_variable_input_dialog(parent: &impl IsA<gtk4::Window>, notebook: &SharedNotebook, snippet: &rustconn_core::Snippet) {
        let var_window = gtk4::Window::builder()
            .title("Enter Variable Values")
            .transient_for(parent)
            .modal(true)
            .default_width(400)
            .build();

        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let execute_btn = Button::builder()
            .label("Execute")
            .css_classes(["suggested-action"])
            .build();
        header.pack_start(&cancel_btn);
        header.pack_end(&execute_btn);
        var_window.set_titlebar(Some(&header));

        let content = gtk4::Box::new(Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let grid = gtk4::Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .build();

        let mut entries: Vec<(String, gtk4::Entry)> = Vec::new();
        let variables = rustconn_core::SnippetManager::extract_variables(&snippet.command);

        for (i, var_name) in variables.iter().enumerate() {
            let label = Label::builder()
                .label(&format!("{}:", var_name))
                .halign(gtk4::Align::End)
                .build();
            
            let entry = gtk4::Entry::builder()
                .hexpand(true)
                .build();

            // Set default value if available
            if let Some(var_def) = snippet.variables.iter().find(|v| &v.name == var_name) {
                if let Some(ref default) = var_def.default_value {
                    entry.set_text(default);
                }
                if let Some(ref desc) = var_def.description {
                    entry.set_placeholder_text(Some(desc));
                }
            }

            grid.attach(&label, 0, i as i32, 1, 1);
            grid.attach(&entry, 1, i as i32, 1, 1);
            entries.push((var_name.clone(), entry));
        }

        content.append(&grid);
        var_window.set_child(Some(&content));

        // Connect cancel
        let window_clone = var_window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Connect execute
        let window_clone = var_window.clone();
        let notebook_clone = notebook.clone();
        let command = snippet.command.clone();
        execute_btn.connect_clicked(move |_| {
            let mut values = std::collections::HashMap::new();
            for (name, entry) in &entries {
                values.insert(name.clone(), entry.text().to_string());
            }
            
            let substituted = rustconn_core::SnippetManager::substitute_variables(&command, &values);
            notebook_clone.send_text(&format!("{}\n", substituted));
            window_clone.close();
        });

        var_window.present();
    }

    // ========== Session Management Methods ==========

    /// Shows the sessions manager window
    fn show_sessions_manager(window: &ApplicationWindow, state: SharedAppState, notebook: SharedNotebook) {
        let manager_window = gtk4::Window::builder()
            .title("Active Sessions")
            .transient_for(window)
            .modal(true)
            .default_width(500)
            .default_height(400)
            .build();

        // Create header bar
        let header = HeaderBar::new();
        let close_btn = Button::builder().label("Close").build();
        let refresh_btn = Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Refresh")
            .build();
        header.pack_start(&close_btn);
        header.pack_end(&refresh_btn);
        manager_window.set_titlebar(Some(&header));

        // Create main content
        let content = gtk4::Box::new(Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Session count label
        let count_label = Label::builder()
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label"])
            .build();
        content.append(&count_label);

        // Sessions list
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let sessions_list = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();
        scrolled.set_child(Some(&sessions_list));
        content.append(&scrolled);

        // Action buttons
        let button_box = gtk4::Box::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::End);
        
        let switch_btn = Button::builder().label("Switch To").sensitive(false).build();
        let terminate_btn = Button::builder()
            .label("Terminate")
            .sensitive(false)
            .css_classes(["destructive-action"])
            .build();
        
        button_box.append(&switch_btn);
        button_box.append(&terminate_btn);
        content.append(&button_box);

        manager_window.set_child(Some(&content));

        // Populate sessions list
        Self::populate_sessions_list(&state, &notebook, &sessions_list, &count_label);

        // Connect selection changed
        let switch_clone = switch_btn.clone();
        let terminate_clone = terminate_btn.clone();
        sessions_list.connect_row_selected(move |_, row| {
            let has_selection = row.is_some();
            switch_clone.set_sensitive(has_selection);
            terminate_clone.set_sensitive(has_selection);
        });

        // Connect close button
        let window_clone = manager_window.clone();
        close_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Connect refresh button
        let state_clone = state.clone();
        let notebook_clone = notebook.clone();
        let list_clone = sessions_list.clone();
        let count_clone = count_label.clone();
        refresh_btn.connect_clicked(move |_| {
            Self::populate_sessions_list(&state_clone, &notebook_clone, &list_clone, &count_clone);
        });

        // Connect switch button
        let notebook_clone = notebook.clone();
        let list_clone = sessions_list.clone();
        let window_clone = manager_window.clone();
        switch_btn.connect_clicked(move |_| {
            if let Some(row) = list_clone.selected_row() {
                if let Some(id_str) = row.widget_name().as_str().strip_prefix("session-") {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        notebook_clone.switch_to_tab(id);
                        window_clone.close();
                    }
                }
            }
        });

        // Connect terminate button
        let state_clone = state.clone();
        let notebook_clone = notebook.clone();
        let list_clone = sessions_list.clone();
        let count_clone = count_label.clone();
        let manager_clone = manager_window.clone();
        terminate_btn.connect_clicked(move |_| {
            if let Some(row) = list_clone.selected_row() {
                if let Some(id_str) = row.widget_name().as_str().strip_prefix("session-") {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        let alert = gtk4::AlertDialog::builder()
                            .message("Terminate Session?")
                            .detail("Are you sure you want to terminate this session?")
                            .buttons(["Cancel", "Terminate"])
                            .default_button(0)
                            .cancel_button(0)
                            .modal(true)
                            .build();
                        
                        let state_inner = state_clone.clone();
                        let notebook_inner = notebook_clone.clone();
                        let list_inner = list_clone.clone();
                        let count_inner = count_clone.clone();
                        alert.choose(Some(&manager_clone), gio::Cancellable::NONE, move |result| {
                            if result == Ok(1) {
                                // Terminate session in state manager
                                if let Ok(mut state_mut) = state_inner.try_borrow_mut() {
                                    let _ = state_mut.terminate_session(id);
                                }
                                // Close the tab
                                notebook_inner.close_tab(id);
                                // Refresh the list
                                Self::populate_sessions_list(&state_inner, &notebook_inner, &list_inner, &count_inner);
                            }
                        });
                    }
                }
            }
        });

        manager_window.present();
    }

    /// Populates the sessions list
    fn populate_sessions_list(
        state: &SharedAppState,
        notebook: &SharedNotebook,
        list: &gtk4::ListBox,
        count_label: &Label,
    ) {
        // Clear existing rows
        while let Some(row) = list.row_at_index(0) {
            list.remove(&row);
        }

        // Get sessions from notebook (UI sessions)
        let session_ids = notebook.session_ids();
        let session_count = session_ids.len();
        
        count_label.set_text(&format!("{} active session(s)", session_count));

        for session_id in session_ids {
            if let Some(info) = notebook.get_session_info(session_id) {
                let row = gtk4::ListBoxRow::new();
                row.set_widget_name(&format!("session-{}", session_id));

                let hbox = gtk4::Box::new(Orientation::Horizontal, 12);
                hbox.set_margin_top(8);
                hbox.set_margin_bottom(8);
                hbox.set_margin_start(12);
                hbox.set_margin_end(12);

                // Protocol icon
                let icon_name = match info.protocol.as_str() {
                    "ssh" | "local" => "utilities-terminal-symbolic",
                    "rdp" => "computer-symbolic",
                    "vnc" => "video-display-symbolic",
                    _ => "network-server-symbolic",
                };
                let icon = gtk4::Image::from_icon_name(icon_name);
                hbox.append(&icon);

                let vbox = gtk4::Box::new(Orientation::Vertical, 4);
                vbox.set_hexpand(true);

                let name_label = Label::builder()
                    .label(&info.name)
                    .halign(gtk4::Align::Start)
                    .css_classes(["heading"])
                    .build();
                vbox.append(&name_label);

                // Get connection info if available
                let state_ref = state.borrow();
                let connection_info = if info.connection_id != Uuid::nil() {
                    state_ref.get_connection(info.connection_id)
                        .map(|c| format!("{} ({})", c.host, info.protocol.to_uppercase()))
                } else {
                    Some(format!("{}", info.protocol.to_uppercase()))
                };
                drop(state_ref);

                if let Some(conn_info) = connection_info {
                    let info_label = Label::builder()
                        .label(&conn_info)
                        .halign(gtk4::Align::Start)
                        .css_classes(["dim-label"])
                        .build();
                    vbox.append(&info_label);
                }

                // Session type indicator
                let type_label = Label::builder()
                    .label(if info.is_embedded { "Embedded" } else { "External" })
                    .halign(gtk4::Align::Start)
                    .css_classes(["dim-label"])
                    .build();
                vbox.append(&type_label);

                hbox.append(&vbox);
                row.set_child(Some(&hbox));
                list.append(&row);
            }
        }
    }

    // ========== Group Hierarchy Methods ==========

    /// Shows the move to group dialog for the selected connection
    fn show_move_to_group_dialog(window: &ApplicationWindow, state: &SharedAppState, sidebar: &SharedSidebar) {
        // Get selected item using sidebar's method (works in both single and multi-selection modes)
        let Some(conn_item) = sidebar.get_selected_item() else { return };
        
        // Can only move connections, not groups
        if conn_item.is_group() {
            let alert = gtk4::AlertDialog::builder()
                .message("Cannot Move Group")
                .detail("Use drag and drop to reorganize groups.")
                .modal(true)
                .build();
            alert.show(Some(window));
            return;
        }
        
        let id_str = conn_item.id();
        let Ok(connection_id) = Uuid::parse_str(&id_str) else { return };
        let connection_name = conn_item.name();
        
        // Get current group
        let state_ref = state.borrow();
        let current_group_id = state_ref.get_connection(connection_id)
            .and_then(|c| c.group_id);
        drop(state_ref);
        
        // Create dialog
        let move_window = gtk4::Window::builder()
            .title("Move Connection")
            .transient_for(window)
            .modal(true)
            .default_width(350)
            .build();

        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let move_btn = Button::builder()
            .label("Move")
            .css_classes(["suggested-action"])
            .build();
        header.pack_start(&cancel_btn);
        header.pack_end(&move_btn);
        move_window.set_titlebar(Some(&header));

        let content = gtk4::Box::new(Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let info_label = Label::builder()
            .label(&format!("Move '{}' to:", connection_name))
            .halign(gtk4::Align::Start)
            .build();
        content.append(&info_label);

        // Group dropdown
        let state_ref = state.borrow();
        let groups: Vec<_> = state_ref.list_groups().iter().map(|g| (*g).clone()).collect();
        drop(state_ref);
        
        let mut group_ids: Vec<Option<Uuid>> = vec![None];
        let mut strings: Vec<String> = vec!["(Ungrouped)".to_string()];
        let mut current_index = 0u32;
        
        for group in &groups {
            let state_ref = state.borrow();
            let path = state_ref.get_group_path(group.id).unwrap_or_else(|| group.name.clone());
            drop(state_ref);
            
            strings.push(path);
            group_ids.push(Some(group.id));
            
            if current_group_id == Some(group.id) {
                current_index = (group_ids.len() - 1) as u32;
            }
        }
        
        let string_list = gtk4::StringList::new(&strings.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let group_dropdown = gtk4::DropDown::builder()
            .model(&string_list)
            .selected(current_index)
            .build();
        
        content.append(&group_dropdown);
        move_window.set_child(Some(&content));

        // Connect cancel
        let window_clone = move_window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Connect move
        let state_clone = state.clone();
        let sidebar_clone = sidebar.clone();
        let window_clone = move_window.clone();
        move_btn.connect_clicked(move |_| {
            let selected_idx = group_dropdown.selected() as usize;
            let target_group_id = if selected_idx < group_ids.len() {
                group_ids[selected_idx]
            } else {
                None
            };
            
            if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
                match state_mut.move_connection_to_group(connection_id, target_group_id) {
                    Ok(()) => {
                        drop(state_mut);
                        Self::reload_sidebar(&state_clone, &sidebar_clone);
                        window_clone.close();
                    }
                    Err(e) => {
                        let alert = gtk4::AlertDialog::builder()
                            .message("Error Moving Connection")
                            .detail(&e)
                            .modal(true)
                            .build();
                        alert.show(Some(&window_clone));
                    }
                }
            }
        });

        move_window.present();
    }
}
