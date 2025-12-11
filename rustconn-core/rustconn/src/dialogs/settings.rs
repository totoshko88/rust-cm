//! Settings dialog
use gtk4::prelude::*;
use gtk4::{Dialog, ResponseType, Window};
use rustconn_core::AppSettings;

pub struct SettingsDialog { dialog: Dialog }

impl SettingsDialog {
    #[must_use]
    #[allow(deprecated)]
    pub fn new(parent: Option<&Window>) -> Self {
        let dialog = Dialog::builder().title("Settings").modal(true).default_width(500).default_height(400).build();
        if let Some(p) = parent { dialog.set_transient_for(Some(p)); }
        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.add_button("OK", ResponseType::Accept);
        Self { dialog }
    }
    pub fn set_settings(&self, _s: &AppSettings) {}
    #[allow(deprecated)]
    pub fn run<F: Fn(Option<AppSettings>) + 'static>(&self, cb: F) {
        let d = self.dialog.clone();
        d.connect_response(move |dlg, r| { cb(if r == ResponseType::Accept { None } else { None }); dlg.close(); });
        self.dialog.present();
    }
    #[must_use] pub fn dialog(&self) -> &Dialog { &self.dialog }
}
