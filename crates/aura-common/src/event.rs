//! Agent-agnostic event types
//!
//! These events are emitted by adapters (Claude Code, PTY wrapper, etc.)
//! and consumed by the Aura daemon. The daemon doesn't know about
//! agent-specific details.

use serde::{Deserialize, Serialize};

/// Type of AI code agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode,
    GeminiCli,
    Codex,
    OpenCode,
    Custom(String),
}

/// Agent-agnostic event
///
/// Adapters convert agent-specific events (e.g., Claude Code hooks)
/// into these generic events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// New session started
    SessionStarted {
        session_id: String,
        cwd: String,
        agent: AgentType,
    },
    /// Activity detected (health check)
    Activity { session_id: String },
    /// Tool execution started
    ToolStarted {
        session_id: String,
        tool_id: String,
        tool_name: String,
        tool_label: Option<String>,
    },
    /// Tool execution completed
    ToolCompleted { session_id: String, tool_id: String },
    /// Agent needs user attention (e.g., permission request)
    NeedsAttention {
        session_id: String,
        message: Option<String>,
    },
    /// Context compacting in progress
    Compacting { session_id: String },
    /// Agent is idle, waiting for user input
    Idle { session_id: String },
    /// Session ended
    SessionEnded { session_id: String },
}

impl AgentEvent {
    /// Get session_id from any event
    pub fn session_id(&self) -> &str {
        match self {
            Self::SessionStarted { session_id, .. } => session_id,
            Self::Activity { session_id } => session_id,
            Self::ToolStarted { session_id, .. } => session_id,
            Self::ToolCompleted { session_id, .. } => session_id,
            Self::NeedsAttention { session_id, .. } => session_id,
            Self::Compacting { session_id } => session_id,
            Self::Idle { session_id } => session_id,
            Self::SessionEnded { session_id } => session_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_extraction() {
        let events = [
            AgentEvent::SessionStarted {
                session_id: "s1".into(),
                cwd: "/tmp".into(),
                agent: AgentType::ClaudeCode,
            },
            AgentEvent::Activity {
                session_id: "s2".into(),
            },
            AgentEvent::ToolStarted {
                session_id: "s3".into(),
                tool_id: "t1".into(),
                tool_name: "Read".into(),
                tool_label: None,
            },
            AgentEvent::ToolCompleted {
                session_id: "s4".into(),
                tool_id: "t1".into(),
            },
            AgentEvent::NeedsAttention {
                session_id: "s5".into(),
                message: Some("Permission needed".into()),
            },
            AgentEvent::Compacting {
                session_id: "s6".into(),
            },
            AgentEvent::Idle {
                session_id: "s7".into(),
            },
            AgentEvent::SessionEnded {
                session_id: "s8".into(),
            },
        ];

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.session_id(), format!("s{}", i + 1));
        }
    }

    #[test]
    fn agent_event_serialization() {
        let event = AgentEvent::ToolStarted {
            session_id: "abc123".into(),
            tool_id: "toolu_01".into(),
            tool_name: "Read".into(),
            tool_label: Some("config.rs".into()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"tool_started\""));
        assert!(json.contains("\"session_id\":\"abc123\""));

        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), "abc123");
    }

    #[test]
    fn agent_type_serialization() {
        assert_eq!(
            serde_json::to_string(&AgentType::ClaudeCode).unwrap(),
            "\"claude_code\""
        );
        assert_eq!(
            serde_json::to_string(&AgentType::Custom("my-agent".into())).unwrap(),
            "{\"custom\":\"my-agent\"}"
        );
    }
}
