//! Aura Hook Handler
//!
//! Invoked by Claude Code on hook events.
//! Reads JSON from stdin, converts to AgentEvent, sends to daemon via IPC.

use aura_common::adapters::claude_code::{parse_hook, HookEvent};
use aura_common::AgentEvent;
use std::io::{self, Read};

fn main() {
    // Read JSON from stdin
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("Failed to read stdin: {}", e);
        std::process::exit(1);
    }

    // Parse Claude Code hook
    let hook: HookEvent = match parse_hook(&input) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to parse hook: {}", e);
            std::process::exit(1);
        }
    };

    // Convert to generic AgentEvent
    let event: AgentEvent = hook.into();

    // TODO: Send to daemon via IPC
    eprintln!("Received: session={}", event.session_id());
}
