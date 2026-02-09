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
#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum HookAgent {
    ClaudeCode,
    Codex,
    GeminiCli,
    OpenCode,
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

            let mut events = vec![AgentEvent::ToolStarted {
                session_id: session_id.clone(),
                cwd,
                tool_id,
                tool_name: tool_name.clone(),
                tool_label,
            }];

            if tool_name == "Bash" {
                if let Some(tool_input) = hook.get("tool_input") {
                    if let Some(name) = parse_set_name_command(tool_input) {
                        events.push(AgentEvent::SessionNameUpdated {
                            session_id,
                            name,
                        });
                    }
                }
            }

            events
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

        "SubagentStart" | "SubagentStop" => {
            vec![AgentEvent::Activity { session_id, cwd }]
        }

        _ => return None,
    };

    Some(messages)
}

/// Parse `aura set-name "..."` from a Bash tool_input's `command` field.
///
/// Supports double quotes, single quotes, and unquoted single-token names.
/// Returns the extracted name, or `None` if the command is not an `aura set-name` invocation.
fn parse_set_name_command(tool_input: &Value) -> Option<String> {
    let command = tool_input.get("command")?.as_str()?;
    let trimmed = command.trim();

    // Use split_whitespace to skip arbitrary interior whitespace, then verify
    // the first two tokens are "<something>/aura" (or just "aura") and "set-name".
    // Accepts: "aura", "./aura", "/usr/local/bin/aura", "../aura", etc.
    let mut tokens = trimmed.split_whitespace();
    let binary = tokens.next()?;
    let basename = binary.rsplit('/').next().unwrap_or(binary);
    if basename != "aura" {
        return None;
    }
    if tokens.next() != Some("set-name") {
        return None;
    }

    // Find where the name argument starts in the original string (after "set-name" + whitespace)
    let set_name_pos = trimmed.find("set-name")?;
    let after_keyword = &trimmed[set_name_pos + "set-name".len()..];
    let rest = after_keyword.trim();
    if rest.is_empty() {
        return None;
    }

    // Strip matching quotes if present
    if (rest.starts_with('"') && rest.ends_with('"'))
        || (rest.starts_with('\'') && rest.ends_with('\''))
    {
        let inner = &rest[1..rest.len() - 1];
        if inner.is_empty() {
            return None;
        }
        return Some(inner.to_string());
    }

    Some(rest.to_string())
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
        "NotebookEdit" => input
            .get("notebook_path")
            .and_then(|v| v.as_str())
            .map(super::short_path),
        "Skill" => input
            .get("skill")
            .and_then(|v| v.as_str())
            .map(String::from),
        "AskUserQuestion" => Some("AskUserQuestion".to_string()),
        "EnterPlanMode" => Some("EnterPlanMode".to_string()),
        name if name.starts_with("mcp__") => {
            let stripped = name.strip_prefix("mcp__").unwrap();
            Some(stripped.rsplit("__").next().unwrap_or(stripped).to_string())
        }
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
    fn subagent_start_emits_activity() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "SubagentStart",
            "agent_type": "Explore"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("\"type\":\"activity\""));
    }

    #[test]
    fn subagent_stop_emits_activity() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "SubagentStop",
            "agent_type": "Explore"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("\"type\":\"activity\""));
    }

    // --- Event normalization tests ---

    #[test]
    fn post_tool_use_failure_normalizes_to_tool_completed() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "PostToolUseFailure",
            "tool_name": "Bash",
            "tool_use_id": "toolu_fail",
            "error": "Command exited with non-zero status code 1"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("tool_completed"));
        assert!(json.contains("toolu_fail"));
    }

    #[test]
    fn notification_and_permission_request_produce_same_event() {
        let notification = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "Notification",
            "notification_type": "permission_prompt",
            "tool_name": "Bash",
            "message": "Allow Bash command?"
        });
        let permission = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "PermissionRequest",
            "tool_name": "Bash"
        });

        let n_msgs = convert_claude_code(&notification).unwrap();
        let p_msgs = convert_claude_code(&permission).unwrap();

        // Both produce exactly one NeedsAttention event
        assert_eq!(n_msgs.len(), 1);
        assert_eq!(p_msgs.len(), 1);

        let n_json = serde_json::to_string(&n_msgs[0]).unwrap();
        let p_json = serde_json::to_string(&p_msgs[0]).unwrap();

        // Both are needs_attention with message "Bash"
        assert!(n_json.contains("needs_attention"));
        assert!(p_json.contains("needs_attention"));
        assert!(n_json.contains("\"message\":\"Bash\""));
        assert!(p_json.contains("\"message\":\"Bash\""));
    }

    #[test]
    fn notification_auth_success_uses_message_field() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "Notification",
            "notification_type": "auth_success",
            "message": "Authenticated successfully"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("needs_attention"));
        assert!(json.contains("Authenticated successfully"));
    }

    #[test]
    fn notification_elicitation_dialog_uses_message_field() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "Notification",
            "notification_type": "elicitation_dialog",
            "message": "Choose an option"
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 1);
        let json = serde_json::to_string(&msgs[0]).unwrap();
        assert!(json.contains("needs_attention"));
        assert!(json.contains("Choose an option"));
    }

    #[test]
    fn activity_hooks_all_produce_same_event_type() {
        let hooks = vec![
            serde_json::json!({
                "session_id": "abc123",
                "cwd": "/home/user/project",
                "hook_event_name": "UserPromptSubmit",
                "prompt": "fix the bug"
            }),
            serde_json::json!({
                "session_id": "abc123",
                "cwd": "/home/user/project",
                "hook_event_name": "SubagentStart",
                "agent_type": "Explore"
            }),
            serde_json::json!({
                "session_id": "abc123",
                "cwd": "/home/user/project",
                "hook_event_name": "SubagentStop",
                "agent_type": "Explore"
            }),
        ];

        for hook in &hooks {
            let event_name = hook["hook_event_name"].as_str().unwrap();
            let msgs = convert_claude_code(hook).unwrap();
            assert_eq!(msgs.len(), 1, "{event_name} should produce exactly one event");
            let json = serde_json::to_string(&msgs[0]).unwrap();
            assert!(
                json.contains("\"type\":\"activity\""),
                "{event_name} should produce Activity, got: {json}"
            );
            assert!(
                json.contains("\"session_id\":\"abc123\""),
                "{event_name} should preserve session_id"
            );
        }
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

    #[test]
    fn tool_label_notebook_edit() {
        let hook = serde_json::json!({
            "tool_name": "NotebookEdit",
            "tool_input": { "notebook_path": "/home/user/project/analysis.ipynb" }
        });
        assert_eq!(extract_tool_label(&hook), Some("analysis.ipynb".to_string()));
    }

    #[test]
    fn tool_label_skill() {
        let hook = serde_json::json!({
            "tool_name": "Skill",
            "tool_input": { "skill": "commit" }
        });
        assert_eq!(extract_tool_label(&hook), Some("commit".to_string()));
    }

    #[test]
    fn tool_label_ask_user_question() {
        let hook = serde_json::json!({
            "tool_name": "AskUserQuestion",
            "tool_input": { "questions": [] }
        });
        assert_eq!(extract_tool_label(&hook), Some("AskUserQuestion".to_string()));
    }

    #[test]
    fn tool_label_enter_plan_mode() {
        let hook = serde_json::json!({
            "tool_name": "EnterPlanMode",
            "tool_input": {}
        });
        assert_eq!(extract_tool_label(&hook), Some("EnterPlanMode".to_string()));
    }

    #[test]
    fn tool_label_mcp_tool() {
        let hook = serde_json::json!({
            "tool_name": "mcp__memory__memory_search",
            "tool_input": { "query": "test" }
        });
        assert_eq!(extract_tool_label(&hook), Some("memory_search".to_string()));
    }

    // --- parse_set_name_command tests ---

    #[test]
    fn parse_set_name_double_quotes() {
        let input = serde_json::json!({ "command": "aura set-name \"fix login bug\"" });
        assert_eq!(
            parse_set_name_command(&input),
            Some("fix login bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_single_quotes() {
        let input = serde_json::json!({ "command": "aura set-name 'fix login bug'" });
        assert_eq!(
            parse_set_name_command(&input),
            Some("fix login bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_no_quotes() {
        let input = serde_json::json!({ "command": "aura set-name fix-login-bug" });
        assert_eq!(
            parse_set_name_command(&input),
            Some("fix-login-bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_extra_whitespace() {
        let input = serde_json::json!({ "command": "aura  set-name  \"fix login bug\"" });
        assert_eq!(
            parse_set_name_command(&input),
            Some("fix login bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_not_matching() {
        let input = serde_json::json!({ "command": "echo hello" });
        assert_eq!(parse_set_name_command(&input), None);
    }

    #[test]
    fn parse_set_name_relative_path() {
        let input = serde_json::json!({ "command": "./aura set-name \"fix bug\"" });
        assert_eq!(
            parse_set_name_command(&input),
            Some("fix bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_absolute_path() {
        let input = serde_json::json!({ "command": "/usr/local/bin/aura set-name \"fix bug\"" });
        assert_eq!(
            parse_set_name_command(&input),
            Some("fix bug".to_string())
        );
    }

    #[test]
    fn convert_pre_tool_use_with_set_name() {
        let hook = serde_json::json!({
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_use_id": "toolu_02",
            "tool_input": {
                "command": "aura set-name \"test session\"",
                "description": "Set session name"
            }
        });
        let msgs = convert_claude_code(&hook).unwrap();
        assert_eq!(msgs.len(), 2, "expected ToolStarted + SessionNameUpdated");

        let first = serde_json::to_string(&msgs[0]).unwrap();
        assert!(first.contains("tool_started"));

        let second = serde_json::to_string(&msgs[1]).unwrap();
        assert!(second.contains("session_name_updated"));
        assert!(second.contains("test session"));
        assert!(second.contains("abc123"));
    }
}
