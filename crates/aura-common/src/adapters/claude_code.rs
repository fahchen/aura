//! Claude Code adapter
//!
//! Parses Claude Code hook JSON and converts to AgentEvent.

use crate::{AgentEvent, AgentType};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Hook event name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookEventName {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PermissionRequest,
    Notification,
    Stop,
    SubagentStop,
    PreCompact,
    SessionEnd,
}

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
    /// Get session_id from any event
    pub fn session_id(&self) -> &str {
        match self {
            Self::SessionStart(p) => &p.common.session_id,
            Self::UserPromptSubmit(p) => &p.common.session_id,
            Self::PreToolUse(p) => &p.common.session_id,
            Self::PostToolUse(p) => &p.common.session_id,
            Self::PermissionRequest(p) => &p.common.session_id,
            Self::Notification(p) => &p.common.session_id,
            Self::Stop(p) => &p.common.session_id,
            Self::SubagentStop(p) => &p.common.session_id,
            Self::PreCompact(p) => &p.common.session_id,
            Self::SessionEnd(p) => &p.common.session_id,
        }
    }

    /// Get cwd from any event
    pub fn cwd(&self) -> &str {
        match self {
            Self::SessionStart(p) => &p.common.cwd,
            Self::UserPromptSubmit(p) => &p.common.cwd,
            Self::PreToolUse(p) => &p.common.cwd,
            Self::PostToolUse(p) => &p.common.cwd,
            Self::PermissionRequest(p) => &p.common.cwd,
            Self::Notification(p) => &p.common.cwd,
            Self::Stop(p) => &p.common.cwd,
            Self::SubagentStop(p) => &p.common.cwd,
            Self::PreCompact(p) => &p.common.cwd,
            Self::SessionEnd(p) => &p.common.cwd,
        }
    }
}

/// Parse Claude Code hook JSON into HookEvent
pub fn parse_hook(json: &str) -> Result<HookEvent, serde_json::Error> {
    serde_json::from_str(json)
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
            },
            HookEvent::PreToolUse(p) => AgentEvent::ToolStarted {
                session_id: p.common.session_id,
                tool_id: p.tool_use_id,
                tool_name: p.tool_name,
            },
            HookEvent::PostToolUse(p) => AgentEvent::ToolCompleted {
                session_id: p.common.session_id,
                tool_id: p.tool_use_id,
            },
            HookEvent::PermissionRequest(p) => AgentEvent::NeedsAttention {
                session_id: p.common.session_id,
                message: p.tool_name,
            },
            HookEvent::Notification(p) => {
                if p.notification_type.as_deref() == Some("permission_prompt") {
                    AgentEvent::NeedsAttention {
                        session_id: p.common.session_id,
                        message: p.message,
                    }
                } else {
                    AgentEvent::Activity {
                        session_id: p.common.session_id,
                    }
                }
            }
            HookEvent::Stop(p) => AgentEvent::Idle {
                session_id: p.common.session_id,
            },
            HookEvent::SubagentStop(p) => AgentEvent::Activity {
                session_id: p.common.session_id,
            },
            HookEvent::PreCompact(p) => AgentEvent::Compacting {
                session_id: p.common.session_id,
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
            "tool_use_id": "toolu_01ABC"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::ToolStarted {
                session_id,
                tool_id,
                tool_name,
            } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(tool_id, "toolu_01ABC");
                assert_eq!(tool_name, "Read");
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
            AgentEvent::NeedsAttention { session_id, message } => {
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
            AgentEvent::NeedsAttention { session_id, message } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(message, Some("Claude needs permission to run Bash".into()));
            }
            _ => panic!("Expected NeedsAttention for permission_prompt"),
        }
    }

    #[test]
    fn parse_notification_other() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "Notification",
            "notification_type": "idle_prompt"
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::Activity { session_id } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected Activity for non-permission notification"),
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
            AgentEvent::Idle { session_id } => {
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
            AgentEvent::Compacting { session_id } => {
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
            AgentEvent::Activity { session_id } => {
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
            AgentEvent::Activity { session_id } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected Activity"),
        }
    }
}
