//! Agent hook handler
//!
//! Reads hook JSON from stdin, converts to `AgentEvent`s, sends to daemon via Unix socket.
//! Invoked as `aura hook --agent <name>` subcommand.
//!
//! Each agent has its own stdin JSON format. The `--agent` flag selects the parser.
//!
//! # Claude Code hooks config:
//! ```json
//! { "type": "command", "command": "aura hook --agent claude-code" }
//! ```

use crate::ipc;
use crate::{AgentEvent, AgentType};
use serde_json::Value;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

/// Agent identifier for the `--agent` CLI flag.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum HookAgent {
    ClaudeCode,
    Codex,
    GeminiCli,
    OpenCode,
}

/// All Claude Code hook events that Aura subscribes to.
const CLAUDE_CODE_HOOK_EVENTS: &[&str] = &[
    "SessionStart",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "Notification",
    "PermissionRequest",
    "Stop",
    "PreCompact",
    "SessionEnd",
    "UserPromptSubmit",
];

/// Print Claude Code hooks config JSON for `~/.claude/settings.json`.
pub fn print_install_config() {
    let hook_obj = serde_json::json!({
        "type": "command",
        "command": "aura hook --agent claude-code"
    });

    let mut hooks = serde_json::Map::new();
    for event in CLAUDE_CODE_HOOK_EVENTS {
        hooks.insert(
            event.to_string(),
            serde_json::json!([{ "hooks": [hook_obj] }]),
        );
    }

    let output = serde_json::to_string_pretty(&serde_json::Value::Object(hooks)).unwrap();
    println!("Add the following to your ~/.claude/settings.json under \"hooks\":\n");
    println!("{output}");
}

/// Entry point for `aura hook` subcommand.
pub fn run(agent: &HookAgent) {
    let converter: fn(&Value) -> Option<Vec<AgentEvent>> = match agent {
        HookAgent::ClaudeCode => convert_claude_code,
        other => {
            eprintln!("hook handler for {other:?} is not yet implemented");
            return;
        }
    };

    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let hook: Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => return,
    };

    let Some(messages) = converter(&hook) else {
        return;
    };

    let path = ipc::socket_path();
    let mut stream = match UnixStream::connect(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("daemon not running ({:?}): {}", path.display(), e);
            return;
        }
    };

    for msg in messages {
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = writeln!(stream, "{json}");
        }
    }
}

fn common_fields(hook: &Value) -> Option<(String, String)> {
    let session_id = hook.get("session_id")?.as_str()?.to_string();
    let cwd = hook
        .get("cwd")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Some((session_id, cwd))
}

/// Convert Claude Code hook JSON to agent events.
///
/// Claude Code hooks deliver JSON via stdin with a `hook_event_name` field.
/// See: https://docs.anthropic.com/en/docs/claude-code/hooks
fn convert_claude_code(hook: &Value) -> Option<Vec<AgentEvent>> {
    let event_name = hook.get("hook_event_name")?.as_str()?;
    let (session_id, cwd) = common_fields(hook)?;

    let messages = match event_name {
        "SessionStart" => {
            vec![AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent: AgentType::ClaudeCode,
            }]
        }

        "PreToolUse" => {
            let tool_name = hook
                .get("tool_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let tool_id = hook
                .get("tool_use_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let tool_label = extract_tool_label(hook);

            vec![AgentEvent::ToolStarted {
                session_id,
                cwd,
                tool_id,
                tool_name,
                tool_label,
            }]
        }

        "PostToolUse" | "PostToolUseFailure" => {
            let tool_id = hook
                .get("tool_use_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            vec![AgentEvent::ToolCompleted {
                session_id,
                cwd,
                tool_id,
            }]
        }

        "Notification" => {
            let notification_type = hook
                .get("notification_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match notification_type {
                "permission_prompt" => {
                    let message = hook
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    vec![AgentEvent::NeedsAttention {
                        session_id,
                        cwd,
                        message,
                    }]
                }
                "idle_prompt" => {
                    let message = hook
                        .get("message")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    vec![AgentEvent::WaitingForInput {
                        session_id,
                        cwd,
                        message,
                    }]
                }
                _ => {
                    let message = hook
                        .get("message")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    vec![AgentEvent::NeedsAttention {
                        session_id,
                        cwd,
                        message,
                    }]
                }
            }
        }

        "PermissionRequest" => {
            let message = hook
                .get("tool_name")
                .and_then(|v| v.as_str())
                .map(String::from);
            vec![AgentEvent::NeedsAttention {
                session_id,
                cwd,
                message,
            }]
        }

        "Stop" => {
            vec![AgentEvent::Idle { session_id, cwd }]
        }

        "PreCompact" => {
            vec![AgentEvent::Compacting { session_id, cwd }]
        }

        "SessionEnd" => {
            vec![AgentEvent::SessionEnded { session_id }]
        }

        "UserPromptSubmit" => {
            vec![AgentEvent::Activity { session_id, cwd }]
        }

        "SubagentStart" | "SubagentStop" => return None,

        _ => return None,
    };

    Some(messages)
}

/// Extract a human-readable label for a tool invocation
fn extract_tool_label(hook: &Value) -> Option<String> {
    let tool_name = hook.get("tool_name")?.as_str()?;
    let input = hook.get("tool_input")?;

    match tool_name {
        "Bash" => input
            .get("description")
            .and_then(|v| v.as_str())
            .or_else(|| input.get("command").and_then(|v| v.as_str()))
            .map(|s| super::truncate(s, 60).to_string()),
        "Read" | "Write" | "Edit" => input
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(super::short_path),
        "Glob" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(String::from),
        "Grep" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| super::truncate(s, 40).to_string()),
        "WebFetch" => input
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| super::truncate(s, 60).to_string()),
        "WebSearch" => input
            .get("query")
            .and_then(|v| v.as_str())
            .map(|s| super::truncate(s, 60).to_string()),
        "Task" => input
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| super::truncate(s, 60).to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_session_start() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "SessionStart",
            "source": "startup"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("session_started"));
    }

    #[test]
    fn convert_pre_tool_use() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_use_id": "toolu_01",
            "tool_input": {
                "command": "npm test",
                "description": "Run test suite"
            }
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("tool_started"));
        assert!(json.contains("Run test suite"));
    }

    #[test]
    fn convert_post_tool_use() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_use_id": "toolu_01"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("tool_completed"));
    }

    #[test]
    fn convert_notification_permission() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "Notification",
            "notification_type": "permission_prompt",
            "tool_name": "Bash",
            "message": "Allow Bash command?"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("needs_attention"));
        assert!(json.contains("Bash"));
    }

    #[test]
    fn convert_notification_idle_prompt() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "Notification",
            "notification_type": "idle_prompt",
            "message": "What would you like to do?"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("waiting_for_input"));
    }

    #[test]
    fn convert_stop() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "Stop"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("\"type\":\"idle\""));
    }

    #[test]
    fn convert_pre_compact() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "PreCompact"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("compacting"));
    }

    #[test]
    fn convert_session_end() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "SessionEnd"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("session_ended"));
    }

    #[test]
    fn convert_user_prompt_submit() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "UserPromptSubmit",
            "prompt": "fix the bug"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("\"type\":\"activity\""));
    }

    #[test]
    fn convert_permission_request() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "PermissionRequest",
            "tool_name": "Write"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("needs_attention"));
        assert!(json.contains("Write"));
    }

    #[test]
    fn subagent_events_ignored() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "SubagentStart"
        });
        assert!(convert_claude_code(&hook).is_none());
    }

    #[test]
    fn unknown_event_ignored() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "SomeUnknownEvent"
        });
        assert!(convert_claude_code(&hook).is_none());
    }

    #[test]
    fn tool_label_extraction() {
        let hook = serde_json::json!({
            "tool_name": "Read",
            "tool_input": { "file_path": "/home/user/project/src/main.rs" }
        });
        let label = extract_tool_label(&hook);
        assert_eq!(label, Some("main.rs".to_string()));
    }
}
