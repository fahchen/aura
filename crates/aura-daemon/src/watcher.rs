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
    AgentEvent, AgentType,
    time::parse_rfc3339_system_time,
};
use crate::parsers::{self, claude as claude_parser, codex as codex_parser, ACTIVE_THRESHOLD_SECS};
use serde_json::Value;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info, trace, warn};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

/// Debounce window for file events.
/// Multiple rapid changes to the same file within this window are batched.
const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// History backfill size for Codex activity (bytes).
const CODEX_HISTORY_BACKFILL_BYTES: u64 = 64 * 1024;
/// History backfill size for Claude activity (bytes).
const CLAUDE_HISTORY_BACKFILL_BYTES: u64 = 64 * 1024;
/// Transcript backfill size for tool events (bytes).
const TRANSCRIPT_BACKFILL_BYTES: u64 = 64 * 1024;

/// State tracking for a transcript file
#[derive(Debug)]
struct FileState {
    /// Last known modification time
    mtime: SystemTime,
    /// Last known file length
    len: u64,
    /// Session ID extracted from this file
    session_id: String,
    /// Last read offset for incremental parsing
    offset: u64,
}

/// Events emitted by the watcher
#[derive(Debug, Clone)]
pub enum WatcherEvent {
    /// A new or updated session was detected
    SessionUpdate {
        session_id: String,
        cwd: String,
        agent: AgentType,
        meta: Box<TranscriptMeta>,
        /// True if the file was modified recently (within ACTIVE_THRESHOLD_SECS)
        is_active: bool,
        /// Best-effort last activity time (from transcript timestamp or file mtime)
        last_event_time: Option<SystemTime>,
    },
    /// Parsed agent event from transcript/history
    AgentEvent { event: AgentEvent },
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
    /// Pending history events (for debouncing)
    pending_history: Option<Instant>,
    /// Pending Claude history events (for debouncing)
    pending_claude_history: Option<Instant>,
    /// Directories being watched
    watched_dirs: Vec<PathBuf>,
    /// Whether the initial full scan has completed
    initial_scan_done: bool,
    /// Codex history file path (if present)
    codex_history_path: Option<PathBuf>,
    /// Last read offset for Codex history file
    codex_history_offset: u64,
    /// Claude history file path (if present)
    claude_history_path: Option<PathBuf>,
    /// Last read offset for Claude history file
    claude_history_offset: u64,
    /// History sessions seen (Codex)
    codex_history_sessions: HashSet<String>,
    /// History sessions seen (Claude)
    claude_history_sessions: HashSet<String>,
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
            pending_history: None,
            pending_claude_history: None,
            watched_dirs: Vec::new(),
            initial_scan_done: false,
            codex_history_path: None,
            codex_history_offset: 0,
            claude_history_path: None,
            claude_history_offset: 0,
            codex_history_sessions: HashSet::new(),
            claude_history_sessions: HashSet::new(),
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

        // Watch Codex history file for activity signals
        let history_path = codex::history_path();
        if history_path.exists() {
            info!("Watching Codex history: {}", history_path.display());
            self.watcher
                .watch(&history_path, RecursiveMode::NonRecursive)
                .map_err(|e| WatcherError::WatchDir {
                    path: history_path.clone(),
                    error: e,
                })?;
            self.codex_history_path = Some(history_path.clone());
            self.watched_dirs.push(history_path.clone());

            if let Ok(metadata) = history_path.metadata() {
                let len = metadata.len();
                self.codex_history_offset = len.saturating_sub(CODEX_HISTORY_BACKFILL_BYTES);
                self.pending_history = Some(Instant::now() - DEBOUNCE_DURATION);
            }
        } else {
            debug!(
                "Codex history file not found: {}",
                history_path.display()
            );
        }

        // Watch Claude history file for activity signals
        let claude_history = claude_code::history_path();
        if claude_history.exists() {
            info!("Watching Claude history: {}", claude_history.display());
            self.watcher
                .watch(&claude_history, RecursiveMode::NonRecursive)
                .map_err(|e| WatcherError::WatchDir {
                    path: claude_history.clone(),
                    error: e,
                })?;
            self.claude_history_path = Some(claude_history.clone());
            self.watched_dirs.push(claude_history.clone());

            if let Ok(metadata) = claude_history.metadata() {
                let len = metadata.len();
                self.claude_history_offset = len.saturating_sub(CLAUDE_HISTORY_BACKFILL_BYTES);
                self.pending_claude_history = Some(Instant::now() - DEBOUNCE_DURATION);
            }
        } else {
            debug!(
                "Claude history file not found: {}",
                claude_history.display()
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
            events.extend(self.process_file(&path, true));
        }

        if let Some(queued_at) = self.pending_history {
            if now.duration_since(queued_at) >= DEBOUNCE_DURATION {
                self.pending_history = None;
                events.extend(self.process_codex_history());
            }
        }

        if let Some(queued_at) = self.pending_claude_history {
            if now.duration_since(queued_at) >= DEBOUNCE_DURATION {
                self.pending_claude_history = None;
                events.extend(self.process_claude_history());
            }
        }

        if !self.initial_scan_done {
            events.extend(self.full_scan());
        }

        events
    }

    /// Handle a file system event from notify.
    fn handle_fs_event(&mut self, event: Event) {
        for path in event.paths {
            if let Some(history_path) = self.codex_history_path.as_ref()
                && is_same_path(&path, history_path)
            {
                self.pending_history = Some(Instant::now());
                continue;
            }

            if let Some(history_path) = self.claude_history_path.as_ref()
                && is_same_path(&path, history_path)
            {
                self.pending_claude_history = Some(Instant::now());
                continue;
            }

            // Only process .jsonl files
            if path.extension().is_none_or(|e| e != "jsonl") {
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

    /// Process a single file and emit events if needed.
    fn process_file(&mut self, path: &Path, emit_inactive: bool) -> Vec<WatcherEvent> {
        let mut events = Vec::new();
        // Determine agent type from path
        let agent = if is_under_dir(path, &claude_code::projects_dir()) {
            AgentType::ClaudeCode
        } else if is_under_dir(path, &codex::sessions_dir()) {
            AgentType::Codex
        } else {
            debug!("Ignoring file outside known directories: {}", path.display());
            return events;
        };

        // Get modification time
        let metadata = match path.metadata() {
            Ok(meta) => meta,
            Err(_) => return events,
        };
        let mtime = match metadata.modified() {
            Ok(time) => time,
            Err(_) => return events,
        };
        let file_len = metadata.len();

        // Check if we need to process this file
        if let Some(state) = self.files.get(path)
            && state.mtime >= mtime
            && state.len >= file_len
        {
            trace!("File unchanged: {}", path.display());
            return events; // No change
        }

        // Read transcript metadata
        let meta = match agent {
            AgentType::ClaudeCode => match claude_code::read_transcript_meta(path) {
                Ok(meta) => meta,
                Err(e) => {
                    warn!("Failed to parse Claude transcript {}: {}", path.display(), e);
                    return events;
                }
            },
            AgentType::Codex => match codex::read_transcript_meta(path) {
                Ok(meta) => meta,
                Err(e) => {
                    warn!("Failed to parse Codex transcript {}: {}", path.display(), e);
                    return events;
                }
            },
            _ => return events,
        };

        // Extract session ID (prefer transcript metadata, fallback to path/filename)
        let session_id = if let Some(session_id) = meta.session_id.clone() {
            session_id
        } else {
            match agent {
                AgentType::ClaudeCode => match claude_code::session_id_from_path(path) {
                    Some(id) => id,
                    None => {
                        warn!("Failed to extract Claude session id from {}", path.display());
                        return events;
                    }
                },
                AgentType::Codex => {
                    let filename = match path.file_name().and_then(|n| n.to_str()) {
                        Some(name) => name,
                        None => {
                            warn!("Failed to read Codex filename for {}", path.display());
                            return events;
                        }
                    };
                    match codex::session_id_from_filename(filename) {
                        Some(id) => id,
                        None => {
                            warn!("Failed to extract Codex session id from {}", path.display());
                            return events;
                        }
                    }
                }
                _ => return events,
            }
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
        let previous_offset = self
            .files
            .get(path)
            .map(|state| state.offset)
            .unwrap_or(0);
        let start_offset = if is_new {
            file_len.saturating_sub(TRANSCRIPT_BACKFILL_BYTES)
        } else {
            previous_offset.min(file_len)
        };

        let session_id_for_events = session_id.clone();
        let cwd_for_events = cwd.clone();
        let (new_values, new_offset) = read_jsonl_tail(path, start_offset);
        for value in new_values {
            events.extend(parsers::events_from_transcript_line(
                &agent,
                &value,
                &session_id_for_events,
                &cwd_for_events,
            ));
        }

        // Update tracked state
        self.files.insert(
            path.to_path_buf(),
            FileState {
                mtime,
                len: file_len,
                session_id: session_id.clone(),
                offset: new_offset,
            },
        );

        // Best-effort last activity time: transcript timestamp or file mtime
        let last_event_time = meta
            .last_event_timestamp
            .as_ref()
            .and_then(|ts| parse_rfc3339_system_time(ts))
            .map(|ts| if mtime > ts { mtime } else { ts })
            .or(Some(mtime));

        // Check if the last event in the transcript was recent (indicates active session)
        let is_active = last_event_time
            .as_ref()
            .map(|event_time| {
                let now = std::time::SystemTime::now();
                now.duration_since(*event_time)
                    .map(|elapsed| elapsed.as_secs() < ACTIVE_THRESHOLD_SECS)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        if is_active || emit_inactive {
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

            debug!(
                "Session update event: {} (session_id={}, active={}, mtime={:?}, last_event_time={:?})",
                path.display(),
                session_id,
                is_active,
                mtime,
                last_event_time
            );

            events.push(WatcherEvent::SessionUpdate {
                session_id,
                cwd,
                agent: agent.clone(),
                meta: Box::new(meta),
                is_active,
                last_event_time,
            });

            if is_new {
                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::SessionStarted {
                        session_id: session_id_for_events,
                        cwd: cwd_for_events,
                        agent,
                    },
                });
            }
        }

        events
    }

    fn process_codex_history(&mut self) -> Vec<WatcherEvent> {
        let Some(history_path) = self.codex_history_path.as_ref() else {
            return Vec::new();
        };

        let (values, new_offset) =
            read_jsonl_tail(history_path, self.codex_history_offset);
        self.codex_history_offset = new_offset;

        let mut events = Vec::new();
        for value in values {
            let is_recent = parsers::history_is_recent_seconds(&value, "ts");

            if let Some(session_id) = value.get("session_id").and_then(|v| v.as_str())
                && is_recent
                && self.codex_history_sessions.insert(session_id.to_string())
            {
                let has_transcript = self.files.values().any(|state| state.session_id == session_id);
                if !has_transcript {
                    let mut meta = TranscriptMeta::default();
                    meta.session_id = Some(session_id.to_string());
                    let last_event_time = value
                        .get("ts")
                        .and_then(|v| v.as_u64())
                        .map(|ts| SystemTime::UNIX_EPOCH + Duration::from_secs(ts));
                    events.push(WatcherEvent::SessionUpdate {
                        session_id: session_id.to_string(),
                        cwd: String::new(),
                        agent: AgentType::Codex,
                        meta: Box::new(meta),
                        is_active: true,
                        last_event_time,
                    });
                }

                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::SessionStarted {
                        session_id: session_id.to_string(),
                        cwd: String::new(),
                        agent: AgentType::Codex,
                    },
                });
            }

            events.extend(codex_parser::events_from_history(&value));
        }

        events
    }

    fn process_claude_history(&mut self) -> Vec<WatcherEvent> {
        let Some(history_path) = self.claude_history_path.as_ref() else {
            return Vec::new();
        };

        let (values, new_offset) =
            read_jsonl_tail(history_path, self.claude_history_offset);
        self.claude_history_offset = new_offset;

        let mut events = Vec::new();
        for value in values {
            let is_recent = parsers::history_is_recent_millis(&value, "timestamp");

            if let Some(session_id) = value.get("sessionId").and_then(|v| v.as_str())
                && is_recent
                && self.claude_history_sessions.insert(session_id.to_string())
            {
                let cwd = value
                    .get("project")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::SessionStarted {
                        session_id: session_id.to_string(),
                        cwd,
                        agent: AgentType::ClaudeCode,
                    },
                });
            }

            events.extend(claude_parser::events_from_history(&value));
        }

        events
    }


    /// Perform a full scan of all transcript directories.
    ///
    /// This catches files that may have been created before the watcher started
    /// or files that were missed due to filesystem event race conditions.
    fn full_scan(&mut self) -> Vec<WatcherEvent> {
        let mut events = Vec::new();

        // Scan Claude Code transcripts
        let mut claude_seen = 0usize;
        let emit_inactive = self.initial_scan_done;
        for path in claude_code::discover_transcripts(None) {
            if !self.files.contains_key(&path) {
                events.extend(self.process_file(&path, emit_inactive));
            }
            claude_seen += 1;
        }

        // Scan Codex sessions
        let mut codex_seen = 0usize;
        for path in codex::discover_sessions(None) {
            if !self.files.contains_key(&path) {
                events.extend(self.process_file(&path, emit_inactive));
            }
            codex_seen += 1;
        }

        debug!(
            "Full scan complete: claude_files={}, codex_files={}, new_events={}",
            claude_seen,
            codex_seen,
            events.len()
        );

        if !self.initial_scan_done {
            self.initial_scan_done = true;
            let filtered = filter_active_events(events);
            debug!(
                "Initial scan filtered to {} active session(s)",
                filtered
                    .iter()
                    .filter(|e| matches!(e, WatcherEvent::SessionUpdate { .. }))
                    .count()
            );
            return filtered;
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

fn is_under_dir(path: &Path, dir: &Path) -> bool {
    let path_canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let dir_canon = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    path_canon.starts_with(&dir_canon)
}

fn is_same_path(left: &Path, right: &Path) -> bool {
    let left_canon = std::fs::canonicalize(left).unwrap_or_else(|_| left.to_path_buf());
    let right_canon = std::fs::canonicalize(right).unwrap_or_else(|_| right.to_path_buf());
    left_canon == right_canon
}

fn read_jsonl_tail(path: &Path, start_offset: u64) -> (Vec<Value>, u64) {
    let Ok(mut file) = File::open(path) else {
        return (Vec::new(), start_offset);
    };
    let Ok(metadata) = file.metadata() else {
        return (Vec::new(), start_offset);
    };

    let file_len = metadata.len();
    let offset = if start_offset > file_len { 0 } else { start_offset };

    if file.seek(SeekFrom::Start(offset)).is_err() {
        return (Vec::new(), start_offset);
    }

    let mut buffer = Vec::new();
    if file.read_to_end(&mut buffer).is_err() || buffer.is_empty() {
        return (Vec::new(), offset);
    }

    let Some(last_newline) = buffer.iter().rposition(|b| *b == b'\n') else {
        return (Vec::new(), offset);
    };

    let parse_len = last_newline + 1;
    let new_offset = offset.saturating_add(parse_len as u64);
    let parse_bytes = &buffer[..parse_len];

    let mut values = Vec::new();
    for line in String::from_utf8_lossy(parse_bytes).lines() {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            values.push(value);
        }
    }

    (values, new_offset)
}

fn filter_active_events(events: Vec<WatcherEvent>) -> Vec<WatcherEvent> {
    let mut active_ids = HashSet::new();
    for event in &events {
        if let WatcherEvent::SessionUpdate {
            session_id,
            is_active,
            ..
        } = event
        {
            if *is_active {
                active_ids.insert(session_id.clone());
            }
        }
    }

    events
        .into_iter()
        .filter(|event| match event {
            WatcherEvent::SessionUpdate { is_active, .. } => *is_active,
            WatcherEvent::AgentEvent { event } => active_ids.contains(event.session_id()),
            WatcherEvent::SessionRemoved { session_id } => active_ids.contains(session_id),
        })
        .collect()
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
    fn initial_scan_filters_inactive_sessions() {
        let mut watcher = TranscriptWatcher::new().expect("watcher");
        watcher.initial_scan_done = false;
        let events = watcher.full_scan();
        assert!(watcher.initial_scan_done);
        assert!(events.iter().all(|event| match event {
            WatcherEvent::SessionUpdate { is_active, .. } => *is_active,
            _ => true,
        }));
    }

    // ==================== WatcherEvent ====================

    #[test]
    fn watcher_event_session_update() {
        let event = WatcherEvent::SessionUpdate {
            session_id: "test-session".into(),
            cwd: "/tmp/test".into(),
            agent: AgentType::ClaudeCode,
            meta: Box::new(TranscriptMeta::default()),
            is_active: true,
            last_event_time: None,
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
        };

        assert_eq!(state.session_id, "session-123");
    }

}
