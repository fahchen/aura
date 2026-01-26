//! Aura Daemon - IPC server + HUD UI
//!
//! Receives events from hook handlers, tracks session state,
//! and renders the notch-flanking HUD icons.

use aura_daemon::{registry::SessionRegistry, server, ui};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, error, info};
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

/// Install CLI tools to /usr/local/bin when running from an app bundle.
/// This mimics VS Code's behavior of installing the `code` command.
fn install_cli_tools() {
    let Ok(exe_path) = std::env::current_exe() else {
        debug!("Could not determine current exe path");
        return;
    };

    let exe_path_str = exe_path.to_string_lossy();

    // Check if running from an app bundle
    if !exe_path_str.contains(".app/Contents/MacOS") {
        debug!("Not running from app bundle, skipping CLI install");
        return;
    }

    // Get the directory containing the daemon binary
    let Some(macos_dir) = exe_path.parent() else {
        debug!("Could not determine MacOS directory");
        return;
    };

    // Check if hook binary exists next to daemon
    let hook_source = macos_dir.join("aura-claude-code-hook");
    if !hook_source.exists() {
        debug!("Hook binary not found at {:?}", hook_source);
        return;
    }

    let target_path = PathBuf::from("/usr/local/bin/aura-claude-code-hook");

    // Check if symlink already exists and points to correct location
    if target_path.is_symlink() {
        if let Ok(link_target) = std::fs::read_link(&target_path) {
            if link_target == hook_source {
                debug!("CLI tool already installed correctly");
                return;
            }
            info!(
                "CLI symlink exists but points to {:?}, updating",
                link_target
            );
        }
    } else if target_path.exists() {
        info!("CLI path exists but is not a symlink, skipping");
        return;
    }

    // Create symlink with admin privileges using osascript
    info!("Installing CLI tool to /usr/local/bin");

    let hook_source_str = hook_source.to_string_lossy();
    let script = format!(
        "do shell script \"mkdir -p /usr/local/bin && ln -sf '{}' '/usr/local/bin/aura-claude-code-hook'\" with administrator privileges",
        hook_source_str
    );

    match ProcessCommand::new("osascript")
        .args(["-e", &script])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                info!("CLI tool installed successfully");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("User canceled") || stderr.contains("(-128)") {
                    info!("User canceled CLI installation");
                } else {
                    debug!("CLI installation failed: {}", stderr);
                }
            }
        }
        Err(e) => {
            debug!("Failed to run osascript: {}", e);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // Handle set-name subcommand (just prints success message)
    if let Some(Command::SetName { name }) = cli.command {
        println!("Session name updated to: {name}");
        return;
    }

    init_tracing(cli.verbose);

    // Install CLI tools if running from app bundle
    install_cli_tools();

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
