//! Unix socket server for receiving agent events from aura-hook
//!
//! Listens on `/tmp/aura.sock` for newline-delimited JSON messages.
//! Each message is deserialized directly as an `AgentEvent`.

use crate::AgentEvent;
use crate::ipc;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;
use tracing::{debug, info, trace, warn};

use crate::registry::SessionRegistry;

/// Start the Unix socket server.
///
/// Removes any stale socket file, binds to the path, and spawns a task
/// that accepts connections and processes messages.
pub async fn start(registry: Arc<Mutex<SessionRegistry>>, dirty: Arc<AtomicBool>) {
    let path = ipc::socket_path();

    // Remove stale socket if it exists
    if path.exists()
        && let Err(e) = std::fs::remove_file(&path)
    {
        warn!("Failed to remove stale socket {}: {}", path.display(), e);
        return;
    }

    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            warn!("Failed to bind Unix socket {}: {}", path.display(), e);
            return;
        }
    };

    info!("IPC server listening on {}", path.display());

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let reg = Arc::clone(&registry);
                let dirty = Arc::clone(&dirty);
                tokio::spawn(async move {
                    let reader = BufReader::new(stream);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        if line.is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<AgentEvent>(&line) {
                            Ok(event) => {
                                debug!(?event, "ipc event");
                                if let Ok(mut reg) = reg.lock() {
                                    reg.process_event(event);
                                    dirty.store(true, Ordering::Relaxed);
                                }
                            }
                            Err(e) => {
                                trace!("Failed to parse IPC message: {} (line: {})", e, line);
                            }
                        }
                    }
                });
            }
            Err(e) => {
                warn!("Failed to accept socket connection: {}", e);
            }
        }
    }
}
