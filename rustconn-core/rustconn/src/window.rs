//! Main application window

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, HeaderBar, Label, Orientation, Paned};

use crate::sidebar::ConnectionSidebar;
use crate::terminal::TerminalNotebook;

pub struct MainWindow {
    window: ApplicationWindow,
    sidebar: ConnectionSidebar,
    terminal_notebook: TerminalNotebook,
}

impl MainWindow {
    #[must_use]
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("RustConn")
            .default_width(1200)
            .default_height(800)
            .build();

        let header_bar = Self::create_header_bar();
        window.set_titlebar(Some(&header_bar));

        let paned = Paned::new(Orientation::Horizontal);
        paned.set_position(280);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);

        let sidebar = ConnectionSidebar::new();
        paned.set_start_child(Some(sidebar.widget()));

        let terminal_notebook = TerminalNotebook::new();
        paned.set_end_child(Some(terminal_notebook.widget()));

        window.set_child(Some(&paned));

        Self { window, sidebar, terminal_notebook }
    }

    fn create_header_bar() -> HeaderBar {
        let header_bar = HeaderBar::new();
        header_bar.set_show_title_buttons(true);

        let title = Label::new(Some("RustConn"));
        title.add_css_class("title");
        header_bar.set_title_widget(Some(&title));

        header_bar
    }

    pub fn present(&self) { self.window.present(); }
    #[must_use] pub fn gtk_window(&self) -> &ApplicationWindow { &self.window }
    #[must_use] pub fn sidebar(&self) -> &ConnectionSidebar { &self.sidebar }
    #[must_use] pub fn terminal_notebook(&self) -> &TerminalNotebook { &self.terminal_notebook }
}
