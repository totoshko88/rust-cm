//! UI settings tab

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, CheckButton, Label, Orientation, ScrolledWindow, SpinButton, ToggleButton,
};
use rustconn_core::config::{ColorScheme, SessionRestoreSettings, UiSettings};

/// Creates the UI settings tab
#[allow(clippy::type_complexity)]
pub fn create_ui_tab() -> (
    ScrolledWindow,
    GtkBox,
    CheckButton,
    CheckButton,
    CheckButton,
    CheckButton,
    CheckButton,
    SpinButton,
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

    // === Appearance section ===
    let appearance_header = Label::builder()
        .label("Appearance")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();
    main_vbox.append(&appearance_header);

    let color_scheme_label = Label::builder()
        .label("Color scheme:")
        .halign(gtk4::Align::Start)
        .margin_start(6)
        .margin_top(6)
        .build();
    main_vbox.append(&color_scheme_label);

    // Button group for color scheme selection
    let color_scheme_box = GtkBox::new(Orientation::Horizontal, 0);
    color_scheme_box.add_css_class("linked");
    color_scheme_box.set_halign(gtk4::Align::Start);
    color_scheme_box.set_margin_start(6);
    color_scheme_box.set_margin_top(6);

    let system_btn = ToggleButton::with_label("System");
    let light_btn = ToggleButton::with_label("Light");
    let dark_btn = ToggleButton::with_label("Dark");

    light_btn.set_group(Some(&system_btn));
    dark_btn.set_group(Some(&system_btn));
    system_btn.set_active(true);

    system_btn.connect_toggled(move |btn| {
        if btn.is_active() {
            crate::app::apply_color_scheme(ColorScheme::System);
        }
    });

    light_btn.connect_toggled(move |btn| {
        if btn.is_active() {
            crate::app::apply_color_scheme(ColorScheme::Light);
        }
    });

    dark_btn.connect_toggled(move |btn| {
        if btn.is_active() {
            crate::app::apply_color_scheme(ColorScheme::Dark);
        }
    });

    color_scheme_box.append(&system_btn);
    color_scheme_box.append(&light_btn);
    color_scheme_box.append(&dark_btn);
    main_vbox.append(&color_scheme_box);

    // === Window section ===
    let window_header = Label::builder()
        .label("Window")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&window_header);

    let remember_geometry = CheckButton::with_label("Remember window size and position");
    remember_geometry.set_margin_start(6);
    remember_geometry.set_margin_top(6);
    main_vbox.append(&remember_geometry);

    // === System Tray section ===
    let tray_header = Label::builder()
        .label("System Tray")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&tray_header);

    let enable_tray_icon = CheckButton::with_label("Show icon in system tray");
    enable_tray_icon.set_margin_start(6);
    enable_tray_icon.set_margin_top(6);
    main_vbox.append(&enable_tray_icon);

    let minimize_to_tray = CheckButton::with_label("Minimize to tray when closing window");
    minimize_to_tray.set_margin_start(6);
    main_vbox.append(&minimize_to_tray);

    let tray_note = Label::builder()
        .label("Note: Tray icon requires desktop environment with system tray support.")
        .wrap(true)
        .css_classes(["dim-label"])
        .halign(gtk4::Align::Start)
        .margin_start(6)
        .margin_top(6)
        .build();
    main_vbox.append(&tray_note);

    // === Session Restore section ===
    let session_header = Label::builder()
        .label("Session Restore")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&session_header);

    let session_restore_enabled = CheckButton::with_label("Restore sessions on start-up");
    session_restore_enabled.set_margin_start(6);
    session_restore_enabled.set_margin_top(6);
    main_vbox.append(&session_restore_enabled);

    let prompt_on_restore = CheckButton::with_label("Ask before restoring sessions");
    prompt_on_restore.set_margin_start(6);
    main_vbox.append(&prompt_on_restore);

    let max_age_hbox = GtkBox::new(Orientation::Horizontal, 6);
    max_age_hbox.set_margin_start(6);
    max_age_hbox.set_margin_top(6);
    max_age_hbox.append(&Label::new(Some("Maximum session age (hours):")));
    let max_age_spin = SpinButton::new(
        Some(&gtk4::Adjustment::new(24.0, 1.0, 168.0, 1.0, 24.0, 0.0)),
        1.0,
        0,
    );
    max_age_hbox.append(&max_age_spin);
    main_vbox.append(&max_age_hbox);

    let session_note = Label::builder()
        .label("Sessions older than the specified age will be discarded.")
        .wrap(true)
        .css_classes(["dim-label"])
        .halign(gtk4::Align::Start)
        .margin_start(6)
        .margin_top(6)
        .build();
    main_vbox.append(&session_note);

    scrolled.set_child(Some(&main_vbox));

    (
        scrolled,
        color_scheme_box,
        remember_geometry,
        enable_tray_icon,
        minimize_to_tray,
        session_restore_enabled,
        prompt_on_restore,
        max_age_spin,
    )
}

/// Loads UI settings into UI controls
#[allow(clippy::too_many_arguments)]
pub fn load_ui_settings(
    color_scheme_box: &GtkBox,
    remember_geometry: &CheckButton,
    enable_tray_icon: &CheckButton,
    minimize_to_tray: &CheckButton,
    session_restore_enabled: &CheckButton,
    prompt_on_restore: &CheckButton,
    max_age_spin: &SpinButton,
    settings: &UiSettings,
) {
    let target_index = match settings.color_scheme {
        ColorScheme::System => 0,
        ColorScheme::Light => 1,
        ColorScheme::Dark => 2,
    };

    let mut child = color_scheme_box.first_child();
    let mut index = 0;
    while let Some(widget) = child {
        if let Some(btn) = widget.downcast_ref::<ToggleButton>() {
            if index == target_index {
                btn.set_active(true);
                crate::app::apply_color_scheme(settings.color_scheme);
                break;
            }
        }
        child = widget.next_sibling();
        index += 1;
    }

    remember_geometry.set_active(settings.remember_window_geometry);
    enable_tray_icon.set_active(settings.enable_tray_icon);
    minimize_to_tray.set_active(settings.minimize_to_tray);
    session_restore_enabled.set_active(settings.session_restore.enabled);
    prompt_on_restore.set_active(settings.session_restore.prompt_on_restore);
    max_age_spin.set_value(f64::from(settings.session_restore.max_age_hours));
}

/// Collects UI settings from UI controls
#[allow(clippy::too_many_arguments)]
pub fn collect_ui_settings(
    color_scheme_box: &GtkBox,
    remember_geometry: &CheckButton,
    enable_tray_icon: &CheckButton,
    minimize_to_tray: &CheckButton,
    session_restore_enabled: &CheckButton,
    prompt_on_restore: &CheckButton,
    max_age_spin: &SpinButton,
) -> UiSettings {
    let mut selected_scheme = ColorScheme::System;
    let mut child = color_scheme_box.first_child();
    let mut index = 0;
    while let Some(widget) = child {
        if let Some(btn) = widget.downcast_ref::<ToggleButton>() {
            if btn.is_active() {
                selected_scheme = match index {
                    0 => ColorScheme::System,
                    1 => ColorScheme::Light,
                    2 => ColorScheme::Dark,
                    _ => ColorScheme::System,
                };
                break;
            }
        }
        child = widget.next_sibling();
        index += 1;
    }

    UiSettings {
        color_scheme: selected_scheme,
        remember_window_geometry: remember_geometry.is_active(),
        window_width: None,
        window_height: None,
        sidebar_width: None,
        enable_tray_icon: enable_tray_icon.is_active(),
        minimize_to_tray: minimize_to_tray.is_active(),
        expanded_groups: std::collections::HashSet::new(),
        session_restore: SessionRestoreSettings {
            enabled: session_restore_enabled.is_active(),
            prompt_on_restore: prompt_on_restore.is_active(),
            #[allow(clippy::cast_sign_loss)]
            max_age_hours: max_age_spin.value().max(0.0) as u32,
            saved_sessions: Vec::new(),
        },
    }
}
