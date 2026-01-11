//! IPC server - listens for events from hook handlers

use crate::registry::SessionRegistry;
use aura_common::{socket_path, IpcMessage, IpcResponse};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

/// Handle a parsed IPC message and update the registry.
///
/// This is the core message handling logic, extracted for testability.
pub fn handle_message(message: IpcMessage, registry: &Arc<Mutex<SessionRegistry>>) -> IpcResponse {
    match message {
        IpcMessage::Event(event) => {
            if let Ok(mut reg) = registry.lock() {
                reg.process_event(event);
            }
            IpcResponse::Ok
        }
        IpcMessage::Ping => IpcResponse::Pong,
    }
}

/// Parse a JSON line into an IpcMessage and handle it.
///
/// Returns an IpcResponse for both valid and invalid messages.
pub fn parse_and_handle_message(
    line: &str,
    registry: &Arc<Mutex<SessionRegistry>>,
) -> IpcResponse {
    match serde_json::from_str::<IpcMessage>(line) {
        Ok(message) => handle_message(message, registry),
        Err(e) => IpcResponse::Error {
            message: format!("invalid message: {e}"),
        },
    }
}

/// Start the IPC server
pub async fn run(registry: Arc<Mutex<SessionRegistry>>) -> std::io::Result<()> {
    let path = socket_path();

    // Remove existing socket if present
    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    let listener = UnixListener::bind(&path)?;
    info!("listening on {}", path.display());

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let registry = Arc::clone(&registry);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, registry).await {
                        warn!("connection error: {e}");
                    }
                });
            }
            Err(e) => {
                error!("accept error: {e}");
            }
        }
    }
}

async fn handle_connection(
    stream: UnixStream,
    registry: Arc<Mutex<SessionRegistry>>,
) -> std::io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read one line (JSON message)
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        return Ok(()); // EOF
    }

    // Parse and handle message
    let response = parse_and_handle_message(&line, &registry);

    // Send response
    let response_json = serde_json::to_string(&response).unwrap();
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_common::{AgentEvent, AgentType, SessionState};

    fn create_registry() -> Arc<Mutex<SessionRegistry>> {
        Arc::new(Mutex::new(SessionRegistry::new()))
    }

    // ==================== handle_message tests ====================

    #[test]
    fn handle_message_event_updates_registry() {
        let registry = create_registry();

        let event = AgentEvent::SessionStarted {
            session_id: "test-session".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        };
        let message = IpcMessage::Event(event);

        let response = handle_message(message, &registry);

        assert!(matches!(response, IpcResponse::Ok));

        let reg = registry.lock().unwrap();
        assert_eq!(reg.len(), 1);
        let sessions = reg.get_all();
        assert_eq!(sessions[0].session_id, "test-session");
        assert_eq!(sessions[0].state, SessionState::Running);
    }

    #[test]
    fn handle_message_ping_returns_pong() {
        let registry = create_registry();

        let response = handle_message(IpcMessage::Ping, &registry);

        assert!(matches!(response, IpcResponse::Pong));
        // Registry should be unchanged
        assert_eq!(registry.lock().unwrap().len(), 0);
    }

    #[test]
    fn handle_message_tool_started_adds_tool() {
        let registry = create_registry();

        // First start a session
        let start_event = AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        };
        handle_message(IpcMessage::Event(start_event), &registry);

        // Then start a tool
        let tool_event = AgentEvent::ToolStarted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
            tool_label: Some("main.rs".into()),
        };
        let response = handle_message(IpcMessage::Event(tool_event), &registry);

        assert!(matches!(response, IpcResponse::Ok));

        let sessions = registry.lock().unwrap().get_all();
        assert_eq!(sessions[0].running_tools.len(), 1);
        assert_eq!(sessions[0].running_tools[0].tool_name, "Read");
    }

    #[test]
    fn handle_message_tool_completed_moves_to_recent() {
        let registry = create_registry();

        // Setup: session with a running tool
        handle_message(
            IpcMessage::Event(AgentEvent::SessionStarted {
                session_id: "s1".into(),
                cwd: "/tmp".into(),
                agent: AgentType::ClaudeCode,
            }),
            &registry,
        );
        handle_message(
            IpcMessage::Event(AgentEvent::ToolStarted {
                session_id: "s1".into(),
                tool_id: "t1".into(),
                tool_name: "Read".into(),
                tool_label: None,
            }),
            &registry,
        );

        // Complete the tool
        let response = handle_message(
            IpcMessage::Event(AgentEvent::ToolCompleted {
                session_id: "s1".into(),
                tool_id: "t1".into(),
            }),
            &registry,
        );

        assert!(matches!(response, IpcResponse::Ok));
        let sessions = registry.lock().unwrap().get_all();
        // Tool should still be visible via recent_tools (minimum display duration)
        assert_eq!(sessions[0].running_tools.len(), 1);
        assert!(sessions[0].running_tools[0].tool_id.starts_with("recent_"));
    }

    // ==================== parse_and_handle_message tests ====================

    #[test]
    fn parse_and_handle_valid_ping_json() {
        let registry = create_registry();
        let json = r#"{"msg":"ping"}"#;

        let response = parse_and_handle_message(json, &registry);

        assert!(matches!(response, IpcResponse::Pong));
    }

    #[test]
    fn parse_and_handle_valid_event_json() {
        let registry = create_registry();
        let json = r#"{
            "msg": "event",
            "type": "session_started",
            "session_id": "abc123",
            "cwd": "/home/user",
            "agent": "claude_code"
        }"#;

        let response = parse_and_handle_message(json, &registry);

        assert!(matches!(response, IpcResponse::Ok));
        assert_eq!(registry.lock().unwrap().len(), 1);
    }

    #[test]
    fn parse_and_handle_invalid_json_returns_error() {
        let registry = create_registry();
        let invalid_json = "not valid json at all";

        let response = parse_and_handle_message(invalid_json, &registry);

        match response {
            IpcResponse::Error { message } => {
                assert!(message.contains("invalid message"));
            }
            _ => panic!("Expected Error response"),
        }
        // Registry should be unchanged
        assert_eq!(registry.lock().unwrap().len(), 0);
    }

    #[test]
    fn parse_and_handle_malformed_ipc_message_returns_error() {
        let registry = create_registry();
        // Valid JSON but wrong structure
        let malformed = r#"{"foo": "bar"}"#;

        let response = parse_and_handle_message(malformed, &registry);

        match response {
            IpcResponse::Error { message } => {
                assert!(message.contains("invalid message"));
            }
            _ => panic!("Expected Error response"),
        }
    }

    #[test]
    fn parse_and_handle_incomplete_event_returns_error() {
        let registry = create_registry();
        // Has msg:event but missing required event fields
        let incomplete = r#"{"msg": "event"}"#;

        let response = parse_and_handle_message(incomplete, &registry);

        match response {
            IpcResponse::Error { message } => {
                assert!(message.contains("invalid message"));
            }
            _ => panic!("Expected Error response"),
        }
    }

    #[test]
    fn parse_and_handle_empty_string_returns_error() {
        let registry = create_registry();

        let response = parse_and_handle_message("", &registry);

        match response {
            IpcResponse::Error { message } => {
                assert!(message.contains("invalid message"));
            }
            _ => panic!("Expected Error response"),
        }
    }

    #[test]
    fn parse_and_handle_json_with_extra_whitespace() {
        let registry = create_registry();
        // JSON with newlines and trailing whitespace (as would come from read_line)
        let json = "  {\"msg\":\"ping\"}  \n";

        let response = parse_and_handle_message(json, &registry);

        assert!(matches!(response, IpcResponse::Pong));
    }

    #[test]
    fn parse_and_handle_unknown_msg_type_returns_error() {
        let registry = create_registry();
        let unknown = r#"{"msg": "unknown_type"}"#;

        let response = parse_and_handle_message(unknown, &registry);

        match response {
            IpcResponse::Error { message } => {
                assert!(message.contains("invalid message"));
            }
            _ => panic!("Expected Error response"),
        }
    }
}
