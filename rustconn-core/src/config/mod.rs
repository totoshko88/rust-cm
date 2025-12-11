//! Configuration management for `RustConn`
//!
//! This module provides the `ConfigManager` for loading and saving
//! configuration files in TOML format.

mod manager;
pub mod settings;

pub use manager::ConfigManager;
pub use settings::{
    AppSettings, LoggingSettings, SecretBackendType, SecretSettings, TerminalSettings, UiSettings,
};
