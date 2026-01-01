//! Property tests for session restore models

use proptest::prelude::*;
use rustconn_core::config::{SavedSession, SessionRestoreSettings};
use uuid::Uuid;

/// Strategy for generating valid protocol names
fn protocol_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("ssh".to_string()),
        Just("rdp".to_string()),
        Just("vnc".to_string()),
        Just("spice".to_string()),
    ]
}

/// Strategy for generating valid port numbers
fn port_strategy() -> impl Strategy<Value = u16> {
    prop_oneof![
        Just(22_u16),
        Just(3389_u16),
        Just(5900_u16),
        Just(5930_u16),
        1..=65535_u16,
    ]
}

/// Strategy for generating valid hostnames
fn host_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("localhost".to_string()),
        Just("127.0.0.1".to_string()),
        "[a-z]{3,10}\\.[a-z]{2,4}",
    ]
}

/// Strategy for generating connection names
fn name_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 _-]{2,20}"
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// SavedSession creation produces valid session
    #[test]
    fn prop_saved_session_valid(
        name in name_strategy(),
        host in host_strategy(),
        port in port_strategy(),
        protocol in protocol_strategy(),
    ) {
        let session = SavedSession {
            connection_id: Uuid::new_v4(),
            connection_name: name.clone(),
            protocol: protocol.clone(),
            host: host.clone(),
            port,
            saved_at: chrono::Utc::now(),
        };

        prop_assert_eq!(&session.connection_name, &name);
        prop_assert_eq!(&session.host, &host);
        prop_assert_eq!(session.port, port);
        prop_assert_eq!(&session.protocol, &protocol);
    }

    /// SavedSession serialization round-trip preserves data
    #[test]
    fn prop_saved_session_serialization_roundtrip(
        name in name_strategy(),
        host in host_strategy(),
        port in port_strategy(),
        protocol in protocol_strategy(),
    ) {
        let session = SavedSession {
            connection_id: Uuid::new_v4(),
            connection_name: name,
            protocol,
            host,
            port,
            saved_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&session).unwrap();
        let restored: SavedSession = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(session.connection_id, restored.connection_id);
        prop_assert_eq!(session.connection_name, restored.connection_name);
        prop_assert_eq!(session.host, restored.host);
        prop_assert_eq!(session.port, restored.port);
        prop_assert_eq!(session.protocol, restored.protocol);
    }

    /// SessionRestoreSettings with various configurations
    #[test]
    fn prop_session_restore_settings_valid(
        enabled in proptest::bool::ANY,
        prompt_on_restore in proptest::bool::ANY,
        max_age_hours in 0..168_u32,
    ) {
        let settings = SessionRestoreSettings {
            enabled,
            prompt_on_restore,
            max_age_hours,
            saved_sessions: Vec::new(),
        };

        prop_assert_eq!(settings.enabled, enabled);
        prop_assert_eq!(settings.prompt_on_restore, prompt_on_restore);
        prop_assert_eq!(settings.max_age_hours, max_age_hours);
        prop_assert!(settings.saved_sessions.is_empty());
    }

    /// SessionRestoreSettings serialization round-trip
    #[test]
    fn prop_session_restore_settings_serialization_roundtrip(
        enabled in proptest::bool::ANY,
        prompt_on_restore in proptest::bool::ANY,
        max_age_hours in 0..168_u32,
    ) {
        let settings = SessionRestoreSettings {
            enabled,
            prompt_on_restore,
            max_age_hours,
            saved_sessions: Vec::new(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let restored: SessionRestoreSettings = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(settings.enabled, restored.enabled);
        prop_assert_eq!(settings.prompt_on_restore, restored.prompt_on_restore);
        prop_assert_eq!(settings.max_age_hours, restored.max_age_hours);
    }

    /// SessionRestoreSettings with sessions serialization
    #[test]
    fn prop_session_restore_with_sessions_roundtrip(
        session_count in 0..5_usize,
    ) {
        let sessions: Vec<SavedSession> = (0..session_count)
            .map(|i| SavedSession {
                connection_id: Uuid::new_v4(),
                connection_name: format!("Session {i}"),
                protocol: "ssh".to_string(),
                host: format!("host{i}.example.com"),
                port: 22,
                saved_at: chrono::Utc::now(),
            })
            .collect();

        let settings = SessionRestoreSettings {
            enabled: true,
            prompt_on_restore: false,
            max_age_hours: 24,
            saved_sessions: sessions.clone(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let restored: SessionRestoreSettings = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(settings.saved_sessions.len(), restored.saved_sessions.len());
        for (orig, rest) in settings.saved_sessions.iter().zip(restored.saved_sessions.iter()) {
            prop_assert_eq!(orig.connection_id, rest.connection_id);
            prop_assert_eq!(&orig.connection_name, &rest.connection_name);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_session_restore_settings_default() {
        let settings = SessionRestoreSettings::default();
        assert!(!settings.enabled);
        assert!(settings.prompt_on_restore);
        assert_eq!(settings.max_age_hours, 24);
        assert!(settings.saved_sessions.is_empty());
    }

    #[test]
    fn test_saved_session_equality() {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let session1 = SavedSession {
            connection_id: id,
            connection_name: "Test".to_string(),
            protocol: "ssh".to_string(),
            host: "localhost".to_string(),
            port: 22,
            saved_at: now,
        };

        let session2 = SavedSession {
            connection_id: id,
            connection_name: "Test".to_string(),
            protocol: "ssh".to_string(),
            host: "localhost".to_string(),
            port: 22,
            saved_at: now,
        };

        assert_eq!(session1, session2);
    }
}
