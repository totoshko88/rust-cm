//! GTK4 Application setup and initialization

use gtk4::prelude::*;
use gtk4::{gio, glib, Application};

use crate::window::MainWindow;

pub const APP_ID: &str = "org.rustconn.RustConn";

#[must_use]
pub fn create_application() -> Application {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::default())
        .build();

    app.connect_activate(build_ui);
    app
}

fn build_ui(app: &Application) {
    let window = MainWindow::new(app);
    window.present();
}

pub fn run() -> glib::ExitCode {
    let app = create_application();
    app.run()
}
