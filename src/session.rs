//! Session state and tool icon definitions

use serde::{Deserialize, Serialize};

/// A currently running tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningTool {
    pub tool_id: String,
    pub tool_name: String,
    pub tool_label: Option<String>,
}

/// Session information for UI rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub cwd: String,
    pub state: SessionState,
    pub running_tools: Vec<RunningTool>,
    /// Custom session name (if set by user)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Unix timestamp when stopped
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<u64>,
    /// Unix timestamp when became stale
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_at: Option<u64>,
    /// Tool requesting permission
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_tool: Option<String>,
    /// Recent activity labels (most recent last)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_activity: Vec<String>,
}

/// Placeholder texts displayed when agent is thinking/processing
pub const PLACEHOLDER_TEXTS: &[&str] = &[
    "thinking...",
    "drafting...",
    "building...",
    "planning...",
    "analyzing...",
    "pondering...",
    "processing...",
    "reasoning...",
];

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
    /// Waiting for user input (idle_prompt)
    Waiting,
    /// Context window compacting
    Compacting,
    /// No activity for 10min+
    Stale,
}

/// SVG icon name for session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateIcon {
    Play,
    Stop,
    Bell,
    Fan,
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
            Self::Waiting => StateIcon::Fan,
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
            Self::Waiting => "#EAB308",   // Yellow (same as Attention)
            Self::Compacting => "#A855F7", // Purple
            Self::Stale => "#6B7280",     // Gray
        }
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
        assert_eq!(SessionState::Waiting.icon(), StateIcon::Fan);
        assert_eq!(SessionState::Compacting.icon(), StateIcon::Refresh);
        assert_eq!(SessionState::Stale.icon(), StateIcon::Pause);
    }

    #[test]
    fn state_colors() {
        assert_eq!(SessionState::Running.color(), "#22C55E");
        assert_eq!(SessionState::Idle.color(), "#3B82F6");
        assert_eq!(SessionState::Attention.color(), "#EAB308");
        assert_eq!(SessionState::Waiting.color(), "#EAB308");
        assert_eq!(SessionState::Compacting.color(), "#A855F7");
        assert_eq!(SessionState::Stale.color(), "#6B7280");
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
