//! Application settings model
//!
//! This module defines the application-wide settings stored in config.toml.

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application-wide settings
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Terminal settings
    #[serde(default)]
    pub terminal: TerminalSettings,
    /// Logging settings
    #[serde(default)]
    pub logging: LoggingSettings,
    /// Secret storage settings
    #[serde(default)]
    pub secrets: SecretSettings,
    /// UI settings
    #[serde(default)]
    pub ui: UiSettings,
}

/// Terminal-related settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSettings {
    /// Font family for terminal
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// Font size in points
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    /// Scrollback buffer lines
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: u32,
}

fn default_font_family() -> String {
    "Monospace".to_string()
}

const fn default_font_size() -> u32 {
    12
}

const fn default_scrollback() -> u32 {
    10000
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            scrollback_lines: default_scrollback(),
        }
    }
}

/// Logging settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Enable session logging
    #[serde(default)]
    pub enabled: bool,
    /// Directory for log files (relative to config dir if not absolute)
    #[serde(default = "default_log_dir")]
    pub log_directory: PathBuf,
    /// Number of days to retain logs
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

fn default_log_dir() -> PathBuf {
    PathBuf::from("logs")
}

const fn default_retention_days() -> u32 {
    30
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            log_directory: default_log_dir(),
            retention_days: default_retention_days(),
        }
    }
}

/// Secret storage settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSettings {
    /// Preferred secret backend
    #[serde(default)]
    pub preferred_backend: SecretBackendType,
    /// Enable fallback to libsecret if `KeePassXC` unavailable
    #[serde(default = "default_true")]
    pub enable_fallback: bool,
    /// Path to `KeePass` database file (.kdbx)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kdbx_path: Option<PathBuf>,
    /// Whether `KeePass` integration is enabled
    #[serde(default)]
    pub kdbx_enabled: bool,
    /// `KeePass` database password (NOT serialized for security)
    #[serde(skip)]
    pub kdbx_password: Option<SecretString>,
}

const fn default_true() -> bool {
    true
}

impl Default for SecretSettings {
    fn default() -> Self {
        Self {
            preferred_backend: SecretBackendType::default(),
            enable_fallback: true,
            kdbx_path: None,
            kdbx_enabled: false,
            kdbx_password: None,
        }
    }
}

impl PartialEq for SecretSettings {
    fn eq(&self, other: &Self) -> bool {
        self.preferred_backend == other.preferred_backend
            && self.enable_fallback == other.enable_fallback
            && self.kdbx_path == other.kdbx_path
            && self.kdbx_enabled == other.kdbx_enabled
        // Note: kdbx_password is intentionally excluded from equality comparison
        // as it's a runtime-only field that shouldn't affect settings equality
    }
}

impl Eq for SecretSettings {}

/// Secret backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretBackendType {
    /// `KeePassXC` browser integration
    #[default]
    KeePassXc,
    /// Direct KDBX file access
    KdbxFile,
    /// libsecret (GNOME Keyring/KDE Wallet)
    LibSecret,
}

/// UI settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSettings {
    /// Remember window geometry
    #[serde(default = "default_true")]
    pub remember_window_geometry: bool,
    /// Window width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_width: Option<i32>,
    /// Window height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_height: Option<i32>,
    /// Sidebar width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidebar_width: Option<i32>,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            remember_window_geometry: true,
            window_width: None,
            window_height: None,
            sidebar_width: None,
        }
    }
}
