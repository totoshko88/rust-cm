//! RustConn - Modern Connection Manager for Linux

mod app;
pub mod dialogs;
mod sidebar;
mod terminal;
mod window;

fn main() -> gtk4::glib::ExitCode {
    app::run()
}
