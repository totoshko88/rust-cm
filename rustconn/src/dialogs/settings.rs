//! Settings dialog for application preferences
//!
//! Provides a GTK4 dialog for configuring terminal settings, logging options,
//! and secret storage preferences.
//!
//! Updated for GTK 4.10+ compatibility using `DropDown` instead of `ComboBoxText`
//! and Window instead of Dialog.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, FileDialog, FileFilter, Frame, Grid,
    HeaderBar, Label, Notebook, Orientation, PasswordEntry, ScrolledWindow, SpinButton,
    StringList, Switch, Window,
};
use rustconn_core::config::{
    AppSettings, LoggingSettings, SecretBackendType, SecretSettings, TerminalSettings, UiSettings,
};
use rustconn_core::secret::KeePassStatus;
use rustconn_core::{detect_rdp_client, detect_ssh_client, detect_vnc_client, ClientInfo};
use secrecy::SecretString;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// Settings dialog for application preferences
pub struct SettingsDialog {
    window: Window,
    // Terminal settings
    font_family_entry: Entry,
    font_size_spin: SpinButton,
    scrollback_spin: SpinButton,
    // Logging settings
    logging_enabled: CheckButton,
    log_dir_entry: Entry,
    retention_spin: SpinButton,
    // Secret settings
    secret_backend_dropdown: DropDown,
    enable_fallback: CheckButton,
    // KeePass settings
    kdbx_path_entry: Entry,
    kdbx_password_entry: PasswordEntry,
    kdbx_enabled_switch: Switch,
    kdbx_unlock_button: Button,
    kdbx_status_label: Label,
    keepassxc_status_container: GtkBox,
    // UI settings
    remember_geometry: CheckButton,
    // Current settings
    settings: Rc<RefCell<AppSettings>>,
    // KeePass unlock state
    kdbx_unlocked: Rc<RefCell<bool>>,
    // Callback
    on_save: super::SettingsCallback,
}

impl SettingsDialog {
    /// Creates a new settings dialog
    #[must_use]
    pub fn new(parent: Option<&Window>) -> Self {
        // Create window instead of deprecated Dialog
        let window = Window::builder()
            .title("Settings")
            .modal(true)
            .default_width(500)
            .resizable(true)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        // Create header bar with Cancel/Save buttons
        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let save_btn = Button::builder()
            .label("Save")
            .css_classes(["suggested-action"])
            .build();
        header.pack_start(&cancel_btn);
        header.pack_end(&save_btn);
        window.set_titlebar(Some(&header));

        // Create main content area
        let content = GtkBox::new(Orientation::Vertical, 0);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Create notebook for tabs - make it expand to fill available space
        let notebook = Notebook::new();
        notebook.set_vexpand(true);
        content.append(&notebook);
        window.set_child(Some(&content));

        // === Terminal Tab ===
        let (terminal_page, font_family_entry, font_size_spin, scrollback_spin) =
            Self::create_terminal_tab();
        notebook.append_page(&terminal_page, Some(&Label::new(Some("Terminal"))));

        // === Logging Tab ===
        let (logging_page, logging_enabled, log_dir_entry, retention_spin) =
            Self::create_logging_tab();
        notebook.append_page(&logging_page, Some(&Label::new(Some("Logging"))));

        // === Secrets Tab ===
        let (
            secrets_page,
            secret_backend_dropdown,
            enable_fallback,
            kdbx_path_entry,
            kdbx_password_entry,
            kdbx_enabled_switch,
            kdbx_unlock_button,
            kdbx_status_label,
            keepassxc_status_container,
        ) = Self::create_secrets_tab();
        notebook.append_page(&secrets_page, Some(&Label::new(Some("Secrets"))));

        // === UI Tab ===
        let (ui_page, remember_geometry) = Self::create_ui_tab();
        notebook.append_page(&ui_page, Some(&Label::new(Some("Interface"))));

        // === Clients Tab ===
        let clients_page = Self::create_clients_tab();
        notebook.append_page(&clients_page, Some(&Label::new(Some("Clients"))));

        let on_save: super::SettingsCallback = Rc::new(RefCell::new(None));

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
            font_family_entry,
            font_size_spin,
            scrollback_spin,
            logging_enabled,
            log_dir_entry,
            retention_spin,
            secret_backend_dropdown,
            enable_fallback,
            kdbx_path_entry,
            kdbx_password_entry,
            kdbx_enabled_switch,
            kdbx_unlock_button,
            kdbx_status_label,
            keepassxc_status_container,
            remember_geometry,
            settings: Rc::new(RefCell::new(AppSettings::default())),
            kdbx_unlocked: Rc::new(RefCell::new(false)),
            on_save,
        }
    }

    fn create_terminal_tab() -> (Frame, Entry, SpinButton, SpinButton) {
        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let mut row = 0;

        // Font family
        let font_label = Label::builder()
            .label("Font Family:")
            .halign(gtk4::Align::End)
            .build();
        let font_family_entry = Entry::builder().hexpand(true).text("Monospace").build();
        grid.attach(&font_label, 0, row, 1, 1);
        grid.attach(&font_family_entry, 1, row, 1, 1);
        row += 1;

        // Font size
        let size_label = Label::builder()
            .label("Font Size:")
            .halign(gtk4::Align::End)
            .build();
        let size_adj = gtk4::Adjustment::new(12.0, 6.0, 72.0, 1.0, 2.0, 0.0);
        let font_size_spin = SpinButton::builder()
            .adjustment(&size_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&size_label, 0, row, 1, 1);
        grid.attach(&font_size_spin, 1, row, 1, 1);
        row += 1;

        // Scrollback lines
        let scrollback_label = Label::builder()
            .label("Scrollback Lines:")
            .halign(gtk4::Align::End)
            .build();
        let scrollback_adj = gtk4::Adjustment::new(10000.0, 100.0, 1_000_000.0, 100.0, 1000.0, 0.0);
        let scrollback_spin = SpinButton::builder()
            .adjustment(&scrollback_adj)
            .climb_rate(100.0)
            .digits(0)
            .build();
        grid.attach(&scrollback_label, 0, row, 1, 1);
        grid.attach(&scrollback_spin, 1, row, 1, 1);

        let frame = Frame::builder()
            .label("Terminal Settings")
            .child(&grid)
            .margin_top(12)
            .valign(gtk4::Align::Start)
            .build();

        (frame, font_family_entry, font_size_spin, scrollback_spin)
    }

    fn create_logging_tab() -> (Frame, CheckButton, Entry, SpinButton) {
        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        // Enable logging checkbox
        let logging_enabled = CheckButton::builder()
            .label("Enable session logging")
            .build();
        vbox.append(&logging_enabled);

        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .margin_top(8)
            .build();

        let mut row = 0;

        // Log directory
        let dir_label = Label::builder()
            .label("Log Directory:")
            .halign(gtk4::Align::End)
            .build();
        let log_dir_entry = Entry::builder()
            .hexpand(true)
            .text("logs")
            .placeholder_text("Relative to config dir or absolute path")
            .build();
        grid.attach(&dir_label, 0, row, 1, 1);
        grid.attach(&log_dir_entry, 1, row, 1, 1);
        row += 1;

        // Retention days
        let retention_label = Label::builder()
            .label("Retention (days):")
            .halign(gtk4::Align::End)
            .build();
        let retention_adj = gtk4::Adjustment::new(30.0, 1.0, 365.0, 1.0, 7.0, 0.0);
        let retention_spin = SpinButton::builder()
            .adjustment(&retention_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&retention_label, 0, row, 1, 1);
        grid.attach(&retention_spin, 1, row, 1, 1);

        vbox.append(&grid);

        // Connect checkbox to enable/disable other controls
        let dir_entry_clone = log_dir_entry.clone();
        let retention_clone = retention_spin.clone();
        logging_enabled.connect_toggled(move |check| {
            let enabled = check.is_active();
            dir_entry_clone.set_sensitive(enabled);
            retention_clone.set_sensitive(enabled);
        });

        // Initial state
        log_dir_entry.set_sensitive(false);
        retention_spin.set_sensitive(false);

        let frame = Frame::builder()
            .label("Session Logging")
            .child(&vbox)
            .margin_top(12)
            .valign(gtk4::Align::Start)
            .build();

        (frame, logging_enabled, log_dir_entry, retention_spin)
    }

    #[allow(clippy::type_complexity)]
    fn create_secrets_tab() -> (
        ScrolledWindow,
        DropDown,
        CheckButton,
        Entry,
        PasswordEntry,
        Switch,
        Button,
        Label,
        GtkBox,
    ) {
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();

        let main_vbox = GtkBox::new(Orientation::Vertical, 12);
        main_vbox.set_margin_top(12);
        main_vbox.set_margin_bottom(12);
        main_vbox.set_margin_start(12);
        main_vbox.set_margin_end(12);
        main_vbox.set_valign(gtk4::Align::Start);

        // KeePass Database Section
        let (kdbx_frame, kdbx_path_entry, kdbx_password_entry, kdbx_enabled_switch, kdbx_unlock_button, kdbx_status_label, browse_button) =
            Self::create_kdbx_section();
        main_vbox.append(&kdbx_frame);

        // KeePassXC Status Section
        let keepassxc_status_container = Self::create_keepassxc_status_section();
        main_vbox.append(&keepassxc_status_container.parent().unwrap().downcast::<Frame>().unwrap());

        // Backend Selection Section
        let (backend_frame, secret_backend_dropdown, enable_fallback) = Self::create_backend_section();
        main_vbox.append(&backend_frame);

        scrolled.set_child(Some(&main_vbox));

        // Connect browse button
        let path_entry_clone = kdbx_path_entry.clone();
        browse_button.connect_clicked(move |btn| {
            Self::show_kdbx_file_dialog(btn, &path_entry_clone);
        });

        // Connect enable switch to control sensitivity
        Self::connect_kdbx_enable_switch(
            &kdbx_enabled_switch,
            &kdbx_path_entry,
            &kdbx_password_entry,
            &kdbx_unlock_button,
            &browse_button,
        );

        (
            scrolled,
            secret_backend_dropdown,
            enable_fallback,
            kdbx_path_entry,
            kdbx_password_entry,
            kdbx_enabled_switch,
            kdbx_unlock_button,
            kdbx_status_label,
            keepassxc_status_container,
        )
    }

    /// Creates the KDBX database section
    #[allow(clippy::type_complexity)]
    fn create_kdbx_section() -> (Frame, Entry, PasswordEntry, Switch, Button, Label, Button) {
        let kdbx_frame = Frame::builder().label("KeePass Database").build();
        let kdbx_vbox = GtkBox::new(Orientation::Vertical, 8);
        kdbx_vbox.set_margin_top(8);
        kdbx_vbox.set_margin_bottom(8);
        kdbx_vbox.set_margin_start(8);
        kdbx_vbox.set_margin_end(8);

        // Enable switch row
        let enable_row = GtkBox::new(Orientation::Horizontal, 12);
        let enable_label = Label::builder()
            .label("Enable KeePass Integration")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        let kdbx_enabled_switch = Switch::builder().valign(gtk4::Align::Center).build();
        enable_row.append(&enable_label);
        enable_row.append(&kdbx_enabled_switch);
        kdbx_vbox.append(&enable_row);

        // KDBX file path and password grid
        let path_grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .margin_top(8)
            .build();

        let path_label = Label::builder()
            .label("Database File:")
            .halign(gtk4::Align::End)
            .build();
        let kdbx_path_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Select a .kdbx file...")
            .editable(false)
            .sensitive(false)
            .build();
        let browse_button = Button::builder().label("Browse...").sensitive(false).build();

        path_grid.attach(&path_label, 0, 0, 1, 1);
        path_grid.attach(&kdbx_path_entry, 1, 0, 1, 1);
        path_grid.attach(&browse_button, 2, 0, 1, 1);

        let password_label = Label::builder()
            .label("Password:")
            .halign(gtk4::Align::End)
            .build();
        let kdbx_password_entry = PasswordEntry::builder()
            .hexpand(true)
            .placeholder_text("Enter database password")
            .show_peek_icon(true)
            .sensitive(false)
            .build();

        path_grid.attach(&password_label, 0, 1, 1, 1);
        path_grid.attach(&kdbx_password_entry, 1, 1, 2, 1);
        kdbx_vbox.append(&path_grid);

        // Unlock button and status
        let unlock_row = GtkBox::new(Orientation::Horizontal, 12);
        unlock_row.set_margin_top(8);

        let kdbx_unlock_button = Button::builder()
            .label("Unlock")
            .css_classes(["suggested-action"])
            .sensitive(false)
            .build();
        let kdbx_status_label = Label::builder()
            .label("🔒 Locked")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["dim-label"])
            .build();

        unlock_row.append(&kdbx_unlock_button);
        unlock_row.append(&kdbx_status_label);
        kdbx_vbox.append(&unlock_row);

        kdbx_frame.set_child(Some(&kdbx_vbox));

        (kdbx_frame, kdbx_path_entry, kdbx_password_entry, kdbx_enabled_switch, kdbx_unlock_button, kdbx_status_label, browse_button)
    }

    /// Creates the `KeePassXC` status section
    fn create_keepassxc_status_section() -> GtkBox {
        let keepassxc_frame = Frame::builder().label("KeePassXC Status").build();
        let keepassxc_status_container = GtkBox::new(Orientation::Vertical, 6);
        keepassxc_status_container.set_margin_top(8);
        keepassxc_status_container.set_margin_bottom(8);
        keepassxc_status_container.set_margin_start(8);
        keepassxc_status_container.set_margin_end(8);

        Self::populate_keepassxc_status(&keepassxc_status_container);
        keepassxc_frame.set_child(Some(&keepassxc_status_container));

        keepassxc_status_container
    }

    /// Creates the backend selection section
    fn create_backend_section() -> (Frame, DropDown, CheckButton) {
        let backend_frame = Frame::builder().label("Secret Backend").build();
        let backend_vbox = GtkBox::new(Orientation::Vertical, 8);
        backend_vbox.set_margin_top(8);
        backend_vbox.set_margin_bottom(8);
        backend_vbox.set_margin_start(8);
        backend_vbox.set_margin_end(8);

        let backend_grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .build();

        let backend_label = Label::builder()
            .label("Preferred Backend:")
            .halign(gtk4::Align::End)
            .build();
        let backend_list = StringList::new(&[
            "KeePassXC (Browser Integration)",
            "KDBX File (Direct Access)",
            "libsecret (System Keyring)",
        ]);
        let secret_backend_dropdown = DropDown::new(Some(backend_list), gtk4::Expression::NONE);
        secret_backend_dropdown.set_selected(0);

        backend_grid.attach(&backend_label, 0, 0, 1, 1);
        backend_grid.attach(&secret_backend_dropdown, 1, 0, 1, 1);
        backend_vbox.append(&backend_grid);

        let enable_fallback = CheckButton::builder()
            .label("Enable fallback to libsecret if primary backend unavailable")
            .active(true)
            .margin_top(4)
            .build();
        backend_vbox.append(&enable_fallback);

        let info = Label::builder()
            .label("KeePassXC requires browser integration enabled.\nKDBX File provides direct database access.\nlibsecret uses GNOME Keyring or KDE Wallet.")
            .halign(gtk4::Align::Start)
            .wrap(true)
            .css_classes(["dim-label"])
            .margin_top(8)
            .build();
        backend_vbox.append(&info);

        backend_frame.set_child(Some(&backend_vbox));

        (backend_frame, secret_backend_dropdown, enable_fallback)
    }

    /// Connects the KDBX enable switch to control sensitivity of related widgets
    fn connect_kdbx_enable_switch(
        switch: &Switch,
        path_entry: &Entry,
        password_entry: &PasswordEntry,
        unlock_button: &Button,
        browse_button: &Button,
    ) {
        let path_entry = path_entry.clone();
        let password_entry = password_entry.clone();
        let unlock_button = unlock_button.clone();
        let browse_button = browse_button.clone();

        switch.connect_state_set(move |_, state| {
            path_entry.set_sensitive(state);
            password_entry.set_sensitive(state);
            unlock_button.set_sensitive(state);
            browse_button.set_sensitive(state);
            gtk4::glib::Propagation::Proceed
        });
    }

    /// Populates the `KeePassXC` status container with current detection info
    fn populate_keepassxc_status(container: &GtkBox) {
        // Clear existing children
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let status = KeePassStatus::detect();

        let grid = Grid::builder()
            .row_spacing(6)
            .column_spacing(12)
            .build();

        let mut row = 0;

        // Installation status
        let status_label = Label::builder()
            .label("Status:")
            .halign(gtk4::Align::End)
            .build();
        let status_value = if status.keepassxc_installed {
            Label::builder()
                .label("✓ Installed")
                .halign(gtk4::Align::Start)
                .css_classes(["success"])
                .build()
        } else {
            Label::builder()
                .label("✗ Not installed")
                .halign(gtk4::Align::Start)
                .css_classes(["error"])
                .build()
        };
        grid.attach(&status_label, 0, row, 1, 1);
        grid.attach(&status_value, 1, row, 1, 1);
        row += 1;

        if status.keepassxc_installed {
            // Version
            if let Some(version) = &status.keepassxc_version {
                let version_label = Label::builder()
                    .label("Version:")
                    .halign(gtk4::Align::End)
                    .build();
                let version_value = Label::builder()
                    .label(version)
                    .halign(gtk4::Align::Start)
                    .selectable(true)
                    .build();
                grid.attach(&version_label, 0, row, 1, 1);
                grid.attach(&version_value, 1, row, 1, 1);
                row += 1;
            }

            // Path
            if let Some(path) = &status.keepassxc_path {
                let path_label = Label::builder()
                    .label("Path:")
                    .halign(gtk4::Align::End)
                    .build();
                let path_value = Label::builder()
                    .label(path.to_string_lossy().as_ref())
                    .halign(gtk4::Align::Start)
                    .selectable(true)
                    .ellipsize(gtk4::pango::EllipsizeMode::Middle)
                    .build();
                grid.attach(&path_label, 0, row, 1, 1);
                grid.attach(&path_value, 1, row, 1, 1);
            }
        } else {
            // Installation hint
            let hint_label = Label::builder()
                .label("Install:")
                .halign(gtk4::Align::End)
                .build();
            let hint_value = Label::builder()
                .label("Install KeePassXC from your package manager\nor https://keepassxc.org")
                .halign(gtk4::Align::Start)
                .wrap(true)
                .css_classes(["dim-label"])
                .build();
            grid.attach(&hint_label, 0, row, 1, 1);
            grid.attach(&hint_value, 1, row, 1, 1);
        }

        container.append(&grid);

        // Refresh button
        let refresh_btn = Button::builder()
            .label("Refresh")
            .halign(gtk4::Align::Start)
            .margin_top(8)
            .build();

        let container_clone = container.clone();
        refresh_btn.connect_clicked(move |_| {
            Self::populate_keepassxc_status(&container_clone);
        });

        container.append(&refresh_btn);
    }

    /// Shows the KDBX file selection dialog
    fn show_kdbx_file_dialog(button: &Button, path_entry: &Entry) {
        let dialog = FileDialog::builder()
            .title("Select KeePass Database")
            .modal(true)
            .build();

        // Create filter for .kdbx files
        let filter = FileFilter::new();
        filter.add_pattern("*.kdbx");
        filter.add_pattern("*.KDBX");
        filter.set_name(Some("KeePass Database (*.kdbx)"));

        let filters = gtk4::gio::ListStore::new::<FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));
        dialog.set_default_filter(Some(&filter));

        let path_entry_clone = path_entry.clone();

        // Get the window from the button
        let window = button
            .root()
            .and_then(|r| r.downcast::<Window>().ok());

        dialog.open(window.as_ref(), gtk4::gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    // Validate the path
                    match KeePassStatus::validate_kdbx_path(&path) {
                        Ok(()) => {
                            path_entry_clone.set_text(&path.to_string_lossy());
                        }
                        Err(e) => {
                            eprintln!("Invalid KDBX path: {e}");
                            // Could show an error dialog here
                        }
                    }
                }
            }
        });
    }

    fn create_ui_tab() -> (Frame, CheckButton) {
        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let remember_geometry = CheckButton::builder()
            .label("Remember window size and position")
            .active(true)
            .build();
        vbox.append(&remember_geometry);

        let frame = Frame::builder()
            .label("Window Settings")
            .child(&vbox)
            .margin_top(12)
            .valign(gtk4::Align::Start)
            .build();

        (frame, remember_geometry)
    }

    /// Creates the Clients tab showing detected protocol clients
    fn create_clients_tab() -> ScrolledWindow {
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);
        vbox.set_valign(gtk4::Align::Start);

        // Create a container for client info that can be refreshed
        let clients_container = GtkBox::new(Orientation::Vertical, 12);
        Self::populate_clients_info(&clients_container);
        vbox.append(&clients_container);

        // Add refresh button
        let refresh_btn = Button::builder()
            .label("Refresh")
            .halign(gtk4::Align::Start)
            .margin_top(12)
            .build();

        let container_clone = clients_container;
        refresh_btn.connect_clicked(move |_| {
            // Clear existing children
            while let Some(child) = container_clone.first_child() {
                container_clone.remove(&child);
            }
            // Re-populate with fresh detection
            Self::populate_clients_info(&container_clone);
        });

        vbox.append(&refresh_btn);
        scrolled.set_child(Some(&vbox));
        scrolled
    }

    /// Populates the clients container with detected client information
    fn populate_clients_info(container: &GtkBox) {
        // Detect all clients
        let ssh_info = detect_ssh_client();
        let rdp_info = detect_rdp_client();
        let vnc_info = detect_vnc_client();

        // Add SSH client info
        let ssh_frame = Self::create_client_frame("SSH Client", &ssh_info);
        container.append(&ssh_frame);

        // Add RDP client info
        let rdp_frame = Self::create_client_frame("RDP Client", &rdp_info);
        container.append(&rdp_frame);

        // Add VNC client info
        let vnc_frame = Self::create_client_frame("VNC Client", &vnc_info);
        container.append(&vnc_frame);
    }

    /// Creates a frame displaying information about a single client
    fn create_client_frame(title: &str, info: &ClientInfo) -> Frame {
        let grid = Grid::builder()
            .row_spacing(6)
            .column_spacing(12)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let mut row = 0;

        // Status row
        let status_label = Label::builder()
            .label("Status:")
            .halign(gtk4::Align::End)
            .build();
        let status_value = if info.installed {
            Label::builder()
                .label("✓ Installed")
                .halign(gtk4::Align::Start)
                .css_classes(["success"])
                .build()
        } else {
            Label::builder()
                .label("✗ Not installed")
                .halign(gtk4::Align::Start)
                .css_classes(["error"])
                .build()
        };
        grid.attach(&status_label, 0, row, 1, 1);
        grid.attach(&status_value, 1, row, 1, 1);
        row += 1;

        // Name row
        let name_label = Label::builder()
            .label("Name:")
            .halign(gtk4::Align::End)
            .build();
        let name_value = Label::builder()
            .label(&info.name)
            .halign(gtk4::Align::Start)
            .selectable(true)
            .build();
        grid.attach(&name_label, 0, row, 1, 1);
        grid.attach(&name_value, 1, row, 1, 1);
        row += 1;

        if info.installed {
            // Path row (only if installed)
            if let Some(path) = &info.path {
                let path_label = Label::builder()
                    .label("Path:")
                    .halign(gtk4::Align::End)
                    .build();
                let path_value = Label::builder()
                    .label(path.to_string_lossy().as_ref())
                    .halign(gtk4::Align::Start)
                    .selectable(true)
                    .ellipsize(gtk4::pango::EllipsizeMode::Middle)
                    .build();
                grid.attach(&path_label, 0, row, 1, 1);
                grid.attach(&path_value, 1, row, 1, 1);
                row += 1;
            }

            // Version row (only if installed and version available)
            if let Some(version) = &info.version {
                let version_label = Label::builder()
                    .label("Version:")
                    .halign(gtk4::Align::End)
                    .build();
                let version_value = Label::builder()
                    .label(version)
                    .halign(gtk4::Align::Start)
                    .selectable(true)
                    .wrap(true)
                    .max_width_chars(50)
                    .build();
                grid.attach(&version_label, 0, row, 1, 1);
                grid.attach(&version_value, 1, row, 1, 1);
            }
        } else if let Some(hint) = &info.install_hint {
            // Installation hint (only if not installed)
            let hint_label = Label::builder()
                .label("Install:")
                .halign(gtk4::Align::End)
                .valign(gtk4::Align::Start)
                .build();
            let hint_value = Label::builder()
                .label(hint)
                .halign(gtk4::Align::Start)
                .selectable(true)
                .wrap(true)
                .max_width_chars(50)
                .css_classes(["dim-label"])
                .build();
            grid.attach(&hint_label, 0, row, 1, 1);
            grid.attach(&hint_value, 1, row, 1, 1);
        }

        Frame::builder().label(title).child(&grid).build()
    }

    /// Populates the dialog with existing settings
    pub fn set_settings(&self, settings: &AppSettings) {
        *self.settings.borrow_mut() = settings.clone();

        // Terminal settings
        self.font_family_entry
            .set_text(&settings.terminal.font_family);
        self.font_size_spin
            .set_value(f64::from(settings.terminal.font_size));
        self.scrollback_spin
            .set_value(f64::from(settings.terminal.scrollback_lines));

        // Logging settings
        self.logging_enabled.set_active(settings.logging.enabled);
        self.log_dir_entry
            .set_text(&settings.logging.log_directory.to_string_lossy());
        self.retention_spin
            .set_value(f64::from(settings.logging.retention_days));
        self.log_dir_entry.set_sensitive(settings.logging.enabled);
        self.retention_spin.set_sensitive(settings.logging.enabled);

        // Secret settings - using dropdown index
        let backend_idx = match settings.secrets.preferred_backend {
            SecretBackendType::KeePassXc => 0,
            SecretBackendType::KdbxFile => 1,
            SecretBackendType::LibSecret => 2,
        };
        self.secret_backend_dropdown.set_selected(backend_idx);
        self.enable_fallback
            .set_active(settings.secrets.enable_fallback);

        // KeePass settings
        self.kdbx_enabled_switch
            .set_active(settings.secrets.kdbx_enabled);
        if let Some(ref path) = settings.secrets.kdbx_path {
            self.kdbx_path_entry.set_text(&path.to_string_lossy());
        } else {
            self.kdbx_path_entry.set_text("");
        }

        // Update sensitivity based on enabled state
        let enabled = settings.secrets.kdbx_enabled;
        self.kdbx_path_entry.set_sensitive(enabled);
        self.kdbx_password_entry.set_sensitive(enabled);
        self.kdbx_unlock_button.set_sensitive(enabled);

        // Reset unlock state
        *self.kdbx_unlocked.borrow_mut() = false;
        self.kdbx_status_label.set_text("🔒 Locked");
        self.kdbx_unlock_button.set_label("Unlock");
        self.kdbx_unlock_button
            .remove_css_class("destructive-action");
        self.kdbx_unlock_button.add_css_class("suggested-action");

        // Refresh KeePassXC status
        Self::populate_keepassxc_status(&self.keepassxc_status_container);

        // UI settings
        self.remember_geometry
            .set_active(settings.ui.remember_window_geometry);
    }

    /// Runs the dialog and calls the callback with the result
    ///
    /// Note: The settings are built inline in the click handler closure
    /// because the closure needs to capture individual fields rather than `&self`.
    pub fn run<F: Fn(Option<AppSettings>) + 'static>(&self, cb: F) {
        // Store callback
        *self.on_save.borrow_mut() = Some(Box::new(cb));

        // Connect unlock button
        self.connect_unlock_button();

        // Get the save button from header bar and connect it
        if let Some(titlebar) = self.window.titlebar() {
            if let Some(header) = titlebar.downcast_ref::<HeaderBar>() {
                if let Some(save_btn) = header.last_child() {
                    if let Some(btn) = save_btn.downcast_ref::<Button>() {
                        let window = self.window.clone();
                        let on_save = self.on_save.clone();
                        let font_family_entry = self.font_family_entry.clone();
                        let font_size_spin = self.font_size_spin.clone();
                        let scrollback_spin = self.scrollback_spin.clone();
                        let logging_enabled = self.logging_enabled.clone();
                        let log_dir_entry = self.log_dir_entry.clone();
                        let retention_spin = self.retention_spin.clone();
                        let secret_backend_dropdown = self.secret_backend_dropdown.clone();
                        let enable_fallback = self.enable_fallback.clone();
                        let kdbx_path_entry = self.kdbx_path_entry.clone();
                        let kdbx_password_entry = self.kdbx_password_entry.clone();
                        let kdbx_enabled_switch = self.kdbx_enabled_switch.clone();
                        let remember_geometry = self.remember_geometry.clone();
                        let settings = self.settings.clone();

                        btn.connect_clicked(move |_| {
                            // SpinButton values are constrained by their adjustments to valid u32 ranges
                            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            let terminal = TerminalSettings {
                                font_family: font_family_entry.text().to_string(),
                                font_size: font_size_spin.value() as u32,
                                scrollback_lines: scrollback_spin.value() as u32,
                            };

                            // SpinButton values are constrained by their adjustments to valid u32 ranges
                            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            let logging = LoggingSettings {
                                enabled: logging_enabled.is_active(),
                                log_directory: PathBuf::from(log_dir_entry.text().to_string()),
                                retention_days: retention_spin.value() as u32,
                            };

                            // Map dropdown index to backend type
                            let preferred_backend = match secret_backend_dropdown.selected() {
                                0 => SecretBackendType::KeePassXc,
                                1 => SecretBackendType::KdbxFile,
                                _ => SecretBackendType::LibSecret,
                            };

                            // Get KDBX path from entry
                            let kdbx_path_text = kdbx_path_entry.text();
                            let kdbx_path = if kdbx_path_text.is_empty() {
                                None
                            } else {
                                Some(PathBuf::from(kdbx_path_text.to_string()))
                            };

                            // Get password (only kept in memory, not persisted)
                            let kdbx_password_text = kdbx_password_entry.text();
                            let kdbx_password = if kdbx_password_text.is_empty() {
                                None
                            } else {
                                Some(SecretString::from(kdbx_password_text.to_string()))
                            };

                            let secrets = SecretSettings {
                                preferred_backend,
                                enable_fallback: enable_fallback.is_active(),
                                kdbx_path,
                                kdbx_enabled: kdbx_enabled_switch.is_active(),
                                kdbx_password,
                            };

                            let existing = settings.borrow();
                            let ui = UiSettings {
                                remember_window_geometry: remember_geometry.is_active(),
                                window_width: existing.ui.window_width,
                                window_height: existing.ui.window_height,
                                sidebar_width: existing.ui.sidebar_width,
                            };

                            let new_settings = AppSettings {
                                terminal,
                                logging,
                                secrets,
                                ui,
                            };

                            if let Some(ref cb) = *on_save.borrow() {
                                cb(Some(new_settings));
                            }
                            window.close();
                        });
                    }
                }
            }
        }

        self.window.present();
    }

    /// Connects the unlock/lock button functionality
    fn connect_unlock_button(&self) {
        let status_label = self.kdbx_status_label.clone();
        let password_entry = self.kdbx_password_entry.clone();
        let path_entry = self.kdbx_path_entry.clone();
        let unlocked = self.kdbx_unlocked.clone();

        self.kdbx_unlock_button.connect_clicked(move |btn| {
            let is_unlocked = *unlocked.borrow();

            if is_unlocked {
                // Lock the database
                *unlocked.borrow_mut() = false;
                status_label.set_text("🔒 Locked");
                btn.set_label("Unlock");
                btn.remove_css_class("destructive-action");
                btn.add_css_class("suggested-action");
                password_entry.set_text("");
            } else {
                // Try to unlock
                let path_text = path_entry.text();
                let password_text = password_entry.text();

                if path_text.is_empty() {
                    status_label.set_text("⚠️ No database file selected");
                    return;
                }

                if password_text.is_empty() {
                    status_label.set_text("⚠️ Password required");
                    return;
                }

                // Validate the path
                let path = PathBuf::from(path_text.to_string());
                if let Err(e) = KeePassStatus::validate_kdbx_path(&path) {
                    status_label.set_text(&format!("⚠️ {e}"));
                    return;
                }

                // For now, we just mark as unlocked
                // Actual database verification would happen in the core library
                *unlocked.borrow_mut() = true;
                status_label.set_text("🔓 Unlocked");
                btn.set_label("Lock");
                btn.remove_css_class("suggested-action");
                btn.add_css_class("destructive-action");
            }
        });
    }

    /// Returns a reference to the underlying window
    #[must_use]
    pub const fn window(&self) -> &Window {
        &self.window
    }
}
