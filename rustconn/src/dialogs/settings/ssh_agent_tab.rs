//! SSH Agent settings tab using libadwaita components

use adw::prelude::*;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, ListBox, Orientation, Spinner};
use libadwaita as adw;
use rustconn_core::ssh_agent::SshAgentManager;
use std::cell::RefCell;
use std::rc::Rc;

/// Creates the SSH Agent settings page using AdwPreferencesPage
#[allow(clippy::type_complexity)]
pub fn create_ssh_agent_page() -> (
    adw::PreferencesPage,
    Label,
    Label,
    Button,
    ListBox,
    Button,
    Spinner,
    Label,
    Button,
) {
    let page = adw::PreferencesPage::builder()
        .title("SSH Agent")
        .icon_name("network-server-symbolic")
        .build();

    // === Agent Status Group ===
    let status_group = adw::PreferencesGroup::builder()
        .title("Agent Status")
        .build();

    let ssh_agent_status_label = Label::builder()
        .label("Checking...")
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .build();
    let status_row = adw::ActionRow::builder().title("Status").build();
    status_row.add_suffix(&ssh_agent_status_label);
    status_group.add(&status_row);

    let ssh_agent_socket_label = Label::builder()
        .label("")
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .css_classes(["dim-label"])
        .selectable(true)
        .ellipsize(gtk4::pango::EllipsizeMode::Middle)
        .max_width_chars(40)
        .build();
    let socket_row = adw::ActionRow::builder().title("Socket").build();
    socket_row.add_suffix(&ssh_agent_socket_label);
    status_group.add(&socket_row);

    // Control buttons row
    let ssh_agent_start_button = Button::builder()
        .label("Start Agent")
        .valign(gtk4::Align::Center)
        .build();
    let ssh_agent_refresh_button = Button::builder()
        .icon_name("view-refresh-symbolic")
        .valign(gtk4::Align::Center)
        .tooltip_text("Refresh status")
        .build();

    let buttons_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .build();
    buttons_box.append(&ssh_agent_start_button);
    buttons_box.append(&ssh_agent_refresh_button);

    let control_row = adw::ActionRow::builder().title("Controls").build();
    control_row.add_suffix(&buttons_box);
    status_group.add(&control_row);

    page.add(&status_group);

    // === Loaded Keys Group ===
    let keys_group = adw::PreferencesGroup::builder()
        .title("Loaded Keys")
        .description("Keys currently loaded in the SSH agent")
        .build();

    let ssh_agent_keys_list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    keys_group.add(&ssh_agent_keys_list);

    let ssh_agent_loading_spinner = Spinner::new();
    let ssh_agent_error_label = Label::builder()
        .label("")
        .halign(gtk4::Align::Start)
        .css_classes(["error"])
        .build();

    // Add Key button
    let ssh_agent_add_key_button = Button::builder()
        .label("Add Key")
        .valign(gtk4::Align::Center)
        .css_classes(["suggested-action"])
        .build();
    let add_key_row = adw::ActionRow::builder()
        .title("Add SSH Key")
        .subtitle("Load a key from file")
        .activatable(true)
        .build();
    add_key_row.add_suffix(&ssh_agent_add_key_button);
    keys_group.add(&add_key_row);

    page.add(&keys_group);

    // === Available Key Files Group ===
    let available_group = adw::PreferencesGroup::builder()
        .title("Available Key Files")
        .description("Key files found in ~/.ssh/")
        .build();

    let available_keys_list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();

    if let Ok(key_files) = SshAgentManager::list_key_files() {
        if key_files.is_empty() {
            let empty_row = adw::ActionRow::builder()
                .title("No SSH key files found")
                .subtitle("Generate keys with ssh-keygen")
                .build();
            available_keys_list.append(&empty_row);
        } else {
            for key_file in key_files {
                let key_name = key_file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let key_path = key_file.display().to_string();

                let key_row = adw::ActionRow::builder()
                    .title(&key_name)
                    .subtitle(&key_path)
                    .build();

                let load_button = Button::builder()
                    .icon_name("list-add-symbolic")
                    .valign(gtk4::Align::Center)
                    .tooltip_text("Load this key")
                    .build();
                key_row.add_suffix(&load_button);

                available_keys_list.append(&key_row);
            }
        }
    } else {
        let error_row = adw::ActionRow::builder()
            .title("Failed to scan ~/.ssh/ directory")
            .build();
        error_row.add_css_class("error");
        available_keys_list.append(&error_row);
    }

    available_group.add(&available_keys_list);
    page.add(&available_group);

    (
        page,
        ssh_agent_status_label,
        ssh_agent_socket_label,
        ssh_agent_start_button,
        ssh_agent_keys_list,
        ssh_agent_add_key_button,
        ssh_agent_loading_spinner,
        ssh_agent_error_label,
        ssh_agent_refresh_button,
    )
}

/// Loads SSH agent settings into UI controls
pub fn load_ssh_agent_settings(
    ssh_agent_status_label: &Label,
    ssh_agent_socket_label: &Label,
    ssh_agent_keys_list: &ListBox,
    ssh_agent_manager: &Rc<RefCell<SshAgentManager>>,
) {
    let manager = ssh_agent_manager.borrow();

    if let Ok(status) = manager.get_status() {
        let status_text = if status.running {
            "Running"
        } else {
            "Not running"
        };
        ssh_agent_status_label.set_text(status_text);

        ssh_agent_status_label.remove_css_class("error");
        ssh_agent_status_label.remove_css_class("success");
        ssh_agent_status_label.remove_css_class("dim-label");

        if status.running {
            ssh_agent_status_label.add_css_class("success");
        } else {
            ssh_agent_status_label.add_css_class("dim-label");
        }
    } else {
        ssh_agent_status_label.set_text("Error");
        ssh_agent_status_label.add_css_class("error");
    }

    if let Some(socket) = manager.socket_path() {
        ssh_agent_socket_label.set_text(socket);
    } else {
        ssh_agent_socket_label.set_text("Not available");
    }

    // Clear existing keys
    while let Some(child) = ssh_agent_keys_list.first_child() {
        ssh_agent_keys_list.remove(&child);
    }

    if let Ok(status) = manager.get_status() {
        if status.running {
            if status.keys.is_empty() {
                let empty_row = adw::ActionRow::builder()
                    .title("No keys loaded")
                    .subtitle("Add keys using ssh-add or the button above")
                    .build();
                ssh_agent_keys_list.append(&empty_row);
            } else {
                for key in &status.keys {
                    let key_row = create_loaded_key_row(key);
                    ssh_agent_keys_list.append(&key_row);
                }
            }
        } else {
            let empty_row = adw::ActionRow::builder()
                .title("Agent not running")
                .subtitle("Start the agent to manage keys")
                .build();
            ssh_agent_keys_list.append(&empty_row);
        }
    } else {
        let empty_row = adw::ActionRow::builder()
            .title("Agent not running")
            .subtitle("Start the agent to manage keys")
            .build();
        ssh_agent_keys_list.append(&empty_row);
    }
}

fn create_loaded_key_row(key: &rustconn_core::ssh_agent::AgentKey) -> adw::ActionRow {
    let title = format!("{} ({} bits)", key.key_type, key.bits);
    let subtitle = if key.comment.is_empty() {
        format!("SHA256:{}", key.fingerprint)
    } else {
        format!("{} • SHA256:{}", key.comment, key.fingerprint)
    };

    let row = adw::ActionRow::builder()
        .title(&title)
        .subtitle(&subtitle)
        .build();

    let remove_button = Button::builder()
        .icon_name("user-trash-symbolic")
        .valign(gtk4::Align::Center)
        .tooltip_text("Remove from agent")
        .css_classes(["destructive-action", "flat"])
        .build();

    let fingerprint = key.fingerprint.clone();
    remove_button.connect_clicked(move |_| {
        tracing::info!("Remove key requested: {}", fingerprint);
    });

    row.add_suffix(&remove_button);
    row
}
