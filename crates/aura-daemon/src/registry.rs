//! Session registry - tracks active sessions and their state

use aura_common::{AgentEvent, AgentType, RunningTool, SessionInfo, SessionState};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, trace};

/// Minimum duration to keep completed tools visible
const MIN_TOOL_DISPLAY: Duration = Duration::from_secs(1);

/// A tool that was recently completed but should remain visible briefly
#[derive(Debug, Clone)]
pub struct RecentTool {
    pub tool_name: String,
    pub tool_label: Option<String>,
    pub expires_at: Instant,
}

/// Session data tracked by the daemon
#[derive(Debug)]
pub struct Session {
    pub session_id: String,
    pub cwd: String,
    pub agent: AgentType,
    pub state: SessionState,
    pub running_tools: Vec<RunningTool>,
    pub recent_tools: Vec<RecentTool>,
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
            recent_tools: Vec::new(),
            last_activity: Instant::now(),
        }
    }

    fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Get all visible tools (running + non-expired recent)
    fn visible_tools(&self) -> Vec<RunningTool> {
        let now = Instant::now();
        let mut tools = self.running_tools.clone();

        tools.extend(
            self.recent_tools
                .iter()
                .filter(|t| t.expires_at > now)
                .map(|t| RunningTool {
                    tool_id: format!("recent_{}", t.tool_name),
                    tool_name: t.tool_name.clone(),
                    tool_label: t.tool_label.clone(),
                }),
        );

        tools
    }

    pub fn to_info(&self) -> SessionInfo {
        SessionInfo {
            session_id: self.session_id.clone(),
            cwd: self.cwd.clone(),
            state: self.state,
            running_tools: self.visible_tools(),
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

    /// Get existing session or create a new one for late-joining sessions
    fn get_or_create_session(&mut self, session_id: &str, cwd: Option<&str>) -> &mut Session {
        if !self.sessions.contains_key(session_id) {
            info!(%session_id, "late session registration");
            let session = Session::new(
                session_id.to_string(),
                cwd.unwrap_or("unknown").to_string(),
                AgentType::ClaudeCode,
            );
            self.sessions.insert(session_id.to_string(), session);
        }
        self.sessions.get_mut(session_id).unwrap()
    }

    /// Process an agent event and update session state
    pub fn process_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent,
            } => {
                info!(%session_id, %cwd, ?agent, "session started");
                let session = Session::new(session_id.clone(), cwd, agent);
                self.sessions.insert(session_id, session);
            }

            AgentEvent::Activity { session_id } => {
                trace!(%session_id, "activity");
                let session = self.get_or_create_session(&session_id, None);
                session.touch();
                // Only update state if not in Attention or Compacting
                if session.state == SessionState::Idle || session.state == SessionState::Stale {
                    session.state = SessionState::Running;
                }
            }

            AgentEvent::ToolStarted {
                session_id,
                tool_id,
                tool_name,
                tool_label,
            } => {
                debug!(%session_id, %tool_name, "tool started");
                let session = self.get_or_create_session(&session_id, None);
                session.touch();
                session.state = SessionState::Running;
                session
                    .running_tools
                    .push(RunningTool { tool_id, tool_name, tool_label });
            }

            AgentEvent::ToolCompleted {
                session_id,
                tool_id,
            } => {
                debug!(%session_id, %tool_id, "tool completed");
                let session = self.get_or_create_session(&session_id, None);
                session.touch();
                // Move to recent_tools for minimum display duration
                if let Some(pos) = session.running_tools.iter().position(|t| t.tool_id == tool_id) {
                    let tool = session.running_tools.remove(pos);
                    session.recent_tools.push(RecentTool {
                        tool_name: tool.tool_name,
                        tool_label: tool.tool_label,
                        expires_at: Instant::now() + MIN_TOOL_DISPLAY,
                    });
                }
            }

            AgentEvent::NeedsAttention {
                session_id,
                message: _,
            } => {
                info!(%session_id, "needs attention");
                let session = self.get_or_create_session(&session_id, None);
                session.touch();
                session.state = SessionState::Attention;
            }

            AgentEvent::Compacting { session_id } => {
                info!(%session_id, "compacting");
                let session = self.get_or_create_session(&session_id, None);
                session.touch();
                session.state = SessionState::Compacting;
            }

            AgentEvent::Idle { session_id } => {
                debug!(%session_id, "idle");
                let session = self.get_or_create_session(&session_id, None);
                session.touch();
                session.state = SessionState::Idle;
                session.running_tools.clear();
            }

            AgentEvent::SessionEnded { session_id } => {
                info!(%session_id, "session ended");
                self.sessions.remove(&session_id);
            }
        }
    }

    /// Mark sessions as stale if no activity for the given duration
    pub fn mark_stale(&mut self, timeout: Duration) {
        let now = Instant::now();
        for session in self.sessions.values_mut() {
            // Clean up expired recent tools
            session.recent_tools.retain(|t| t.expires_at > now);

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
            tool_label: Some("config.rs".into()),
        });
        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 1);

        // Tool completed - tool moves to recent_tools and stays visible
        registry.process_event(AgentEvent::ToolCompleted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
        });
        let sessions = registry.get_all();
        // Tool should still be visible due to MIN_TOOL_DISPLAY
        assert_eq!(sessions[0].running_tools.len(), 1);
        assert!(sessions[0].running_tools[0].tool_id.starts_with("recent_"));

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
            tool_label: Some("main.rs".into()),
        });
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "s1".into(),
            tool_id: "t2".into(),
            tool_name: "Bash".into(),
            tool_label: Some("cargo build".into()),
        });

        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 2);

        // Complete one - it moves to recent_tools but still visible
        registry.process_event(AgentEvent::ToolCompleted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
        });

        let sessions = registry.get_all();
        // Should have 2 tools: "Bash" (running) + "Read" (recent)
        assert_eq!(sessions[0].running_tools.len(), 2);
        // Find the still-running tool
        let running_bash = sessions[0]
            .running_tools
            .iter()
            .find(|t| t.tool_name == "Bash" && t.tool_id == "t2");
        assert!(running_bash.is_some());
    }

    #[test]
    fn unknown_session_auto_created() {
        let mut registry = SessionRegistry::new();

        // Events for unknown session should auto-create the session
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "late".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
            tool_label: None,
        });

        assert_eq!(registry.len(), 1);
        let sessions = registry.get_all();
        assert_eq!(sessions[0].session_id, "late");
        assert_eq!(sessions[0].cwd, "unknown");
        assert_eq!(sessions[0].running_tools.len(), 1);
    }

    #[test]
    fn minimum_tool_display_duration() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        // Start and immediately complete a tool
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
            tool_label: Some("test.rs".into()),
        });
        registry.process_event(AgentEvent::ToolCompleted {
            session_id: "s1".into(),
            tool_id: "t1".into(),
        });

        // Tool should still be visible immediately after completion
        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 1);
        assert_eq!(sessions[0].running_tools[0].tool_name, "Read");
        assert!(sessions[0].running_tools[0].tool_id.starts_with("recent_"));

        // Verify the internal recent_tools has the expiration set
        let session = registry.sessions.get("s1").unwrap();
        assert_eq!(session.running_tools.len(), 0); // actual running_tools is empty
        assert_eq!(session.recent_tools.len(), 1);
        assert!(session.recent_tools[0].expires_at > Instant::now());
    }

    #[test]
    fn recent_tools_cleanup_on_mark_stale() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        // Manually add an expired recent tool
        {
            let session = registry.sessions.get_mut("s1").unwrap();
            session.recent_tools.push(RecentTool {
                tool_name: "OldTool".into(),
                tool_label: None,
                expires_at: Instant::now() - Duration::from_secs(10),
            });
        }

        // mark_stale should clean up expired recent tools
        registry.mark_stale(Duration::from_secs(60));

        let session = registry.sessions.get("s1").unwrap();
        assert_eq!(session.recent_tools.len(), 0);
    }
}
