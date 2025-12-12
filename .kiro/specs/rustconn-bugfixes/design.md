# Design Document

## Overview

This document describes the design for fixing bugs and adding enhancements to RustConn. The changes address:
1. Connection dialog Save button not working
2. Asbru-CM import issues
3. Context menu operations not functioning
4. Group-scoped sorting
5. Drag-and-drop reordering
6. Sort Recent feature
7. Client detection in Settings
8. Protocol selection in Quick Connect
9. Wiring up unused code

## Architecture

The fixes primarily affect the GUI layer (`rustconn` crate) with some enhancements to the core library (`rustconn-core`).

```
┌─────────────────────────────────────────────────────────────┐
│                     rustconn (GUI)                          │
├─────────────────────────────────────────────────────────────┤
│  window.rs      - Action handlers, context menu wiring      │
│  sidebar.rs     - Drag-and-drop, sorting UI                 │
│  dialogs/       - Connection, Settings, Quick Connect       │
│  state.rs       - Connection operations                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   rustconn-core (Library)                   │
├─────────────────────────────────────────────────────────────┤
│  connection/manager.rs - Sorting, reordering logic          │
│  import/asbru.rs       - Improved hostname extraction       │
│  protocol/             - Client detection utilities         │
│  models/connection.rs  - last_connected timestamp           │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. Connection Dialog Fix

The Save button click handler needs to be properly connected. The issue is in `run()` method where the button is found via `header.last_child()` which may not reliably find the Save button.

**Solution**: Store reference to save button during construction and connect handler directly.

```rust
pub struct ConnectionDialog {
    // ... existing fields
    save_button: Button,  // Add explicit reference
}
```

### 2. Asbru Import Enhancement

Add fallback hostname extraction from additional fields:

```rust
impl AsbruImporter {
    fn extract_hostname(&self, entry: &AsbruEntry) -> Option<String> {
        // Try primary fields
        entry.ip.as_ref()
            .or(entry.host.as_ref())
            // Try extracting from name if it looks like hostname
            .or_else(|| self.extract_hostname_from_name(&entry.name))
            // Try extracting from title
            .or_else(|| self.extract_hostname_from_name(&entry.title))
            .filter(|h| !h.is_empty() && h != "tmp")
            .cloned()
    }
    
    fn extract_hostname_from_name(&self, name: &Option<String>) -> Option<&String> {
        name.as_ref().filter(|n| {
            // Check if name looks like a hostname (contains dots or is IP-like)
            n.contains('.') || n.parse::<std::net::IpAddr>().is_ok()
        })
    }
}
```

### 3. Context Menu Actions

Wire up the existing action handlers properly. The actions are defined but `connect_selected` is empty.

```rust
fn connect_selected(state: &SharedAppState, sidebar: &SharedSidebar, notebook: &SharedNotebook) {
    if let Some(item) = sidebar.get_selected_item() {
        if !item.is_group() {
            if let Ok(conn_id) = Uuid::parse_str(&item.id()) {
                Self::start_connection(state, notebook, conn_id);
            }
        }
    }
}
```

### 4. Group-Scoped Sorting

Add sorting methods to `ConnectionManager`:

```rust
impl ConnectionManager {
    /// Sorts connections within a specific group
    pub fn sort_group(&mut self, group_id: Uuid) -> Result<(), String> {
        let connections = self.get_connections_by_group(group_id);
        let mut sorted: Vec<_> = connections.iter().collect();
        sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        for (idx, conn) in sorted.iter().enumerate() {
            self.update_connection_sort_order(conn.id, idx as i32)?;
        }
        Ok(())
    }
    
    /// Sorts all connections globally
    pub fn sort_all(&mut self) -> Result<(), String> {
        // Sort root groups
        let mut groups: Vec<_> = self.get_root_groups();
        groups.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        for (idx, group) in groups.iter().enumerate() {
            self.update_group_sort_order(group.id, idx as i32)?;
            self.sort_group(group.id)?;
        }
        
        // Sort ungrouped connections
        let mut ungrouped: Vec<_> = self.get_ungrouped_connections();
        ungrouped.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        for (idx, conn) in ungrouped.iter().enumerate() {
            self.update_connection_sort_order(conn.id, idx as i32)?;
        }
        
        Ok(())
    }
}
```

### 5. Drag-and-Drop Implementation

Implement the drag-and-drop handlers in sidebar:

```rust
// In setup_list_item
drag_source.connect_prepare(|source, _x, _y| {
    if let Some(item) = get_item_from_source(source) {
        let data = format!("{}:{}", if item.is_group() { "group" } else { "conn" }, item.id());
        Some(gdk::ContentProvider::for_value(&data.to_value()))
    } else {
        None
    }
});

drop_target.connect_drop(|target, value, _x, _y| {
    let data = value.get::<String>().ok()?;
    let (item_type, item_id) = data.split_once(':')?;
    let target_item = get_item_from_target(target)?;
    
    // Emit signal to handle reordering in window.rs
    true
});
```

### 6. Sort Recent Feature

Add `last_connected` field to Connection model and sorting method:

```rust
// In models/connection.rs
pub struct Connection {
    // ... existing fields
    pub last_connected: Option<DateTime<Utc>>,
}

// In connection/manager.rs
impl ConnectionManager {
    pub fn sort_by_recent(&mut self) -> Result<(), String> {
        let mut connections: Vec<_> = self.list_connections().to_vec();
        connections.sort_by(|a, b| {
            match (&b.last_connected, &a.last_connected) {
                (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            }
        });
        
        for (idx, conn) in connections.iter().enumerate() {
            self.update_connection_sort_order(conn.id, idx as i32)?;
        }
        Ok(())
    }
    
    pub fn update_last_connected(&mut self, connection_id: Uuid) -> Result<(), String> {
        if let Some(conn) = self.get_connection_mut(connection_id) {
            conn.last_connected = Some(Utc::now());
            self.save()?;
        }
        Ok(())
    }
}
```

### 7. Client Detection

Add client detection module:

```rust
// In rustconn-core/src/protocol/detection.rs
pub struct ClientInfo {
    pub name: String,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub installed: bool,
}

pub fn detect_ssh_client() -> ClientInfo {
    detect_client("ssh", &["ssh", "openssh"], &["-V"])
}

pub fn detect_rdp_client() -> ClientInfo {
    detect_client("xfreerdp", &["xfreerdp", "xfreerdp3"], &["--version"])
        .or_else(|| detect_client("rdesktop", &["rdesktop"], &["--version"]))
}

pub fn detect_vnc_client() -> ClientInfo {
    detect_client("vncviewer", &["vncviewer", "tigervnc"], &["-h"])
}

fn detect_client(name: &str, binaries: &[&str], version_args: &[&str]) -> ClientInfo {
    for binary in binaries {
        if let Ok(output) = Command::new(binary).args(version_args).output() {
            let version = parse_version(&output.stderr, &output.stdout);
            return ClientInfo {
                name: name.to_string(),
                path: which::which(binary).ok(),
                version,
                installed: true,
            };
        }
    }
    ClientInfo {
        name: name.to_string(),
        path: None,
        version: None,
        installed: false,
    }
}
```

### 8. Quick Connect Protocol Selection

Update Quick Connect dialog to include protocol dropdown:

```rust
fn show_quick_connect_dialog(window: &ApplicationWindow, notebook: SharedNotebook) {
    let dialog = Window::builder()
        .title("Quick Connect")
        .modal(true)
        .transient_for(window)
        .build();
    
    // Protocol dropdown
    let protocol_list = StringList::new(&["SSH", "RDP", "VNC"]);
    let protocol_dropdown = DropDown::new(Some(protocol_list), gtk4::Expression::NONE);
    
    // Host entry
    let host_entry = Entry::new();
    
    // Port spin (updates based on protocol)
    let port_spin = SpinButton::with_range(1.0, 65535.0, 1.0);
    port_spin.set_value(22.0);
    
    // Connect protocol change to port update
    let port_clone = port_spin.clone();
    protocol_dropdown.connect_selected_notify(move |dropdown| {
        let default_port = match dropdown.selected() {
            0 => 22.0,   // SSH
            1 => 3389.0, // RDP
            2 => 5900.0, // VNC
            _ => 22.0,
        };
        port_clone.set_value(default_port);
    });
    
    // ... rest of dialog
}
```

## Data Models

### Connection Model Update

```rust
pub struct Connection {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub protocol_config: ProtocolConfig,
    pub group_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub sort_order: i32,
    pub last_connected: Option<DateTime<Utc>>,  // NEW
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Client Detection Result

```rust
pub struct ClientInfo {
    pub name: String,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub installed: bool,
}

pub struct ClientDetectionResult {
    pub ssh: ClientInfo,
    pub rdp: ClientInfo,
    pub vnc: ClientInfo,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing the prework, the following properties are consolidated:
- Properties 4.1, 4.2, 4.3 can be combined into a single sorting property
- Properties 5.1, 5.2, 5.3, 5.4 can be combined into a drag-drop property
- Properties 7.2, 7.3, 7.4 can be combined into a client detection property
- Properties 8.2, 8.3, 8.4 are examples, not properties

### Property 1: Connection Validation

*For any* connection data with empty name or empty host, validation SHALL return an error.

**Validates: Requirements 1.1, 1.2**

### Property 2: Asbru Hostname Extraction

*For any* Asbru entry with hostname in ip, host, name, or title field, the importer SHALL extract a valid hostname.

**Validates: Requirements 2.1, 2.2**

### Property 3: Connection Duplication

*For any* connection, duplicating it SHALL create a new connection with "(copy)" suffix and different UUID while preserving all other fields.

**Validates: Requirements 3.3**

### Property 4: Group Deletion Cascade

*For any* group with connections, deleting the group SHALL remove all connections within that group.

**Validates: Requirements 3.7**

### Property 5: Group-Scoped Sorting

*For any* set of connections, sorting within a group SHALL only reorder connections in that group, leaving other groups unchanged.

**Validates: Requirements 4.1, 4.2, 4.3, 4.4**

### Property 6: Drag-Drop Reordering

*For any* connection moved via drag-drop, the connection's group_id and sort_order SHALL be updated correctly.

**Validates: Requirements 5.1, 5.2, 5.3, 5.5**

### Property 7: Recent Sort Ordering

*For any* set of connections with timestamps, sorting by recent SHALL place connections with more recent timestamps first, and connections without timestamps last.

**Validates: Requirements 6.1, 6.2**

### Property 8: Last Connected Update

*For any* connection, after connecting, the last_connected timestamp SHALL be updated to current time.

**Validates: Requirements 6.4**

### Property 9: Client Detection

*For any* installed client binary, detection SHALL return installed=true with version string.

**Validates: Requirements 7.2, 7.3, 7.4**

### Property 10: Protocol Port Defaults

*For any* protocol selection in Quick Connect, the default port SHALL match the protocol standard (SSH=22, RDP=3389, VNC=5900).

**Validates: Requirements 8.2, 8.3, 8.4, 8.5**

## Error Handling

- Connection dialog validation errors display via `AlertDialog`
- Import errors collected and displayed in summary
- Client detection failures return `installed: false` with helpful message
- Drag-drop failures silently ignored (no state change)
- Sorting failures logged but don't crash application

## Testing Strategy

### Property-Based Testing

Using `proptest` crate for property-based tests:

1. **Connection Validation**: Generate random connection data, verify validation catches invalid inputs
2. **Asbru Import**: Generate Asbru YAML entries, verify hostname extraction
3. **Sorting**: Generate connection lists, verify sort order properties
4. **Drag-Drop**: Generate reorder operations, verify state consistency
5. **Recent Sort**: Generate connections with timestamps, verify ordering

### Unit Tests

- Client detection with mocked binaries
- Protocol port defaults
- Connection duplication
- Group deletion cascade

### Integration Tests

- Full import workflow
- Context menu action execution
- Settings persistence

