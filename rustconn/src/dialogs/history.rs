//! Connection history dialog
//!
//! This module provides a dialog for viewing connection history and statistics.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SearchEntry,
    Window,
};
use rustconn_core::models::{ConnectionHistoryEntry, ConnectionStatistics};
use std::cell::RefCell;
use std::rc::Rc;

/// Connection history dialog
#[allow(dead_code)] // search_entry kept for GTK widget lifecycle
pub struct HistoryDialog {
    window: Window,
    list_box: ListBox,
    search_entry: SearchEntry,
    total_label: Label,
    success_label: Label,
    failed_label: Label,
    entries: Rc<RefCell<Vec<ConnectionHistoryEntry>>>,
    statistics: Rc<RefCell<Vec<ConnectionStatistics>>>,
    on_connect: Rc<RefCell<Option<Box<dyn Fn(&ConnectionHistoryEntry) + 'static>>>>,
}

impl HistoryDialog {
    /// Creates a new history dialog
    #[must_use]
    pub fn new(parent: Option<&impl IsA<Window>>) -> Self {
        let window = Window::builder()
            .title("Connection History")
            .default_width(750)
            .default_height(500)
            .modal(true)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        // Header bar with Close/Connect buttons (GNOME HIG)
        let header = gtk4::HeaderBar::new();
        header.set_show_title_buttons(false);
        let close_btn = Button::builder().label("Close").build();
        let connect_btn = Button::builder()
            .label("Connect")
            .css_classes(["suggested-action"])
            .sensitive(false)
            .build();
        header.pack_start(&close_btn);
        header.pack_end(&connect_btn);
        window.set_titlebar(Some(&header));

        // Close button handler
        let window_clone = window.clone();
        close_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Main content
        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Search entry
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search history...")
            .hexpand(true)
            .build();
        content.append(&search_entry);

        // Statistics summary
        let stats_box = GtkBox::new(Orientation::Horizontal, 24);
        stats_box.set_halign(gtk4::Align::Center);
        stats_box.set_margin_top(8);
        stats_box.set_margin_bottom(8);

        let total_label = Label::builder()
            .label("Total: 0")
            .css_classes(["dim-label"])
            .build();
        let success_label = Label::builder()
            .label("Successful: 0")
            .css_classes(["dim-label"])
            .build();
        let failed_label = Label::builder()
            .label("Failed: 0")
            .css_classes(["dim-label"])
            .build();

        stats_box.append(&total_label);
        stats_box.append(&success_label);
        stats_box.append(&failed_label);
        content.append(&stats_box);

        // History list
        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .min_content_height(300)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();

        scrolled.set_child(Some(&list_box));
        content.append(&scrolled);

        // Action buttons (Reset at bottom left)
        let button_box = GtkBox::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::Start);

        let reset_btn = Button::builder()
            .label("Reset")
            .css_classes(["destructive-action"])
            .build();

        button_box.append(&reset_btn);
        content.append(&button_box);

        window.set_child(Some(&content));

        let dialog = Self {
            window: window.clone(),
            list_box: list_box.clone(),
            search_entry: search_entry.clone(),
            total_label: total_label.clone(),
            success_label: success_label.clone(),
            failed_label: failed_label.clone(),
            entries: Rc::new(RefCell::new(Vec::new())),
            statistics: Rc::new(RefCell::new(Vec::new())),
            on_connect: Rc::new(RefCell::new(None)),
        };

        // Connect search
        let entries = dialog.entries.clone();
        let list_box_clone = list_box.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_lowercase();
            Self::filter_list(&list_box_clone, &entries.borrow(), &query);
        });

        let connect_btn_clone = connect_btn.clone();
        list_box.connect_row_selected(move |_, row| {
            connect_btn_clone.set_sensitive(row.is_some());
        });

        // Connect button
        let entries_clone = dialog.entries.clone();
        let list_box_clone = list_box.clone();
        let on_connect = dialog.on_connect.clone();
        let window_clone = window.clone();
        connect_btn.connect_clicked(move |_| {
            if let Some(row) = list_box_clone.selected_row() {
                let index = row.index();
                if index >= 0 {
                    let entries_ref = entries_clone.borrow();
                    #[allow(clippy::cast_sign_loss)]
                    if let Some(entry) = entries_ref.get(index as usize) {
                        if let Some(ref callback) = *on_connect.borrow() {
                            callback(entry);
                        }
                        window_clone.close();
                    }
                }
            }
        });

        // Clear history button (now in header as Reset)
        let entries_clear = dialog.entries.clone();
        let list_box_clear = list_box;
        let total_label_clear = dialog.total_label.clone();
        let success_label_clear = dialog.success_label.clone();
        let failed_label_clear = dialog.failed_label.clone();
        reset_btn.connect_clicked(move |_| {
            entries_clear.borrow_mut().clear();
            while let Some(row) = list_box_clear.row_at_index(0) {
                list_box_clear.remove(&row);
            }
            total_label_clear.set_label("Total: 0");
            success_label_clear.set_label("Successful: 0");
            failed_label_clear.set_label("Failed: 0");
        });

        dialog
    }

    /// Sets the history entries to display
    pub fn set_entries(&self, entries: Vec<ConnectionHistoryEntry>) {
        // Clear existing rows
        while let Some(row) = self.list_box.row_at_index(0) {
            self.list_box.remove(&row);
        }

        // Calculate statistics
        let total = entries.len();
        let successful = entries.iter().filter(|e| e.successful).count();
        let failed = total - successful;

        // Update statistics labels
        self.total_label.set_label(&format!("Total: {total}"));
        self.success_label
            .set_label(&format!("Successful: {successful}"));
        self.failed_label.set_label(&format!("Failed: {failed}"));

        // Add rows
        for entry in &entries {
            let row = self.create_history_row(entry);
            self.list_box.append(&row);
        }

        *self.entries.borrow_mut() = entries;
    }

    /// Sets the connection statistics
    pub fn set_statistics(&self, statistics: Vec<ConnectionStatistics>) {
        *self.statistics.borrow_mut() = statistics;
    }

    /// Creates a list row for a history entry
    fn create_history_row(&self, entry: &ConnectionHistoryEntry) -> ListBoxRow {
        let row = ListBoxRow::new();

        let content = GtkBox::new(Orientation::Horizontal, 12);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Status indicator
        let status_icon = if entry.successful {
            "emblem-ok-symbolic"
        } else {
            "dialog-error-symbolic"
        };
        let status = gtk4::Image::from_icon_name(status_icon);
        if entry.successful {
            status.add_css_class("success");
        } else {
            status.add_css_class("error");
        }
        content.append(&status);

        // Connection info
        let info_box = GtkBox::new(Orientation::Vertical, 4);
        info_box.set_hexpand(true);

        let name_label = Label::builder()
            .label(&entry.connection_name)
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        info_box.append(&name_label);

        let details = format!(
            "{} • {}:{} • {}",
            entry.protocol.to_uppercase(),
            entry.host,
            entry.port,
            entry.username.as_deref().unwrap_or("(no user)")
        );
        let details_label = Label::builder()
            .label(&details)
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label", "caption"])
            .build();
        info_box.append(&details_label);

        content.append(&info_box);

        // Timestamp and duration
        let time_box = GtkBox::new(Orientation::Vertical, 4);
        time_box.set_halign(gtk4::Align::End);

        let time_str = entry.started_at.format("%Y-%m-%d %H:%M").to_string();
        let time_label = Label::builder()
            .label(&time_str)
            .halign(gtk4::Align::End)
            .css_classes(["dim-label", "caption"])
            .build();
        time_box.append(&time_label);

        if let Some(duration) = entry.duration_seconds {
            let duration_str = ConnectionStatistics::format_duration(duration);
            let duration_label = Label::builder()
                .label(&duration_str)
                .halign(gtk4::Align::End)
                .css_classes(["dim-label", "caption"])
                .build();
            time_box.append(&duration_label);
        }

        content.append(&time_box);

        row.set_child(Some(&content));
        row
    }

    /// Filters the list based on search query
    fn filter_list(list_box: &ListBox, entries: &[ConnectionHistoryEntry], query: &str) {
        let mut index = 0;
        while let Some(row) = list_box.row_at_index(index) {
            #[allow(clippy::cast_sign_loss)]
            if let Some(entry) = entries.get(index as usize) {
                let matches = query.is_empty()
                    || entry.connection_name.to_lowercase().contains(query)
                    || entry.host.to_lowercase().contains(query)
                    || entry.protocol.to_lowercase().contains(query)
                    || entry
                        .username
                        .as_ref()
                        .is_some_and(|u| u.to_lowercase().contains(query));
                row.set_visible(matches);
            }
            index += 1;
        }
    }

    /// Connects a callback for when user wants to connect to a history entry
    pub fn connect_on_connect<F>(&self, callback: F)
    where
        F: Fn(&ConnectionHistoryEntry) + 'static,
    {
        *self.on_connect.borrow_mut() = Some(Box::new(callback));
    }

    /// Shows the dialog
    pub fn present(&self) {
        self.window.present();
    }
}
