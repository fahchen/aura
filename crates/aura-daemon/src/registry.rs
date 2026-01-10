//! Session registry - tracks active sessions and their state

use aura_common::{AgentEvent, AgentType, RunningTool, SessionInfo, SessionState};
use std::collections::HashMap;
use std::time::Instant;

/// Session data tracked by the daemon
#[derive(Debug)]
pub struct Session {
    pub session_id: String,
    pub cwd: String,
    pub agent: AgentType,
    pub state: SessionState,
    pub running_tools: Vec<RunningTool>,
    pub last_activity: Instant,
}

impl Session {
    fn new(session_id: String, cwd: String, agent: AgentType) -> Self {
        Self {
            session_id,
            cwd,
            agent,
            state: SessionState::Running,
            running_tools: Vec::new(),
            last_activity: Instant::now(),
        }
    }

    fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn to_info(&self) -> SessionInfo {
        SessionInfo {
            session_id: self.session_id.clone(),
            cwd: self.cwd.clone(),
            state: self.state,
            running_tools: self.running_tools.clone(),
        }
    }
}

/// Registry of active sessions
#[derive(Debug, Default)]
pub struct SessionRegistry {
    sessions: HashMap<String, Session>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process an agent event and update session state
    pub fn process_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent,
            } => {
                let session = Session::new(session_id.clone(), cwd, agent);
                self.sessions.insert(session_id, session);
            }

            AgentEvent::Activity { session_id } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.touch();
                    // Only update state if not in Attention or Compacting
                    if session.state == SessionState::Idle || session.state == SessionState::Stale {
                        session.state = SessionState::Running;
                    }
                }
            }

            AgentEvent::ToolStarted {
                session_id,
                tool_id,
                tool_name,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.touch();
                    session.state = SessionState::Running;
                    session.running_tools.push(RunningTool { tool_id, tool_name });
                }
            }

            AgentEvent::ToolCompleted {
                session_id,
                tool_id,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.touch();
                    session.running_tools.retain(|t| t.tool_id != tool_id);
                }
            }

            AgentEvent::NeedsAttention {
                session_id,
                message: _,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.touch();
                    session.state = SessionState::Attention;
                }
            }

            AgentEvent::Compacting { session_id } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.touch();
                    session.state = SessionState::Compacting;
                }
            }

            AgentEvent::Idle { session_id } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.touch();
                    session.state = SessionState::Idle;
                    session.running_tools.clear();
                }
            }

            AgentEvent::SessionEnded { session_id } => {
                self.sessions.remove(&session_id);
            }
        }
    }

    /// Mark sessions as stale if no activity for the given duration
    pub fn mark_stale(&mut self, timeout: std::time::Duration) {
        let now = Instant::now();
        for session in self.sessions.values_mut() {
            if now.duration_since(session.last_activity) > timeout {
                // Only mark stale if not already in a terminal state
                if session.state != SessionState::Idle {
                    session.state = SessionState::Stale;
                }
            }
        }
    }

    /// Get all sessions as SessionInfo
    pub fn get_all(&self) -> Vec<SessionInfo> {
        self.sessions.values().map(|s| s.to_info()).collect()
    }

    /// Get session count
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_lifecycle() {
        let mut registry = SessionRegistry::new();

        // Start session
        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });
        assert_eq!(registry.len(), 1);

        let sessions = registry.get_all();
        assert_eq!(sessions[0].state, SessionState::Running);

        // Tool started
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
        });
        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 1);

        // Tool completed
        registry.process_event(AgentEvent::ToolCompleted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
        });
        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 0);

        // Idle
        registry.process_event(AgentEvent::Idle {
            session_id: "s1".into(),
        });
        let sessions = registry.get_all();
        assert_eq!(sessions[0].state, SessionState::Idle);

        // End session
        registry.process_event(AgentEvent::SessionEnded {
            session_id: "s1".into(),
        });
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn attention_state() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        registry.process_event(AgentEvent::NeedsAttention {
            session_id: "s1".into(),
            message: Some("Permission needed".into()),
        });

        let sessions = registry.get_all();
        assert_eq!(sessions[0].state, SessionState::Attention);
    }

    #[test]
    fn compacting_state() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        registry.process_event(AgentEvent::Compacting {
            session_id: "s1".into(),
        });

        let sessions = registry.get_all();
        assert_eq!(sessions[0].state, SessionState::Compacting);
    }

    #[test]
    fn multiple_tools() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        // Start two tools
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
        });
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "s1".into(),
            tool_id: "t2".into(),
            tool_name: "Bash".into(),
        });

        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 2);

        // Complete one
        registry.process_event(AgentEvent::ToolCompleted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
        });

        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 1);
        assert_eq!(sessions[0].running_tools[0].tool_name, "Bash");
    }

    #[test]
    fn unknown_session_ignored() {
        let mut registry = SessionRegistry::new();

        // Events for unknown session should be ignored
        registry.process_event(AgentEvent::Activity {
            session_id: "unknown".into(),
        });
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "unknown".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
        });

        assert_eq!(registry.len(), 0);
    }
}
