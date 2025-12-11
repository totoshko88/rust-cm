# Requirements Document

## Introduction

This document describes enhancements for RustConn - a connection manager for Linux. The enhancements include:
1. Embedded tabs for VNC and RDP connections instead of external windows
2. Full-featured Group Operations Mode with multi-selection
3. Bulk deletion of selected connections
4. Wire up unused dialog methods and state management functions
5. Snippet management UI integration
6. Session logging functionality

## Glossary

- **RustConn**: Connection manager application for Linux
- **Multi-Selection Mode**: Mode allowing selection of multiple items simultaneously
- **Group Operations Mode**: Mode for bulk actions on connections
- **Embedded Tab**: Tab with embedded widget inside the main window
- **FreeRDP**: RDP client with embedding support
- **VNC Viewer**: VNC client for remote graphical access
- **GTK4 MultiSelection**: GTK4 selection model for multiple item selection
- **Snippet**: Reusable command template with variable substitution
- **Session Logging**: Recording terminal output to timestamped log files

## Requirements

### Requirement 1: Embedded VNC/RDP Tabs

**User Story:** As a user, I want VNC and RDP connections to open in tabs within the main window, so that I have a unified interface for all connection types.

#### Acceptance Criteria

1. WHEN a user initiates an RDP connection THEN RustConn SHALL display the RDP session in an embedded tab within the main window using FreeRDP with embedding support
2. WHEN a user initiates a VNC connection THEN RustConn SHALL display the VNC session in an embedded tab within the main window
3. WHEN an embedded RDP/VNC session is active THEN RustConn SHALL provide controls for fullscreen toggle and session disconnect
4. WHEN the embedded session window is resized THEN RustConn SHALL scale the remote desktop view appropriately
5. IF embedding fails due to Wayland limitations THEN RustConn SHALL fall back to launching an external window and notify the user

### Requirement 2: Group Operations Mode with Multi-Selection

**User Story:** As an administrator, I want to select multiple connections simultaneously, so that I can perform bulk operations on them.

#### Acceptance Criteria

1. WHEN a user activates Group Operations Mode THEN RustConn SHALL switch the connection list to multi-selection mode with checkboxes
2. WHEN Group Operations Mode is active THEN RustConn SHALL allow selection of multiple connections using Ctrl+Click for individual items
3. WHEN Group Operations Mode is active THEN RustConn SHALL allow selection of all visible connections using Ctrl+A
4. WHEN Group Operations Mode is active THEN RustConn SHALL display a toolbar with available bulk actions (Delete Selected, Move to Group)
5. WHEN a user clicks outside the list or presses Escape THEN RustConn SHALL deselect all items but remain in Group Operations Mode
6. WHEN a user deactivates Group Operations Mode THEN RustConn SHALL return to single-selection mode and clear all selections

### Requirement 3: Bulk Connection Deletion

**User Story:** As a user, I want to delete multiple connections at once, so that I can quickly clean up outdated entries.

#### Acceptance Criteria

1. WHEN a user selects multiple connections and clicks Delete Selected THEN RustConn SHALL display a confirmation dialog listing all selected items
2. WHEN the user confirms deletion THEN RustConn SHALL remove all selected connections from storage
3. WHEN deletion completes THEN RustConn SHALL update the sidebar and display a summary of deleted items count
4. IF any connection fails to delete THEN RustConn SHALL continue deleting remaining items and report failures at the end

### Requirement 4: Connection Dialog Integration

**User Story:** As a user, I want the connection dialog to properly validate and save connections, so that I can create and edit connections reliably.

#### Acceptance Criteria

1. WHEN a user fills the connection form and clicks Save THEN RustConn SHALL validate all required fields before saving
2. WHEN validation fails THEN RustConn SHALL display specific error messages for invalid fields
3. WHEN a user edits an existing connection THEN RustConn SHALL populate all fields with current values including protocol-specific options
4. WHEN building SSH config THEN RustConn SHALL parse custom options text into key-value pairs
5. WHEN building RDP/VNC config THEN RustConn SHALL parse custom arguments into argument list

### Requirement 5: Import Dialog Integration

**User Story:** As a user migrating from another tool, I want the import dialog to show progress and results, so that I know what was imported.

#### Acceptance Criteria

1. WHEN a user selects an import source THEN RustConn SHALL detect available sources and enable the import button
2. WHEN import completes THEN RustConn SHALL display a results summary with successful, skipped, and failed counts
3. WHEN import encounters errors THEN RustConn SHALL display error details for each failed entry

### Requirement 6: Settings Dialog Integration

**User Story:** As a user, I want to configure application settings, so that I can customize RustConn behavior.

#### Acceptance Criteria

1. WHEN a user opens settings THEN RustConn SHALL display current settings values
2. WHEN a user saves settings THEN RustConn SHALL persist all changed values to configuration file
3. WHEN settings are saved THEN RustConn SHALL apply changes immediately without restart

### Requirement 7: Snippet Management

**User Story:** As a power user, I want to create and execute command snippets, so that I can quickly run frequently used commands.

#### Acceptance Criteria

1. WHEN a user creates a snippet THEN RustConn SHALL validate name and command fields before saving
2. WHEN a user adds variables to a snippet THEN RustConn SHALL store variable name, description, and default value
3. WHEN a user executes a snippet THEN RustConn SHALL prompt for variable values and send the substituted command to the active terminal
4. WHEN a user searches snippets THEN RustConn SHALL filter by name, command content, or category

### Requirement 8: Session Management

**User Story:** As a user, I want to manage active sessions, so that I can monitor and control my connections.

#### Acceptance Criteria

1. WHEN a user starts a session THEN RustConn SHALL track the session in the session manager
2. WHEN a user terminates a session THEN RustConn SHALL close the terminal and clean up resources
3. WHEN session logging is enabled THEN RustConn SHALL write terminal output to timestamped log files
4. WHEN a terminal child process exits THEN RustConn SHALL update the session status and optionally close the tab

### Requirement 9: Group Hierarchy Management

**User Story:** As a user, I want to organize connections in nested groups, so that I can structure my connections logically.

#### Acceptance Criteria

1. WHEN a user creates a group with a parent THEN RustConn SHALL create the group as a child of the specified parent
2. WHEN a user moves a connection to a group THEN RustConn SHALL update the connection's group_id
3. WHEN a user requests a group path THEN RustConn SHALL return the full path from root to the group
4. WHEN a user reorders groups THEN RustConn SHALL update sort_order values accordingly

### Requirement 10: Fix GTK Dialog Warning

**User Story:** As a developer, I want dialogs to have proper transient parents, so that GTK warnings are eliminated.

#### Acceptance Criteria

1. WHEN any dialog is created THEN RustConn SHALL set the transient parent to the main window
2. WHEN a dialog is mapped THEN RustConn SHALL NOT produce GTK warnings about missing transient parent
