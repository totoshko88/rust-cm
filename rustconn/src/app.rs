//! GTK4 Application setup and initialization
//!
//! This module provides the main application entry point and configuration
//! for the `RustConn` GTK4 application, including state management and
//! action setup.

use gtk4::prelude::*;
use gtk4::{gio, glib, Application};

use crate::state::{create_shared_state, SharedAppState};
use crate::window::MainWindow;

/// Application ID for `RustConn`
pub const APP_ID: &str = "org.rustconn.RustConn";

/// Creates and configures the GTK4 Application
///
/// Sets up the application with Wayland-native configuration and
/// connects the activate signal to create the main window.
#[must_use]
pub fn create_application() -> Application {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::default())
        .build();

    app.connect_activate(build_ui);

    app
}

/// Builds the main UI when the application is activated
fn build_ui(app: &Application) {
    // Load CSS styles for split view panes
    load_css_styles();

    // Create shared application state
    let state = match create_shared_state() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to initialize application state: {e}");
            show_error_dialog(app, "Initialization Error", &e);
            return;
        }
    };

    // Create main window with state
    let window = MainWindow::new(app, state.clone());

    // Set up application actions
    setup_app_actions(app, &window, state);

    window.present();
}

/// Loads CSS styles for the application
fn load_css_styles() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(
        r"
        /* Split view pane styles */
        .focused-pane {
            border: 2px solid @accent_color;
            border-radius: 4px;
        }

        .unfocused-pane {
            border: 1px solid @borders;
            border-radius: 4px;
        }

        /* Pane placeholder styles */
        .dim-label {
            opacity: 0.6;
        }

        /* Floating controls styles - Requirement 5.2 */
        .floating-controls {
            background-color: alpha(@window_bg_color, 0.85);
            border-radius: 8px;
            padding: 6px 12px;
            box-shadow: 0 2px 8px alpha(black, 0.3);
            border: 1px solid alpha(@borders, 0.5);
        }

        .floating-control-button {
            min-width: 36px;
            min-height: 36px;
            padding: 8px;
            border-radius: 6px;
            background-color: transparent;
            transition: background-color 150ms ease-in-out,
                        transform 100ms ease-in-out;
        }

        .floating-control-button:hover {
            background-color: alpha(@accent_color, 0.15);
            transform: scale(1.05);
        }

        .floating-control-button:active {
            background-color: alpha(@accent_color, 0.25);
            transform: scale(0.95);
        }

        .floating-control-button.destructive-action {
            color: @error_color;
        }

        .floating-control-button.destructive-action:hover {
            background-color: alpha(@error_color, 0.15);
        }

        .floating-control-button.destructive-action:active {
            background-color: alpha(@error_color, 0.25);
        }

        /* VNC display placeholder */
        .vnc-display {
            background-color: @view_bg_color;
        }
        ",
    );

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not get default display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Sets up application-level actions
fn setup_app_actions(app: &Application, window: &MainWindow, _state: SharedAppState) {
    // Quit action
    let quit_action = gio::SimpleAction::new("quit", None);
    let app_weak = app.downgrade();
    quit_action.connect_activate(move |_, _| {
        if let Some(app) = app_weak.upgrade() {
            app.quit();
        }
    });
    app.add_action(&quit_action);

    // About action
    let about_action = gio::SimpleAction::new("about", None);
    let window_weak = window.gtk_window().downgrade();
    about_action.connect_activate(move |_, _| {
        if let Some(window) = window_weak.upgrade() {
            show_about_dialog(&window);
        }
    });
    app.add_action(&about_action);

    // Set up keyboard shortcuts
    // Application shortcuts
    app.set_accels_for_action("app.quit", &["<Control>q"]);
    app.set_accels_for_action("app.about", &["F1"]);

    // Connection management shortcuts
    app.set_accels_for_action("win.new-connection", &["<Control>n"]);
    app.set_accels_for_action("win.new-group", &["<Control><Shift>n"]);
    app.set_accels_for_action("win.import", &["<Control>i"]);
    // Note: Enter key is NOT bound globally to avoid intercepting terminal input
    // Use double-click on sidebar items to connect instead
    app.set_accels_for_action("win.edit-connection", &["<Control>e"]);
    app.set_accels_for_action("win.delete-connection", &["Delete"]);
    app.set_accels_for_action("win.duplicate-connection", &["<Control>d"]);

    // Navigation shortcuts
    app.set_accels_for_action("win.search", &["<Control>f", "<Control>k"]);
    app.set_accels_for_action("win.focus-sidebar", &["<Control>1", "<Alt>1"]);
    app.set_accels_for_action("win.focus-terminal", &["<Control>2", "<Alt>2"]);

    // Terminal shortcuts
    app.set_accels_for_action("win.copy", &["<Control><Shift>c"]);
    app.set_accels_for_action("win.paste", &["<Control><Shift>v"]);
    app.set_accels_for_action("win.close-tab", &["<Control>w"]);
    app.set_accels_for_action("win.next-tab", &["<Control>Tab", "<Control>Page_Down"]);
    app.set_accels_for_action("win.prev-tab", &["<Control><Shift>Tab", "<Control>Page_Up"]);

    // Settings
    app.set_accels_for_action("win.settings", &["<Control>comma"]);

    // New actions
    app.set_accels_for_action("win.local-shell", &["<Control><Shift>t"]);
    app.set_accels_for_action("win.quick-connect", &["<Control><Shift>q"]);
    app.set_accels_for_action("win.export", &["<Control><Shift>e"]);

    // Split view shortcuts
    app.set_accels_for_action("win.split-horizontal", &["<Control><Shift>h"]);
    app.set_accels_for_action("win.split-vertical", &["<Control><Shift>s"]);
    app.set_accels_for_action("win.close-pane", &["<Control><Shift>w"]);
    app.set_accels_for_action("win.focus-next-pane", &["<Control>grave"]); // Ctrl+`
}

/// Shows the about dialog
fn show_about_dialog(parent: &gtk4::ApplicationWindow) {
    let about = gtk4::AboutDialog::builder()
        .transient_for(parent)
        .modal(true)
        .program_name("RustConn")
        .version("0.1.0")
        .comments("A modern connection manager for Linux\n\nWayland-native GTK4 application for managing SSH, RDP, and VNC connections.")
        .website("https://github.com/totoshko88/rustconn")
        .website_label("GitHub Repository")
        .license_type(gtk4::License::Gpl30)
        .authors(vec!["Anton Isaiev <totoshko88@gmail.com>"])
        .build();

    // Add support/sponsorship information
    about.add_credit_section(
        "Support the Project",
        &[
            "Ko-Fi: https://ko-fi.com/totoshko88",
            "PayPal/Payoneer: totoshko88@gmail.com",
            "UAH: https://send.monobank.ua/jar/2UgaGcQ3JC",
        ],
    );

    about.add_credit_section(
        "Acknowledgments",
        &[
            "GTK4 and the GNOME project",
            "The Rust community",
            "FreeRDP project",
            "All contributors and supporters",
        ],
    );

    // Load and set the application logo
    let logo_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/rustconn.svg");
    if let Ok(logo) = gtk4::gdk::Texture::from_filename(logo_path) {
        about.set_logo(Some(&logo));
    }

    about.present();
}

/// Shows an error dialog
fn show_error_dialog(app: &Application, title: &str, message: &str) {
    let dialog = gtk4::AlertDialog::builder()
        .message(title)
        .detail(message)
        .modal(true)
        .build();

    // Create a temporary window to show the dialog
    let window = gtk4::ApplicationWindow::builder().application(app).build();

    dialog.show(Some(&window));
}

/// Runs the GTK4 application
///
/// This is the main entry point that initializes GTK and runs the event loop.
pub fn run() -> glib::ExitCode {
    let app = create_application();
    app.run()
}
