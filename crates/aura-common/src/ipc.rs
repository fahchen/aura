//! IPC message protocol between adapters and daemon

use crate::{AgentEvent, SessionState};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Socket name for IPC communication
pub const SOCKET_NAME: &str = "aura.sock";

/// Get the socket path for IPC communication
///
/// Uses XDG_RUNTIME_DIR if available, falls back to /tmp
pub fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    runtime_dir.join(SOCKET_NAME)
}

/// Message from adapter to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg", rename_all = "snake_case")]
pub enum IpcMessage {
    /// Agent event (generic, from any adapter)
    Event(AgentEvent),
    /// Ping to check if daemon is alive
    Ping,
}

/// Response from daemon to adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg", rename_all = "snake_case")]
pub enum IpcResponse {
    /// Acknowledgment
    Ok,
    /// Pong response to ping
    Pong,
    /// Error message
    Error { message: String },
}

/// Session information for IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub cwd: String,
    pub state: SessionState,
    pub running_tools: Vec<RunningTool>,
}

/// A currently running tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningTool {
    pub tool_id: String,
    pub tool_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentType;

    #[test]
    fn socket_path_ends_with_socket_name() {
        let path = socket_path();
        assert!(path.ends_with(SOCKET_NAME));
    }

    #[test]
    fn ipc_message_event_serialization() {
        let event = AgentEvent::SessionStarted {
            session_id: "abc123".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        };
        let msg = IpcMessage::Event(event);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"msg\":\"event\""));
        assert!(json.contains("\"session_id\":\"abc123\""));

        let parsed: IpcMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            IpcMessage::Event(e) => assert_eq!(e.session_id(), "abc123"),
            _ => panic!("Expected Event"),
        }
    }

    #[test]
    fn ipc_message_ping_serialization() {
        let msg = IpcMessage::Ping;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, "{\"msg\":\"ping\"}");

        let parsed: IpcMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcMessage::Ping));
    }

    #[test]
    fn ipc_response_ok_serialization() {
        let resp = IpcResponse::Ok;
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, "{\"msg\":\"ok\"}");

        let parsed: IpcResponse = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcResponse::Ok));
    }

    #[test]
    fn ipc_response_pong_serialization() {
        let resp = IpcResponse::Pong;
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, "{\"msg\":\"pong\"}");
    }

    #[test]
    fn ipc_response_error_serialization() {
        let resp = IpcResponse::Error {
            message: "test error".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"msg\":\"error\""));
        assert!(json.contains("\"message\":\"test error\""));

        let parsed: IpcResponse = serde_json::from_str(&json).unwrap();
        match parsed {
            IpcResponse::Error { message } => assert_eq!(message, "test error"),
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn session_info_serialization() {
        let info = SessionInfo {
            session_id: "abc123".into(),
            cwd: "/tmp".into(),
            state: SessionState::Running,
            running_tools: vec![RunningTool {
                tool_id: "t1".into(),
                tool_name: "Read".into(),
            }],
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"session_id\":\"abc123\""));
        assert!(json.contains("\"state\":\"running\""));
        assert!(json.contains("\"tool_name\":\"Read\""));

        let parsed: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id, "abc123");
        assert_eq!(parsed.running_tools.len(), 1);
    }
}
