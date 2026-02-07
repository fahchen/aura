# Aura

A floating HUD for real-time AI coding session awareness.

## What is Aura?

Aura monitors AI coding agents as they work, displaying session state and active tools without taking up screen space. Built with Rust/gpui for native macOS performance.

**Key features:**
- Real-time tool visibility (Read, Write, Grep, Bash, etc.)
- State indicators: Running → Idle → Attention → Waiting → Compacting → Stale
- Multi-session tracking with minimal 36×36 collapsed indicator
- Glassmorphism design with liquid glass aesthetic
- Supports Claude Code (hooks) and Codex (app-server)

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
aura

# Install Claude Code hooks
aura hook-install
# Add the output to ~/.claude/settings.json under "hooks"

# Install Claude Code plugin (in Claude Code)
/plugin marketplace add fahchen/skills
/plugin install aura@fahchen-skills
```

---

## Development

```bash
cargo test --workspace           # Run tests
cargo build -p aura              # Build daemon only
./scripts/bundle-macos.sh        # Create .app bundle

# Prototype (React reference)
cd prototype && bun dev
```

## Architecture

```
aura-common            # Shared types: AgentEvent, SessionState, IPC
    ↓
aura                   # Hooks + Codex client + gpui HUD
```

**Event flow:**
- Claude Code hooks → Unix socket → SessionRegistry → gpui render
- Codex app-server → JSON-RPC (stdio) → SessionRegistry → gpui render

## Documentation

- [`CLAUDE.md`](./CLAUDE.md) - Developer guide
- [`docs/design-spec.md`](./docs/design-spec.md) - Visual specifications

## License

MIT
