//! Secrets settings tab using libadwaita components

use adw::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, Label, Orientation, PasswordEntry,
    StringList, Switch,
};
use libadwaita as adw;
use rustconn_core::config::{SecretBackendType, SecretSettings};
use std::cell::RefCell;
use std::rc::Rc;

/// Creates the secrets settings page using AdwPreferencesPage
#[allow(clippy::type_complexity)]
pub fn create_secrets_page() -> (
    adw::PreferencesPage,
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
    Switch, // kdbx_use_key_file_check changed to Switch
    Switch, // kdbx_use_password_check changed to Switch
) {
    let page = adw::PreferencesPage::builder()
        .title("Secrets")
        .icon_name("dialog-password-symbolic")
        .build();

    // === Secret Backend Group ===
    let backend_group = adw::PreferencesGroup::builder()
        .title("Secret Backend")
        .description("Choose how passwords are stored")
        .build();

    let backend_strings = StringList::new(&["KeePassXC", "libsecret", "KDBX File"]);
    let secret_backend_dropdown = DropDown::builder()
        .model(&backend_strings)
        .selected(0)
        .valign(gtk4::Align::Center)
        .build();
    let backend_row = adw::ActionRow::builder()
        .title("Backend")
        .subtitle("Primary password storage method")
        .build();
    backend_row.add_suffix(&secret_backend_dropdown);
    backend_row.set_activatable_widget(Some(&secret_backend_dropdown));
    backend_group.add(&backend_row);

    let enable_fallback = CheckButton::builder()
        .valign(gtk4::Align::Center)
        .active(true)
        .build();
    let fallback_row = adw::ActionRow::builder()
        .title("Enable fallback")
        .subtitle("Use libsecret if KeePassXC unavailable")
        .activatable_widget(&enable_fallback)
        .build();
    fallback_row.add_prefix(&enable_fallback);
    backend_group.add(&fallback_row);

    page.add(&backend_group);

    // === KeePass Database Group ===
    let kdbx_group = adw::PreferencesGroup::builder()
        .title("KeePass Database")
        .description("Configure KDBX file integration")
        .build();

    let kdbx_enabled_switch = Switch::builder().valign(gtk4::Align::Center).build();
    let kdbx_enabled_row = adw::ActionRow::builder()
        .title("KeePass Integration")
        .subtitle("Enable database connection")
        .build();
    kdbx_enabled_row.add_suffix(&kdbx_enabled_switch);
    kdbx_enabled_row.set_activatable_widget(Some(&kdbx_enabled_switch));
    kdbx_group.add(&kdbx_enabled_row);

    // Database path with browse button
    let kdbx_path_entry = Entry::builder()
        .placeholder_text("Select .kdbx file")
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();
    let kdbx_browse_button = Button::builder()
        .icon_name("folder-open-symbolic")
        .valign(gtk4::Align::Center)
        .tooltip_text("Browse for database file")
        .build();
    let kdbx_path_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .build();
    kdbx_path_box.append(&kdbx_path_entry);
    kdbx_path_box.append(&kdbx_browse_button);

    let kdbx_path_row = adw::ActionRow::builder().title("Database File").build();
    kdbx_path_row.add_suffix(&kdbx_path_box);
    kdbx_group.add(&kdbx_path_row);

    page.add(&kdbx_group);

    // === Authentication Group ===
    let auth_group = adw::PreferencesGroup::builder()
        .title("Authentication")
        .description("Database unlock methods")
        .build();

    // Use password switch
    let kdbx_use_password_check = Switch::builder()
        .active(true)
        .valign(gtk4::Align::Center)
        .build();
    let use_password_row = adw::ActionRow::builder().title("Use password").build();
    use_password_row.add_suffix(&kdbx_use_password_check);
    use_password_row.set_activatable_widget(Some(&kdbx_use_password_check));
    auth_group.add(&use_password_row);

    // Password entry
    let kdbx_password_entry = PasswordEntry::builder()
        .placeholder_text("Database password")
        .hexpand(true)
        .show_peek_icon(true)
        .valign(gtk4::Align::Center)
        .build();
    let password_row = adw::ActionRow::builder().title("Password").build();
    password_row.add_suffix(&kdbx_password_entry);
    password_row.set_activatable_widget(Some(&kdbx_password_entry));
    auth_group.add(&password_row);

    // Save password checkbox
    let kdbx_save_password_check = CheckButton::builder().valign(gtk4::Align::Center).build();
    let save_password_row = adw::ActionRow::builder()
        .title("Save password")
        .subtitle("Encrypted storage")
        .activatable_widget(&kdbx_save_password_check)
        .build();
    save_password_row.add_prefix(&kdbx_save_password_check);
    auth_group.add(&save_password_row);

    // Use key file switch
    let kdbx_use_key_file_check = Switch::builder().valign(gtk4::Align::Center).build();
    let use_key_file_row = adw::ActionRow::builder().title("Use key file").build();
    use_key_file_row.add_suffix(&kdbx_use_key_file_check);
    use_key_file_row.set_activatable_widget(Some(&kdbx_use_key_file_check));
    auth_group.add(&use_key_file_row);

    // Key file path with browse button
    let kdbx_key_file_entry = Entry::builder()
        .placeholder_text("Select .keyx or .key file")
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();
    let kdbx_key_file_browse_button = Button::builder()
        .icon_name("folder-open-symbolic")
        .valign(gtk4::Align::Center)
        .tooltip_text("Browse for key file")
        .build();
    let key_file_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .build();
    key_file_box.append(&kdbx_key_file_entry);
    key_file_box.append(&kdbx_key_file_browse_button);

    let key_file_row = adw::ActionRow::builder().title("Key File").build();
    key_file_row.add_suffix(&key_file_box);
    auth_group.add(&key_file_row);

    page.add(&auth_group);

    // === Status Group ===
    let status_group = adw::PreferencesGroup::builder().title("Status").build();

    let kdbx_status_label = Label::builder()
        .label("Not connected")
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .css_classes(["dim-label"])
        .build();
    let status_row = adw::ActionRow::builder().title("Connection Status").build();
    status_row.add_suffix(&kdbx_status_label);
    status_group.add(&status_row);

    // Check KeePassXC installation
    let keepassxc_installed = std::process::Command::new("which")
        .arg("keepassxc-cli")
        .output()
        .is_ok_and(|output| output.status.success());

    let (status_text, status_css) = if keepassxc_installed {
        ("Installed", "success")
    } else {
        ("Not installed", "warning")
    };

    let keepassxc_label = Label::builder()
        .label(status_text)
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .css_classes([status_css])
        .build();
    let keepassxc_row = adw::ActionRow::builder()
        .title("KeePassXC CLI")
        .subtitle(if keepassxc_installed {
            "Ready for use"
        } else {
            "Install for database integration"
        })
        .build();
    keepassxc_row.add_suffix(&keepassxc_label);
    status_group.add(&keepassxc_row);

    page.add(&status_group);

    // Setup sensitivity connections
    let password_row_clone = password_row.clone();
    let kdbx_password_entry_clone = kdbx_password_entry.clone();
    let kdbx_save_password_check_clone = kdbx_save_password_check.clone();
    kdbx_use_password_check.connect_state_set(move |_, state| {
        password_row_clone.set_sensitive(state);
        kdbx_password_entry_clone.set_sensitive(state);
        kdbx_save_password_check_clone.set_sensitive(state);
        glib::Propagation::Proceed
    });

    let key_file_row_clone = key_file_row.clone();
    let kdbx_key_file_entry_clone = kdbx_key_file_entry.clone();
    let kdbx_key_file_browse_button_clone = kdbx_key_file_browse_button.clone();
    kdbx_use_key_file_check.connect_state_set(move |_, state| {
        key_file_row_clone.set_sensitive(state);
        kdbx_key_file_entry_clone.set_sensitive(state);
        kdbx_key_file_browse_button_clone.set_sensitive(state);
        glib::Propagation::Proceed
    });

    // Initial sensitivity
    key_file_row.set_sensitive(false);
    kdbx_key_file_entry.set_sensitive(false);
    kdbx_key_file_browse_button.set_sensitive(false);

    let keepassxc_status_container = GtkBox::new(Orientation::Vertical, 6);

    (
        page,
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
    kdbx_use_key_file_check: &Switch,
    kdbx_use_password_check: &Switch,
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

    kdbx_status_label.set_text(status_text);

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
    kdbx_use_key_file_check: &Switch,
    kdbx_use_password_check: &Switch,
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
