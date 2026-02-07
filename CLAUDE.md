# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build
cargo build --release

# Run daemon
cargo run

# Run with verbosity (-v info, -vv debug, -vvv trace)
cargo run -- -vv

# Run all tests
cargo test

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

Aura is a floating HUD that monitors AI coding sessions via hooks and app-server integration. Single crate with modular structure:

```
src/
├── event.rs, session.rs    # Core domain types (AgentEvent, SessionState)
├── ipc.rs                  # Unix socket path utility
├── registry.rs             # Session state machine
├── server.rs               # Unix socket server (receives hook events)
├── agents/                 # Agent implementations
│   ├── claude_code.rs      # Hook parser + install config
│   └── codex.rs            # App-server JSON-RPC client
└── ui/                     # gpui HUD (indicator + session list)
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
| `src/event.rs` | `AgentEvent` enum (Running, Idle, Attention, etc.) |
| `src/session.rs` | `SessionState` and session metadata |
| `src/ipc.rs` | Socket path utility |
| `src/agents/claude_code.rs` | Hook handler (stdin JSON → Unix socket) |
| `src/agents/codex.rs` | Codex app-server JSON-RPC client |
| `src/server.rs` | Unix socket server |
| `src/registry.rs` | Session state machine, tool tracking |
| `src/ui/mod.rs` | Two-window HUD driver |
| `src/ui/indicator.rs` | Collapsed 36×36 indicator window |
| `src/ui/session_list.rs` | Expanded session list window |

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
