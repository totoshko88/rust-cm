//! Terminal notebook for session tabs

use gtk4::prelude::*;
use gtk4::{Label, Notebook};

pub struct TerminalNotebook {
    notebook: Notebook,
}

impl TerminalNotebook {
    #[must_use]
    pub fn new() -> Self {
        let notebook = Notebook::new();
        notebook.set_scrollable(true);
        notebook.set_show_border(false);

        let placeholder = Label::new(Some("No active sessions"));
        placeholder.add_css_class("dim-label");
        notebook.append_page(&placeholder, Some(&Label::new(Some("Welcome"))));

        Self { notebook }
    }

    #[must_use]
    pub fn widget(&self) -> &Notebook {
        &self.notebook
    }
}

impl Default for TerminalNotebook {
    fn default() -> Self { Self::new() }
}
