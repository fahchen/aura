//! Codex app-server JSON-RPC client
//!
//! Spawns `codex app-server` as a subprocess and communicates via JSONL over stdio.
//! Discovers threads via `thread/list`, resumes them for event subscriptions,
//! and maps notifications to `AgentEvent` for the `SessionRegistry`.
//!
//! Protocol: JSON-RPC 2.0 without the `"jsonrpc":"2.0"` header on the wire.

use crate::{AgentEvent, AgentType};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, info, trace, warn};

use crate::registry::SessionRegistry;

/// How often to poll for new threads
const THREAD_POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Extract thread ID from JSON params, with optional fallback object.
fn thread_id(params: &Value, fallback: Option<&Value>) -> String {
    params
        .get("threadId")
        .or_else(|| fallback.and_then(|f| f.get("threadId")))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Client for Codex app-server
pub(crate) struct CodexClient {
    child: Child,
    writer: tokio::process::ChildStdin,
    next_id: u64,
    registry: Arc<Mutex<SessionRegistry>>,
    dirty: Arc<AtomicBool>,
    /// Thread IDs we've already resumed/subscribed to
    known_threads: HashSet<String>,
}

impl CodexClient {
    /// Spawn `codex app-server` and perform the initialize handshake.
    pub(crate) async fn connect(
        registry: Arc<Mutex<SessionRegistry>>,
        dirty: Arc<AtomicBool>,
    ) -> Option<(Self, tokio::io::Lines<BufReader<tokio::process::ChildStdout>>)> {
        let mut child = match Command::new("codex")
            .arg("app-server")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                debug!("codex app-server not available: {}", e);
                return None;
            }
        };

        let writer = child.stdin.take()?;
        let stdout = child.stdout.take()?;
        let reader = BufReader::new(stdout).lines();

        let mut client = Self {
            child,
            writer,
            next_id: 0,
            registry,
            dirty,
            known_threads: HashSet::new(),
        };

        // Perform initialize handshake
        if !client.initialize(&reader).await {
            warn!("codex app-server initialize handshake failed");
            let _ = client.child.kill().await;
            let _ = client.child.wait().await;
            return None;
        }

        info!("codex app-server connected");
        Some((client, reader))
    }

    async fn initialize(
        &mut self,
        _reader: &tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    ) -> bool {
        let init_req = json!({
            "method": "initialize",
            "id": self.next_id(),
            "params": {
                "clientInfo": {
                    "name": "aura",
                    "title": "Aura HUD",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        if self.send(&init_req).await.is_err() {
            return false;
        }

        // Send initialized notification
        let initialized = json!({
            "method": "initialized",
            "params": {}
        });
        self.send(&initialized).await.is_ok()
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    async fn send(&mut self, msg: &Value) -> Result<(), std::io::Error> {
        let line = serde_json::to_string(msg)?;
        trace!("codex -> {}", line);
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await
    }

    /// Send a thread/list request to discover active threads.
    pub(crate) async fn list_threads(&mut self) -> Result<(), std::io::Error> {
        let req = json!({
            "method": "thread/list",
            "id": self.next_id(),
            "params": {
                "limit": 50,
                "sortKey": "updated_at"
            }
        });
        self.send(&req).await
    }

    /// Resume a thread to subscribe to its events.
    pub(crate) async fn resume_thread(&mut self, thread_id: &str) -> Result<(), std::io::Error> {
        if self.known_threads.contains(thread_id) {
            return Ok(());
        }
        let req = json!({
            "method": "thread/resume",
            "id": self.next_id(),
            "params": {
                "threadId": thread_id
            }
        });
        self.send(&req).await?;
        self.known_threads.insert(thread_id.to_string());
        Ok(())
    }

    /// Process a line from the app-server stdout.
    pub(crate) async fn handle_message(&mut self, line: &str) {
        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                warn!("malformed JSON from codex: {e}");
                return;
            }
        };

        trace!("codex <- {}", line);

        // Check if this is a response to one of our requests
        if let Some(id) = msg.get("id") {
            if let Some(result) = msg.get("result") {
                self.handle_response(id, result).await;
            }
            // If it has both `id` and `method`, it's a server request (e.g., requestApproval)
            if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
                let request_id = match id.as_u64() {
                    Some(n) => n,
                    None => {
                        warn!("codex server request has non-numeric id: {id}, skipping");
                        return;
                    }
                };
                self.handle_server_request(
                    request_id,
                    method,
                    msg.get("params").unwrap_or(&Value::Null),
                )
                .await;
            }
            return;
        }

        // Notification (no id)
        if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
            let params = msg.get("params").unwrap_or(&Value::Null);
            self.handle_notification(method, params);
        }
    }

    async fn handle_response(&mut self, _id: &Value, result: &Value) {
        // Handle thread/list response
        if let Some(data) = result.get("data").and_then(|d| d.as_array()) {
            for thread in data {
                if let Some(thread_id) = thread.get("id").and_then(|id| id.as_str()) {
                    let preview = thread
                        .get("preview")
                        .and_then(|p| p.as_str())
                        .unwrap_or("");

                    // Register thread as a session
                    let event = AgentEvent::SessionStarted {
                        session_id: thread_id.to_string(),
                        cwd: String::new(),
                        agent: AgentType::Codex,
                    };
                    self.emit_event(event);

                    if !preview.is_empty() {
                        let name_event = AgentEvent::SessionNameUpdated {
                            session_id: thread_id.to_string(),
                            name: super::truncate(preview, 40).to_string(),
                        };
                        self.emit_event(name_event);
                    }

                    // Resume thread to subscribe to its events
                    if let Err(e) = self.resume_thread(thread_id).await {
                        debug!("failed to resume thread {}: {}", thread_id, e);
                    }
                }
            }
        }
    }

    async fn handle_server_request(&mut self, id: u64, method: &str, params: &Value) {
        match method {
            "item/commandExecution/requestApproval"
            | "item/fileChange/requestApproval"
            | "item/tool/call" => {
                // Map to NeedsAttention
                let thread_id = thread_id(params, None);
                let tool_name = if method.contains("commandExecution") {
                    params
                        .get("command")
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                        .map(String::from)
                } else if method.contains("fileChange") {
                    Some("FileChange".to_string())
                } else {
                    params
                        .get("toolName")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                };

                let event = AgentEvent::NeedsAttention {
                    session_id: thread_id,
                    cwd: params
                        .get("cwd")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    message: tool_name,
                };
                self.emit_event(event);

                // Don't auto-respond — let the user handle approval in their Codex UI.
                // The HUD just shows the attention state.
                let _ = id;
            }
            "tool/requestUserInput" => {
                let thread_id = thread_id(params, None);
                let event = AgentEvent::WaitingForInput {
                    session_id: thread_id,
                    cwd: String::new(),
                    message: params
                        .get("message")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                };
                self.emit_event(event);
            }
            _ => {
                trace!("unhandled server request: {}", method);
            }
        }
    }

    fn handle_notification(&mut self, method: &str, params: &Value) {
        match method {
            "thread/started" => {
                if let Some(thread) = params.get("thread") {
                    let thread_id = thread
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let event = AgentEvent::SessionStarted {
                        session_id: thread_id.to_string(),
                        cwd: String::new(),
                        agent: AgentType::Codex,
                    };
                    self.emit_event(event);
                }
            }

            "turn/started" => {
                if let Some(turn) = params.get("turn") {
                    let thread_id = thread_id(params, Some(turn));
                    let event = AgentEvent::Activity {
                        session_id: thread_id,
                        cwd: String::new(),
                    };
                    self.emit_event(event);
                }
            }

            "turn/completed" => {
                let thread_id = thread_id(params, params.get("turn"));
                let event = AgentEvent::Idle {
                    session_id: thread_id,
                    cwd: String::new(),
                };
                self.emit_event(event);
            }

            "item/started" => {
                if let Some(item) = params.get("item") {
                    let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let thread_id = thread_id(params, None);

                    match item_type {
                        "commandExecution" | "mcpToolCall" | "collabToolCall" => {
                            let item_id = item
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let tool_name = if item_type == "commandExecution" {
                                item.get("command")
                                    .and_then(|c| c.as_array())
                                    .and_then(|arr| arr.first())
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("command")
                                    .to_string()
                            } else {
                                item_type.to_string()
                            };
                            let tool_label = if item_type == "commandExecution" {
                                item.get("command")
                                    .and_then(|c| c.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| v.as_str())
                                            .collect::<Vec<_>>()
                                            .join(" ")
                                    })
                                    .map(|s| super::truncate(&s, 60).to_string())
                            } else {
                                None
                            };

                            let event = AgentEvent::ToolStarted {
                                session_id: thread_id.clone(),
                                cwd: item
                                    .get("cwd")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                tool_id: item_id.to_string(),
                                tool_name,
                                tool_label,
                            };
                            self.emit_event(event);
                        }
                        "fileChange" => {
                            let item_id = item
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let event = AgentEvent::ToolStarted {
                                session_id: thread_id.clone(),
                                cwd: String::new(),
                                tool_id: item_id.to_string(),
                                tool_name: "FileChange".to_string(),
                                tool_label: item
                                    .get("filePath")
                                    .and_then(|v| v.as_str())
                                    .map(super::short_path),
                            };
                            self.emit_event(event);
                        }
                        "contextCompaction" => {
                            let event = AgentEvent::Compacting {
                                session_id: thread_id,
                                cwd: String::new(),
                            };
                            self.emit_event(event);
                        }
                        _ => {}
                    }
                }
            }

            "item/completed" => {
                if let Some(item) = params.get("item") {
                    let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let thread_id = thread_id(params, None);

                    match item_type {
                        "commandExecution" | "mcpToolCall" | "collabToolCall" | "fileChange" => {
                            let item_id = item
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let event = AgentEvent::ToolCompleted {
                                session_id: thread_id,
                                cwd: item
                                    .get("cwd")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                tool_id: item_id.to_string(),
                            };
                            self.emit_event(event);
                        }
                        _ => {}
                    }
                }
            }

            _ => {
                trace!("unhandled notification: {}", method);
            }
        }
    }

    fn emit_event(&self, event: AgentEvent) {
        debug!(?event, "codex event");
        if let Ok(mut reg) = self.registry.lock() {
            reg.process_event_from(event, AgentType::Codex);
            self.dirty.store(true, Ordering::Relaxed);
        }
    }
}

/// Minimum time a connection must last before backoff is reset.
const STABLE_CONNECTION: Duration = Duration::from_secs(30);

/// Start the Codex app-server client.
///
/// Spawns `codex app-server`, initializes the handshake, discovers threads,
/// and processes events in a loop. Reconnects with exponential backoff
/// (1 s → 60 s) on failure or disconnect. Backoff only resets after a
/// connection stays up for [`STABLE_CONNECTION`].
pub async fn start(registry: Arc<Mutex<SessionRegistry>>, dirty: Arc<AtomicBool>) {
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(60);

    loop {
        match CodexClient::connect(Arc::clone(&registry), Arc::clone(&dirty)).await {
            Some((mut client, mut reader)) => {
                let connected_at = tokio::time::Instant::now();

                if let Err(e) = client.list_threads().await {
                    warn!("failed to list codex threads: {}", e);
                }

                let mut poll_interval = tokio::time::interval(THREAD_POLL_INTERVAL);
                loop {
                    tokio::select! {
                        line = reader.next_line() => {
                            match line {
                                Ok(Some(line)) if !line.is_empty() => {
                                    client.handle_message(&line).await;
                                }
                                Ok(None) | Err(_) => {
                                    info!("codex app-server disconnected, reconnecting");
                                    break;
                                }
                                _ => {}
                            }
                        }
                        _ = poll_interval.tick() => {
                            if let Err(e) = client.list_threads().await {
                                debug!("thread poll failed: {}", e);
                                break;
                            }
                        }
                    }
                }

                // Best-effort kill and reap the child process to avoid zombies
                let _ = client.child.kill().await;
                let _ = client.child.wait().await;

                // Only reset backoff if connection was stable
                if connected_at.elapsed() >= STABLE_CONNECTION {
                    backoff = Duration::from_secs(1);
                }
            }
            None => {
                debug!("codex app-server not available, retrying in {:?}", backoff);
            }
        }

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short() {
        assert_eq!(super::super::truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long() {
        assert_eq!(super::super::truncate("hello world", 5), "hello");
    }

    fn make_registry() -> (Arc<Mutex<SessionRegistry>>, Arc<AtomicBool>) {
        (
            Arc::new(Mutex::new(SessionRegistry::new())),
            Arc::new(AtomicBool::new(false)),
        )
    }

    #[test]
    fn handle_turn_started_notification() {
        let (reg, dirty) = make_registry();
        // We can't easily test handle_notification directly since it requires
        // a mutable CodexClient, but we can test the event mapping logic
        let params: Value = serde_json::json!({
            "threadId": "thr_1",
            "turn": { "id": "turn_1", "status": "inProgress" }
        });

        // Simulate what handle_notification("turn/started", ...) does
        let thread_id = params
            .get("threadId")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let event = AgentEvent::Activity {
            session_id: thread_id.to_string(),
            cwd: String::new(),
        };

        if let Ok(mut r) = reg.lock() {
            // First register the session
            r.process_event(AgentEvent::SessionStarted {
                session_id: "thr_1".into(),
                cwd: String::new(),
                agent: AgentType::Codex,
            });
            r.process_event(event);
            dirty.store(true, Ordering::Relaxed);
        }

        let sessions = reg.lock().expect("registry lock poisoned").get_all();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "thr_1");
    }

    #[test]
    fn handle_item_started_command() {
        let (reg, _dirty) = make_registry();
        let item: Value = serde_json::json!({
            "type": "commandExecution",
            "id": "item_1",
            "command": ["npm", "test"],
            "cwd": "/project",
            "status": "inProgress"
        });

        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap();
        assert_eq!(item_type, "commandExecution");

        let tool_name = item
            .get("command")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap();
        assert_eq!(tool_name, "npm");

        // Register session then process event
        if let Ok(mut r) = reg.lock() {
            r.process_event(AgentEvent::SessionStarted {
                session_id: "thr_1".into(),
                cwd: "/project".into(),
                agent: AgentType::Codex,
            });
            r.process_event(AgentEvent::ToolStarted {
                session_id: "thr_1".into(),
                cwd: "/project".into(),
                tool_id: "item_1".into(),
                tool_name: "npm".into(),
                tool_label: Some("npm test".into()),
            });
        }

        let sessions = reg.lock().expect("registry lock poisoned").get_all();
        assert_eq!(sessions[0].running_tools.len(), 1);
        assert_eq!(sessions[0].running_tools[0].tool_name, "npm");
    }
}
