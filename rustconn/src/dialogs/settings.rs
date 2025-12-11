//! Settings dialog for application preferences
//!
//! Provides a GTK4 dialog for configuring terminal settings, logging options,
//! and secret storage preferences.
//!
//! Updated for GTK 4.10+ compatibility using DropDown instead of ComboBoxText
//! and Window instead of Dialog.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, Frame, Grid, HeaderBar, Label, Notebook,
    Orientation, SpinButton, StringList, Window,
};
use rustconn_core::config::{
    AppSettings, LoggingSettings, SecretBackendType, SecretSettings, TerminalSettings, UiSettings,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// Settings dialog for application preferences
pub struct SettingsDialog {
    window: Window,
    // Terminal settings
    font_family_entry: Entry,
    font_size_spin: SpinButton,
    scrollback_spin: SpinButton,
    // Logging settings
    logging_enabled: CheckButton,
    log_dir_entry: Entry,
    retention_spin: SpinButton,
    // Secret settings
    secret_backend_dropdown: DropDown,
    enable_fallback: CheckButton,
    // UI settings
    remember_geometry: CheckButton,
    // Current settings
    settings: Rc<RefCell<AppSettings>>,
    // Callback
    on_save: Rc<RefCell<Option<Box<dyn Fn(Option<AppSettings>)>>>>,
}


impl SettingsDialog {
    /// Creates a new settings dialog
    #[must_use]
    pub fn new(parent: Option<&Window>) -> Self {
        // Create window instead of deprecated Dialog
        let window = Window::builder()
            .title("Settings")
            .modal(true)
            .default_width(500)
            .default_height(450)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        // Create header bar with Cancel/Save buttons
        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let save_btn = Button::builder().label("Save").css_classes(["suggested-action"]).build();
        header.pack_start(&cancel_btn);
        header.pack_end(&save_btn);
        window.set_titlebar(Some(&header));

        // Create main content area
        let content = GtkBox::new(Orientation::Vertical, 0);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Create notebook for tabs
        let notebook = Notebook::new();
        content.append(&notebook);
        window.set_child(Some(&content));

        // === Terminal Tab ===
        let (terminal_page, font_family_entry, font_size_spin, scrollback_spin) =
            Self::create_terminal_tab();
        notebook.append_page(&terminal_page, Some(&Label::new(Some("Terminal"))));

        // === Logging Tab ===
        let (logging_page, logging_enabled, log_dir_entry, retention_spin) =
            Self::create_logging_tab();
        notebook.append_page(&logging_page, Some(&Label::new(Some("Logging"))));

        // === Secrets Tab ===
        let (secrets_page, secret_backend_dropdown, enable_fallback) = Self::create_secrets_tab();
        notebook.append_page(&secrets_page, Some(&Label::new(Some("Secrets"))));

        // === UI Tab ===
        let (ui_page, remember_geometry) = Self::create_ui_tab();
        notebook.append_page(&ui_page, Some(&Label::new(Some("Interface"))));

        let on_save: Rc<RefCell<Option<Box<dyn Fn(Option<AppSettings>)>>>> = Rc::new(RefCell::new(None));

        // Connect cancel button
        let window_clone = window.clone();
        let on_save_clone = on_save.clone();
        cancel_btn.connect_clicked(move |_| {
            if let Some(ref cb) = *on_save_clone.borrow() {
                cb(None);
            }
            window_clone.close();
        });

        Self {
            window,
            font_family_entry,
            font_size_spin,
            scrollback_spin,
            logging_enabled,
            log_dir_entry,
            retention_spin,
            secret_backend_dropdown,
            enable_fallback,
            remember_geometry,
            settings: Rc::new(RefCell::new(AppSettings::default())),
            on_save,
        }
    }

    fn create_terminal_tab() -> (Frame, Entry, SpinButton, SpinButton) {
        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let mut row = 0;

        // Font family
        let font_label = Label::builder()
            .label("Font Family:")
            .halign(gtk4::Align::End)
            .build();
        let font_family_entry = Entry::builder()
            .hexpand(true)
            .text("Monospace")
            .build();
        grid.attach(&font_label, 0, row, 1, 1);
        grid.attach(&font_family_entry, 1, row, 1, 1);
        row += 1;

        // Font size
        let size_label = Label::builder()
            .label("Font Size:")
            .halign(gtk4::Align::End)
            .build();
        let size_adj = gtk4::Adjustment::new(12.0, 6.0, 72.0, 1.0, 2.0, 0.0);
        let font_size_spin = SpinButton::builder()
            .adjustment(&size_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&size_label, 0, row, 1, 1);
        grid.attach(&font_size_spin, 1, row, 1, 1);
        row += 1;

        // Scrollback lines
        let scrollback_label = Label::builder()
            .label("Scrollback Lines:")
            .halign(gtk4::Align::End)
            .build();
        let scrollback_adj = gtk4::Adjustment::new(10000.0, 100.0, 1_000_000.0, 100.0, 1000.0, 0.0);
        let scrollback_spin = SpinButton::builder()
            .adjustment(&scrollback_adj)
            .climb_rate(100.0)
            .digits(0)
            .build();
        grid.attach(&scrollback_label, 0, row, 1, 1);
        grid.attach(&scrollback_spin, 1, row, 1, 1);

        let frame = Frame::builder()
            .label("Terminal Settings")
            .child(&grid)
            .margin_top(12)
            .build();

        (frame, font_family_entry, font_size_spin, scrollback_spin)
    }


    fn create_logging_tab() -> (Frame, CheckButton, Entry, SpinButton) {
        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        // Enable logging checkbox
        let logging_enabled = CheckButton::builder()
            .label("Enable session logging")
            .build();
        vbox.append(&logging_enabled);

        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .margin_top(8)
            .build();

        let mut row = 0;

        // Log directory
        let dir_label = Label::builder()
            .label("Log Directory:")
            .halign(gtk4::Align::End)
            .build();
        let log_dir_entry = Entry::builder()
            .hexpand(true)
            .text("logs")
            .placeholder_text("Relative to config dir or absolute path")
            .build();
        grid.attach(&dir_label, 0, row, 1, 1);
        grid.attach(&log_dir_entry, 1, row, 1, 1);
        row += 1;

        // Retention days
        let retention_label = Label::builder()
            .label("Retention (days):")
            .halign(gtk4::Align::End)
            .build();
        let retention_adj = gtk4::Adjustment::new(30.0, 1.0, 365.0, 1.0, 7.0, 0.0);
        let retention_spin = SpinButton::builder()
            .adjustment(&retention_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&retention_label, 0, row, 1, 1);
        grid.attach(&retention_spin, 1, row, 1, 1);

        vbox.append(&grid);

        // Connect checkbox to enable/disable other controls
        let dir_entry_clone = log_dir_entry.clone();
        let retention_clone = retention_spin.clone();
        logging_enabled.connect_toggled(move |check| {
            let enabled = check.is_active();
            dir_entry_clone.set_sensitive(enabled);
            retention_clone.set_sensitive(enabled);
        });

        // Initial state
        log_dir_entry.set_sensitive(false);
        retention_spin.set_sensitive(false);

        let frame = Frame::builder()
            .label("Session Logging")
            .child(&vbox)
            .margin_top(12)
            .build();

        (frame, logging_enabled, log_dir_entry, retention_spin)
    }

    fn create_secrets_tab() -> (Frame, DropDown, CheckButton) {
        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .build();

        // Preferred backend - using DropDown
        let backend_label = Label::builder()
            .label("Preferred Backend:")
            .halign(gtk4::Align::End)
            .build();
        let backend_list = StringList::new(&["KeePassXC", "libsecret (GNOME Keyring/KDE Wallet)"]);
        let secret_backend_dropdown = DropDown::new(Some(backend_list), gtk4::Expression::NONE);
        secret_backend_dropdown.set_selected(0); // KeePassXC by default
        grid.attach(&backend_label, 0, 0, 1, 1);
        grid.attach(&secret_backend_dropdown, 1, 0, 1, 1);

        vbox.append(&grid);

        // Enable fallback
        let enable_fallback = CheckButton::builder()
            .label("Enable fallback to libsecret if KeePassXC unavailable")
            .active(true)
            .margin_top(8)
            .build();
        vbox.append(&enable_fallback);

        // Info label
        let info = Label::builder()
            .label("KeePassXC requires the browser integration feature to be enabled.\nlibsecret uses your system's keyring (GNOME Keyring or KDE Wallet).")
            .halign(gtk4::Align::Start)
            .wrap(true)
            .css_classes(["dim-label"])
            .margin_top(12)
            .build();
        vbox.append(&info);

        let frame = Frame::builder()
            .label("Secret Storage")
            .child(&vbox)
            .margin_top(12)
            .build();

        (frame, secret_backend_dropdown, enable_fallback)
    }

    fn create_ui_tab() -> (Frame, CheckButton) {
        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let remember_geometry = CheckButton::builder()
            .label("Remember window size and position")
            .active(true)
            .build();
        vbox.append(&remember_geometry);

        let frame = Frame::builder()
            .label("Window Settings")
            .child(&vbox)
            .margin_top(12)
            .build();

        (frame, remember_geometry)
    }


    /// Populates the dialog with existing settings
    pub fn set_settings(&self, settings: &AppSettings) {
        *self.settings.borrow_mut() = settings.clone();

        // Terminal settings
        self.font_family_entry.set_text(&settings.terminal.font_family);
        self.font_size_spin.set_value(f64::from(settings.terminal.font_size));
        self.scrollback_spin.set_value(f64::from(settings.terminal.scrollback_lines));

        // Logging settings
        self.logging_enabled.set_active(settings.logging.enabled);
        self.log_dir_entry.set_text(&settings.logging.log_directory.to_string_lossy());
        self.retention_spin.set_value(f64::from(settings.logging.retention_days));
        self.log_dir_entry.set_sensitive(settings.logging.enabled);
        self.retention_spin.set_sensitive(settings.logging.enabled);

        // Secret settings - using dropdown index
        let backend_idx = match settings.secrets.preferred_backend {
            SecretBackendType::KeePassXc => 0,
            SecretBackendType::LibSecret => 1,
        };
        self.secret_backend_dropdown.set_selected(backend_idx);
        self.enable_fallback.set_active(settings.secrets.enable_fallback);

        // UI settings
        self.remember_geometry.set_active(settings.ui.remember_window_geometry);
    }

    /// Builds settings from the dialog fields
    fn build_settings(&self) -> AppSettings {
        let terminal = TerminalSettings {
            font_family: self.font_family_entry.text().to_string(),
            font_size: self.font_size_spin.value() as u32,
            scrollback_lines: self.scrollback_spin.value() as u32,
        };

        let logging = LoggingSettings {
            enabled: self.logging_enabled.is_active(),
            log_directory: PathBuf::from(self.log_dir_entry.text().to_string()),
            retention_days: self.retention_spin.value() as u32,
        };

        // Map dropdown index to backend type: 0->KeePassXC, 1->libsecret
        let preferred_backend = match self.secret_backend_dropdown.selected() {
            1 => SecretBackendType::LibSecret,
            _ => SecretBackendType::KeePassXc,
        };
        let secrets = SecretSettings {
            preferred_backend,
            enable_fallback: self.enable_fallback.is_active(),
        };

        // Preserve existing window dimensions
        let existing = self.settings.borrow();
        let ui = UiSettings {
            remember_window_geometry: self.remember_geometry.is_active(),
            window_width: existing.ui.window_width,
            window_height: existing.ui.window_height,
            sidebar_width: existing.ui.sidebar_width,
        };

        AppSettings {
            terminal,
            logging,
            secrets,
            ui,
        }
    }

    /// Runs the dialog and calls the callback with the result
    pub fn run<F: Fn(Option<AppSettings>) + 'static>(&self, cb: F) {
        // Store callback
        *self.on_save.borrow_mut() = Some(Box::new(cb));

        // Get the save button from header bar and connect it
        if let Some(titlebar) = self.window.titlebar() {
            if let Some(header) = titlebar.downcast_ref::<HeaderBar>() {
                if let Some(save_btn) = header.last_child() {
                    if let Some(btn) = save_btn.downcast_ref::<Button>() {
                        let window = self.window.clone();
                        let on_save = self.on_save.clone();
                        let font_family_entry = self.font_family_entry.clone();
                        let font_size_spin = self.font_size_spin.clone();
                        let scrollback_spin = self.scrollback_spin.clone();
                        let logging_enabled = self.logging_enabled.clone();
                        let log_dir_entry = self.log_dir_entry.clone();
                        let retention_spin = self.retention_spin.clone();
                        let secret_backend_dropdown = self.secret_backend_dropdown.clone();
                        let enable_fallback = self.enable_fallback.clone();
                        let remember_geometry = self.remember_geometry.clone();
                        let settings = self.settings.clone();

                        btn.connect_clicked(move |_| {
                            let terminal = TerminalSettings {
                                font_family: font_family_entry.text().to_string(),
                                font_size: font_size_spin.value() as u32,
                                scrollback_lines: scrollback_spin.value() as u32,
                            };

                            let logging = LoggingSettings {
                                enabled: logging_enabled.is_active(),
                                log_directory: PathBuf::from(log_dir_entry.text().to_string()),
                                retention_days: retention_spin.value() as u32,
                            };

                            // Map dropdown index to backend type
                            let preferred_backend = match secret_backend_dropdown.selected() {
                                1 => SecretBackendType::LibSecret,
                                _ => SecretBackendType::KeePassXc,
                            };
                            let secrets = SecretSettings {
                                preferred_backend,
                                enable_fallback: enable_fallback.is_active(),
                            };

                            let existing = settings.borrow();
                            let ui = UiSettings {
                                remember_window_geometry: remember_geometry.is_active(),
                                window_width: existing.ui.window_width,
                                window_height: existing.ui.window_height,
                                sidebar_width: existing.ui.sidebar_width,
                            };

                            let new_settings = AppSettings {
                                terminal,
                                logging,
                                secrets,
                                ui,
                            };

                            if let Some(ref cb) = *on_save.borrow() {
                                cb(Some(new_settings));
                            }
                            window.close();
                        });
                    }
                }
            }
        }

        self.window.present();
    }

    /// Returns a reference to the underlying window
    #[must_use]
    pub fn window(&self) -> &Window {
        &self.window
    }
}
