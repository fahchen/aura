# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build all crates
cargo build --release

# Build individual crates
cargo build -p aura-daemon          # Main daemon + HUD
cargo build -p aura-claude-code-hook  # Hook handler binary
cargo build -p aura-common          # Shared types

# Run daemon
cargo run -p aura-daemon

# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p aura-common
cargo test -p aura-daemon
cargo test -p aura-claude-code-hook

# Visual tests (requires feature flag)
cargo run -p aura-daemon --features visual-tests --bin aura_visual_tests

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

Aura is a floating HUD that monitors AI coding sessions via hooks. Three crates form the core:

```
aura-common          # Shared types: AgentEvent, IpcMessage, SessionState
    ↓
aura-daemon          # IPC server + gpui HUD (two windows: Indicator + SessionList)
    ↑
aura-claude-code-hook  # Receives Claude Code hook JSON → sends IPC to daemon
```

### Threading Model

- **Main thread**: gpui runs the HUD windows (must not block)
- **Background thread**: tokio runtime handles IPC server + stale detection
- **Shared state**: `Arc<Mutex<SessionRegistry>>` bridges async and UI

### Event Flow

```
Claude Code hook (stdin JSON) → aura-claude-code-hook parses
    → converts to AgentEvent
    → IpcMessage over Unix socket
    → daemon server receives
    → SessionRegistry::process_event() updates state
    → gpui polls registry each frame
    → renders Indicator + SessionList windows
```

### Key Modules

| File | Purpose |
|------|---------|
| `aura-common/src/event.rs` | `AgentEvent` enum (8 variants) |
| `aura-common/src/adapters/claude_code.rs` | Hook parsing + tool label extraction |
| `aura-daemon/src/registry.rs` | Session state machine, tool tracking |
| `aura-daemon/src/server.rs` | tokio IPC server |
| `aura-daemon/src/ui/mod.rs` | Two-window HUD driver |
| `aura-daemon/src/ui/indicator.rs` | Collapsed 36×36 indicator window |
| `aura-daemon/src/ui/session_list.rs` | Expanded session list window |

### Session States

Running → Idle → Stale (10min timeout), or Running → Attention (permission needed), or Running → Compacting (context compact).

### Design Reference

- `prototype/` - React implementation (source of truth for visual design)
- `docs/design-spec.md` - Detailed visual specs (colors, animations, dimensions)

## Claude Code Integration

Install the plugin to connect Claude Code sessions to the HUD:

```bash
# Build hook handler and add to PATH
cargo build --release -p aura-claude-code-hook
export PATH="/path/to/aura/target/release:$PATH"

# Install plugin
/plugin install /path/to/aura/plugins/aura
```

The hook gracefully fails if daemon isn't running—Claude Code continues normally.

## Sending Test Events

Send IPC messages directly to test the HUD without running Claude Code.

```bash
# Find socket path
SOCK="$(fd -t s aura.sock /var/folders 2>/dev/null | head -1)"

# Ping test
echo '{"msg":"ping"}' | nc -U "$SOCK"  # Response: {"msg":"pong"}

# Create a test session
SESSION="test-$(date +%s)"
echo '{"msg":"event","type":"session_started","session_id":"'$SESSION'","cwd":"/tmp/test","agent":"claude_code"}' | nc -U "$SOCK"

# Tool events
echo '{"msg":"event","type":"tool_started","session_id":"'$SESSION'","tool_id":"t1","tool_name":"Read","tool_label":"main.rs","cwd":"/tmp/test"}' | nc -U "$SOCK"
echo '{"msg":"event","type":"tool_completed","session_id":"'$SESSION'","tool_id":"t1","cwd":"/tmp/test"}' | nc -U "$SOCK"

# State transitions
echo '{"msg":"event","type":"needs_attention","session_id":"'$SESSION'","message":"Permission required","cwd":"/tmp/test"}' | nc -U "$SOCK"  # Yellow
echo '{"msg":"event","type":"activity","session_id":"'$SESSION'","cwd":"/tmp/test"}' | nc -U "$SOCK"  # Green
echo '{"msg":"event","type":"compacting","session_id":"'$SESSION'","cwd":"/tmp/test"}' | nc -U "$SOCK"  # Purple
echo '{"msg":"event","type":"idle","session_id":"'$SESSION'","cwd":"/tmp/test"}' | nc -U "$SOCK"  # Blue

# End session
echo '{"msg":"event","type":"session_ended","session_id":"'$SESSION'","cwd":"/tmp/test"}' | nc -U "$SOCK"
```

| Event Type | HUD State | Color |
|------------|-----------|-------|
| `session_started` | Running | Green |
| `tool_started/completed` | Running + tool | Green |
| `needs_attention` | Attention | Yellow |
| `activity` | Running | Green |
| `compacting` | Compacting | Purple |
| `idle` | Idle | Blue |
| `session_ended` | Removed | — |
