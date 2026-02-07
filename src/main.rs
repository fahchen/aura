//! Aura — HUD for AI coding agents
//!
//! Monitors AI coding sessions via hooks (Claude Code) and app-server (Codex),
//! and renders the notch-flanking HUD icons.

use aura::agents::claude_code::HookAgent;
use aura::{registry::SessionRegistry, ui};
use clap::Parser;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
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
    /// Handle agent hook events (reads JSON from stdin, forwards to daemon)
    Hook {
        /// Agent type whose hook format to parse
        #[arg(long, value_enum)]
        agent: HookAgent,
    },
    /// Print Claude Code hooks config JSON for integration setup
    HookInstall,
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

    // Handle subcommands that exit early
    match cli.command {
        Some(Command::SetName { name }) => {
            println!("Session name updated to: {name}");
            return;
        }
        Some(Command::Hook { ref agent }) => {
            aura::agents::claude_code::run(agent);
            return;
        }
        Some(Command::HookInstall) => {
            aura::agents::claude_code::print_install_config();
            return;
        }
        None => {}
    }

    init_tracing(cli.verbose);

    // Shared registry between background tasks and UI
    // Using std::sync::Mutex so it's accessible from both tokio and gpui threads
    let registry = Arc::new(Mutex::new(SessionRegistry::new()));
    let registry_dirty = Arc::new(AtomicBool::new(true));

    // Spawn tokio runtime in background thread
    let bg_registry = Arc::clone(&registry);
    let bg_dirty = Arc::clone(&registry_dirty);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async move {
            // Spawn stale detection task
            let stale_registry = Arc::clone(&bg_registry);
            let stale_dirty = Arc::clone(&bg_dirty);
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

            // Spawn Codex app-server client
            let codex_registry = Arc::clone(&bg_registry);
            let codex_dirty = Arc::clone(&bg_dirty);
            tokio::spawn(async move {
                aura::agents::codex::start(codex_registry, codex_dirty).await;
            });

            // Start IPC socket server (accepts hook events via Unix socket)
            let ipc_registry = Arc::clone(&bg_registry);
            let ipc_dirty = Arc::clone(&bg_dirty);
            aura::server::start(ipc_registry, ipc_dirty).await;
        });
    });

    // Run gpui on main thread (blocks)
    ui::run_hud(registry, registry_dirty);
}
