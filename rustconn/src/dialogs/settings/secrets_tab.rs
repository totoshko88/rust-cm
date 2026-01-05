//! Secrets settings tab

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, Label, Orientation, PasswordEntry,
    ScrolledWindow, StringList, Switch,
};
use rustconn_core::config::{SecretBackendType, SecretSettings};
use std::cell::RefCell;
use std::rc::Rc;

/// Creates the secrets settings tab
#[allow(clippy::type_complexity)]
pub fn create_secrets_tab() -> (
    ScrolledWindow,
    DropDown,
    CheckButton,
    Entry,
    PasswordEntry,
    Switch,
    CheckButton,
    Label,
    Button,
    GtkBox,
    Entry,
    Button,
    CheckButton,
    CheckButton,
) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();

    let main_vbox = GtkBox::new(Orientation::Vertical, 6);
    main_vbox.set_margin_top(12);
    main_vbox.set_margin_bottom(12);
    main_vbox.set_margin_start(12);
    main_vbox.set_margin_end(12);
    main_vbox.set_valign(gtk4::Align::Start);

    // === Secret Backend section ===
    let backend_header = Label::builder()
        .label("Secret Backend")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();
    main_vbox.append(&backend_header);

    let backend_selection_hbox = GtkBox::new(Orientation::Horizontal, 12);
    backend_selection_hbox.set_margin_start(6);
    backend_selection_hbox.set_margin_top(6);
    backend_selection_hbox.append(&Label::new(Some("Backend:")));

    let backend_strings = StringList::new(&["KeePassXC", "libsecret", "KDBX File"]);
    let secret_backend_dropdown = DropDown::builder()
        .model(&backend_strings)
        .selected(0)
        .hexpand(true)
        .build();
    backend_selection_hbox.append(&secret_backend_dropdown);
    main_vbox.append(&backend_selection_hbox);

    let enable_fallback =
        CheckButton::with_label("Enable fallback to libsecret if KeePassXC unavailable");
    enable_fallback.set_active(true);
    enable_fallback.set_margin_start(6);
    main_vbox.append(&enable_fallback);

    // === KeePass Database section ===
    let kdbx_header = Label::builder()
        .label("KeePass Database")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&kdbx_header);

    // KeePass Integration switch
    let kdbx_enabled_hbox = GtkBox::new(Orientation::Horizontal, 12);
    kdbx_enabled_hbox.set_margin_start(6);
    kdbx_enabled_hbox.set_margin_top(6);
    let kdbx_integration_label = Label::new(Some("KeePass Integration"));
    kdbx_integration_label.set_halign(gtk4::Align::Start);
    kdbx_integration_label.set_hexpand(true);

    let kdbx_enabled_switch = Switch::new();
    kdbx_enabled_switch.set_halign(gtk4::Align::End);

    kdbx_enabled_hbox.append(&kdbx_integration_label);
    kdbx_enabled_hbox.append(&kdbx_enabled_switch);
    main_vbox.append(&kdbx_enabled_hbox);

    // Database path
    let kdbx_path_hbox = GtkBox::new(Orientation::Horizontal, 12);
    kdbx_path_hbox.set_margin_start(6);
    kdbx_path_hbox.set_margin_top(6);
    let database_label = Label::new(Some("Database:"));
    database_label.set_size_request(100, -1);
    database_label.set_halign(gtk4::Align::Start);

    let kdbx_path_entry = Entry::builder()
        .placeholder_text("Select .kdbx file")
        .hexpand(true)
        .build();
    let kdbx_browse_button = Button::with_label("Browse");

    kdbx_path_hbox.append(&database_label);
    kdbx_path_hbox.append(&kdbx_path_entry);
    kdbx_path_hbox.append(&kdbx_browse_button);
    main_vbox.append(&kdbx_path_hbox);

    // Authentication methods
    let auth_label = Label::builder()
        .label("Authentication:")
        .halign(gtk4::Align::Start)
        .margin_top(12)
        .margin_start(6)
        .css_classes(["dim-label"])
        .build();
    main_vbox.append(&auth_label);

    let kdbx_use_password_check = CheckButton::with_label("Use password");
    kdbx_use_password_check.set_active(true);
    kdbx_use_password_check.set_margin_start(6);
    main_vbox.append(&kdbx_use_password_check);

    // Password entry
    let kdbx_password_hbox = GtkBox::new(Orientation::Horizontal, 12);
    kdbx_password_hbox.set_margin_start(6);
    let password_label = Label::new(Some("Password:"));
    password_label.set_size_request(100, -1);
    password_label.set_halign(gtk4::Align::Start);

    let kdbx_password_entry = PasswordEntry::builder()
        .placeholder_text("Database password")
        .hexpand(true)
        .show_peek_icon(true)
        .build();

    kdbx_password_hbox.append(&password_label);
    kdbx_password_hbox.append(&kdbx_password_entry);
    main_vbox.append(&kdbx_password_hbox);

    let kdbx_save_password_check = CheckButton::with_label("Save password (encrypted)");
    kdbx_save_password_check.set_margin_start(6);
    main_vbox.append(&kdbx_save_password_check);

    let kdbx_use_key_file_check = CheckButton::with_label("Use key file");
    kdbx_use_key_file_check.set_margin_start(6);
    kdbx_use_key_file_check.set_margin_top(6);
    main_vbox.append(&kdbx_use_key_file_check);

    // Key file entry
    let kdbx_key_file_hbox = GtkBox::new(Orientation::Horizontal, 12);
    kdbx_key_file_hbox.set_margin_start(6);
    let key_file_label = Label::new(Some("Key file:"));
    key_file_label.set_size_request(100, -1);
    key_file_label.set_halign(gtk4::Align::Start);

    let kdbx_key_file_entry = Entry::builder()
        .placeholder_text("Select .keyx or .key file")
        .hexpand(true)
        .build();
    let kdbx_key_file_browse_button = Button::with_label("Browse");

    kdbx_key_file_hbox.append(&key_file_label);
    kdbx_key_file_hbox.append(&kdbx_key_file_entry);
    kdbx_key_file_hbox.append(&kdbx_key_file_browse_button);
    main_vbox.append(&kdbx_key_file_hbox);

    // Status display
    let status_hbox = GtkBox::new(Orientation::Horizontal, 6);
    status_hbox.set_margin_start(6);
    status_hbox.set_margin_top(6);
    let status_icon = Label::new(Some("●"));
    status_icon.add_css_class("dim-label");

    let kdbx_status_label = Label::builder()
        .label("Status: Not connected")
        .halign(gtk4::Align::Start)
        .build();

    status_hbox.append(&status_icon);
    status_hbox.append(&kdbx_status_label);
    main_vbox.append(&status_hbox);

    // Setup checkbox functionality
    setup_checkbox_functionality(
        &kdbx_use_password_check,
        &kdbx_use_key_file_check,
        &kdbx_password_hbox,
        &kdbx_key_file_hbox,
    );

    // === KeePassXC Status section ===
    let keepassxc_header = Label::builder()
        .label("KeePassXC Status")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&keepassxc_header);

    // Check KeePassXC installation
    let keepassxc_installed = std::process::Command::new("which")
        .arg("keepassxc-cli")
        .output()
        .is_ok_and(|output| output.status.success());

    let (status_text, status_icon_text, status_css_class) = if keepassxc_installed {
        ("Installed", "✓", "success")
    } else {
        ("Not installed", "✗", "error")
    };

    let status_display_hbox = GtkBox::new(Orientation::Horizontal, 6);
    status_display_hbox.set_margin_start(6);
    status_display_hbox.set_margin_top(6);
    let keepassxc_status_icon = Label::new(Some(status_icon_text));
    keepassxc_status_icon.add_css_class(status_css_class);

    let keepassxc_status_label = Label::builder()
        .label(&format!("Status: {status_text}"))
        .halign(gtk4::Align::Start)
        .build();

    status_display_hbox.append(&keepassxc_status_icon);
    status_display_hbox.append(&keepassxc_status_label);
    main_vbox.append(&status_display_hbox);

    if keepassxc_installed {
        if let Ok(output) = std::process::Command::new("keepassxc-cli")
            .arg("--version")
            .output()
        {
            if let Ok(version_str) = String::from_utf8(output.stdout) {
                let version = version_str.lines().next().unwrap_or("Unknown version");
                let version_label = Label::builder()
                    .label(&format!("Version: {version}"))
                    .halign(gtk4::Align::Start)
                    .css_classes(["dim-label"])
                    .margin_start(6)
                    .build();
                main_vbox.append(&version_label);
            }
        }

        if let Ok(output) = std::process::Command::new("which")
            .arg("keepassxc-cli")
            .output()
        {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                let path_label = Label::builder()
                    .label(&format!("Path: {path}"))
                    .halign(gtk4::Align::Start)
                    .css_classes(["dim-label"])
                    .margin_start(6)
                    .build();
                main_vbox.append(&path_label);
            }
        }
    } else {
        let help_label = Label::builder()
            .label("Install KeePassXC package for database integration")
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label"])
            .wrap(true)
            .margin_start(6)
            .build();
        main_vbox.append(&help_label);
    }

    scrolled.set_child(Some(&main_vbox));

    let keepassxc_status_container = GtkBox::new(Orientation::Vertical, 6);

    (
        scrolled,
        secret_backend_dropdown,
        enable_fallback,
        kdbx_path_entry,
        kdbx_password_entry,
        kdbx_enabled_switch,
        kdbx_save_password_check,
        kdbx_status_label,
        kdbx_browse_button,
        keepassxc_status_container,
        kdbx_key_file_entry,
        kdbx_key_file_browse_button,
        kdbx_use_key_file_check,
        kdbx_use_password_check,
    )
}

/// Sets up checkbox functionality to enable/disable related fields
fn setup_checkbox_functionality(
    password_check: &CheckButton,
    key_file_check: &CheckButton,
    password_hbox: &GtkBox,
    key_file_hbox: &GtkBox,
) {
    let password_hbox_clone = password_hbox.clone();
    let key_file_hbox_clone = key_file_hbox.clone();

    password_check.connect_toggled(move |check| {
        password_hbox_clone.set_sensitive(check.is_active());
    });

    key_file_check.connect_toggled(move |check| {
        key_file_hbox_clone.set_sensitive(check.is_active());
    });

    password_hbox.set_sensitive(password_check.is_active());
    key_file_hbox.set_sensitive(key_file_check.is_active());
}

/// Loads secret settings into UI controls
#[allow(clippy::too_many_arguments)]
pub fn load_secret_settings(
    secret_backend_dropdown: &DropDown,
    enable_fallback: &CheckButton,
    kdbx_path_entry: &Entry,
    _kdbx_password_entry: &PasswordEntry,
    kdbx_enabled_switch: &Switch,
    kdbx_save_password_check: &CheckButton,
    kdbx_status_label: &Label,
    _keepassxc_status_container: &GtkBox,
    kdbx_key_file_entry: &Entry,
    kdbx_use_key_file_check: &CheckButton,
    kdbx_use_password_check: &CheckButton,
    settings: &SecretSettings,
) {
    let backend_index = match settings.preferred_backend {
        SecretBackendType::KeePassXc => 0,
        SecretBackendType::LibSecret => 1,
        SecretBackendType::KdbxFile => 2,
    };
    secret_backend_dropdown.set_selected(backend_index);
    enable_fallback.set_active(settings.enable_fallback);
    kdbx_enabled_switch.set_active(settings.kdbx_enabled);

    if let Some(path) = &settings.kdbx_path {
        kdbx_path_entry.set_text(&path.display().to_string());
    }

    if let Some(key_file) = &settings.kdbx_key_file {
        kdbx_key_file_entry.set_text(&key_file.display().to_string());
    }

    kdbx_use_password_check.set_active(settings.kdbx_use_password);
    kdbx_use_key_file_check.set_active(settings.kdbx_use_key_file);
    kdbx_save_password_check.set_active(settings.kdbx_password_encrypted.is_some());

    let status_text = if settings.kdbx_enabled {
        if settings.kdbx_path.is_some() {
            "Configured"
        } else {
            "Database path required"
        }
    } else {
        "Disabled"
    };

    kdbx_status_label.set_text(&format!("Status: {status_text}"));

    kdbx_status_label.remove_css_class("success");
    kdbx_status_label.remove_css_class("warning");
    kdbx_status_label.remove_css_class("error");
    kdbx_status_label.remove_css_class("dim-label");

    let status_css_class = if settings.kdbx_enabled {
        if settings.kdbx_path.is_some() {
            "success"
        } else {
            "warning"
        }
    } else {
        "dim-label"
    };
    kdbx_status_label.add_css_class(status_css_class);
}

/// Collects secret settings from UI controls
#[allow(clippy::too_many_arguments)]
pub fn collect_secret_settings(
    secret_backend_dropdown: &DropDown,
    enable_fallback: &CheckButton,
    kdbx_path_entry: &Entry,
    kdbx_password_entry: &PasswordEntry,
    kdbx_enabled_switch: &Switch,
    kdbx_save_password_check: &CheckButton,
    kdbx_key_file_entry: &Entry,
    kdbx_use_key_file_check: &CheckButton,
    kdbx_use_password_check: &CheckButton,
    settings: &Rc<RefCell<rustconn_core::config::AppSettings>>,
) -> SecretSettings {
    let preferred_backend = match secret_backend_dropdown.selected() {
        0 => SecretBackendType::KeePassXc,
        1 => SecretBackendType::LibSecret,
        2 => SecretBackendType::KdbxFile,
        _ => SecretBackendType::default(),
    };

    let kdbx_path = {
        let path_text = kdbx_path_entry.text();
        if path_text.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(path_text.as_str()))
        }
    };

    let kdbx_key_file = {
        let key_file_text = kdbx_key_file_entry.text();
        if key_file_text.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(key_file_text.as_str()))
        }
    };

    let (kdbx_password, kdbx_password_encrypted) = if kdbx_save_password_check.is_active() {
        let password_text = kdbx_password_entry.text();
        if password_text.is_empty() {
            (None, None)
        } else {
            let password = secrecy::SecretString::new(password_text.to_string().into());
            let encrypted = settings
                .borrow()
                .secrets
                .kdbx_password_encrypted
                .clone()
                .or_else(|| Some("encrypted_password_placeholder".to_string()));
            (Some(password), encrypted)
        }
    } else {
        (None, None)
    };

    SecretSettings {
        preferred_backend,
        enable_fallback: enable_fallback.is_active(),
        kdbx_path,
        kdbx_enabled: kdbx_enabled_switch.is_active(),
        kdbx_password,
        kdbx_password_encrypted,
        kdbx_key_file,
        kdbx_use_key_file: kdbx_use_key_file_check.is_active(),
        kdbx_use_password: kdbx_use_password_check.is_active(),
    }
}
