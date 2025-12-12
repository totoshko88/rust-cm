//! Connection dialog for creating and editing connections
//!
//! Provides a GTK4 dialog with protocol-specific fields, input validation,
//! and portal integration for file selection (SSH keys).
//!
//! Updated for GTK 4.10+ compatibility using `DropDown` instead of `ComboBoxText`
//! and Window instead of Dialog.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, FileDialog, Grid, HeaderBar, Label,
    Notebook, Orientation, ScrolledWindow, SpinButton, Stack, StringList, Window,
};
use rustconn_core::models::{
    Connection, PasswordSource, ProtocolConfig, RdpConfig, Resolution, SharedFolder,
    SpiceConfig, SpiceImageCompression, SshAuthMethod, SshConfig, VncConfig,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use uuid::Uuid;

/// Connection dialog for creating/editing connections
pub struct ConnectionDialog {
    window: Window,
    // Header bar buttons - stored for potential future use (e.g., enabling/disabling based on validation)
    #[allow(dead_code)]
    save_button: Button,
    // Basic fields
    name_entry: Entry,
    host_entry: Entry,
    port_spin: SpinButton,
    username_entry: Entry,
    tags_entry: Entry,
    protocol_dropdown: DropDown,
    protocol_stack: Stack,
    // Password source selection
    password_source_dropdown: DropDown,
    // Password entry and KeePass save button
    password_entry: Entry,
    save_to_keepass_button: Button,
    // SSH fields
    ssh_auth_dropdown: DropDown,
    ssh_key_entry: Entry,
    ssh_key_button: Button,
    ssh_proxy_entry: Entry,
    ssh_control_master: CheckButton,
    ssh_startup_entry: Entry,
    ssh_options_entry: Entry,
    // RDP fields
    rdp_width_spin: SpinButton,
    rdp_height_spin: SpinButton,
    rdp_color_dropdown: DropDown,
    rdp_audio_check: CheckButton,
    rdp_gateway_entry: Entry,
    rdp_shared_folders: Rc<RefCell<Vec<SharedFolder>>>,
    rdp_shared_folders_list: gtk4::ListBox,
    rdp_custom_args_entry: Entry,
    // VNC fields
    vnc_encoding_entry: Entry,
    vnc_compression_spin: SpinButton,
    vnc_quality_spin: SpinButton,
    vnc_view_only_check: CheckButton,
    vnc_scaling_check: CheckButton,
    vnc_clipboard_check: CheckButton,
    // SPICE fields
    spice_tls_check: CheckButton,
    spice_ca_cert_entry: Entry,
    spice_ca_cert_button: Button,
    spice_skip_verify_check: CheckButton,
    spice_usb_check: CheckButton,
    spice_clipboard_check: CheckButton,
    spice_compression_dropdown: DropDown,
    spice_shared_folders: Rc<RefCell<Vec<SharedFolder>>>,
    spice_shared_folders_list: gtk4::ListBox,
    // State
    editing_id: Rc<RefCell<Option<Uuid>>>,
    // Callback
    on_save: super::ConnectionCallback,
}

impl ConnectionDialog {
    /// Creates a new connection dialog
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new(parent: Option<&Window>) -> Self {
        let (window, cancel_btn, save_btn) = Self::create_window_with_header(parent);
        let notebook = Self::create_notebook(&window);

        // === Basic Tab ===
        let (
            basic_grid,
            name_entry,
            host_entry,
            port_spin,
            username_entry,
            tags_entry,
            protocol_dropdown,
            password_source_dropdown,
            password_entry,
            save_to_keepass_button,
        ) = Self::create_basic_tab();
        notebook.append_page(&basic_grid, Some(&Label::new(Some("Basic"))));

        // === Protocol-specific Tab ===
        let protocol_stack = Self::create_protocol_stack(&notebook);

        // SSH options
        let (
            ssh_box,
            ssh_auth_dropdown,
            ssh_key_entry,
            ssh_key_button,
            ssh_proxy_entry,
            ssh_control_master,
            ssh_startup_entry,
            ssh_options_entry,
        ) = Self::create_ssh_options();
        protocol_stack.add_named(&ssh_box, Some("ssh"));

        // RDP options
        let (
            rdp_box,
            rdp_width_spin,
            rdp_height_spin,
            rdp_color_dropdown,
            rdp_audio_check,
            rdp_gateway_entry,
            rdp_shared_folders,
            rdp_shared_folders_list,
            rdp_custom_args_entry,
        ) = Self::create_rdp_options();
        protocol_stack.add_named(&rdp_box, Some("rdp"));

        // VNC options
        let (
            vnc_box,
            vnc_encoding_entry,
            vnc_compression_spin,
            vnc_quality_spin,
            vnc_view_only_check,
            vnc_scaling_check,
            vnc_clipboard_check,
        ) = Self::create_vnc_options();
        protocol_stack.add_named(&vnc_box, Some("vnc"));

        // SPICE options
        let (
            spice_box,
            spice_tls_check,
            spice_ca_cert_entry,
            spice_ca_cert_button,
            spice_skip_verify_check,
            spice_usb_check,
            spice_clipboard_check,
            spice_compression_dropdown,
            spice_shared_folders,
            spice_shared_folders_list,
        ) = Self::create_spice_options();
        protocol_stack.add_named(&spice_box, Some("spice"));

        // Set initial protocol view
        protocol_stack.set_visible_child_name("ssh");

        // Connect protocol dropdown to stack
        Self::connect_protocol_dropdown(&protocol_dropdown, &protocol_stack, &port_spin);

        let on_save: super::ConnectionCallback = Rc::new(RefCell::new(None));
        let editing_id: Rc<RefCell<Option<Uuid>>> = Rc::new(RefCell::new(None));

        // Connect cancel button
        Self::connect_cancel_button(&cancel_btn, &window, &on_save);

        // Connect save button handler
        Self::connect_save_button(
            &save_btn,
            &window,
            &on_save,
            &editing_id,
            &name_entry,
            &host_entry,
            &port_spin,
            &username_entry,
            &tags_entry,
            &protocol_dropdown,
            &password_source_dropdown,
            &ssh_auth_dropdown,
            &ssh_key_entry,
            &ssh_proxy_entry,
            &ssh_control_master,
            &ssh_startup_entry,
            &ssh_options_entry,
            &rdp_width_spin,
            &rdp_height_spin,
            &rdp_color_dropdown,
            &rdp_audio_check,
            &rdp_gateway_entry,
            &rdp_shared_folders,
            &rdp_custom_args_entry,
            &vnc_encoding_entry,
            &vnc_compression_spin,
            &vnc_quality_spin,
            &vnc_view_only_check,
            &vnc_scaling_check,
            &vnc_clipboard_check,
            &spice_tls_check,
            &spice_ca_cert_entry,
            &spice_skip_verify_check,
            &spice_usb_check,
            &spice_clipboard_check,
            &spice_compression_dropdown,
            &spice_shared_folders,
        );

        Self {
            window,
            save_button: save_btn,
            name_entry,
            host_entry,
            port_spin,
            username_entry,
            tags_entry,
            protocol_dropdown,
            protocol_stack,
            password_source_dropdown,
            password_entry,
            save_to_keepass_button,
            ssh_auth_dropdown,
            ssh_key_entry,
            ssh_key_button,
            ssh_proxy_entry,
            ssh_control_master,
            ssh_startup_entry,
            ssh_options_entry,
            rdp_width_spin,
            rdp_height_spin,
            rdp_color_dropdown,
            rdp_audio_check,
            rdp_gateway_entry,
            rdp_shared_folders,
            rdp_shared_folders_list,
            rdp_custom_args_entry,
            vnc_encoding_entry,
            vnc_compression_spin,
            vnc_quality_spin,
            vnc_view_only_check,
            vnc_scaling_check,
            vnc_clipboard_check,
            spice_tls_check,
            spice_ca_cert_entry,
            spice_ca_cert_button,
            spice_skip_verify_check,
            spice_usb_check,
            spice_clipboard_check,
            spice_compression_dropdown,
            spice_shared_folders,
            spice_shared_folders_list,
            editing_id,
            on_save,
        }
    }

    /// Creates the main window with header bar containing Cancel/Save buttons
    fn create_window_with_header(parent: Option<&Window>) -> (Window, Button, Button) {
        let window = Window::builder()
            .title("New Connection")
            .modal(true)
            .default_width(550)
            .default_height(500)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let save_btn = Button::builder()
            .label("Save")
            .css_classes(["suggested-action"])
            .build();
        header.pack_start(&cancel_btn);
        header.pack_end(&save_btn);
        window.set_titlebar(Some(&header));
        window.set_default_widget(Some(&save_btn));

        (window, cancel_btn, save_btn)
    }

    /// Creates the notebook widget and adds it to the window
    fn create_notebook(window: &Window) -> Notebook {
        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let notebook = Notebook::new();
        content.append(&notebook);
        window.set_child(Some(&content));

        notebook
    }

    /// Creates the protocol stack and adds it to the notebook
    fn create_protocol_stack(notebook: &Notebook) -> Stack {
        let protocol_stack = Stack::new();
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&protocol_stack)
            .build();
        notebook.append_page(&scrolled, Some(&Label::new(Some("Protocol"))));
        protocol_stack
    }

    /// Connects the protocol dropdown to update the stack and port
    fn connect_protocol_dropdown(dropdown: &DropDown, stack: &Stack, port_spin: &SpinButton) {
        let stack_clone = stack.clone();
        let port_clone = port_spin.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            let protocols = ["ssh", "rdp", "vnc", "spice"];
            let selected = dropdown.selected() as usize;
            if selected < protocols.len() {
                let protocol_id = protocols[selected];
                stack_clone.set_visible_child_name(protocol_id);
                let default_port = Self::get_default_port(protocol_id);
                if Self::is_default_port(port_clone.value()) {
                    port_clone.set_value(default_port);
                }
            }
        });
    }

    /// Returns the default port for a protocol
    fn get_default_port(protocol_id: &str) -> f64 {
        match protocol_id {
            "rdp" => 3389.0,
            "vnc" | "spice" => 5900.0,
            _ => 22.0,
        }
    }

    /// Checks if the port value is one of the default ports
    fn is_default_port(port: f64) -> bool {
        const EPSILON: f64 = 0.5;
        (port - 22.0).abs() < EPSILON
            || (port - 3389.0).abs() < EPSILON
            || (port - 5900.0).abs() < EPSILON
    }

    /// Connects the cancel button to close the dialog
    fn connect_cancel_button(
        cancel_btn: &Button,
        window: &Window,
        on_save: &super::ConnectionCallback,
    ) {
        let window_clone = window.clone();
        let on_save_clone = on_save.clone();
        cancel_btn.connect_clicked(move |_| {
            if let Some(ref cb) = *on_save_clone.borrow() {
                cb(None);
            }
            window_clone.close();
        });
    }

    /// Connects the save button to validate and save the connection
    #[allow(clippy::too_many_arguments)]
    fn connect_save_button(
        save_btn: &Button,
        window: &Window,
        on_save: &super::ConnectionCallback,
        editing_id: &Rc<RefCell<Option<Uuid>>>,
        name_entry: &Entry,
        host_entry: &Entry,
        port_spin: &SpinButton,
        username_entry: &Entry,
        tags_entry: &Entry,
        protocol_dropdown: &DropDown,
        password_source_dropdown: &DropDown,
        ssh_auth_dropdown: &DropDown,
        ssh_key_entry: &Entry,
        ssh_proxy_entry: &Entry,
        ssh_control_master: &CheckButton,
        ssh_startup_entry: &Entry,
        ssh_options_entry: &Entry,
        rdp_width_spin: &SpinButton,
        rdp_height_spin: &SpinButton,
        rdp_color_dropdown: &DropDown,
        rdp_audio_check: &CheckButton,
        rdp_gateway_entry: &Entry,
        rdp_shared_folders: &Rc<RefCell<Vec<SharedFolder>>>,
        rdp_custom_args_entry: &Entry,
        vnc_encoding_entry: &Entry,
        vnc_compression_spin: &SpinButton,
        vnc_quality_spin: &SpinButton,
        vnc_view_only_check: &CheckButton,
        vnc_scaling_check: &CheckButton,
        vnc_clipboard_check: &CheckButton,
        spice_tls_check: &CheckButton,
        spice_ca_cert_entry: &Entry,
        spice_skip_verify_check: &CheckButton,
        spice_usb_check: &CheckButton,
        spice_clipboard_check: &CheckButton,
        spice_compression_dropdown: &DropDown,
        spice_shared_folders: &Rc<RefCell<Vec<SharedFolder>>>,
    ) {
        let window = window.clone();
        let on_save = on_save.clone();
        let name_entry = name_entry.clone();
        let host_entry = host_entry.clone();
        let port_spin = port_spin.clone();
        let username_entry = username_entry.clone();
        let tags_entry = tags_entry.clone();
        let protocol_dropdown = protocol_dropdown.clone();
        let password_source_dropdown = password_source_dropdown.clone();
        let ssh_auth_dropdown = ssh_auth_dropdown.clone();
        let ssh_key_entry = ssh_key_entry.clone();
        let ssh_proxy_entry = ssh_proxy_entry.clone();
        let ssh_control_master = ssh_control_master.clone();
        let ssh_startup_entry = ssh_startup_entry.clone();
        let ssh_options_entry = ssh_options_entry.clone();
        let rdp_width_spin = rdp_width_spin.clone();
        let rdp_height_spin = rdp_height_spin.clone();
        let rdp_color_dropdown = rdp_color_dropdown.clone();
        let rdp_audio_check = rdp_audio_check.clone();
        let rdp_gateway_entry = rdp_gateway_entry.clone();
        let rdp_shared_folders = rdp_shared_folders.clone();
        let rdp_custom_args_entry = rdp_custom_args_entry.clone();
        let vnc_encoding_entry = vnc_encoding_entry.clone();
        let vnc_compression_spin = vnc_compression_spin.clone();
        let vnc_quality_spin = vnc_quality_spin.clone();
        let vnc_view_only_check = vnc_view_only_check.clone();
        let vnc_scaling_check = vnc_scaling_check.clone();
        let vnc_clipboard_check = vnc_clipboard_check.clone();
        let spice_tls_check = spice_tls_check.clone();
        let spice_ca_cert_entry = spice_ca_cert_entry.clone();
        let spice_skip_verify_check = spice_skip_verify_check.clone();
        let spice_usb_check = spice_usb_check.clone();
        let spice_clipboard_check = spice_clipboard_check.clone();
        let spice_compression_dropdown = spice_compression_dropdown.clone();
        let spice_shared_folders = spice_shared_folders.clone();
        let editing_id = editing_id.clone();

        save_btn.connect_clicked(move |_| {
            let data = ConnectionDialogData {
                name_entry: &name_entry,
                host_entry: &host_entry,
                port_spin: &port_spin,
                username_entry: &username_entry,
                tags_entry: &tags_entry,
                protocol_dropdown: &protocol_dropdown,
                password_source_dropdown: &password_source_dropdown,
                ssh_auth_dropdown: &ssh_auth_dropdown,
                ssh_key_entry: &ssh_key_entry,
                ssh_proxy_entry: &ssh_proxy_entry,
                ssh_control_master: &ssh_control_master,
                ssh_startup_entry: &ssh_startup_entry,
                ssh_options_entry: &ssh_options_entry,
                rdp_width_spin: &rdp_width_spin,
                rdp_height_spin: &rdp_height_spin,
                rdp_color_dropdown: &rdp_color_dropdown,
                rdp_audio_check: &rdp_audio_check,
                rdp_gateway_entry: &rdp_gateway_entry,
                rdp_shared_folders: &rdp_shared_folders,
                rdp_custom_args_entry: &rdp_custom_args_entry,
                vnc_encoding_entry: &vnc_encoding_entry,
                vnc_compression_spin: &vnc_compression_spin,
                vnc_quality_spin: &vnc_quality_spin,
                vnc_view_only_check: &vnc_view_only_check,
                vnc_scaling_check: &vnc_scaling_check,
                vnc_clipboard_check: &vnc_clipboard_check,
                spice_tls_check: &spice_tls_check,
                spice_ca_cert_entry: &spice_ca_cert_entry,
                spice_skip_verify_check: &spice_skip_verify_check,
                spice_usb_check: &spice_usb_check,
                spice_clipboard_check: &spice_clipboard_check,
                spice_compression_dropdown: &spice_compression_dropdown,
                spice_shared_folders: &spice_shared_folders,
                editing_id: &editing_id,
            };

            if let Err(err) = data.validate() {
                let alert = gtk4::AlertDialog::builder()
                    .message("Validation Error")
                    .detail(&err)
                    .modal(true)
                    .build();
                alert.show(Some(&window));
                return;
            }

            if let Some(conn) = data.build_connection() {
                if let Some(ref cb) = *on_save.borrow() {
                    cb(Some(conn));
                }
                window.close();
            }
        });
    }

    /// Creates a labeled entry row in a grid
    fn create_labeled_entry(grid: &Grid, row: &mut i32, label: &str, placeholder: &str) -> Entry {
        let label_widget = Label::builder()
            .label(label)
            .halign(gtk4::Align::End)
            .build();
        let entry = Entry::builder()
            .hexpand(true)
            .placeholder_text(placeholder)
            .build();
        grid.attach(&label_widget, 0, *row, 1, 1);
        grid.attach(&entry, 1, *row, 2, 1);
        *row += 1;
        entry
    }

    /// Creates a labeled dropdown row in a grid
    fn create_labeled_dropdown(
        grid: &Grid,
        row: &mut i32,
        label: &str,
        options: &[&str],
        default: u32,
    ) -> DropDown {
        let label_widget = Label::builder()
            .label(label)
            .halign(gtk4::Align::End)
            .build();
        let list = StringList::new(options);
        let dropdown = DropDown::new(Some(list), gtk4::Expression::NONE);
        dropdown.set_selected(default);
        grid.attach(&label_widget, 0, *row, 1, 1);
        grid.attach(&dropdown, 1, *row, 2, 1);
        *row += 1;
        dropdown
    }

    #[allow(clippy::type_complexity)]
    fn create_basic_tab() -> (
        Grid,
        Entry,
        Entry,
        SpinButton,
        Entry,
        Entry,
        DropDown,
        DropDown,
        Entry,
        Button,
    ) {
        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let mut row = 0;

        // Name
        let name_entry = Self::create_labeled_entry(&grid, &mut row, "Name:", "Connection name");

        // Protocol
        let protocol_dropdown = Self::create_labeled_dropdown(
            &grid,
            &mut row,
            "Protocol:",
            &["SSH", "RDP", "VNC", "SPICE"],
            0,
        );

        // Host
        let host_entry = Self::create_labeled_entry(&grid, &mut row, "Host:", "hostname or IP");

        // Port
        let port_spin = Self::create_port_spin(&grid, &mut row);

        // Username
        let username_entry = Self::create_username_entry(&grid, &mut row);

        // Password Source
        let password_source_dropdown = Self::create_labeled_dropdown(
            &grid,
            &mut row,
            "Password:",
            &["Prompt", "Stored", "KeePass", "Keyring", "None"],
            0,
        );

        // Password entry with Save to KeePass button
        let (password_entry, save_to_keepass_button) =
            Self::create_password_entry_row(&grid, &mut row);

        // Tags
        let tags_entry = Self::create_labeled_entry(&grid, &mut row, "Tags:", "tag1, tag2, ...");

        (
            grid,
            name_entry,
            host_entry,
            port_spin,
            username_entry,
            tags_entry,
            protocol_dropdown,
            password_source_dropdown,
            password_entry,
            save_to_keepass_button,
        )
    }

    /// Creates the port spin button row
    fn create_port_spin(grid: &Grid, row: &mut i32) -> SpinButton {
        let port_label = Label::builder()
            .label("Port:")
            .halign(gtk4::Align::End)
            .build();
        let port_adj = gtk4::Adjustment::new(22.0, 1.0, 65535.0, 1.0, 10.0, 0.0);
        let port_spin = SpinButton::builder()
            .adjustment(&port_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&port_label, 0, *row, 1, 1);
        grid.attach(&port_spin, 1, *row, 1, 1);
        *row += 1;
        port_spin
    }

    /// Creates the username entry with current user as placeholder
    fn create_username_entry(grid: &Grid, row: &mut i32) -> Entry {
        let username_label = Label::builder()
            .label("Username:")
            .halign(gtk4::Align::End)
            .build();
        let current_user = std::env::var("USER").unwrap_or_default();
        let placeholder = if current_user.is_empty() {
            "(optional)".to_string()
        } else {
            format!("(default: {current_user})")
        };
        let username_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text(&placeholder)
            .build();
        grid.attach(&username_label, 0, *row, 1, 1);
        grid.attach(&username_entry, 1, *row, 2, 1);
        *row += 1;
        username_entry
    }

    /// Creates the password entry row with Save to `KeePass` button
    fn create_password_entry_row(grid: &Grid, row: &mut i32) -> (Entry, Button) {
        let password_entry_label = Label::builder()
            .label("Password Value:")
            .halign(gtk4::Align::End)
            .build();
        let password_hbox = GtkBox::new(Orientation::Horizontal, 4);
        let password_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Enter password (for Stored or KeePass)")
            .visibility(false)
            .input_purpose(gtk4::InputPurpose::Password)
            .build();
        let save_to_keepass_button = Button::builder()
            .label("Save to KeePass")
            .tooltip_text("Save password to KeePass database")
            .sensitive(false)
            .build();
        password_hbox.append(&password_entry);
        password_hbox.append(&save_to_keepass_button);
        grid.attach(&password_entry_label, 0, *row, 1, 1);
        grid.attach(&password_hbox, 1, *row, 2, 1);
        *row += 1;
        (password_entry, save_to_keepass_button)
    }

    fn create_ssh_options() -> (
        GtkBox,
        DropDown,
        Entry,
        Button,
        Entry,
        CheckButton,
        Entry,
        Entry,
    ) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        // Auth method - using DropDown
        let auth_label = Label::builder()
            .label("Auth Method:")
            .halign(gtk4::Align::End)
            .build();
        let auth_list = StringList::new(&[
            "Password",
            "Public Key",
            "Keyboard Interactive",
            "SSH Agent",
        ]);
        let auth_dropdown = DropDown::new(Some(auth_list), gtk4::Expression::NONE);
        auth_dropdown.set_selected(0); // Password by default
        grid.attach(&auth_label, 0, row, 1, 1);
        grid.attach(&auth_dropdown, 1, row, 2, 1);
        row += 1;

        // Key path with file chooser button (uses portal on Wayland)
        let key_label = Label::builder()
            .label("Key File:")
            .halign(gtk4::Align::End)
            .build();
        let key_hbox = GtkBox::new(Orientation::Horizontal, 4);
        let key_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Path to SSH key")
            .build();
        let key_button = Button::builder().label("Browse...").build();
        key_hbox.append(&key_entry);
        key_hbox.append(&key_button);
        grid.attach(&key_label, 0, row, 1, 1);
        grid.attach(&key_hbox, 1, row, 2, 1);
        row += 1;

        // ProxyJump
        let proxy_label = Label::builder()
            .label("ProxyJump:")
            .halign(gtk4::Align::End)
            .build();
        let proxy_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("user@jumphost")
            .build();
        grid.attach(&proxy_label, 0, row, 1, 1);
        grid.attach(&proxy_entry, 1, row, 2, 1);
        row += 1;

        // ControlMaster
        let control_master = CheckButton::builder()
            .label("Enable ControlMaster (connection multiplexing)")
            .build();
        grid.attach(&control_master, 1, row, 2, 1);
        row += 1;

        // Startup command
        let startup_label = Label::builder()
            .label("Startup Command:")
            .halign(gtk4::Align::End)
            .build();
        let startup_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Command to run on connect")
            .build();
        grid.attach(&startup_label, 0, row, 1, 1);
        grid.attach(&startup_entry, 1, row, 2, 1);
        row += 1;

        // Custom options
        let options_label = Label::builder()
            .label("Custom Options:")
            .halign(gtk4::Align::End)
            .build();
        let options_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Key=Value, Key2=Value2")
            .build();
        grid.attach(&options_label, 0, row, 1, 1);
        grid.attach(&options_entry, 1, row, 2, 1);

        (
            vbox,
            auth_dropdown,
            key_entry,
            key_button,
            proxy_entry,
            control_master,
            startup_entry,
            options_entry,
        )
    }

    #[allow(clippy::type_complexity)]
    fn create_rdp_options() -> (
        GtkBox,
        SpinButton,
        SpinButton,
        DropDown,
        CheckButton,
        Entry,
        Rc<RefCell<Vec<SharedFolder>>>,
        gtk4::ListBox,
        Entry,
    ) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        // Resolution
        let (width_spin, height_spin) = Self::create_rdp_resolution_row(&grid, &mut row);

        // Color depth
        let color_dropdown = Self::create_rdp_color_depth_row(&grid, &mut row);

        // Audio redirect
        let audio_check = CheckButton::builder()
            .label("Enable audio redirection")
            .build();
        grid.attach(&audio_check, 1, row, 2, 1);
        row += 1;

        // Gateway
        let gateway_entry = Self::create_rdp_gateway_row(&grid, &mut row);

        // Shared Folders section
        let (shared_folders, folders_list) =
            Self::create_rdp_shared_folders_section(&grid, &mut row);

        // Custom args
        let args_entry = Self::create_custom_args_row(&grid, &mut row);

        (
            vbox,
            width_spin,
            height_spin,
            color_dropdown,
            audio_check,
            gateway_entry,
            shared_folders,
            folders_list,
            args_entry,
        )
    }

    /// Creates the RDP resolution row
    fn create_rdp_resolution_row(grid: &Grid, row: &mut i32) -> (SpinButton, SpinButton) {
        let res_label = Label::builder()
            .label("Resolution:")
            .halign(gtk4::Align::End)
            .build();
        let res_hbox = GtkBox::new(Orientation::Horizontal, 4);
        let width_adj = gtk4::Adjustment::new(1920.0, 640.0, 7680.0, 1.0, 100.0, 0.0);
        let width_spin = SpinButton::builder()
            .adjustment(&width_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        let x_label = Label::new(Some("x"));
        let height_adj = gtk4::Adjustment::new(1080.0, 480.0, 4320.0, 1.0, 100.0, 0.0);
        let height_spin = SpinButton::builder()
            .adjustment(&height_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        res_hbox.append(&width_spin);
        res_hbox.append(&x_label);
        res_hbox.append(&height_spin);
        grid.attach(&res_label, 0, *row, 1, 1);
        grid.attach(&res_hbox, 1, *row, 2, 1);
        *row += 1;

        (width_spin, height_spin)
    }

    /// Creates the RDP color depth row
    fn create_rdp_color_depth_row(grid: &Grid, row: &mut i32) -> DropDown {
        let color_label = Label::builder()
            .label("Color Depth:")
            .halign(gtk4::Align::End)
            .build();
        let color_list = StringList::new(&[
            "32-bit (True Color)",
            "24-bit",
            "16-bit (High Color)",
            "15-bit",
            "8-bit",
        ]);
        let color_dropdown = DropDown::new(Some(color_list), gtk4::Expression::NONE);
        color_dropdown.set_selected(0);
        grid.attach(&color_label, 0, *row, 1, 1);
        grid.attach(&color_dropdown, 1, *row, 2, 1);
        *row += 1;

        color_dropdown
    }

    /// Creates the RDP gateway row
    fn create_rdp_gateway_row(grid: &Grid, row: &mut i32) -> Entry {
        let gateway_label = Label::builder()
            .label("RDP Gateway:")
            .halign(gtk4::Align::End)
            .build();
        let gateway_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("gateway.example.com")
            .build();
        grid.attach(&gateway_label, 0, *row, 1, 1);
        grid.attach(&gateway_entry, 1, *row, 2, 1);
        *row += 1;

        gateway_entry
    }

    /// Creates the shared folders section for RDP
    fn create_rdp_shared_folders_section(
        grid: &Grid,
        row: &mut i32,
    ) -> (Rc<RefCell<Vec<SharedFolder>>>, gtk4::ListBox) {
        let folders_label = Label::builder()
            .label("Shared Folders:")
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::Start)
            .build();
        grid.attach(&folders_label, 0, *row, 1, 1);

        let folders_vbox = GtkBox::new(Orientation::Vertical, 4);
        folders_vbox.set_hexpand(true);

        let folders_list = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();
        folders_list.set_placeholder(Some(&Label::new(Some("No shared folders"))));

        let folders_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_height(80)
            .max_content_height(120)
            .child(&folders_list)
            .build();
        folders_vbox.append(&folders_scrolled);

        let folders_buttons = GtkBox::new(Orientation::Horizontal, 4);
        let add_folder_btn = Button::builder().label("Add...").build();
        let remove_folder_btn = Button::builder().label("Remove").sensitive(false).build();
        folders_buttons.append(&add_folder_btn);
        folders_buttons.append(&remove_folder_btn);
        folders_vbox.append(&folders_buttons);

        grid.attach(&folders_vbox, 1, *row, 2, 1);
        *row += 1;

        let shared_folders: Rc<RefCell<Vec<SharedFolder>>> = Rc::new(RefCell::new(Vec::new()));

        // Connect add folder button
        Self::connect_add_folder_button(&add_folder_btn, &folders_list, &shared_folders);

        // Connect remove folder button
        Self::connect_remove_folder_button(&remove_folder_btn, &folders_list, &shared_folders);

        // Enable/disable remove button based on selection
        let remove_btn_for_selection = remove_folder_btn;
        folders_list.connect_row_selected(move |_, row| {
            remove_btn_for_selection.set_sensitive(row.is_some());
        });

        (shared_folders, folders_list)
    }

    /// Connects the add folder button to show file dialog and add folder
    fn connect_add_folder_button(
        add_btn: &Button,
        folders_list: &gtk4::ListBox,
        shared_folders: &Rc<RefCell<Vec<SharedFolder>>>,
    ) {
        let folders_list_clone = folders_list.clone();
        let shared_folders_clone = shared_folders.clone();
        add_btn.connect_clicked(move |btn| {
            let file_dialog = FileDialog::builder()
                .title("Select Folder to Share")
                .modal(true)
                .build();

            let folders_list = folders_list_clone.clone();
            let shared_folders = shared_folders_clone.clone();
            let parent = btn.root().and_then(|r| r.downcast::<Window>().ok());

            file_dialog.select_folder(
                parent.as_ref(),
                gtk4::gio::Cancellable::NONE,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let share_name = path
                                .file_name()
                                .map_or_else(|| "Share".to_string(), |n| n.to_string_lossy().to_string());

                            let folder = SharedFolder {
                                local_path: path.clone(),
                                share_name: share_name.clone(),
                            };

                            shared_folders.borrow_mut().push(folder);
                            Self::add_folder_row_to_list(&folders_list, &path, &share_name);
                        }
                    }
                },
            );
        });
    }

    /// Adds a folder row to the list UI
    fn add_folder_row_to_list(folders_list: &gtk4::ListBox, path: &std::path::Path, share_name: &str) {
        let row_box = GtkBox::new(Orientation::Horizontal, 8);
        row_box.set_margin_top(4);
        row_box.set_margin_bottom(4);
        row_box.set_margin_start(8);
        row_box.set_margin_end(8);

        let path_label = Label::builder()
            .label(path.to_string_lossy().as_ref())
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::Middle)
            .build();
        let name_label = Label::builder()
            .label(format!("→ {share_name}"))
            .halign(gtk4::Align::End)
            .build();

        row_box.append(&path_label);
        row_box.append(&name_label);
        folders_list.append(&row_box);
    }

    /// Connects the remove folder button
    fn connect_remove_folder_button(
        remove_btn: &Button,
        folders_list: &gtk4::ListBox,
        shared_folders: &Rc<RefCell<Vec<SharedFolder>>>,
    ) {
        let folders_list_clone = folders_list.clone();
        let shared_folders_clone = shared_folders.clone();
        remove_btn.connect_clicked(move |_| {
            if let Some(selected_row) = folders_list_clone.selected_row() {
                if let Ok(index) = usize::try_from(selected_row.index()) {
                    if index < shared_folders_clone.borrow().len() {
                        shared_folders_clone.borrow_mut().remove(index);
                        folders_list_clone.remove(&selected_row);
                    }
                }
            }
        });
    }

    /// Creates a custom args row for protocol options
    fn create_custom_args_row(grid: &Grid, row: &mut i32) -> Entry {
        let args_label = Label::builder()
            .label("Custom Args:")
            .halign(gtk4::Align::End)
            .build();
        let args_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Additional command-line arguments")
            .build();
        grid.attach(&args_label, 0, *row, 1, 1);
        grid.attach(&args_entry, 1, *row, 2, 1);
        *row += 1;

        args_entry
    }

    #[allow(clippy::type_complexity)]
    fn create_vnc_options() -> (GtkBox, Entry, SpinButton, SpinButton, CheckButton, CheckButton, CheckButton) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        // Encoding
        let encoding_label = Label::builder()
            .label("Encoding:")
            .halign(gtk4::Align::End)
            .build();
        let encoding_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("tight, zrle, hextile")
            .build();
        grid.attach(&encoding_label, 0, row, 1, 1);
        grid.attach(&encoding_entry, 1, row, 2, 1);
        row += 1;

        // Compression
        let compression_label = Label::builder()
            .label("Compression:")
            .halign(gtk4::Align::End)
            .build();
        let compression_adj = gtk4::Adjustment::new(6.0, 0.0, 9.0, 1.0, 1.0, 0.0);
        let compression_spin = SpinButton::builder()
            .adjustment(&compression_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&compression_label, 0, row, 1, 1);
        grid.attach(&compression_spin, 1, row, 1, 1);
        row += 1;

        // Quality
        let quality_label = Label::builder()
            .label("Quality:")
            .halign(gtk4::Align::End)
            .build();
        let quality_adj = gtk4::Adjustment::new(6.0, 0.0, 9.0, 1.0, 1.0, 0.0);
        let quality_spin = SpinButton::builder()
            .adjustment(&quality_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&quality_label, 0, row, 1, 1);
        grid.attach(&quality_spin, 1, row, 1, 1);
        row += 1;

        // View-only mode
        let view_only_check = CheckButton::builder()
            .label("View-only mode (no input)")
            .build();
        grid.attach(&view_only_check, 1, row, 2, 1);
        row += 1;

        // Scaling
        let scaling_check = CheckButton::builder()
            .label("Scale display to fit window")
            .active(true)
            .build();
        grid.attach(&scaling_check, 1, row, 2, 1);
        row += 1;

        // Clipboard sharing
        let clipboard_check = CheckButton::builder()
            .label("Enable clipboard sharing")
            .active(true)
            .build();
        grid.attach(&clipboard_check, 1, row, 2, 1);

        (vbox, encoding_entry, compression_spin, quality_spin, view_only_check, scaling_check, clipboard_check)
    }

    #[allow(clippy::type_complexity, clippy::too_many_lines)]
    fn create_spice_options() -> (
        GtkBox,
        CheckButton,
        Entry,
        Button,
        CheckButton,
        CheckButton,
        CheckButton,
        DropDown,
        Rc<RefCell<Vec<SharedFolder>>>,
        gtk4::ListBox,
    ) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        // TLS enabled
        let tls_check = CheckButton::builder()
            .label("Enable TLS encryption")
            .build();
        grid.attach(&tls_check, 1, row, 2, 1);
        row += 1;

        // CA certificate path
        let ca_cert_label = Label::builder()
            .label("CA Certificate:")
            .halign(gtk4::Align::End)
            .build();
        let ca_cert_hbox = GtkBox::new(Orientation::Horizontal, 4);
        let ca_cert_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Path to CA certificate (optional)")
            .build();
        let ca_cert_button = Button::builder().label("Browse...").build();
        ca_cert_hbox.append(&ca_cert_entry);
        ca_cert_hbox.append(&ca_cert_button);
        grid.attach(&ca_cert_label, 0, row, 1, 1);
        grid.attach(&ca_cert_hbox, 1, row, 2, 1);
        row += 1;

        // Skip certificate verification
        let skip_verify_check = CheckButton::builder()
            .label("Skip certificate verification (insecure)")
            .build();
        grid.attach(&skip_verify_check, 1, row, 2, 1);
        row += 1;

        // USB redirection
        let usb_check = CheckButton::builder()
            .label("Enable USB redirection")
            .build();
        grid.attach(&usb_check, 1, row, 2, 1);
        row += 1;

        // Clipboard sharing
        let clipboard_check = CheckButton::builder()
            .label("Enable clipboard sharing")
            .active(true)
            .build();
        grid.attach(&clipboard_check, 1, row, 2, 1);
        row += 1;

        // Image compression
        let compression_label = Label::builder()
            .label("Image Compression:")
            .halign(gtk4::Align::End)
            .build();
        let compression_list = StringList::new(&["Auto", "Off", "GLZ", "LZ", "QUIC"]);
        let compression_dropdown = DropDown::new(Some(compression_list), gtk4::Expression::NONE);
        compression_dropdown.set_selected(0); // Auto by default
        grid.attach(&compression_label, 0, row, 1, 1);
        grid.attach(&compression_dropdown, 1, row, 2, 1);
        row += 1;

        // Shared Folders section
        let folders_label = Label::builder()
            .label("Shared Folders:")
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::Start)
            .build();
        grid.attach(&folders_label, 0, row, 1, 1);

        let folders_vbox = GtkBox::new(Orientation::Vertical, 4);
        folders_vbox.set_hexpand(true);

        let folders_list = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();
        folders_list.set_placeholder(Some(&Label::new(Some("No shared folders"))));

        let folders_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_height(80)
            .max_content_height(120)
            .child(&folders_list)
            .build();
        folders_vbox.append(&folders_scrolled);

        let folders_buttons = GtkBox::new(Orientation::Horizontal, 4);
        let add_folder_btn = Button::builder().label("Add...").build();
        let remove_folder_btn = Button::builder().label("Remove").sensitive(false).build();
        folders_buttons.append(&add_folder_btn);
        folders_buttons.append(&remove_folder_btn);
        folders_vbox.append(&folders_buttons);

        grid.attach(&folders_vbox, 1, row, 2, 1);

        let shared_folders: Rc<RefCell<Vec<SharedFolder>>> = Rc::new(RefCell::new(Vec::new()));

        // Connect add folder button
        Self::connect_add_folder_button(&add_folder_btn, &folders_list, &shared_folders);

        // Connect remove folder button
        Self::connect_remove_folder_button(&remove_folder_btn, &folders_list, &shared_folders);

        // Enable/disable remove button based on selection
        let remove_btn_for_selection = remove_folder_btn;
        folders_list.connect_row_selected(move |_, row| {
            remove_btn_for_selection.set_sensitive(row.is_some());
        });

        (
            vbox,
            tls_check,
            ca_cert_entry,
            ca_cert_button,
            skip_verify_check,
            usb_check,
            clipboard_check,
            compression_dropdown,
            shared_folders,
            folders_list,
        )
    }

    /// Sets up the file chooser button for SSH key selection using portal
    pub fn setup_key_file_chooser(&self, parent_window: Option<&Window>) {
        let key_entry = self.ssh_key_entry.clone();
        let parent = parent_window.cloned();

        self.ssh_key_button.connect_clicked(move |_| {
            let file_dialog = FileDialog::builder()
                .title("Select SSH Key")
                .modal(true)
                .build();

            // Set initial folder to ~/.ssh if it exists
            if let Some(home) = std::env::var_os("HOME") {
                let ssh_dir = PathBuf::from(home).join(".ssh");
                if ssh_dir.exists() {
                    let gio_file = gtk4::gio::File::for_path(&ssh_dir);
                    file_dialog.set_initial_folder(Some(&gio_file));
                }
            }

            let entry = key_entry.clone();
            file_dialog.open(
                parent.as_ref(),
                gtk4::gio::Cancellable::NONE,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            entry.set_text(&path.to_string_lossy());
                        }
                    }
                },
            );
        });
    }

    /// Sets up the file chooser button for SPICE CA certificate selection using portal
    pub fn setup_ca_cert_file_chooser(&self, parent_window: Option<&Window>) {
        let ca_cert_entry = self.spice_ca_cert_entry.clone();
        let parent = parent_window.cloned();

        self.spice_ca_cert_button.connect_clicked(move |_| {
            let file_dialog = FileDialog::builder()
                .title("Select CA Certificate")
                .modal(true)
                .build();

            let entry = ca_cert_entry.clone();
            file_dialog.open(
                parent.as_ref(),
                gtk4::gio::Cancellable::NONE,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            entry.set_text(&path.to_string_lossy());
                        }
                    }
                },
            );
        });
    }

    /// Populates the dialog with an existing connection for editing
    pub fn set_connection(&self, conn: &Connection) {
        self.window.set_title(Some("Edit Connection"));
        *self.editing_id.borrow_mut() = Some(conn.id);

        // Basic fields
        self.name_entry.set_text(&conn.name);
        self.host_entry.set_text(&conn.host);
        self.port_spin.set_value(f64::from(conn.port));
        if let Some(ref username) = conn.username {
            self.username_entry.set_text(username);
        }
        self.tags_entry.set_text(&conn.tags.join(", "));

        // Password source - map enum to dropdown index
        // Dropdown order: Prompt(0), Stored(1), KeePass(2), Keyring(3), None(4)
        let password_source_idx = match conn.password_source {
            PasswordSource::Prompt => 0,
            PasswordSource::Stored => 1,
            PasswordSource::KeePass => 2,
            PasswordSource::Keyring => 3,
            PasswordSource::None => 4,
        };
        self.password_source_dropdown.set_selected(password_source_idx);

        // Protocol and protocol-specific fields
        match &conn.protocol_config {
            ProtocolConfig::Ssh(ssh) => {
                self.protocol_dropdown.set_selected(0); // SSH
                self.protocol_stack.set_visible_child_name("ssh");
                self.set_ssh_config(ssh);
            }
            ProtocolConfig::Rdp(rdp) => {
                self.protocol_dropdown.set_selected(1); // RDP
                self.protocol_stack.set_visible_child_name("rdp");
                self.set_rdp_config(rdp);
            }
            ProtocolConfig::Vnc(vnc) => {
                self.protocol_dropdown.set_selected(2); // VNC
                self.protocol_stack.set_visible_child_name("vnc");
                self.set_vnc_config(vnc);
            }
            ProtocolConfig::Spice(spice) => {
                self.protocol_dropdown.set_selected(3); // SPICE
                self.protocol_stack.set_visible_child_name("spice");
                self.set_spice_config(spice);
            }
        }
    }

    fn set_ssh_config(&self, ssh: &SshConfig) {
        let auth_idx = match ssh.auth_method {
            SshAuthMethod::Password => 0,
            SshAuthMethod::PublicKey => 1,
            SshAuthMethod::KeyboardInteractive => 2,
            SshAuthMethod::Agent => 3,
        };
        self.ssh_auth_dropdown.set_selected(auth_idx);

        if let Some(ref key_path) = ssh.key_path {
            self.ssh_key_entry.set_text(&key_path.to_string_lossy());
        }
        if let Some(ref proxy) = ssh.proxy_jump {
            self.ssh_proxy_entry.set_text(proxy);
        }
        self.ssh_control_master.set_active(ssh.use_control_master);
        if let Some(ref cmd) = ssh.startup_command {
            self.ssh_startup_entry.set_text(cmd);
        }

        // Format custom options as "Key=Value, Key2=Value2"
        if !ssh.custom_options.is_empty() {
            let opts: Vec<String> = ssh
                .custom_options
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            self.ssh_options_entry.set_text(&opts.join(", "));
        }
    }

    fn set_rdp_config(&self, rdp: &RdpConfig) {
        if let Some(ref res) = rdp.resolution {
            self.rdp_width_spin.set_value(f64::from(res.width));
            self.rdp_height_spin.set_value(f64::from(res.height));
        }
        if let Some(depth) = rdp.color_depth {
            // Map color depth to dropdown index: 32->0, 24->1, 16->2, 15->3, 8->4
            let idx = match depth {
                24 => 1,
                16 => 2,
                15 => 3,
                8 => 4,
                _ => 0, // 32 and any other value default to 0
            };
            self.rdp_color_dropdown.set_selected(idx);
        }
        self.rdp_audio_check.set_active(rdp.audio_redirect);
        if let Some(ref gw) = rdp.gateway {
            self.rdp_gateway_entry.set_text(&gw.hostname);
        }

        // Populate shared folders
        self.rdp_shared_folders.borrow_mut().clear();
        // Clear existing list items
        while let Some(row) = self.rdp_shared_folders_list.row_at_index(0) {
            self.rdp_shared_folders_list.remove(&row);
        }
        for folder in &rdp.shared_folders {
            self.rdp_shared_folders.borrow_mut().push(folder.clone());

            // Add to UI
            let row_box = GtkBox::new(Orientation::Horizontal, 8);
            row_box.set_margin_top(4);
            row_box.set_margin_bottom(4);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);

            let path_label = Label::builder()
                .label(folder.local_path.to_string_lossy().as_ref())
                .hexpand(true)
                .halign(gtk4::Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::Middle)
                .build();
            let name_label = Label::builder()
                .label(format!("→ {}", folder.share_name))
                .halign(gtk4::Align::End)
                .build();

            row_box.append(&path_label);
            row_box.append(&name_label);
            self.rdp_shared_folders_list.append(&row_box);
        }

        if !rdp.custom_args.is_empty() {
            self.rdp_custom_args_entry
                .set_text(&rdp.custom_args.join(" "));
        }
    }

    fn set_vnc_config(&self, vnc: &VncConfig) {
        if let Some(ref enc) = vnc.encoding {
            self.vnc_encoding_entry.set_text(enc);
        }
        if let Some(comp) = vnc.compression {
            self.vnc_compression_spin.set_value(f64::from(comp));
        }
        if let Some(qual) = vnc.quality {
            self.vnc_quality_spin.set_value(f64::from(qual));
        }
        // Native embedding options - defaults for now
        self.vnc_view_only_check.set_active(false);
        self.vnc_scaling_check.set_active(true);
        self.vnc_clipboard_check.set_active(true);
    }

    fn set_spice_config(&self, spice: &SpiceConfig) {
        self.spice_tls_check.set_active(spice.tls_enabled);
        if let Some(ref path) = spice.ca_cert_path {
            self.spice_ca_cert_entry.set_text(&path.to_string_lossy());
        }
        self.spice_skip_verify_check.set_active(spice.skip_cert_verify);
        self.spice_usb_check.set_active(spice.usb_redirection);
        self.spice_clipboard_check.set_active(spice.clipboard_enabled);

        // Map compression mode to dropdown index
        let compression_idx = match spice.image_compression {
            Some(SpiceImageCompression::Off) => 1,
            Some(SpiceImageCompression::Glz) => 2,
            Some(SpiceImageCompression::Lz) => 3,
            Some(SpiceImageCompression::Quic) => 4,
            _ => 0, // Auto or None
        };
        self.spice_compression_dropdown.set_selected(compression_idx);

        // Populate shared folders
        self.spice_shared_folders.borrow_mut().clear();
        while let Some(row) = self.spice_shared_folders_list.row_at_index(0) {
            self.spice_shared_folders_list.remove(&row);
        }
        for folder in &spice.shared_folders {
            self.spice_shared_folders.borrow_mut().push(folder.clone());
            Self::add_folder_row_to_list(
                &self.spice_shared_folders_list,
                &folder.local_path,
                &folder.share_name,
            );
        }
    }

    /// Runs the dialog and calls the callback with the result
    pub fn run<F: Fn(Option<Connection>) + 'static>(&self, cb: F) {
        // Store callback - the save button handler was connected in the constructor
        // and will invoke this callback when clicked
        *self.on_save.borrow_mut() = Some(Box::new(cb));

        self.window.present();
    }

    /// Returns a reference to the underlying window
    #[must_use]
    pub const fn window(&self) -> &Window {
        &self.window
    }

    /// Sets whether `KeePass` integration is enabled
    ///
    /// This controls the sensitivity of the "Save to `KeePass`" button.
    /// When `KeePass` is not enabled, the button is disabled.
    pub fn set_keepass_enabled(&self, enabled: bool) {
        self.save_to_keepass_button.set_sensitive(enabled);
    }

    /// Sets up the callback for the "Save to `KeePass`" button
    ///
    /// The callback receives the connection name, host, and password to save.
    pub fn connect_save_to_keepass<F: Fn(&str, &str, &str) + 'static>(&self, callback: F) {
        let name_entry = self.name_entry.clone();
        let host_entry = self.host_entry.clone();
        let password_entry = self.password_entry.clone();
        let window = self.window.clone();

        self.save_to_keepass_button.connect_clicked(move |_| {
            let name = name_entry.text();
            let host = host_entry.text();
            let password = password_entry.text();

            if password.is_empty() {
                let alert = gtk4::AlertDialog::builder()
                    .message("No Password")
                    .detail("Please enter a password to save to KeePass.")
                    .modal(true)
                    .build();
                alert.show(Some(&window));
                return;
            }

            if name.trim().is_empty() && host.trim().is_empty() {
                let alert = gtk4::AlertDialog::builder()
                    .message("Missing Information")
                    .detail("Please enter a connection name or host before saving to KeePass.")
                    .modal(true)
                    .build();
                alert.show(Some(&window));
                return;
            }

            callback(&name, &host, &password);
        });
    }

    /// Returns the password entry widget for external access
    #[must_use]
    pub const fn password_entry(&self) -> &Entry {
        &self.password_entry
    }
}

/// Helper struct for validation and building in the response callback
struct ConnectionDialogData<'a> {
    name_entry: &'a Entry,
    host_entry: &'a Entry,
    port_spin: &'a SpinButton,
    username_entry: &'a Entry,
    tags_entry: &'a Entry,
    protocol_dropdown: &'a DropDown,
    password_source_dropdown: &'a DropDown,
    ssh_auth_dropdown: &'a DropDown,
    ssh_key_entry: &'a Entry,
    ssh_proxy_entry: &'a Entry,
    ssh_control_master: &'a CheckButton,
    ssh_startup_entry: &'a Entry,
    ssh_options_entry: &'a Entry,
    rdp_width_spin: &'a SpinButton,
    rdp_height_spin: &'a SpinButton,
    rdp_color_dropdown: &'a DropDown,
    rdp_audio_check: &'a CheckButton,
    rdp_gateway_entry: &'a Entry,
    rdp_shared_folders: &'a Rc<RefCell<Vec<SharedFolder>>>,
    rdp_custom_args_entry: &'a Entry,
    vnc_encoding_entry: &'a Entry,
    vnc_compression_spin: &'a SpinButton,
    vnc_quality_spin: &'a SpinButton,
    vnc_view_only_check: &'a CheckButton,
    vnc_scaling_check: &'a CheckButton,
    vnc_clipboard_check: &'a CheckButton,
    spice_tls_check: &'a CheckButton,
    spice_ca_cert_entry: &'a Entry,
    spice_skip_verify_check: &'a CheckButton,
    spice_usb_check: &'a CheckButton,
    spice_clipboard_check: &'a CheckButton,
    spice_compression_dropdown: &'a DropDown,
    spice_shared_folders: &'a Rc<RefCell<Vec<SharedFolder>>>,
    editing_id: &'a Rc<RefCell<Option<Uuid>>>,
}

impl ConnectionDialogData<'_> {
    fn validate(&self) -> Result<(), String> {
        let name = self.name_entry.text();
        if name.trim().is_empty() {
            return Err("Connection name is required".to_string());
        }

        let host = self.host_entry.text();
        if host.trim().is_empty() {
            return Err("Host is required".to_string());
        }

        let host_str = host.trim();
        if host_str.contains(' ') {
            return Err("Host cannot contain spaces".to_string());
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let port = self.port_spin.value() as u16;
        if port == 0 {
            return Err("Port must be greater than 0".to_string());
        }

        // Protocol-specific validation using dropdown indices
        let protocol_idx = self.protocol_dropdown.selected();
        if protocol_idx == 0 {
            // SSH
            let auth_idx = self.ssh_auth_dropdown.selected();
            if auth_idx == 1 {
                // Public Key
                let key_path = self.ssh_key_entry.text();
                if key_path.trim().is_empty() {
                    return Err(
                        "SSH key path is required for public key authentication".to_string(),
                    );
                }
            }
        }
        // RDP (1) and VNC (2) use native embedding, no client validation needed

        Ok(())
    }

    fn build_connection(&self) -> Option<Connection> {
        let name = self.name_entry.text().trim().to_string();
        let host = self.host_entry.text().trim().to_string();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let port = self.port_spin.value() as u16;

        let protocol_config = self.build_protocol_config()?;

        let mut conn = Connection::new(name, host, port, protocol_config);

        let username = self.username_entry.text();
        if !username.trim().is_empty() {
            conn.username = Some(username.trim().to_string());
        }

        let tags_text = self.tags_entry.text();
        if !tags_text.trim().is_empty() {
            conn.tags = tags_text
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Password source - map dropdown index to enum
        // Dropdown order: Prompt(0), Stored(1), KeePass(2), Keyring(3), None(4)
        conn.password_source = match self.password_source_dropdown.selected() {
            1 => PasswordSource::Stored,
            2 => PasswordSource::KeePass,
            3 => PasswordSource::Keyring,
            4 => PasswordSource::None,
            _ => PasswordSource::Prompt, // 0 and any other value default to Prompt
        };

        if let Some(id) = *self.editing_id.borrow() {
            conn.id = id;
        }

        Some(conn)
    }

    fn build_protocol_config(&self) -> Option<ProtocolConfig> {
        let protocol_idx = self.protocol_dropdown.selected();

        match protocol_idx {
            0 => Some(ProtocolConfig::Ssh(self.build_ssh_config())),
            1 => Some(ProtocolConfig::Rdp(self.build_rdp_config())),
            2 => Some(ProtocolConfig::Vnc(self.build_vnc_config())),
            3 => Some(ProtocolConfig::Spice(self.build_spice_config())),
            _ => None,
        }
    }

    fn build_ssh_config(&self) -> SshConfig {
        let auth_method = match self.ssh_auth_dropdown.selected() {
            1 => SshAuthMethod::PublicKey,
            2 => SshAuthMethod::KeyboardInteractive,
            3 => SshAuthMethod::Agent,
            _ => SshAuthMethod::Password, // 0 and any other value default to Password
        };

        let key_path = {
            let text = self.ssh_key_entry.text();
            if text.trim().is_empty() {
                None
            } else {
                Some(PathBuf::from(text.trim().to_string()))
            }
        };

        let proxy_jump = {
            let text = self.ssh_proxy_entry.text();
            if text.trim().is_empty() {
                None
            } else {
                Some(text.trim().to_string())
            }
        };

        let startup_command = {
            let text = self.ssh_startup_entry.text();
            if text.trim().is_empty() {
                None
            } else {
                Some(text.trim().to_string())
            }
        };

        let custom_options = Self::parse_custom_options(&self.ssh_options_entry.text());

        SshConfig {
            auth_method,
            key_path,
            proxy_jump,
            use_control_master: self.ssh_control_master.is_active(),
            custom_options,
            startup_command,
        }
    }

    fn build_rdp_config(&self) -> RdpConfig {
        // Client selection removed - native embedding will be used
        // The dropdown is kept for UI consistency but ignored

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let resolution = Some(Resolution::new(
            self.rdp_width_spin.value() as u32,
            self.rdp_height_spin.value() as u32,
        ));

        // Map dropdown index to color depth: 0->32, 1->24, 2->16, 3->15, 4->8
        let color_depth = Some(match self.rdp_color_dropdown.selected() {
            1 => 24,
            2 => 16,
            3 => 15,
            4 => 8,
            _ => 32, // 0 and any other value default to 32
        });

        let gateway = {
            let text = self.rdp_gateway_entry.text();
            if text.trim().is_empty() {
                None
            } else {
                Some(rustconn_core::models::RdpGateway {
                    hostname: text.trim().to_string(),
                    port: 443,
                    username: None,
                })
            }
        };

        let custom_args = Self::parse_args(&self.rdp_custom_args_entry.text());

        let shared_folders = self.rdp_shared_folders.borrow().clone();

        RdpConfig {
            resolution,
            color_depth,
            audio_redirect: self.rdp_audio_check.is_active(),
            gateway,
            shared_folders,
            custom_args,
        }
    }

    fn build_vnc_config(&self) -> VncConfig {
        // Native embedding - no external client needed

        let encoding = {
            let text = self.vnc_encoding_entry.text();
            if text.trim().is_empty() {
                None
            } else {
                Some(text.trim().to_string())
            }
        };

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let compression = Some(self.vnc_compression_spin.value() as u8);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let quality = Some(self.vnc_quality_spin.value() as u8);

        // Note: view_only, scaling, and clipboard are handled by the session widget
        // These fields are stored in VncConfig for future use if needed
        let _ = self.vnc_view_only_check.is_active();
        let _ = self.vnc_scaling_check.is_active();
        let _ = self.vnc_clipboard_check.is_active();

        VncConfig {
            encoding,
            compression,
            quality,
            custom_args: Vec::new(), // No longer used with native embedding
        }
    }

    fn build_spice_config(&self) -> SpiceConfig {
        let ca_cert_path = {
            let text = self.spice_ca_cert_entry.text();
            if text.trim().is_empty() {
                None
            } else {
                Some(PathBuf::from(text.trim().to_string()))
            }
        };

        // Map dropdown index to compression mode: 0->Auto, 1->Off, 2->Glz, 3->Lz, 4->Quic
        let image_compression = match self.spice_compression_dropdown.selected() {
            1 => Some(SpiceImageCompression::Off),
            2 => Some(SpiceImageCompression::Glz),
            3 => Some(SpiceImageCompression::Lz),
            4 => Some(SpiceImageCompression::Quic),
            _ => Some(SpiceImageCompression::Auto), // 0 and any other value default to Auto
        };

        SpiceConfig {
            tls_enabled: self.spice_tls_check.is_active(),
            ca_cert_path,
            skip_cert_verify: self.spice_skip_verify_check.is_active(),
            usb_redirection: self.spice_usb_check.is_active(),
            shared_folders: self.spice_shared_folders.borrow().clone(),
            clipboard_enabled: self.spice_clipboard_check.is_active(),
            image_compression,
        }
    }

    fn parse_custom_options(text: &str) -> HashMap<String, String> {
        let mut options = HashMap::new();
        if text.trim().is_empty() {
            return options;
        }
        for part in text.split(',') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                if !key.is_empty() {
                    options.insert(key, value);
                }
            }
        }
        options
    }

    fn parse_args(text: &str) -> Vec<String> {
        if text.trim().is_empty() {
            return Vec::new();
        }
        text.split_whitespace().map(std::string::ToString::to_string).collect()
    }
}
