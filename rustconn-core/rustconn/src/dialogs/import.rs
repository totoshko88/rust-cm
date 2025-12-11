//! Import dialog
use gtk4::prelude::*;
use gtk4::{Dialog, ResponseType, Window};
use rustconn_core::import::ImportResult;

pub struct ImportDialog { dialog: Dialog }

impl ImportDialog {
    #[must_use]
    #[allow(deprecated)]
    pub fn new(parent: Option<&Window>) -> Self {
        let dialog = Dialog::builder().title("Import").modal(true).default_width(500).default_height(400).build();
        if let Some(p) = parent { dialog.set_transient_for(Some(p)); }
        dialog.add_button("Close", ResponseType::Close);
        Self { dialog }
    }
    #[allow(deprecated)]
    pub fn run<F: Fn(Option<ImportResult>) + 'static>(&self, cb: F) {
        let d = self.dialog.clone();
        d.connect_response(move |dlg, _| { cb(None); dlg.close(); });
        self.dialog.present();
    }
    #[must_use] pub fn dialog(&self) -> &Dialog { &self.dialog }
}
