use aura_common::AgentType;
use serde_json::Value;
use std::time::{Duration, SystemTime};

use crate::watcher::WatcherEvent;

pub mod claude;
pub mod codex;

/// How recently a log event must occur to be considered "active" (10 minutes).
pub const ACTIVE_THRESHOLD_SECS: u64 = 600;

pub fn events_from_transcript_line(
    agent: &AgentType,
    value: &Value,
    fallback_session_id: &str,
    fallback_cwd: &str,
) -> Vec<WatcherEvent> {
    match agent {
        AgentType::ClaudeCode => {
            claude::events_from_transcript(value, fallback_session_id, fallback_cwd)
        }
        AgentType::Codex => codex::events_from_transcript(value, fallback_session_id, fallback_cwd),
        _ => Vec::new(),
    }
}

pub(crate) fn history_is_recent_seconds(value: &Value, field: &str) -> bool {
    let Some(ts) = value
        .get(field)
        .and_then(|v| v.as_u64())
        .or_else(|| value.get(field).and_then(|v| v.as_i64()).and_then(|v| v.try_into().ok()))
    else {
        return false;
    };

    let event_time = SystemTime::UNIX_EPOCH + Duration::from_secs(ts);
    SystemTime::now()
        .duration_since(event_time)
        .map(|elapsed| elapsed.as_secs() < ACTIVE_THRESHOLD_SECS)
        .unwrap_or(false)
}

pub(crate) fn history_is_recent_millis(value: &Value, field: &str) -> bool {
    let Some(ts) = value
        .get(field)
        .and_then(|v| v.as_u64())
        .or_else(|| value.get(field).and_then(|v| v.as_i64()).and_then(|v| v.try_into().ok()))
    else {
        return false;
    };

    let event_time = SystemTime::UNIX_EPOCH + Duration::from_millis(ts);
    SystemTime::now()
        .duration_since(event_time)
        .map(|elapsed| elapsed.as_secs() < ACTIVE_THRESHOLD_SECS)
        .unwrap_or(false)
}
