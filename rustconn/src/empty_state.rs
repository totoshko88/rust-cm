//! Empty state widgets for showing helpful messages when content is empty
//!
//! Provides consistent empty state UI following GNOME HIG patterns.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Image, Label, Orientation};

/// Creates an empty state widget with icon, title, description, and optional action
///
/// # Arguments
///
/// * `icon_name` - The icon to display (symbolic icon name)
/// * `title` - The main title text
/// * `description` - A helpful description or suggestion
/// * `action_label` - Optional button label for a primary action
/// * `action_name` - Optional action name to trigger when button is clicked
///
/// # Returns
///
/// A `GtkBox` containing the empty state UI
#[must_use]
pub fn create_empty_state(
    icon_name: &str,
    title: &str,
    description: &str,
    action_label: Option<&str>,
    action_name: Option<&str>,
) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .halign(Align::Center)
        .valign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .css_classes(["empty-state"])
        .build();

    // Icon
    let icon = Image::builder()
        .icon_name(icon_name)
        .pixel_size(96)
        .css_classes(["empty-state-icon"])
        .build();
    container.append(&icon);

    // Title
    let title_label = Label::builder()
        .label(title)
        .css_classes(["empty-state-title"])
        .build();
    container.append(&title_label);

    // Description
    let desc_label = Label::builder()
        .label(description)
        .wrap(true)
        .max_width_chars(40)
        .justify(gtk4::Justification::Center)
        .css_classes(["empty-state-description"])
        .build();
    container.append(&desc_label);

    // Optional action button
    if let (Some(label), Some(action)) = (action_label, action_name) {
        let button = Button::builder()
            .label(label)
            .action_name(action)
            .css_classes(["suggested-action", "pill"])
            .margin_top(12)
            .build();
        container.append(&button);
    }

    container
}

/// Creates an empty state for no connections
#[must_use]
pub fn no_connections() -> GtkBox {
    create_empty_state(
        "network-server-symbolic",
        "No Connections",
        "Create your first connection to get started",
        Some("New Connection"),
        Some("win.new-connection"),
    )
}

/// Creates an empty state for no search results
#[must_use]
pub fn no_search_results(query: &str) -> GtkBox {
    create_empty_state(
        "edit-find-symbolic",
        "No Results Found",
        &format!("No connections match \"{query}\""),
        None,
        None,
    )
}

/// Creates an empty state for no sessions
#[must_use]
pub fn no_sessions() -> GtkBox {
    create_empty_state(
        "utilities-terminal-symbolic",
        "No Active Sessions",
        "Connect to a server to start a session",
        Some("Quick Connect"),
        Some("win.quick-connect"),
    )
}

/// Creates an empty state for no groups
#[must_use]
pub fn no_groups() -> GtkBox {
    create_empty_state(
        "folder-symbolic",
        "No Groups",
        "Create groups to organize your connections",
        Some("New Group"),
        Some("win.new-group"),
    )
}

/// Creates an empty state for no snippets
#[must_use]
pub fn no_snippets() -> GtkBox {
    create_empty_state(
        "edit-paste-symbolic",
        "No Snippets",
        "Create command snippets for quick access",
        Some("New Snippet"),
        Some("win.new-snippet"),
    )
}

/// Creates an empty state for no templates
#[must_use]
pub fn no_templates() -> GtkBox {
    create_empty_state(
        "document-new-symbolic",
        "No Templates",
        "Create templates for common connection configurations",
        Some("New Template"),
        Some("win.new-template"),
    )
}

/// Creates an empty state for no history
#[must_use]
pub fn no_history() -> GtkBox {
    create_empty_state(
        "document-open-recent-symbolic",
        "No Connection History",
        "Your recent connections will appear here",
        None,
        None,
    )
}

/// Creates a loading state widget
#[must_use]
pub fn loading_state(message: &str) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .halign(Align::Center)
        .valign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build();

    let spinner = gtk4::Spinner::builder()
        .spinning(true)
        .css_classes(["loading-spinner"])
        .build();
    spinner.set_size_request(48, 48);
    container.append(&spinner);

    let label = Label::builder()
        .label(message)
        .css_classes(["dim-label"])
        .build();
    container.append(&label);

    container
}
