# Aura

A floating HUD for real-time AI coding session awareness.

## What is Aura?

Aura monitors Claude Code as it works, displaying session state and active tools without taking up screen space. Built with Rust/gpui for native macOS performance.

**Key features:**
- Real-time tool visibility (Read, Write, Grep, Bash, etc.)
- State indicators: Running → Idle → Attention → Waiting → Compacting → Stale
- Multi-session tracking with minimal 36×36 collapsed indicator
- Glassmorphism design with liquid glass aesthetic
- Currently supports Claude Code (other agents planned for future versions)

## Screenshots

| Liquid Dark | Liquid Light |
|-------------|--------------|
| ![Liquid Dark](docs/screenshots/theme-liquid-dark.png) | ![Liquid Light](docs/screenshots/theme-liquid-light.png) |

| Solid Dark | Solid Light |
|------------|-------------|
| ![Solid Dark](docs/screenshots/theme-solid-dark.png) | ![Solid Light](docs/screenshots/theme-solid-light.png) |

## Session States

| State | Icon | Description |
|-------|------|-------------|
| Running | 📹 cctv | Session actively processing |
| Idle | 💬 message-square-code | Waiting for user input |
| Attention | 🔔 bell-ring | Needs permission or action (shakes) |
| Waiting | 🌀 fan | Waiting for user input (spins) |
| Compacting | 🍪 cookie | Compacting context |
| Stale | 👻 ghost | No activity for 10 minutes |

Sessions automatically transition to **Stale** after 10 minutes of inactivity.

## Installation

```bash
# Build
cargo build --release

# Add to PATH (in your shell config)
export PATH="/path/to/aura/target/release:$PATH"

# Start daemon
aura-daemon

# Install Claude Code plugin (in Claude Code)
/plugin marketplace add fahchen/skills
/plugin install aura@fahchen-skills
```

---

## Development

```bash
cargo test --workspace           # Run tests
cargo build -p aura-daemon       # Build daemon only
./scripts/bundle-macos.sh        # Create .app bundle

# Prototype (React reference)
cd prototype && bun dev
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

## Testing with IPC

```bash
SOCK="${XDG_RUNTIME_DIR:-/tmp}/aura.sock"

# Create session
echo '{"msg":"event","type":"activity","session_id":"test","cwd":"/tmp"}' | nc -U "$SOCK"

# Tool event
echo '{"msg":"event","type":"tool_started","session_id":"test","tool_id":"t1","tool_name":"Read","tool_label":"main.rs","cwd":"/tmp"}' | nc -U "$SOCK"
```

| Event Type | State |
|------------|-------|
| `activity` | Running |
| `idle` | Idle |
| `needs_attention` | Attention |
| `waiting_for_input` | Waiting |
| `compacting` | Compacting |

## Documentation

- [`CLAUDE.md`](./CLAUDE.md) - Developer guide
- [`docs/design-spec.md`](./docs/design-spec.md) - Visual specifications

## License

MIT
