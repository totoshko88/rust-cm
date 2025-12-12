# Requirements Document

## Introduction

This document describes bug fixes and enhancements for RustConn - a connection manager for Linux. The issues include:
1. Save button not clickable in New Connection dialog
2. Asbru-CM import skipping entries with "No hostname specified"
3. Context menu operations not working (Connect, Edit, Duplicate, Move to Group, Delete)
4. Sort button sorting all connections instead of only selected group
5. Drag-and-drop connection reordering not implemented
6. Missing "Sort Recent" button for recently used connections
7. Missing Settings tab for checking installed RDP/SSH/VNC clients
8. Quick Connection missing protocol selection
9. Dead code warnings indicating unused functionality

## Glossary

- **RustConn**: Connection manager application for Linux
- **Connection Dialog**: Modal window for creating/editing connections
- **Context Menu**: Right-click popup menu with item-specific actions
- **Asbru-CM**: Legacy connection manager application (import source)
- **Quick Connect**: Feature to connect without saving connection details
- **Sort Recent**: Sorting connections by last used timestamp
- **Protocol Client**: External application for connecting (ssh, xfreerdp, vncviewer)

## Requirements

### Requirement 1: Fix Connection Dialog Save Button

**User Story:** As a user, I want the Save button to work when creating a new connection, so that I can save my connection details.

#### Acceptance Criteria

1. WHEN a user fills in valid connection details and clicks Save THEN RustConn SHALL save the connection and close the dialog
2. WHEN a user clicks Save with invalid data THEN RustConn SHALL display validation errors and keep the dialog open
3. WHEN a user clicks Cancel THEN RustConn SHALL close the dialog without saving
4. WHEN the connection is saved THEN RustConn SHALL refresh the sidebar to show the new connection

### Requirement 2: Fix Asbru-CM Import

**User Story:** As a user migrating from Asbru-CM, I want to import my connections including those with special configurations, so that I can use them in RustConn.

#### Acceptance Criteria

1. WHEN importing Asbru-CM entries with "tmp" or placeholder names THEN RustConn SHALL attempt to extract hostname from other fields
2. WHEN an entry has no hostname in any field THEN RustConn SHALL skip the entry with a descriptive message
3. WHEN importing entries with dynamic variables THEN RustConn SHALL preserve the variable syntax for later substitution
4. WHEN import completes THEN RustConn SHALL display count of successful, skipped, and failed imports

### Requirement 3: Fix Context Menu Operations

**User Story:** As a user, I want context menu actions to work on the selected connection, so that I can quickly perform operations.

#### Acceptance Criteria

1. WHEN a user right-clicks a connection and selects Connect THEN RustConn SHALL initiate a connection to that host
2. WHEN a user right-clicks a connection and selects Edit THEN RustConn SHALL open the connection dialog with current values
3. WHEN a user right-clicks a connection and selects Duplicate THEN RustConn SHALL create a copy with "(copy)" suffix
4. WHEN a user right-clicks a connection and selects Move to Group THEN RustConn SHALL display a group selection dialog
5. WHEN a user right-clicks a connection and selects Delete THEN RustConn SHALL display a confirmation dialog before deletion
6. WHEN a user right-clicks a group and selects Edit THEN RustConn SHALL open a group rename dialog
7. WHEN a user right-clicks a group and selects Delete THEN RustConn SHALL confirm and delete the group with its connections

### Requirement 4: Fix Group-Scoped Sorting

**User Story:** As a user, I want to sort connections contextually based on selection, so that I can organize my connections efficiently.

#### Acceptance Criteria

1. WHEN a user selects a group and clicks Sort THEN RustConn SHALL sort only connections within that selected group alphabetically
2. WHEN no group is selected and user clicks Sort THEN RustConn SHALL sort all connections across all groups alphabetically
3. WHEN a connection is selected (not a group) and user clicks Sort THEN RustConn SHALL sort all connections across all groups alphabetically
4. WHEN sorting completes THEN RustConn SHALL persist the sort order to configuration
5. WHEN a group is expanded THEN RustConn SHALL display connections in their sorted order

### Requirement 5: Implement Drag-and-Drop Reordering

**User Story:** As a user, I want to drag connections to reorder them or move them between groups, so that I can organize my connections visually.

#### Acceptance Criteria

1. WHEN a user drags a connection within the same group THEN RustConn SHALL reorder the connection to the drop position
2. WHEN a user drags a connection to a different group THEN RustConn SHALL move the connection to that group
3. WHEN a user drags a connection to root level THEN RustConn SHALL remove the connection from its current group
4. WHEN a user drags a group THEN RustConn SHALL reorder the group among other groups
5. WHEN drag operation completes THEN RustConn SHALL persist the new order to configuration

### Requirement 6: Add Sort Recent Button

**User Story:** As a user, I want to sort connections by recent usage, so that I can quickly access frequently used connections.

#### Acceptance Criteria

1. WHEN a user clicks Sort Recent THEN RustConn SHALL sort connections by last_connected timestamp descending
2. WHEN connections have no last_connected timestamp THEN RustConn SHALL place them at the end of the list
3. WHEN Sort Recent is active THEN RustConn SHALL display a visual indicator on the button
4. WHEN a user connects to a host THEN RustConn SHALL update the last_connected timestamp

### Requirement 7: Add Client Detection Settings Tab

**User Story:** As a user, I want to see which protocol clients are installed on my system, so that I can verify my setup.

#### Acceptance Criteria

1. WHEN a user opens Settings THEN RustConn SHALL display a "Clients" tab with detected clients
2. WHEN detecting SSH client THEN RustConn SHALL check for ssh binary and display its version
3. WHEN detecting RDP client THEN RustConn SHALL check for xfreerdp/rdesktop and display version
4. WHEN detecting VNC client THEN RustConn SHALL check for vncviewer/tigervnc and display version
5. WHEN a client is not found THEN RustConn SHALL display "Not installed" with installation hint
6. WHEN a user clicks Refresh THEN RustConn SHALL re-detect all clients

### Requirement 8: Add Protocol Selection to Quick Connect

**User Story:** As a user, I want to select the protocol when using Quick Connect, so that I can connect to different types of servers.

#### Acceptance Criteria

1. WHEN a user opens Quick Connect THEN RustConn SHALL display a protocol dropdown with SSH, RDP, VNC options
2. WHEN SSH is selected THEN RustConn SHALL use port 22 as default
3. WHEN RDP is selected THEN RustConn SHALL use port 3389 as default
4. WHEN VNC is selected THEN RustConn SHALL use port 5900 as default
5. WHEN a user changes protocol THEN RustConn SHALL update the default port if port was not manually changed

### Requirement 9: Wire Up Unused Code

**User Story:** As a developer, I want all implemented functionality to be connected to the UI, so that there are no dead code warnings.

#### Acceptance Criteria

1. WHEN building RustConn THEN the compiler SHALL NOT produce dead_code warnings for public API methods
2. WHEN EmbeddedSessionTab methods exist THEN RustConn SHALL use them for RDP/VNC session management
3. WHEN SessionControls methods exist THEN RustConn SHALL connect them to UI buttons
4. WHEN AppState methods exist THEN RustConn SHALL call them from appropriate UI actions
5. WHEN TerminalNotebook methods exist THEN RustConn SHALL use them for session operations

