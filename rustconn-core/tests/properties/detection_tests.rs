//! Property-based tests for client detection
//!
//! These tests validate the correctness properties for protocol client detection
//! as defined in the design document.
//!
//! **Feature: rustconn-bugfixes, Property 9: Client Detection**
//! **Validates: Requirements 7.2, 7.3, 7.4**

use proptest::prelude::*;
use rustconn_core::{
    detect_rdp_client, detect_ssh_client, detect_vnc_client, ClientDetectionResult, ClientInfo,
};

// ============================================================================
// Property Tests for Client Detection
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: rustconn-bugfixes, Property 9: Client Detection**
    // **Validates: Requirements 7.2, 7.3, 7.4**
    //
    // For any installed client binary, detection SHALL return installed=true
    // with version string.

    /// Property: ClientInfo structure is consistent
    /// If installed is true, path must be Some
    /// If installed is false, install_hint should be Some
    #[test]
    fn prop_client_info_consistency(_seed in any::<u64>()) {
        // Test SSH client detection
        let ssh_info = detect_ssh_client();
        validate_client_info_consistency(&ssh_info);

        // Test RDP client detection
        let rdp_info = detect_rdp_client();
        validate_client_info_consistency(&rdp_info);

        // Test VNC client detection
        let vnc_info = detect_vnc_client();
        validate_client_info_consistency(&vnc_info);
    }

    /// Property: Detection results are deterministic
    /// Multiple calls should return the same result
    #[test]
    fn prop_detection_is_deterministic(_seed in any::<u64>()) {
        // SSH detection should be deterministic
        let ssh1 = detect_ssh_client();
        let ssh2 = detect_ssh_client();
        prop_assert_eq!(ssh1.installed, ssh2.installed);
        prop_assert_eq!(ssh1.name, ssh2.name);
        prop_assert_eq!(ssh1.path, ssh2.path);

        // RDP detection should be deterministic
        let rdp1 = detect_rdp_client();
        let rdp2 = detect_rdp_client();
        prop_assert_eq!(rdp1.installed, rdp2.installed);
        prop_assert_eq!(rdp1.name, rdp2.name);
        prop_assert_eq!(rdp1.path, rdp2.path);

        // VNC detection should be deterministic
        let vnc1 = detect_vnc_client();
        let vnc2 = detect_vnc_client();
        prop_assert_eq!(vnc1.installed, vnc2.installed);
        prop_assert_eq!(vnc1.name, vnc2.name);
        prop_assert_eq!(vnc1.path, vnc2.path);
    }

    /// Property: ClientDetectionResult contains all three protocols
    #[test]
    fn prop_detection_result_complete(_seed in any::<u64>()) {
        let result = ClientDetectionResult::detect_all();

        // All three clients should have non-empty names
        prop_assert!(!result.ssh.name.is_empty(), "SSH client name should not be empty");
        prop_assert!(!result.rdp.name.is_empty(), "RDP client name should not be empty");
        prop_assert!(!result.vnc.name.is_empty(), "VNC client name should not be empty");
    }

    /// Property: Installed clients have valid paths
    #[test]
    fn prop_installed_clients_have_valid_paths(_seed in any::<u64>()) {
        let ssh_info = detect_ssh_client();
        if ssh_info.installed {
            prop_assert!(ssh_info.path.is_some(), "Installed SSH client must have path");
            if let Some(path) = &ssh_info.path {
                prop_assert!(path.exists(), "SSH client path must exist: {:?}", path);
            }
        }

        let rdp_info = detect_rdp_client();
        if rdp_info.installed {
            prop_assert!(rdp_info.path.is_some(), "Installed RDP client must have path");
            if let Some(path) = &rdp_info.path {
                prop_assert!(path.exists(), "RDP client path must exist: {:?}", path);
            }
        }

        let vnc_info = detect_vnc_client();
        if vnc_info.installed {
            prop_assert!(vnc_info.path.is_some(), "Installed VNC client must have path");
            if let Some(path) = &vnc_info.path {
                prop_assert!(path.exists(), "VNC client path must exist: {:?}", path);
            }
        }
    }

    /// Property: Not installed clients have installation hints
    #[test]
    fn prop_not_installed_clients_have_hints(_seed in any::<u64>()) {
        let ssh_info = detect_ssh_client();
        if !ssh_info.installed {
            prop_assert!(
                ssh_info.install_hint.is_some(),
                "Not installed SSH client must have install hint"
            );
        }

        let rdp_info = detect_rdp_client();
        if !rdp_info.installed {
            prop_assert!(
                rdp_info.install_hint.is_some(),
                "Not installed RDP client must have install hint"
            );
        }

        let vnc_info = detect_vnc_client();
        if !vnc_info.installed {
            prop_assert!(
                vnc_info.install_hint.is_some(),
                "Not installed VNC client must have install hint"
            );
        }
    }
}

/// Helper function to validate ClientInfo consistency
fn validate_client_info_consistency(info: &ClientInfo) {
    // Name should never be empty
    assert!(!info.name.is_empty(), "Client name should not be empty");

    if info.installed {
        // Installed clients must have a path
        assert!(
            info.path.is_some(),
            "Installed client '{}' must have a path",
            info.name
        );
        // Install hint is not needed for installed clients
    } else {
        // Not installed clients should have an install hint
        assert!(
            info.install_hint.is_some(),
            "Not installed client '{}' should have an install hint",
            info.name
        );
        // Path should be None for not installed clients
        assert!(
            info.path.is_none(),
            "Not installed client '{}' should not have a path",
            info.name
        );
    }
}

// ============================================================================
// Unit Tests for Client Detection
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_client_info_installed_constructor() {
        use std::path::PathBuf;

        let info = ClientInfo::installed(
            "Test",
            PathBuf::from("/usr/bin/test"),
            Some("1.0".to_string()),
        );
        assert!(info.installed);
        assert_eq!(info.name, "Test");
        assert_eq!(info.path, Some(PathBuf::from("/usr/bin/test")));
        assert_eq!(info.version, Some("1.0".to_string()));
        assert!(info.install_hint.is_none());
    }

    #[test]
    fn test_client_info_not_installed_constructor() {
        let info = ClientInfo::not_installed("Test", "Install with: apt install test");
        assert!(!info.installed);
        assert_eq!(info.name, "Test");
        assert!(info.path.is_none());
        assert!(info.version.is_none());
        assert_eq!(
            info.install_hint,
            Some("Install with: apt install test".to_string())
        );
    }

    #[test]
    fn test_detect_all_returns_three_clients() {
        let result = ClientDetectionResult::detect_all();

        // Should have all three protocol clients
        assert!(!result.ssh.name.is_empty());
        assert!(!result.rdp.name.is_empty());
        assert!(!result.vnc.name.is_empty());
    }

    #[test]
    fn test_ssh_detection_returns_valid_info() {
        let info = detect_ssh_client();

        // Name should be set
        assert!(!info.name.is_empty());

        // Consistency check
        if info.installed {
            assert!(info.path.is_some());
        } else {
            assert!(info.install_hint.is_some());
        }
    }

    #[test]
    fn test_rdp_detection_returns_valid_info() {
        let info = detect_rdp_client();

        // Name should be set
        assert!(!info.name.is_empty());

        // Consistency check
        if info.installed {
            assert!(info.path.is_some());
        } else {
            assert!(info.install_hint.is_some());
        }
    }

    #[test]
    fn test_vnc_detection_returns_valid_info() {
        let info = detect_vnc_client();

        // Name should be set
        assert!(!info.name.is_empty());

        // Consistency check
        if info.installed {
            assert!(info.path.is_some());
        } else {
            assert!(info.install_hint.is_some());
        }
    }
}
