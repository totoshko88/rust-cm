//! Password prompt dialog for connection authentication
//!
//! Provides a simple dialog for entering credentials when connecting
//! to RDP/VNC sessions that require authentication.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, Entry, Grid, HeaderBar, Label, Orientation, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Result from password dialog
#[derive(Debug, Clone)]
pub struct PasswordDialogResult {
    /// Username (may be updated by user)
    pub username: String,
    /// Password entered by user
    pub password: String,
    /// Domain for Windows authentication
    pub domain: String,
    /// Whether to save credentials
    pub save_credentials: bool,
}

/// Password prompt dialog
pub struct PasswordDialog {
    window: Window,
    username_entry: Entry,
    password_entry: Entry,
    domain_entry: Entry,
    #[allow(dead_code)]
    save_check: CheckButton,
    result: Rc<RefCell<Option<PasswordDialogResult>>>,
}

impl PasswordDialog {
    /// Creates a new password dialog
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new(parent: Option<&impl IsA<gtk4::Window>>) -> Self {
        let window = Window::builder()
            .title("Authentication Required")
            .modal(true)
            .default_width(400)
            .resizable(false)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        // Header bar
        let header = HeaderBar::new();
        let cancel_btn = Button::builder().label("Cancel").build();
        let connect_btn = Button::builder()
            .label("Connect")
            .css_classes(["suggested-action"])
            .build();
        header.pack_start(&cancel_btn);
        header.pack_end(&connect_btn);
        window.set_titlebar(Some(&header));
        window.set_default_widget(Some(&connect_btn));

        // Content
        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Info label
        let info_label = Label::builder()
            .label("Enter credentials for this connection:")
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label"])
            .build();
        content.append(&info_label);

        // Grid for fields
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        content.append(&grid);

        let mut row = 0;

        // Domain
        let domain_label = Label::builder()
            .label("Domain:")
            .halign(gtk4::Align::End)
            .build();
        let domain_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("(optional)")
            .build();
        grid.attach(&domain_label, 0, row, 1, 1);
        grid.attach(&domain_entry, 1, row, 1, 1);
        row += 1;

        // Username
        let username_label = Label::builder()
            .label("Username:")
            .halign(gtk4::Align::End)
            .build();
        let username_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("username")
            .build();
        grid.attach(&username_label, 0, row, 1, 1);
        grid.attach(&username_entry, 1, row, 1, 1);
        row += 1;

        // Password
        let password_label = Label::builder()
            .label("Password:")
            .halign(gtk4::Align::End)
            .build();
        let password_entry = Entry::builder()
            .hexpand(true)
            .visibility(false)
            .input_purpose(gtk4::InputPurpose::Password)
            .placeholder_text("password")
            .build();
        grid.attach(&password_label, 0, row, 1, 1);
        grid.attach(&password_entry, 1, row, 1, 1);
        row += 1;

        // Save credentials checkbox
        let save_check = CheckButton::builder()
            .label("Save credentials to keyring")
            .build();
        grid.attach(&save_check, 1, row, 1, 1);

        window.set_child(Some(&content));

        let result: Rc<RefCell<Option<PasswordDialogResult>>> = Rc::new(RefCell::new(None));

        // Connect cancel
        let window_clone = window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Connect connect button
        let window_clone = window.clone();
        let username_clone = username_entry.clone();
        let password_clone = password_entry.clone();
        let domain_clone = domain_entry.clone();
        let save_clone = save_check.clone();
        let result_clone = result.clone();
        connect_btn.connect_clicked(move |_| {
            *result_clone.borrow_mut() = Some(PasswordDialogResult {
                username: username_clone.text().to_string(),
                password: password_clone.text().to_string(),
                domain: domain_clone.text().to_string(),
                save_credentials: save_clone.is_active(),
            });
            window_clone.close();
        });

        // Connect Enter key in password field
        let connect_btn_clone = connect_btn;
        password_entry.connect_activate(move |_| {
            connect_btn_clone.emit_clicked();
        });

        Self {
            window,
            username_entry,
            password_entry,
            domain_entry,
            save_check,
            result,
        }
    }

    /// Sets the initial username
    pub fn set_username(&self, username: &str) {
        self.username_entry.set_text(username);
    }

    /// Sets the initial domain
    pub fn set_domain(&self, domain: &str) {
        self.domain_entry.set_text(domain);
    }

    /// Sets the connection name in the title
    pub fn set_connection_name(&self, name: &str) {
        self.window.set_title(Some(&format!("Connect to {name}")));
    }

    /// Shows the dialog and calls callback with result
    pub fn show<F: Fn(Option<PasswordDialogResult>) + 'static>(&self, callback: F) {
        let result = self.result.clone();
        let callback = Rc::new(callback);

        self.window.connect_close_request(move |_| {
            let res = result.borrow().clone();
            callback(res);
            glib::Propagation::Proceed
        });

        self.window.present();

        // Focus password field if username is set
        if self.username_entry.text().is_empty() {
            self.username_entry.grab_focus();
        } else {
            self.password_entry.grab_focus();
        }
    }

    /// Returns the window widget
    #[must_use]
    pub const fn window(&self) -> &Window {
        &self.window
    }
}
