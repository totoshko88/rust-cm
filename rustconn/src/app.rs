//! GTK4 Application setup and initialization
//!
//! This module provides the main application entry point and configuration
//! for the RustConn GTK4 application, including state management and
//! action setup.

use gtk4::prelude::*;
use gtk4::{gio, glib, Application};

use crate::state::{create_shared_state, SharedAppState};
use crate::window::MainWindow;

/// Application ID for RustConn
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
    // Create shared application state
    let state = match create_shared_state() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to initialize application state: {}", e);
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
    app.set_accels_for_action("win.connect", &["Return", "KP_Enter"]);
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
}

/// Shows the about dialog
fn show_about_dialog(parent: &gtk4::ApplicationWindow) {
    let about = gtk4::AboutDialog::builder()
        .transient_for(parent)
        .modal(true)
        .program_name("RustConn")
        .version("0.1.0")
        .comments("A modern connection manager for Linux\n\nWayland-native GTK4 application for managing SSH, RDP, and VNC connections.")
        .website("https://github.com/totoshko88/rust-cm")
        .website_label("GitHub Repository")
        .license_type(gtk4::License::Gpl30)
        .authors(vec!["Anton Isaiev <totoshko88@gmail.com>"])
        .build();
    
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
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .build();
    
    dialog.show(Some(&window));
}

/// Runs the GTK4 application
///
/// This is the main entry point that initializes GTK and runs the event loop.
pub fn run() -> glib::ExitCode {
    let app = create_application();
    app.run()
}
