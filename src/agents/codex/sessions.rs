//! Codex session rollout JSONL watcher.
//!
//! This module watches Codex session rollouts under `~/.codex/sessions/**.jsonl`
//! (or `$CODEX_HOME/sessions`) and emits [`AgentEvent`]s on a best-effort stream.
//!
//! This integration works even when Codex is started externally (e.g. via `codex` CLI)
//! because it consumes Codex's public session rollout files.

use crate::{AgentEvent, AgentType};
use chrono::{Datelike, Local, NaiveDate};
use notify::{RecursiveMode, Watcher};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader};
use tokio::sync::{Notify, broadcast};
use tracing::{debug, info, trace, warn};

const BOOTSTRAP_REPLAY_MAX_EVENTS: usize = 4;
const VISIBILITY_WINDOW: Duration = Duration::from_secs(10 * 60);
const FALLBACK_SCAN_INTERVAL: Duration = Duration::from_secs(2);

fn codex_home() -> PathBuf {
    if let Some(home) = std::env::var_os("CODEX_HOME") {
        return PathBuf::from(home);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
}

fn is_jsonl(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jsonl"))
}

fn session_id_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Common format: `rollout-<timestamp>-<uuid>.jsonl` (uuid has 5 `-` segments).
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() >= 6 {
        return parts[parts.len() - 5..].join("-");
    }

    stem.to_string()
}

fn read_dir_recursive(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        let entries = match std::fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let p = entry.path();
            match entry.file_type() {
                Ok(t) if t.is_dir() => stack.push(p),
                Ok(t) if t.is_file() && is_jsonl(&p) => out.push(p),
                _ => {}
            }
        }
    }

    out
}

fn date_dir(root: &Path, date: NaiveDate) -> PathBuf {
    root.join(format!("{:04}", date.year()))
        .join(format!("{:02}", date.month()))
        .join(format!("{:02}", date.day()))
}

fn max_numeric_child_dir(parent: &Path, len: usize) -> Option<PathBuf> {
    let entries = std::fs::read_dir(parent).ok()?;
    let mut best: Option<(String, PathBuf)> = None;

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.len() != len || !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        match &best {
            Some((best_name, _)) if name <= best_name.as_str() => {}
            _ => best = Some((name.to_string(), entry.path())),
        }
    }

    best.map(|(_, path)| path)
}

fn latest_day_dir(root: &Path) -> Option<PathBuf> {
    let year = max_numeric_child_dir(root, 4)?;
    let month = max_numeric_child_dir(&year, 2)?;
    max_numeric_child_dir(&month, 2)
}

fn candidate_scan_dirs(root: &Path) -> Vec<PathBuf> {
    let today = Local::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);

    let mut dirs = vec![
        root.to_path_buf(),
        date_dir(root, today),
        date_dir(root, yesterday),
    ];
    if let Some(latest) = latest_day_dir(root) {
        dirs.push(latest);
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

async fn read_dir_jsonl(dir: &Path) -> Vec<PathBuf> {
    let Ok(mut rd) = tokio::fs::read_dir(dir).await else {
        return Vec::new();
    };

    let mut out = Vec::new();
    while let Ok(Some(entry)) = rd.next_entry().await {
        let path = entry.path();
        let Ok(file_type) = entry.file_type().await else {
            continue;
        };
        if file_type.is_file() && is_jsonl(&path) {
            out.push(path);
        }
    }

    out
}

async fn scan_recent_rollouts(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();

    for dir in candidate_scan_dirs(root) {
        for path in read_dir_jsonl(&dir).await {
            if modified_within(&path, VISIBILITY_WINDOW).await {
                out.push(path);
            }
        }
    }

    out
}

fn json_string_field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

fn truncate_owned(value: &str, max: usize) -> String {
    crate::agents::truncate(value, max).to_string()
}

fn first_shell_token(command: &str) -> Option<String> {
    command
        .split_whitespace()
        .next()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(ToString::to_string)
}

fn tool_label_from_args(tool_name: &str, args_json: &Value) -> Option<String> {
    // Common shapes across built-in tools.
    if let Some(path) = json_string_field(args_json, &["path", "file_path", "filePath"]) {
        Some(crate::agents::short_path(path))
    } else if let Some(cmd) = json_string_field(args_json, &["cmd", "command"]) {
        Some(truncate_owned(cmd, 60))
    } else if let Some(query) = json_string_field(args_json, &["query", "q"]) {
        Some(truncate_owned(query, 60))
    } else if tool_name.starts_with("mcp__") {
        // MCP payloads often put useful UX context in `arguments` as structured JSON.
        json_string_field(args_json, &["arguments", "input", "message"])
            .map(|v| truncate_owned(v, 60))
    } else {
        None
    }
}

fn parse_json_string(value: &Value) -> Option<Value> {
    let s = value.as_str()?;
    serde_json::from_str(s).ok()
}

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

async fn modified_within(path: &Path, window: Duration) -> bool {
    let Ok(meta) = tokio::fs::metadata(path).await else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::ZERO);
    age <= window
}

#[derive(Debug, Clone)]
struct RolloutState {
    session_id: String,
    cwd: String,
    session_emitted: bool,
    web_search_seq: u64,
}

impl RolloutState {
    fn new(session_id: String, cwd: String) -> Self {
        Self {
            session_id,
            cwd,
            session_emitted: false,
            web_search_seq: 0,
        }
    }

    fn ensure_session_event(&mut self) -> Option<AgentEvent> {
        if self.session_emitted {
            return None;
        }
        self.session_emitted = true;
        Some(AgentEvent::SessionStarted {
            session_id: self.session_id.clone(),
            cwd: self.cwd.clone(),
            agent: AgentType::Codex,
        })
    }

    fn maybe_update_cwd(&mut self, next_cwd: &str) -> Option<AgentEvent> {
        let next_cwd = next_cwd.trim();
        if next_cwd.is_empty() || next_cwd == self.cwd {
            return None;
        }
        self.cwd = next_cwd.to_string();
        self.session_emitted.then(|| AgentEvent::SessionStarted {
            session_id: self.session_id.clone(),
            cwd: self.cwd.clone(),
            agent: AgentType::Codex,
        })
    }

    fn apply_line(&mut self, value: &Value) -> Vec<AgentEvent> {
        let mut events = Vec::new();
        let line_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let timestamp = value
            .get("timestamp")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty());

        match line_type {
            "session_meta" => {
                if let Some(payload) = value.get("payload").and_then(|v| v.as_object()) {
                    if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                        // Prefer session_meta id when we haven't emitted events yet. (It should
                        // match the filename suffix; this also handles any parsing drift.)
                        if !self.session_emitted {
                            self.session_id = id.to_string();
                        } else if self.session_id != id {
                            warn!(
                                current = %self.session_id,
                                meta = %id,
                                "codex rollout session id changed after emission; ignoring"
                            );
                        }
                    }
                    if let Some(cwd) = payload.get("cwd").and_then(|v| v.as_str()) {
                        if let Some(update) = self.maybe_update_cwd(cwd) {
                            events.push(update);
                        } else if self.cwd.is_empty() {
                            self.cwd = cwd.to_string();
                        }
                    }
                }

                if let Some(start) = self.ensure_session_event() {
                    events.push(start);
                }
                return events;
            }
            "turn_context" => {
                if let Some(payload) = value.get("payload")
                    && let Some(cwd) = payload.get("cwd").and_then(|v| v.as_str())
                {
                    if let Some(update) = self.maybe_update_cwd(cwd) {
                        events.push(update);
                    } else if self.cwd.is_empty() {
                        self.cwd = cwd.to_string();
                    }
                }
                // No user-facing state change; treat as activity at most.
                return events;
            }
            _ => {}
        }

        if let Some(start) = self.ensure_session_event() {
            events.push(start);
        }

        match line_type {
            "event_msg" => {
                let payload = value.get("payload").unwrap_or(&Value::Null);
                let msg_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match msg_type {
                    "task_started"
                    | "user_message"
                    | "agent_message"
                    | "entered_review_mode"
                    | "exited_review_mode" => {
                        events.push(AgentEvent::Activity {
                            session_id: self.session_id.clone(),
                            cwd: self.cwd.clone(),
                        });
                    }
                    "context_compacted" => {
                        events.push(AgentEvent::Compacting {
                            session_id: self.session_id.clone(),
                            cwd: self.cwd.clone(),
                        });
                    }
                    "task_complete" | "turn_aborted" => {
                        events.push(AgentEvent::Idle {
                            session_id: self.session_id.clone(),
                            cwd: self.cwd.clone(),
                        });
                    }
                    "request_user_input" => {
                        events.push(AgentEvent::WaitingForInput {
                            session_id: self.session_id.clone(),
                            cwd: self.cwd.clone(),
                            message: None,
                        });
                    }
                    // High-frequency / non-UX events.
                    "token_count" | "agent_reasoning" => {}
                    _ => {}
                }
            }
            "compacted" => {
                events.push(AgentEvent::Compacting {
                    session_id: self.session_id.clone(),
                    cwd: self.cwd.clone(),
                });
            }
            "response_item" => {
                let payload = value.get("payload").unwrap_or(&Value::Null);
                self.apply_response_item(payload, timestamp, &mut events);
            }
            // Older rollouts may emit response item variants directly without a `payload` wrapper.
            "function_call"
            | "function_call_output"
            | "custom_tool_call"
            | "custom_tool_call_output"
            | "message"
            | "reasoning"
            | "web_search_call"
            | "ghost_snapshot" => {
                self.apply_response_item(value, timestamp, &mut events);
            }
            _ => {}
        }

        events
    }

    fn apply_response_item(
        &mut self,
        payload: &Value,
        timestamp: Option<&str>,
        events: &mut Vec<AgentEvent>,
    ) {
        let payload_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match payload_type {
            "function_call" => {
                let tool_id =
                    json_string_field(payload, &["call_id", "callId"]).unwrap_or("unknown");
                let tool_name_raw = json_string_field(payload, &["name"]).unwrap_or("tool");
                let args = payload.get("arguments");
                let args_json = args.and_then(parse_json_string);

                let (tool_name, tool_label, session_name) = if tool_name_raw == "exec_command" {
                    let cmd = args_json
                        .as_ref()
                        .and_then(|v| json_string_field(v, &["cmd"]))
                        .unwrap_or("");
                    let name = first_shell_token(cmd).unwrap_or_else(|| "exec".to_string());
                    let label = (!cmd.is_empty()).then(|| truncate_owned(cmd, 60));
                    let session_name = crate::agents::parse_aura_set_name_command(cmd);
                    (name, label, session_name)
                } else {
                    let label = args_json
                        .as_ref()
                        .and_then(|v| tool_label_from_args(tool_name_raw, v));
                    (tool_name_raw.to_string(), label, None)
                };

                events.push(AgentEvent::ToolStarted {
                    session_id: self.session_id.clone(),
                    cwd: self.cwd.clone(),
                    tool_id: tool_id.to_string(),
                    tool_name,
                    tool_label,
                });
                if let Some(name) = session_name {
                    events.push(AgentEvent::SessionNameUpdated {
                        session_id: self.session_id.clone(),
                        name,
                    });
                }
            }
            "function_call_output" => {
                let tool_id =
                    json_string_field(payload, &["call_id", "callId"]).unwrap_or("unknown");
                events.push(AgentEvent::ToolCompleted {
                    session_id: self.session_id.clone(),
                    cwd: self.cwd.clone(),
                    tool_id: tool_id.to_string(),
                });
            }
            "custom_tool_call" => {
                let tool_id =
                    json_string_field(payload, &["call_id", "callId"]).unwrap_or("unknown");
                let tool_name = json_string_field(payload, &["name"]).unwrap_or("custom_tool");
                events.push(AgentEvent::ToolStarted {
                    session_id: self.session_id.clone(),
                    cwd: self.cwd.clone(),
                    tool_id: tool_id.to_string(),
                    tool_name: tool_name.to_string(),
                    tool_label: None,
                });
            }
            "custom_tool_call_output" => {
                let tool_id =
                    json_string_field(payload, &["call_id", "callId"]).unwrap_or("unknown");
                events.push(AgentEvent::ToolCompleted {
                    session_id: self.session_id.clone(),
                    cwd: self.cwd.clone(),
                    tool_id: tool_id.to_string(),
                });
            }
            "web_search_call" => {
                // This response item is already `status: completed` in observed rollouts, so we
                // emit an immediate start+complete to show it as "recent activity" for a moment.
                let tool_id = timestamp
                    .map(|t| format!("web_search:{t}"))
                    .unwrap_or_else(|| {
                        self.web_search_seq += 1;
                        format!("web_search:seq:{}", self.web_search_seq)
                    });

                let query = payload
                    .get("action")
                    .and_then(|v| v.get("query"))
                    .and_then(|v| v.as_str())
                    .map(|q| truncate_owned(q, 60));

                events.push(AgentEvent::ToolStarted {
                    session_id: self.session_id.clone(),
                    cwd: self.cwd.clone(),
                    tool_id: tool_id.clone(),
                    tool_name: "WebSearch".to_string(),
                    tool_label: query,
                });
                events.push(AgentEvent::ToolCompleted {
                    session_id: self.session_id.clone(),
                    cwd: self.cwd.clone(),
                    tool_id,
                });
            }
            "message" | "reasoning" => {
                events.push(AgentEvent::Activity {
                    session_id: self.session_id.clone(),
                    cwd: self.cwd.clone(),
                });
            }
            // Not surfaced in the HUD (state is derived from other messages).
            "ghost_snapshot" => {}
            _ => {}
        }
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
    if !modified_within(&watched.path, VISIBILITY_WINDOW).await {
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
    let home_raw = codex_home();
    let home = std::fs::canonicalize(&home_raw).unwrap_or_else(|_| home_raw.clone());
    let root = home.join("sessions");
    let root_raw = home_raw.join("sessions");

    info!(path = %root.display(), "watching codex sessions");

    let dirty = Arc::new(DirtyRollouts::default());

    let dirty_cb = Arc::clone(&dirty);
    let sessions_root = root.clone();
    let sessions_root_alt = root_raw.clone();
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
                if is_jsonl(&path) {
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
        for path in read_dir_recursive(&root) {
            let mut session_id = session_id_from_path(&path);
            let mut cwd = String::new();
            if let Some((meta_id, meta_cwd)) = read_first_session_meta(&path).await {
                session_id = meta_id;
                cwd = meta_cwd;
            }

            let mut rollout = WatchedRollout::new_existing(path.clone(), session_id, cwd, 0);
            if modified_within(&path, VISIBILITY_WINDOW).await {
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
            paths.extend(scan_recent_rollouts(&root).await);
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
                paths.extend(read_dir_recursive(&root));
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
                let mut session_id = session_id_from_path(&path);
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

    #[test]
    fn session_meta_emits_session_started() {
        let mut state = RolloutState::new("fallback".to_string(), "".to_string());
        let events = state.apply_line(&json!({
            "type": "session_meta",
            "payload": { "id": "sess_1", "cwd": "/tmp/project" }
        }));

        assert_eq!(events.len(), 1);
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
    }

    #[test]
    fn exec_command_function_call_maps_to_tool_with_cmd_label() {
        let mut state = RolloutState::new("sess_1".to_string(), "/tmp".to_string());
        let _ = state.ensure_session_event();

        let events = state.apply_line(&json!({
            "type": "response_item",
            "timestamp": "2026-02-14T00:00:00Z",
            "payload": {
                "type": "function_call",
                "call_id": "call_1",
                "name": "exec_command",
                "arguments": "{\"cmd\":\"rg -n foo src\"}"
            }
        }));

        assert_eq!(events.len(), 1);
        match &events[0] {
            AgentEvent::ToolStarted {
                tool_id,
                tool_name,
                tool_label,
                ..
            } => {
                assert_eq!(tool_id, "call_1");
                assert_eq!(tool_name, "rg");
                assert_eq!(tool_label.as_deref(), Some("rg -n foo src"));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn mcp_tool_call_extracts_query_label() {
        let mut state = RolloutState::new("sess_1".to_string(), "/tmp".to_string());
        let _ = state.ensure_session_event();

        let events = state.apply_line(&json!({
            "type": "response_item",
            "payload": {
                "type": "function_call",
                "call_id": "call_2",
                "name": "mcp__nowledge-mem__memory_search",
                "arguments": "{\"query\":\"hello world\"}"
            }
        }));

        assert_eq!(events.len(), 1);
        match &events[0] {
            AgentEvent::ToolStarted {
                tool_name,
                tool_label,
                ..
            } => {
                assert_eq!(tool_name, "mcp__nowledge-mem__memory_search");
                assert_eq!(tool_label.as_deref(), Some("hello world"));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn web_search_call_emits_immediate_tool_lifecycle() {
        let mut state = RolloutState::new("sess_1".to_string(), "/tmp".to_string());
        let _ = state.ensure_session_event();

        let events = state.apply_line(&json!({
            "type": "response_item",
            "timestamp": "2026-02-14T00:00:00Z",
            "payload": {
                "type": "web_search_call",
                "status": "completed",
                "action": { "type": "search", "query": "thread/loaded/list vs thread/list" }
            }
        }));

        assert_eq!(events.len(), 2);
        match (&events[0], &events[1]) {
            (
                AgentEvent::ToolStarted {
                    tool_id: start_id,
                    tool_name,
                    tool_label,
                    ..
                },
                AgentEvent::ToolCompleted {
                    tool_id: end_id, ..
                },
            ) => {
                assert_eq!(tool_name, "WebSearch");
                assert_eq!(
                    tool_label.as_deref(),
                    Some("thread/loaded/list vs thread/list")
                );
                assert_eq!(start_id, end_id);
            }
            other => panic!("unexpected events: {other:?}"),
        }
    }

    #[test]
    fn exec_command_set_name_emits_session_name_updated() {
        let mut state = RolloutState::new("sess_1".to_string(), "/tmp".to_string());
        let _ = state.ensure_session_event();

        let args = serde_json::json!({ "cmd": "aura set-name \"my session\"" });
        let args_str = serde_json::to_string(&args).unwrap();

        let events = state.apply_line(&json!({
            "type": "response_item",
            "payload": {
                "type": "function_call",
                "call_id": "call_1",
                "name": "exec_command",
                "arguments": args_str
            }
        }));

        assert_eq!(events.len(), 2);
        match (&events[0], &events[1]) {
            (
                AgentEvent::ToolStarted {
                    tool_id,
                    tool_name,
                    tool_label,
                    ..
                },
                AgentEvent::SessionNameUpdated { session_id, name },
            ) => {
                assert_eq!(tool_id, "call_1");
                assert_eq!(tool_name, "aura");
                assert_eq!(tool_label.as_deref(), Some("aura set-name \"my session\""));
                assert_eq!(session_id, "sess_1");
                assert_eq!(name, "my session");
            }
            other => panic!("unexpected events: {other:?}"),
        }
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
