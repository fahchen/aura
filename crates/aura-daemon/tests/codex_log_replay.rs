use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use aura_daemon::parsers::codex::events_from_transcript;
use aura_daemon::watcher::WatcherEvent;
use aura_common::AgentEvent;
use serde_json::Value;

fn count_events(events: Vec<WatcherEvent>) -> HashMap<&'static str, usize> {
    let mut counts = HashMap::new();
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
fn replay_codex_fixture_produces_events() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/codex-session-sample.jsonl");

    let file = File::open(&path).expect("fixture file");
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line.expect("line");
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).expect("json");
        events.extend(events_from_transcript(&value, "sess-fixture", "/tmp"));
    }

    let counts = count_events(events);
    assert_eq!(counts.get("Activity"), Some(&1));
    assert_eq!(counts.get("ToolStarted"), Some(&1));
    assert_eq!(counts.get("ToolCompleted"), Some(&1));
    assert_eq!(counts.get("NeedsAttention"), Some(&1));
    assert_eq!(counts.get("Compacting"), Some(&1));
}
