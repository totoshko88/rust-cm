//! Utility functions for the RustConn GUI
//!
//! This module provides common utility functions used across the application,
//! including safe display access, CSS provider management, and accessibility helpers.

use gtk4::gdk;
use std::sync::LazyLock;

/// Gets the default GDK display, returning None if unavailable
///
/// This is safer than using `gdk::Display::default().expect(...)` which
/// would panic in headless environments or during testing.
///
/// # Returns
///
/// `Some(Display)` if a display is available, `None` otherwise.
#[must_use]
pub fn get_display() -> Option<gdk::Display> {
    gdk::Display::default()
}

/// Adds a CSS provider to the default display if available
///
/// This is a safe wrapper around `style_context_add_provider_for_display`
/// that gracefully handles the case where no display is available.
///
/// # Arguments
///
/// * `provider` - The CSS provider to add
/// * `priority` - The priority for the provider
///
/// # Returns
///
/// `true` if the provider was added, `false` if no display was available.
pub fn add_css_provider(provider: &gtk4::CssProvider, priority: u32) -> bool {
    if let Some(display) = get_display() {
        gtk4::style_context_add_provider_for_display(&display, provider, priority);
        true
    } else {
        tracing::warn!("No display available, CSS provider not added");
        false
    }
}

/// Removes a CSS provider from the default display if available
///
/// # Arguments
///
/// * `provider` - The CSS provider to remove
///
/// # Returns
///
/// `true` if the provider was removed, `false` if no display was available.
pub fn remove_css_provider(provider: &gtk4::CssProvider) -> bool {
    if let Some(display) = get_display() {
        gtk4::style_context_remove_provider_for_display(&display, provider);
        true
    } else {
        false
    }
}

/// Sets accessible properties on a widget
///
/// Helper function to set common accessibility properties in a consistent way.
///
/// # Arguments
///
/// * `widget` - The widget to update
/// * `label` - The accessible label (read by screen readers)
/// * `description` - Optional description providing more context
pub fn set_accessible_properties(
    widget: &impl gtk4::prelude::AccessibleExtManual,
    label: &str,
    description: Option<&str>,
) {
    let mut properties = vec![gtk4::accessible::Property::Label(label)];

    if let Some(desc) = description {
        properties.push(gtk4::accessible::Property::Description(desc));
    }

    widget.update_property(&properties);
}

/// Sets an accessible label on a widget
///
/// Shorthand for setting just the accessible label.
///
/// # Arguments
///
/// * `widget` - The widget to update
/// * `label` - The accessible label
pub fn set_accessible_label(widget: &impl gtk4::prelude::AccessibleExtManual, label: &str) {
    widget.update_property(&[gtk4::accessible::Property::Label(label)]);
}

/// Regex pattern for extracting variable names from templates
///
/// Matches patterns like `${variable_name}` and captures the variable name.
pub static VARIABLE_PATTERN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").expect("Variable pattern regex is valid")
});

/// Extracts variable names from a template string
///
/// Finds all `${variable_name}` patterns and returns the variable names.
///
/// # Arguments
///
/// * `template` - The template string to search
///
/// # Returns
///
/// A vector of unique variable names found in the template.
#[must_use]
pub fn extract_variables(template: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();

    for cap in VARIABLE_PATTERN.captures_iter(template) {
        if let Some(var_match) = cap.get(1) {
            let var_name = var_match.as_str().to_string();
            if !found.contains(&var_name) {
                found.push(var_name);
            }
        }
    }

    found
}

/// Truncates a string to a maximum length, adding ellipsis if needed
///
/// # Arguments
///
/// * `s` - The string to truncate
/// * `max_len` - Maximum length (including ellipsis)
///
/// # Returns
///
/// The truncated string with "…" appended if it was shortened.
#[must_use]
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 1 {
        "…".to_string()
    } else {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{truncated}…")
    }
}

/// Formats a duration in a human-readable way
///
/// # Arguments
///
/// * `seconds` - Duration in seconds
///
/// # Returns
///
/// A human-readable string like "2h 30m" or "45s".
#[must_use]
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        if secs == 0 {
            format!("{minutes}m")
        } else {
            format!("{minutes}m {secs}s")
        }
    } else {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        if minutes == 0 {
            format!("{hours}h")
        } else {
            format!("{hours}h {minutes}m")
        }
    }
}

/// Formats a byte count in a human-readable way
///
/// # Arguments
///
/// * `bytes` - Number of bytes
///
/// # Returns
///
/// A human-readable string like "1.5 MB" or "256 KB".
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variables() {
        let template = "Hello ${name}, your ID is ${user_id}";
        let vars = extract_variables(template);
        assert_eq!(vars, vec!["name", "user_id"]);
    }

    #[test]
    fn test_extract_variables_duplicates() {
        let template = "${var} and ${var} again";
        let vars = extract_variables(template);
        assert_eq!(vars, vec!["var"]);
    }

    #[test]
    fn test_extract_variables_empty() {
        let template = "No variables here";
        let vars = extract_variables(template);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello w…");
        assert_eq!(truncate_string("hi", 2), "hi");
        assert_eq!(truncate_string("hi", 1), "…");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(60), "1m");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3600), "1h");
        assert_eq!(format_duration(3660), "1h 1m");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }
}
