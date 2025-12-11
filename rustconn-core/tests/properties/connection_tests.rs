//! Property-based tests for Connection CRUD operations
//!
//! **Feature: rustconn, Property 1: Connection CRUD Data Integrity**
//! **Validates: Requirements 1.1, 1.2, 1.3**

use proptest::prelude::*;
use rustconn_core::{
    ConfigManager, Connection, ConnectionManager, ProtocolConfig, RdpClient, RdpConfig,
    RdpGateway, Resolution, SshAuthMethod, SshConfig, VncClient, VncConfig,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;

// ========== Generators ==========

// Strategy for generating valid connection names (non-empty)
fn arb_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,31}".prop_map(|s| s)
}

// Strategy for generating valid hostnames (non-empty)
fn arb_host() -> impl Strategy<Value = String> {
    "[a-z0-9]([a-z0-9-]{0,15}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,15}[a-z0-9])?)*"
        .prop_map(|s| s)
}

// Strategy for generating valid ports (non-zero)
fn arb_port() -> impl Strategy<Value = u16> {
    1u16..=65535u16
}

// Strategy for generating optional usernames
fn arb_username() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[a-z][a-z0-9_]{0,15}".prop_map(Some),
    ]
}

// Strategy for generating tags
fn arb_tags() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-z]{1,10}", 0..5)
}

// Strategy for SSH auth method
fn arb_ssh_auth_method() -> impl Strategy<Value = SshAuthMethod> {
    prop_oneof![
        Just(SshAuthMethod::Password),
        Just(SshAuthMethod::PublicKey),
        Just(SshAuthMethod::KeyboardInteractive),
        Just(SshAuthMethod::Agent),
    ]
}

// Strategy for optional PathBuf
fn arb_optional_path() -> impl Strategy<Value = Option<PathBuf>> {
    prop_oneof![
        Just(None),
        "[a-z]{1,10}(/[a-z]{1,10}){0,3}".prop_map(|s| Some(PathBuf::from(s))),
    ]
}

// Strategy for optional string
fn arb_optional_string() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[a-zA-Z0-9_-]{1,20}".prop_map(Some),
    ]
}

// Strategy for custom SSH options
fn arb_custom_options() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map("[A-Za-z]{1,20}", "[a-zA-Z0-9]{1,10}", 0..3)
}

// Strategy for SSH config
fn arb_ssh_config() -> impl Strategy<Value = SshConfig> {
    (
        arb_ssh_auth_method(),
        arb_optional_path(),
        arb_optional_string(),
        any::<bool>(),
        arb_custom_options(),
        arb_optional_string(),
    )
        .prop_map(
            |(auth_method, key_path, proxy_jump, use_control_master, custom_options, startup_command)| {
                SshConfig {
                    auth_method,
                    key_path,
                    proxy_jump,
                    use_control_master,
                    custom_options,
                    startup_command,
                }
            },
        )
}

// Strategy for RDP client
fn arb_rdp_client() -> impl Strategy<Value = RdpClient> {
    prop_oneof![
        Just(RdpClient::FreeRdp),
        "[a-z]{1,10}(/[a-z]{1,10}){0,2}".prop_map(|s| RdpClient::Custom(PathBuf::from(s))),
    ]
}

// Strategy for optional resolution
fn arb_optional_resolution() -> impl Strategy<Value = Option<Resolution>> {
    prop_oneof![
        Just(None),
        (640u32..4096u32, 480u32..2160u32).prop_map(|(w, h)| Some(Resolution::new(w, h))),
    ]
}

// Strategy for optional color depth
fn arb_optional_color_depth() -> impl Strategy<Value = Option<u8>> {
    prop_oneof![
        Just(None),
        prop_oneof![Just(8u8), Just(15u8), Just(16u8), Just(24u8), Just(32u8)].prop_map(Some),
    ]
}

// Strategy for optional RDP gateway
fn arb_optional_gateway() -> impl Strategy<Value = Option<RdpGateway>> {
    prop_oneof![
        Just(None),
        (arb_host(), 1u16..65535u16, arb_optional_string()).prop_map(|(hostname, port, username)| {
            Some(RdpGateway {
                hostname,
                port,
                username,
            })
        }),
    ]
}

// Strategy for custom args
fn arb_custom_args() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-zA-Z0-9_=-]{1,20}", 0..3)
}

// Strategy for RDP config
fn arb_rdp_config() -> impl Strategy<Value = RdpConfig> {
    (
        arb_rdp_client(),
        arb_optional_resolution(),
        arb_optional_color_depth(),
        any::<bool>(),
        arb_optional_gateway(),
        arb_custom_args(),
    )
        .prop_map(
            |(client, resolution, color_depth, audio_redirect, gateway, custom_args)| RdpConfig {
                client,
                resolution,
                color_depth,
                audio_redirect,
                gateway,
                custom_args,
            },
        )
}

// Strategy for VNC client
fn arb_vnc_client() -> impl Strategy<Value = VncClient> {
    prop_oneof![
        Just(VncClient::TightVnc),
        Just(VncClient::TigerVnc),
        "[a-z]{1,10}(/[a-z]{1,10}){0,2}".prop_map(|s| VncClient::Custom(PathBuf::from(s))),
    ]
}

// Strategy for optional encoding
fn arb_optional_encoding() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        prop_oneof![
            Just("tight".to_string()),
            Just("zrle".to_string()),
            Just("hextile".to_string()),
        ]
        .prop_map(Some),
    ]
}

// Strategy for optional compression/quality (0-9)
fn arb_optional_level() -> impl Strategy<Value = Option<u8>> {
    prop_oneof![
        Just(None),
        (0u8..=9u8).prop_map(Some),
    ]
}

// Strategy for VNC config
fn arb_vnc_config() -> impl Strategy<Value = VncConfig> {
    (
        arb_vnc_client(),
        arb_optional_encoding(),
        arb_optional_level(),
        arb_optional_level(),
        arb_custom_args(),
    )
        .prop_map(|(client, encoding, compression, quality, custom_args)| VncConfig {
            client,
            encoding,
            compression,
            quality,
            custom_args,
        })
}

// Strategy for protocol config
fn arb_protocol_config() -> impl Strategy<Value = ProtocolConfig> {
    prop_oneof![
        arb_ssh_config().prop_map(ProtocolConfig::Ssh),
        arb_rdp_config().prop_map(ProtocolConfig::Rdp),
        arb_vnc_config().prop_map(ProtocolConfig::Vnc),
    ]
}

// Strategy for generating a complete Connection
fn arb_connection() -> impl Strategy<Value = Connection> {
    (
        arb_name(),
        arb_host(),
        arb_port(),
        arb_protocol_config(),
        arb_username(),
        arb_tags(),
    )
        .prop_map(|(name, host, port, protocol_config, username, tags)| {
            let mut conn = Connection::new(name, host, port, protocol_config);
            if let Some(u) = username {
                conn = conn.with_username(u);
            }
            if !tags.is_empty() {
                conn = conn.with_tags(tags);
            }
            conn
        })
}

// Helper to create a test ConnectionManager
fn create_test_manager() -> (ConnectionManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config_manager = ConfigManager::with_config_dir(temp_dir.path().to_path_buf());
    let manager = ConnectionManager::new(config_manager).unwrap();
    (manager, temp_dir)
}


proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: rustconn, Property 1: Connection CRUD Data Integrity**
    /// **Validates: Requirements 1.1, 1.2, 1.3**
    ///
    /// For any valid connection configuration, creating a connection and then
    /// retrieving it by ID should return a connection with identical name, host,
    /// port, protocol type, and all other configuration fields.
    #[test]
    fn create_then_retrieve_preserves_data(
        name in arb_name(),
        host in arb_host(),
        port in arb_port(),
        protocol_config in arb_protocol_config(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create connection
        let id = manager
            .create_connection(name.clone(), host.clone(), port, protocol_config.clone())
            .expect("Should create connection");

        // Retrieve connection
        let retrieved = manager
            .get_connection(id)
            .expect("Should retrieve connection");

        // Verify all fields are preserved
        prop_assert_eq!(retrieved.id, id, "ID should match");
        prop_assert_eq!(&retrieved.name, &name, "Name should be preserved");
        prop_assert_eq!(&retrieved.host, &host, "Host should be preserved");
        prop_assert_eq!(retrieved.port, port, "Port should be preserved");
        prop_assert_eq!(&retrieved.protocol_config, &protocol_config, "Protocol config should be preserved");
        prop_assert_eq!(retrieved.protocol, protocol_config.protocol_type(), "Protocol type should match config");
    }

    /// **Feature: rustconn, Property 1: Connection CRUD Data Integrity**
    /// **Validates: Requirements 1.1, 1.2**
    ///
    /// For any existing connection and valid update, updating the connection
    /// should preserve the original ID while changing only the specified fields.
    #[test]
    fn update_preserves_id_and_changes_fields(
        original_name in arb_name(),
        original_host in arb_host(),
        original_port in arb_port(),
        original_config in arb_protocol_config(),
        new_name in arb_name(),
        new_host in arb_host(),
        new_port in arb_port(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create original connection
        let id = manager
            .create_connection(original_name, original_host, original_port, original_config.clone())
            .expect("Should create connection");

        let original_created_at = manager.get_connection(id).unwrap().created_at;

        // Create updated connection with new values
        let mut updated = manager.get_connection(id).unwrap().clone();
        updated.name = new_name.clone();
        updated.host = new_host.clone();
        updated.port = new_port;

        // Update connection
        manager
            .update_connection(id, updated)
            .expect("Should update connection");

        // Retrieve updated connection
        let retrieved = manager
            .get_connection(id)
            .expect("Should retrieve updated connection");

        // Verify ID is preserved
        prop_assert_eq!(retrieved.id, id, "ID should be preserved after update");

        // Verify created_at is preserved
        prop_assert_eq!(
            retrieved.created_at.timestamp(),
            original_created_at.timestamp(),
            "Created timestamp should be preserved"
        );

        // Verify fields are updated
        prop_assert_eq!(&retrieved.name, &new_name, "Name should be updated");
        prop_assert_eq!(&retrieved.host, &new_host, "Host should be updated");
        prop_assert_eq!(retrieved.port, new_port, "Port should be updated");

        // Verify updated_at changed (should be >= created_at)
        prop_assert!(
            retrieved.updated_at >= retrieved.created_at,
            "Updated timestamp should be >= created timestamp"
        );
    }

    /// **Feature: rustconn, Property 1: Connection CRUD Data Integrity**
    /// **Validates: Requirements 1.3**
    ///
    /// For any existing connection, deleting it should result in the connection
    /// being absent from all queries.
    #[test]
    fn delete_removes_connection(
        name in arb_name(),
        host in arb_host(),
        port in arb_port(),
        protocol_config in arb_protocol_config(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create connection
        let id = manager
            .create_connection(name, host, port, protocol_config)
            .expect("Should create connection");

        // Verify it exists
        prop_assert!(manager.get_connection(id).is_some(), "Connection should exist before delete");
        prop_assert_eq!(manager.connection_count(), 1, "Should have 1 connection");

        // Delete connection
        manager
            .delete_connection(id)
            .expect("Should delete connection");

        // Verify it's gone
        prop_assert!(manager.get_connection(id).is_none(), "Connection should not exist after delete");
        prop_assert_eq!(manager.connection_count(), 0, "Should have 0 connections");

        // Verify it's not in list
        let all_connections = manager.list_connections();
        prop_assert!(
            !all_connections.iter().any(|c| c.id == id),
            "Deleted connection should not appear in list"
        );
    }

    /// **Feature: rustconn, Property 1: Connection CRUD Data Integrity**
    /// **Validates: Requirements 1.1, 1.3**
    ///
    /// Creating multiple connections and deleting one should only remove that
    /// specific connection, leaving others intact.
    #[test]
    fn delete_only_affects_target_connection(
        conn1 in arb_connection(),
        conn2 in arb_connection(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create two connections
        let id1 = manager
            .create_connection_from(conn1.clone())
            .expect("Should create first connection");

        let id2 = manager
            .create_connection_from(conn2.clone())
            .expect("Should create second connection");

        prop_assert_eq!(manager.connection_count(), 2, "Should have 2 connections");

        // Delete first connection
        manager
            .delete_connection(id1)
            .expect("Should delete first connection");

        // Verify first is gone
        prop_assert!(manager.get_connection(id1).is_none(), "First connection should be deleted");

        // Verify second still exists with correct data
        let remaining = manager
            .get_connection(id2)
            .expect("Second connection should still exist");

        prop_assert_eq!(&remaining.name, &conn2.name, "Second connection name should be preserved");
        prop_assert_eq!(&remaining.host, &conn2.host, "Second connection host should be preserved");
        prop_assert_eq!(remaining.port, conn2.port, "Second connection port should be preserved");
    }

    /// **Feature: rustconn, Property 1: Connection CRUD Data Integrity**
    /// **Validates: Requirements 1.1**
    ///
    /// Creating a connection from an existing Connection object should preserve
    /// all fields including the original ID.
    #[test]
    fn create_from_preserves_all_fields(conn in arb_connection()) {
        let (mut manager, _temp) = create_test_manager();

        let original_id = conn.id;

        // Create from existing connection
        let id = manager
            .create_connection_from(conn.clone())
            .expect("Should create connection from existing");

        // ID should be the same as the original
        prop_assert_eq!(id, original_id, "Should preserve original ID");

        // Retrieve and verify all fields
        let retrieved = manager
            .get_connection(id)
            .expect("Should retrieve connection");

        prop_assert_eq!(retrieved.id, conn.id, "ID should be preserved");
        prop_assert_eq!(&retrieved.name, &conn.name, "Name should be preserved");
        prop_assert_eq!(&retrieved.host, &conn.host, "Host should be preserved");
        prop_assert_eq!(retrieved.port, conn.port, "Port should be preserved");
        prop_assert_eq!(&retrieved.username, &conn.username, "Username should be preserved");
        prop_assert_eq!(&retrieved.tags, &conn.tags, "Tags should be preserved");
        prop_assert_eq!(&retrieved.protocol_config, &conn.protocol_config, "Protocol config should be preserved");
    }
}


// ========== Group Hierarchy Property Tests ==========

// Strategy for generating valid group names
fn arb_group_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_ -]{0,31}".prop_map(|s| s)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: rustconn, Property 14: Group Hierarchy Integrity**
    /// **Validates: Requirements 1.4**
    ///
    /// For any sequence of group creation and nesting operations, the resulting
    /// hierarchy should be acyclic (no group is its own ancestor).
    #[test]
    fn group_hierarchy_is_acyclic_after_creation(
        names in prop::collection::vec(arb_group_name(), 1..10),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create groups with random parent relationships
        let mut group_ids = Vec::new();

        for (i, name) in names.iter().enumerate() {
            let id = if i == 0 || group_ids.is_empty() {
                // First group is always root
                manager.create_group(name.clone()).expect("Should create root group")
            } else {
                // Randomly choose to be root or have a parent
                if i % 2 == 0 {
                    manager.create_group(name.clone()).expect("Should create root group")
                } else {
                    // Pick a random existing group as parent
                    let parent_idx = i % group_ids.len();
                    let parent_id = group_ids[parent_idx];
                    manager
                        .create_group_with_parent(name.clone(), parent_id)
                        .expect("Should create child group")
                }
            };
            group_ids.push(id);
        }

        // Verify hierarchy is acyclic
        prop_assert!(
            manager.validate_hierarchy(),
            "Hierarchy should be acyclic after group creation"
        );
    }

    /// **Feature: rustconn, Property 14: Group Hierarchy Integrity**
    /// **Validates: Requirements 1.4**
    ///
    /// Moving a group should never create a cycle in the hierarchy.
    #[test]
    fn move_group_prevents_cycles(
        names in prop::collection::vec(arb_group_name(), 3..6),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create a chain of groups: A -> B -> C -> ...
        let mut group_ids = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let id = if i == 0 {
                manager.create_group(name.clone()).expect("Should create root group")
            } else {
                let parent_id = group_ids[i - 1];
                manager
                    .create_group_with_parent(name.clone(), parent_id)
                    .expect("Should create child group")
            };
            group_ids.push(id);
        }

        // Try to move the root to be a child of the last group (would create cycle)
        if group_ids.len() >= 2 {
            let root_id = group_ids[0];
            let last_id = group_ids[group_ids.len() - 1];

            let result = manager.move_group(root_id, Some(last_id));
            prop_assert!(
                result.is_err(),
                "Moving root to be child of descendant should fail"
            );

            // Hierarchy should still be valid
            prop_assert!(
                manager.validate_hierarchy(),
                "Hierarchy should remain acyclic after failed move"
            );
        }
    }

    /// **Feature: rustconn, Property 14: Group Hierarchy Integrity**
    /// **Validates: Requirements 1.4**
    ///
    /// All parent references should point to existing groups.
    #[test]
    fn all_parent_references_are_valid(
        names in prop::collection::vec(arb_group_name(), 1..8),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create groups with various parent relationships
        let mut group_ids = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let id = if i == 0 {
                manager.create_group(name.clone()).expect("Should create root group")
            } else if i % 3 == 0 {
                // Create as root
                manager.create_group(name.clone()).expect("Should create root group")
            } else {
                // Create with parent
                let parent_idx = (i - 1) % group_ids.len();
                let parent_id = group_ids[parent_idx];
                manager
                    .create_group_with_parent(name.clone(), parent_id)
                    .expect("Should create child group")
            };
            group_ids.push(id);
        }

        // Verify all parent references point to existing groups
        for group in manager.list_groups() {
            if let Some(parent_id) = group.parent_id {
                prop_assert!(
                    manager.get_group(parent_id).is_some(),
                    "Parent reference should point to existing group"
                );
            }
        }
    }

    /// **Feature: rustconn, Property 14: Group Hierarchy Integrity**
    /// **Validates: Requirements 1.4**
    ///
    /// Deleting a group should maintain valid parent references for child groups.
    #[test]
    fn delete_group_maintains_valid_references(
        parent_name in arb_group_name(),
        child_name in arb_group_name(),
        grandchild_name in arb_group_name(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create three-level hierarchy
        let parent_id = manager.create_group(parent_name).expect("Should create parent");
        let child_id = manager
            .create_group_with_parent(child_name, parent_id)
            .expect("Should create child");
        let grandchild_id = manager
            .create_group_with_parent(grandchild_name, child_id)
            .expect("Should create grandchild");

        // Delete the middle group
        manager.delete_group(child_id).expect("Should delete child group");

        // Grandchild should now point to parent (the deleted group's parent)
        let grandchild = manager.get_group(grandchild_id).expect("Grandchild should exist");
        prop_assert_eq!(
            grandchild.parent_id,
            Some(parent_id),
            "Grandchild should be moved to deleted group's parent"
        );

        // Hierarchy should still be valid
        prop_assert!(
            manager.validate_hierarchy(),
            "Hierarchy should be valid after group deletion"
        );

        // All parent references should be valid
        for group in manager.list_groups() {
            if let Some(pid) = group.parent_id {
                prop_assert!(
                    manager.get_group(pid).is_some(),
                    "All parent references should be valid after deletion"
                );
            }
        }
    }

    /// **Feature: rustconn, Property 14: Group Hierarchy Integrity**
    /// **Validates: Requirements 1.4**
    ///
    /// Moving a group to root (None parent) should always succeed and maintain hierarchy.
    #[test]
    fn move_to_root_always_succeeds(
        parent_name in arb_group_name(),
        child_name in arb_group_name(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create parent-child relationship
        let parent_id = manager.create_group(parent_name).expect("Should create parent");
        let child_id = manager
            .create_group_with_parent(child_name, parent_id)
            .expect("Should create child");

        // Move child to root
        manager
            .move_group(child_id, None)
            .expect("Moving to root should succeed");

        // Verify child is now root
        let child = manager.get_group(child_id).expect("Child should exist");
        prop_assert!(child.parent_id.is_none(), "Child should be root after move");

        // Hierarchy should be valid
        prop_assert!(
            manager.validate_hierarchy(),
            "Hierarchy should be valid after move to root"
        );
    }

    /// **Feature: rustconn-enhancements, Property 6: Group Hierarchy Acyclicity**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// For any sequence of group creation and move operations, the resulting
    /// group hierarchy must remain acyclic - no group can be its own ancestor.
    #[test]
    fn group_hierarchy_acyclicity_property(
        group_names in prop::collection::vec(arb_group_name(), 2..8),
        move_attempts in prop::collection::vec((0usize..8usize, 0usize..8usize), 1..5),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create groups with various parent relationships
        let mut group_ids = Vec::new();
        for (i, name) in group_names.iter().enumerate() {
            let id = if i == 0 {
                // First group is always root
                manager.create_group(name.clone()).expect("Should create root group")
            } else {
                // Alternate between root and child groups
                if i % 2 == 0 {
                    manager.create_group(name.clone()).expect("Should create root group")
                } else {
                    // Create as child of a previous group
                    let parent_idx = (i - 1) % group_ids.len();
                    let parent_id = group_ids[parent_idx];
                    manager
                        .create_group_with_parent(name.clone(), parent_id)
                        .expect("Should create child group")
                }
            };
            group_ids.push(id);
        }

        // Verify initial hierarchy is acyclic
        prop_assert!(
            manager.validate_hierarchy(),
            "Initial hierarchy should be acyclic"
        );

        // Attempt various move operations
        for (from_idx, to_idx) in move_attempts {
            if from_idx < group_ids.len() && to_idx < group_ids.len() {
                let from_id = group_ids[from_idx];
                let to_id = if from_idx == to_idx {
                    None // Move to root
                } else {
                    Some(group_ids[to_idx])
                };

                // Attempt the move - it may succeed or fail depending on cycle detection
                let _ = manager.move_group(from_id, to_id);

                // After any move attempt (success or failure), hierarchy must remain acyclic
                prop_assert!(
                    manager.validate_hierarchy(),
                    "Hierarchy must remain acyclic after move attempt from {} to {:?}",
                    from_idx, to_idx
                );
            }
        }

        // Final verification: all parent references must be valid
        for group in manager.list_groups() {
            if let Some(parent_id) = group.parent_id {
                prop_assert!(
                    manager.get_group(parent_id).is_some(),
                    "All parent references must point to existing groups"
                );
            }
        }
    }

    /// **Feature: rustconn-enhancements, Property 6: Group Hierarchy Acyclicity**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// Creating a group with a parent should correctly establish the parent-child
    /// relationship and the group path should reflect the hierarchy.
    #[test]
    fn create_group_with_parent_establishes_hierarchy(
        root_name in arb_group_name(),
        child_name in arb_group_name(),
        grandchild_name in arb_group_name(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create three-level hierarchy
        let root_id = manager.create_group(root_name.clone()).expect("Should create root");
        let child_id = manager
            .create_group_with_parent(child_name.clone(), root_id)
            .expect("Should create child");
        let grandchild_id = manager
            .create_group_with_parent(grandchild_name.clone(), child_id)
            .expect("Should create grandchild");

        // Verify parent relationships
        let root = manager.get_group(root_id).expect("Root should exist");
        let child = manager.get_group(child_id).expect("Child should exist");
        let grandchild = manager.get_group(grandchild_id).expect("Grandchild should exist");

        prop_assert!(root.parent_id.is_none(), "Root should have no parent");
        prop_assert_eq!(child.parent_id, Some(root_id), "Child should have root as parent");
        prop_assert_eq!(grandchild.parent_id, Some(child_id), "Grandchild should have child as parent");

        // Verify group paths
        let root_path = manager.get_group_path(root_id).expect("Root path should exist");
        let child_path = manager.get_group_path(child_id).expect("Child path should exist");
        let grandchild_path = manager.get_group_path(grandchild_id).expect("Grandchild path should exist");

        prop_assert_eq!(&root_path, &root_name, "Root path should be just the root name");
        prop_assert!(
            child_path.contains(&root_name) && child_path.contains(&child_name),
            "Child path should contain both root and child names"
        );
        prop_assert!(
            grandchild_path.contains(&root_name) && grandchild_path.contains(&child_name) && grandchild_path.contains(&grandchild_name),
            "Grandchild path should contain all three names"
        );

        // Hierarchy should be acyclic
        prop_assert!(
            manager.validate_hierarchy(),
            "Hierarchy should be acyclic"
        );
    }

    /// **Feature: rustconn-enhancements, Property 6: Group Hierarchy Acyclicity**
    /// **Validates: Requirements 9.1, 9.2**
    ///
    /// Moving a connection to a group should update the connection's group_id correctly.
    #[test]
    fn move_connection_to_group_updates_group_id(
        conn in arb_connection(),
        group_name in arb_group_name(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create connection and group
        let conn_id = manager.create_connection_from(conn).expect("Should create connection");
        let group_id = manager.create_group(group_name).expect("Should create group");

        // Initially connection should be ungrouped
        let connection = manager.get_connection(conn_id).expect("Connection should exist");
        prop_assert!(connection.group_id.is_none(), "Connection should initially be ungrouped");

        // Move connection to group
        manager
            .move_connection_to_group(conn_id, Some(group_id))
            .expect("Should move connection to group");

        // Verify connection is now in the group
        let connection = manager.get_connection(conn_id).expect("Connection should exist");
        prop_assert_eq!(
            connection.group_id,
            Some(group_id),
            "Connection should be in the group after move"
        );

        // Move connection back to ungrouped
        manager
            .move_connection_to_group(conn_id, None)
            .expect("Should move connection to ungrouped");

        // Verify connection is ungrouped again
        let connection = manager.get_connection(conn_id).expect("Connection should exist");
        prop_assert!(
            connection.group_id.is_none(),
            "Connection should be ungrouped after move to None"
        );
    }
}


// ========== Search Property Tests ==========

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: rustconn, Property 2: Connection Search Correctness**
    /// **Validates: Requirements 1.5, 1.6**
    ///
    /// For any set of connections and search query, all returned results must
    /// match the query against at least one of: name, host, tags, or group path.
    #[test]
    fn search_results_match_query(
        connections in prop::collection::vec(arb_connection(), 1..20),
        query in "[a-z]{1,5}",
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Add all connections
        for conn in &connections {
            let _ = manager.create_connection_from(conn.clone());
        }

        // Perform search
        let results = manager.search(&query);
        let query_lower = query.to_lowercase();

        // Verify all results match the query
        for result in results {
            let name_matches = result.name.to_lowercase().contains(&query_lower);
            let host_matches = result.host.to_lowercase().contains(&query_lower);
            let tags_match = result.tags.iter().any(|t| t.to_lowercase().contains(&query_lower));
            let group_matches = result.group_id
                .and_then(|gid| manager.get_group_path(gid))
                .map(|path| path.to_lowercase().contains(&query_lower))
                .unwrap_or(false);

            prop_assert!(
                name_matches || host_matches || tags_match || group_matches,
                "Search result should match query in name, host, tags, or group path. \
                 Query: '{}', Name: '{}', Host: '{}', Tags: {:?}",
                query, result.name, result.host, result.tags
            );
        }
    }

    /// **Feature: rustconn, Property 2: Connection Search Correctness**
    /// **Validates: Requirements 1.5, 1.6**
    ///
    /// No connection matching the query should be excluded from results.
    #[test]
    fn search_includes_all_matches(
        name in arb_name(),
        host in arb_host(),
        port in arb_port(),
        protocol_config in arb_protocol_config(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create a connection
        let id = manager
            .create_connection(name.clone(), host.clone(), port, protocol_config)
            .expect("Should create connection");

        // Search by exact name (should find it)
        let results = manager.search(&name);
        prop_assert!(
            results.iter().any(|c| c.id == id),
            "Search by exact name should find the connection"
        );

        // Search by partial name (first 3 chars if long enough)
        if name.len() >= 3 {
            let partial = &name[0..3];
            let results = manager.search(partial);
            prop_assert!(
                results.iter().any(|c| c.id == id),
                "Search by partial name should find the connection"
            );
        }

        // Search by host
        let results = manager.search(&host);
        prop_assert!(
            results.iter().any(|c| c.id == id),
            "Search by host should find the connection"
        );
    }

    /// **Feature: rustconn, Property 2: Connection Search Correctness**
    /// **Validates: Requirements 1.6**
    ///
    /// Tag-based filtering should return only connections with the specified tag.
    #[test]
    fn filter_by_tag_returns_only_tagged_connections(
        conn1 in arb_connection(),
        conn2 in arb_connection(),
        tag in "[a-z]{3,10}",
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create first connection with the tag
        let mut conn1_with_tag = conn1.clone();
        conn1_with_tag.tags = vec![tag.clone()];
        let id1 = manager
            .create_connection_from(conn1_with_tag)
            .expect("Should create first connection");

        // Create second connection without the tag
        let mut conn2_without_tag = conn2.clone();
        conn2_without_tag.tags = vec!["other_tag".to_string()];
        let id2 = manager
            .create_connection_from(conn2_without_tag)
            .expect("Should create second connection");

        // Filter by tag
        let results = manager.filter_by_tag(&tag);

        // Should include first connection
        prop_assert!(
            results.iter().any(|c| c.id == id1),
            "Filter should include connection with the tag"
        );

        // Should not include second connection
        prop_assert!(
            !results.iter().any(|c| c.id == id2),
            "Filter should not include connection without the tag"
        );
    }

    /// **Feature: rustconn, Property 2: Connection Search Correctness**
    /// **Validates: Requirements 1.6**
    ///
    /// Filtering by multiple tags should return only connections with ALL tags.
    #[test]
    fn filter_by_multiple_tags_uses_and_logic(
        conn in arb_connection(),
        tag1 in "[a-z]{3,8}",
        tag2 in "[a-z]{3,8}",
    ) {
        // Skip if tags are the same
        prop_assume!(tag1 != tag2);

        let (mut manager, _temp) = create_test_manager();

        // Create connection with both tags
        let mut conn_both = conn.clone();
        conn_both.tags = vec![tag1.clone(), tag2.clone()];
        let id_both = manager
            .create_connection_from(conn_both)
            .expect("Should create connection with both tags");

        // Create connection with only first tag
        let mut conn_one = Connection::new(
            "Single Tag".to_string(),
            "single.example.com".to_string(),
            22,
            ProtocolConfig::Ssh(SshConfig::default()),
        );
        conn_one.tags = vec![tag1.clone()];
        let id_one = manager
            .create_connection_from(conn_one)
            .expect("Should create connection with one tag");

        // Filter by both tags
        let results = manager.filter_by_tags(&[tag1.clone(), tag2.clone()]);

        // Should include connection with both tags
        prop_assert!(
            results.iter().any(|c| c.id == id_both),
            "Filter should include connection with both tags"
        );

        // Should not include connection with only one tag
        prop_assert!(
            !results.iter().any(|c| c.id == id_one),
            "Filter should not include connection with only one tag"
        );
    }

    /// **Feature: rustconn, Property 2: Connection Search Correctness**
    /// **Validates: Requirements 1.5**
    ///
    /// Search should be case-insensitive.
    #[test]
    fn search_is_case_insensitive(
        name in "[A-Z][a-z]{2,10}",
        host in arb_host(),
        port in arb_port(),
        protocol_config in arb_protocol_config(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create connection with mixed case name
        let id = manager
            .create_connection(name.clone(), host, port, protocol_config)
            .expect("Should create connection");

        // Search with lowercase
        let results_lower = manager.search(&name.to_lowercase());
        prop_assert!(
            results_lower.iter().any(|c| c.id == id),
            "Lowercase search should find mixed case name"
        );

        // Search with uppercase
        let results_upper = manager.search(&name.to_uppercase());
        prop_assert!(
            results_upper.iter().any(|c| c.id == id),
            "Uppercase search should find mixed case name"
        );
    }

    /// **Feature: rustconn, Property 2: Connection Search Correctness**
    /// **Validates: Requirements 1.5**
    ///
    /// Search by group path should find connections in that group.
    #[test]
    fn search_by_group_path_finds_connections(
        group_name in arb_group_name(),
        conn_name in arb_name(),
        host in arb_host(),
        port in arb_port(),
        protocol_config in arb_protocol_config(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create group
        let group_id = manager.create_group(group_name.clone()).expect("Should create group");

        // Create connection in group
        let conn_id = manager
            .create_connection(conn_name, host, port, protocol_config)
            .expect("Should create connection");

        manager
            .move_connection_to_group(conn_id, Some(group_id))
            .expect("Should move connection to group");

        // Search by group name
        let results = manager.search(&group_name);

        prop_assert!(
            results.iter().any(|c| c.id == conn_id),
            "Search by group name should find connection in that group"
        );
    }

    /// **Feature: rustconn-enhancements, Property 2: Bulk Delete Completeness**
    /// **Validates: Requirements 3.2, 3.3, 3.4**
    ///
    /// For any set of selected connections, after bulk delete completes successfully,
    /// none of the deleted connection IDs should exist in the connection manager,
    /// and the count of deleted items should equal the original selection count minus any failures.
    #[test]
    fn bulk_delete_removes_all_selected_connections(
        connections in prop::collection::vec(arb_connection(), 2..10),
        delete_indices in prop::collection::hash_set(0usize..10usize, 1..5),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create all connections
        let mut created_ids = Vec::new();
        for conn in &connections {
            let id = manager
                .create_connection_from(conn.clone())
                .expect("Should create connection");
            created_ids.push(id);
        }

        let initial_count = manager.connection_count();
        prop_assert_eq!(initial_count, connections.len(), "Should have all connections created");

        // Select connections to delete (filter to valid indices)
        let ids_to_delete: Vec<Uuid> = delete_indices
            .iter()
            .filter(|&&idx| idx < created_ids.len())
            .map(|&idx| created_ids[idx])
            .collect();

        let delete_count = ids_to_delete.len();

        // Perform bulk delete
        let mut success_count = 0;
        let mut failures: Vec<(Uuid, String)> = Vec::new();

        for id in &ids_to_delete {
            match manager.delete_connection(*id) {
                Ok(()) => success_count += 1,
                Err(e) => failures.push((*id, e.to_string())),
            }
        }

        // Property 1: Success count + failure count should equal total delete attempts
        prop_assert_eq!(
            success_count + failures.len(),
            delete_count,
            "Success + failures should equal total delete attempts"
        );

        // Property 2: None of the successfully deleted IDs should exist
        for id in &ids_to_delete {
            if !failures.iter().any(|(fid, _)| fid == id) {
                prop_assert!(
                    manager.get_connection(*id).is_none(),
                    "Deleted connection {:?} should not exist in manager",
                    id
                );
            }
        }

        // Property 3: Connection count should be reduced by success_count
        let final_count = manager.connection_count();
        prop_assert_eq!(
            final_count,
            initial_count - success_count,
            "Connection count should be reduced by number of successful deletions"
        );

        // Property 4: Non-deleted connections should still exist
        for (idx, id) in created_ids.iter().enumerate() {
            if !ids_to_delete.contains(id) {
                prop_assert!(
                    manager.get_connection(*id).is_some(),
                    "Non-deleted connection at index {} should still exist",
                    idx
                );
            }
        }
    }

    /// **Feature: rustconn-enhancements, Property 2: Bulk Delete Completeness**
    /// **Validates: Requirements 3.2, 3.3, 3.4**
    ///
    /// Bulk delete should continue processing remaining items even if some deletions fail.
    /// This tests the "continue on failure" behavior.
    #[test]
    fn bulk_delete_continues_on_failure(
        connections in prop::collection::vec(arb_connection(), 3..8),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create connections
        let mut created_ids = Vec::new();
        for conn in &connections {
            let id = manager
                .create_connection_from(conn.clone())
                .expect("Should create connection");
            created_ids.push(id);
        }

        // Create a list with some valid IDs and some invalid (non-existent) IDs
        let mut ids_to_delete = Vec::new();
        
        // Add first valid ID
        if !created_ids.is_empty() {
            ids_to_delete.push(created_ids[0]);
        }
        
        // Add a non-existent ID (will fail)
        ids_to_delete.push(Uuid::new_v4());
        
        // Add second valid ID if available
        if created_ids.len() > 1 {
            ids_to_delete.push(created_ids[1]);
        }

        let initial_count = manager.connection_count();

        // Perform bulk delete
        let mut success_count = 0;
        let mut failure_count = 0;

        for id in &ids_to_delete {
            match manager.delete_connection(*id) {
                Ok(()) => success_count += 1,
                Err(_) => failure_count += 1,
            }
        }

        // Should have at least one failure (the non-existent ID)
        prop_assert!(
            failure_count >= 1,
            "Should have at least one failure for non-existent ID"
        );

        // Should have successfully deleted the valid IDs
        let expected_successes = ids_to_delete
            .iter()
            .filter(|id| created_ids.contains(id))
            .count();
        
        prop_assert_eq!(
            success_count,
            expected_successes,
            "Should successfully delete all valid IDs despite failures"
        );

        // Verify the valid IDs were actually deleted
        for id in &ids_to_delete {
            if created_ids.contains(id) {
                prop_assert!(
                    manager.get_connection(*id).is_none(),
                    "Valid ID {:?} should be deleted",
                    id
                );
            }
        }

        // Verify remaining connections are intact
        let final_count = manager.connection_count();
        prop_assert_eq!(
            final_count,
            initial_count - success_count,
            "Remaining connection count should be correct"
        );
    }

    /// **Feature: rustconn-enhancements, Property 2: Bulk Delete Completeness**
    /// **Validates: Requirements 3.2, 3.3, 3.4**
    ///
    /// Bulk delete of groups should also work correctly, moving connections to ungrouped.
    #[test]
    fn bulk_delete_groups_moves_connections_to_ungrouped(
        group_names in prop::collection::vec(arb_group_name(), 2..5),
        conn in arb_connection(),
    ) {
        let (mut manager, _temp) = create_test_manager();

        // Create groups
        let mut group_ids = Vec::new();
        for name in &group_names {
            let id = manager.create_group(name.clone()).expect("Should create group");
            group_ids.push(id);
        }

        // Create a connection in the first group
        let conn_id = manager
            .create_connection_from(conn)
            .expect("Should create connection");
        
        manager
            .move_connection_to_group(conn_id, Some(group_ids[0]))
            .expect("Should move connection to group");

        // Verify connection is in group
        let conn_before = manager.get_connection(conn_id).expect("Connection should exist");
        prop_assert_eq!(
            conn_before.group_id,
            Some(group_ids[0]),
            "Connection should be in first group"
        );

        // Delete the first group
        manager
            .delete_group(group_ids[0])
            .expect("Should delete group");

        // Verify connection is now ungrouped
        let conn_after = manager.get_connection(conn_id).expect("Connection should still exist");
        prop_assert!(
            conn_after.group_id.is_none(),
            "Connection should be ungrouped after group deletion"
        );

        // Verify the group is gone
        prop_assert!(
            manager.get_group(group_ids[0]).is_none(),
            "Deleted group should not exist"
        );

        // Verify other groups still exist
        for &gid in group_ids.iter().skip(1) {
            prop_assert!(
                manager.get_group(gid).is_some(),
                "Other groups should still exist"
            );
        }
    }
}
