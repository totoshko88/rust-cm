# Implementation Plan

- [x] 1. Extend SecretSettings model
  - [x] 1.1 Add KDBX configuration fields to SecretSettings
    - Add `kdbx_path: Option<PathBuf>` field
    - Add `kdbx_enabled: bool` field
    - Add `SecretBackendType::KdbxFile` variant
    - Ensure password is NOT serialized (use `#[serde(skip)]`)
    - _Requirements: 1.1, 5.1, 5.3_
  - [x] 1.2 Write property test for settings serialization round-trip
    - **Property 3: Settings Serialization Round-Trip**
    - **Validates: Requirements 5.1, 5.2, 5.3**

- [x] 2. Implement KeePass status detection
  - [x] 2.1 Create KeePassStatus struct and detection logic
    - Implement `KeePassStatus::detect()` to find KeePassXC binary
    - Implement version parsing from `keepassxc-cli --version`
    - Implement `validate_kdbx_path()` for file validation
    - _Requirements: 2.1, 2.2, 2.3_
  - [x] 2.2 Write property test for KDBX path validation
    - **Property 1: KDBX Path Validation**
    - **Validates: Requirements 1.2**
  - [x] 2.3 Write property test for version string parsing
    - **Property 7: Version String Parsing**
    - **Validates: Requirements 2.2**

- [x] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement credential resolution chain
  - [x] 4.1 Create CredentialResolver with lookup key generation
    - Implement `generate_lookup_key()` using connection name/host
    - Implement `resolve()` method with fallback chain logic
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - [x] 4.2 Write property test for lookup key generation
    - **Property 6: Lookup Key Generation**
    - **Validates: Requirements 4.4**
  - [x] 4.3 Write property test for credential resolution chain
    - **Property 5: Credential Resolution Chain**
    - **Validates: Requirements 4.1, 4.2, 4.3**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Update Settings dialog - Secrets tab
  - [x] 6.1 Redesign Secrets tab with KeePass configuration
    - Add KDBX file path entry with browse button
    - Add password entry field (masked)
    - Add unlock/lock button with status indicator
    - Add KeePassXC status display (installed, version, path)
    - Add integration enable/disable switch
    - _Requirements: 1.1, 1.5, 2.1, 2.2, 2.3, 2.4_
  - [x] 6.2 Implement KDBX file selection with FileDialog
    - Use GTK4 FileDialog for portal-compatible file selection
    - Filter for .kdbx files
    - Validate selected path
    - _Requirements: 1.2_

- [x] 7. Update Connection dialog with password options
  - [x] 7.1 Add password source selection to connection dialog
    - Add PasswordSource dropdown (None, Stored, KeePass, Keyring, Prompt)
    - Wire up to Connection.password_source field
    - _Requirements: 3.1, 3.5_
  - [x] 7.2 Add "Save to KeePass" button
    - Add button to connection dialog
    - Enable/disable based on KeePass integration state
    - Implement save action using SecretManager
    - _Requirements: 3.2, 3.3, 3.4_
  - [x] 7.3 Write property test for button state consistency
    - **Property 4: Button State Consistency**
    - **Validates: Requirements 3.2, 3.3**

- [x] 8. Integrate credential resolution into connection flow
  - [x] 8.1 Update connection launch to use CredentialResolver
    - Integrate resolver into connection establishment
    - Handle fallback scenarios
    - Prompt user when no credentials available
    - _Requirements: 4.1, 4.2, 4.3_

- [x] 9. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
