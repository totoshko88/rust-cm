//! Secrets settings tab using libadwaita components

use adw::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, FileDialog, FileFilter, Label,
    Orientation, PasswordEntry, StringList, Switch,
};
use libadwaita as adw;
use rustconn_core::config::{SecretBackendType, SecretSettings};
use std::cell::RefCell;
use std::rc::Rc;

/// Return type for secrets page - contains all widgets needed for dynamic visibility
#[allow(dead_code)] // Fields kept for GTK widget lifecycle
pub struct SecretsPageWidgets {
    pub page: adw::PreferencesPage,
    pub secret_backend_dropdown: DropDown,
    pub enable_fallback: CheckButton,
    pub kdbx_path_entry: Entry,
    pub kdbx_password_entry: PasswordEntry,
    pub kdbx_enabled_switch: Switch,
    pub kdbx_save_password_check: CheckButton,
    pub kdbx_status_label: Label,
    pub kdbx_browse_button: Button,
    pub kdbx_check_button: Button,
    pub keepassxc_status_container: GtkBox,
    pub kdbx_key_file_entry: Entry,
    pub kdbx_key_file_browse_button: Button,
    pub kdbx_use_key_file_check: Switch,
    pub kdbx_use_password_check: Switch,
    // Additional rows for visibility control
    pub kdbx_group: adw::PreferencesGroup,
    pub auth_group: adw::PreferencesGroup,
    pub status_group: adw::PreferencesGroup,
    pub password_row: adw::ActionRow,
    pub save_password_row: adw::ActionRow,
    pub key_file_row: adw::ActionRow,
}

/// Creates the secrets settings page using AdwPreferencesPage
#[allow(clippy::type_complexity)]
pub fn create_secrets_page() -> SecretsPageWidgets {
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

    // Check connection button
    let kdbx_check_button = Button::builder()
        .label("Check")
        .valign(gtk4::Align::Center)
        .tooltip_text("Test database connection")
        .build();

    let kdbx_status_label = Label::builder()
        .label("Not connected")
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .css_classes(["dim-label"])
        .build();

    let status_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .valign(gtk4::Align::Center)
        .build();
    status_box.append(&kdbx_status_label);
    status_box.append(&kdbx_check_button);

    let status_row = adw::ActionRow::builder().title("Connection Status").build();
    status_row.add_suffix(&status_box);
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

    // Setup visibility connections for password fields
    let password_row_clone = password_row.clone();
    let save_password_row_clone = save_password_row.clone();
    kdbx_use_password_check.connect_state_set(move |_, state| {
        password_row_clone.set_visible(state);
        save_password_row_clone.set_visible(state);
        glib::Propagation::Proceed
    });

    // Setup visibility connections for key file fields
    let key_file_row_clone = key_file_row.clone();
    kdbx_use_key_file_check.connect_state_set(move |_, state| {
        key_file_row_clone.set_visible(state);
        glib::Propagation::Proceed
    });

    // Setup visibility for KeePass sections when integration is enabled/disabled
    let auth_group_clone = auth_group.clone();
    let status_group_clone = status_group.clone();
    kdbx_enabled_switch.connect_state_set(move |_, state| {
        auth_group_clone.set_visible(state);
        status_group_clone.set_visible(state);
        glib::Propagation::Proceed
    });

    // Initial visibility based on default states
    // Key file row hidden by default (use_key_file is false)
    key_file_row.set_visible(false);
    // Password row visible by default (use_password is true)
    password_row.set_visible(true);
    save_password_row.set_visible(true);
    // Auth and Status groups hidden by default (kdbx_enabled is false)
    auth_group.set_visible(false);
    status_group.set_visible(false);

    // Setup browse button for database file
    let kdbx_path_entry_clone = kdbx_path_entry.clone();
    kdbx_browse_button.connect_clicked(move |button| {
        let entry = kdbx_path_entry_clone.clone();
        let dialog = FileDialog::builder()
            .title("Select KeePass Database")
            .modal(true)
            .build();

        // Add filter for .kdbx files
        let filter = FileFilter::new();
        filter.add_pattern("*.kdbx");
        filter.set_name(Some("KeePass Database (*.kdbx)"));

        let filters = gtk4::gio::ListStore::new::<FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));
        dialog.set_default_filter(Some(&filter));

        // Get the root window
        let root = button.root();
        let window = root.and_then(|r| r.downcast::<gtk4::Window>().ok());

        dialog.open(
            window.as_ref(),
            gtk4::gio::Cancellable::NONE,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        entry.set_text(&path.display().to_string());
                    }
                }
            },
        );
    });

    // Setup browse button for key file
    let kdbx_key_file_entry_clone = kdbx_key_file_entry.clone();
    kdbx_key_file_browse_button.connect_clicked(move |button| {
        let entry = kdbx_key_file_entry_clone.clone();
        let dialog = FileDialog::builder()
            .title("Select Key File")
            .modal(true)
            .build();

        // Add filter for key files
        let filter = FileFilter::new();
        filter.add_pattern("*.keyx");
        filter.add_pattern("*.key");
        filter.set_name(Some("Key Files (*.keyx, *.key)"));

        let all_filter = FileFilter::new();
        all_filter.add_pattern("*");
        all_filter.set_name(Some("All Files"));

        let filters = gtk4::gio::ListStore::new::<FileFilter>();
        filters.append(&filter);
        filters.append(&all_filter);
        dialog.set_filters(Some(&filters));
        dialog.set_default_filter(Some(&filter));

        // Get the root window
        let root = button.root();
        let window = root.and_then(|r| r.downcast::<gtk4::Window>().ok());

        dialog.open(
            window.as_ref(),
            gtk4::gio::Cancellable::NONE,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        entry.set_text(&path.display().to_string());
                    }
                }
            },
        );
    });

    // Setup check connection button
    let kdbx_path_entry_check = kdbx_path_entry.clone();
    let kdbx_password_entry_check = kdbx_password_entry.clone();
    let kdbx_key_file_entry_check = kdbx_key_file_entry.clone();
    let kdbx_use_password_check_clone = kdbx_use_password_check.clone();
    let kdbx_use_key_file_check_clone = kdbx_use_key_file_check.clone();
    let kdbx_status_label_check = kdbx_status_label.clone();
    kdbx_check_button.connect_clicked(move |_| {
        let path_text = kdbx_path_entry_check.text();
        if path_text.is_empty() {
            update_status_label(&kdbx_status_label_check, "No database selected", "warning");
            return;
        }

        let kdbx_path = std::path::Path::new(path_text.as_str());

        // Get password if enabled
        let password = if kdbx_use_password_check_clone.is_active() {
            let pwd = kdbx_password_entry_check.text();
            if pwd.is_empty() {
                None
            } else {
                Some(pwd.to_string())
            }
        } else {
            None
        };

        // Get key file if enabled
        let key_file = if kdbx_use_key_file_check_clone.is_active() {
            let kf = kdbx_key_file_entry_check.text();
            if kf.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(kf.as_str()))
            }
        } else {
            None
        };

        // Verify credentials
        let result = rustconn_core::secret::KeePassStatus::verify_kdbx_credentials(
            kdbx_path,
            password.as_deref(),
            key_file.as_deref(),
        );

        match result {
            Ok(()) => {
                update_status_label(&kdbx_status_label_check, "Connected", "success");
            }
            Err(e) => {
                update_status_label(&kdbx_status_label_check, &e, "error");
            }
        }
    });

    let keepassxc_status_container = GtkBox::new(Orientation::Vertical, 6);

    SecretsPageWidgets {
        page,
        secret_backend_dropdown,
        enable_fallback,
        kdbx_path_entry,
        kdbx_password_entry,
        kdbx_enabled_switch,
        kdbx_save_password_check,
        kdbx_status_label,
        kdbx_browse_button,
        kdbx_check_button,
        keepassxc_status_container,
        kdbx_key_file_entry,
        kdbx_key_file_browse_button,
        kdbx_use_key_file_check,
        kdbx_use_password_check,
        kdbx_group,
        auth_group,
        status_group,
        password_row,
        save_password_row,
        key_file_row,
    }
}

/// Updates the status label with text and CSS class
fn update_status_label(label: &Label, text: &str, css_class: &str) {
    label.set_text(text);
    label.remove_css_class("success");
    label.remove_css_class("warning");
    label.remove_css_class("error");
    label.remove_css_class("dim-label");
    label.add_css_class(css_class);
}

/// Loads secret settings into UI controls
#[allow(clippy::too_many_arguments)]
pub fn load_secret_settings(widgets: &SecretsPageWidgets, settings: &SecretSettings) {
    let backend_index = match settings.preferred_backend {
        SecretBackendType::KeePassXc => 0,
        SecretBackendType::LibSecret => 1,
        SecretBackendType::KdbxFile => 2,
    };
    widgets.secret_backend_dropdown.set_selected(backend_index);
    widgets.enable_fallback.set_active(settings.enable_fallback);
    widgets
        .kdbx_enabled_switch
        .set_active(settings.kdbx_enabled);

    if let Some(path) = &settings.kdbx_path {
        widgets
            .kdbx_path_entry
            .set_text(&path.display().to_string());
    }

    if let Some(key_file) = &settings.kdbx_key_file {
        widgets
            .kdbx_key_file_entry
            .set_text(&key_file.display().to_string());
    }

    widgets
        .kdbx_use_password_check
        .set_active(settings.kdbx_use_password);
    widgets
        .kdbx_use_key_file_check
        .set_active(settings.kdbx_use_key_file);
    widgets
        .kdbx_save_password_check
        .set_active(settings.kdbx_password_encrypted.is_some());

    // Update visibility based on loaded settings
    widgets.auth_group.set_visible(settings.kdbx_enabled);
    widgets.status_group.set_visible(settings.kdbx_enabled);
    widgets.password_row.set_visible(settings.kdbx_use_password);
    widgets
        .save_password_row
        .set_visible(settings.kdbx_use_password);
    widgets.key_file_row.set_visible(settings.kdbx_use_key_file);

    let status_text = if settings.kdbx_enabled {
        if settings.kdbx_path.is_some() {
            "Configured"
        } else {
            "Database path required"
        }
    } else {
        "Disabled"
    };

    widgets.kdbx_status_label.set_text(status_text);

    widgets.kdbx_status_label.remove_css_class("success");
    widgets.kdbx_status_label.remove_css_class("warning");
    widgets.kdbx_status_label.remove_css_class("error");
    widgets.kdbx_status_label.remove_css_class("dim-label");

    let status_css_class = if settings.kdbx_enabled {
        if settings.kdbx_path.is_some() {
            "success"
        } else {
            "warning"
        }
    } else {
        "dim-label"
    };
    widgets.kdbx_status_label.add_css_class(status_css_class);
}

/// Collects secret settings from UI controls
pub fn collect_secret_settings(
    widgets: &SecretsPageWidgets,
    settings: &Rc<RefCell<rustconn_core::config::AppSettings>>,
) -> SecretSettings {
    let preferred_backend = match widgets.secret_backend_dropdown.selected() {
        0 => SecretBackendType::KeePassXc,
        1 => SecretBackendType::LibSecret,
        2 => SecretBackendType::KdbxFile,
        _ => SecretBackendType::default(),
    };

    let kdbx_path = {
        let path_text = widgets.kdbx_path_entry.text();
        if path_text.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(path_text.as_str()))
        }
    };

    let kdbx_key_file = {
        let key_file_text = widgets.kdbx_key_file_entry.text();
        if key_file_text.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(key_file_text.as_str()))
        }
    };

    let (kdbx_password, kdbx_password_encrypted) = if widgets.kdbx_save_password_check.is_active() {
        let password_text = widgets.kdbx_password_entry.text();
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
        enable_fallback: widgets.enable_fallback.is_active(),
        kdbx_path,
        kdbx_enabled: widgets.kdbx_enabled_switch.is_active(),
        kdbx_password,
        kdbx_password_encrypted,
        kdbx_key_file,
        kdbx_use_key_file: widgets.kdbx_use_key_file_check.is_active(),
        kdbx_use_password: widgets.kdbx_use_password_check.is_active(),
    }
}
