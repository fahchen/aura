//! Session state and tool icon definitions

use serde::{Deserialize, Serialize};

/// Session state in the HUD
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Actively working
    #[default]
    Running,
    /// Agent finished, waiting for user
    Idle,
    /// Needs user attention (permission, etc.)
    Attention,
    /// Context window compacting
    Compacting,
    /// No activity for 60s+
    Stale,
}

/// SVG icon name for session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateIcon {
    Play,
    Stop,
    Bell,
    Refresh,
    Pause,
}

impl SessionState {
    /// Get the SVG icon for this state
    pub fn icon(&self) -> StateIcon {
        match self {
            Self::Running => StateIcon::Play,
            Self::Idle => StateIcon::Stop,
            Self::Attention => StateIcon::Bell,
            Self::Compacting => StateIcon::Refresh,
            Self::Stale => StateIcon::Pause,
        }
    }

    /// Hex color for this state
    pub fn color(&self) -> &'static str {
        match self {
            Self::Running => "#22C55E",   // Green
            Self::Idle => "#3B82F6",      // Blue
            Self::Attention => "#EAB308", // Yellow
            Self::Compacting => "#A855F7", // Purple
            Self::Stale => "#6B7280",     // Gray
        }
    }
}

/// SVG icon name for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolIcon {
    Robot,
    Terminal,
    Folder,
    Search,
    Book,
    Pencil,
    File,
    Globe,
    Plug,
    Gear,
}

/// Get SVG icon for a tool name
pub fn tool_icon(tool_name: &str) -> ToolIcon {
    match tool_name {
        "Task" => ToolIcon::Robot,
        "Bash" => ToolIcon::Terminal,
        "Glob" => ToolIcon::Folder,
        "Grep" => ToolIcon::Search,
        "Read" => ToolIcon::Book,
        "Edit" => ToolIcon::Pencil,
        "Write" => ToolIcon::File,
        "WebFetch" | "WebSearch" => ToolIcon::Globe,
        name if name.starts_with("mcp__") => ToolIcon::Plug,
        _ => ToolIcon::Gear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_icons() {
        assert_eq!(SessionState::Running.icon(), StateIcon::Play);
        assert_eq!(SessionState::Idle.icon(), StateIcon::Stop);
        assert_eq!(SessionState::Attention.icon(), StateIcon::Bell);
        assert_eq!(SessionState::Compacting.icon(), StateIcon::Refresh);
        assert_eq!(SessionState::Stale.icon(), StateIcon::Pause);
    }

    #[test]
    fn state_colors() {
        assert_eq!(SessionState::Running.color(), "#22C55E");
        assert_eq!(SessionState::Idle.color(), "#3B82F6");
        assert_eq!(SessionState::Attention.color(), "#EAB308");
        assert_eq!(SessionState::Compacting.color(), "#A855F7");
        assert_eq!(SessionState::Stale.color(), "#6B7280");
    }

    #[test]
    fn tool_icons() {
        assert_eq!(tool_icon("Task"), ToolIcon::Robot);
        assert_eq!(tool_icon("Bash"), ToolIcon::Terminal);
        assert_eq!(tool_icon("Glob"), ToolIcon::Folder);
        assert_eq!(tool_icon("Grep"), ToolIcon::Search);
        assert_eq!(tool_icon("Read"), ToolIcon::Book);
        assert_eq!(tool_icon("Edit"), ToolIcon::Pencil);
        assert_eq!(tool_icon("Write"), ToolIcon::File);
        assert_eq!(tool_icon("WebFetch"), ToolIcon::Globe);
        assert_eq!(tool_icon("WebSearch"), ToolIcon::Globe);
        assert_eq!(tool_icon("mcp__notion__search"), ToolIcon::Plug);
        assert_eq!(tool_icon("mcp__memory__add"), ToolIcon::Plug);
        assert_eq!(tool_icon("UnknownTool"), ToolIcon::Gear);
    }

    #[test]
    fn state_default() {
        assert_eq!(SessionState::default(), SessionState::Running);
    }

    #[test]
    fn state_serialization() {
        assert_eq!(
            serde_json::to_string(&SessionState::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&SessionState::Attention).unwrap(),
            "\"attention\""
        );
    }
}
