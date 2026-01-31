//! File watcher for transcript directories
//!
//! Watches Claude Code and Codex transcript directories for changes
//! and emits events when sessions are created/updated/removed.
//!
//! # Architecture
//!
//! The watcher uses the `notify` crate for cross-platform file system notifications.
//! It maintains internal state to track known files and their modification times,
//! using debouncing to batch rapid file changes (common during active sessions).
//!
//! The watcher is designed to be polled from an async context - call `poll()`
//! periodically to process pending events.

use aura_common::{
    adapters::{claude_code, codex},
    transcript::TranscriptMeta,
    AgentType,
};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info, trace, warn};

/// Debounce window for file events.
/// Multiple rapid changes to the same file within this window are batched.
const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// Minimum time between full directory scans.
/// Full scans catch files that may have been missed by the watcher (e.g., created
/// before the watcher was initialized).
const SCAN_INTERVAL: Duration = Duration::from_secs(30);

/// State tracking for a transcript file
#[derive(Debug)]
struct FileState {
    /// Last known modification time
    mtime: SystemTime,
    /// Session ID extracted from this file
    session_id: String,
    /// Agent type (Claude Code or Codex)
    #[allow(dead_code)]
    agent: AgentType,
    /// CWD from transcript metadata
    #[allow(dead_code)]
    cwd: Option<String>,
    /// Last parsed metadata
    #[allow(dead_code)]
    meta: Option<TranscriptMeta>,
    /// Whether we've emitted SessionStarted for this
    #[allow(dead_code)]
    started: bool,
}

/// How recently a file must be modified to be considered "active" (10 minutes)
const ACTIVE_THRESHOLD_SECS: u64 = 600;

/// Parse an ISO 8601 timestamp string into SystemTime.
///
/// Supports formats like:
/// - "2026-01-31T12:45:31.053Z" (with milliseconds)
/// - "2026-01-31T12:45:31Z" (without milliseconds)
/// - "2026-01-31T12:45:31.053+00:00" (with timezone offset)
fn parse_iso_timestamp(ts: &str) -> Option<SystemTime> {
    // Try to parse the timestamp using a simple approach
    // Format: YYYY-MM-DDTHH:MM:SS[.mmm][Z|+HH:MM]

    // Remove the timezone suffix and get the core datetime
    let ts = ts.trim();

    // Handle 'Z' suffix (UTC)
    let ts = ts.strip_suffix('Z').unwrap_or(ts);

    // Handle timezone offset like +00:00 or -05:00
    let ts = if let Some(pos) = ts.rfind('+') {
        &ts[..pos]
    } else if let Some(pos) = ts.rfind('-') {
        // Only strip if it's a timezone offset (after time portion)
        // Date dashes are at positions 4 and 7
        if pos > 10 {
            &ts[..pos]
        } else {
            ts
        }
    } else {
        ts
    };

    // Split into date and time
    let parts: Vec<&str> = ts.split('T').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    if date_parts.len() != 3 {
        return None;
    }

    let year: i32 = date_parts[0].parse().ok()?;
    let month: u32 = date_parts[1].parse().ok()?;
    let day: u32 = date_parts[2].parse().ok()?;

    // Parse time, handling optional milliseconds
    let time_str = parts[1];
    let (time_main, _millis) = if let Some(dot_pos) = time_str.find('.') {
        let main = &time_str[..dot_pos];
        let millis_str = &time_str[dot_pos + 1..];
        let millis: u32 = millis_str.parse().unwrap_or(0);
        (main, millis)
    } else {
        (time_str, 0u32)
    };

    let time_parts: Vec<&str> = time_main.split(':').collect();
    if time_parts.len() < 2 {
        return None;
    }

    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = time_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    // Convert to Unix timestamp (simplified calculation assuming UTC)
    // Days from year 1970 to target year
    let mut days: i64 = 0;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Days from months in target year
    let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += days_in_months[(m - 1) as usize] as i64;
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }

    // Add day of month (1-indexed)
    days += (day - 1) as i64;

    // Convert to seconds
    let total_secs = days * 86400 + (hour as i64) * 3600 + (minute as i64) * 60 + (second as i64);

    if total_secs < 0 {
        return None;
    }

    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(total_secs as u64))
}

/// Check if a year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Events emitted by the watcher
#[derive(Debug, Clone)]
pub enum WatcherEvent {
    /// A new or updated session was detected
    SessionUpdate {
        session_id: String,
        cwd: String,
        agent: AgentType,
        meta: TranscriptMeta,
        /// True if the file was modified recently (within ACTIVE_THRESHOLD_SECS)
        is_active: bool,
    },
    /// A session file was removed
    SessionRemoved { session_id: String },
}

/// Error type for watcher operations
#[derive(Debug)]
pub enum WatcherError {
    /// Failed to initialize the notify watcher
    NotifyInit(notify::Error),
    /// Failed to watch a directory
    WatchDir { path: PathBuf, error: notify::Error },
}

impl std::fmt::Display for WatcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotifyInit(e) => write!(f, "Failed to initialize file watcher: {}", e),
            Self::WatchDir { path, error } => {
                write!(f, "Failed to watch directory {}: {}", path.display(), error)
            }
        }
    }
}

impl std::error::Error for WatcherError {}

/// File watcher for transcript directories
pub struct TranscriptWatcher {
    /// The underlying notify watcher
    watcher: RecommendedWatcher,
    /// Receiver for file system events
    fs_rx: Receiver<notify::Result<Event>>,
    /// Tracked file states
    files: HashMap<PathBuf, FileState>,
    /// Pending file events (for debouncing)
    pending: HashMap<PathBuf, Instant>,
    /// Last full scan time
    last_scan: Instant,
    /// Directories being watched
    watched_dirs: Vec<PathBuf>,
}

impl TranscriptWatcher {
    /// Create a new transcript watcher.
    ///
    /// Initializes the file watcher and begins watching Claude Code and Codex
    /// transcript directories if they exist.
    pub fn new() -> Result<Self, WatcherError> {
        let (fs_tx, fs_rx) = mpsc::channel();

        let watcher =
            notify::recommended_watcher(move |res| {
                let _ = fs_tx.send(res);
            })
            .map_err(WatcherError::NotifyInit)?;

        let mut w = Self {
            watcher,
            fs_rx,
            files: HashMap::new(),
            pending: HashMap::new(),
            last_scan: Instant::now() - SCAN_INTERVAL, // Force initial scan
            watched_dirs: Vec::new(),
        };

        // Start watching directories
        w.setup_watches()?;

        Ok(w)
    }

    /// Set up watches on transcript directories.
    fn setup_watches(&mut self) -> Result<(), WatcherError> {
        // Watch Claude Code projects directory
        let claude_dir = claude_code::projects_dir();
        if claude_dir.exists() {
            info!("Watching Claude Code projects: {}", claude_dir.display());
            self.watcher
                .watch(&claude_dir, RecursiveMode::Recursive)
                .map_err(|e| WatcherError::WatchDir {
                    path: claude_dir.clone(),
                    error: e,
                })?;
            self.watched_dirs.push(claude_dir);
        } else {
            debug!(
                "Claude Code projects directory not found: {}",
                claude_dir.display()
            );
        }

        // Watch Codex sessions directory
        let codex_dir = codex::sessions_dir();
        if codex_dir.exists() {
            info!("Watching Codex sessions: {}", codex_dir.display());
            self.watcher
                .watch(&codex_dir, RecursiveMode::Recursive)
                .map_err(|e| WatcherError::WatchDir {
                    path: codex_dir.clone(),
                    error: e,
                })?;
            self.watched_dirs.push(codex_dir);
        } else {
            debug!(
                "Codex sessions directory not found: {}",
                codex_dir.display()
            );
        }

        Ok(())
    }

    /// Process pending file system events.
    ///
    /// Call this periodically (e.g., every 50ms) to process file changes and
    /// emit watcher events. Returns a vector of events that occurred since the
    /// last poll.
    pub fn poll(&mut self) -> Vec<WatcherEvent> {
        let mut events = Vec::new();

        // Drain file system events from the channel
        loop {
            match self.fs_rx.try_recv() {
                Ok(Ok(event)) => self.handle_fs_event(event),
                Ok(Err(e)) => warn!("Watch error: {:?}", e),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    warn!("File watcher channel disconnected");
                    break;
                }
            }
        }

        // Process debounced events
        let now = Instant::now();
        let ready: Vec<PathBuf> = self
            .pending
            .iter()
            .filter(|(_, time)| now.duration_since(**time) >= DEBOUNCE_DURATION)
            .map(|(path, _)| path.clone())
            .collect();

        for path in ready {
            self.pending.remove(&path);
            if let Some(event) = self.process_file(&path) {
                events.push(event);
            }
        }

        // Periodic full scan to catch missed files
        if now.duration_since(self.last_scan) >= SCAN_INTERVAL {
            self.last_scan = now;
            events.extend(self.full_scan());
        }

        events
    }

    /// Handle a file system event from notify.
    fn handle_fs_event(&mut self, event: Event) {
        for path in event.paths {
            // Only process .jsonl files
            if !path.extension().is_some_and(|e| e == "jsonl") {
                continue;
            }

            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    trace!("File changed: {}", path.display());
                    self.pending.insert(path, Instant::now());
                }
                EventKind::Remove(_) => {
                    trace!("File removed: {}", path.display());
                    self.handle_file_removal(&path);
                }
                _ => {}
            }
        }
    }

    /// Handle removal of a transcript file.
    fn handle_file_removal(&mut self, path: &Path) {
        // Remove from pending if present
        self.pending.remove(path);

        // If we were tracking this file, emit a removal event
        if let Some(state) = self.files.remove(path) {
            debug!(
                "Session file removed: {} (session_id={})",
                path.display(),
                state.session_id
            );
            // Note: We don't send SessionRemoved here because:
            // 1. The registry handles session removal via SessionEnded events
            // 2. File removal during active sessions is rare and might be temporary
            // If you need to handle orphaned sessions, do it via stale detection instead.
        }
    }

    /// Process a single file and emit event if needed.
    fn process_file(&mut self, path: &Path) -> Option<WatcherEvent> {
        // Determine agent type from path
        let path_str = path.to_string_lossy();
        let agent = if path_str.contains(".claude/projects") {
            AgentType::ClaudeCode
        } else if path_str.contains(".codex/sessions") {
            AgentType::Codex
        } else {
            debug!("Ignoring file outside known directories: {}", path.display());
            return None;
        };

        // Get modification time
        let metadata = path.metadata().ok()?;
        let mtime = metadata.modified().ok()?;

        // Check if we need to process this file
        if let Some(state) = self.files.get(path) {
            if state.mtime >= mtime {
                trace!("File unchanged: {}", path.display());
                return None; // No change
            }
        }

        // Read transcript metadata
        let meta = match agent {
            AgentType::ClaudeCode => claude_code::read_transcript_meta(path).ok()?,
            AgentType::Codex => codex::read_transcript_meta(path).ok()?,
            _ => return None,
        };

        // Extract session ID
        let session_id = match agent {
            AgentType::ClaudeCode => claude_code::session_id_from_path(path)?,
            AgentType::Codex => {
                let filename = path.file_name()?.to_str()?;
                codex::session_id_from_filename(filename)?
            }
            _ => return None,
        };

        // Derive CWD
        let cwd = meta.cwd.clone().unwrap_or_else(|| {
            // For Claude Code, derive from directory name
            if agent == AgentType::ClaudeCode {
                path.parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(claude_code::unescape_path)
                    .unwrap_or_default()
            } else {
                String::new()
            }
        });

        let is_new = !self.files.contains_key(path);

        // Update tracked state
        self.files.insert(
            path.to_path_buf(),
            FileState {
                mtime,
                session_id: session_id.clone(),
                agent: agent.clone(),
                cwd: Some(cwd.clone()),
                meta: Some(meta.clone()),
                started: true,
            },
        );

        // Check if the last event in the transcript was recent (indicates active session)
        let is_active = meta
            .last_event_timestamp
            .as_ref()
            .and_then(|ts| parse_iso_timestamp(ts))
            .map(|event_time| {
                let now = std::time::SystemTime::now();
                now.duration_since(event_time)
                    .map(|elapsed| elapsed.as_secs() < ACTIVE_THRESHOLD_SECS)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        if is_new {
            debug!(
                "New session detected: {} (session_id={}, active={})",
                path.display(),
                session_id,
                is_active
            );
        } else {
            trace!(
                "Session updated: {} (session_id={}, active={})",
                path.display(),
                session_id,
                is_active
            );
        }

        Some(WatcherEvent::SessionUpdate {
            session_id,
            cwd,
            agent,
            meta,
            is_active,
        })
    }

    /// Perform a full scan of all transcript directories.
    ///
    /// This catches files that may have been created before the watcher started
    /// or files that were missed due to filesystem event race conditions.
    fn full_scan(&mut self) -> Vec<WatcherEvent> {
        let mut events = Vec::new();

        // Scan Claude Code transcripts
        for path in claude_code::discover_transcripts(None) {
            if !self.files.contains_key(&path) {
                if let Some(event) = self.process_file(&path) {
                    events.push(event);
                }
            }
        }

        // Scan Codex sessions
        for path in codex::discover_sessions(None) {
            if !self.files.contains_key(&path) {
                if let Some(event) = self.process_file(&path) {
                    events.push(event);
                }
            }
        }

        if !events.is_empty() {
            debug!("Full scan found {} new sessions", events.len());
        }

        events
    }

    /// Get the number of tracked files.
    pub fn tracked_count(&self) -> usize {
        self.files.len()
    }

    /// Get the directories being watched.
    pub fn watched_dirs(&self) -> &[PathBuf] {
        &self.watched_dirs
    }

    /// Check if any directories are being watched.
    pub fn is_watching(&self) -> bool {
        !self.watched_dirs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Agent type detection ====================

    #[test]
    fn detect_claude_code_from_path() {
        let paths = [
            "/Users/test/.claude/projects/-Users-test-project/abc123.jsonl",
            "/home/user/.claude/projects/-home-user-code/session.jsonl",
        ];

        for path_str in paths {
            let path = Path::new(path_str);
            let path_lossy = path.to_string_lossy();
            let is_claude = path_lossy.contains(".claude/projects");
            assert!(is_claude, "Expected Claude Code path: {}", path_str);
        }
    }

    #[test]
    fn detect_codex_from_path() {
        let paths = [
            "/Users/test/.codex/sessions/2025/08/10/rollout-2025-08-10T12-50-53-uuid.jsonl",
            "/home/user/.codex/sessions/2025/01/15/rollout-timestamp-uuid.jsonl",
        ];

        for path_str in paths {
            let path = Path::new(path_str);
            let path_lossy = path.to_string_lossy();
            let is_codex = path_lossy.contains(".codex/sessions");
            assert!(is_codex, "Expected Codex path: {}", path_str);
        }
    }

    #[test]
    fn detect_unknown_path() {
        let paths = [
            "/tmp/random/file.jsonl",
            "/Users/test/projects/session.jsonl",
        ];

        for path_str in paths {
            let path = Path::new(path_str);
            let path_lossy = path.to_string_lossy();
            let is_claude = path_lossy.contains(".claude/projects");
            let is_codex = path_lossy.contains(".codex/sessions");
            assert!(
                !is_claude && !is_codex,
                "Expected unknown path: {}",
                path_str
            );
        }
    }

    // ==================== Session ID extraction ====================

    #[test]
    fn session_id_from_claude_code_path() {
        let path = Path::new("/Users/test/.claude/projects/-Users-test-project/abc123-def456.jsonl");
        let session_id = claude_code::session_id_from_path(path);
        assert_eq!(session_id, Some("abc123-def456".into()));
    }

    #[test]
    fn session_id_from_codex_filename() {
        let filename = "rollout-2025-08-10T12-50-53-a3953a61-af96-4bfc-8a05-f8355309f025.jsonl";
        let session_id = codex::session_id_from_filename(filename);
        assert_eq!(
            session_id,
            Some("a3953a61-af96-4bfc-8a05-f8355309f025".into())
        );
    }

    // ==================== Debouncing ====================

    #[test]
    fn debounce_duration_is_reasonable() {
        // Debounce should be short enough to feel responsive
        // but long enough to batch rapid changes
        assert!(DEBOUNCE_DURATION >= Duration::from_millis(50));
        assert!(DEBOUNCE_DURATION <= Duration::from_millis(500));
    }

    #[test]
    fn scan_interval_is_reasonable() {
        // Scan interval should be long enough to not impact performance
        // but short enough to catch missed files reasonably quickly
        assert!(SCAN_INTERVAL >= Duration::from_secs(10));
        assert!(SCAN_INTERVAL <= Duration::from_secs(120));
    }

    // ==================== WatcherEvent ====================

    #[test]
    fn watcher_event_session_update() {
        let event = WatcherEvent::SessionUpdate {
            session_id: "test-session".into(),
            cwd: "/tmp/test".into(),
            agent: AgentType::ClaudeCode,
            meta: TranscriptMeta::default(),
            is_active: true,
        };

        match event {
            WatcherEvent::SessionUpdate {
                session_id,
                cwd,
                agent,
                is_active,
                ..
            } => {
                assert_eq!(session_id, "test-session");
                assert_eq!(cwd, "/tmp/test");
                assert_eq!(agent, AgentType::ClaudeCode);
                assert!(is_active);
            }
            _ => panic!("Expected SessionUpdate"),
        }
    }

    #[test]
    fn watcher_event_session_removed() {
        let event = WatcherEvent::SessionRemoved {
            session_id: "test-session".into(),
        };

        match event {
            WatcherEvent::SessionRemoved { session_id } => {
                assert_eq!(session_id, "test-session");
            }
            _ => panic!("Expected SessionRemoved"),
        }
    }

    // ==================== WatcherError ====================

    #[test]
    fn watcher_error_display() {
        let error = WatcherError::WatchDir {
            path: PathBuf::from("/test/path"),
            error: notify::Error::generic("test error"),
        };
        let display = format!("{}", error);
        assert!(display.contains("/test/path"));
    }

    // ==================== FileState ====================

    #[test]
    fn file_state_fields() {
        let state = FileState {
            mtime: SystemTime::now(),
            session_id: "session-123".into(),
            agent: AgentType::Codex,
            cwd: Some("/tmp/project".into()),
            meta: None,
            started: false,
        };

        assert_eq!(state.session_id, "session-123");
        assert_eq!(state.agent, AgentType::Codex);
        assert_eq!(state.cwd, Some("/tmp/project".into()));
        assert!(!state.started);
    }

    // ==================== ISO timestamp parsing ====================

    #[test]
    fn parse_iso_timestamp_with_z_suffix() {
        let ts = "2026-01-31T12:45:31.053Z";
        let result = parse_iso_timestamp(ts);
        assert!(result.is_some());

        // Verify the parsed time is reasonable (after Unix epoch)
        let system_time = result.unwrap();
        assert!(system_time > SystemTime::UNIX_EPOCH);
    }

    #[test]
    fn parse_iso_timestamp_without_millis() {
        let ts = "2026-01-31T12:45:31Z";
        let result = parse_iso_timestamp(ts);
        assert!(result.is_some());
    }

    #[test]
    fn parse_iso_timestamp_with_offset() {
        let ts = "2026-01-31T12:45:31.053+00:00";
        let result = parse_iso_timestamp(ts);
        assert!(result.is_some());
    }

    #[test]
    fn parse_iso_timestamp_with_negative_offset() {
        let ts = "2026-01-31T07:45:31.053-05:00";
        let result = parse_iso_timestamp(ts);
        assert!(result.is_some());
    }

    #[test]
    fn parse_iso_timestamp_invalid() {
        assert!(parse_iso_timestamp("not a timestamp").is_none());
        assert!(parse_iso_timestamp("2026-01-31").is_none()); // missing time
        assert!(parse_iso_timestamp("12:45:31").is_none()); // missing date
        assert!(parse_iso_timestamp("").is_none());
    }

    #[test]
    fn parse_iso_timestamp_is_recent() {
        // A timestamp from now should be considered recent
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let secs = duration_since_epoch.as_secs();

        // Convert to components (simplified, assumes UTC)
        let days = secs / 86400;
        let remaining = secs % 86400;
        let hours = remaining / 3600;
        let minutes = (remaining % 3600) / 60;
        let seconds = remaining % 60;

        // Calculate year/month/day from days since epoch (simplified)
        let mut year = 1970;
        let mut remaining_days = days as i64;
        while remaining_days >= if is_leap_year(year) { 366 } else { 365 } {
            remaining_days -= if is_leap_year(year) { 366 } else { 365 };
            year += 1;
        }

        let days_in_months: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut month = 1;
        for (i, &days_in_month) in days_in_months.iter().enumerate() {
            let mut dm = days_in_month;
            if i == 1 && is_leap_year(year) {
                dm = 29;
            }
            if remaining_days < dm {
                break;
            }
            remaining_days -= dm;
            month += 1;
        }
        let day = remaining_days + 1;

        let ts = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year, month, day, hours, minutes, seconds
        );

        let parsed = parse_iso_timestamp(&ts).unwrap();
        let elapsed = now.duration_since(parsed).unwrap_or(Duration::ZERO);

        // Should be within a few seconds of now
        assert!(elapsed.as_secs() < 5, "Parsed time should be recent");
    }
}
