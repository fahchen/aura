//! IPC message types for Unix socket communication
//!
//! Used by aura-hook (Claude Code hooks → socket) and the daemon's socket server.

use serde::{Deserialize, Serialize};

use crate::{AgentEvent, AgentType};

/// Message sent over the Unix socket (newline-delimited JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum IpcMessage {
    SessionStarted {
        session_id: String,
        cwd: String,
        agent: AgentType,
    },
    Activity {
        session_id: String,
        cwd: String,
    },
    ToolStarted {
        session_id: String,
        cwd: String,
        tool_id: String,
        tool_name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_label: Option<String>,
    },
    ToolCompleted {
        session_id: String,
        cwd: String,
        tool_id: String,
    },
    NeedsAttention {
        session_id: String,
        cwd: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    WaitingForInput {
        session_id: String,
        cwd: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Compacting {
        session_id: String,
        cwd: String,
    },
    Idle {
        session_id: String,
        cwd: String,
    },
    SessionEnded {
        session_id: String,
    },
    SessionNameUpdated {
        session_id: String,
        name: String,
    },
}

impl From<IpcMessage> for AgentEvent {
    fn from(msg: IpcMessage) -> Self {
        match msg {
            IpcMessage::SessionStarted {
                session_id,
                cwd,
                agent,
            } => AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent,
            },
            IpcMessage::Activity { session_id, cwd } => {
                AgentEvent::Activity { session_id, cwd }
            }
            IpcMessage::ToolStarted {
                session_id,
                cwd,
                tool_id,
                tool_name,
                tool_label,
            } => AgentEvent::ToolStarted {
                session_id,
                cwd,
                tool_id,
                tool_name,
                tool_label,
            },
            IpcMessage::ToolCompleted {
                session_id,
                cwd,
                tool_id,
            } => AgentEvent::ToolCompleted {
                session_id,
                cwd,
                tool_id,
            },
            IpcMessage::NeedsAttention {
                session_id,
                cwd,
                message,
            } => AgentEvent::NeedsAttention {
                session_id,
                cwd,
                message,
            },
            IpcMessage::WaitingForInput {
                session_id,
                cwd,
                message,
            } => AgentEvent::WaitingForInput {
                session_id,
                cwd,
                message,
            },
            IpcMessage::Compacting { session_id, cwd } => {
                AgentEvent::Compacting { session_id, cwd }
            }
            IpcMessage::Idle { session_id, cwd } => AgentEvent::Idle { session_id, cwd },
            IpcMessage::SessionEnded { session_id } => {
                AgentEvent::SessionEnded { session_id }
            }
            IpcMessage::SessionNameUpdated { session_id, name } => {
                AgentEvent::SessionNameUpdated { session_id, name }
            }
        }
    }
}

/// Default socket path for the daemon.
pub fn socket_path() -> std::path::PathBuf {
    std::env::temp_dir().join("aura.sock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_message_roundtrip() {
        let msg = IpcMessage::ToolStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            tool_id: "t1".into(),
            tool_name: "Bash".into(),
            tool_label: Some("npm test".into()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"event\":\"tool_started\""));

        let parsed: IpcMessage = serde_json::from_str(&json).unwrap();
        let event: AgentEvent = parsed.into();
        assert_eq!(event.session_id(), "s1");
    }

    #[test]
    fn ipc_message_session_started() {
        let json = r#"{"event":"session_started","session_id":"abc","cwd":"/path","agent":"claude_code"}"#;
        let msg: IpcMessage = serde_json::from_str(json).unwrap();
        let event: AgentEvent = msg.into();
        assert_eq!(event.session_id(), "abc");
        assert_eq!(event.cwd(), "/path");
    }

    #[test]
    fn ipc_message_session_ended() {
        let json = r#"{"event":"session_ended","session_id":"abc"}"#;
        let msg: IpcMessage = serde_json::from_str(json).unwrap();
        let event: AgentEvent = msg.into();
        assert_eq!(event.session_id(), "abc");
    }

    #[test]
    fn ipc_message_needs_attention() {
        let json = r#"{"event":"needs_attention","session_id":"abc","cwd":"/path","message":"Permission needed"}"#;
        let msg: IpcMessage = serde_json::from_str(json).unwrap();
        let event: AgentEvent = msg.into();
        assert_eq!(event.session_id(), "abc");
    }

    #[test]
    fn ipc_message_all_variants() {
        let messages = vec![
            r#"{"event":"session_started","session_id":"s1","cwd":"/tmp","agent":"claude_code"}"#,
            r#"{"event":"activity","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"event":"tool_started","session_id":"s1","cwd":"/tmp","tool_id":"t1","tool_name":"Read"}"#,
            r#"{"event":"tool_completed","session_id":"s1","cwd":"/tmp","tool_id":"t1"}"#,
            r#"{"event":"needs_attention","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"event":"waiting_for_input","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"event":"compacting","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"event":"idle","session_id":"s1","cwd":"/tmp"}"#,
            r#"{"event":"session_ended","session_id":"s1"}"#,
            r#"{"event":"session_name_updated","session_id":"s1","name":"fix login"}"#,
        ];

        for json in messages {
            let msg: IpcMessage = serde_json::from_str(json).unwrap();
            let _event: AgentEvent = msg.into();
        }
    }
}
