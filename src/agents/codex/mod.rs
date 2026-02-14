//! Codex agent integrations.
//!
//! Aura consumes Codex **session rollout JSONL files** under `~/.codex/sessions/**.jsonl`
//! (or `$CODEX_HOME/sessions`).

use crate::AgentEvent;
use tokio::sync::broadcast;

pub mod sessions;

const EVENT_BUFFER: usize = 4096;

#[derive(Debug, Clone)]
pub struct CodexEventStream {
    tx: broadcast::Sender<AgentEvent>,
}

#[derive(Debug)]
pub struct CodexEventRx {
    rx: broadcast::Receiver<AgentEvent>,
}

impl CodexEventStream {
    /// Subscribe to Codex agent events.
    ///
    /// This receiver intentionally swallows `broadcast::RecvError::Lagged` since
    /// the Codex integration is best-effort and Aura does not attempt to recover
    /// missed events.
    pub fn subscribe(&self) -> CodexEventRx {
        CodexEventRx { rx: self.tx.subscribe() }
    }
}

impl CodexEventRx {
    pub async fn recv(&mut self) -> Option<AgentEvent> {
        loop {
            match self.rx.recv().await {
                Ok(ev) => return Some(ev),
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}

/// Spawn the Codex integration and return an event stream handle.
pub fn spawn() -> CodexEventStream {
    let (tx, _rx) = broadcast::channel(EVENT_BUFFER);
    sessions::spawn(tx.clone());
    CodexEventStream { tx }
}
