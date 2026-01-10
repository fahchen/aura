//! Aura Hook Handler
//!
//! Invoked by Claude Code on hook events.
//! Reads JSON from stdin, converts to AgentEvent, sends to daemon via IPC.

use aura_common::adapters::claude_code::{parse_hook, HookEvent};
use aura_common::{socket_path, AgentEvent, IpcMessage, IpcResponse};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

fn main() {
    // Read JSON from stdin
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("aura-claude-code-hook: failed to read stdin: {e}");
        std::process::exit(1);
    }

    // Parse Claude Code hook
    let hook: HookEvent = match parse_hook(&input) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("aura-claude-code-hook: failed to parse hook: {e}");
            std::process::exit(1);
        }
    };

    // Convert to generic AgentEvent
    let event: AgentEvent = hook.into();

    // Send to daemon via IPC (fail gracefully if daemon not running)
    if let Err(e) = send_to_daemon(&event) {
        eprintln!("aura-claude-code-hook: {e}");
        // Exit 0 so Claude Code doesn't fail
    }
}

fn send_to_daemon(event: &AgentEvent) -> Result<(), String> {
    let path = socket_path();

    // Connect with timeout
    let mut stream = UnixStream::connect(&path)
        .map_err(|e| format!("daemon not running ({path:?}): {e}"))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("failed to set read timeout: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("failed to set write timeout: {e}"))?;

    // Send message as JSON line
    let message = IpcMessage::Event(event.clone());
    let json = serde_json::to_string(&message).map_err(|e| format!("failed to serialize: {e}"))?;

    stream
        .write_all(json.as_bytes())
        .map_err(|e| format!("failed to write: {e}"))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("failed to write newline: {e}"))?;
    stream.flush().map_err(|e| format!("failed to flush: {e}"))?;

    // Read response
    let mut reader = BufReader::new(&stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("failed to read response: {e}"))?;

    let response: IpcResponse =
        serde_json::from_str(&response_line).map_err(|e| format!("invalid response: {e}"))?;

    match response {
        IpcResponse::Ok => Ok(()),
        IpcResponse::Pong => Ok(()),
        IpcResponse::Error { message } => Err(format!("daemon error: {message}")),
    }
}
