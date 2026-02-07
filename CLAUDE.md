# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build all crates
cargo build --release

# Build individual crates
cargo build -p aura            # Main daemon + HUD
cargo build -p aura-common     # Shared types

# Run daemon
cargo run -p aura

# Run with verbosity (-v info, -vv debug, -vvv trace)
cargo run -p aura -- -vv

# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p aura-common
cargo test -p aura

# Visual tests (requires feature flag)
cargo run -p aura --features visual-tests --bin aura_visual_tests

# Build macOS app bundle
./scripts/bundle-macos.sh
```

### Prototype (React reference implementation)

```bash
cd prototype
bun install
bun dev        # Dev server at localhost:5173
bun run build  # Production build
```

## Architecture

Aura is a floating HUD that monitors AI coding sessions via hooks and app-server integration. Two crates form the core:

```
aura-common          # Shared types: AgentEvent, SessionState, IPC messages
    ↓
aura                 # Hooks + app-server client + gpui HUD
```

### Session Sources

- **Claude Code**: Hooks system — `aura hook --agent claude-code` receives events via stdin, forwards to daemon over Unix socket
- **Codex**: App-server JSON-RPC client — spawns `codex app-server` subprocess, communicates via stdio

### Threading Model

- **Main thread**: gpui runs the HUD windows (must not block)
- **Background thread**: tokio runtime handles IPC socket server, Codex client, and stale detection
- **Shared state**: `Arc<Mutex<SessionRegistry>>` bridges async and UI

### Event Flow

```
Claude Code hooks → aura hook --agent claude-code → Unix socket → SessionRegistry
Codex app-server ← JSON-RPC (stdio) ← aura                    → SessionRegistry
                                                                       ↓
                                                              gpui polls each frame
                                                                       ↓
                                                         Indicator + SessionList windows
```

### Key Modules

| File | Purpose |
|------|---------|
| `aura-common/src/event.rs` | `AgentEvent` enum (Running, Idle, Attention, etc.) |
| `aura-common/src/session.rs` | `SessionState` and session metadata |
| `aura-common/src/ipc.rs` | IPC message types for hook → daemon communication |
| `aura/src/hook.rs` | Hook handler (stdin JSON → IPC socket) |
| `aura/src/server.rs` | Unix socket server for IPC |
| `aura/src/codex_client.rs` | Codex app-server JSON-RPC client |
| `aura/src/registry.rs` | Session state machine, tool tracking |
| `aura/src/ui/mod.rs` | Two-window HUD driver |
| `aura/src/ui/indicator.rs` | Collapsed 36×36 indicator window |
| `aura/src/ui/session_list.rs` | Expanded session list window |

### Session States

Running → Idle → Stale (10min timeout), or Running → Attention (permission needed), or Running → Compacting (context compact).

### Design Reference

- `prototype/` - React implementation (source of truth for visual design)
- `docs/design-spec.md` - Detailed visual specs (colors, animations, dimensions)

## CLI Usage

```bash
# Start the HUD daemon
aura

# Set a custom session name (for HUD display)
aura set-name "fixing auth bug"

# Handle hook events (called by Claude Code hooks config)
aura hook --agent claude-code

# Print Claude Code hooks config for ~/.claude/settings.json
aura hook-install
```

## Claude Code Integration

Install hooks for real-time session monitoring:

```bash
# Generate hooks config
aura hook-install
# Then add the output to your ~/.claude/settings.json under "hooks"
```

For enhanced session naming, install the Claude Code skill:
```bash
# In Claude Code
/plugin marketplace add fahchen/skills
/plugin install aura@fahchen-skills
```
