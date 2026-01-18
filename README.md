# Aura

A floating HUD for real-time AI coding session awareness.

## What is Aura?

Aura monitors Claude Code and other AI agents as they work, displaying session state and active tools without taking up screen space. Built with Rust/gpui for native macOS performance.

**Key features:**
- Real-time tool visibility (Read, Write, Grep, Bash, etc.)
- State indicators: Running → Idle → Attention → Waiting → Compacting → Stale
- Multi-session tracking with minimal 36×36 collapsed indicator
- Glassmorphism design with liquid glass aesthetic
- Agent-agnostic architecture (Claude Code, Codex, custom agents)

## Quick Start

```bash
# Build
cargo build --release

# Start daemon
cargo run -p aura-daemon

# Install Claude Code plugin
/plugin install /path/to/aura/plugins/aura
```

## Architecture

```
aura-common            # Shared types: AgentEvent, SessionState
    ↓
aura-daemon            # IPC server + gpui HUD
    ↑
aura-claude-code-hook  # Hook handler for Claude Code
```

**Event flow:** Claude Code hook → IPC message → daemon → SessionRegistry → gpui render

## Usage

### Running the Daemon

```bash
cargo run -p aura-daemon        # Default
cargo run -p aura-daemon -- -v  # Verbose logging
```

### Testing with IPC

```bash
SOCK="${XDG_RUNTIME_DIR:-/tmp}/aura.sock"

# Create session
echo '{"msg":"event","type":"activity","session_id":"test","cwd":"/tmp"}' | nc -U "$SOCK"

# Tool event
echo '{"msg":"event","type":"tool_started","session_id":"test","tool_id":"t1","tool_name":"Read","tool_label":"main.rs","cwd":"/tmp"}' | nc -U "$SOCK"
```

| Event Type | State | Color |
|------------|-------|-------|
| `activity` | Running | Green |
| `idle` | Idle | Blue |
| `needs_attention` | Attention | Yellow |
| `waiting_for_input` | Waiting | Yellow |
| `compacting` | Compacting | Purple |

## Development

```bash
cargo test --workspace           # Run tests
cargo build -p aura-daemon       # Build daemon only
./scripts/bundle-macos.sh        # Create .app bundle

# Prototype (React reference)
cd prototype && bun dev
```

## Documentation

- [`CLAUDE.md`](./CLAUDE.md) - Developer guide
- [`docs/design-spec.md`](./docs/design-spec.md) - Visual specifications
- [`plugins/aura/README.md`](./plugins/aura/README.md) - Plugin setup

## License

MIT
