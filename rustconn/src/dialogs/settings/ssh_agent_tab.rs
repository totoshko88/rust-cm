//! SSH Agent settings tab

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, ListBox, Orientation, ScrolledWindow, Spinner};
use rustconn_core::ssh_agent::SshAgentManager;
use std::cell::RefCell;
use std::rc::Rc;

/// Creates the SSH Agent settings tab
#[allow(clippy::type_complexity)]
pub fn create_ssh_agent_tab() -> (
    ScrolledWindow,
    Label,
    Label,
    Button,
    ListBox,
    Button,
    Spinner,
    Label,
    Button,
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

    // === Agent Status section ===
    let status_header = Label::builder()
        .label("Agent Status")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();
    main_vbox.append(&status_header);

    let status_grid = gtk4::Grid::builder()
        .row_spacing(4)
        .column_spacing(12)
        .margin_start(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    let status_label = Label::builder()
        .label("Status:")
        .halign(gtk4::Align::Start)
        .build();

    let ssh_agent_status_label = Label::builder()
        .label("Checking...")
        .halign(gtk4::Align::Start)
        .build();

    let socket_label = Label::builder()
        .label("Socket:")
        .halign(gtk4::Align::Start)
        .build();

    let ssh_agent_socket_label = Label::builder()
        .label("")
        .halign(gtk4::Align::Start)
        .css_classes(["dim-label"])
        .selectable(true)
        .build();

    status_grid.attach(&status_label, 0, 0, 1, 1);
    status_grid.attach(&ssh_agent_status_label, 1, 0, 1, 1);
    status_grid.attach(&socket_label, 0, 1, 1, 1);
    status_grid.attach(&ssh_agent_socket_label, 1, 1, 1, 1);

    main_vbox.append(&status_grid);

    // Control buttons
    let control_hbox = GtkBox::new(Orientation::Horizontal, 6);
    control_hbox.set_margin_start(6);
    control_hbox.set_margin_bottom(12);

    let ssh_agent_start_button = Button::with_label("Start Agent");
    let ssh_agent_refresh_button = Button::with_label("Refresh");

    control_hbox.append(&ssh_agent_start_button);
    control_hbox.append(&ssh_agent_refresh_button);
    main_vbox.append(&control_hbox);

    // === Loaded Keys section ===
    let keys_header = Label::builder()
        .label("Loaded Keys")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(6)
        .build();
    main_vbox.append(&keys_header);

    // Keys list without ScrolledWindow wrapper - just plain ListBox
    let ssh_agent_keys_list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .build();
    main_vbox.append(&ssh_agent_keys_list);

    let ssh_agent_loading_spinner = Spinner::new();
    main_vbox.append(&ssh_agent_loading_spinner);

    let ssh_agent_error_label = Label::builder()
        .label("")
        .halign(gtk4::Align::Start)
        .css_classes(["error"])
        .build();
    main_vbox.append(&ssh_agent_error_label);

    // Add Key button
    let add_key_hbox = GtkBox::new(Orientation::Horizontal, 6);
    add_key_hbox.set_margin_top(6);

    let ssh_agent_add_key_button = Button::with_label("Add Key");
    add_key_hbox.append(&ssh_agent_add_key_button);
    main_vbox.append(&add_key_hbox);

    // === Available Key Files section ===
    let available_header = Label::builder()
        .label("Available Key Files")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&available_header);

    let available_desc = Label::builder()
        .label("Key files found in ~/.ssh/")
        .halign(gtk4::Align::Start)
        .css_classes(["dim-label"])
        .margin_start(6)
        .margin_bottom(6)
        .build();
    main_vbox.append(&available_desc);

    // Available keys list without ScrolledWindow
    let available_keys_list = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    if let Ok(key_files) = SshAgentManager::list_key_files() {
        if key_files.is_empty() {
            let empty_row = create_empty_row("No SSH key files found");
            available_keys_list.append(&empty_row);
        } else {
            for key_file in key_files {
                let key_row = create_available_key_row(&key_file);
                available_keys_list.append(&key_row);
            }
        }
    } else {
        let error_row = create_error_row("Failed to scan ~/.ssh/ directory");
        available_keys_list.append(&error_row);
    }

    main_vbox.append(&available_keys_list);

    scrolled.set_child(Some(&main_vbox));

    (
        scrolled,
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
            "✓ Running"
        } else {
            "○ Not running"
        };
        ssh_agent_status_label.set_text(status_text);

        if status.running {
            ssh_agent_status_label.remove_css_class("error");
            ssh_agent_status_label.add_css_class("success");
        } else {
            ssh_agent_status_label.remove_css_class("success");
            ssh_agent_status_label.add_css_class("dim-label");
        }
    } else {
        ssh_agent_status_label.set_text("✗ Error");
        ssh_agent_status_label.add_css_class("error");
    }

    if let Some(socket) = manager.socket_path() {
        ssh_agent_socket_label.set_text(socket);
    } else {
        ssh_agent_socket_label.set_text("Not available");
    }

    while let Some(child) = ssh_agent_keys_list.first_child() {
        ssh_agent_keys_list.remove(&child);
    }

    if let Ok(status) = manager.get_status() {
        if status.running {
            if status.keys.is_empty() {
                let empty_row = create_empty_row("No keys loaded in agent");
                ssh_agent_keys_list.append(&empty_row);
            } else {
                for key in &status.keys {
                    let key_row = create_loaded_key_row(key);
                    ssh_agent_keys_list.append(&key_row);
                }
            }
        } else {
            let empty_row = create_empty_row("Agent not running");
            ssh_agent_keys_list.append(&empty_row);
        }
    } else {
        let empty_row = create_empty_row("Agent not running");
        ssh_agent_keys_list.append(&empty_row);
    }
}

fn create_loaded_key_row(key: &rustconn_core::ssh_agent::AgentKey) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();

    let key_box = GtkBox::new(Orientation::Horizontal, 12);
    key_box.set_margin_top(4);
    key_box.set_margin_bottom(4);
    key_box.set_margin_start(6);
    key_box.set_margin_end(6);

    let key_info = GtkBox::new(Orientation::Vertical, 2);
    key_info.set_hexpand(true);

    let key_type_label = Label::builder()
        .label(&format!("{} ({} bits)", key.key_type, key.bits))
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();

    let fingerprint_label = Label::builder()
        .label(&format!("SHA256:{}", key.fingerprint))
        .halign(gtk4::Align::Start)
        .css_classes(["dim-label", "caption"])
        .ellipsize(gtk4::pango::EllipsizeMode::Middle)
        .build();

    let comment_text = if key.comment.is_empty() {
        "No comment"
    } else {
        &key.comment
    };
    let comment_label = Label::builder()
        .label(comment_text)
        .halign(gtk4::Align::Start)
        .css_classes(["dim-label"])
        .build();

    key_info.append(&key_type_label);
    key_info.append(&fingerprint_label);
    key_info.append(&comment_label);

    let remove_button = Button::builder()
        .label("Remove")
        .css_classes(["destructive-action"])
        .valign(gtk4::Align::Center)
        .build();

    let fingerprint = key.fingerprint.clone();
    remove_button.connect_clicked(move |_| {
        tracing::info!("Remove key requested: {}", fingerprint);
    });

    key_box.append(&key_info);
    key_box.append(&remove_button);

    row.set_child(Some(&key_box));
    row
}

fn create_available_key_row(key_path: &std::path::Path) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();

    let key_box = GtkBox::new(Orientation::Horizontal, 12);
    key_box.set_margin_top(2);
    key_box.set_margin_bottom(2);
    key_box.set_margin_start(6);
    key_box.set_margin_end(6);

    let key_name = key_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");

    let key_label = Label::builder()
        .label(key_name)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    let path_label = Label::builder()
        .label(&key_path.display().to_string())
        .halign(gtk4::Align::End)
        .css_classes(["dim-label"])
        .ellipsize(gtk4::pango::EllipsizeMode::Start)
        .build();

    key_box.append(&key_label);
    key_box.append(&path_label);

    row.set_child(Some(&key_box));
    row
}

fn create_empty_row(message: &str) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();

    let label = Label::builder()
        .label(message)
        .halign(gtk4::Align::Start)
        .css_classes(["dim-label"])
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    row.set_child(Some(&label));
    row
}

fn create_error_row(message: &str) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();

    let label = Label::builder()
        .label(message)
        .halign(gtk4::Align::Start)
        .css_classes(["error"])
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    row.set_child(Some(&label));
    row
}
