//! IPC utilities for Unix socket communication
//!
//! The hook subprocess and daemon server both live in the same binary,
//! so `AgentEvent` (from `event.rs`) is sent directly on the wire as
//! newline-delimited JSON. This module only provides the shared socket path.

/// Default socket path for the daemon.
pub fn socket_path() -> std::path::PathBuf {
    std::env::temp_dir().join("aura.sock")
}

#[cfg(test)]
mod tests {
    use crate::AgentEvent;

    #[test]
    fn agent_event_ipc_roundtrip() {
        let event = AgentEvent::ToolStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            tool_id: "t1".into(),
            tool_name: "Bash".into(),
            tool_label: Some("npm test".into()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"tool_started\""));

        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), "s1");
    }

    #[test]
    fn agent_event_ipc_session_started() {
        let json = r#"{"type":"session_started","session_id":"abc","cwd":"/path","agent":"claude_code"}"#;
        let event: AgentEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.session_id(), "abc");
        assert_eq!(event.cwd(), "/path");
    }

    #[test]
    fn agent_event_ipc_session_ended() {
        let json = r#"{"type":"session_ended","session_id":"abc"}"#;
        let event: AgentEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.session_id(), "abc");
    }

    #[test]
    fn agent_event_ipc_needs_attention() {
        let json = r#"{"type":"needs_attention","session_id":"abc","cwd":"/path","message":"Permission needed"}"#;
        let event: AgentEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.session_id(), "abc");
    }

    #[test]
    fn agent_event_ipc_all_variants() {
        let messages = vec![
            r#"{"type":"session_started","session_id":"s1","cwd":"/tmp","agent":"claude_code"}"#,
            r#"{"type":"activity","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"type":"tool_started","session_id":"s1","cwd":"/tmp","tool_id":"t1","tool_name":"Read"}"#,
            r#"{"type":"tool_completed","session_id":"s1","cwd":"/tmp","tool_id":"t1"}"#,
            r#"{"type":"needs_attention","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"type":"waiting_for_input","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"type":"compacting","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"type":"idle","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"type":"session_ended","session_id":"s1"}"#,
            r#"{"type":"session_name_updated","session_id":"s1","name":"fix login"}"#,
        ];

        for json in messages {
            let _event: AgentEvent = serde_json::from_str(json).unwrap();
        }
    }
}
