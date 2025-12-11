//! Import dialog for importing connections from external sources
//!
//! Provides a GTK4 dialog with source selection, progress display,
//! and result summary for importing connections from Asbru-CM, SSH config,
//! Remmina, and Ansible inventory files.
//!
//! Updated for GTK 4.10+ compatibility using Window instead of Dialog.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Frame, HeaderBar, Label, ListBox, ListBoxRow, Orientation, ProgressBar,
    ScrolledWindow, Separator, Stack, Window,
};
use rustconn_core::import::{
    AnsibleInventoryImporter, AsbruImporter, ImportResult, ImportSource, RemminaImporter,
    SshConfigImporter,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Import dialog for importing connections from external sources
pub struct ImportDialog {
    window: Window,
    stack: Stack,
    source_list: ListBox,
    progress_bar: ProgressBar,
    progress_label: Label,
    result_label: Label,
    result_details: Label,
    import_button: Button,
    close_button: Button,
    result: Rc<RefCell<Option<ImportResult>>>,
    source_name: Rc<RefCell<String>>,
    on_complete: Rc<RefCell<Option<Box<dyn Fn(Option<ImportResult>)>>>>,
    on_complete_with_source: Rc<RefCell<Option<Box<dyn Fn(Option<ImportResult>, String)>>>>,
}


impl ImportDialog {
    /// Creates a new import dialog
    #[must_use]
    pub fn new(parent: Option<&Window>) -> Self {
        // Create window instead of deprecated Dialog
        let window = Window::builder()
            .title("Import Connections")
            .modal(true)
            .default_width(500)
            .default_height(450)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        // Create header bar with Close/Import buttons
        let header = HeaderBar::new();
        let close_button = Button::builder().label("Close").build();
        let import_button = Button::builder().label("Import").css_classes(["suggested-action"]).build();
        header.pack_start(&close_button);
        header.pack_end(&import_button);
        window.set_titlebar(Some(&header));

        // Create main content area
        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Create stack for different views
        let stack = Stack::new();
        stack.set_vexpand(true);
        content.append(&stack);
        window.set_child(Some(&content));

        // === Source Selection Page ===
        let source_page = Self::create_source_page();
        stack.add_named(&source_page.0, Some("source"));

        // === Progress Page ===
        let (progress_page, progress_bar, progress_label) = Self::create_progress_page();
        stack.add_named(&progress_page, Some("progress"));

        // === Result Page ===
        let (result_page, result_label, result_details) = Self::create_result_page();
        stack.add_named(&result_page, Some("result"));

        // Set initial page
        stack.set_visible_child_name("source");

        let on_complete: Rc<RefCell<Option<Box<dyn Fn(Option<ImportResult>)>>>> = Rc::new(RefCell::new(None));
        let on_complete_with_source: Rc<RefCell<Option<Box<dyn Fn(Option<ImportResult>, String)>>>> = Rc::new(RefCell::new(None));
        let source_name: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        // Connect close button
        let window_clone = window.clone();
        let on_complete_clone = on_complete.clone();
        let on_complete_with_source_clone = on_complete_with_source.clone();
        close_button.connect_clicked(move |_| {
            if let Some(ref cb) = *on_complete_clone.borrow() {
                cb(None);
            }
            if let Some(ref cb) = *on_complete_with_source_clone.borrow() {
                cb(None, String::new());
            }
            window_clone.close();
        });

        let dialog = Self {
            window,
            stack,
            source_list: source_page.1,
            progress_bar,
            progress_label,
            result_label,
            result_details,
            import_button,
            close_button,
            result: Rc::new(RefCell::new(None)),
            source_name,
            on_complete,
            on_complete_with_source,
        };

        // Wire up source selection to import button state (Requirement 5.1)
        dialog.connect_source_selection_to_import_button();

        dialog
    }

    /// Connects source list selection changes to import button enabled state
    /// 
    /// When a source is selected, the import button is enabled.
    /// When no source is selected or the selected source is unavailable, the button is disabled.
    fn connect_source_selection_to_import_button(&self) {
        let import_button = self.import_button.clone();
        
        // Update button state based on initial selection
        self.update_import_button_state();
        
        // Connect to selection changes
        self.source_list.connect_row_selected(move |_, row| {
            let should_enable = row.map_or(false, |r| r.is_sensitive());
            import_button.set_sensitive(should_enable);
        });
    }

    /// Updates the import button state based on current selection
    fn update_import_button_state(&self) {
        let should_enable = self.source_list
            .selected_row()
            .is_some_and(|row| row.is_sensitive());
        self.import_button.set_sensitive(should_enable);
    }

    fn create_source_page() -> (GtkBox, ListBox) {
        let vbox = GtkBox::new(Orientation::Vertical, 12);

        let header = Label::builder()
            .label("Select Import Source")
            .css_classes(["title-3"])
            .halign(gtk4::Align::Start)
            .build();
        vbox.append(&header);

        let description = Label::builder()
            .label("Choose the source from which to import connections:")
            .halign(gtk4::Align::Start)
            .wrap(true)
            .build();
        vbox.append(&description);

        // Create list box for sources
        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();

        // Add import sources
        let sources: Vec<(&str, &str, &str, bool)> = vec![
            ("ssh_config", "SSH Config", "Import from ~/.ssh/config", SshConfigImporter::new().is_available()),
            ("asbru", "Asbru-CM", "Import from Asbru-CM/PAC Manager config", AsbruImporter::new().is_available()),
            ("asbru_file", "Asbru-CM YAML File", "Import from a specific Asbru-CM YAML file", true),
            ("remmina", "Remmina", "Import from Remmina connection files", RemminaImporter::new().is_available()),
            ("ansible", "Ansible Inventory", "Import from Ansible inventory files", AnsibleInventoryImporter::new().is_available()),
        ];

        for (id, name, desc, available) in sources {
            let row = Self::create_source_row(id, name, desc, available);
            list_box.append(&row);
        }

        // Select first available row
        if let Some(row) = list_box.row_at_index(0) {
            list_box.select_row(Some(&row));
        }

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .child(&list_box)
            .build();

        let frame = Frame::builder().child(&scrolled).build();
        vbox.append(&frame);

        (vbox, list_box)
    }

    fn create_source_row(id: &str, name: &str, description: &str, available: bool) -> ListBoxRow {
        let hbox = GtkBox::new(Orientation::Horizontal, 12);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);
        hbox.set_margin_start(12);
        hbox.set_margin_end(12);

        let vbox = GtkBox::new(Orientation::Vertical, 4);
        vbox.set_hexpand(true);

        let name_label = Label::builder()
            .label(name)
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        vbox.append(&name_label);

        let desc_label = Label::builder()
            .label(description)
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label"])
            .build();
        vbox.append(&desc_label);

        hbox.append(&vbox);

        // Status indicator
        let status = if available {
            Label::builder()
                .label("Available")
                .css_classes(["success"])
                .build()
        } else {
            Label::builder()
                .label("Not Found")
                .css_classes(["dim-label"])
                .build()
        };
        hbox.append(&status);

        let row = ListBoxRow::builder()
            .child(&hbox)
            .sensitive(available)
            .name(id)
            .build();

        row
    }


    fn create_progress_page() -> (GtkBox, ProgressBar, Label) {
        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_valign(gtk4::Align::Center);

        let header = Label::builder()
            .label("Importing...")
            .css_classes(["title-3"])
            .build();
        vbox.append(&header);

        let progress_bar = ProgressBar::builder()
            .show_text(true)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        vbox.append(&progress_bar);

        let progress_label = Label::builder()
            .label("Scanning for connections...")
            .css_classes(["dim-label"])
            .build();
        vbox.append(&progress_label);

        (vbox, progress_bar, progress_label)
    }

    fn create_result_page() -> (GtkBox, Label, Label) {
        let vbox = GtkBox::new(Orientation::Vertical, 12);

        let header = Label::builder()
            .label("Import Complete")
            .css_classes(["title-3"])
            .halign(gtk4::Align::Start)
            .build();
        vbox.append(&header);

        let result_label = Label::builder()
            .halign(gtk4::Align::Start)
            .wrap(true)
            .build();
        vbox.append(&result_label);

        vbox.append(&Separator::new(Orientation::Horizontal));

        let details_header = Label::builder()
            .label("Details")
            .css_classes(["heading"])
            .halign(gtk4::Align::Start)
            .margin_top(8)
            .build();
        vbox.append(&details_header);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let result_details = Label::builder()
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Start)
            .wrap(true)
            .selectable(true)
            .build();
        scrolled.set_child(Some(&result_details));

        vbox.append(&scrolled);

        (vbox, result_label, result_details)
    }

    /// Gets the selected import source ID
    /// 
    /// Returns the source ID string (e.g., "ssh_config", "asbru") if a source is selected,
    /// or None if no source is selected.
    #[must_use]
    pub fn get_selected_source(&self) -> Option<String> {
        self.source_list
            .selected_row()
            .and_then(|row| {
                let name = row.widget_name();
                if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                }
            })
    }

    /// Gets the display name for a source ID
    #[must_use]
    pub fn get_source_display_name(source_id: &str) -> &'static str {
        match source_id {
            "ssh_config" => "SSH Config",
            "asbru" => "Asbru-CM",
            "asbru_file" => "Asbru-CM File",
            "remmina" => "Remmina",
            "ansible" => "Ansible",
            _ => "Unknown",
        }
    }

    /// Performs the import operation for the given source ID
    /// 
    /// This method executes the appropriate importer based on the source ID
    /// and returns the import result containing connections, groups, skipped entries, and errors.
    #[must_use]
    pub fn do_import(&self, source_id: &str) -> ImportResult {
        match source_id {
            "ssh_config" => {
                let importer = SshConfigImporter::new();
                importer.import().unwrap_or_default()
            }
            "asbru" => {
                let importer = AsbruImporter::new();
                importer.import().unwrap_or_default()
            }
            "remmina" => {
                let importer = RemminaImporter::new();
                importer.import().unwrap_or_default()
            }
            "ansible" => {
                let importer = AnsibleInventoryImporter::new();
                importer.import().unwrap_or_default()
            }
            _ => ImportResult::default(),
        }
    }

    /// Updates the result page with import results
    /// 
    /// Displays a summary of successful imports and detailed information about:
    /// - Successfully imported connections and groups
    /// - Skipped entries with reasons (Requirement 5.2)
    /// - Errors encountered during import (Requirement 5.3)
    pub fn show_results(&self, result: &ImportResult) {
        self.show_results_with_source(result, None);
    }

    /// Updates the result page with import results and optional source name
    /// 
    /// Displays a summary including the source name if provided.
    pub fn show_results_with_source(&self, result: &ImportResult, source_name: Option<&str>) {
        let summary = if let Some(name) = source_name {
            format!(
                "Successfully imported {} connection(s) and {} group(s).\nConnections will be added to '{}' group.",
                result.connections.len(),
                result.groups.len(),
                format!("{name} Import")
            )
        } else {
            format!(
                "Successfully imported {} connection(s) and {} group(s).",
                result.connections.len(),
                result.groups.len()
            )
        };
        self.result_label.set_text(&summary);

        let details = Self::format_import_details(result);
        self.result_details.set_text(&details);
    }

    /// Formats import result details into a displayable string
    #[must_use]
    pub fn format_import_details(result: &ImportResult) -> String {
        let mut details = String::new();

        // List imported connections
        if !result.connections.is_empty() {
            details.push_str("Imported connections:\n");
            for conn in &result.connections {
                details.push_str(&format!("  • {} ({}:{})\n", conn.name, conn.host, conn.port));
            }
            details.push('\n');
        }

        // List skipped entries (Requirement 5.2)
        if !result.skipped.is_empty() {
            details.push_str(&format!("Skipped {} entries:\n", result.skipped.len()));
            for skipped in &result.skipped {
                details.push_str(&format!("  • {}: {}\n", skipped.identifier, skipped.reason));
            }
            details.push('\n');
        }

        // List errors (Requirement 5.3)
        if !result.errors.is_empty() {
            details.push_str(&format!("Errors ({}):\n", result.errors.len()));
            for error in &result.errors {
                details.push_str(&format!("  • {error}\n"));
            }
        }

        if details.is_empty() {
            details = "No connections found in the selected source.".to_string();
        }

        details
    }


    /// Runs the dialog and calls the callback with the result
    /// 
    /// The import button is wired to:
    /// 1. Get the selected source via `get_selected_source()` (Requirement 5.1)
    /// 2. Perform import via `do_import()` (Requirement 5.1)
    /// 3. Display results via `show_results()` (Requirements 5.2, 5.3)
    pub fn run<F: Fn(Option<ImportResult>) + 'static>(&self, cb: F) {
        // Store callback
        *self.on_complete.borrow_mut() = Some(Box::new(cb));

        let window = self.window.clone();
        let stack = self.stack.clone();
        let source_list = self.source_list.clone();
        let progress_bar = self.progress_bar.clone();
        let progress_label = self.progress_label.clone();
        let result_label = self.result_label.clone();
        let result_details = self.result_details.clone();
        let import_button = self.import_button.clone();
        let result_cell = self.result.clone();
        let on_complete = self.on_complete.clone();

        // Wire import button click to do_import() (Requirement 5.1)
        import_button.connect_clicked(move |btn| {
            let current_page = stack.visible_child_name();

            if current_page.as_deref() == Some("result") {
                // Done - close dialog
                if let Some(ref cb) = *on_complete.borrow() {
                    cb(result_cell.borrow_mut().take());
                }
                window.close();
                return;
            }

            // Get selected source using get_selected_source() pattern (Requirement 5.1)
            let source_id = source_list
                .selected_row()
                .and_then(|row| {
                    let name = row.widget_name();
                    if name.is_empty() {
                        None
                    } else {
                        Some(name.to_string())
                    }
                });

            if let Some(source_id) = source_id {
                // Show progress page
                stack.set_visible_child_name("progress");
                btn.set_sensitive(false);
                progress_bar.set_fraction(0.0);
                
                let display_name = Self::get_source_display_name(&source_id);
                progress_label.set_text(&format!("Importing from {display_name}..."));

                // Perform import using do_import() pattern (Requirement 5.1)
                let result = match source_id.as_str() {
                    "ssh_config" => {
                        let importer = SshConfigImporter::new();
                        importer.import().unwrap_or_default()
                    }
                    "asbru" => {
                        let importer = AsbruImporter::new();
                        importer.import().unwrap_or_default()
                    }
                    "remmina" => {
                        let importer = RemminaImporter::new();
                        importer.import().unwrap_or_default()
                    }
                    "ansible" => {
                        let importer = AnsibleInventoryImporter::new();
                        importer.import().unwrap_or_default()
                    }
                    _ => ImportResult::default(),
                };

                progress_bar.set_fraction(1.0);

                // Show results using show_results() pattern (Requirements 5.2, 5.3)
                let summary = format!(
                    "Successfully imported {} connection(s) and {} group(s).",
                    result.connections.len(),
                    result.groups.len()
                );
                result_label.set_text(&summary);

                let details = Self::format_import_details(&result);
                result_details.set_text(&details);

                *result_cell.borrow_mut() = Some(result);
                stack.set_visible_child_name("result");
                btn.set_label("Done");
                btn.set_sensitive(true);
            }
        });

        self.window.present();
    }

    /// Runs the dialog and calls the callback with the result and source name
    /// 
    /// Similar to `run()` but also provides the source name to the callback.
    /// The import button is wired to:
    /// 1. Get the selected source via `get_selected_source()` (Requirement 5.1)
    /// 2. Perform import via `do_import()` (Requirement 5.1)
    /// 3. Display results via `show_results_with_source()` (Requirements 5.2, 5.3)
    pub fn run_with_source<F: Fn(Option<ImportResult>, String) + 'static>(&self, cb: F) {
        // Store callback
        *self.on_complete_with_source.borrow_mut() = Some(Box::new(cb));

        let window = self.window.clone();
        let stack = self.stack.clone();
        let source_list = self.source_list.clone();
        let progress_bar = self.progress_bar.clone();
        let progress_label = self.progress_label.clone();
        let result_label = self.result_label.clone();
        let result_details = self.result_details.clone();
        let import_button = self.import_button.clone();
        let result_cell = self.result.clone();
        let source_name_cell = self.source_name.clone();
        let on_complete_with_source = self.on_complete_with_source.clone();

        // Wire import button click to do_import() (Requirement 5.1)
        import_button.connect_clicked(move |btn| {
            let current_page = stack.visible_child_name();

            if current_page.as_deref() == Some("result") {
                // Done - close dialog
                if let Some(ref cb) = *on_complete_with_source.borrow() {
                    let source = source_name_cell.borrow().clone();
                    cb(result_cell.borrow_mut().take(), source);
                }
                window.close();
                return;
            }

            // Get selected source using get_selected_source() pattern (Requirement 5.1)
            let source_id = source_list
                .selected_row()
                .and_then(|row| {
                    let name = row.widget_name();
                    if name.is_empty() {
                        None
                    } else {
                        Some(name.to_string())
                    }
                });

            if let Some(source_id) = source_id {
                // Show progress page
                stack.set_visible_child_name("progress");
                btn.set_sensitive(false);
                progress_bar.set_fraction(0.0);
                
                let display_name = Self::get_source_display_name(&source_id);
                progress_label.set_text(&format!("Importing from {display_name}..."));

                // Handle special case for file-based import
                if source_id == "asbru_file" {
                    Self::handle_asbru_file_import(
                        &window,
                        &stack,
                        &progress_bar,
                        &progress_label,
                        &result_label,
                        &result_details,
                        &result_cell,
                        &source_name_cell,
                        btn,
                    );
                    return;
                }

                // Perform import using do_import() pattern (Requirement 5.1)
                let result = match source_id.as_str() {
                    "ssh_config" => {
                        let importer = SshConfigImporter::new();
                        importer.import().unwrap_or_default()
                    }
                    "asbru" => {
                        let importer = AsbruImporter::new();
                        importer.import().unwrap_or_default()
                    }
                    "remmina" => {
                        let importer = RemminaImporter::new();
                        importer.import().unwrap_or_default()
                    }
                    "ansible" => {
                        let importer = AnsibleInventoryImporter::new();
                        importer.import().unwrap_or_default()
                    }
                    _ => ImportResult::default(),
                };

                // Store source name
                *source_name_cell.borrow_mut() = display_name.to_string();

                progress_bar.set_fraction(1.0);

                // Show results using show_results_with_source() pattern (Requirements 5.2, 5.3)
                let summary = format!(
                    "Successfully imported {} connection(s) and {} group(s).\nConnections will be added to '{}' group.",
                    result.connections.len(),
                    result.groups.len(),
                    format!("{display_name} Import")
                );
                result_label.set_text(&summary);

                let details = Self::format_import_details(&result);
                result_details.set_text(&details);

                *result_cell.borrow_mut() = Some(result);
                stack.set_visible_child_name("result");
                btn.set_label("Done");
                btn.set_sensitive(true);
            }
        });

        self.window.present();
    }

    /// Handles the special case of importing from an Asbru-CM YAML file
    #[allow(clippy::too_many_arguments)]
    fn handle_asbru_file_import(
        window: &Window,
        stack: &Stack,
        progress_bar: &ProgressBar,
        progress_label: &Label,
        result_label: &Label,
        result_details: &Label,
        result_cell: &Rc<RefCell<Option<ImportResult>>>,
        source_name_cell: &Rc<RefCell<String>>,
        btn: &Button,
    ) {
        // Use file dialog
        let file_dialog = gtk4::FileDialog::builder()
            .title("Select Asbru-CM YAML File")
            .modal(true)
            .build();

        // Set filter for YAML files
        let filter = gtk4::FileFilter::new();
        filter.add_pattern("*.yml");
        filter.add_pattern("*.yaml");
        filter.set_name(Some("YAML files"));
        let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);
        file_dialog.set_filters(Some(&filters));

        let stack_clone = stack.clone();
        let progress_bar_clone = progress_bar.clone();
        let progress_label_clone = progress_label.clone();
        let result_label_clone = result_label.clone();
        let result_details_clone = result_details.clone();
        let result_cell_clone = result_cell.clone();
        let source_name_cell_clone = source_name_cell.clone();
        let btn_clone = btn.clone();

        file_dialog.open(
            Some(window),
            gtk4::gio::Cancellable::NONE,
            move |file_result| {
                if let Ok(file) = file_result {
                    if let Some(path) = file.path() {
                        stack_clone.set_visible_child_name("progress");
                        btn_clone.set_sensitive(false);
                        progress_bar_clone.set_fraction(0.5);
                        progress_label_clone
                            .set_text(&format!("Importing from {}...", path.display()));

                        let importer = AsbruImporter::new();
                        let result = importer.import_from_path(&path).unwrap_or_default();

                        // Extract filename for display
                        let filename = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Asbru-CM File".to_string());

                        *source_name_cell_clone.borrow_mut() = filename.clone();

                        progress_bar_clone.set_fraction(1.0);

                        // Show results using format_import_details() (Requirements 5.2, 5.3)
                        let summary = format!(
                            "Successfully imported {} connection(s) and {} group(s).\nConnections will be added to '{}' group.",
                            result.connections.len(),
                            result.groups.len(),
                            format!("{filename} Import")
                        );
                        result_label_clone.set_text(&summary);

                        let details = Self::format_import_details(&result);
                        result_details_clone.set_text(&details);

                        *result_cell_clone.borrow_mut() = Some(result);
                        stack_clone.set_visible_child_name("result");
                        btn_clone.set_label("Done");
                        btn_clone.set_sensitive(true);
                    }
                }
            },
        );
    }

    /// Returns a reference to the underlying window
    #[must_use]
    pub fn window(&self) -> &Window {
        &self.window
    }
}
