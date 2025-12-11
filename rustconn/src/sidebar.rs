//! Connection tree sidebar
//!
//! This module provides the sidebar widget for displaying and managing
//! the connection hierarchy with drag-and-drop support.

use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::{
    gdk, gio, glib, Box as GtkBox, Button, DragSource, DropTarget, EventControllerKey,
    GestureClick, Label, ListItem, ListView, MultiSelection, Orientation, PolicyType,
    ScrolledWindow, SearchEntry, SignalListItemFactory, SingleSelection, TreeExpander,
    TreeListModel, TreeListRow, Widget,
};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

/// Wrapper to switch between selection models
/// Supports switching between SingleSelection and MultiSelection modes
pub enum SelectionModelWrapper {
    /// Single selection mode (default)
    Single(SingleSelection),
    /// Multi-selection mode for group operations
    Multi(MultiSelection),
}

impl SelectionModelWrapper {
    /// Creates a new single selection wrapper
    #[must_use]
    pub fn new_single(model: TreeListModel) -> Self {
        Self::Single(SingleSelection::new(Some(model)))
    }

    /// Creates a new multi-selection wrapper
    #[must_use]
    pub fn new_multi(model: TreeListModel) -> Self {
        Self::Multi(MultiSelection::new(Some(model)))
    }

    /// Returns the underlying selection model as a SelectionModel
    #[must_use]
    pub fn as_selection_model(&self) -> &impl IsA<gtk4::SelectionModel> {
        match self {
            Self::Single(s) => s,
            Self::Multi(_) => panic!("Cannot return MultiSelection as SelectionModel reference"),
        }
    }

    /// Returns true if in multi-selection mode
    #[must_use]
    pub fn is_multi(&self) -> bool {
        matches!(self, Self::Multi(_))
    }

    /// Gets all selected item positions
    #[must_use]
    pub fn get_selected_positions(&self) -> Vec<u32> {
        match self {
            Self::Single(s) => {
                let selected = s.selected();
                if selected != gtk4::INVALID_LIST_POSITION {
                    vec![selected]
                } else {
                    vec![]
                }
            }
            Self::Multi(m) => {
                let selection = m.selection();
                let mut positions = Vec::new();
                // Iterate through the bitset using nth() which returns the nth set bit
                let size = selection.size();
                for i in 0..size {
                    #[allow(clippy::cast_possible_truncation)]
                    let pos = selection.nth(i as u32);
                    if pos != u32::MAX {
                        positions.push(pos);
                    }
                }
                positions
            }
        }
    }

    /// Selects all items (only works in multi-selection mode)
    pub fn select_all(&self) {
        if let Self::Multi(m) = self {
            m.select_all();
        }
    }

    /// Clears all selections
    pub fn clear_selection(&self) {
        match self {
            Self::Single(s) => {
                s.set_selected(gtk4::INVALID_LIST_POSITION);
            }
            Self::Multi(m) => {
                m.unselect_all();
            }
        }
    }

    /// Gets the underlying model
    #[must_use]
    pub fn model(&self) -> Option<gio::ListModel> {
        match self {
            Self::Single(s) => s.model(),
            Self::Multi(m) => m.model(),
        }
    }
}

/// Sidebar widget for connection tree display
pub struct ConnectionSidebar {
    container: GtkBox,
    search_entry: SearchEntry,
    list_view: ListView,
    /// Store for connection data - will be populated from ConnectionManager
    store: gio::ListStore,
    /// Tree list model for hierarchical display
    tree_model: TreeListModel,
    /// Selection model - switches between Single and Multi
    selection_model: Rc<RefCell<SelectionModelWrapper>>,
    /// Bulk actions toolbar (visible in group ops mode)
    bulk_actions_bar: GtkBox,
    /// Current mode
    group_ops_mode: Rc<RefCell<bool>>,
}

impl ConnectionSidebar {
    /// Creates a new connection sidebar
    #[must_use]
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_width_request(250);
        container.add_css_class("sidebar");

        // Search entry at the top
        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search connections..."));
        search_entry.set_margin_start(8);
        search_entry.set_margin_end(8);
        search_entry.set_margin_top(8);
        search_entry.set_margin_bottom(8);
        // Accessibility: set label for screen readers
        search_entry.update_property(&[
            gtk4::accessible::Property::Label("Search connections"),
        ]);
        container.append(&search_entry);

        // Create bulk actions toolbar (hidden by default)
        let bulk_actions_bar = Self::create_bulk_actions_bar();
        bulk_actions_bar.set_visible(false);
        container.append(&bulk_actions_bar);

        // Create the list store for connection items
        let store = gio::ListStore::new::<ConnectionItem>();

        // Create tree list model for hierarchical display
        let tree_model = TreeListModel::new(store.clone(), false, true, |item| {
            item.downcast_ref::<ConnectionItem>()
                .and_then(|conn_item| conn_item.children())
        });

        // Create selection model (starts in single selection mode)
        let selection_wrapper = SelectionModelWrapper::new_single(tree_model.clone());
        let selection_model = Rc::new(RefCell::new(selection_wrapper));

        // Create the factory for list items
        let factory = SignalListItemFactory::new();
        let group_ops_mode = Rc::new(RefCell::new(false));
        let group_ops_mode_clone = group_ops_mode.clone();
        factory.connect_setup(move |factory, obj| {
            if let Some(list_item) = obj.downcast_ref::<ListItem>() {
                Self::setup_list_item(factory, list_item, *group_ops_mode_clone.borrow());
            }
        });
        factory.connect_bind(|factory, obj| {
            if let Some(list_item) = obj.downcast_ref::<ListItem>() {
                Self::bind_list_item(factory, list_item);
            }
        });

        // Create the list view with single selection initially
        let list_view = {
            let sel = selection_model.borrow();
            match &*sel {
                SelectionModelWrapper::Single(s) => ListView::new(Some(s.clone()), Some(factory)),
                SelectionModelWrapper::Multi(m) => ListView::new(Some(m.clone()), Some(factory)),
            }
        };
        list_view.add_css_class("navigation-sidebar");
        
        // Set accessibility properties
        list_view.update_property(&[
            gtk4::accessible::Property::Label("Connection list"),
        ]);
        list_view.set_focusable(true);
        list_view.set_can_focus(true);

        // Set up keyboard navigation
        let group_ops_mode_clone = group_ops_mode.clone();
        let selection_model_clone = selection_model.clone();
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(move |_controller, key, _code, state| {
            let is_group_ops = *group_ops_mode_clone.borrow();
            
            // Handle keyboard navigation
            match key {
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    // Activate selected item - handled by ListView's activate signal
                    glib::Propagation::Stop
                }
                gdk::Key::Delete => {
                    // Delete selected item - will be handled by window action
                    glib::Propagation::Proceed
                }
                gdk::Key::a | gdk::Key::A if state.contains(gdk::ModifierType::CONTROL_MASK) && is_group_ops => {
                    // Ctrl+A: Select all in group operations mode
                    selection_model_clone.borrow().select_all();
                    glib::Propagation::Stop
                }
                gdk::Key::Escape if is_group_ops => {
                    // Escape: Clear selection in group operations mode
                    selection_model_clone.borrow().clear_selection();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        list_view.add_controller(key_controller);

        // Wrap in scrolled window
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .child(&list_view)
            .build();

        container.append(&scrolled);

        // Add buttons at the bottom
        let button_box = Self::create_button_box();
        container.append(&button_box);

        Self {
            container,
            search_entry,
            list_view,
            store,
            tree_model,
            selection_model,
            bulk_actions_bar,
            group_ops_mode,
        }
    }

    /// Creates the bulk actions toolbar for group operations mode
    fn create_bulk_actions_bar() -> GtkBox {
        let bar = GtkBox::new(Orientation::Horizontal, 4);
        bar.set_margin_start(8);
        bar.set_margin_end(8);
        bar.set_margin_top(4);
        bar.set_margin_bottom(4);
        bar.add_css_class("bulk-actions-bar");

        // Delete Selected button
        let delete_button = Button::with_label("Delete Selected");
        delete_button.set_tooltip_text(Some("Delete all selected items"));
        delete_button.set_action_name(Some("win.delete-selected"));
        delete_button.add_css_class("destructive-action");
        delete_button.update_property(&[
            gtk4::accessible::Property::Label("Delete selected connections"),
        ]);
        bar.append(&delete_button);

        // Move to Group dropdown button
        let move_button = Button::with_label("Move to Group...");
        move_button.set_tooltip_text(Some("Move selected items to a group"));
        move_button.set_action_name(Some("win.move-selected-to-group"));
        move_button.update_property(&[
            gtk4::accessible::Property::Label("Move selected connections to group"),
        ]);
        bar.append(&move_button);

        // Select All button
        let select_all_button = Button::with_label("Select All");
        select_all_button.set_tooltip_text(Some("Select all items (Ctrl+A)"));
        select_all_button.set_action_name(Some("win.select-all"));
        select_all_button.update_property(&[
            gtk4::accessible::Property::Label("Select all connections"),
        ]);
        bar.append(&select_all_button);

        // Clear Selection button
        let clear_button = Button::with_label("Clear");
        clear_button.set_tooltip_text(Some("Clear selection (Escape)"));
        clear_button.set_action_name(Some("win.clear-selection"));
        clear_button.update_property(&[
            gtk4::accessible::Property::Label("Clear selection"),
        ]);
        bar.append(&clear_button);

        bar
    }

    /// Creates the button box at the bottom of the sidebar
    fn create_button_box() -> GtkBox {
        let button_box = GtkBox::new(Orientation::Horizontal, 4);
        button_box.set_margin_start(8);
        button_box.set_margin_end(8);
        button_box.set_margin_top(8);
        button_box.set_margin_bottom(8);
        button_box.set_halign(gtk4::Align::Center);

        // Add connection button
        let add_button = Button::from_icon_name("list-add-symbolic");
        add_button.set_tooltip_text(Some("Add Connection (Ctrl+N)"));
        add_button.set_action_name(Some("win.new-connection"));
        add_button.update_property(&[
            gtk4::accessible::Property::Label("Add new connection"),
        ]);
        button_box.append(&add_button);

        // Delete button
        let delete_button = Button::from_icon_name("list-remove-symbolic");
        delete_button.set_tooltip_text(Some("Delete Selected (Delete)"));
        delete_button.set_action_name(Some("win.delete-connection"));
        delete_button.update_property(&[
            gtk4::accessible::Property::Label("Delete selected connection or group"),
        ]);
        button_box.append(&delete_button);

        // Add group button
        let add_group_button = Button::from_icon_name("folder-new-symbolic");
        add_group_button.set_tooltip_text(Some("Add Group (Ctrl+Shift+N)"));
        add_group_button.set_action_name(Some("win.new-group"));
        add_group_button.update_property(&[
            gtk4::accessible::Property::Label("Add new group"),
        ]);
        button_box.append(&add_group_button);

        // Quick connect button
        let quick_connect_button = Button::from_icon_name("network-transmit-symbolic");
        quick_connect_button.set_tooltip_text(Some("Quick Connect (without saving)"));
        quick_connect_button.set_action_name(Some("win.quick-connect"));
        quick_connect_button.update_property(&[
            gtk4::accessible::Property::Label("Quick connect without saving"),
        ]);
        button_box.append(&quick_connect_button);

        // Group operations button
        let group_ops_button = Button::from_icon_name("view-list-symbolic");
        group_ops_button.set_tooltip_text(Some("Group Operations Mode"));
        group_ops_button.set_action_name(Some("win.group-operations"));
        group_ops_button.update_property(&[
            gtk4::accessible::Property::Label("Enable group operations mode for multi-select"),
        ]);
        button_box.append(&group_ops_button);

        // Sort button
        let sort_button = Button::from_icon_name("view-sort-ascending-symbolic");
        sort_button.set_tooltip_text(Some("Sort Alphabetically"));
        sort_button.set_action_name(Some("win.sort-connections"));
        sort_button.update_property(&[
            gtk4::accessible::Property::Label("Sort connections alphabetically"),
        ]);
        button_box.append(&sort_button);

        // Import button
        let import_button = Button::from_icon_name("document-open-symbolic");
        import_button.set_tooltip_text(Some("Import Connections (Ctrl+I)"));
        import_button.set_action_name(Some("win.import"));
        import_button.update_property(&[
            gtk4::accessible::Property::Label("Import connections from external sources"),
        ]);
        button_box.append(&import_button);

        // Export button
        let export_button = Button::from_icon_name("document-save-symbolic");
        export_button.set_tooltip_text(Some("Export Configuration"));
        export_button.set_action_name(Some("win.export"));
        export_button.update_property(&[
            gtk4::accessible::Property::Label("Export configuration to file"),
        ]);
        button_box.append(&export_button);

        button_box
    }

    /// Sets up a list item widget
    fn setup_list_item(_factory: &SignalListItemFactory, list_item: &ListItem, _group_ops_mode: bool) {
        let expander = TreeExpander::new();

        let content_box = GtkBox::new(Orientation::Horizontal, 8);
        content_box.set_margin_start(4);
        content_box.set_margin_end(4);
        content_box.set_margin_top(4);
        content_box.set_margin_bottom(4);

        let icon = gtk4::Image::from_icon_name("network-server-symbolic");
        content_box.append(&icon);

        let label = Label::new(None);
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);
        content_box.append(&label);

        expander.set_child(Some(&content_box));
        list_item.set_child(Some(&expander));

        // Set up drag source for reorganization
        let drag_source = DragSource::new();
        drag_source.set_actions(gdk::DragAction::MOVE);
        drag_source.connect_prepare(|_source, _x, _y| {
            // Prepare drag data
            Some(gdk::ContentProvider::for_value(&"connection".to_value()))
        });
        expander.add_controller(drag_source);

        // Set up drop target
        let drop_target = DropTarget::new(glib::Type::STRING, gdk::DragAction::MOVE);
        drop_target.connect_drop(|_target, _value, _x, _y| {
            // Handle drop - reorganize connections
            true
        });
        expander.add_controller(drop_target);

        // Set up right-click context menu
        // Note: is_group will be determined at bind time via list_item data
        let gesture = GestureClick::new();
        gesture.set_button(gdk::BUTTON_SECONDARY);
        let list_item_weak = list_item.downgrade();
        gesture.connect_pressed(move |gesture, _n_press, x, y| {
            if let Some(widget) = gesture.widget() {
                // First, select this item so context menu actions work on it
                if let Some(list_item) = list_item_weak.upgrade() {
                    // Get the position of this item and select it
                    let position = list_item.position();
                    if let Some(list_view) = widget.ancestor(ListView::static_type()) {
                        if let Some(list_view) = list_view.downcast_ref::<ListView>() {
                            if let Some(model) = list_view.model() {
                                if let Some(selection) = model.downcast_ref::<SingleSelection>() {
                                    selection.set_selected(position);
                                } else if let Some(selection) = model.downcast_ref::<MultiSelection>() {
                                    // In multi-selection mode, select only this item for context menu
                                    selection.unselect_all();
                                    selection.select_item(position, false);
                                }
                            }
                        }
                    }
                }
                
                // Check if this is a group by looking at the icon
                let is_group = widget
                    .first_child()
                    .and_then(|c| c.first_child())
                    .and_then(|c| c.downcast::<gtk4::Image>().ok())
                    .map(|img| img.icon_name().map(|n| n.as_str() == "folder-symbolic").unwrap_or(false))
                    .unwrap_or(false);
                Self::show_context_menu_for_item(&widget, x, y, is_group);
            }
        });
        expander.add_controller(gesture);
    }

    /// Binds data to a list item
    fn bind_list_item(_factory: &SignalListItemFactory, list_item: &ListItem) {
        let Some(expander) = list_item.child().and_downcast::<TreeExpander>() else {
            return;
        };

        let Some(row) = list_item.item().and_downcast::<TreeListRow>() else {
            return;
        };

        expander.set_list_row(Some(&row));

        let Some(item) = row.item().and_downcast::<ConnectionItem>() else {
            return;
        };

        let Some(content_box) = expander.child().and_downcast::<GtkBox>() else {
            return;
        };

        // Update icon based on item type
        if let Some(icon) = content_box.first_child().and_downcast::<gtk4::Image>() {
            if item.is_group() {
                icon.set_icon_name(Some("folder-symbolic"));
            } else {
                let icon_name = match item.protocol().as_str() {
                    "ssh" => "utilities-terminal-symbolic",
                    "rdp" => "computer-symbolic",
                    "vnc" => "video-display-symbolic",
                    _ => "network-server-symbolic",
                };
                icon.set_icon_name(Some(icon_name));
            }
        }

        // Update label
        if let Some(label) = content_box.last_child().and_downcast::<Label>() {
            label.set_text(&item.name());
        }
    }

    /// Shows the context menu for a connection item
    #[allow(dead_code)]
    fn show_context_menu(widget: &impl IsA<Widget>, x: f64, y: f64) {
        Self::show_context_menu_for_item(widget, x, y, false);
    }

    /// Shows the context menu for a connection item with group awareness
    fn show_context_menu_for_item(widget: &impl IsA<Widget>, x: f64, y: f64, is_group: bool) {
        let menu = gio::Menu::new();
        
        if !is_group {
            menu.append(Some("Connect"), Some("win.connect"));
        }
        menu.append(Some("Edit"), Some("win.edit-connection"));
        if !is_group {
            menu.append(Some("Duplicate"), Some("win.duplicate-connection"));
            menu.append(Some("Move to Group..."), Some("win.move-to-group"));
        }
        menu.append(Some("Delete"), Some("win.delete-connection"));

        let popover = gtk4::PopoverMenu::from_model(Some(&menu));
        popover.set_parent(widget);
        popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover.set_autohide(true);
        
        // Connect to closed signal to unparent the popover
        popover.connect_closed(|p| {
            p.unparent();
        });
        
        popover.popup();
    }

    /// Returns the main widget for this sidebar
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Returns the search entry widget
    #[must_use]
    pub fn search_entry(&self) -> &SearchEntry {
        &self.search_entry
    }

    /// Returns the list view widget
    #[must_use]
    pub fn list_view(&self) -> &ListView {
        &self.list_view
    }

    /// Returns the underlying store
    #[must_use]
    pub fn store(&self) -> &gio::ListStore {
        &self.store
    }

    /// Returns the tree list model
    #[must_use]
    pub fn tree_model(&self) -> &TreeListModel {
        &self.tree_model
    }

    /// Returns the bulk actions bar
    #[must_use]
    pub fn bulk_actions_bar(&self) -> &GtkBox {
        &self.bulk_actions_bar
    }

    /// Returns whether group operations mode is active
    #[must_use]
    pub fn is_group_operations_mode(&self) -> bool {
        *self.group_ops_mode.borrow()
    }

    /// Toggles group operations mode
    /// Switches between SingleSelection and MultiSelection models
    pub fn set_group_operations_mode(&self, enabled: bool) {
        // Update mode flag
        *self.group_ops_mode.borrow_mut() = enabled;

        // Show/hide bulk actions toolbar
        self.bulk_actions_bar.set_visible(enabled);

        // Create new selection model
        let new_wrapper = if enabled {
            SelectionModelWrapper::new_multi(self.tree_model.clone())
        } else {
            SelectionModelWrapper::new_single(self.tree_model.clone())
        };

        // Update the list view with new selection model
        match &new_wrapper {
            SelectionModelWrapper::Single(s) => {
                self.list_view.set_model(Some(s));
            }
            SelectionModelWrapper::Multi(m) => {
                self.list_view.set_model(Some(m));
            }
        }

        // Store the new wrapper
        *self.selection_model.borrow_mut() = new_wrapper;

        // Update CSS class for visual feedback
        if enabled {
            self.list_view.add_css_class("group-operations-mode");
        } else {
            self.list_view.remove_css_class("group-operations-mode");
        }
    }

    /// Gets all selected connection/group IDs
    #[must_use]
    pub fn get_selected_ids(&self) -> Vec<Uuid> {
        let selection = self.selection_model.borrow();
        let positions = selection.get_selected_positions();
        
        let mut ids = Vec::new();
        for pos in positions {
            if let Some(model) = selection.model() {
                if let Some(item) = model.item(pos) {
                    // Handle TreeListRow wrapping
                    let conn_item = if let Some(row) = item.downcast_ref::<TreeListRow>() {
                        row.item().and_downcast::<ConnectionItem>()
                    } else {
                        item.downcast::<ConnectionItem>().ok()
                    };
                    
                    if let Some(conn_item) = conn_item {
                        if let Ok(uuid) = Uuid::parse_str(&conn_item.id()) {
                            ids.push(uuid);
                        }
                    }
                }
            }
        }
        ids
    }

    /// Gets the first selected ConnectionItem (works in both single and multi-selection modes)
    #[must_use]
    pub fn get_selected_item(&self) -> Option<ConnectionItem> {
        let selection = self.selection_model.borrow();
        let positions = selection.get_selected_positions();
        
        if let Some(&pos) = positions.first() {
            if let Some(model) = selection.model() {
                if let Some(item) = model.item(pos) {
                    // Handle TreeListRow wrapping
                    return if let Some(row) = item.downcast_ref::<TreeListRow>() {
                        row.item().and_downcast::<ConnectionItem>()
                    } else {
                        item.downcast::<ConnectionItem>().ok()
                    };
                }
            }
        }
        None
    }

    /// Selects all visible items (only works in group operations mode)
    pub fn select_all(&self) {
        self.selection_model.borrow().select_all();
    }

    /// Clears all selections
    pub fn clear_selection(&self) {
        self.selection_model.borrow().clear_selection();
    }

    /// Returns the selection model wrapper
    pub fn selection_model(&self) -> Rc<RefCell<SelectionModelWrapper>> {
        self.selection_model.clone()
    }
}

impl Default for ConnectionSidebar {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Connection Item GObject wrapper
// ============================================================================

mod imp {
    use super::*;
    use glib::subclass::prelude::*;
    use glib::Properties;
    use std::cell::RefCell;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ConnectionItem)]
    pub struct ConnectionItem {
        #[property(get, set)]
        id: RefCell<String>,
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        protocol: RefCell<String>,
        #[property(get, set)]
        is_group: RefCell<bool>,
        #[property(get, set)]
        host: RefCell<String>,
        pub(super) children: RefCell<Option<gio::ListStore>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ConnectionItem {
        const NAME: &'static str = "RustConnConnectionItem";
        type Type = super::ConnectionItem;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ConnectionItem {}
}

glib::wrapper! {
    /// A GObject wrapper for connection/group items in the tree view
    pub struct ConnectionItem(ObjectSubclass<imp::ConnectionItem>);
}

impl ConnectionItem {
    /// Creates a new connection item
    #[must_use]
    pub fn new_connection(id: &str, name: &str, protocol: &str, host: &str) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("name", name)
            .property("protocol", protocol)
            .property("is-group", false)
            .property("host", host)
            .build()
    }

    /// Creates a new group item
    #[must_use]
    pub fn new_group(id: &str, name: &str) -> Self {
        let item: Self = glib::Object::builder()
            .property("id", id)
            .property("name", name)
            .property("protocol", "")
            .property("is-group", true)
            .property("host", "")
            .build();

        // Initialize children store for groups
        *item.imp().children.borrow_mut() = Some(gio::ListStore::new::<ConnectionItem>());

        item
    }

    /// Returns the children list store for groups
    pub fn children(&self) -> Option<gio::ListModel> {
        self.imp()
            .children
            .borrow()
            .as_ref()
            .map(|store| store.clone().upcast())
    }

    /// Adds a child item to this group
    pub fn add_child(&self, child: &ConnectionItem) {
        if let Some(ref store) = *self.imp().children.borrow() {
            store.append(child);
        }
    }
}

impl Default for ConnectionItem {
    fn default() -> Self {
        Self::new_connection("", "Unnamed", "ssh", "")
    }
}
