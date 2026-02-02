//! Aura Daemon - HUD UI + session tracking
//!
//! Monitors AI coding sessions by watching transcript files directly,
//! and renders the notch-flanking HUD icons.
//!
//! Session sources:
//! - Transcript watcher: Discovers sessions from `~/.claude/projects/` and `~/.codex/sessions/`

use aura_daemon::{registry::SessionRegistry, ui, watcher::TranscriptWatcher};
use clap::Parser;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tracing::{debug, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

/// Stale detection interval
const STALE_CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// Stale timeout - mark session stale after 10min of no activity
const STALE_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Parser)]
#[command(name = "aura", about = "Aura HUD daemon")]
struct Cli {
    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Set the session name displayed in the HUD
    SetName {
        /// The name to display for the current session
        name: String,
    },
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

/// Watcher poll interval
const WATCHER_POLL_INTERVAL: Duration = Duration::from_millis(50);

fn main() {
    let cli = Cli::parse();

    // Handle set-name subcommand (just prints success message)
    if let Some(Command::SetName { name }) = cli.command {
        println!("Session name updated to: {name}");
        return;
    }

    init_tracing(cli.verbose);

    // Shared registry between watcher and UI
    // Using std::sync::Mutex so it's accessible from both tokio and gpui threads
    let registry = Arc::new(Mutex::new(SessionRegistry::new()));
    let registry_dirty = Arc::new(AtomicBool::new(true));

    // Spawn tokio runtime in background thread for watcher
    let server_registry = Arc::clone(&registry);
    let server_dirty = Arc::clone(&registry_dirty);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async move {
            // Spawn stale detection task
            let stale_registry = Arc::clone(&server_registry);
            let stale_dirty = Arc::clone(&server_dirty);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(STALE_CHECK_INTERVAL);
                loop {
                    interval.tick().await;
                    if let Ok(mut reg) = stale_registry.lock() {
                        reg.mark_stale(STALE_TIMEOUT);
                        stale_dirty.store(true, Ordering::Relaxed);

                    }
                }
            });

            // Spawn watcher task
            let watcher_registry = Arc::clone(&server_registry);
            let watcher_dirty = Arc::clone(&server_dirty);
            tokio::spawn(async move {
                // Create watcher
                let mut watcher = match TranscriptWatcher::new() {
                    Ok(w) => w,
                    Err(e) => {
                        warn!("Failed to create transcript watcher: {}", e);
                        return;
                    }
                };

                if !watcher.is_watching() {
                    info!("No transcript directories to watch");
                    return;
                }

                info!(
                    "Transcript watcher started, tracking {} directories",
                    watcher.watched_dirs().len()
                );

                let mut interval = tokio::time::interval(WATCHER_POLL_INTERVAL);
                loop {
                    interval.tick().await;

                    for event in watcher.poll() {
                        match event {
                            aura_daemon::watcher::WatcherEvent::SessionUpdate {
                                session_id,
                                cwd,
                                agent,
                                meta,
                                is_active,
                                last_event_time,
                            } => {
                                if let Ok(mut reg) = watcher_registry.lock() {
                                    reg.update_from_watcher(
                                        &session_id,
                                        &cwd,
                                        agent,
                                        *meta,
                                        None,
                                        is_active,
                                        last_event_time,
                                    );
                                    watcher_dirty.store(true, Ordering::Relaxed);
                                }
                            }
                            aura_daemon::watcher::WatcherEvent::SessionRemoved { session_id } => {
                                // Just log - let stale detection handle cleanup
                                debug!("Session file removed: {}", session_id);
                            }
                            aura_daemon::watcher::WatcherEvent::AgentEvent { event } => {
                                debug!(?event, "agent event");
                                if let Ok(mut reg) = watcher_registry.lock() {
                                    reg.process_event(event);
                                    watcher_dirty.store(true, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                }
            });

            // Keep the runtime alive now that IPC is removed
            std::future::pending::<()>().await;

        });
    });

    // Run gpui on main thread (blocks)
    ui::run_hud(registry, registry_dirty);
}
