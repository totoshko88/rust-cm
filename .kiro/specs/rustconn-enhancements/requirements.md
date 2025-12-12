# Requirements Document

## Introduction

This document specifies enhancements for RustConn connection manager including custom SSH config file import, RDP shared folder support, progress indicators for long operations, and split-screen terminal views.

## Glossary

- **RustConn**: The connection manager application
- **SSH Config**: OpenSSH configuration file format (~/.ssh/config)
- **RDP**: Remote Desktop Protocol for Windows remote connections
- **Shared Folder**: A local directory shared with the remote RDP session
- **Progress Indicator**: Visual feedback showing operation progress
- **Split View**: Multiple terminal panes in a single window with shared tab list

## Requirements

### Requirement 1: Custom SSH Config File Import

**User Story:** As a user, I want to import connections from any SSH config file, so that I can use configuration files stored in non-standard locations.

#### Acceptance Criteria

1. WHEN a user selects "SSH Config File" import source THEN the system SHALL display a file chooser dialog for selecting any file
2. WHEN a user selects a valid SSH config file THEN the system SHALL parse all Host entries and display them for import
3. WHEN parsing an SSH config file THEN the system SHALL extract hostname, port, user, and identity file settings
4. IF the selected file is not a valid SSH config format THEN the system SHALL display an error message describing the issue
5. WHEN displaying parsed hosts THEN the system SHALL show a preview with connection count before import

### Requirement 2: RDP Shared Folder Configuration

**User Story:** As a user, I want to configure shared folders for RDP connections, so that I can access local files from the remote Windows session.

#### Acceptance Criteria

1. WHEN editing an RDP connection THEN the system SHALL display a "Shared Folders" section in the connection dialog
2. WHEN a user adds a shared folder THEN the system SHALL allow selecting a local directory path
3. WHEN a user adds a shared folder THEN the system SHALL allow specifying a share name for the remote session
4. WHEN connecting via RDP with shared folders THEN the system SHALL pass the folder configuration to the RDP client
5. WHEN a user removes a shared folder THEN the system SHALL update the connection configuration immediately
6. WHEN displaying shared folders THEN the system SHALL show the local path and share name in a list view

### Requirement 3: Progress Indicators for Long Operations

**User Story:** As a user, I want to see progress during long operations, so that I know the application is working and can estimate completion time.

#### Acceptance Criteria

1. WHEN importing connections from external sources THEN the system SHALL display a progress dialog with current item count
2. WHEN exporting configuration THEN the system SHALL display a progress indicator during file writing
3. WHEN performing bulk operations on connections THEN the system SHALL display progress with item count
4. WHILE a long operation is in progress THEN the system SHALL keep the UI responsive using async processing
5. WHEN a progress dialog is shown THEN the system SHALL display a cancel button for interruptible operations
6. WHEN an operation completes THEN the system SHALL close the progress dialog and show a summary

### Requirement 4: Split-Screen Terminal Views

**User Story:** As a user, I want to split the terminal area horizontally or vertically, so that I can view multiple sessions simultaneously while maintaining a single tab list.

#### Acceptance Criteria

1. WHEN a user triggers horizontal split THEN the system SHALL divide the current pane into top and bottom sections
2. WHEN a user triggers vertical split THEN the system SHALL divide the current pane into left and right sections
3. WHEN split views exist THEN the system SHALL maintain a single unified tab list for all sessions
4. WHEN a user selects a tab THEN the system SHALL display that session in the currently focused pane
5. WHEN a user drags a pane divider THEN the system SHALL resize adjacent panes dynamically
6. WHEN a user closes a split pane THEN the system SHALL merge the remaining content appropriately
7. WHEN multiple panes exist THEN the system SHALL highlight the currently focused pane visually
8. WHEN a user presses a keyboard shortcut THEN the system SHALL cycle focus between split panes
