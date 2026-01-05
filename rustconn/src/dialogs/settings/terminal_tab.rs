//! Terminal settings tab

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, CheckButton, DropDown, Entry, Label, Orientation, ScrolledWindow, SpinButton,
    StringList,
};
use rustconn_core::config::TerminalSettings;
use rustconn_core::terminal_themes::TerminalTheme;

/// Creates the terminal settings tab
#[allow(clippy::type_complexity)]
pub fn create_terminal_tab() -> (
    ScrolledWindow,
    Entry,
    SpinButton,
    SpinButton,
    DropDown,
    DropDown,
    DropDown,
    CheckButton,
    CheckButton,
    CheckButton,
    CheckButton,
    CheckButton,
) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let main_vbox = GtkBox::new(Orientation::Vertical, 6);
    main_vbox.set_margin_top(12);
    main_vbox.set_margin_bottom(12);
    main_vbox.set_margin_start(12);
    main_vbox.set_margin_end(12);
    main_vbox.set_valign(gtk4::Align::Start);

    // === Font Settings ===
    let font_header = Label::builder()
        .label("Font")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();
    main_vbox.append(&font_header);

    // Font family row
    let font_family_hbox = GtkBox::new(Orientation::Horizontal, 12);
    font_family_hbox.set_margin_start(6);
    font_family_hbox.set_margin_top(6);
    let font_family_label = Label::new(Some("Font Family:"));
    font_family_label.set_size_request(120, -1);
    font_family_label.set_halign(gtk4::Align::Start);

    let font_family_entry = Entry::builder().hexpand(true).text("Monospace").build();

    font_family_hbox.append(&font_family_label);
    font_family_hbox.append(&font_family_entry);
    main_vbox.append(&font_family_hbox);

    // Font size row
    let font_size_hbox = GtkBox::new(Orientation::Horizontal, 12);
    font_size_hbox.set_margin_start(6);
    let font_size_label = Label::new(Some("Font Size:"));
    font_size_label.set_size_request(120, -1);
    font_size_label.set_halign(gtk4::Align::Start);

    let size_adj = gtk4::Adjustment::new(12.0, 6.0, 72.0, 1.0, 2.0, 0.0);
    let font_size_spin = SpinButton::builder()
        .adjustment(&size_adj)
        .climb_rate(1.0)
        .digits(0)
        .build();

    font_size_hbox.append(&font_size_label);
    font_size_hbox.append(&font_size_spin);
    main_vbox.append(&font_size_hbox);

    // === Color Theme ===
    let color_header = Label::builder()
        .label("Colors")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&color_header);

    // Color theme dropdown
    let theme_hbox = GtkBox::new(Orientation::Horizontal, 12);
    theme_hbox.set_margin_start(6);
    theme_hbox.set_margin_top(6);
    let theme_label = Label::new(Some("Color Theme:"));
    theme_label.set_size_request(120, -1);
    theme_label.set_halign(gtk4::Align::Start);

    let theme_names = TerminalTheme::theme_names();
    let theme_list = StringList::new(&theme_names.iter().map(String::as_str).collect::<Vec<_>>());
    let color_theme_dropdown = DropDown::builder()
        .model(&theme_list)
        .selected(0)
        .hexpand(true)
        .build();

    theme_hbox.append(&theme_label);
    theme_hbox.append(&color_theme_dropdown);
    main_vbox.append(&theme_hbox);

    // === Cursor Settings ===
    let cursor_header = Label::builder()
        .label("Cursor")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&cursor_header);

    // Cursor shape
    let cursor_shape_hbox = GtkBox::new(Orientation::Horizontal, 12);
    cursor_shape_hbox.set_margin_start(6);
    cursor_shape_hbox.set_margin_top(6);
    let cursor_shape_label = Label::new(Some("Cursor Shape:"));
    cursor_shape_label.set_size_request(120, -1);
    cursor_shape_label.set_halign(gtk4::Align::Start);

    let cursor_shapes = ["Block", "IBeam", "Underline"];
    let cursor_shape_list = StringList::new(&cursor_shapes);
    let cursor_shape_dropdown = DropDown::builder()
        .model(&cursor_shape_list)
        .selected(0)
        .hexpand(true)
        .build();

    cursor_shape_hbox.append(&cursor_shape_label);
    cursor_shape_hbox.append(&cursor_shape_dropdown);
    main_vbox.append(&cursor_shape_hbox);

    // Cursor blink
    let cursor_blink_hbox = GtkBox::new(Orientation::Horizontal, 12);
    cursor_blink_hbox.set_margin_start(6);
    let cursor_blink_label = Label::new(Some("Cursor Blink:"));
    cursor_blink_label.set_size_request(120, -1);
    cursor_blink_label.set_halign(gtk4::Align::Start);

    let cursor_blink_modes = ["On", "Off", "System"];
    let cursor_blink_list = StringList::new(&cursor_blink_modes);
    let cursor_blink_dropdown = DropDown::builder()
        .model(&cursor_blink_list)
        .selected(0)
        .hexpand(true)
        .build();

    cursor_blink_hbox.append(&cursor_blink_label);
    cursor_blink_hbox.append(&cursor_blink_dropdown);
    main_vbox.append(&cursor_blink_hbox);

    // === Scrolling Settings ===
    let scroll_header = Label::builder()
        .label("Scrolling")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&scroll_header);

    // Scrollback lines
    let scrollback_hbox = GtkBox::new(Orientation::Horizontal, 12);
    scrollback_hbox.set_margin_start(6);
    scrollback_hbox.set_margin_top(6);
    let scrollback_label = Label::new(Some("Scrollback Lines:"));
    scrollback_label.set_size_request(120, -1);
    scrollback_label.set_halign(gtk4::Align::Start);

    let scrollback_adj = gtk4::Adjustment::new(10000.0, 100.0, 1_000_000.0, 100.0, 1000.0, 0.0);
    let scrollback_spin = SpinButton::builder()
        .adjustment(&scrollback_adj)
        .climb_rate(100.0)
        .digits(0)
        .build();

    scrollback_hbox.append(&scrollback_label);
    scrollback_hbox.append(&scrollback_spin);
    main_vbox.append(&scrollback_hbox);

    // Scroll checkboxes
    let scroll_checks_box = GtkBox::new(Orientation::Vertical, 4);
    scroll_checks_box.set_margin_start(6);
    scroll_checks_box.set_margin_top(6);

    let scroll_on_output_check = CheckButton::builder()
        .label("Scroll on output")
        .active(false)
        .build();
    scroll_checks_box.append(&scroll_on_output_check);

    let scroll_on_keystroke_check = CheckButton::builder()
        .label("Scroll on keystroke")
        .active(true)
        .build();
    scroll_checks_box.append(&scroll_on_keystroke_check);

    main_vbox.append(&scroll_checks_box);

    // === Behavior Settings ===
    let behavior_header = Label::builder()
        .label("Behavior")
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .margin_top(18)
        .build();
    main_vbox.append(&behavior_header);

    let behavior_checks_box = GtkBox::new(Orientation::Vertical, 4);
    behavior_checks_box.set_margin_start(6);
    behavior_checks_box.set_margin_top(6);

    let allow_hyperlinks_check = CheckButton::builder()
        .label("Allow hyperlinks")
        .active(true)
        .build();
    behavior_checks_box.append(&allow_hyperlinks_check);

    let mouse_autohide_check = CheckButton::builder()
        .label("Hide mouse when typing")
        .active(true)
        .build();
    behavior_checks_box.append(&mouse_autohide_check);

    let audible_bell_check = CheckButton::builder()
        .label("Audible bell")
        .active(false)
        .build();
    behavior_checks_box.append(&audible_bell_check);

    main_vbox.append(&behavior_checks_box);

    scrolled.set_child(Some(&main_vbox));

    (
        scrolled,
        font_family_entry,
        font_size_spin,
        scrollback_spin,
        color_theme_dropdown,
        cursor_shape_dropdown,
        cursor_blink_dropdown,
        scroll_on_output_check,
        scroll_on_keystroke_check,
        allow_hyperlinks_check,
        mouse_autohide_check,
        audible_bell_check,
    )
}

/// Loads terminal settings into UI controls
#[allow(clippy::too_many_arguments)]
pub fn load_terminal_settings(
    font_family_entry: &Entry,
    font_size_spin: &SpinButton,
    scrollback_spin: &SpinButton,
    color_theme_dropdown: &DropDown,
    cursor_shape_dropdown: &DropDown,
    cursor_blink_dropdown: &DropDown,
    scroll_on_output_check: &CheckButton,
    scroll_on_keystroke_check: &CheckButton,
    allow_hyperlinks_check: &CheckButton,
    mouse_autohide_check: &CheckButton,
    audible_bell_check: &CheckButton,
    settings: &TerminalSettings,
) {
    font_family_entry.set_text(&settings.font_family);
    font_size_spin.set_value(f64::from(settings.font_size));
    scrollback_spin.set_value(f64::from(settings.scrollback_lines));

    // Set color theme
    let theme_names = TerminalTheme::theme_names();
    if let Some(index) = theme_names
        .iter()
        .position(|name| name == &settings.color_theme)
    {
        color_theme_dropdown.set_selected(index as u32);
    }

    // Set cursor shape
    let cursor_shape_index = match settings.cursor_shape.as_str() {
        "Block" => 0,
        "IBeam" => 1,
        "Underline" => 2,
        _ => 0,
    };
    cursor_shape_dropdown.set_selected(cursor_shape_index);

    // Set cursor blink
    let cursor_blink_index = match settings.cursor_blink.as_str() {
        "On" => 0,
        "Off" => 1,
        "System" => 2,
        _ => 0,
    };
    cursor_blink_dropdown.set_selected(cursor_blink_index);

    scroll_on_output_check.set_active(settings.scroll_on_output);
    scroll_on_keystroke_check.set_active(settings.scroll_on_keystroke);
    allow_hyperlinks_check.set_active(settings.allow_hyperlinks);
    mouse_autohide_check.set_active(settings.mouse_autohide);
    audible_bell_check.set_active(settings.audible_bell);
}

/// Collects terminal settings from UI controls
#[allow(clippy::too_many_arguments)]
pub fn collect_terminal_settings(
    font_family_entry: &Entry,
    font_size_spin: &SpinButton,
    scrollback_spin: &SpinButton,
    color_theme_dropdown: &DropDown,
    cursor_shape_dropdown: &DropDown,
    cursor_blink_dropdown: &DropDown,
    scroll_on_output_check: &CheckButton,
    scroll_on_keystroke_check: &CheckButton,
    allow_hyperlinks_check: &CheckButton,
    mouse_autohide_check: &CheckButton,
    audible_bell_check: &CheckButton,
) -> TerminalSettings {
    let theme_names = TerminalTheme::theme_names();
    let color_theme = theme_names
        .get(color_theme_dropdown.selected() as usize)
        .cloned()
        .unwrap_or_else(|| "Dark".to_string());

    let cursor_shapes = ["Block", "IBeam", "Underline"];
    let cursor_shape = cursor_shapes
        .get(cursor_shape_dropdown.selected() as usize)
        .unwrap_or(&"Block")
        .to_string();

    let cursor_blink_modes = ["On", "Off", "System"];
    let cursor_blink_mode = cursor_blink_modes
        .get(cursor_blink_dropdown.selected() as usize)
        .unwrap_or(&"On")
        .to_string();

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    TerminalSettings {
        font_family: font_family_entry.text().to_string(),
        font_size: font_size_spin.value() as u32,
        scrollback_lines: scrollback_spin.value() as u32,
        color_theme,
        cursor_shape,
        cursor_blink: cursor_blink_mode,
        scroll_on_output: scroll_on_output_check.is_active(),
        scroll_on_keystroke: scroll_on_keystroke_check.is_active(),
        allow_hyperlinks: allow_hyperlinks_check.is_active(),
        mouse_autohide: mouse_autohide_check.is_active(),
        audible_bell: audible_bell_check.is_active(),
    }
}
