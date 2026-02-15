//! Aura — HUD for AI coding agents
//!
//! Monitors AI coding sessions via hooks (Claude Code) and Codex session rollouts,
//! and renders the notch-flanking HUD icons.

use aura::agents::claude_code::HookAgent;
use aura::{registry::SessionRegistry, ui};
use clap::Parser;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command as ProcessCommand;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
#[cfg(target_os = "macos")]
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt};

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
}

fn init_tracing(verbose: u8) {
    let default_level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let filter =
        EnvFilter::try_from_env("AURA_LOG").unwrap_or_else(|_| EnvFilter::new(default_level));
    fmt().with_env_filter(filter).with_target(false).init();
}

#[cfg(target_os = "macos")]
fn escape_single_quotes_for_shell(s: &str) -> String {
    // POSIX shell: to embed a single quote inside a single-quoted string, close it, escape, reopen.
    // Example: abc'def -> 'abc'\''def'
    s.replace('\'', "'\\''")
}

/// Install the `aura` CLI to `/usr/local/bin` when running from an app bundle.
///
/// This mimics VS Code's behavior of installing a shell command for the GUI app.
#[cfg(target_os = "macos")]
fn install_cli_tool() {
    let Ok(exe_path) = std::env::current_exe() else {
        debug!("Could not determine current exe path");
        return;
    };

    let exe_path_str = exe_path.to_string_lossy();
    if !exe_path_str.contains(".app/Contents/MacOS") {
        debug!("Not running from app bundle, skipping CLI install");
        return;
    }

    let target_path = PathBuf::from("/usr/local/bin/aura");

    // Check if symlink already exists and points to correct location.
    if target_path.is_symlink() {
        if let Ok(link_target) = std::fs::read_link(&target_path) {
            if link_target == exe_path {
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

    info!("Installing CLI tool to /usr/local/bin");

    let exe_path_escaped = escape_single_quotes_for_shell(&exe_path_str);
    let script = format!(
        "do shell script \"mkdir -p /usr/local/bin && ln -sf '{}' '/usr/local/bin/aura'\" with administrator privileges",
        exe_path_escaped
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

#[cfg(not(target_os = "macos"))]
fn install_cli_tool() {}

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
        None => {}
    }

    init_tracing(cli.verbose);

    // Install CLI tool if running from app bundle (macOS only).
    install_cli_tool();

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
            // Spawn stale detection task — sleeps until the next session is due
            // to go stale instead of polling at a fixed interval.
            let stale_registry = Arc::clone(&bg_registry);
            let stale_dirty = Arc::clone(&bg_dirty);
            tokio::spawn(async move {
                loop {
                    let sleep_duration = {
                        if let Ok(reg) = stale_registry.lock() {
                            reg.next_stale_at(STALE_TIMEOUT)
                                .map(|t| {
                                    t.saturating_duration_since(Instant::now())
                                        + Duration::from_millis(100)
                                })
                                .unwrap_or(Duration::from_secs(30))
                        } else {
                            Duration::from_secs(5)
                        }
                    };

                    tokio::time::sleep(sleep_duration).await;

                    if let Ok(mut reg) = stale_registry.lock() {
                        reg.mark_stale(STALE_TIMEOUT);
                        stale_dirty.store(true, Ordering::Relaxed);
                    }
                }
            });

            // Spawn Codex session rollout watcher (event stream producer)
            let codex_stream = aura::agents::codex::spawn();
            let codex_registry = Arc::clone(&bg_registry);
            let codex_dirty = Arc::clone(&bg_dirty);
            tokio::spawn(async move {
                let mut rx = codex_stream.subscribe();
                while let Some(event) = rx.recv().await {
                    if let Ok(mut reg) = codex_registry.lock() {
                        reg.process_event_from(event, aura::AgentType::Codex);
                        codex_dirty.store(true, Ordering::Relaxed);
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_no_subcommand() {
        let cli = Cli::try_parse_from(["aura"]).unwrap();
        assert!(cli.command.is_none());
        assert_eq!(cli.verbose, 0);
    }

    #[test]
    fn cli_verbose_one() {
        let cli = Cli::try_parse_from(["aura", "-v"]).unwrap();
        assert_eq!(cli.verbose, 1);
    }

    #[test]
    fn cli_verbose_three() {
        let cli = Cli::try_parse_from(["aura", "-vvv"]).unwrap();
        assert_eq!(cli.verbose, 3);
    }

    #[test]
    fn cli_set_name() {
        let cli = Cli::try_parse_from(["aura", "set-name", "fix bug"]).unwrap();
        match cli.command {
            Some(Command::SetName { name }) => assert_eq!(name, "fix bug"),
            _ => panic!("expected SetName command"),
        }
    }

    #[test]
    fn cli_hook_claude_code() {
        let cli = Cli::try_parse_from(["aura", "hook", "--agent", "claude-code"]).unwrap();
        match cli.command {
            Some(Command::Hook { agent }) => assert_eq!(agent, HookAgent::ClaudeCode),
            _ => panic!("expected Hook command"),
        }
    }
}
