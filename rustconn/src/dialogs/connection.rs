//! Connection dialog for creating and editing connections
//!
//! Provides a GTK4 dialog with protocol-specific fields, input validation,
//! and portal integration for file selection (SSH keys).
//!
//! Updated for GTK 4.10+ compatibility using DropDown instead of ComboBoxText
//! and Window instead of Dialog.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, FileDialog, Grid, HeaderBar, Label,
    Notebook, Orientation, ScrolledWindow, SpinButton, Stack, StringList, Window,
};
use rustconn_core::models::{
    Connection, ProtocolConfig, RdpClient, RdpConfig, Resolution, SshAuthMethod, SshConfig,
    VncClient, VncConfig,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use uuid::Uuid;

/// Connection dialog for creating/editing connections
pub struct ConnectionDialog {
    window: Window,
    // Basic fields
    name_entry: Entry,
    host_entry: Entry,
    port_spin: SpinButton,
    username_entry: Entry,
    tags_entry: Entry,
    protocol_dropdown: DropDown,
    protocol_stack: Stack,
    // SSH fields
    ssh_auth_dropdown: DropDown,
    ssh_key_entry: Entry,
    ssh_key_button: Button,
    ssh_proxy_entry: Entry,
    ssh_control_master: CheckButton,
    ssh_startup_entry: Entry,
    ssh_options_entry: Entry,
    // RDP fields
    rdp_client_dropdown: DropDown,
    rdp_custom_client_entry: Entry,
    rdp_width_spin: SpinButton,
    rdp_height_spin: SpinButton,
    rdp_color_dropdown: DropDown,
    rdp_audio_check: CheckButton,
    rdp_gateway_entry: Entry,
    rdp_custom_args_entry: Entry,
    // VNC fields
    vnc_client_dropdown: DropDown,
    vnc_custom_client_entry: Entry,
    vnc_encoding_entry: Entry,
    vnc_compression_spin: SpinButton,
    vnc_quality_spin: SpinButton,
    vnc_custom_args_entry: Entry,
    // State
    editing_id: Rc<RefCell<Option<Uuid>>>,
    // Callback
    on_save: Rc<RefCell<Option<Box<dyn Fn(Option<Connection>)>>>>,
}


impl ConnectionDialog {
    /// Creates a new connection dialog
    #[must_use]
    pub fn new(parent: Option<&Window>) -> Self {
        // Create window instead of deprecated Dialog
        let window = Window::builder()
            .title("New Connection")
            .modal(true)
            .default_width(550)
            .default_height(500)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        // Create header bar with Cancel/Save buttons
        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let save_btn = Button::builder().label("Save").css_classes(["suggested-action"]).build();
        header.pack_start(&cancel_btn);
        header.pack_end(&save_btn);
        window.set_titlebar(Some(&header));

        // Create main content area
        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Create notebook for tabs
        let notebook = Notebook::new();
        content.append(&notebook);
        window.set_child(Some(&content));

        // === Basic Tab ===
        let (basic_grid, name_entry, host_entry, port_spin, username_entry, tags_entry, protocol_dropdown) =
            Self::create_basic_tab();
        notebook.append_page(&basic_grid, Some(&Label::new(Some("Basic"))));

        // === Protocol-specific Tab ===
        let protocol_stack = Stack::new();
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&protocol_stack)
            .build();
        notebook.append_page(&scrolled, Some(&Label::new(Some("Protocol"))));

        // SSH options
        let (ssh_box, ssh_auth_dropdown, ssh_key_entry, ssh_key_button, ssh_proxy_entry,
             ssh_control_master, ssh_startup_entry, ssh_options_entry) = Self::create_ssh_options();
        protocol_stack.add_named(&ssh_box, Some("ssh"));

        // RDP options
        let (rdp_box, rdp_client_dropdown, rdp_custom_client_entry, rdp_width_spin, rdp_height_spin,
             rdp_color_dropdown, rdp_audio_check, rdp_gateway_entry, rdp_custom_args_entry) =
            Self::create_rdp_options();
        protocol_stack.add_named(&rdp_box, Some("rdp"));

        // VNC options
        let (vnc_box, vnc_client_dropdown, vnc_custom_client_entry, vnc_encoding_entry,
             vnc_compression_spin, vnc_quality_spin, vnc_custom_args_entry) = Self::create_vnc_options();
        protocol_stack.add_named(&vnc_box, Some("vnc"));

        // Set initial protocol view
        protocol_stack.set_visible_child_name("ssh");

        // Connect protocol dropdown to stack
        let stack_clone = protocol_stack.clone();
        let port_clone = port_spin.clone();
        protocol_dropdown.connect_selected_notify(move |dropdown| {
            let protocols = ["ssh", "rdp", "vnc"];
            let selected = dropdown.selected() as usize;
            if selected < protocols.len() {
                let protocol_id = protocols[selected];
                stack_clone.set_visible_child_name(protocol_id);
                // Update default port
                let default_port = match protocol_id {
                    "ssh" => 22.0,
                    "rdp" => 3389.0,
                    "vnc" => 5900.0,
                    _ => 22.0,
                };
                if port_clone.value() == 22.0 || port_clone.value() == 3389.0 || port_clone.value() == 5900.0 {
                    port_clone.set_value(default_port);
                }
            }
        });

        let on_save: Rc<RefCell<Option<Box<dyn Fn(Option<Connection>)>>>> = Rc::new(RefCell::new(None));

        // Connect cancel button
        let window_clone = window.clone();
        let on_save_clone = on_save.clone();
        cancel_btn.connect_clicked(move |_| {
            if let Some(ref cb) = *on_save_clone.borrow() {
                cb(None);
            }
            window_clone.close();
        });

        Self {
            window,
            name_entry,
            host_entry,
            port_spin,
            username_entry,
            tags_entry,
            protocol_dropdown,
            protocol_stack,
            ssh_auth_dropdown,
            ssh_key_entry,
            ssh_key_button,
            ssh_proxy_entry,
            ssh_control_master,
            ssh_startup_entry,
            ssh_options_entry,
            rdp_client_dropdown,
            rdp_custom_client_entry,
            rdp_width_spin,
            rdp_height_spin,
            rdp_color_dropdown,
            rdp_audio_check,
            rdp_gateway_entry,
            rdp_custom_args_entry,
            vnc_client_dropdown,
            vnc_custom_client_entry,
            vnc_encoding_entry,
            vnc_compression_spin,
            vnc_quality_spin,
            vnc_custom_args_entry,
            editing_id: Rc::new(RefCell::new(None)),
            on_save,
        }
    }


    fn create_basic_tab() -> (Grid, Entry, Entry, SpinButton, Entry, Entry, DropDown) {
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
        let name_label = Label::builder().label("Name:").halign(gtk4::Align::End).build();
        let name_entry = Entry::builder().hexpand(true).placeholder_text("Connection name").build();
        grid.attach(&name_label, 0, row, 1, 1);
        grid.attach(&name_entry, 1, row, 2, 1);
        row += 1;

        // Protocol - using DropDown instead of ComboBoxText
        let protocol_label = Label::builder().label("Protocol:").halign(gtk4::Align::End).build();
        let protocol_list = StringList::new(&["SSH", "RDP", "VNC"]);
        let protocol_dropdown = DropDown::new(Some(protocol_list), gtk4::Expression::NONE);
        protocol_dropdown.set_selected(0); // SSH by default
        grid.attach(&protocol_label, 0, row, 1, 1);
        grid.attach(&protocol_dropdown, 1, row, 2, 1);
        row += 1;

        // Host
        let host_label = Label::builder().label("Host:").halign(gtk4::Align::End).build();
        let host_entry = Entry::builder().hexpand(true).placeholder_text("hostname or IP").build();
        grid.attach(&host_label, 0, row, 1, 1);
        grid.attach(&host_entry, 1, row, 2, 1);
        row += 1;

        // Port
        let port_label = Label::builder().label("Port:").halign(gtk4::Align::End).build();
        let port_adj = gtk4::Adjustment::new(22.0, 1.0, 65535.0, 1.0, 10.0, 0.0);
        let port_spin = SpinButton::builder().adjustment(&port_adj).climb_rate(1.0).digits(0).build();
        grid.attach(&port_label, 0, row, 1, 1);
        grid.attach(&port_spin, 1, row, 1, 1);
        row += 1;

        // Username - show current user as placeholder hint
        let username_label = Label::builder().label("Username:").halign(gtk4::Align::End).build();
        let current_user = std::env::var("USER").unwrap_or_default();
        let placeholder = if current_user.is_empty() {
            "(optional)".to_string()
        } else {
            format!("(default: {})", current_user)
        };
        let username_entry = Entry::builder().hexpand(true).placeholder_text(&placeholder).build();
        grid.attach(&username_label, 0, row, 1, 1);
        grid.attach(&username_entry, 1, row, 2, 1);
        row += 1;

        // Tags
        let tags_label = Label::builder().label("Tags:").halign(gtk4::Align::End).build();
        let tags_entry = Entry::builder().hexpand(true).placeholder_text("tag1, tag2, ...").build();
        grid.attach(&tags_label, 0, row, 1, 1);
        grid.attach(&tags_entry, 1, row, 2, 1);

        (grid, name_entry, host_entry, port_spin, username_entry, tags_entry, protocol_dropdown)
    }


    fn create_ssh_options() -> (GtkBox, DropDown, Entry, Button, Entry, CheckButton, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        // Auth method - using DropDown
        let auth_label = Label::builder().label("Auth Method:").halign(gtk4::Align::End).build();
        let auth_list = StringList::new(&["Password", "Public Key", "Keyboard Interactive", "SSH Agent"]);
        let auth_dropdown = DropDown::new(Some(auth_list), gtk4::Expression::NONE);
        auth_dropdown.set_selected(0); // Password by default
        grid.attach(&auth_label, 0, row, 1, 1);
        grid.attach(&auth_dropdown, 1, row, 2, 1);
        row += 1;

        // Key path with file chooser button (uses portal on Wayland)
        let key_label = Label::builder().label("Key File:").halign(gtk4::Align::End).build();
        let key_hbox = GtkBox::new(Orientation::Horizontal, 4);
        let key_entry = Entry::builder().hexpand(true).placeholder_text("Path to SSH key").build();
        let key_button = Button::builder().label("Browse...").build();
        key_hbox.append(&key_entry);
        key_hbox.append(&key_button);
        grid.attach(&key_label, 0, row, 1, 1);
        grid.attach(&key_hbox, 1, row, 2, 1);
        row += 1;

        // ProxyJump
        let proxy_label = Label::builder().label("ProxyJump:").halign(gtk4::Align::End).build();
        let proxy_entry = Entry::builder().hexpand(true).placeholder_text("user@jumphost").build();
        grid.attach(&proxy_label, 0, row, 1, 1);
        grid.attach(&proxy_entry, 1, row, 2, 1);
        row += 1;

        // ControlMaster
        let control_master = CheckButton::builder().label("Enable ControlMaster (connection multiplexing)").build();
        grid.attach(&control_master, 1, row, 2, 1);
        row += 1;

        // Startup command
        let startup_label = Label::builder().label("Startup Command:").halign(gtk4::Align::End).build();
        let startup_entry = Entry::builder().hexpand(true).placeholder_text("Command to run on connect").build();
        grid.attach(&startup_label, 0, row, 1, 1);
        grid.attach(&startup_entry, 1, row, 2, 1);
        row += 1;

        // Custom options
        let options_label = Label::builder().label("Custom Options:").halign(gtk4::Align::End).build();
        let options_entry = Entry::builder().hexpand(true).placeholder_text("Key=Value, Key2=Value2").build();
        grid.attach(&options_label, 0, row, 1, 1);
        grid.attach(&options_entry, 1, row, 2, 1);

        (vbox, auth_dropdown, key_entry, key_button, proxy_entry, control_master, startup_entry, options_entry)
    }


    fn create_rdp_options() -> (GtkBox, DropDown, Entry, SpinButton, SpinButton, DropDown, CheckButton, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        // Client - using DropDown
        let client_label = Label::builder().label("RDP Client:").halign(gtk4::Align::End).build();
        let client_list = StringList::new(&["FreeRDP (xfreerdp)", "Custom"]);
        let client_dropdown = DropDown::new(Some(client_list), gtk4::Expression::NONE);
        client_dropdown.set_selected(0); // FreeRDP by default
        grid.attach(&client_label, 0, row, 1, 1);
        grid.attach(&client_dropdown, 1, row, 2, 1);
        row += 1;

        // Custom client path
        let custom_label = Label::builder().label("Custom Client:").halign(gtk4::Align::End).build();
        let custom_entry = Entry::builder().hexpand(true).placeholder_text("Path to RDP client").sensitive(false).build();
        grid.attach(&custom_label, 0, row, 1, 1);
        grid.attach(&custom_entry, 1, row, 2, 1);
        row += 1;

        // Resolution
        let res_label = Label::builder().label("Resolution:").halign(gtk4::Align::End).build();
        let res_hbox = GtkBox::new(Orientation::Horizontal, 4);
        let width_adj = gtk4::Adjustment::new(1920.0, 640.0, 7680.0, 1.0, 100.0, 0.0);
        let width_spin = SpinButton::builder().adjustment(&width_adj).climb_rate(1.0).digits(0).build();
        let x_label = Label::new(Some("x"));
        let height_adj = gtk4::Adjustment::new(1080.0, 480.0, 4320.0, 1.0, 100.0, 0.0);
        let height_spin = SpinButton::builder().adjustment(&height_adj).climb_rate(1.0).digits(0).build();
        res_hbox.append(&width_spin);
        res_hbox.append(&x_label);
        res_hbox.append(&height_spin);
        grid.attach(&res_label, 0, row, 1, 1);
        grid.attach(&res_hbox, 1, row, 2, 1);
        row += 1;

        // Color depth - using DropDown
        let color_label = Label::builder().label("Color Depth:").halign(gtk4::Align::End).build();
        let color_list = StringList::new(&["32-bit (True Color)", "24-bit", "16-bit (High Color)", "15-bit", "8-bit"]);
        let color_dropdown = DropDown::new(Some(color_list), gtk4::Expression::NONE);
        color_dropdown.set_selected(0); // 32-bit by default
        grid.attach(&color_label, 0, row, 1, 1);
        grid.attach(&color_dropdown, 1, row, 2, 1);
        row += 1;

        // Audio redirect
        let audio_check = CheckButton::builder().label("Enable audio redirection").build();
        grid.attach(&audio_check, 1, row, 2, 1);
        row += 1;

        // Gateway
        let gateway_label = Label::builder().label("RDP Gateway:").halign(gtk4::Align::End).build();
        let gateway_entry = Entry::builder().hexpand(true).placeholder_text("gateway.example.com").build();
        grid.attach(&gateway_label, 0, row, 1, 1);
        grid.attach(&gateway_entry, 1, row, 2, 1);
        row += 1;

        // Custom args
        let args_label = Label::builder().label("Custom Args:").halign(gtk4::Align::End).build();
        let args_entry = Entry::builder().hexpand(true).placeholder_text("Additional command-line arguments").build();
        grid.attach(&args_label, 0, row, 1, 1);
        grid.attach(&args_entry, 1, row, 2, 1);

        // Connect client dropdown to custom entry sensitivity
        let custom_clone = custom_entry.clone();
        client_dropdown.connect_selected_notify(move |dropdown| {
            custom_clone.set_sensitive(dropdown.selected() == 1); // 1 = Custom
        });

        (vbox, client_dropdown, custom_entry, width_spin, height_spin, color_dropdown, audio_check, gateway_entry, args_entry)
    }


    fn create_vnc_options() -> (GtkBox, DropDown, Entry, Entry, SpinButton, SpinButton, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        // Client - using DropDown
        let client_label = Label::builder().label("VNC Client:").halign(gtk4::Align::End).build();
        let client_list = StringList::new(&["TightVNC", "TigerVNC", "Custom"]);
        let client_dropdown = DropDown::new(Some(client_list), gtk4::Expression::NONE);
        client_dropdown.set_selected(0); // TightVNC by default
        grid.attach(&client_label, 0, row, 1, 1);
        grid.attach(&client_dropdown, 1, row, 2, 1);
        row += 1;

        // Custom client path
        let custom_label = Label::builder().label("Custom Client:").halign(gtk4::Align::End).build();
        let custom_entry = Entry::builder().hexpand(true).placeholder_text("Path to VNC client").sensitive(false).build();
        grid.attach(&custom_label, 0, row, 1, 1);
        grid.attach(&custom_entry, 1, row, 2, 1);
        row += 1;

        // Encoding
        let encoding_label = Label::builder().label("Encoding:").halign(gtk4::Align::End).build();
        let encoding_entry = Entry::builder().hexpand(true).placeholder_text("tight, zrle, hextile").build();
        grid.attach(&encoding_label, 0, row, 1, 1);
        grid.attach(&encoding_entry, 1, row, 2, 1);
        row += 1;

        // Compression
        let compression_label = Label::builder().label("Compression:").halign(gtk4::Align::End).build();
        let compression_adj = gtk4::Adjustment::new(6.0, 0.0, 9.0, 1.0, 1.0, 0.0);
        let compression_spin = SpinButton::builder().adjustment(&compression_adj).climb_rate(1.0).digits(0).build();
        grid.attach(&compression_label, 0, row, 1, 1);
        grid.attach(&compression_spin, 1, row, 1, 1);
        row += 1;

        // Quality
        let quality_label = Label::builder().label("Quality:").halign(gtk4::Align::End).build();
        let quality_adj = gtk4::Adjustment::new(6.0, 0.0, 9.0, 1.0, 1.0, 0.0);
        let quality_spin = SpinButton::builder().adjustment(&quality_adj).climb_rate(1.0).digits(0).build();
        grid.attach(&quality_label, 0, row, 1, 1);
        grid.attach(&quality_spin, 1, row, 1, 1);
        row += 1;

        // Custom args
        let args_label = Label::builder().label("Custom Args:").halign(gtk4::Align::End).build();
        let args_entry = Entry::builder().hexpand(true).placeholder_text("Additional command-line arguments").build();
        grid.attach(&args_label, 0, row, 1, 1);
        grid.attach(&args_entry, 1, row, 2, 1);

        // Connect client dropdown to custom entry sensitivity
        let custom_clone = custom_entry.clone();
        client_dropdown.connect_selected_notify(move |dropdown| {
            custom_clone.set_sensitive(dropdown.selected() == 2); // 2 = Custom
        });

        (vbox, client_dropdown, custom_entry, encoding_entry, compression_spin, quality_spin, args_entry)
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
            file_dialog.open(parent.as_ref(), gtk4::gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        entry.set_text(&path.to_string_lossy());
                    }
                }
            });
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
            let opts: Vec<String> = ssh.custom_options.iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            self.ssh_options_entry.set_text(&opts.join(", "));
        }
    }

    fn set_rdp_config(&self, rdp: &RdpConfig) {
        match &rdp.client {
            RdpClient::FreeRdp => {
                self.rdp_client_dropdown.set_selected(0); // FreeRDP
            }
            RdpClient::Custom(path) => {
                self.rdp_client_dropdown.set_selected(1); // Custom
                self.rdp_custom_client_entry.set_text(&path.to_string_lossy());
                self.rdp_custom_client_entry.set_sensitive(true);
            }
        }

        if let Some(ref res) = rdp.resolution {
            self.rdp_width_spin.set_value(f64::from(res.width));
            self.rdp_height_spin.set_value(f64::from(res.height));
        }
        if let Some(depth) = rdp.color_depth {
            // Map color depth to dropdown index: 32->0, 24->1, 16->2, 15->3, 8->4
            let idx = match depth {
                32 => 0,
                24 => 1,
                16 => 2,
                15 => 3,
                8 => 4,
                _ => 0,
            };
            self.rdp_color_dropdown.set_selected(idx);
        }
        self.rdp_audio_check.set_active(rdp.audio_redirect);
        if let Some(ref gw) = rdp.gateway {
            self.rdp_gateway_entry.set_text(&gw.hostname);
        }
        if !rdp.custom_args.is_empty() {
            self.rdp_custom_args_entry.set_text(&rdp.custom_args.join(" "));
        }
    }

    fn set_vnc_config(&self, vnc: &VncConfig) {
        match &vnc.client {
            VncClient::TightVnc => {
                self.vnc_client_dropdown.set_selected(0); // TightVNC
            }
            VncClient::TigerVnc => {
                self.vnc_client_dropdown.set_selected(1); // TigerVNC
            }
            VncClient::Custom(path) => {
                self.vnc_client_dropdown.set_selected(2); // Custom
                self.vnc_custom_client_entry.set_text(&path.to_string_lossy());
                self.vnc_custom_client_entry.set_sensitive(true);
            }
        }

        if let Some(ref enc) = vnc.encoding {
            self.vnc_encoding_entry.set_text(enc);
        }
        if let Some(comp) = vnc.compression {
            self.vnc_compression_spin.set_value(f64::from(comp));
        }
        if let Some(qual) = vnc.quality {
            self.vnc_quality_spin.set_value(f64::from(qual));
        }
        if !vnc.custom_args.is_empty() {
            self.vnc_custom_args_entry.set_text(&vnc.custom_args.join(" "));
        }
    }

    /// Runs the dialog and calls the callback with the result
    pub fn run<F: Fn(Option<Connection>) + 'static>(&self, cb: F) {
        // Store callback
        *self.on_save.borrow_mut() = Some(Box::new(cb));

        // Get the save button from header bar and connect it
        if let Some(titlebar) = self.window.titlebar() {
            if let Some(header) = titlebar.downcast_ref::<HeaderBar>() {
                // Find save button (it's at the end)
                if let Some(save_btn) = header.last_child() {
                    if let Some(btn) = save_btn.downcast_ref::<Button>() {
                        let window = self.window.clone();
                        let on_save = self.on_save.clone();
                        let name_entry = self.name_entry.clone();
                        let host_entry = self.host_entry.clone();
                        let port_spin = self.port_spin.clone();
                        let username_entry = self.username_entry.clone();
                        let tags_entry = self.tags_entry.clone();
                        let protocol_dropdown = self.protocol_dropdown.clone();
                        let ssh_auth_dropdown = self.ssh_auth_dropdown.clone();
                        let ssh_key_entry = self.ssh_key_entry.clone();
                        let ssh_proxy_entry = self.ssh_proxy_entry.clone();
                        let ssh_control_master = self.ssh_control_master.clone();
                        let ssh_startup_entry = self.ssh_startup_entry.clone();
                        let ssh_options_entry = self.ssh_options_entry.clone();
                        let rdp_client_dropdown = self.rdp_client_dropdown.clone();
                        let rdp_custom_client_entry = self.rdp_custom_client_entry.clone();
                        let rdp_width_spin = self.rdp_width_spin.clone();
                        let rdp_height_spin = self.rdp_height_spin.clone();
                        let rdp_color_dropdown = self.rdp_color_dropdown.clone();
                        let rdp_audio_check = self.rdp_audio_check.clone();
                        let rdp_gateway_entry = self.rdp_gateway_entry.clone();
                        let rdp_custom_args_entry = self.rdp_custom_args_entry.clone();
                        let vnc_client_dropdown = self.vnc_client_dropdown.clone();
                        let vnc_custom_client_entry = self.vnc_custom_client_entry.clone();
                        let vnc_encoding_entry = self.vnc_encoding_entry.clone();
                        let vnc_compression_spin = self.vnc_compression_spin.clone();
                        let vnc_quality_spin = self.vnc_quality_spin.clone();
                        let vnc_custom_args_entry = self.vnc_custom_args_entry.clone();
                        let editing_id = self.editing_id.clone();

                        btn.connect_clicked(move |_| {
                            let data = ConnectionDialogData {
                                name_entry: &name_entry,
                                host_entry: &host_entry,
                                port_spin: &port_spin,
                                username_entry: &username_entry,
                                tags_entry: &tags_entry,
                                protocol_dropdown: &protocol_dropdown,
                                ssh_auth_dropdown: &ssh_auth_dropdown,
                                ssh_key_entry: &ssh_key_entry,
                                ssh_proxy_entry: &ssh_proxy_entry,
                                ssh_control_master: &ssh_control_master,
                                ssh_startup_entry: &ssh_startup_entry,
                                ssh_options_entry: &ssh_options_entry,
                                rdp_client_dropdown: &rdp_client_dropdown,
                                rdp_custom_client_entry: &rdp_custom_client_entry,
                                rdp_width_spin: &rdp_width_spin,
                                rdp_height_spin: &rdp_height_spin,
                                rdp_color_dropdown: &rdp_color_dropdown,
                                rdp_audio_check: &rdp_audio_check,
                                rdp_gateway_entry: &rdp_gateway_entry,
                                rdp_custom_args_entry: &rdp_custom_args_entry,
                                vnc_client_dropdown: &vnc_client_dropdown,
                                vnc_custom_client_entry: &vnc_custom_client_entry,
                                vnc_encoding_entry: &vnc_encoding_entry,
                                vnc_compression_spin: &vnc_compression_spin,
                                vnc_quality_spin: &vnc_quality_spin,
                                vnc_custom_args_entry: &vnc_custom_args_entry,
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
                }
            }
        }

        self.window.present();
    }

    /// Returns a reference to the underlying window
    #[must_use]
    pub fn window(&self) -> &Window {
        &self.window
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
    ssh_auth_dropdown: &'a DropDown,
    ssh_key_entry: &'a Entry,
    ssh_proxy_entry: &'a Entry,
    ssh_control_master: &'a CheckButton,
    ssh_startup_entry: &'a Entry,
    ssh_options_entry: &'a Entry,
    rdp_client_dropdown: &'a DropDown,
    rdp_custom_client_entry: &'a Entry,
    rdp_width_spin: &'a SpinButton,
    rdp_height_spin: &'a SpinButton,
    rdp_color_dropdown: &'a DropDown,
    rdp_audio_check: &'a CheckButton,
    rdp_gateway_entry: &'a Entry,
    rdp_custom_args_entry: &'a Entry,
    vnc_client_dropdown: &'a DropDown,
    vnc_custom_client_entry: &'a Entry,
    vnc_encoding_entry: &'a Entry,
    vnc_compression_spin: &'a SpinButton,
    vnc_quality_spin: &'a SpinButton,
    vnc_custom_args_entry: &'a Entry,
    editing_id: &'a Rc<RefCell<Option<Uuid>>>,
}

impl<'a> ConnectionDialogData<'a> {
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

        let port = self.port_spin.value() as u16;
        if port == 0 {
            return Err("Port must be greater than 0".to_string());
        }

        // Protocol-specific validation using dropdown indices
        let protocol_idx = self.protocol_dropdown.selected();
        match protocol_idx {
            0 => { // SSH
                let auth_idx = self.ssh_auth_dropdown.selected();
                if auth_idx == 1 { // Public Key
                    let key_path = self.ssh_key_entry.text();
                    if key_path.trim().is_empty() {
                        return Err("SSH key path is required for public key authentication".to_string());
                    }
                }
            }
            1 => { // RDP
                let client_idx = self.rdp_client_dropdown.selected();
                if client_idx == 1 { // Custom
                    let client_path = self.rdp_custom_client_entry.text();
                    if client_path.trim().is_empty() {
                        return Err("Custom RDP client path is required".to_string());
                    }
                }
            }
            2 => { // VNC
                let client_idx = self.vnc_client_dropdown.selected();
                if client_idx == 2 { // Custom
                    let client_path = self.vnc_custom_client_entry.text();
                    if client_path.trim().is_empty() {
                        return Err("Custom VNC client path is required".to_string());
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }


    fn build_connection(&self) -> Option<Connection> {
        let name = self.name_entry.text().trim().to_string();
        let host = self.host_entry.text().trim().to_string();
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
            _ => None,
        }
    }

    fn build_ssh_config(&self) -> SshConfig {
        let auth_method = match self.ssh_auth_dropdown.selected() {
            0 => SshAuthMethod::Password,
            1 => SshAuthMethod::PublicKey,
            2 => SshAuthMethod::KeyboardInteractive,
            3 => SshAuthMethod::Agent,
            _ => SshAuthMethod::Password,
        };

        let key_path = {
            let text = self.ssh_key_entry.text();
            if text.trim().is_empty() { None } else { Some(PathBuf::from(text.trim().to_string())) }
        };

        let proxy_jump = {
            let text = self.ssh_proxy_entry.text();
            if text.trim().is_empty() { None } else { Some(text.trim().to_string()) }
        };

        let startup_command = {
            let text = self.ssh_startup_entry.text();
            if text.trim().is_empty() { None } else { Some(text.trim().to_string()) }
        };

        let custom_options = self.parse_custom_options(&self.ssh_options_entry.text());

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
        let client = match self.rdp_client_dropdown.selected() {
            1 => { // Custom
                let path = self.rdp_custom_client_entry.text();
                RdpClient::Custom(PathBuf::from(path.trim().to_string()))
            }
            _ => RdpClient::FreeRdp,
        };

        let resolution = Some(Resolution::new(
            self.rdp_width_spin.value() as u32,
            self.rdp_height_spin.value() as u32,
        ));

        // Map dropdown index to color depth: 0->32, 1->24, 2->16, 3->15, 4->8
        let color_depth = Some(match self.rdp_color_dropdown.selected() {
            0 => 32,
            1 => 24,
            2 => 16,
            3 => 15,
            4 => 8,
            _ => 32,
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

        let custom_args = self.parse_args(&self.rdp_custom_args_entry.text());

        RdpConfig {
            client,
            resolution,
            color_depth,
            audio_redirect: self.rdp_audio_check.is_active(),
            gateway,
            custom_args,
        }
    }

    fn build_vnc_config(&self) -> VncConfig {
        let client = match self.vnc_client_dropdown.selected() {
            1 => VncClient::TigerVnc,
            2 => { // Custom
                let path = self.vnc_custom_client_entry.text();
                VncClient::Custom(PathBuf::from(path.trim().to_string()))
            }
            _ => VncClient::TightVnc,
        };

        let encoding = {
            let text = self.vnc_encoding_entry.text();
            if text.trim().is_empty() { None } else { Some(text.trim().to_string()) }
        };

        let compression = Some(self.vnc_compression_spin.value() as u8);
        let quality = Some(self.vnc_quality_spin.value() as u8);

        let custom_args = self.parse_args(&self.vnc_custom_args_entry.text());

        VncConfig {
            client,
            encoding,
            compression,
            quality,
            custom_args,
        }
    }

    fn parse_custom_options(&self, text: &str) -> HashMap<String, String> {
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

    fn parse_args(&self, text: &str) -> Vec<String> {
        if text.trim().is_empty() {
            return Vec::new();
        }
        text.split_whitespace().map(|s| s.to_string()).collect()
    }
}
