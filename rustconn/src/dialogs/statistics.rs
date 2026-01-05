//! Connection statistics dialog
//!
//! This module provides a dialog for viewing detailed connection statistics.

use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use gtk4::{Button, Grid, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use rustconn_core::models::ConnectionStatistics;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

/// Connection statistics dialog
pub struct StatisticsDialog {
    window: adw::Window,
    content_box: GtkBox,
    on_clear: Rc<RefCell<Option<Box<dyn Fn() + 'static>>>>,
}

impl StatisticsDialog {
    /// Creates a new statistics dialog
    #[must_use]
    pub fn new(parent: Option<&impl IsA<gtk4::Window>>) -> Self {
        let window = adw::Window::builder()
            .title("Connection Statistics")
            .default_width(750)
            .default_height(500)
            .modal(true)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        // Header bar with Close/Reset buttons (GNOME HIG)
        let header = adw::HeaderBar::new();
        header.set_show_end_title_buttons(false);
        header.set_show_start_title_buttons(false);
        let close_btn = Button::builder().label("Close").build();
        let reset_btn = Button::builder()
            .label("Reset")
            .css_classes(["destructive-action"])
            .build();
        header.pack_start(&close_btn);
        header.pack_end(&reset_btn);

        // Close button handler
        let window_clone = window.clone();
        close_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Scrolled content
        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let content_box = GtkBox::new(Orientation::Vertical, 16);
        content_box.set_margin_top(16);
        content_box.set_margin_bottom(16);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);

        scrolled.set_child(Some(&content_box));

        // Use GtkBox with HeaderBar for adw::Window (libadwaita 0.8)
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header);
        main_box.append(&scrolled);
        window.set_content(Some(&main_box));

        let on_clear: Rc<RefCell<Option<Box<dyn Fn() + 'static>>>> = Rc::new(RefCell::new(None));

        // Reset button handler
        let on_clear_clone = on_clear.clone();
        let window_clone = window.clone();
        reset_btn.connect_clicked(move |_| {
            if let Some(ref callback) = *on_clear_clone.borrow() {
                callback();
            }
            window_clone.close();
        });

        Self {
            window,
            content_box,
            on_clear,
        }
    }

    /// Connects a callback for when user clears statistics
    pub fn connect_on_clear<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        *self.on_clear.borrow_mut() = Some(Box::new(callback));
    }

    /// Sets the statistics to display for a single connection
    pub fn set_connection_statistics(&self, name: &str, stats: &ConnectionStatistics) {
        // Clear existing content
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }

        // Connection name header
        let header = Label::builder()
            .label(name)
            .css_classes(["title-1"])
            .halign(gtk4::Align::Start)
            .build();
        self.content_box.append(&header);

        // Statistics grid
        let grid = self.create_stats_grid(stats);
        self.content_box.append(&grid);

        // Success rate visualization
        let rate_box = self.create_success_rate_box(stats);
        self.content_box.append(&rate_box);
    }

    /// Sets statistics for multiple connections (overview)
    pub fn set_overview_statistics(&self, stats: &[(String, ConnectionStatistics)]) {
        // Clear existing content
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }

        // Overview header
        let header = Label::builder()
            .label("Connection Statistics Overview")
            .css_classes(["title-1"])
            .halign(gtk4::Align::Start)
            .build();
        self.content_box.append(&header);

        // Calculate totals
        let total_connections: u32 = stats.iter().map(|(_, s)| s.total_connections).sum();
        let total_successful: u32 = stats.iter().map(|(_, s)| s.successful_connections).sum();
        let total_failed: u32 = stats.iter().map(|(_, s)| s.failed_connections).sum();
        let total_duration: i64 = stats.iter().map(|(_, s)| s.total_duration_seconds).sum();

        // Summary box
        let summary = GtkBox::new(Orientation::Horizontal, 24);
        summary.set_halign(gtk4::Align::Center);
        summary.set_margin_top(16);
        summary.set_margin_bottom(16);

        let total_box = self.create_stat_card("Total Sessions", &total_connections.to_string());
        let success_box = self.create_stat_card("Successful", &total_successful.to_string());
        let failed_box = self.create_stat_card("Failed", &total_failed.to_string());
        let duration_box = self.create_stat_card(
            "Total Time",
            &ConnectionStatistics::format_duration(total_duration),
        );

        summary.append(&total_box);
        summary.append(&success_box);
        summary.append(&failed_box);
        summary.append(&duration_box);
        self.content_box.append(&summary);

        // Per-connection breakdown
        if !stats.is_empty() {
            let breakdown_header = Label::builder()
                .label("Per-Connection Breakdown")
                .css_classes(["title-3"])
                .halign(gtk4::Align::Start)
                .margin_top(16)
                .build();
            self.content_box.append(&breakdown_header);

            for (name, stat) in stats {
                let row = self.create_connection_row(name, stat);
                self.content_box.append(&row);
            }
        }
    }

    /// Creates a statistics grid for a single connection
    fn create_stats_grid(&self, stats: &ConnectionStatistics) -> Grid {
        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(16)
            .margin_top(16)
            .build();

        let rows = [
            ("Total Connections:", stats.total_connections.to_string()),
            ("Successful:", stats.successful_connections.to_string()),
            ("Failed:", stats.failed_connections.to_string()),
            ("Success Rate:", format!("{:.1}%", stats.success_rate())),
            (
                "Total Time Connected:",
                ConnectionStatistics::format_duration(stats.total_duration_seconds),
            ),
            (
                "Average Session:",
                ConnectionStatistics::format_duration(stats.average_duration_seconds),
            ),
            (
                "Longest Session:",
                ConnectionStatistics::format_duration(stats.longest_session_seconds),
            ),
            (
                "Shortest Session:",
                stats
                    .shortest_session_seconds
                    .map_or_else(|| "N/A".to_string(), ConnectionStatistics::format_duration),
            ),
            (
                "First Connected:",
                stats.first_connected.map_or_else(
                    || "Never".to_string(),
                    |dt| dt.format("%Y-%m-%d %H:%M").to_string(),
                ),
            ),
            (
                "Last Connected:",
                stats.last_connected.map_or_else(
                    || "Never".to_string(),
                    |dt| dt.format("%Y-%m-%d %H:%M").to_string(),
                ),
            ),
        ];

        for (i, (label, value)) in rows.iter().enumerate() {
            let label_widget = Label::builder()
                .label(*label)
                .halign(gtk4::Align::End)
                .css_classes(["dim-label"])
                .build();

            let value_widget = Label::builder()
                .label(value)
                .halign(gtk4::Align::Start)
                .selectable(true)
                .build();

            #[allow(clippy::cast_possible_truncation)]
            let row = i as i32;
            grid.attach(&label_widget, 0, row, 1, 1);
            grid.attach(&value_widget, 1, row, 1, 1);
        }

        grid
    }

    /// Creates a success rate visualization box
    fn create_success_rate_box(&self, stats: &ConnectionStatistics) -> GtkBox {
        let container = GtkBox::new(Orientation::Vertical, 8);
        container.set_margin_top(16);

        let label = Label::builder()
            .label("Success Rate")
            .css_classes(["title-4"])
            .halign(gtk4::Align::Start)
            .build();
        container.append(&label);

        // Progress bar for success rate
        let progress = gtk4::ProgressBar::builder()
            .fraction(stats.success_rate() / 100.0)
            .show_text(true)
            .text(format!("{:.1}%", stats.success_rate()))
            .build();

        // Color based on success rate
        if stats.success_rate() >= 90.0 {
            progress.add_css_class("success");
        } else if stats.success_rate() >= 70.0 {
            progress.add_css_class("warning");
        } else {
            progress.add_css_class("error");
        }

        container.append(&progress);
        container
    }

    /// Creates a stat card widget
    fn create_stat_card(&self, title: &str, value: &str) -> GtkBox {
        let card = GtkBox::new(Orientation::Vertical, 4);
        card.add_css_class("card");
        card.set_margin_start(8);
        card.set_margin_end(8);

        let value_label = Label::builder()
            .label(value)
            .css_classes(["title-2"])
            .build();

        let title_label = Label::builder()
            .label(title)
            .css_classes(["dim-label", "caption"])
            .build();

        card.append(&value_label);
        card.append(&title_label);
        card
    }

    /// Creates a row for per-connection breakdown
    fn create_connection_row(&self, name: &str, stats: &ConnectionStatistics) -> GtkBox {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_top(8);
        row.set_margin_bottom(8);
        row.add_css_class("card");

        let name_label = Label::builder()
            .label(name)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();
        row.append(&name_label);

        let sessions_label = Label::builder()
            .label(format!("{} sessions", stats.total_connections))
            .css_classes(["dim-label"])
            .build();
        row.append(&sessions_label);

        let rate_label = Label::builder()
            .label(format!("{:.0}%", stats.success_rate()))
            .width_chars(5)
            .build();

        if stats.success_rate() >= 90.0 {
            rate_label.add_css_class("success");
        } else if stats.success_rate() >= 70.0 {
            rate_label.add_css_class("warning");
        } else {
            rate_label.add_css_class("error");
        }

        row.append(&rate_label);

        let duration_label = Label::builder()
            .label(ConnectionStatistics::format_duration(
                stats.total_duration_seconds,
            ))
            .css_classes(["dim-label"])
            .width_chars(10)
            .build();
        row.append(&duration_label);

        row
    }

    /// Shows the dialog
    pub fn present(&self) {
        self.window.present();
    }
}

/// Creates statistics for a connection that has no history yet
#[must_use]
pub fn empty_statistics(connection_id: Uuid) -> ConnectionStatistics {
    ConnectionStatistics::new(connection_id)
}
