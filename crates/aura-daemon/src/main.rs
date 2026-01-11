//! Aura Daemon - IPC server + HUD UI
//!
//! Receives events from hook handlers, tracks session state,
//! and renders the notch-flanking HUD icons.

mod registry;
mod server;
mod ui;

use clap::Parser;
use registry::SessionRegistry;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, error};
use tracing_subscriber::{fmt, EnvFilter};

/// Stale detection interval
const STALE_CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// Stale timeout - mark session stale after 60s of no activity
const STALE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Parser)]
#[command(name = "aura", about = "Aura HUD daemon")]
struct Cli {
    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn init_tracing(verbose: u8) {
    let default_level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_from_env("AURA_LOG")
        .unwrap_or_else(|_| EnvFilter::new(default_level));
    fmt().with_env_filter(filter).with_target(false).init();
}

fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

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

                        // Debug: log session count
                        let count = reg.len();
                        if count > 0 {
                            debug!("{count} active session(s)");
                        }
                    }
                }
            });

            // Run IPC server
            if let Err(e) = server::run(server_registry).await {
                error!("server error: {e}");
            }
        });
    });

    // Run gpui on main thread (blocks)
    ui::run_hud(registry);
}
