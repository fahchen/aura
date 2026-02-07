//! Session registry - tracks active sessions and their state

use aura_common::{AgentEvent, AgentType, RunningTool, SessionInfo, SessionState};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tracing::{debug, info, trace};

/// Convert an Instant to a Unix timestamp (seconds since epoch)
fn instant_to_unix_timestamp(instant: Instant) -> u64 {
    let elapsed = Instant::now().saturating_duration_since(instant);
    let system_time = std::time::SystemTime::now() - elapsed;
    system_time
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Minimum duration to keep completed tools visible
const MIN_TOOL_DISPLAY: Duration = Duration::from_secs(1);
/// Maximum number of recent activity items to keep
const RECENT_ACTIVITY_MAX: usize = 6;

/// Prefix for recent tool IDs in the visible tools list
const RECENT_TOOL_PREFIX: &str = "recent_";

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
    pub recent_activity: VecDeque<String>,
    pub last_activity: Instant,
    /// Custom session name (if set by user via `aura set-name`)
    pub name: Option<String>,
    /// When the session became idle
    pub stopped_at: Option<Instant>,
    /// When the session became stale
    pub stale_at: Option<Instant>,
    /// Tool requesting permission (from NeedsAttention message)
    pub permission_tool: Option<String>,
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
            recent_activity: VecDeque::new(),
            last_activity: Instant::now(),
            name: None,
            stopped_at: None,
            stale_at: None,
            permission_tool: None,
        }
    }

    /// Clear timestamp fields when transitioning to Running state
    fn clear_timestamps(&mut self) {
        self.stopped_at = None;
        self.stale_at = None;
        self.permission_tool = None;
    }

    fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    fn push_recent_activity(&mut self, label: String) {
        if label.is_empty() {
            return;
        }
        if self.recent_activity.back().is_some_and(|last| last == &label) {
            return;
        }
        if let Some(pos) = self
            .recent_activity
            .iter()
            .position(|existing| existing == &label)
        {
            self.recent_activity.remove(pos);
        }
        self.recent_activity.push_back(label);
        while self.recent_activity.len() > RECENT_ACTIVITY_MAX {
            self.recent_activity.pop_front();
        }
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
                    tool_id: format!("{}{}", RECENT_TOOL_PREFIX, t.tool_name),
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
            name: self.name.clone(),
            stopped_at: self.stopped_at.map(instant_to_unix_timestamp),
            stale_at: self.stale_at.map(instant_to_unix_timestamp),
            permission_tool: self.permission_tool.clone(),
            git_branch: None,
            message_count: None,
            last_prompt_preview: None,
            recent_activity: self.recent_activity.iter().cloned().collect(),
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

    /// Get existing session or create a new one (late registration), update it, and touch.
    fn update_session<F>(&mut self, session_id: &str, cwd: &str, updater: F)
    where
        F: FnOnce(&mut Session),
    {
        let session = self.sessions.entry(session_id.to_string()).or_insert_with(|| {
            info!(%session_id, %cwd, "late session registration");
            Session::new(session_id.to_string(), cwd.to_string(), AgentType::ClaudeCode)
        });
        session.touch();
        updater(session);
    }

    /// Process an agent event and update session state
    pub fn process_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    // Session already exists (e.g., subagent transcript discovered).
                    // Update metadata but keep tool/state history.
                    session.cwd = cwd;
                    session.agent = agent;
                    session.touch();
                } else {
                    info!(%session_id, %cwd, ?agent, "session started");
                    self.sessions
                        .insert(session_id.clone(), Session::new(session_id, cwd, agent));
                    debug!("{} total session(s)", self.sessions.len());
                }
            }

            AgentEvent::Activity { session_id, cwd } => {
                trace!(%session_id, "activity");
                self.update_session(&session_id, &cwd, |session| {
                    if session.state == SessionState::Idle || session.state == SessionState::Stale {
                        session.state = SessionState::Running;
                        session.clear_timestamps();
                    }
                });
            }

            AgentEvent::ToolStarted {
                session_id,
                cwd,
                tool_id,
                tool_name,
                tool_label,
            } => {
                debug!(%session_id, %tool_name, "tool started");
                self.update_session(&session_id, &cwd, |session| {
                    session.state = SessionState::Running;
                    session.clear_timestamps();
                    session.permission_tool = None;
                    session
                        .running_tools
                        .push(RunningTool { tool_id, tool_name, tool_label });
                });
            }

            AgentEvent::ToolCompleted {
                session_id,
                cwd,
                tool_id,
            } => {
                debug!(%session_id, %tool_id, "tool completed");
                self.update_session(&session_id, &cwd, |session| {
                    if session.state != SessionState::Running {
                        session.state = SessionState::Running;
                        session.clear_timestamps();
                        session.permission_tool = None;
                    }
                    if let Some(pos) =
                        session.running_tools.iter().position(|t| t.tool_id == tool_id)
                    {
                        let tool = session.running_tools.remove(pos);
                        let label = tool
                            .tool_label
                            .clone()
                            .unwrap_or_else(|| tool.tool_name.clone());
                        session.recent_tools.push(RecentTool {
                            tool_name: tool.tool_name,
                            tool_label: tool.tool_label,
                            expires_at: Instant::now() + MIN_TOOL_DISPLAY,
                        });
                        session.push_recent_activity(label);
                    }
                });
            }

            AgentEvent::NeedsAttention {
                session_id,
                cwd,
                message,
            } => {
                info!(%session_id, "needs attention");
                self.update_session(&session_id, &cwd, |session| {
                    session.state = SessionState::Attention;
                    session.permission_tool = message;
                });
            }

            AgentEvent::WaitingForInput {
                session_id,
                cwd,
                message: _,
            } => {
                info!(%session_id, "waiting for input");
                self.update_session(&session_id, &cwd, |session| {
                    session.state = SessionState::Waiting;
                });
            }

            AgentEvent::Compacting { session_id, cwd } => {
                info!(%session_id, "compacting");
                self.update_session(&session_id, &cwd, |session| {
                    session.state = SessionState::Compacting;
                });
            }

            AgentEvent::Idle { session_id, cwd } => {
                debug!(%session_id, "idle");
                self.update_session(&session_id, &cwd, |session| {
                    session.state = SessionState::Idle;
                    session.running_tools.clear();
                    session.stopped_at = Some(Instant::now());
                    session.permission_tool = None;
                });
            }

            AgentEvent::SessionEnded { session_id } => {
                info!(%session_id, "session ended");
                self.sessions.remove(&session_id);
                debug!("{} total session(s)", self.sessions.len());
            }

            AgentEvent::SessionNameUpdated { session_id, name } => {
                info!(%session_id, %name, "session name updated");
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.name = Some(name);
                    session.touch();
                }
            }
        }
    }

    /// Mark sessions as stale if no activity for the given duration
    pub fn mark_stale(&mut self, timeout: Duration) {
        let now = Instant::now();
        let mut remove_ids = Vec::new();
        for session in self.sessions.values_mut() {
            // Clean up expired recent tools
            session.recent_tools.retain(|t| t.expires_at > now);

            if now.duration_since(session.last_activity) > timeout {
                // Only mark stale if not already in a terminal state
                if session.state != SessionState::Idle
                    && session.state != SessionState::Waiting
                    && session.state != SessionState::Stale
                {
                    session.state = SessionState::Stale;
                    session.stale_at = Some(Instant::now());
                }

                // Codex sessions do not emit explicit end events reliably.
                // Remove them once they go stale to keep the list accurate.
                if session.agent == AgentType::Codex {
                    remove_ids.push(session.session_id.clone());
                }
            }
        }

        for session_id in remove_ids {
            self.sessions.remove(&session_id);
        }
    }

    /// Get all sessions as SessionInfo
    pub fn get_all(&self) -> Vec<SessionInfo> {
        self.sessions.values().map(|s| s.to_info()).collect()
    }

    /// Remove a session by ID (used by UI when clicking the remove button)
    pub fn remove_session(&mut self, session_id: &str) {
        info!(%session_id, "session removed via UI");
        self.sessions.remove(session_id);
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
            cwd: "/tmp".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
            tool_label: Some("config.rs".into()),
        });
        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 1);

        // Tool completed - tool moves to recent_tools and stays visible
        registry.process_event(AgentEvent::ToolCompleted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            tool_id: "t1".into(),
        });
        let sessions = registry.get_all();
        // Tool should still be visible due to MIN_TOOL_DISPLAY
        assert_eq!(sessions[0].running_tools.len(), 1);
        assert!(sessions[0].running_tools[0].tool_id.starts_with(RECENT_TOOL_PREFIX));

        // Idle
        registry.process_event(AgentEvent::Idle {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
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
            cwd: "/tmp".into(),
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
            cwd: "/tmp".into(),
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
            cwd: "/tmp".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
            tool_label: Some("main.rs".into()),
        });
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            tool_id: "t2".into(),
            tool_name: "Bash".into(),
            tool_label: Some("cargo build".into()),
        });

        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 2);

        // Complete one - it moves to recent_tools but still visible
        registry.process_event(AgentEvent::ToolCompleted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
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
    fn late_session_registration() {
        let mut registry = SessionRegistry::new();

        // Events for unknown session should auto-create the session
        registry.process_event(AgentEvent::ToolStarted {
            session_id: "late".into(),
            cwd: "/tmp".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
            tool_label: None,
        });

        assert_eq!(registry.len(), 1);
        let sessions = registry.get_all();
        assert_eq!(sessions[0].session_id, "late");
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
            cwd: "/tmp".into(),
            tool_id: "t1".into(),
            tool_name: "Read".into(),
            tool_label: Some("test.rs".into()),
        });
        registry.process_event(AgentEvent::ToolCompleted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            tool_id: "t1".into(),
        });

        // Tool should still be visible immediately after completion
        let sessions = registry.get_all();
        assert_eq!(sessions[0].running_tools.len(), 1);
        assert_eq!(sessions[0].running_tools[0].tool_name, "Read");
        assert!(sessions[0].running_tools[0].tool_id.starts_with(RECENT_TOOL_PREFIX));

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

    #[test]
    fn stopped_at_timestamp_set_on_idle() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        // Initially no stopped_at
        let session = registry.sessions.get("s1").unwrap();
        assert!(session.stopped_at.is_none());

        // Go idle
        registry.process_event(AgentEvent::Idle {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
        });

        let session = registry.sessions.get("s1").unwrap();
        assert!(session.stopped_at.is_some());

        // Verify to_info() converts to Unix timestamp
        let info = session.to_info();
        assert!(info.stopped_at.is_some());
        assert!(info.stopped_at.unwrap() > 0);
    }

    #[test]
    fn permission_tool_set_on_needs_attention() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        registry.process_event(AgentEvent::NeedsAttention {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            message: Some("Bash".into()),
        });

        let session = registry.sessions.get("s1").unwrap();
        assert_eq!(session.permission_tool, Some("Bash".into()));

        let info = session.to_info();
        assert_eq!(info.permission_tool, Some("Bash".into()));
    }

    #[test]
    fn timestamps_cleared_on_running() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        // Go idle to set stopped_at
        registry.process_event(AgentEvent::Idle {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
        });

        let session = registry.sessions.get("s1").unwrap();
        assert!(session.stopped_at.is_some());

        // Activity should clear timestamps
        registry.process_event(AgentEvent::Activity {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
        });

        let session = registry.sessions.get("s1").unwrap();
        assert!(session.stopped_at.is_none());
        assert!(session.stale_at.is_none());
        assert!(session.permission_tool.is_none());
    }

    #[test]
    fn stale_at_timestamp_set_on_mark_stale() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        // Manually set last_activity to the past
        {
            let session = registry.sessions.get_mut("s1").unwrap();
            session.last_activity = Instant::now() - Duration::from_secs(120);
        }

        // Initially no stale_at
        let session = registry.sessions.get("s1").unwrap();
        assert!(session.stale_at.is_none());

        // mark_stale with short timeout
        registry.mark_stale(Duration::from_secs(60));

        let session = registry.sessions.get("s1").unwrap();
        assert_eq!(session.state, SessionState::Stale);
        assert!(session.stale_at.is_some());

        // Verify to_info() converts to Unix timestamp
        let info = session.to_info();
        assert!(info.stale_at.is_some());
        assert!(info.stale_at.unwrap() > 0);
    }

    #[test]
    fn waiting_state() {
        let mut registry = SessionRegistry::new();

        registry.process_event(AgentEvent::SessionStarted {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            agent: AgentType::ClaudeCode,
        });

        registry.process_event(AgentEvent::WaitingForInput {
            session_id: "s1".into(),
            cwd: "/tmp".into(),
            message: None,
        });

        let sessions = registry.get_all();
        assert_eq!(sessions[0].state, SessionState::Waiting);
    }
}
