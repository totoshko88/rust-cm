//! RustConn - Modern Connection Manager for Linux
//!
//! A GTK4-based connection manager supporting SSH, RDP, and VNC protocols
//! with Wayland-native support and KeePassXC integration.

mod app;
pub mod dialogs;
mod embedded;
mod sidebar;
mod state;
mod terminal;
mod window;

fn main() -> gtk4::glib::ExitCode {
    app::run()
}
