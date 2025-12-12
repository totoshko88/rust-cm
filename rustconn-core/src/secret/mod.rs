//! Secret management module for `RustConn`
//!
//! This module provides secure credential storage through multiple backends:
//! - `KeePassXC` via browser integration protocol (primary)
//! - libsecret for GNOME Keyring/KDE Wallet integration (fallback)
//! - Direct KDBX file access
//!
//! The `SecretManager` provides a unified interface with automatic fallback
//! when the primary backend is unavailable.

mod backend;
mod kdbx;
mod keepassxc;
mod libsecret;
mod manager;
mod resolver;
mod status;

pub use backend::SecretBackend;
pub use kdbx::KdbxExporter;
pub use keepassxc::KeePassXcBackend;
pub use libsecret::LibSecretBackend;
pub use manager::SecretManager;
pub use resolver::CredentialResolver;
pub use status::{parse_keepassxc_version, KeePassStatus};
