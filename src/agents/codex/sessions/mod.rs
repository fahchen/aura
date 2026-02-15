//! Codex session rollout JSONL watcher.
//!
//! This module watches Codex session rollouts under `~/.codex/sessions/**.jsonl`
//! (or `$CODEX_HOME/sessions`) and emits [`AgentEvent`]s on a best-effort stream.
//!
//! This integration works even when Codex is started externally (e.g. via `codex` CLI)
//! because it consumes Codex's public session rollout files.

mod parser;
mod paths;

use self::parser::RolloutState;
use crate::{AgentEvent, AgentType};
use notify::{RecursiveMode, Watcher};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader};
use tokio::sync::{Notify, broadcast};
use tracing::{debug, info, trace, warn};

const BOOTSTRAP_REPLAY_MAX_EVENTS: usize = 4;
const VISIBILITY_WINDOW: Duration = Duration::from_secs(10 * 60);
const FALLBACK_SCAN_INTERVAL: Duration = Duration::from_secs(2);

fn parse_json_line(path: &Path, line: &str, context: &'static str) -> Option<Value> {
    match serde_json::from_str(line) {
        Ok(v) => Some(v),
        Err(e) => {
            // Avoid logging the full line (may contain user content).
            warn!(path = %path.display(), error = %e, "{context}");
            None
        }
    }
}

fn drain_jsonl_lines(buffer: &mut String, mut on_line: impl FnMut(&str)) {
    let mut start = 0usize;
    let bytes = buffer.as_bytes();

    for (idx, b) in bytes.iter().enumerate() {
        if *b != b'\n' {
            continue;
        }

        let line = buffer[start..idx].trim();
        start = idx + 1;
        if line.is_empty() {
            continue;
        }
        on_line(line);
    }

    if start > 0 {
        *buffer = buffer[start..].to_string();
    }
}

#[derive(Debug)]
struct WatchedRollout {
    path: PathBuf,
    offset: u64,
    buffer: String,
    state: RolloutState,
}

impl WatchedRollout {
    fn new_existing(path: PathBuf, session_id: String, cwd: String, offset: u64) -> Self {
        Self {
            path,
            offset,
            buffer: String::new(),
            state: RolloutState::new(session_id, cwd),
        }
    }

    fn new_fresh(path: PathBuf, session_id: String, cwd: String) -> Self {
        Self {
            path,
            offset: 0,
            buffer: String::new(),
            state: RolloutState::new(session_id, cwd),
        }
    }
}

fn emit_events(tx: &broadcast::Sender<AgentEvent>, events: Vec<AgentEvent>) {
    if events.is_empty() {
        return;
    }
    for event in events {
        trace!(?event, "codex rollout event");
        let _ = tx.send(event);
    }
}

async fn read_first_session_meta(path: &Path) -> Option<(String, String)> {
    let file = tokio::fs::File::open(path).await.ok()?;
    let mut reader = BufReader::new(file);

    // Read the first non-empty line.
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await.ok()?;
        if n == 0 {
            return None;
        }
        if !line.trim().is_empty() {
            break;
        }
    }

    let value: Value = serde_json::from_str(line.trim()).ok()?;

    if value.get("type").and_then(|v| v.as_str()) != Some("session_meta") {
        return None;
    }

    let payload = value.get("payload").unwrap_or(&Value::Null);
    let id = payload.get("id").and_then(|v| v.as_str())?.to_string();
    let cwd = payload
        .get("cwd")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Some((id, cwd))
}

async fn file_len(path: &Path) -> Option<u64> {
    tokio::fs::metadata(path).await.ok().map(|m| m.len())
}

async fn bootstrap_rollout(watched: &mut WatchedRollout, tx: &broadcast::Sender<AgentEvent>) {
    // Ignore stale sessions to avoid flooding the HUD with historical rollouts.
    if !paths::modified_within(&watched.path, VISIBILITY_WINDOW).await {
        watched.offset = file_len(&watched.path).await.unwrap_or(watched.offset);
        watched.buffer.clear();
        return;
    }

    let mut file = match tokio::fs::File::open(&watched.path).await {
        Ok(f) => f,
        Err(e) => {
            debug!(path = %watched.path.display(), error = %e, "failed to open codex rollout for bootstrap");
            return;
        }
    };

    let mut buf = Vec::new();
    if let Err(e) = file.read_to_end(&mut buf).await {
        debug!(path = %watched.path.display(), error = %e, "failed to read codex rollout for bootstrap");
        return;
    }
    watched.offset = buf.len() as u64;

    let mut scan_state =
        RolloutState::new(watched.state.session_id.clone(), watched.state.cwd.clone());
    let mut replay: std::collections::VecDeque<AgentEvent> =
        std::collections::VecDeque::with_capacity(BOOTSTRAP_REPLAY_MAX_EVENTS);
    let mut latest_name: Option<String> = None;

    let path = watched.path.clone();
    watched.buffer = String::from_utf8_lossy(&buf).to_string();
    drain_jsonl_lines(&mut watched.buffer, |line| {
        let Some(value) = parse_json_line(
            path.as_path(),
            line,
            "malformed JSON in codex rollout bootstrap",
        ) else {
            return;
        };
        for event in scan_state.apply_line(&value) {
            match event {
                AgentEvent::SessionStarted { .. } => {}
                AgentEvent::SessionNameUpdated { name, .. } => latest_name = Some(name),
                other => {
                    if replay.len() == BOOTSTRAP_REPLAY_MAX_EVENTS {
                        replay.pop_front();
                    }
                    replay.push_back(other);
                }
            }
        }
    });

    // Emit bootstrap events (SessionStarted + latest SessionNameUpdated + last N events).
    let mut out = Vec::with_capacity(2 + replay.len());
    out.push(AgentEvent::SessionStarted {
        session_id: scan_state.session_id.clone(),
        cwd: scan_state.cwd.clone(),
        agent: AgentType::Codex,
    });
    if let Some(name) = latest_name {
        out.push(AgentEvent::SessionNameUpdated {
            session_id: scan_state.session_id.clone(),
            name,
        });
    }
    out.extend(replay.into_iter());
    emit_events(tx, out);

    watched.state = scan_state;
    watched.state.session_emitted = true;
}

async fn tail_rollout(watched: &mut WatchedRollout, tx: &broadcast::Sender<AgentEvent>) {
    loop {
        let Some(len) = file_len(&watched.path).await else {
            return;
        };

        if len < watched.offset {
            debug!(path = %watched.path.display(), "codex rollout truncated; resetting cursor");
            watched.offset = 0;
            watched.buffer.clear();
            watched.state.session_emitted = false;

            // A truncation is effectively a new rollout stream. Re-bootstrap immediately
            // so we don't replay the full history and flood the HUD.
            bootstrap_rollout(watched, tx).await;

            // Catch any bytes appended during the bootstrap scan.
            continue;
        }

        if len == watched.offset {
            return;
        }

        let start_offset = watched.offset;
        let mut file = match tokio::fs::File::open(&watched.path).await {
            Ok(f) => f,
            Err(e) => {
                debug!(path = %watched.path.display(), error = %e, "failed to open codex rollout");
                return;
            }
        };

        if let Err(e) = file.seek(std::io::SeekFrom::Start(watched.offset)).await {
            debug!(path = %watched.path.display(), error = %e, "failed to seek codex rollout");
            return;
        }

        let mut buf = Vec::new();
        if let Err(e) = file.read_to_end(&mut buf).await {
            debug!(path = %watched.path.display(), error = %e, "failed to read codex rollout");
            return;
        }
        watched.offset = start_offset + buf.len() as u64;

        watched.buffer.push_str(&String::from_utf8_lossy(&buf));

        // Process complete JSONL lines, leaving any partial line in `buffer`.
        let path = watched.path.clone();
        let state = &mut watched.state;
        let buffer = &mut watched.buffer;
        drain_jsonl_lines(buffer, |line| {
            let Some(value) =
                parse_json_line(path.as_path(), line, "malformed JSON in codex rollout")
            else {
                return;
            };
            let events = state.apply_line(&value);
            emit_events(tx, events);
        });

        return;
    }
}

#[derive(Debug, Default)]
struct DirtyRollouts {
    inner: Mutex<DirtyRolloutsInner>,
    notify: Notify,
}

#[derive(Debug, Default)]
struct DirtyRolloutsInner {
    paths: HashSet<PathBuf>,
    rescan: bool,
}

impl DirtyRollouts {
    fn mark(&self, path: PathBuf) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.paths.insert(path);
        }
        self.notify.notify_one();
    }

    fn mark_rescan(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.rescan = true;
        }
        self.notify.notify_one();
    }
    fn drain(&self) -> (bool, Vec<PathBuf>) {
        let Ok(mut inner) = self.inner.lock() else {
            return (false, Vec::new());
        };

        let rescan = inner.rescan;
        inner.rescan = false;
        let paths = inner.paths.drain().collect();
        (rescan, paths)
    }
}

async fn run(tx: broadcast::Sender<AgentEvent>) {
    let codex_paths = paths::CodexPaths::detect();
    let home = codex_paths.home;
    let root = codex_paths.sessions_root;
    let root_alt = codex_paths.sessions_root_alt;

    info!(path = %root.display(), "watching codex sessions");

    let dirty = Arc::new(DirtyRollouts::default());

    let dirty_cb = Arc::clone(&dirty);
    let sessions_root = root.clone();
    let sessions_root_alt = root_alt.clone();
    let mut watcher =
        match notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            let event = match res {
                Ok(e) => e,
                Err(err) => {
                    debug!(error = %err, "codex file watcher error");
                    dirty_cb.mark_rescan();
                    return;
                }
            };

            if matches!(event.kind, notify::event::EventKind::Other) {
                // Backends may emit `Other` when details are unreliable (buffer overflow).
                dirty_cb.mark_rescan();
                return;
            }

            if event.paths.is_empty() {
                // Some backends can emit events without paths (or with paths filtered
                // out internally). A rescan is cheap and ensures we don't stall.
                dirty_cb.mark_rescan();
                return;
            }

            for path in event.paths {
                if !path.starts_with(&sessions_root) && !path.starts_with(&sessions_root_alt) {
                    continue;
                }
                if paths::is_jsonl(&path) {
                    dirty_cb.mark(path);
                } else {
                    // Directory-level changes (new subdirs, renames) may not include rollout file paths.
                    dirty_cb.mark_rescan();
                }
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                warn!(error = %e, "failed to initialize codex file watcher");
                return;
            }
        };

    // Avoid overlapping watches on macOS (FSEvents) where a parent NonRecursive watch
    // can mask a child Recursive watch. Prefer watching the sessions root directly.
    let mut sessions_watched = false;
    let mut home_watched = false;

    if root.exists() {
        match watcher.watch(&root, RecursiveMode::Recursive) {
            Ok(()) => sessions_watched = true,
            Err(e) => {
                warn!(
                    path = %root.display(),
                    error = %e,
                    "failed to watch codex sessions; falling back to codex home"
                );
                if let Err(e) = watcher.watch(&home, RecursiveMode::Recursive) {
                    warn!(path = %home.display(), error = %e, "failed to watch codex home");
                    return;
                }
                home_watched = true;
            }
        }
    } else {
        // Watch Codex home (non-recursive) to detect `sessions/` creation.
        if let Err(e) = watcher.watch(&home, RecursiveMode::NonRecursive) {
            warn!(path = %home.display(), error = %e, "failed to watch codex home");
            return;
        }
        home_watched = true;
    }

    // Bootstrap: register all existing rollouts. For "recent" rollouts (mtime <= 10m),
    // emit a bounded replay to seed the HUD.
    let mut watched: HashMap<PathBuf, WatchedRollout> = HashMap::new();
    if root.exists() {
        for path in paths::read_dir_recursive(&root) {
            let mut session_id = paths::session_id_from_path(&path);
            let mut cwd = String::new();
            if let Some((meta_id, meta_cwd)) = read_first_session_meta(&path).await {
                session_id = meta_id;
                cwd = meta_cwd;
            }

            let mut rollout = WatchedRollout::new_existing(path.clone(), session_id, cwd, 0);
            if paths::modified_within(&path, VISIBILITY_WINDOW).await {
                bootstrap_rollout(&mut rollout, &tx).await;
                // Catch any bytes appended during bootstrap scan.
                tail_rollout(&mut rollout, &tx).await;
            } else {
                rollout.offset = file_len(&path).await.unwrap_or(0);
            }

            watched.insert(path.clone(), rollout);
        }
        debug!("codex rollouts registered: {}", watched.len());
    }

    let mut scan_tick = tokio::time::interval(FALLBACK_SCAN_INTERVAL);
    scan_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        let mut ticked = false;
        tokio::select! {
            biased;
            _ = scan_tick.tick() => ticked = true,
            _ = dirty.notify.notified() => {},
        }

        let (rescan, mut paths) = dirty.drain();
        if ticked && root.exists() {
            paths.extend(paths::scan_recent_rollouts(&root, VISIBILITY_WINDOW).await);
        }

        if rescan || ticked {
            if !sessions_watched && root.exists() {
                match watcher.watch(&root, RecursiveMode::Recursive) {
                    Ok(()) => {
                        sessions_watched = true;
                        if home_watched {
                            if let Err(e) = watcher.unwatch(&home) {
                                warn!(
                                    path = %home.display(),
                                    error = %e,
                                    "failed to unwatch codex home"
                                );
                            } else {
                                home_watched = false;
                            }
                        }
                    }
                    Err(e) => warn!(
                        path = %root.display(),
                        error = %e,
                        "failed to watch codex sessions"
                    ),
                }
            }

            if rescan && root.exists() {
                paths.extend(paths::read_dir_recursive(&root));
            }
        }

        // Stable order reduces jitter when many files are active.
        paths.sort();
        paths.dedup();

        if paths.is_empty() {
            continue;
        }

        for path in paths {
            if !watched.contains_key(&path) {
                let mut session_id = paths::session_id_from_path(&path);
                let mut cwd = String::new();
                if let Some((meta_id, meta_cwd)) = read_first_session_meta(&path).await {
                    session_id = meta_id;
                    cwd = meta_cwd;
                }

                info!(path = %path.display(), %session_id, "discovered new codex rollout");
                watched.insert(
                    path.clone(),
                    WatchedRollout::new_fresh(path.clone(), session_id, cwd),
                );
            }

            if let Some(w) = watched.get_mut(&path) {
                if !w.state.session_emitted {
                    bootstrap_rollout(w, &tx).await;
                }
                if w.state.session_emitted {
                    tail_rollout(w, &tx).await;
                } else {
                    // Keep the cursor pinned to EOF so we don't accidentally replay old rollouts.
                    w.offset = file_len(&path).await.unwrap_or(w.offset);
                }
            }
        }
    }
}

/// Spawn the Codex session rollout watcher.
pub fn spawn(tx: broadcast::Sender<AgentEvent>) {
    tokio::spawn(async move {
        run(tx).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::{FileTime, set_file_mtime};
    use serde_json::json;
    use std::time::SystemTime;
    use tempfile::TempDir;

    fn write_jsonl(path: &Path, lines: &[Value]) {
        let mut out = String::new();
        for line in lines {
            out.push_str(&serde_json::to_string(line).unwrap());
            out.push('\n');
        }
        std::fs::write(path, out).unwrap();
    }

    fn drain_rx(rx: &mut broadcast::Receiver<AgentEvent>) -> Vec<AgentEvent> {
        let mut out = Vec::new();
        loop {
            match rx.try_recv() {
                Ok(ev) => out.push(ev),
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                Err(broadcast::error::TryRecvError::Closed) => break,
            }
        }
        out
    }

    #[tokio::test]
    async fn bootstrap_replays_last_four_events_and_keeps_latest_session_name() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("rollout-2026-02-14-sess_1.jsonl");

        let set_name_args = serde_json::json!({ "cmd": "aura set-name \"named session\"" });
        let set_name_args_str = serde_json::to_string(&set_name_args).unwrap();
        let rg_args = serde_json::json!({ "cmd": "rg -n foo src" });
        let rg_args_str = serde_json::to_string(&rg_args).unwrap();

        write_jsonl(
            &path,
            &[
                json!({
                    "type": "session_meta",
                    "payload": { "id": "sess_1", "cwd": "/tmp/project" }
                }),
                json!({
                    "type": "response_item",
                    "payload": {
                        "type": "function_call",
                        "call_id": "call_set",
                        "name": "exec_command",
                        "arguments": set_name_args_str
                    }
                }),
                json!({ "type": "event_msg", "payload": { "type": "task_started" } }),
                json!({
                    "type": "response_item",
                    "payload": {
                        "type": "function_call",
                        "call_id": "call_rg",
                        "name": "exec_command",
                        "arguments": rg_args_str
                    }
                }),
                json!({
                    "type": "response_item",
                    "payload": { "type": "function_call_output", "call_id": "call_rg" }
                }),
                json!({ "type": "event_msg", "payload": { "type": "task_complete" } }),
                json!({ "type": "event_msg", "payload": { "type": "context_compacted" } }),
                json!({ "type": "event_msg", "payload": { "type": "request_user_input" } }),
            ],
        );

        let (tx, mut rx) = broadcast::channel(32);
        let mut watched =
            WatchedRollout::new_existing(path.clone(), "fallback".to_string(), "".to_string(), 0);

        bootstrap_rollout(&mut watched, &tx).await;

        let events = drain_rx(&mut rx);
        assert_eq!(events.len(), 6);

        match &events[0] {
            AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent,
            } => {
                assert_eq!(session_id, "sess_1");
                assert_eq!(cwd, "/tmp/project");
                assert_eq!(agent, &AgentType::Codex);
            }
            other => panic!("unexpected event: {other:?}"),
        }

        match &events[1] {
            AgentEvent::SessionNameUpdated { session_id, name } => {
                assert_eq!(session_id, "sess_1");
                assert_eq!(name, "named session");
            }
            other => panic!("unexpected event: {other:?}"),
        }

        match (&events[2], &events[3], &events[4], &events[5]) {
            (
                AgentEvent::ToolCompleted {
                    session_id,
                    cwd,
                    tool_id,
                },
                AgentEvent::Idle {
                    session_id: s2,
                    cwd: c2,
                },
                AgentEvent::Compacting {
                    session_id: s3,
                    cwd: c3,
                },
                AgentEvent::WaitingForInput {
                    session_id: s4,
                    cwd: c4,
                    message: _,
                },
            ) => {
                assert_eq!(session_id, "sess_1");
                assert_eq!(cwd, "/tmp/project");
                assert_eq!(tool_id, "call_rg");

                assert_eq!(s2, "sess_1");
                assert_eq!(c2, "/tmp/project");
                assert_eq!(s3, "sess_1");
                assert_eq!(c3, "/tmp/project");
                assert_eq!(s4, "sess_1");
                assert_eq!(c4, "/tmp/project");
            }
            other => panic!("unexpected replay events: {other:?}"),
        }

        assert!(watched.state.session_emitted);
        assert_eq!(watched.state.session_id, "sess_1");
        assert_eq!(watched.state.cwd, "/tmp/project");

        let len = std::fs::metadata(&path).unwrap().len();
        assert_eq!(watched.offset, len);
    }

    #[tokio::test]
    async fn bootstrap_skips_stale_rollout_by_mtime() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("rollout-2026-02-14-sess_1.jsonl");
        write_jsonl(
            &path,
            &[json!({
                "type": "session_meta",
                "payload": { "id": "sess_1", "cwd": "/tmp/project" }
            })],
        );

        let old = SystemTime::now() - (VISIBILITY_WINDOW + Duration::from_secs(1));
        set_file_mtime(&path, FileTime::from_system_time(old)).unwrap();

        let (tx, mut rx) = broadcast::channel(8);
        let mut watched =
            WatchedRollout::new_existing(path.clone(), "fallback".to_string(), "".to_string(), 0);

        bootstrap_rollout(&mut watched, &tx).await;

        let events = drain_rx(&mut rx);
        assert!(events.is_empty());
        assert!(!watched.state.session_emitted);

        let len = std::fs::metadata(&path).unwrap().len();
        assert_eq!(watched.offset, len);
        assert!(watched.buffer.is_empty());
    }

    #[tokio::test]
    async fn bootstrap_counts_activity_towards_replay_limit() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("rollout-2026-02-14-sess_1.jsonl");

        let rg_args = serde_json::json!({ "cmd": "rg -n foo src" });
        let rg_args_str = serde_json::to_string(&rg_args).unwrap();

        write_jsonl(
            &path,
            &[
                json!({
                    "type": "session_meta",
                    "payload": { "id": "sess_1", "cwd": "/tmp/project" }
                }),
                json!({
                    "type": "response_item",
                    "payload": {
                        "type": "function_call",
                        "call_id": "call_rg",
                        "name": "exec_command",
                        "arguments": rg_args_str
                    }
                }),
                json!({
                    "type": "response_item",
                    "payload": { "type": "function_call_output", "call_id": "call_rg" }
                }),
                json!({ "type": "event_msg", "payload": { "type": "user_message" } }),
                json!({ "type": "event_msg", "payload": { "type": "agent_message" } }),
                json!({ "type": "event_msg", "payload": { "type": "task_started" } }),
                json!({
                    "type": "response_item",
                    "payload": { "type": "message", "content": [] }
                }),
            ],
        );

        let (tx, mut rx) = broadcast::channel(16);
        let mut watched =
            WatchedRollout::new_existing(path.clone(), "fallback".to_string(), "".to_string(), 0);

        bootstrap_rollout(&mut watched, &tx).await;

        let events = drain_rx(&mut rx);
        assert_eq!(events.len(), 5, "SessionStarted + 4 Activity events");

        match &events[0] {
            AgentEvent::SessionStarted {
                session_id,
                cwd,
                agent,
            } => {
                assert_eq!(session_id, "sess_1");
                assert_eq!(cwd, "/tmp/project");
                assert_eq!(agent, &AgentType::Codex);
            }
            other => panic!("unexpected event: {other:?}"),
        }

        for ev in &events[1..] {
            match ev {
                AgentEvent::Activity { session_id, cwd } => {
                    assert_eq!(session_id, "sess_1");
                    assert_eq!(cwd, "/tmp/project");
                }
                other => panic!("expected Activity event, got: {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn truncation_re_bootstraps_instead_of_replaying_full_history() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("rollout-2026-02-14-sess_1.jsonl");

        let set_name_args = serde_json::json!({ "cmd": "aura set-name \"named session\"" });
        let set_name_args_str = serde_json::to_string(&set_name_args).unwrap();
        let rg_args = serde_json::json!({ "cmd": "rg -n foo src" });
        let rg_args_str = serde_json::to_string(&rg_args).unwrap();

        write_jsonl(
            &path,
            &[
                json!({
                    "type": "session_meta",
                    "payload": { "id": "sess_1", "cwd": "/tmp/project" }
                }),
                json!({
                    "type": "response_item",
                    "payload": {
                        "type": "function_call",
                        "call_id": "call_set",
                        "name": "exec_command",
                        "arguments": set_name_args_str
                    }
                }),
                json!({ "type": "event_msg", "payload": { "type": "task_started" } }),
                json!({
                    "type": "response_item",
                    "payload": {
                        "type": "function_call",
                        "call_id": "call_rg",
                        "name": "exec_command",
                        "arguments": rg_args_str
                    }
                }),
                json!({
                    "type": "response_item",
                    "payload": { "type": "function_call_output", "call_id": "call_rg" }
                }),
                json!({ "type": "event_msg", "payload": { "type": "task_complete" } }),
                json!({ "type": "event_msg", "payload": { "type": "context_compacted" } }),
                json!({ "type": "event_msg", "payload": { "type": "request_user_input" } }),
            ],
        );

        let file_len = std::fs::metadata(&path).unwrap().len();
        let (tx, mut rx) = broadcast::channel(32);

        // Simulate "we were at a later offset" and the file got truncated.
        let mut watched = WatchedRollout::new_existing(
            path.clone(),
            "fallback".to_string(),
            "".to_string(),
            file_len + 10,
        );
        watched.state.session_emitted = true;

        tail_rollout(&mut watched, &tx).await;

        let events = drain_rx(&mut rx);
        assert_eq!(events.len(), 6);

        match (&events[0], &events[1]) {
            (
                AgentEvent::SessionStarted {
                    session_id,
                    cwd,
                    agent,
                },
                AgentEvent::SessionNameUpdated {
                    session_id: s2,
                    name,
                },
            ) => {
                assert_eq!(session_id, "sess_1");
                assert_eq!(cwd, "/tmp/project");
                assert_eq!(agent, &AgentType::Codex);
                assert_eq!(s2, "sess_1");
                assert_eq!(name, "named session");
            }
            other => panic!("unexpected bootstrap events: {other:?}"),
        }

        // Bounded replay tail stays small (we should not replay the entire file).
        assert_eq!(watched.offset, file_len);
        assert!(watched.state.session_emitted);
    }
}
