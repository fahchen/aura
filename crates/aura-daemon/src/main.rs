//! Aura Daemon - IPC server + HUD UI
//!
//! Receives events from hook handlers, tracks session state,
//! and renders the notch-flanking HUD icons.

mod registry;
mod server;
mod ui;

use registry::SessionRegistry;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Stale detection interval
const STALE_CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// Stale timeout - mark session stale after 60s of no activity
const STALE_TIMEOUT: Duration = Duration::from_secs(60);

fn main() {
    // Shared registry between IPC server and UI
    // Using std::sync::Mutex so it's accessible from both tokio and gpui threads
    let registry = Arc::new(Mutex::new(SessionRegistry::new()));

    // Spawn tokio runtime in background thread for IPC server
    let server_registry = Arc::clone(&registry);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async move {
            // Spawn stale detection task
            let stale_registry = Arc::clone(&server_registry);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(STALE_CHECK_INTERVAL);
                loop {
                    interval.tick().await;
                    if let Ok(mut reg) = stale_registry.lock() {
                        reg.mark_stale(STALE_TIMEOUT);

                        // Debug: print session count
                        let count = reg.len();
                        if count > 0 {
                            eprintln!("aura: {} active session(s)", count);
                        }
                    }
                }
            });

            // Run IPC server
            if let Err(e) = server::run(server_registry).await {
                eprintln!("aura: server error: {e}");
            }
        });
    });

    // Run gpui on main thread (blocks)
    ui::run_hud(registry);
}
