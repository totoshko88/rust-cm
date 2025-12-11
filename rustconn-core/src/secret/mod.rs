//! Secret management module for `RustConn`
//!
//! This module provides secure credential storage through multiple backends:
//! - `KeePassXC` via browser integration protocol (primary)
//! - libsecret for GNOME Keyring/KDE Wallet integration (fallback)
//!
//! The `SecretManager` provides a unified interface with automatic fallback
//! when the primary backend is unavailable.

mod backend;
mod keepassxc;
mod kdbx;
mod libsecret;
mod manager;

pub use backend::SecretBackend;
pub use keepassxc::KeePassXcBackend;
pub use kdbx::KdbxExporter;
pub use libsecret::LibSecretBackend;
pub use manager::SecretManager;
