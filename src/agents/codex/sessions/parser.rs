use crate::{AgentEvent, AgentType};
use serde_json::Value;
use tracing::warn;

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

#[derive(Debug, Clone)]
pub(super) struct RolloutState {
    pub(super) session_id: String,
    pub(super) cwd: String,
    pub(super) session_emitted: bool,
    web_search_seq: u64,
}

impl RolloutState {
    pub(super) fn new(session_id: String, cwd: String) -> Self {
        Self {
            session_id,
            cwd,
            session_emitted: false,
            web_search_seq: 0,
        }
    }

    pub(super) fn ensure_session_event(&mut self) -> Option<AgentEvent> {
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

    pub(super) fn apply_line(&mut self, value: &Value) -> Vec<AgentEvent> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
                "call_id": "call_set",
                "name": "exec_command",
                "arguments": args_str
            }
        }));

        assert_eq!(events.len(), 2);
        match (&events[0], &events[1]) {
            (
                AgentEvent::ToolStarted {
                    tool_name,
                    tool_label,
                    ..
                },
                AgentEvent::SessionNameUpdated { name, .. },
            ) => {
                assert_eq!(tool_name, "aura");
                assert_eq!(tool_label.as_deref(), Some("aura set-name \"my session\""));
                assert_eq!(name, "my session");
            }
            other => panic!("unexpected events: {other:?}"),
        }
    }
}
