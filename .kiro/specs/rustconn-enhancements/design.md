# Design Document: RustConn Enhancements

## Overview

This document describes the design for enhancing RustConn with embedded VNC/RDP tabs, multi-selection group operations, and wiring up unused functionality. The enhancements focus on improving user experience and completing the implementation of existing code.

### Key Design Goals

- **Unified Interface**: All connection types displayed in tabs within the main window
- **Bulk Operations**: Multi-selection support for efficient connection management
- **Complete Integration**: Wire up all existing but unused dialog and state methods
- **GTK4 Best Practices**: Proper dialog parenting and modern widget usage

### Technology Constraints

| Constraint | Impact |
|-----------|--------|
| Wayland | No GtkSocket/GtkPlug - must use alternative embedding approaches |
| GTK4 | MultiSelection model for multi-select, no deprecated Dialog |
| FreeRDP | Supports `/parent-window:` on X11, limited Wayland support |
| VNC | Most viewers don't support embedding, may need fallback |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        MainWindow                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────┐  ┌─────────────────────────────────┐  │
│  │   ConnectionSidebar  │  │      TerminalNotebook           │  │
│  │  ┌────────────────┐  │  │  ┌─────────────────────────┐    │  │
│  │  │ MultiSelection │  │  │  │ SSH Terminal (VTE)      │    │  │
│  │  │    ListView    │  │  │  ├─────────────────────────┤    │  │
│  │  └────────────────┘  │  │  │ RDP Embedded (Socket)   │    │  │
│  │  ┌────────────────┐  │  │  ├─────────────────────────┤    │  │
│  │  │ BulkActions    │  │  │  │ VNC Embedded (Socket)   │    │  │
│  │  │   Toolbar      │  │  │  └─────────────────────────┘    │  │
│  │  └────────────────┘  │  └─────────────────────────────────┘  │
│  └──────────────────────┘                                       │
├─────────────────────────────────────────────────────────────────┤
│                        Dialogs                                   │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │
│  │ Connection │ │   Import   │ │  Settings  │ │  Snippet   │   │
│  │   Dialog   │ │   Dialog   │ │   Dialog   │ │   Dialog   │   │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Multi-Selection Sidebar

```rust
/// Enhanced sidebar with multi-selection support
pub struct ConnectionSidebar {
    container: GtkBox,
    search_entry: SearchEntry,
    list_view: ListView,
    store: gio::ListStore,
    /// Selection model - switches between Single and Multi
    selection_model: Rc<RefCell<SelectionModelWrapper>>,
    /// Bulk actions toolbar (visible in group ops mode)
    bulk_actions_bar: GtkBox,
    /// Current mode
    group_ops_mode: Rc<RefCell<bool>>,
}

/// Wrapper to switch between selection models
enum SelectionModelWrapper {
    Single(SingleSelection),
    Multi(MultiSelection),
}

impl ConnectionSidebar {
    /// Toggles group operations mode
    pub fn set_group_operations_mode(&self, enabled: bool);
    
    /// Gets all selected connection IDs
    pub fn get_selected_ids(&self) -> Vec<Uuid>;
    
    /// Selects all visible items
    pub fn select_all(&self);
    
    /// Clears selection
    pub fn clear_selection(&self);
}
```

### Embedded Session Tab

```rust
/// Tab content for embedded RDP/VNC sessions
pub struct EmbeddedSessionTab {
    container: GtkBox,
    /// Socket widget for embedding (X11 only)
    socket: Option<gtk4::Socket>,
    /// Fallback label for Wayland
    fallback_label: Option<Label>,
    /// Session controls
    controls: SessionControls,
    /// Process handle
    process: Option<Child>,
}

pub struct SessionControls {
    fullscreen_button: Button,
    disconnect_button: Button,
    status_label: Label,
}

impl EmbeddedSessionTab {
    /// Creates embedded tab, returns fallback info if embedding not supported
    pub fn new(protocol: &str, connection: &Connection) -> (Self, bool);
    
    /// Starts the embedded session
    pub fn start_session(&mut self, host: &str, port: u16, credentials: Option<&Credentials>) -> Result<(), SessionError>;
    
    /// Toggles fullscreen mode
    pub fn toggle_fullscreen(&self);
    
    /// Disconnects the session
    pub fn disconnect(&mut self);
}
```

### Bulk Actions

```rust
/// Bulk action handler
pub struct BulkActionHandler {
    state: SharedAppState,
    sidebar: SharedSidebar,
}

impl BulkActionHandler {
    /// Deletes all selected connections
    pub fn delete_selected(&self, window: &ApplicationWindow) -> Result<usize, Vec<String>>;
    
    /// Moves selected connections to a group
    pub fn move_to_group(&self, group_id: Option<Uuid>) -> Result<usize, String>;
}
```

### Dialog Integration

```rust
// ConnectionDialog - wire up existing methods
impl ConnectionDialog {
    /// Called when Save button is clicked
    pub fn on_save(&self) -> Option<Connection> {
        if let Err(e) = self.validate() {
            self.show_error(&e);
            return None;
        }
        self.build_connection()
    }
}

// ImportDialog - wire up existing methods  
impl ImportDialog {
    /// Called when Import button is clicked
    pub fn on_import(&self) {
        if let Some(source) = self.get_selected_source() {
            let result = self.do_import(&source);
            self.show_results(&result);
        }
    }
}

// SettingsDialog - wire up existing methods
impl SettingsDialog {
    /// Called when Save button is clicked
    pub fn on_save(&self) -> Option<AppSettings> {
        Some(self.build_settings())
    }
}

// SnippetDialog - wire up existing methods
impl SnippetDialog {
    /// Called when Save button is clicked
    pub fn on_save(&self) -> Option<Snippet> {
        if let Err(e) = self.validate() {
            self.show_error(&e);
            return None;
        }
        self.build_snippet()
    }
}
```

## Data Models

### Selection State

```rust
/// Tracks multi-selection state
pub struct SelectionState {
    /// Selected connection IDs
    pub selected_connections: HashSet<Uuid>,
    /// Selected group IDs
    pub selected_groups: HashSet<Uuid>,
    /// Whether group operations mode is active
    pub group_ops_active: bool,
}
```

### Embedded Session State

```rust
/// State for embedded RDP/VNC session
pub struct EmbeddedSession {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub protocol: String,
    pub process_id: Option<u32>,
    pub window_id: Option<u64>,
    pub is_fullscreen: bool,
    pub started_at: DateTime<Utc>,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Multi-Selection Consistency

*For any* set of selection operations (select, deselect, select-all, clear), the set of selected IDs returned by `get_selected_ids()` must exactly match the items visually indicated as selected in the UI.

**Validates: Requirements 2.1, 2.2, 2.3, 2.6**

### Property 2: Bulk Delete Completeness

*For any* set of selected connections, after bulk delete completes successfully, none of the deleted connection IDs should exist in the connection manager, and the count of deleted items should equal the original selection count minus any failures.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4**

### Property 3: Dialog Validation Round-Trip

*For any* valid connection configuration entered in the dialog, calling `validate()` should succeed, and `build_connection()` should produce a Connection object that, when loaded back into the dialog via `set_connection()`, produces identical field values.

**Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**

### Property 4: Settings Persistence Round-Trip

*For any* settings configuration, saving via `build_settings()` and then loading via `set_settings()` should produce identical settings values.

**Validates: Requirements 6.1, 6.2**

### Property 5: Snippet Variable Extraction

*For any* snippet command containing `${variable}` placeholders, the extracted variable list should contain exactly the unique variable names present in the command, with no duplicates.

**Validates: Requirements 7.2, 7.3**

### Property 6: Group Hierarchy Acyclicity

*For any* sequence of group creation and move operations, the resulting group hierarchy must remain acyclic - no group can be its own ancestor.

**Validates: Requirements 9.1, 9.2**

## Error Handling

### Embedding Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Embedding not supported on Wayland for {protocol}")]
    WaylandNotSupported { protocol: String },
    
    #[error("Failed to create socket: {0}")]
    SocketCreationFailed(String),
    
    #[error("Client process failed to start: {0}")]
    ProcessStartFailed(String),
    
    #[error("Client exited unexpectedly with code {code}")]
    ClientExited { code: i32 },
}
```

### Bulk Operation Errors

```rust
#[derive(Debug)]
pub struct BulkOperationResult {
    pub successful: usize,
    pub failed: Vec<(Uuid, String)>,
}
```

## Testing Strategy

### Property-Based Testing Framework

The project uses **proptest** crate for property-based testing.

### Test Configuration

- Minimum 100 iterations per property test
- Each property test annotated with correctness property reference
- Format: `// **Feature: rustconn-enhancements, Property {N}: {property_name}**`

### Unit Tests

- Selection model state transitions
- Dialog validation edge cases
- Bulk operation partial failure handling

### Integration Tests

- Full multi-select workflow
- Dialog save/load round-trips
- Embedded session lifecycle (where supported)

