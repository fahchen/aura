//! Claude Code adapter
//!
//! Parses Claude Code hook JSON and converts to AgentEvent.
//! Also provides transcript file discovery and parsing utilities.

use crate::transcript::{home_dir, MessageRole, TranscriptEntry, TranscriptError, TranscriptMeta};
use crate::{AgentEvent, AgentType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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

// ============================================================================
// Transcript file utilities
// ============================================================================

/// Escape a path for use as a Claude Code project directory name.
///
/// Claude Code uses `-` as a separator, replacing `/` with `-`.
/// Example: `/Users/fahchen/project` -> `-Users-fahchen-project`
pub fn escape_path(path: &str) -> String {
    // Replace all `/` with `-`
    path.replace('/', "-")
}

/// Unescape a Claude Code project directory name back to a path.
///
/// Example: `-Users-fahchen-project` -> `/Users/fahchen/project`
pub fn unescape_path(escaped: &str) -> String {
    if let Some(stripped) = escaped.strip_prefix('-') {
        // Leading `-` represents the root `/`
        format!("/{}", stripped.replace('-', "/"))
    } else {
        escaped.replace('-', "/")
    }
}

/// Get the Claude Code projects directory.
///
/// Returns `~/.claude/projects`
pub fn projects_dir() -> PathBuf {
    home_dir().join(".claude").join("projects")
}

/// Get the transcript directory for a given working directory.
///
/// Example: `/Users/fahchen/project` -> `~/.claude/projects/-Users-fahchen-project`
pub fn transcript_dir(cwd: &str) -> PathBuf {
    projects_dir().join(escape_path(cwd))
}

/// Discover all transcript files in the Claude Code projects directory.
///
/// If `since` is provided, only returns files modified after that time.
/// Files are returned sorted by modification time (newest first).
pub fn discover_transcripts(since: Option<std::time::SystemTime>) -> Vec<PathBuf> {
    let projects = projects_dir();
    if !projects.exists() {
        return Vec::new();
    }

    let mut transcripts = Vec::new();

    // Walk all project directories
    if let Ok(entries) = std::fs::read_dir(&projects) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Find all .jsonl files in the project directory
            if let Ok(files) = std::fs::read_dir(&path) {
                for file in files.flatten() {
                    let file_path = file.path();
                    if file_path.extension().is_some_and(|ext| ext == "jsonl") {
                        // Check modification time if filter is set
                        if let Some(since_time) = since
                            && let Ok(metadata) = file.metadata()
                            && let Ok(modified) = metadata.modified()
                            && modified < since_time
                        {
                            continue;
                        }
                        transcripts.push(file_path);
                    }
                }
            }
        }
    }

    // Sort by modification time (newest first)
    transcripts.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    transcripts
}

/// Parse a single line from a Claude Code transcript.
///
/// Returns `Ok(Some(entry))` for user/assistant messages,
/// `Ok(None)` for other line types (progress, system, file-history-snapshot, etc.),
/// or an error if the line is invalid JSON.
pub fn parse_transcript_line(line: &str) -> Result<Option<TranscriptEntry>, TranscriptError> {
    let value: Value = serde_json::from_str(line).map_err(|e| TranscriptError::Parse {
        line: 0,
        message: e.to_string(),
    })?;

    let entry_type = value.get("type").and_then(|v| v.as_str());

    match entry_type {
        Some("user") => {
            let timestamp = value
                .get("timestamp")
                .and_then(|v| v.as_str())
                .map(String::from);

            // Extract text content from message.content
            let text_content = value
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .map(String::from);

            Ok(Some(TranscriptEntry {
                timestamp,
                role: MessageRole::User,
                has_tool_use: false,
                tool_names: Vec::new(),
                tool_ids: Vec::new(),
                text_content,
            }))
        }
        Some("assistant") => {
            let timestamp = value
                .get("timestamp")
                .and_then(|v| v.as_str())
                .map(String::from);

            // Check for tool_use in message.content[]
            let mut tool_names = Vec::new();
            let mut tool_ids = Vec::new();
            let mut text_content = None;

            if let Some(content) = value.get("message").and_then(|m| m.get("content"))
                && let Some(content_array) = content.as_array()
            {
                for item in content_array {
                    if let Some(item_type) = item.get("type").and_then(|v| v.as_str()) {
                        if item_type == "tool_use" {
                            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                                tool_names.push(name.to_string());
                            }
                            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                                tool_ids.push(id.to_string());
                            }
                        } else if item_type == "text"
                            && let Some(text) = item.get("text").and_then(|v| v.as_str())
                        {
                            text_content = Some(text.to_string());
                        }
                    }
                }
            }

            let has_tool_use = !tool_names.is_empty();

            Ok(Some(TranscriptEntry {
                timestamp,
                role: MessageRole::Assistant,
                has_tool_use,
                tool_names,
                tool_ids,
                text_content,
            }))
        }
        // Ignore other types: progress, system, file-history-snapshot, etc.
        _ => Ok(None),
    }
}

/// Read transcript metadata by scanning the file.
///
/// Extracts session ID, cwd, git branch from the first valid message,
/// counts messages, finds the last user prompt, and tracks the last event timestamp.
pub fn read_transcript_meta(path: &Path) -> Result<TranscriptMeta, TranscriptError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut meta = TranscriptMeta::default();
    let mut last_user_prompt: Option<String> = None;

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        // Parse the line as JSON
        let value: Value =
            serde_json::from_str(&line).map_err(|e| TranscriptError::Parse {
                line: line_num + 1,
                message: e.to_string(),
            })?;

        // Track the timestamp of every line (last one wins)
        if let Some(ts) = value.get("timestamp").and_then(|v| v.as_str()) {
            meta.last_event_timestamp = Some(ts.to_string());
        }

        // Extract metadata from the first line that has sessionId
        if meta.session_id.is_none()
            && let Some(session_id) = value.get("sessionId").and_then(|v| v.as_str())
        {
            meta.session_id = Some(session_id.to_string());
        }
        if meta.cwd.is_none() && let Some(cwd) = value.get("cwd").and_then(|v| v.as_str()) {
            meta.cwd = Some(cwd.to_string());
        }
        if meta.git_branch.is_none()
            && let Some(branch) = value.get("gitBranch").and_then(|v| v.as_str())
        {
            meta.git_branch = Some(branch.to_string());
        }

        // Count messages
        let entry_type = value.get("type").and_then(|v| v.as_str());
        match entry_type {
            Some("user") => {
                meta.message_count += 1;
                meta.user_message_count += 1;

                // Extract the user prompt text
                if let Some(content) = value
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                {
                    // Skip meta messages (like bash-input, bash-stdout, etc.)
                    if !content.starts_with('<') {
                        last_user_prompt = Some(content.to_string());
                    }
                }
            }
            Some("assistant") => {
                meta.message_count += 1;
            }
            _ => {}
        }
    }

    // Truncate last user prompt
    if let Some(prompt) = last_user_prompt {
        meta.last_user_prompt = Some(TranscriptMeta::truncate_prompt(&prompt));
    }

    Ok(meta)
}

/// Extract session ID from a transcript file path.
///
/// Claude Code transcript files are named `{session-id}.jsonl`.
pub fn session_id_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Label extraction strategy for a tool
enum LabelExtractor {
    /// Extract from a field, apply transformation
    Field(&'static str, fn(&str) -> String),
    /// Extract domain from URL field
    Domain(&'static str),
}

/// Tool name to label extraction mapping
const TOOL_LABEL_EXTRACTORS: &[(&str, LabelExtractor)] = &[
    ("Bash", LabelExtractor::Field("command", extract_bash_label)),
    ("Read", LabelExtractor::Field("file_path", extract_filename)),
    ("Edit", LabelExtractor::Field("file_path", extract_filename)),
    ("Write", LabelExtractor::Field("file_path", extract_filename)),
    ("Glob", LabelExtractor::Field("pattern", str::to_string)),
    ("Grep", LabelExtractor::Field("pattern", |s| truncate_string(s, 15))),
    ("Task", LabelExtractor::Field("subagent_type", str::to_string)),
    ("WebFetch", LabelExtractor::Domain("url")),
    ("WebSearch", LabelExtractor::Field("query", |s| truncate_string(s, 15))),
];

/// Extract a human-readable label from tool_input JSON
pub fn extract_tool_label(tool_name: &str, tool_input: Option<&Value>) -> Option<String> {
    // Handle MCP tools first (they don't need input)
    if tool_name.starts_with("mcp__") {
        return Some(tool_name.rsplit("__").next().unwrap_or(tool_name).to_string());
    }

    let input = tool_input?;

    // Find matching extractor
    let (_, extractor) = TOOL_LABEL_EXTRACTORS
        .iter()
        .find(|(name, _)| *name == tool_name)?;

    match extractor {
        LabelExtractor::Field(field, transform) => input
            .get(*field)
            .and_then(|v| v.as_str())
            .map(transform),
        LabelExtractor::Domain(field) => input
            .get(*field)
            .and_then(|v| v.as_str())
            .and_then(extract_domain),
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

/// Parse `aura set-name "..."` command from a Bash command string.
///
/// Returns `Some(name)` if the command is an `aura set-name` invocation,
/// otherwise `None`.
pub fn parse_aura_set_name_command(command: &str) -> Option<String> {
    let trimmed = command.trim();
    let rest = trimmed.strip_prefix("aura set-name ")?;
    let rest = rest.trim();

    // Handle quoted strings (single or double quotes)
    if (rest.starts_with('"') && rest.ends_with('"'))
        || (rest.starts_with('\'') && rest.ends_with('\''))
    {
        if rest.len() >= 2 {
            Some(rest[1..rest.len() - 1].to_string())
        } else {
            None
        }
    } else {
        // Unquoted: use the rest as the name
        Some(rest.to_string())
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
                // Check if this is an `aura set-name` command
                if p.tool_name == "Bash"
                    && let Some(command) = p.tool_input.as_ref().and_then(|v| v.get("command")).and_then(|v| v.as_str())
                    && let Some(name) = parse_aura_set_name_command(command)
                {
                    return AgentEvent::SessionNameUpdated {
                        session_id: p.common.session_id,
                        name,
                    };
                }

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
                    Some("permission_prompt") => AgentEvent::NeedsAttention {
                        session_id: p.common.session_id,
                        cwd: p.common.cwd,
                        message: p.message,
                    },
                    Some("idle_prompt") => AgentEvent::WaitingForInput {
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
            AgentEvent::WaitingForInput { session_id, .. } => {
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected WaitingForInput for idle_prompt"),
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

    // ==================== aura set-name command parsing ====================

    #[test]
    fn parse_aura_set_name_double_quoted() {
        assert_eq!(
            parse_aura_set_name_command("aura set-name \"My Task\""),
            Some("My Task".into())
        );
    }

    #[test]
    fn parse_aura_set_name_single_quoted() {
        assert_eq!(
            parse_aura_set_name_command("aura set-name 'Implementing Feature X'"),
            Some("Implementing Feature X".into())
        );
    }

    #[test]
    fn parse_aura_set_name_unquoted() {
        assert_eq!(
            parse_aura_set_name_command("aura set-name MyTask"),
            Some("MyTask".into())
        );
    }

    #[test]
    fn parse_aura_set_name_with_whitespace() {
        assert_eq!(
            parse_aura_set_name_command("  aura set-name \"Test\"  "),
            Some("Test".into())
        );
    }

    #[test]
    fn parse_aura_set_name_not_matching() {
        assert_eq!(parse_aura_set_name_command("cargo build"), None);
        assert_eq!(parse_aura_set_name_command("aura status"), None);
        assert_eq!(parse_aura_set_name_command("echo 'aura set-name test'"), None);
    }

    #[test]
    fn pre_tool_use_aura_set_name_returns_session_name_updated() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_use_id": "toolu_01ABC",
            "tool_input": {"command": "aura set-name \"Fix Auth Bug\""}
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::SessionNameUpdated { session_id, name } => {
                assert_eq!(session_id, "abc123");
                assert_eq!(name, "Fix Auth Bug");
            }
            _ => panic!("Expected SessionNameUpdated event, got {:?}", event),
        }
    }

    #[test]
    fn pre_tool_use_regular_bash_not_affected() {
        let json = r#"{
            "session_id": "abc123",
            "cwd": "/tmp",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_use_id": "toolu_01ABC",
            "tool_input": {"command": "cargo build"}
        }"#;

        let hook = parse_hook(json).unwrap();
        let event: AgentEvent = hook.into();

        match event {
            AgentEvent::ToolStarted { tool_name, .. } => {
                assert_eq!(tool_name, "Bash");
            }
            _ => panic!("Expected ToolStarted event, got {:?}", event),
        }
    }

    // ==================== path escaping/unescaping ====================

    #[test]
    fn escape_path_absolute() {
        assert_eq!(escape_path("/Users/fahchen/project"), "-Users-fahchen-project");
        assert_eq!(escape_path("/home/user/code"), "-home-user-code");
    }

    #[test]
    fn escape_path_relative() {
        assert_eq!(escape_path("project/src"), "project-src");
    }

    #[test]
    fn unescape_path_absolute() {
        assert_eq!(unescape_path("-Users-fahchen-project"), "/Users/fahchen/project");
        assert_eq!(unescape_path("-home-user-code"), "/home/user/code");
    }

    #[test]
    fn unescape_path_relative() {
        assert_eq!(unescape_path("project-src"), "project/src");
    }

    #[test]
    fn escape_unescape_roundtrip() {
        let paths = [
            "/Users/fahchen/PersonalProjects/aura",
            "/home/user/code",
            "/tmp/test",
        ];
        for path in paths {
            assert_eq!(unescape_path(&escape_path(path)), path);
        }
    }

    // ==================== transcript line parsing ====================

    #[test]
    fn parse_transcript_user_message() {
        let line = r#"{"type":"user","message":{"role":"user","content":"hello world"},"timestamp":"2026-01-30T13:23:20.237Z","sessionId":"abc123","cwd":"/tmp"}"#;
        let entry = parse_transcript_line(line).unwrap().unwrap();

        assert_eq!(entry.role, MessageRole::User);
        assert!(!entry.has_tool_use);
        assert!(entry.tool_names.is_empty());
        assert_eq!(entry.text_content, Some("hello world".into()));
        assert_eq!(entry.timestamp, Some("2026-01-30T13:23:20.237Z".into()));
    }

    #[test]
    fn parse_transcript_assistant_message_with_text() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Here is my response"}]},"timestamp":"2026-01-30T13:23:45.368Z"}"#;
        let entry = parse_transcript_line(line).unwrap().unwrap();

        assert_eq!(entry.role, MessageRole::Assistant);
        assert!(!entry.has_tool_use);
        assert!(entry.tool_names.is_empty());
        assert_eq!(entry.text_content, Some("Here is my response".into()));
    }

    #[test]
    fn parse_transcript_assistant_message_with_tool_use() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","name":"Read","id":"toolu_01ABC","input":{"file_path":"/tmp/test.rs"}}]},"timestamp":"2026-01-30T13:23:45.368Z"}"#;
        let entry = parse_transcript_line(line).unwrap().unwrap();

        assert_eq!(entry.role, MessageRole::Assistant);
        assert!(entry.has_tool_use);
        assert_eq!(entry.tool_names, vec!["Read"]);
        assert_eq!(entry.tool_ids, vec!["toolu_01ABC"]);
    }

    #[test]
    fn parse_transcript_assistant_message_with_multiple_tools() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","name":"Read","id":"toolu_01"},{"type":"text","text":"Let me read another file"},{"type":"tool_use","name":"Grep","id":"toolu_02"}]}}"#;
        let entry = parse_transcript_line(line).unwrap().unwrap();

        assert!(entry.has_tool_use);
        assert_eq!(entry.tool_names, vec!["Read", "Grep"]);
        assert_eq!(entry.tool_ids, vec!["toolu_01", "toolu_02"]);
    }

    #[test]
    fn parse_transcript_progress_returns_none() {
        let line = r#"{"type":"progress","data":{"type":"hook_progress"}}"#;
        let result = parse_transcript_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_transcript_file_history_snapshot_returns_none() {
        let line = r#"{"type":"file-history-snapshot","messageId":"abc123"}"#;
        let result = parse_transcript_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_transcript_system_returns_none() {
        let line = r#"{"type":"system","subtype":"turn_duration","durationMs":30429}"#;
        let result = parse_transcript_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_transcript_invalid_json() {
        let line = "not valid json";
        let result = parse_transcript_line(line);
        assert!(result.is_err());
    }

    // ==================== session ID extraction ====================

    #[test]
    fn session_id_from_path_basic() {
        let path = PathBuf::from("/Users/fahchen/.claude/projects/-Users-fahchen-project/abc123-def456.jsonl");
        assert_eq!(session_id_from_path(&path), Some("abc123-def456".into()));
    }

    #[test]
    fn session_id_from_path_uuid() {
        let path = PathBuf::from("/path/to/d0b3073a-ed7b-4268-96cc-e757a30d2798.jsonl");
        assert_eq!(session_id_from_path(&path), Some("d0b3073a-ed7b-4268-96cc-e757a30d2798".into()));
    }
}
