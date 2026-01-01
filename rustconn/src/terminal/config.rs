//! Terminal configuration
//!
//! This module handles VTE terminal appearance and behavior configuration.

use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use vte4::prelude::*;
use vte4::Terminal;

/// Configures terminal appearance and behavior
pub fn configure_terminal(terminal: &Terminal) {
    // Cursor settings
    terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
    terminal.set_cursor_shape(vte4::CursorShape::Block);

    // Scrolling behavior
    terminal.set_scroll_on_output(false);
    terminal.set_scroll_on_keystroke(true);
    terminal.set_scrollback_lines(10000);

    // Input handling
    terminal.set_input_enabled(true);
    terminal.set_allow_hyperlink(true);
    terminal.set_mouse_autohide(true);

    // Keyboard shortcuts (Copy/Paste)
    setup_keyboard_shortcuts(terminal);

    // Context menu (Right click)
    setup_context_menu(terminal);

    // Colors and font
    setup_colors(terminal);
    setup_font(terminal);
}

/// Sets up keyboard shortcuts for copy/paste
fn setup_keyboard_shortcuts(terminal: &Terminal) {
    let controller = gtk4::EventControllerKey::new();
    let term = terminal.clone();
    controller.connect_key_pressed(move |_, key, _, state| {
        let mask = gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::SHIFT_MASK;
        if state.contains(mask) {
            match key.name().as_deref() {
                Some("C" | "c") => {
                    term.copy_clipboard_format(vte4::Format::Text);
                    return glib::Propagation::Stop;
                }
                Some("V" | "v") => {
                    term.paste_clipboard();
                    return glib::Propagation::Stop;
                }
                _ => (),
            }
        }
        glib::Propagation::Proceed
    });
    terminal.add_controller(controller);
}

/// Sets up context menu for right-click
fn setup_context_menu(terminal: &Terminal) {
    use gtk4::gio;
    use gtk4::PopoverMenu;

    let click_controller = gtk4::GestureClick::new();
    click_controller.set_button(3); // Right click
    let term_menu = terminal.clone();
    click_controller.connect_pressed(move |_gesture, _, x, y| {
        let menu = gio::Menu::new();
        menu.append(Some("Copy"), Some("terminal.copy"));
        menu.append(Some("Paste"), Some("terminal.paste"));
        menu.append(Some("Select All"), Some("terminal.select-all"));

        let popover = PopoverMenu::from_model(Some(&menu));
        popover.set_parent(&term_menu);
        popover.set_has_arrow(false);

        // Create action group for the menu
        let action_group = gio::SimpleActionGroup::new();

        let term_copy = term_menu.clone();
        let action_copy = gio::SimpleAction::new("copy", None);
        action_copy.connect_activate(move |_, _| {
            term_copy.copy_clipboard_format(vte4::Format::Text);
        });
        action_group.add_action(&action_copy);

        let term_paste = term_menu.clone();
        let action_paste = gio::SimpleAction::new("paste", None);
        action_paste.connect_activate(move |_, _| {
            term_paste.paste_clipboard();
        });
        action_group.add_action(&action_paste);

        let term_select = term_menu.clone();
        let action_select = gio::SimpleAction::new("select-all", None);
        action_select.connect_activate(move |_, _| {
            term_select.select_all();
        });
        action_group.add_action(&action_select);

        term_menu.insert_action_group("terminal", Some(&action_group));

        let rect = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
        popover.set_pointing_to(Some(&rect));
        popover.popup();
    });
    terminal.add_controller(click_controller);
}

/// Sets up terminal colors (dark theme)
fn setup_colors(terminal: &Terminal) {
    let bg_color = gdk::RGBA::new(0.1, 0.1, 0.1, 1.0);
    let fg_color = gdk::RGBA::new(0.9, 0.9, 0.9, 1.0);
    terminal.set_color_background(&bg_color);
    terminal.set_color_foreground(&fg_color);

    // Set up palette colors (standard 16-color palette)
    let palette: [gdk::RGBA; 16] = [
        gdk::RGBA::new(0.0, 0.0, 0.0, 1.0), // Black
        gdk::RGBA::new(0.8, 0.0, 0.0, 1.0), // Red
        gdk::RGBA::new(0.0, 0.8, 0.0, 1.0), // Green
        gdk::RGBA::new(0.8, 0.8, 0.0, 1.0), // Yellow
        gdk::RGBA::new(0.0, 0.0, 0.8, 1.0), // Blue
        gdk::RGBA::new(0.8, 0.0, 0.8, 1.0), // Magenta
        gdk::RGBA::new(0.0, 0.8, 0.8, 1.0), // Cyan
        gdk::RGBA::new(0.8, 0.8, 0.8, 1.0), // White
        gdk::RGBA::new(0.4, 0.4, 0.4, 1.0), // Bright Black
        gdk::RGBA::new(1.0, 0.0, 0.0, 1.0), // Bright Red
        gdk::RGBA::new(0.0, 1.0, 0.0, 1.0), // Bright Green
        gdk::RGBA::new(1.0, 1.0, 0.0, 1.0), // Bright Yellow
        gdk::RGBA::new(0.0, 0.0, 1.0, 1.0), // Bright Blue
        gdk::RGBA::new(1.0, 0.0, 1.0, 1.0), // Bright Magenta
        gdk::RGBA::new(0.0, 1.0, 1.0, 1.0), // Bright Cyan
        gdk::RGBA::new(1.0, 1.0, 1.0, 1.0), // Bright White
    ];
    let palette_refs: Vec<&gdk::RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&fg_color), Some(&bg_color), &palette_refs);
}

/// Sets up terminal font
fn setup_font(terminal: &Terminal) {
    let font_desc = gtk4::pango::FontDescription::from_string("Monospace 11");
    terminal.set_font(Some(&font_desc));
}
