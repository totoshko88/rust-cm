//! Group hierarchy methods for the main window
//!
//! This module contains methods for managing connection groups,
//! including move to group dialog and related functionality.

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Button, HeaderBar, Label, Orientation};
use std::rc::Rc;
use uuid::Uuid;

use crate::sidebar::ConnectionSidebar;
use crate::state::SharedAppState;
use crate::window::MainWindow;

/// Type alias for shared sidebar
pub type SharedSidebar = Rc<ConnectionSidebar>;

/// Shows the move to group dialog for the selected connection
#[allow(clippy::too_many_lines)]
pub fn show_move_to_group_dialog(
    window: &ApplicationWindow,
    state: &SharedAppState,
    sidebar: &SharedSidebar,
) {
    // Get selected item using sidebar's method
    let Some(conn_item) = sidebar.get_selected_item() else {
        return;
    };

    // Can only move connections, not groups
    if conn_item.is_group() {
        let alert = gtk4::AlertDialog::builder()
            .message("Cannot Move Group")
            .detail("Use drag and drop to reorganize groups.")
            .modal(true)
            .build();
        alert.show(Some(window));
        return;
    }

    let id_str = conn_item.id();
    let Ok(connection_id) = Uuid::parse_str(&id_str) else {
        return;
    };
    let connection_name = conn_item.name();

    // Get current group
    let state_ref = state.borrow();
    let current_group_id = state_ref
        .get_connection(connection_id)
        .and_then(|c| c.group_id);
    drop(state_ref);

    // Create dialog
    let move_window = gtk4::Window::builder()
        .title("Move Connection")
        .transient_for(window)
        .modal(true)
        .default_width(350)
        .build();

    let header = HeaderBar::new();
    let cancel_btn = Button::builder().label("Cancel").build();
    let move_btn = Button::builder()
        .label("Move")
        .css_classes(["suggested-action"])
        .build();
    header.pack_start(&cancel_btn);
    header.pack_end(&move_btn);
    move_window.set_titlebar(Some(&header));

    let content = gtk4::Box::new(Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let info_label = Label::builder()
        .label(format!("Move '{connection_name}' to:"))
        .halign(gtk4::Align::Start)
        .build();
    content.append(&info_label);

    // Group dropdown
    let state_ref = state.borrow();
    let groups: Vec<_> = state_ref
        .list_groups()
        .iter()
        .map(|g| (*g).clone())
        .collect();
    drop(state_ref);

    let mut group_ids: Vec<Option<Uuid>> = vec![None];
    let mut strings: Vec<String> = vec!["(Ungrouped)".to_string()];
    let mut current_index = 0u32;

    for group in &groups {
        let state_ref = state.borrow();
        let path = state_ref
            .get_group_path(group.id)
            .unwrap_or_else(|| group.name.clone());
        drop(state_ref);

        strings.push(path);
        group_ids.push(Some(group.id));

        if current_group_id == Some(group.id) {
            #[allow(clippy::cast_possible_truncation)]
            {
                current_index = (group_ids.len() - 1) as u32;
            }
        }
    }

    let string_list = gtk4::StringList::new(
        &strings
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<_>>(),
    );
    let group_dropdown = gtk4::DropDown::builder()
        .model(&string_list)
        .selected(current_index)
        .build();

    content.append(&group_dropdown);
    move_window.set_child(Some(&content));

    // Connect cancel
    let window_clone = move_window.clone();
    cancel_btn.connect_clicked(move |_| {
        window_clone.close();
    });

    // Connect move
    let state_clone = state.clone();
    let sidebar_clone = sidebar.clone();
    let window_clone = move_window.clone();
    move_btn.connect_clicked(move |_| {
        let selected_idx = group_dropdown.selected() as usize;
        let target_group_id = if selected_idx < group_ids.len() {
            group_ids[selected_idx]
        } else {
            None
        };

        if let Ok(mut state_mut) = state_clone.try_borrow_mut() {
            match state_mut.move_connection_to_group(connection_id, target_group_id) {
                Ok(()) => {
                    drop(state_mut);
                    // Preserve tree state when moving connections
                    MainWindow::reload_sidebar_preserving_state(&state_clone, &sidebar_clone);
                    window_clone.close();
                }
                Err(e) => {
                    let alert = gtk4::AlertDialog::builder()
                        .message("Error Moving Connection")
                        .detail(&e)
                        .modal(true)
                        .build();
                    alert.show(Some(&window_clone));
                }
            }
        }
    });

    move_window.present();
}

/// Shows an error toast/notification
pub fn show_error_toast(window: &ApplicationWindow, message: &str) {
    let alert = gtk4::AlertDialog::builder()
        .message("Error")
        .detail(message)
        .modal(true)
        .build();
    alert.show(Some(window));
}
