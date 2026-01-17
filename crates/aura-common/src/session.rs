//! Session state and tool icon definitions

use serde::{Deserialize, Serialize};

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
