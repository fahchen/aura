//! Aura Hook Handler
//!
//! Invoked by Claude Code on hook events.
//! Reads JSON from stdin, converts to AgentEvent, sends to daemon via IPC.

use aura_common::adapters::claude_code::{parse_hook, HookEvent};
use aura_common::{socket_path, AgentEvent, IpcMessage, IpcResponse};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use tracing::error;
use tracing_subscriber::{fmt, EnvFilter};

fn init_tracing() {
    let filter =
        EnvFilter::try_from_env("AURA_LOG").unwrap_or_else(|_| EnvFilter::new("warn"));
    fmt().with_env_filter(filter).with_target(false).init();
}

/// Process hook input: parse JSON and convert to AgentEvent.
///
/// This is the core processing logic, extracted for testability.
pub fn process_hook_input(input: &str) -> Result<AgentEvent, String> {
    let hook: HookEvent = parse_hook(input).map_err(|e| format!("failed to parse hook: {e}"))?;
    Ok(hook.into())
}

fn main() {
    init_tracing();

    // Read JSON from stdin
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        error!("failed to read stdin: {e}");
        std::process::exit(1);
    }

    // Process input
    let event = match process_hook_input(&input) {
        Ok(e) => e,
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    };

    // Send to daemon via IPC (fail gracefully if daemon not running)
    if let Err(e) = send_to_daemon(&event) {
        error!("{e}");
        // Exit 0 so Claude Code doesn't fail
    }
}

fn send_to_daemon(event: &AgentEvent) -> Result<(), String> {
    let path = socket_path();

    // Connect with timeout
    let mut stream = UnixStream::connect(&path)
        .map_err(|e| format!("daemon not running ({path:?}): {e}"))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("failed to set read timeout: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("failed to set write timeout: {e}"))?;

    // Send message as JSON line
    let message = IpcMessage::Event(event.clone());
    let json = serde_json::to_string(&message).map_err(|e| format!("failed to serialize: {e}"))?;

    stream
        .write_all(json.as_bytes())
        .map_err(|e| format!("failed to write: {e}"))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("failed to write newline: {e}"))?;
    stream.flush().map_err(|e| format!("failed to flush: {e}"))?;

    // Read response
    let mut reader = BufReader::new(&stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("failed to read response: {e}"))?;

    let response: IpcResponse =
        serde_json::from_str(&response_line).map_err(|e| format!("invalid response: {e}"))?;

    match response {
        IpcResponse::Ok => Ok(()),
        IpcResponse::Pong => Ok(()),
        IpcResponse::Error { message } => Err(format!("daemon error: {message}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_common::AgentType;

    // ==================== Valid input tests ====================

    #[test]
    fn process_session_start_hook() {
        let json = r#"{
            "hook_event_name": "SessionStart",
            "session_id": "test-session-123",
            "cwd": "/home/user/project",
            "source": "cli"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent,
            } => {
                assert_eq!(session_id, "test-session-123");
                assert_eq!(cwd, "/home/user/project");
                assert_eq!(agent, AgentType::ClaudeCode);
            }
            _ => panic!("Expected SessionStarted event"),
        }
    }

    #[test]
    fn process_pre_tool_use_hook() {
        let json = r#"{
            "hook_event_name": "PreToolUse",
            "session_id": "s1",
            "cwd": "/tmp",
            "tool_name": "Read",
            "tool_use_id": "toolu_abc123"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::ToolStarted {
                session_id,
                tool_id,
                tool_name,
                tool_label,
            } => {
                assert_eq!(session_id, "s1");
                assert_eq!(tool_id, "toolu_abc123");
                assert_eq!(tool_name, "Read");
                assert_eq!(tool_label, None);
            }
            _ => panic!("Expected ToolStarted event"),
        }
    }

    #[test]
    fn process_post_tool_use_hook() {
        let json = r#"{
            "hook_event_name": "PostToolUse",
            "session_id": "s1",
            "cwd": "/tmp",
            "tool_name": "Read",
            "tool_use_id": "toolu_abc123"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::ToolCompleted {
                session_id,
                tool_id,
            } => {
                assert_eq!(session_id, "s1");
                assert_eq!(tool_id, "toolu_abc123");
            }
            _ => panic!("Expected ToolCompleted event"),
        }
    }

    #[test]
    fn process_permission_request_hook() {
        let json = r#"{
            "hook_event_name": "PermissionRequest",
            "session_id": "s1",
            "cwd": "/tmp",
            "tool_name": "Bash"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::NeedsAttention { session_id, message } => {
                assert_eq!(session_id, "s1");
                assert_eq!(message, Some("Bash".into()));
            }
            _ => panic!("Expected NeedsAttention event"),
        }
    }

    #[test]
    fn process_stop_hook() {
        let json = r#"{
            "hook_event_name": "Stop",
            "session_id": "s1",
            "cwd": "/tmp"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::Idle { session_id } => {
                assert_eq!(session_id, "s1");
            }
            _ => panic!("Expected Idle event"),
        }
    }

    #[test]
    fn process_pre_compact_hook() {
        let json = r#"{
            "hook_event_name": "PreCompact",
            "session_id": "s1",
            "cwd": "/tmp",
            "trigger": "auto"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::Compacting { session_id } => {
                assert_eq!(session_id, "s1");
            }
            _ => panic!("Expected Compacting event"),
        }
    }

    #[test]
    fn process_session_end_hook() {
        let json = r#"{
            "hook_event_name": "SessionEnd",
            "session_id": "s1",
            "cwd": "/tmp",
            "reason": "exit"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::SessionEnded { session_id } => {
                assert_eq!(session_id, "s1");
            }
            _ => panic!("Expected SessionEnded event"),
        }
    }

    #[test]
    fn process_user_prompt_submit_hook() {
        let json = r#"{
            "hook_event_name": "UserPromptSubmit",
            "session_id": "s1",
            "cwd": "/tmp",
            "prompt": "fix the bug"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::Activity { session_id } => {
                assert_eq!(session_id, "s1");
            }
            _ => panic!("Expected Activity event"),
        }
    }

    #[test]
    fn process_notification_permission_hook() {
        let json = r#"{
            "hook_event_name": "Notification",
            "session_id": "s1",
            "cwd": "/tmp",
            "notification_type": "permission_prompt",
            "message": "Permission needed for Bash"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::NeedsAttention { session_id, message } => {
                assert_eq!(session_id, "s1");
                assert_eq!(message, Some("Permission needed for Bash".into()));
            }
            _ => panic!("Expected NeedsAttention event"),
        }
    }

    #[test]
    fn process_notification_other_hook() {
        let json = r#"{
            "hook_event_name": "Notification",
            "session_id": "s1",
            "cwd": "/tmp",
            "notification_type": "idle"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::Activity { session_id } => {
                assert_eq!(session_id, "s1");
            }
            _ => panic!("Expected Activity event for non-permission notification"),
        }
    }

    #[test]
    fn process_subagent_stop_hook() {
        let json = r#"{
            "hook_event_name": "SubagentStop",
            "session_id": "s1",
            "cwd": "/tmp"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::Activity { session_id } => {
                assert_eq!(session_id, "s1");
            }
            _ => panic!("Expected Activity event"),
        }
    }

    // ==================== Invalid input tests ====================

    #[test]
    fn process_invalid_json_returns_error() {
        let invalid = "not valid json";

        let result = process_hook_input(invalid);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("failed to parse hook"));
    }

    #[test]
    fn process_empty_input_returns_error() {
        let result = process_hook_input("");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("failed to parse hook"));
    }

    #[test]
    fn process_valid_json_wrong_structure_returns_error() {
        // Valid JSON but not a hook event
        let wrong_structure = r#"{"foo": "bar"}"#;

        let result = process_hook_input(wrong_structure);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("failed to parse hook"));
    }

    #[test]
    fn process_missing_required_field_returns_error() {
        // Missing session_id
        let missing_field = r#"{
            "hook_event_name": "SessionStart",
            "cwd": "/tmp"
        }"#;

        let result = process_hook_input(missing_field);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("failed to parse hook"));
    }

    #[test]
    fn process_unknown_hook_event_returns_error() {
        let unknown = r#"{
            "hook_event_name": "UnknownEvent",
            "session_id": "s1",
            "cwd": "/tmp"
        }"#;

        let result = process_hook_input(unknown);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("failed to parse hook"));
    }

    #[test]
    fn process_hook_with_extra_whitespace() {
        // Real stdin input might have extra whitespace/newlines
        let json = r#"
            {
                "hook_event_name": "Stop",
                "session_id": "s1",
                "cwd": "/tmp"
            }
        "#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::Idle { session_id } => {
                assert_eq!(session_id, "s1");
            }
            _ => panic!("Expected Idle event"),
        }
    }

    #[test]
    fn process_hook_with_optional_fields_omitted() {
        // PreToolUse with only required fields (tool_input is optional)
        let json = r#"{
            "hook_event_name": "PreToolUse",
            "session_id": "s1",
            "cwd": "/tmp",
            "tool_name": "Edit",
            "tool_use_id": "t1"
        }"#;

        let event = process_hook_input(json).unwrap();

        match event {
            AgentEvent::ToolStarted { tool_name, .. } => {
                assert_eq!(tool_name, "Edit");
            }
            _ => panic!("Expected ToolStarted event"),
        }
    }

    #[test]
    fn process_hook_with_extra_unknown_fields() {
        // JSON with extra fields should still parse (serde default behavior)
        let json = r#"{
            "hook_event_name": "Stop",
            "session_id": "s1",
            "cwd": "/tmp",
            "unknown_field": "ignored"
        }"#;

        // This depends on serde config - with deny_unknown_fields it would fail
        // With default config, it should succeed
        let result = process_hook_input(json);
        // Either way is acceptable behavior, just verify it doesn't panic
        match result {
            Ok(AgentEvent::Idle { session_id }) => {
                assert_eq!(session_id, "s1");
            }
            Err(e) => {
                // If deny_unknown_fields is enabled, this is expected
                assert!(e.contains("failed to parse hook"));
            }
            _ => panic!("Unexpected event type"),
        }
    }
}
