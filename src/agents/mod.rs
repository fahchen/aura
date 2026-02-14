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

/// Parse `aura set-name "..."` from a shell command string.
///
/// Supports double quotes, single quotes, and unquoted names (including multi-word).
/// Returns the extracted name, or `None` if the command is not an `aura set-name` invocation.
pub(crate) fn parse_aura_set_name_command(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_set_name_double_quotes() {
        assert_eq!(
            parse_aura_set_name_command("aura set-name \"fix login bug\""),
            Some("fix login bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_single_quotes() {
        assert_eq!(
            parse_aura_set_name_command("aura set-name 'fix login bug'"),
            Some("fix login bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_no_quotes() {
        assert_eq!(
            parse_aura_set_name_command("aura set-name fix-login-bug"),
            Some("fix-login-bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_extra_whitespace() {
        assert_eq!(
            parse_aura_set_name_command("aura  set-name  \"fix login bug\""),
            Some("fix login bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_not_matching() {
        assert_eq!(parse_aura_set_name_command("echo hello"), None);
    }

    #[test]
    fn parse_set_name_relative_path() {
        assert_eq!(
            parse_aura_set_name_command("./aura set-name \"fix bug\""),
            Some("fix bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_absolute_path() {
        assert_eq!(
            parse_aura_set_name_command("/usr/local/bin/aura set-name \"fix bug\""),
            Some("fix bug".to_string())
        );
    }

    #[test]
    fn parse_set_name_empty_rejected() {
        assert_eq!(parse_aura_set_name_command("aura set-name \"\""), None);
    }

    #[test]
    fn parse_set_name_missing_arg_rejected() {
        assert_eq!(parse_aura_set_name_command("aura set-name"), None);
    }
}
