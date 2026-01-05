//! Logging settings tab

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, Entry, Grid, Label, Orientation, ScrolledWindow,
    SpinButton, Switch,
};
use rustconn_core::config::LoggingSettings;
use std::path::PathBuf;

/// Creates the logging settings tab
#[allow(clippy::type_complexity)]
pub fn create_logging_tab() -> (
    ScrolledWindow,
    Switch,
    Entry,
    SpinButton,
    CheckButton,
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

    // === Logging Settings section ===
    let settings_header = Label::builder()
        .label("Logging Settings")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();
    main_vbox.append(&settings_header);

    // Enable logging switch row
    let enable_row = GtkBox::new(Orientation::Horizontal, 12);
    enable_row.set_margin_start(6);
    enable_row.set_margin_top(6);
    let enable_label = Label::builder()
        .label("Persist logs")
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let logging_enabled_switch = Switch::builder().valign(gtk4::Align::Center).build();
    enable_row.append(&enable_label);
    enable_row.append(&logging_enabled_switch);
    main_vbox.append(&enable_row);

    // Logging mode checkboxes
    let mode_label = Label::builder()
        .label("Log content:")
        .halign(gtk4::Align::Start)
        .margin_top(12)
        .margin_start(6)
        .css_classes(["dim-label"])
        .build();
    main_vbox.append(&mode_label);

    let mode_box = GtkBox::new(Orientation::Vertical, 4);
    mode_box.set_margin_start(12);

    let log_activity_check = CheckButton::builder()
        .label("Activity (change counts)")
        .active(true)
        .sensitive(false)
        .build();
    let log_input_check = CheckButton::builder()
        .label("User input (commands)")
        .active(false)
        .sensitive(false)
        .build();
    let log_output_check = CheckButton::builder()
        .label("Terminal output (transcript)")
        .active(false)
        .sensitive(false)
        .build();

    mode_box.append(&log_activity_check);
    mode_box.append(&log_input_check);
    mode_box.append(&log_output_check);
    main_vbox.append(&mode_box);

    let grid = Grid::builder()
        .row_spacing(8)
        .column_spacing(12)
        .margin_top(12)
        .margin_start(6)
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
        .sensitive(false)
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
        .sensitive(false)
        .build();
    grid.attach(&retention_label, 0, row, 1, 1);
    grid.attach(&retention_spin, 1, row, 1, 1);

    main_vbox.append(&grid);

    // Open logs directory button
    let open_logs_btn = Button::builder()
        .label("Open Logs Directory")
        .halign(gtk4::Align::Start)
        .margin_top(8)
        .margin_start(6)
        .sensitive(false)
        .build();

    let log_dir_entry_clone = log_dir_entry.clone();
    open_logs_btn.connect_clicked(move |_| {
        let log_dir = log_dir_entry_clone.text();
        let log_path = if log_dir.starts_with('/') {
            PathBuf::from(log_dir.as_str())
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("rustconn")
                .join(log_dir.as_str())
        };

        if !log_path.exists() {
            if let Err(e) = std::fs::create_dir_all(&log_path) {
                tracing::error!("Failed to create logs directory: {e}");
                return;
            }
        }

        if let Err(e) = open::that(&log_path) {
            tracing::error!("Failed to open logs directory: {e}");
        }
    });

    main_vbox.append(&open_logs_btn);

    // Connect switch to enable/disable other controls
    let dir_entry_clone = log_dir_entry.clone();
    let retention_clone = retention_spin.clone();
    let open_logs_btn_clone = open_logs_btn.clone();
    let log_activity_clone = log_activity_check.clone();
    let log_input_clone = log_input_check.clone();
    let log_output_clone = log_output_check.clone();
    logging_enabled_switch.connect_state_set(move |_, state| {
        dir_entry_clone.set_sensitive(state);
        retention_clone.set_sensitive(state);
        open_logs_btn_clone.set_sensitive(state);
        log_activity_clone.set_sensitive(state);
        log_input_clone.set_sensitive(state);
        log_output_clone.set_sensitive(state);
        gtk4::glib::Propagation::Proceed
    });

    // === Log Format Example section ===
    let example_header = Label::builder()
        .label("Log Format Example")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(24)
        .build();
    main_vbox.append(&example_header);

    let example_desc = Label::builder()
        .label("Logs are stored in plain text format with timestamps:")
        .halign(gtk4::Align::Start)
        .css_classes(["dim-label"])
        .margin_start(6)
        .margin_top(6)
        .build();
    main_vbox.append(&example_desc);

    let example_text = r"[2026-01-05 10:23:45] SESSION_START server1.example.com (SSH)
[2026-01-05 10:23:46] CONNECTED user@server1.example.com:22
[2026-01-05 10:25:12] INPUT: ls -la /var/log
[2026-01-05 10:25:13] OUTPUT: total 1234...
[2026-01-05 10:30:00] SESSION_END duration=6m15s";

    let example_label = Label::builder()
        .label(example_text)
        .halign(gtk4::Align::Start)
        .css_classes(["monospace"])
        .selectable(true)
        .margin_start(6)
        .margin_top(6)
        .build();
    main_vbox.append(&example_label);

    let file_info = Label::builder()
        .label("Log files: <connection_name>_<date>.log")
        .halign(gtk4::Align::Start)
        .css_classes(["dim-label"])
        .margin_start(6)
        .margin_top(6)
        .build();
    main_vbox.append(&file_info);

    scrolled.set_child(Some(&main_vbox));

    (
        scrolled,
        logging_enabled_switch,
        log_dir_entry,
        retention_spin,
        log_activity_check,
        log_input_check,
        log_output_check,
    )
}

/// Loads logging settings into UI controls
pub fn load_logging_settings(
    logging_enabled_switch: &Switch,
    log_dir_entry: &Entry,
    retention_spin: &SpinButton,
    log_activity_check: &CheckButton,
    log_input_check: &CheckButton,
    log_output_check: &CheckButton,
    settings: &LoggingSettings,
) {
    logging_enabled_switch.set_active(settings.enabled);
    log_dir_entry.set_text(&settings.log_directory.to_string_lossy());
    retention_spin.set_value(f64::from(settings.retention_days));
    log_activity_check.set_active(settings.log_activity);
    log_input_check.set_active(settings.log_input);
    log_output_check.set_active(settings.log_output);
}

/// Collects logging settings from UI controls
pub fn collect_logging_settings(
    logging_enabled_switch: &Switch,
    log_dir_entry: &Entry,
    retention_spin: &SpinButton,
    log_activity_check: &CheckButton,
    log_input_check: &CheckButton,
    log_output_check: &CheckButton,
) -> LoggingSettings {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    LoggingSettings {
        enabled: logging_enabled_switch.is_active(),
        log_directory: PathBuf::from(log_dir_entry.text().as_str()),
        retention_days: retention_spin.value() as u32,
        log_activity: log_activity_check.is_active(),
        log_input: log_input_check.is_active(),
        log_output: log_output_check.is_active(),
    }
}
