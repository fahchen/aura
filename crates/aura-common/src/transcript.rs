//! Transcript parsing types and utilities
//!
//! Provides common types for parsing transcript files from various AI coding agents.
//! Each agent adapter implements parsing logic for its specific format.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when parsing transcripts
#[derive(Error, Debug)]
pub enum TranscriptError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Metadata extracted from a transcript file
#[derive(Debug, Clone, Default)]
pub struct TranscriptMeta {
    /// Session identifier (UUID format)
    pub session_id: Option<String>,
    /// Working directory for the session
    pub cwd: Option<String>,
    /// Current git branch (if available)
    pub git_branch: Option<String>,
    /// CLI version (for Codex)
    pub cli_version: Option<String>,
    /// Total number of messages in the transcript
    pub message_count: u32,
    /// Number of user messages
    pub user_message_count: u32,
    /// Last user prompt (truncated to ~100 chars)
    pub last_user_prompt: Option<String>,
    /// Timestamp of the last event in the transcript (ISO 8601 format)
    pub last_event_timestamp: Option<String>,
}

impl TranscriptMeta {
    /// Maximum length for truncated user prompts
    pub const MAX_PROMPT_LENGTH: usize = 100;

    /// Truncate a string to the maximum prompt length
    pub fn truncate_prompt(s: &str) -> String {
        if s.chars().count() <= Self::MAX_PROMPT_LENGTH {
            s.to_string()
        } else {
            format!(
                "{}...",
                s.chars()
                    .take(Self::MAX_PROMPT_LENGTH - 3)
                    .collect::<String>()
            )
        }
    }
}

/// Role of a message in the transcript
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// A single entry from a transcript
#[derive(Debug, Clone)]
pub struct TranscriptEntry {
    /// ISO timestamp string
    pub timestamp: Option<String>,
    /// Role of the message sender
    pub role: MessageRole,
    /// Whether this entry contains tool use
    pub has_tool_use: bool,
    /// Names of tools used in this entry
    pub tool_names: Vec<String>,
    /// IDs of tool uses in this entry
    pub tool_ids: Vec<String>,
    /// Text content of the message (if any)
    pub text_content: Option<String>,
}

impl Default for TranscriptEntry {
    fn default() -> Self {
        Self {
            timestamp: None,
            role: MessageRole::User,
            has_tool_use: false,
            tool_names: Vec::new(),
            tool_ids: Vec::new(),
            text_content: None,
        }
    }
}

/// Get the home directory, with fallback to current directory
pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_prompt() {
        let short = "hello world";
        assert_eq!(TranscriptMeta::truncate_prompt(short), "hello world");
    }

    #[test]
    fn truncate_long_prompt() {
        let long = "a".repeat(150);
        let truncated = TranscriptMeta::truncate_prompt(&long);
        assert!(truncated.ends_with("..."));
        assert_eq!(truncated.chars().count(), 100);
    }

    #[test]
    fn truncate_exact_length() {
        let exact = "a".repeat(100);
        let truncated = TranscriptMeta::truncate_prompt(&exact);
        assert_eq!(truncated, exact);
        assert!(!truncated.ends_with("..."));
    }

    #[test]
    fn message_role_serialization() {
        assert_eq!(
            serde_json::to_string(&MessageRole::User).unwrap(),
            "\"user\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::Assistant).unwrap(),
            "\"assistant\""
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::System).unwrap(),
            "\"system\""
        );
    }

    #[test]
    fn transcript_entry_default() {
        let entry = TranscriptEntry::default();
        assert_eq!(entry.role, MessageRole::User);
        assert!(!entry.has_tool_use);
        assert!(entry.tool_names.is_empty());
        assert!(entry.tool_ids.is_empty());
    }
}
