use aura_common::{adapters::codex, AgentEvent};
use serde_json::Value;

use crate::parsers::history_is_recent_seconds;
use crate::watcher::WatcherEvent;

pub fn events_from_history(value: &Value) -> Vec<WatcherEvent> {
    let mut events = Vec::new();
    let Some(session_id) = value.get("session_id").and_then(|v| v.as_str()) else {
        return events;
    };
    if !history_is_recent_seconds(value, "ts") {
        return events;
    }
    let text = value.get("text").and_then(|v| v.as_str()).unwrap_or("");

    events.push(WatcherEvent::AgentEvent {
        event: AgentEvent::Activity {
            session_id: session_id.to_string(),
            cwd: String::new(),
        },
    });

    let trimmed = text.trim_start();
    if trimmed.starts_with("/exit") || trimmed.starts_with("exit") || trimmed.starts_with("quit") {
        events.push(WatcherEvent::AgentEvent {
            event: AgentEvent::SessionEnded {
                session_id: session_id.to_string(),
            },
        });
    }

    if let Some(name) = codex::parse_aura_set_name_command(text) {
        events.push(WatcherEvent::AgentEvent {
            event: AgentEvent::SessionNameUpdated {
                session_id: session_id.to_string(),
                name,
            },
        });
    }

    events
}

pub fn events_from_transcript(
    value: &Value,
    fallback_session_id: &str,
    fallback_cwd: &str,
) -> Vec<WatcherEvent> {
    let mut events = Vec::new();
    let session_id = fallback_session_id;
    if session_id.is_empty() {
        return events;
    }

    if let Some(entry_type) = value.get("type").and_then(|v| v.as_str()) {
        match entry_type {
            "response_item" => {
                if let Some(payload) = value.get("payload")
                    && let Some(payload_type) = payload.get("type").and_then(|v| v.as_str())
                {
                    match payload_type {
                        "message" => {
                            let role = payload.get("role").and_then(|v| v.as_str());
                            if matches!(role, Some("user") | Some("assistant")) {
                                events.push(WatcherEvent::AgentEvent {
                                    event: AgentEvent::Activity {
                                        session_id: session_id.to_string(),
                                        cwd: fallback_cwd.to_string(),
                                    },
                                });
                            }

                            if let Some(content) = payload.get("content").and_then(|v| v.as_array())
                            {
                                for item in content {
                                    if item.get("type").and_then(|v| v.as_str())
                                        == Some("tool_use")
                                        && let Some(tool_id) =
                                            item.get("id").and_then(|v| v.as_str())
                                        && let Some(tool_name) =
                                            item.get("name").and_then(|v| v.as_str())
                                    {
                                        events.push(WatcherEvent::AgentEvent {
                                            event: AgentEvent::ToolStarted {
                                                session_id: session_id.to_string(),
                                                cwd: fallback_cwd.to_string(),
                                                tool_id: tool_id.to_string(),
                                                tool_name: tool_name.to_string(),
                                                tool_label: None,
                                            },
                                        });
                                    }
                                }
                            }
                        }
                        "function_call" => {
                            let tool_name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let tool_id = payload
                                .get("call_id")
                                .or_else(|| payload.get("id"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if !tool_name.is_empty() && !tool_id.is_empty() {
                                if call_requires_attention(payload) {
                                    events.push(WatcherEvent::AgentEvent {
                                        event: AgentEvent::NeedsAttention {
                                            session_id: session_id.to_string(),
                                            cwd: fallback_cwd.to_string(),
                                            message: None,
                                        },
                                    });
                                } else {
                                    events.push(WatcherEvent::AgentEvent {
                                        event: AgentEvent::ToolStarted {
                                            session_id: session_id.to_string(),
                                            cwd: fallback_cwd.to_string(),
                                            tool_id: tool_id.to_string(),
                                            tool_name: tool_name.to_string(),
                                            tool_label: None,
                                        },
                                    });
                                }
                            }

                            if let Some(args) = payload.get("arguments")
                                && let Some(command) = command_from_arguments(args)
                                && let Some(name) = codex::parse_aura_set_name_command(&command)
                            {
                                events.push(WatcherEvent::AgentEvent {
                                    event: AgentEvent::SessionNameUpdated {
                                        session_id: session_id.to_string(),
                                        name,
                                    },
                                });
                            }
                        }
                        "function_call_output" => {
                            if let Some(call_id) = payload.get("call_id").and_then(|v| v.as_str()) {
                                events.push(WatcherEvent::AgentEvent {
                                    event: AgentEvent::ToolCompleted {
                                        session_id: session_id.to_string(),
                                        cwd: fallback_cwd.to_string(),
                                        tool_id: call_id.to_string(),
                                    },
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            "message" => {
                let role = value.get("role").and_then(|v| v.as_str());
                if matches!(role, Some("user") | Some("assistant")) {
                    events.push(WatcherEvent::AgentEvent {
                        event: AgentEvent::Activity {
                            session_id: session_id.to_string(),
                            cwd: fallback_cwd.to_string(),
                        },
                    });
                }
            }
            "function_call" => {
                let tool_name = value.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let tool_id = value.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if !tool_name.is_empty() && !tool_id.is_empty() {
                    events.push(WatcherEvent::AgentEvent {
                        event: AgentEvent::ToolStarted {
                            session_id: session_id.to_string(),
                            cwd: fallback_cwd.to_string(),
                            tool_id: tool_id.to_string(),
                            tool_name: tool_name.to_string(),
                            tool_label: None,
                        },
                    });
                }
            }
            "function_call_output" => {
                if let Some(tool_id) = value.get("call_id").and_then(|v| v.as_str()) {
                    events.push(WatcherEvent::AgentEvent {
                        event: AgentEvent::ToolCompleted {
                            session_id: session_id.to_string(),
                            cwd: fallback_cwd.to_string(),
                            tool_id: tool_id.to_string(),
                        },
                    });
                }
            }
            "event_msg" => {
                if let Some(payload) = value.get("payload")
                    && let Some(payload_type) = payload.get("type").and_then(|v| v.as_str())
                    && payload_type == "context_compacted"
                {
                    events.push(WatcherEvent::AgentEvent {
                        event: AgentEvent::Compacting {
                            session_id: session_id.to_string(),
                            cwd: fallback_cwd.to_string(),
                        },
                    });
                }
            }
            "compacted" => {
                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::Compacting {
                        session_id: session_id.to_string(),
                        cwd: fallback_cwd.to_string(),
                    },
                });
            }
            _ => {}
        }
    }

    events
}

fn command_from_arguments(args: &Value) -> Option<String> {
    match args {
        Value::String(raw) => {
            let Ok(args_value) = serde_json::from_str::<Value>(raw) else {
                return None;
            };
            args_value
                .get("command")
                .or_else(|| args_value.get("cmd"))
                .and_then(|v| v.as_str())
                .map(String::from)
        }
        Value::Object(map) => map
            .get("command")
            .or_else(|| map.get("cmd"))
            .and_then(|v| v.as_str())
            .map(String::from),
        _ => None,
    }
}

fn call_requires_attention(payload: &Value) -> bool {
    let Some(args) = payload.get("arguments") else {
        return false;
    };

    let args_value = match args {
        Value::String(raw) => serde_json::from_str::<Value>(raw).ok(),
        Value::Object(map) => Some(Value::Object(map.clone())),
        _ => None,
    };

    let Some(Value::Object(map)) = args_value else {
        return false;
    };

    if map
        .get("sandbox_permissions")
        .and_then(|v| v.as_str())
        == Some("require_escalated")
    {
        return true;
    }

    if map.get("justification").is_some() {
        return true;
    }

    map.get("require_approval").and_then(|v| v.as_bool()) == Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::SystemTime;

    fn count_events(events: Vec<WatcherEvent>) -> std::collections::HashMap<&'static str, usize> {
        let mut counts = std::collections::HashMap::new();
        for event in events {
            let key = match event {
                WatcherEvent::AgentEvent { event } => match event {
                    AgentEvent::Activity { .. } => "Activity",
                    AgentEvent::ToolStarted { .. } => "ToolStarted",
                    AgentEvent::ToolCompleted { .. } => "ToolCompleted",
                    AgentEvent::NeedsAttention { .. } => "NeedsAttention",
                    AgentEvent::Compacting { .. } => "Compacting",
                    AgentEvent::SessionEnded { .. } => "SessionEnded",
                    AgentEvent::SessionNameUpdated { .. } => "SessionNameUpdated",
                    AgentEvent::WaitingForInput { .. } => "WaitingForInput",
                    AgentEvent::SessionStarted { .. } => "SessionStarted",
                },
                _ => "Other",
            };
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }

    #[test]
    fn codex_transcript_parses_basic_events() {
        let session_id = "sess-1";
        let cwd = "/tmp";

        let msg = json!({
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": [{"type":"input_text","text":"hi"}]
            }
        });

        let call = json!({
            "type": "response_item",
            "payload": {
                "type": "function_call",
                "name": "exec_command",
                "call_id": "call_1",
                "arguments": {"cmd": "ls"}
            }
        });

        let call_attention = json!({
            "type": "response_item",
            "payload": {
                "type": "function_call",
                "name": "exec_command",
                "call_id": "call_2",
                "arguments": {"sandbox_permissions": "require_escalated", "justification": "need"}
            }
        });

        let output = json!({
            "type": "response_item",
            "payload": {
                "type": "function_call_output",
                "call_id": "call_1",
                "output": "ok"
            }
        });

        let compacted = json!({
            "type": "event_msg",
            "payload": {"type": "context_compacted"}
        });

        let mut events = Vec::new();
        events.extend(events_from_transcript(&msg, session_id, cwd));
        events.extend(events_from_transcript(&call, session_id, cwd));
        events.extend(events_from_transcript(&call_attention, session_id, cwd));
        events.extend(events_from_transcript(&output, session_id, cwd));
        events.extend(events_from_transcript(&compacted, session_id, cwd));

        let counts = count_events(events);
        assert_eq!(counts.get("Activity"), Some(&1));
        assert_eq!(counts.get("ToolStarted"), Some(&1));
        assert_eq!(counts.get("ToolCompleted"), Some(&1));
        assert_eq!(counts.get("NeedsAttention"), Some(&1));
        assert_eq!(counts.get("Compacting"), Some(&1));
    }

    #[test]
    fn codex_history_parses_activity() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let value = json!({
            "session_id": "sess-h",
            "ts": now,
            "text": "hello"
        });

        let events = events_from_history(&value);
        let counts = count_events(events);
        assert_eq!(counts.get("Activity"), Some(&1));
    }
}
