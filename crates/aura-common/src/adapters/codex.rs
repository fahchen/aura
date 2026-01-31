//! Codex adapter - transcript parsing for Codex CLI
//!
//! Codex stores transcripts in `~/.codex/sessions/YYYY/MM/DD/rollout-{timestamp}-{session-id}.jsonl`

use crate::transcript::{home_dir, MessageRole, TranscriptEntry, TranscriptError, TranscriptMeta};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Get the Codex sessions directory.
///
/// Returns `~/.codex/sessions`
pub fn sessions_dir() -> PathBuf {
    home_dir().join(".codex").join("sessions")
}

/// Discover session files in the Codex sessions directory.
///
/// Walks YYYY/MM/DD directories to find `rollout-*.jsonl` files.
/// If `since` is provided, only returns files modified after that time.
/// Files are returned sorted by modification time (newest first).
pub fn discover_sessions(since: Option<std::time::SystemTime>) -> Vec<PathBuf> {
    let sessions = sessions_dir();
    if !sessions.exists() {
        return Vec::new();
    }

    let mut transcripts = Vec::new();

    // Walk YYYY directories
    if let Ok(years) = std::fs::read_dir(&sessions) {
        for year in years.flatten() {
            let year_path = year.path();
            if !year_path.is_dir() {
                continue;
            }

            // Walk MM directories
            if let Ok(months) = std::fs::read_dir(&year_path) {
                for month in months.flatten() {
                    let month_path = month.path();
                    if !month_path.is_dir() {
                        continue;
                    }

                    // Walk DD directories
                    if let Ok(days) = std::fs::read_dir(&month_path) {
                        for day in days.flatten() {
                            let day_path = day.path();
                            if !day_path.is_dir() {
                                continue;
                            }

                            // Find rollout-*.jsonl files
                            if let Ok(files) = std::fs::read_dir(&day_path) {
                                for file in files.flatten() {
                                    let file_path = file.path();
                                    let filename = file_path
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("");

                                    if filename.starts_with("rollout-")
                                        && filename.ends_with(".jsonl")
                                    {
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

/// Extract session ID from a Codex session filename.
///
/// Filename format: `rollout-{timestamp}-{session-id}.jsonl`
/// Example: `rollout-2025-08-10T12-50-53-a3953a61-af96-4bfc-8a05-f8355309f025.jsonl`
///
/// Returns the session ID (UUID) portion.
pub fn session_id_from_filename(filename: &str) -> Option<String> {
    // Remove the extension
    let stem = filename.strip_suffix(".jsonl")?;

    // Remove the "rollout-" prefix
    let rest = stem.strip_prefix("rollout-")?;

    // The format is: YYYY-MM-DDTHH-MM-SS-{uuid}
    // The timestamp is 19 chars (2025-08-10T12-50-53) plus the hyphen separator
    // UUID is 36 chars (a3953a61-af96-4bfc-8a05-f8355309f025)

    // Find the UUID by looking for the pattern after the timestamp
    // The timestamp format: YYYY-MM-DDTHH-MM-SS (19 chars)
    if rest.len() < 20 {
        return None;
    }

    // Skip the timestamp and the separator hyphen
    let uuid_start = 20; // 19 chars + 1 hyphen
    if rest.len() <= uuid_start {
        return None;
    }

    Some(rest[uuid_start..].to_string())
}

/// Parse a line from a Codex transcript.
///
/// Codex has different line types:
/// - First line: session metadata (id, timestamp, git info, etc.)
/// - `type: "message"` with `role: "user"` - user messages
/// - `type: "message"` with `role: "assistant"` - assistant messages
/// - `type: "function_call"` - tool invocations
/// - `type: "function_call_output"` - tool results
/// - `type: "reasoning"` - thinking/reasoning
/// - `record_type: "state"` - state markers (ignore)
///
/// Returns `Ok(Some(entry))` for user/assistant messages,
/// `Ok(None)` for other line types.
pub fn parse_transcript_line(line: &str) -> Result<Option<TranscriptEntry>, TranscriptError> {
    let value: Value = serde_json::from_str(line).map_err(|e| TranscriptError::Parse {
        line: 0,
        message: e.to_string(),
    })?;

    // Skip state markers
    if value.get("record_type").is_some() {
        return Ok(None);
    }

    let entry_type = value.get("type").and_then(|v| v.as_str());

    match entry_type {
        Some("message") => {
            let role = value.get("role").and_then(|v| v.as_str());

            let message_role = match role {
                Some("user") => MessageRole::User,
                Some("assistant") => MessageRole::Assistant,
                Some("system") => MessageRole::System,
                _ => return Ok(None),
            };

            // Extract text content from content array
            let mut text_content = None;
            let mut tool_names = Vec::new();
            let mut tool_ids = Vec::new();

            if let Some(content) = value.get("content")
                && let Some(content_array) = content.as_array()
            {
                for item in content_array {
                    if let Some(item_type) = item.get("type").and_then(|v| v.as_str()) {
                        match item_type {
                            "input_text" | "output_text" => {
                                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                                    text_content = Some(text.to_string());
                                }
                            }
                            "tool_use" => {
                                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                                    tool_names.push(name.to_string());
                                }
                                if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                                    tool_ids.push(id.to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(Some(TranscriptEntry {
                timestamp: None,
                role: message_role,
                has_tool_use: !tool_names.is_empty(),
                tool_names,
                tool_ids,
                text_content,
            }))
        }
        Some("function_call") => {
            // Extract tool name and ID from function calls
            let name = value
                .get("name")
                .and_then(|v| v.as_str())
                .map(String::from);
            let id = value.get("id").and_then(|v| v.as_str()).map(String::from);

            Ok(Some(TranscriptEntry {
                timestamp: None,
                role: MessageRole::Assistant,
                has_tool_use: true,
                tool_names: name.into_iter().collect(),
                tool_ids: id.into_iter().collect(),
                text_content: None,
            }))
        }
        // Skip reasoning, function_call_output, etc.
        _ => Ok(None),
    }
}

/// Parse the first line of a Codex session file to extract session metadata.
///
/// The first line contains: id, timestamp, git info (commit_hash, branch), instructions.
pub fn parse_session_meta_line(line: &str) -> Result<Option<TranscriptMeta>, TranscriptError> {
    let value: Value = serde_json::from_str(line).map_err(|e| TranscriptError::Parse {
        line: 1,
        message: e.to_string(),
    })?;

    // The first line has id, timestamp, git info directly
    let session_id = value.get("id").and_then(|v| v.as_str()).map(String::from);

    // Git info is nested
    let git_branch = value
        .get("git")
        .and_then(|g| g.get("branch"))
        .and_then(|v| v.as_str())
        .map(String::from);

    // No cwd or cli_version in the first line of Codex sessions
    // These might be in other lines

    Ok(Some(TranscriptMeta {
        session_id,
        cwd: None,
        git_branch,
        cli_version: None,
        message_count: 0,
        user_message_count: 0,
        last_user_prompt: None,
        last_event_timestamp: None,
    }))
}

/// Read transcript metadata from a Codex session file.
///
/// Scans the file to extract session info and count messages.
pub fn read_transcript_meta(path: &Path) -> Result<TranscriptMeta, TranscriptError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut meta = TranscriptMeta::default();
    let mut last_user_prompt: Option<String> = None;
    let mut is_first_line = true;

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

        // First line has session metadata
        if is_first_line {
            is_first_line = false;
            if let Some(session_id) = value.get("id").and_then(|v| v.as_str()) {
                meta.session_id = Some(session_id.to_string());
            }
            if let Some(branch) = value
                .get("git")
                .and_then(|g| g.get("branch"))
                .and_then(|v| v.as_str())
            {
                meta.git_branch = Some(branch.to_string());
            }
            continue;
        }

        // Skip state markers
        if value.get("record_type").is_some() {
            continue;
        }

        // Count messages
        let entry_type = value.get("type").and_then(|v| v.as_str());
        if let Some("message") = entry_type {
            let role = value.get("role").and_then(|v| v.as_str());
            match role {
                Some("user") => {
                    meta.message_count += 1;
                    meta.user_message_count += 1;

                    // Extract user prompt text
                    if let Some(content) = value.get("content")
                        && let Some(content_array) = content.as_array()
                    {
                        for item in content_array {
                            if item.get("type").and_then(|v| v.as_str()) == Some("input_text")
                                && let Some(text) = item.get("text").and_then(|v| v.as_str())
                            {
                                last_user_prompt = Some(text.to_string());
                            }
                        }
                    }
                }
                Some("assistant") => {
                    meta.message_count += 1;
                }
                _ => {}
            }
        }
    }

    // Truncate last user prompt
    if let Some(prompt) = last_user_prompt {
        meta.last_user_prompt = Some(TranscriptMeta::truncate_prompt(&prompt));
    }

    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_from_filename_valid() {
        let filename = "rollout-2025-08-10T12-50-53-a3953a61-af96-4bfc-8a05-f8355309f025.jsonl";
        assert_eq!(
            session_id_from_filename(filename),
            Some("a3953a61-af96-4bfc-8a05-f8355309f025".into())
        );
    }

    #[test]
    fn session_id_from_filename_different_timestamp() {
        let filename = "rollout-2025-08-15T23-28-48-253cd330-f205-4bf4-854c-a74017c1378f.jsonl";
        assert_eq!(
            session_id_from_filename(filename),
            Some("253cd330-f205-4bf4-854c-a74017c1378f".into())
        );
    }

    #[test]
    fn session_id_from_filename_invalid() {
        assert_eq!(session_id_from_filename("not-a-rollout.jsonl"), None);
        assert_eq!(session_id_from_filename("rollout-.jsonl"), None);
        assert_eq!(session_id_from_filename(""), None);
    }

    #[test]
    fn parse_codex_user_message() {
        let line = r#"{"type":"message","id":null,"role":"user","content":[{"type":"input_text","text":"Generate a file named AGENTS.md"}]}"#;
        let entry = parse_transcript_line(line).unwrap().unwrap();

        assert_eq!(entry.role, MessageRole::User);
        assert!(!entry.has_tool_use);
        assert_eq!(
            entry.text_content,
            Some("Generate a file named AGENTS.md".into())
        );
    }

    #[test]
    fn parse_codex_assistant_message() {
        let line = r#"{"type":"message","id":"msg_123","role":"assistant","content":[{"type":"output_text","text":"I'll help you with that"}]}"#;
        let entry = parse_transcript_line(line).unwrap().unwrap();

        assert_eq!(entry.role, MessageRole::Assistant);
        assert!(!entry.has_tool_use);
        assert_eq!(entry.text_content, Some("I'll help you with that".into()));
    }

    #[test]
    fn parse_codex_function_call() {
        let line = r#"{"type":"function_call","id":"fc_123","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"ls -la\"]}"}"#;
        let entry = parse_transcript_line(line).unwrap().unwrap();

        assert_eq!(entry.role, MessageRole::Assistant);
        assert!(entry.has_tool_use);
        assert_eq!(entry.tool_names, vec!["shell"]);
        assert_eq!(entry.tool_ids, vec!["fc_123"]);
    }

    #[test]
    fn parse_codex_state_marker_returns_none() {
        let line = r#"{"record_type":"state"}"#;
        let result = parse_transcript_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_codex_reasoning_returns_none() {
        let line = r#"{"type":"reasoning","id":"rs_123","summary":[]}"#;
        let result = parse_transcript_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_codex_function_call_output_returns_none() {
        let line = r#"{"type":"function_call_output","call_id":"call_123","output":"{}"}"#;
        let result = parse_transcript_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_session_meta_line_valid() {
        let line = r#"{"id":"a3953a61-af96-4bfc-8a05-f8355309f025","timestamp":"2025-08-10T12:50:53.445Z","instructions":null,"git":{"commit_hash":"99a1948543c419e6147fce6303c0e5bdcf03d0ee","branch":"main"}}"#;
        let meta = parse_session_meta_line(line).unwrap().unwrap();

        assert_eq!(
            meta.session_id,
            Some("a3953a61-af96-4bfc-8a05-f8355309f025".into())
        );
        assert_eq!(meta.git_branch, Some("main".into()));
    }

    #[test]
    fn parse_session_meta_line_invalid_json() {
        let line = "not valid json";
        let result = parse_session_meta_line(line);
        assert!(result.is_err());
    }
}
