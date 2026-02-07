//! Agent integration modules
//!
//! Each submodule implements the client/hook handler for a specific AI coding agent.

pub mod claude_code;
pub mod codex;

/// Truncate a string to at most `max` characters (by Unicode char boundary).
pub(crate) fn truncate(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Extract the final path component (filename) from a slash-separated path.
pub(crate) fn short_path(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}
