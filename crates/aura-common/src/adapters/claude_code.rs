//! Claude Code adapter
//!
//! Parses Claude Code hook JSON and converts to AgentEvent.

use crate::{AgentEvent, AgentType};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Common fields present in all hook payloads
/// Note: hook_event_name is not here because serde uses it as the enum tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPayloadCommon {
    pub session_id: String,
    #[serde(default)]
    pub transcript_path: Option<String>,
    pub cwd: String,
    #[serde(default)]
    pub permission_mode: Option<String>,
}

/// SessionStart hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartPayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    #[serde(default)]
    pub source: Option<String>,
}

/// UserPromptSubmit hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptSubmitPayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    #[serde(default)]
    pub prompt: Option<String>,
}

/// PreToolUse hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolUsePayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: Option<Value>,
    pub tool_use_id: String,
}

/// PostToolUse hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolUsePayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: Option<Value>,
    #[serde(default)]
    pub tool_response: Option<Value>,
    pub tool_use_id: String,
}

/// PermissionRequest hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequestPayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<Value>,
}

/// Notification hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub notification_type: Option<String>,
}

/// Stop hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopPayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    #[serde(default)]
    pub stop_hook_active: Option<bool>,
}

/// SubagentStop hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentStopPayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
}

/// PreCompact hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCompactPayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    #[serde(default)]
    pub trigger: Option<String>,
    #[serde(default)]
    pub custom_instructions: Option<String>,
}

/// SessionEnd hook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndPayload {
    #[serde(flatten)]
    pub common: HookPayloadCommon,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Parsed hook event with typed payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook_event_name")]
pub enum HookEvent {
    SessionStart(SessionStartPayload),
    UserPromptSubmit(UserPromptSubmitPayload),
    PreToolUse(PreToolUsePayload),
    PostToolUse(PostToolUsePayload),
    PermissionRequest(PermissionRequestPayload),
    Notification(NotificationPayload),
    Stop(StopPayload),
    SubagentStop(SubagentStopPayload),
    PreCompact(PreCompactPayload),
    SessionEnd(SessionEndPayload),
}

impl HookEvent {
    /// Get the common payload fields from any event
    fn common(&self) -> &HookPayloadCommon {
        match self {
            Self::SessionStart(p) => &p.common,
            Self::UserPromptSubmit(p) => &p.common,
            Self::PreToolUse(p) => &p.common,
            Self::PostToolUse(p) => &p.common,
            Self::PermissionRequest(p) => &p.common,
            Self::Notification(p) => &p.common,
            Self::Stop(p) => &p.common,
            Self::SubagentStop(p) => &p.common,
            Self::PreCompact(p) => &p.common,
            Self::SessionEnd(p) => &p.common,
        }
    }

    /// Get session_id from any event
    pub fn session_id(&self) -> &str {
        &self.common().session_id
    }

    /// Get cwd from any event
    pub fn cwd(&self) -> &str {
        &self.common().cwd
    }
}

/// Parse Claude Code hook JSON into HookEvent
pub fn parse_hook(json: &str) -> Result<HookEvent, serde_json::Error> {
    serde_json::from_str(json)
}

/// Extract a human-readable label from tool_input JSON
pub fn extract_tool_label(tool_name: &str, tool_input: Option<&Value>) -> Option<String> {
    // Handle MCP tools first (they don't need input)
    if tool_name.starts_with("mcp__") {
        return Some(tool_name.rsplit("__").next().unwrap_or(tool_name).to_string());
    }

    let input = tool_input?;

    match tool_name {
        "Bash" => input
            .get("command")
            .and_then(|v| v.as_str())
            .map(extract_bash_label),
        "Read" | "Edit" | "Write" => input
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(extract_filename),
        "Glob" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        "Grep" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| truncate_string(s, 15)),
        "Task" => input
            .get("subagent_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        "WebFetch" => input
            .get("url")
            .and_then(|v| v.as_str())
            .and_then(extract_domain),
        "WebSearch" => input
            .get("query")
            .and_then(|v| v.as_str())
            .map(|s| truncate_string(s, 15)),
        _ => None,
    }
}

fn extract_bash_label(command: &str) -> String {
    let parts: Vec<&str> = command.split_whitespace().take(3).collect();
    truncate_string(&parts.join(" "), 20)
}

fn extract_filename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

fn extract_domain(url: &str) -> Option<String> {
    url.split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .map(|domain| domain.to_string())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(max_len.saturating_sub(1)).collect::<String>()
        )
    }
}

/// Convert HookEvent to AgentEvent
impl From<HookEvent> for AgentEvent {
    fn from(hook: HookEvent) -> Self {
        match hook {
            HookEvent::SessionStart(p) => AgentEvent::SessionStarted {
                session_id: p.common.session_id,
                cwd: p.common.cwd,
                agent: AgentType::ClaudeCode,
            },
            HookEvent::UserPromptSubmit(p) => AgentEvent::Activity {
                session_id: p.common.session_id,
                cwd: p.common.cwd,
            },
            HookEvent::PreToolUse(p) => {
                let tool_label = extract_tool_label(&p.tool_name, p.tool_input.as_ref());
                AgentEvent::ToolStarted {
                    session_id: p.common.session_id,
                    cwd: p.common.cwd,
                    tool_id: p.tool_use_id,
                    tool_name: p.tool_name,
                    tool_label,
                }
            }
            HookEvent::PostToolUse(p) => AgentEvent::ToolCompleted {
                session_id: p.common.session_id,
                cwd: p.common.cwd,
                tool_id: p.tool_use_id,
            },
            HookEvent::PermissionRequest(p) => AgentEvent::NeedsAttention {
                session_id: p.common.session_id,
                cwd: p.common.cwd,
                message: p.tool_name,
            },
            HookEvent::Notification(p) => {
                match p.notification_type.as_deref() {
                    Some("permission_prompt") | Some("idle_prompt") => AgentEvent::NeedsAttention {
                        session_id: p.common.session_id,
                        cwd: p.common.cwd,
                        message: p.message,
                    },
                    _ => AgentEvent::Activity {
                        session_id: p.common.session_id,
                        cwd: p.common.cwd,
                    },
                }
            }
            HookEvent::Stop(p) => AgentEvent::Idle {
                session_id: p.common.session_id,
                cwd: p.common.cwd,
            },
            HookEvent::SubagentStop(p) => AgentEvent::Activity {
                session_id: p.common.session_id,
                cwd: p.common.cwd,
            },
            HookEvent::PreCompact(p) => AgentEvent::Compacting {
                session_id: p.common.session_id,
                cwd: p.common.cwd,
            },
            HookEvent::SessionEnd(p) => AgentEvent::SessionEnded {
                session_id: p.common.session_id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_start() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "hook_event_name": "SessionStart",
            "source": "startup"
        }"#;

        let hook = parse_hook(json).unwrap();
        assert_eq!(hook.session_id(), "abc123");
        assert_eq!(hook.cwd(), "/home/user/project");

        let event: AgentEvent = hook.into();
        match event {
            AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent,
            } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(cwd, "/home/user/project");
                assert_eq!(agent, AgentType::ClaudeCode);
            }
            _ => panic!("Expected SessionStarted"),
        }
    }

    #[test]
    fn parse_pre_tool_use() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "PreToolUse",
            "tool_name": "Read",
            "tool_use_id": "toolu_01ABC",
            "tool_input": {"file_path": "/path/to/config.rs"}
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::ToolStarted {
                session_id,
                tool_id,
                tool_name,
                tool_label,
                ..
            } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(tool_id, "toolu_01ABC");
                assert_eq!(tool_name, "Read");
                assert_eq!(tool_label, Some("config.rs".into()));
            }
            _ => panic!("Expected ToolStarted"),
        }
    }

    #[test]
    fn parse_post_tool_use() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "PostToolUse",
            "tool_name": "Read",
            "tool_use_id": "toolu_01ABC"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::ToolCompleted {
                session_id,
                tool_id,
                ..
            } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(tool_id, "toolu_01ABC");
            }
            _ => panic!("Expected ToolCompleted"),
        }
    }

    #[test]
    fn parse_permission_request() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "PermissionRequest",
            "tool_name": "Bash"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::NeedsAttention { session_id, message, .. } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(message, Some("Bash".into()));
            }
            _ => panic!("Expected NeedsAttention"),
        }
    }

    #[test]
    fn parse_notification_permission() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "Notification",
            "notification_type": "permission_prompt",
            "message": "Claude needs permission to run Bash"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::NeedsAttention { session_id, message, .. } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(message, Some("Claude needs permission to run Bash".into()));
            }
            _ => panic!("Expected NeedsAttention for permission_prompt"),
        }
    }

    #[test]
    fn parse_notification_idle_prompt() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "Notification",
            "notification_type": "idle_prompt"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::NeedsAttention { session_id, .. } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected NeedsAttention for idle_prompt"),
        }
    }

    #[test]
    fn parse_notification_other() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "Notification",
            "notification_type": "auth_success"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::Activity { session_id, .. } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected Activity for non-attention notification"),
        }
    }

    #[test]
    fn parse_stop() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "Stop"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::Idle { session_id, .. } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected Idle"),
        }
    }

    #[test]
    fn parse_pre_compact() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "PreCompact",
            "trigger": "auto"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::Compacting { session_id, .. } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected Compacting"),
        }
    }

    #[test]
    fn parse_session_end() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "SessionEnd",
            "reason": "exit"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::SessionEnded { session_id } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected SessionEnded"),
        }
    }

    #[test]
    fn parse_user_prompt_submit() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "UserPromptSubmit"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::Activity { session_id, .. } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected Activity"),
        }
    }

    #[test]
    fn parse_subagent_stop() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "SubagentStop"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::Activity { session_id, .. } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected Activity"),
        }
    }

    #[test]
    fn extract_tool_label_bash() {
        // Bash extracts first 3 words and truncates to 20 chars
        let input = serde_json::json!({"command": "cargo build"});
        let label = extract_tool_label("Bash", Some(&input));
        assert_eq!(label, Some("cargo build".into()));

        // Long command should be truncated at 20 chars
        let input = serde_json::json!({"command": "cargo build --release"});
        let label = extract_tool_label("Bash", Some(&input));
        // "cargo build --release" is 21 chars, should be truncated
        assert_eq!(label, Some("cargo build --relea\u{2026}".into()));

        // Very long command - only first 3 words considered
        let input = serde_json::json!({"command": "very-long-command with lots of arguments and flags"});
        let label = extract_tool_label("Bash", Some(&input));
        assert!(label.as_ref().map(|s| s.chars().count()).unwrap_or(0) <= 20);
    }

    #[test]
    fn extract_tool_label_file_ops() {
        let input = serde_json::json!({"file_path": "/Users/test/project/src/main.rs"});

        assert_eq!(extract_tool_label("Read", Some(&input)), Some("main.rs".into()));
        assert_eq!(extract_tool_label("Edit", Some(&input)), Some("main.rs".into()));
        assert_eq!(extract_tool_label("Write", Some(&input)), Some("main.rs".into()));
    }

    #[test]
    fn extract_tool_label_glob() {
        let input = serde_json::json!({"pattern": "**/*.rs"});
        assert_eq!(extract_tool_label("Glob", Some(&input)), Some("**/*.rs".into()));
    }

    #[test]
    fn extract_tool_label_grep() {
        // Pattern is exactly 15 chars - should not be truncated
        let input = serde_json::json!({"pattern": "fn extract_tool"});
        assert_eq!(extract_tool_label("Grep", Some(&input)), Some("fn extract_tool".into()));

        // Pattern > 15 chars - should be truncated
        let input = serde_json::json!({"pattern": "fn extract_tool_label"});
        assert_eq!(extract_tool_label("Grep", Some(&input)), Some("fn extract_too\u{2026}".into()));

        // Short pattern should not be truncated
        let input = serde_json::json!({"pattern": "TODO"});
        assert_eq!(extract_tool_label("Grep", Some(&input)), Some("TODO".into()));
    }

    #[test]
    fn extract_tool_label_web() {
        let input = serde_json::json!({"url": "https://docs.rs/tokio/latest"});
        assert_eq!(extract_tool_label("WebFetch", Some(&input)), Some("docs.rs".into()));

        let input = serde_json::json!({"query": "rust async await patterns"});
        assert_eq!(extract_tool_label("WebSearch", Some(&input)), Some("rust async awa…".into()));
    }

    #[test]
    fn extract_tool_label_mcp() {
        assert_eq!(
            extract_tool_label("mcp__memory__memory_search", None),
            Some("memory_search".into())
        );
        assert_eq!(
            extract_tool_label("mcp__notion__notion-fetch", None),
            Some("notion-fetch".into())
        );
    }

    #[test]
    fn extract_tool_label_none_for_unknown() {
        let input = serde_json::json!({"some_field": "value"});
        assert_eq!(extract_tool_label("UnknownTool", Some(&input)), None);
        assert_eq!(extract_tool_label("Read", None), None);
    }
}
