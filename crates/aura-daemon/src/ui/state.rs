//! HUD state aggregation from session registry

use aura_common::{SessionInfo, SessionState};

/// Aggregated HUD state derived from all sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HudState {
    /// Whether any session needs attention
    pub has_attention: bool,
    /// Aggregate state (highest priority wins)
    pub aggregate_state: AggregateState,
    /// Whether to show icons (false if no sessions)
    pub visible: bool,
}

/// Aggregate state for the right icon (priority order)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AggregateState {
    /// At least one session is running
    Running,
    /// Any session is compacting (and none running)
    Compacting,
    /// All sessions are idle
    #[default]
    Idle,
    /// All sessions are stale
    Stale,
}

impl HudState {
    /// Compute HUD state from session list
    pub fn from_sessions(sessions: &[SessionInfo]) -> Self {
        if sessions.is_empty() {
            return Self {
                has_attention: false,
                aggregate_state: AggregateState::Idle,
                visible: false,
            };
        }

        let has_attention = sessions
            .iter()
            .any(|s| s.state == SessionState::Attention);

        // Priority: Running > Compacting > Idle > Stale
        let aggregate_state = if sessions.iter().any(|s| s.state == SessionState::Running) {
            AggregateState::Running
        } else if sessions.iter().any(|s| s.state == SessionState::Compacting) {
            AggregateState::Compacting
        } else if sessions.iter().any(|s| s.state == SessionState::Idle) {
            AggregateState::Idle
        } else {
            AggregateState::Stale
        };

        Self {
            has_attention,
            aggregate_state,
            visible: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_common::SessionState;

    fn session(state: SessionState) -> SessionInfo {
        SessionInfo {
            session_id: "test".into(),
            cwd: "/tmp".into(),
            state,
            running_tools: vec![],
        }
    }

    #[test]
    fn empty_sessions_not_visible() {
        let state = HudState::from_sessions(&[]);
        assert!(!state.visible);
    }

    #[test]
    fn running_highest_priority() {
        let sessions = vec![
            session(SessionState::Idle),
            session(SessionState::Running),
            session(SessionState::Stale),
        ];
        let state = HudState::from_sessions(&sessions);
        assert!(state.visible);
        assert_eq!(state.aggregate_state, AggregateState::Running);
    }

    #[test]
    fn attention_detected() {
        let sessions = vec![session(SessionState::Attention)];
        let state = HudState::from_sessions(&sessions);
        assert!(state.has_attention);
    }

    #[test]
    fn compacting_over_idle() {
        let sessions = vec![
            session(SessionState::Idle),
            session(SessionState::Compacting),
        ];
        let state = HudState::from_sessions(&sessions);
        assert_eq!(state.aggregate_state, AggregateState::Compacting);
    }
}
