//! `KeePass` integration status detection
//!
//! This module provides functionality to detect the status of `KeePass` integration,
//! including `KeePassXC` installation detection, version parsing, and KDBX file validation.

use std::path::Path;
use std::process::Command;

/// Status of `KeePass` integration
///
/// This struct provides information about the current state of `KeePass` integration,
/// including whether `KeePassXC` is installed, its version, and KDBX file accessibility.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct KeePassStatus {
    /// Whether `KeePassXC` application is installed
    pub keepassxc_installed: bool,
    /// `KeePassXC` version if installed
    pub keepassxc_version: Option<String>,
    /// Path to `KeePassXC` CLI binary
    pub keepassxc_path: Option<std::path::PathBuf>,
    /// Whether KDBX file is configured
    pub kdbx_configured: bool,
    /// Whether KDBX file exists and is accessible
    pub kdbx_accessible: bool,
    /// Whether integration is currently active (unlocked)
    pub integration_active: bool,
}

impl KeePassStatus {
    /// Detects current `KeePass` status by checking for `KeePassXC` installation
    ///
    /// This method searches for the `keepassxc-cli` binary in common locations
    /// and attempts to determine its version.
    #[must_use]
    pub fn detect() -> Self {
        let mut status = Self::default();

        // Try to find keepassxc-cli in PATH or common locations
        if let Some(path) = Self::find_keepassxc_cli() {
            status.keepassxc_installed = true;
            status.keepassxc_path = Some(path.clone());

            // Try to get version
            if let Some(version) = Self::get_keepassxc_version(&path) {
                status.keepassxc_version = Some(version);
            }
        }

        status
    }

    /// Detects status with a configured KDBX path
    ///
    /// # Arguments
    /// * `kdbx_path` - Optional path to the KDBX database file
    #[must_use]
    pub fn detect_with_kdbx(kdbx_path: Option<&Path>) -> Self {
        let mut status = Self::detect();

        if let Some(path) = kdbx_path {
            status.kdbx_configured = true;
            status.kdbx_accessible = path.exists() && path.is_file();
        }

        status
    }

    /// Validates a KDBX file path
    ///
    /// # Arguments
    /// * `path` - Path to validate
    ///
    /// # Returns
    /// * `Ok(())` if the path is valid (ends with .kdbx and file exists)
    /// * `Err(String)` with a description of the validation failure
    ///
    /// # Errors
    /// Returns an error if:
    /// - The path does not have a .kdbx extension (case-insensitive)
    /// - The file does not exist
    /// - The path points to a directory instead of a file
    pub fn validate_kdbx_path(path: &Path) -> Result<(), String> {
        // Check extension (case-insensitive)
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase);

        if extension.as_deref() != Some("kdbx") {
            return Err("File must have .kdbx extension".to_string());
        }

        // Check if file exists
        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()));
        }

        // Check if it's a file (not a directory)
        if !path.is_file() {
            return Err(format!("Path is not a file: {}", path.display()));
        }

        Ok(())
    }

    /// Finds the `keepassxc-cli` binary
    ///
    /// Searches in PATH and common installation locations.
    fn find_keepassxc_cli() -> Option<std::path::PathBuf> {
        // First, try to find in PATH using `which`
        if let Ok(output) = Command::new("which").arg("keepassxc-cli").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = std::path::PathBuf::from(path_str.trim());
                if path.exists() {
                    return Some(path);
                }
            }
        }

        // Check common installation paths
        let common_paths = [
            "/usr/bin/keepassxc-cli",
            "/usr/local/bin/keepassxc-cli",
            "/snap/bin/keepassxc-cli",
            "/var/lib/flatpak/exports/bin/org.keepassxc.KeePassXC.cli",
        ];

        for path_str in &common_paths {
            let path = std::path::PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Gets the `KeePassXC` version from the CLI
    ///
    /// # Arguments
    /// * `cli_path` - Path to the `keepassxc-cli` binary
    fn get_keepassxc_version(cli_path: &Path) -> Option<String> {
        let output = Command::new(cli_path).arg("--version").output().ok()?;

        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            parse_keepassxc_version(&version_output)
        } else {
            // Some versions output to stderr
            let version_output = String::from_utf8_lossy(&output.stderr);
            parse_keepassxc_version(&version_output)
        }
    }
}

/// Parses a version string from `KeePassXC` CLI output
///
/// The output format is typically: "keepassxc-cli 2.7.6"
/// or just "2.7.6" on some systems.
///
/// # Arguments
/// * `output` - The raw output from `keepassxc-cli --version`
///
/// # Returns
/// * `Some(String)` containing the version number if found
/// * `None` if no valid version could be extracted
#[must_use]
pub fn parse_keepassxc_version(output: &str) -> Option<String> {
    let output = output.trim();

    if output.is_empty() {
        return None;
    }

    // Try to find a version pattern (digits and dots)
    // Common formats:
    // - "keepassxc-cli 2.7.6"
    // - "2.7.6"
    // - "KeePassXC 2.7.6"

    // Split by whitespace and look for version-like strings
    for part in output.split_whitespace() {
        // Check if this part looks like a version (starts with digit, contains dots)
        if part.chars().next().is_some_and(|c| c.is_ascii_digit())
            && part.contains('.')
            && part.chars().all(|c| c.is_ascii_digit() || c == '.')
        {
            return Some(part.to_string());
        }
    }

    // If no version found with dots, try to find any digit sequence
    // This handles edge cases like "2" or "2.7"
    for part in output.split_whitespace() {
        if part.chars().next().is_some_and(|c| c.is_ascii_digit())
            && part.chars().all(|c| c.is_ascii_digit() || c == '.')
        {
            return Some(part.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_kdbx_path_valid_extension() {
        // Create a temp file with .kdbx extension
        let temp_dir = tempfile::tempdir().unwrap();
        let kdbx_path = temp_dir.path().join("test.kdbx");
        std::fs::write(&kdbx_path, b"dummy content").unwrap();

        assert!(KeePassStatus::validate_kdbx_path(&kdbx_path).is_ok());
    }

    #[test]
    fn test_validate_kdbx_path_uppercase_extension() {
        let temp_dir = tempfile::tempdir().unwrap();
        let kdbx_path = temp_dir.path().join("test.KDBX");
        std::fs::write(&kdbx_path, b"dummy content").unwrap();

        assert!(KeePassStatus::validate_kdbx_path(&kdbx_path).is_ok());
    }

    #[test]
    fn test_validate_kdbx_path_wrong_extension() {
        let temp_dir = tempfile::tempdir().unwrap();
        let txt_path = temp_dir.path().join("test.txt");
        std::fs::write(&txt_path, b"dummy content").unwrap();

        let result = KeePassStatus::validate_kdbx_path(&txt_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".kdbx extension"));
    }

    #[test]
    fn test_validate_kdbx_path_nonexistent() {
        let path = std::path::PathBuf::from("/nonexistent/path/test.kdbx");
        let result = KeePassStatus::validate_kdbx_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_validate_kdbx_path_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        // Create a directory with .kdbx name
        let dir_path = temp_dir.path().join("test.kdbx");
        std::fs::create_dir(&dir_path).unwrap();

        let result = KeePassStatus::validate_kdbx_path(&dir_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a file"));
    }

    #[test]
    fn test_parse_version_standard_format() {
        assert_eq!(
            parse_keepassxc_version("keepassxc-cli 2.7.6"),
            Some("2.7.6".to_string())
        );
    }

    #[test]
    fn test_parse_version_just_number() {
        assert_eq!(
            parse_keepassxc_version("2.7.6"),
            Some("2.7.6".to_string())
        );
    }

    #[test]
    fn test_parse_version_with_prefix() {
        assert_eq!(
            parse_keepassxc_version("KeePassXC 2.7.6"),
            Some("2.7.6".to_string())
        );
    }

    #[test]
    fn test_parse_version_empty() {
        assert_eq!(parse_keepassxc_version(""), None);
    }

    #[test]
    fn test_parse_version_whitespace() {
        assert_eq!(parse_keepassxc_version("   "), None);
    }

    #[test]
    fn test_parse_version_no_version() {
        assert_eq!(parse_keepassxc_version("keepassxc-cli"), None);
    }

    #[test]
    fn test_parse_version_with_newline() {
        assert_eq!(
            parse_keepassxc_version("keepassxc-cli 2.7.6\n"),
            Some("2.7.6".to_string())
        );
    }

    #[test]
    fn test_default_status() {
        let status = KeePassStatus::default();
        assert!(!status.keepassxc_installed);
        assert!(status.keepassxc_version.is_none());
        assert!(status.keepassxc_path.is_none());
        assert!(!status.kdbx_configured);
        assert!(!status.kdbx_accessible);
        assert!(!status.integration_active);
    }
}
