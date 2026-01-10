//! Aura Daemon - IPC server + session registry
//!
//! Receives events from hook handlers, tracks session state,
//! and (in Phase 5) renders the HUD.

mod registry;
mod server;

use registry::SessionRegistry;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Stale detection interval
const STALE_CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// Stale timeout - mark session stale after 60s of no activity
const STALE_TIMEOUT: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() {
    let registry = Arc::new(Mutex::new(SessionRegistry::new()));

    // Spawn stale detection task
    let stale_registry = Arc::clone(&registry);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(STALE_CHECK_INTERVAL);
        loop {
            interval.tick().await;
            let mut reg = stale_registry.lock().await;
            reg.mark_stale(STALE_TIMEOUT);

            // Debug: print session count
            let count = reg.len();
            if count > 0 {
                eprintln!("aura: {} active session(s)", count);
            }
        }
    });

    // Run IPC server (blocks)
    if let Err(e) = server::run(registry).await {
        eprintln!("aura: server error: {e}");
        std::process::exit(1);
    }
}
