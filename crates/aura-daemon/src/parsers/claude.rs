use aura_common::{adapters::claude_code, AgentEvent};
use serde_json::Value;

use crate::parsers::history_is_recent_millis;
use crate::watcher::WatcherEvent;

pub fn events_from_history(value: &Value) -> Vec<WatcherEvent> {
    let mut events = Vec::new();
    let Some(session_id) = value.get("sessionId").and_then(|v| v.as_str()) else {
        return events;
    };
    if !history_is_recent_millis(value, "timestamp") {
        return events;
    }
    let cwd = value
        .get("project")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let display = value.get("display").and_then(|v| v.as_str()).unwrap_or("");

    events.push(WatcherEvent::AgentEvent {
        event: AgentEvent::Activity {
            session_id: session_id.to_string(),
            cwd: cwd.clone(),
        },
    });

    if display.trim_start().starts_with("/exit") {
        events.push(WatcherEvent::AgentEvent {
            event: AgentEvent::SessionEnded {
                session_id: session_id.to_string(),
            },
        });
    }

    if let Some(name) = claude_code::parse_aura_set_name_command(display) {
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
    let session_id = value
        .get("sessionId")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_session_id);
    if session_id.is_empty() {
        return events;
    }
    let cwd = value
        .get("cwd")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_cwd);

    if value.get("type").and_then(|v| v.as_str()) == Some("progress") {
        if let Some(hook_event) = value
            .get("data")
            .and_then(|v| v.get("hookEvent"))
            .and_then(|v| v.as_str())
        {
            if hook_event == "Stop" {
                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::Idle {
                        session_id: session_id.to_string(),
                        cwd: cwd.to_string(),
                    },
                });
            } else if hook_event == "SessionStart" {
                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::Activity {
                        session_id: session_id.to_string(),
                        cwd: cwd.to_string(),
                    },
                });
            }
        }
    }

    if let Some(content) = value.get("message").and_then(|m| m.get("content")) {
        if let Some(text) = content.as_str() {
            let exit_command = text.contains("<command-name>/exit</command-name>");
            if exit_command {
                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::SessionEnded {
                        session_id: session_id.to_string(),
                    },
                });
                return events;
            }

            if text.contains("<permission_prompt>") {
                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::NeedsAttention {
                        session_id: session_id.to_string(),
                        cwd: cwd.to_string(),
                        message: None,
                    },
                });
            }
            if text.contains("<idle_prompt>") {
                events.push(WatcherEvent::AgentEvent {
                    event: AgentEvent::WaitingForInput {
                        session_id: session_id.to_string(),
                        cwd: cwd.to_string(),
                        message: None,
                    },
                });
            }
        } else if let Some(items) = content.as_array() {
            for item in items {
                match item.get("type").and_then(|v| v.as_str()) {
                    Some("tool_use") => {
                        let tool_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let tool_name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        if !tool_id.is_empty() && !tool_name.is_empty() {
                            if tool_requires_attention(tool_name) {
                                events.push(WatcherEvent::AgentEvent {
                                    event: AgentEvent::NeedsAttention {
                                        session_id: session_id.to_string(),
                                        cwd: cwd.to_string(),
                                        message: None,
                                    },
                                });
                            } else {
                                events.push(WatcherEvent::AgentEvent {
                                    event: AgentEvent::ToolStarted {
                                        session_id: session_id.to_string(),
                                        cwd: cwd.to_string(),
                                        tool_id: tool_id.to_string(),
                                        tool_name: tool_name.to_string(),
                                        tool_label: None,
                                    },
                                });
                            }
                        }

                        if tool_name == "Bash"
                            && let Some(command) = item
                                .get("input")
                                .and_then(|i| i.get("command"))
                                .and_then(|v| v.as_str())
                            && let Some(name) =
                                claude_code::parse_aura_set_name_command(command)
                        {
                            events.push(WatcherEvent::AgentEvent {
                                event: AgentEvent::SessionNameUpdated {
                                    session_id: session_id.to_string(),
                                    name,
                                },
                            });
                        }
                    }
                    Some("tool_result") => {
                        if let Some(tool_id) = item.get("tool_use_id").and_then(|v| v.as_str()) {
                            events.push(WatcherEvent::AgentEvent {
                                event: AgentEvent::ToolCompleted {
                                    session_id: session_id.to_string(),
                                    cwd: cwd.to_string(),
                                    tool_id: tool_id.to_string(),
                                },
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if matches!(
        value.get("type").and_then(|v| v.as_str()),
        Some("user") | Some("assistant")
    ) {
        events.push(WatcherEvent::AgentEvent {
            event: AgentEvent::Activity {
                session_id: session_id.to_string(),
                cwd: cwd.to_string(),
            },
        });
    }

    events
}

fn tool_requires_attention(tool_name: &str) -> bool {
    matches!(tool_name, "AskUserQuestion" | "ExitPlanMode")
}
