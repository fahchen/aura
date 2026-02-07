# Aura

A floating HUD for real-time AI coding session awareness.

## What is Aura?

Aura monitors AI coding agents as they work, displaying session state and active tools without taking up screen space. Built with Rust/gpui for native macOS performance.

**Key features:**
- Real-time tool visibility (Read, Write, Grep, Bash, etc.)
- Six session states: Running, Idle, Attention, Waiting, Compacting, Stale
- Multi-session tracking with minimal 36×36 collapsed indicator
- Five themes: System, Liquid Dark/Light, Solid Dark/Light
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

## Installation

```bash
# Build
cargo build --release

# Add to PATH (in your shell config)
export PATH="/path/to/aura/target/release:$PATH"

# Start daemon
aura
```

### Claude Code Integration

Install the Aura plugin for hooks and session naming:

```bash
# In Claude Code
/plugin marketplace add fahchen/skills
/plugin install aura@fahchen-skills
```

## Development

```bash
cargo test                       # Run tests
cargo build --release            # Build release binary
./scripts/bundle-macos.sh        # Create .app bundle

# Prototype (React reference)
cd prototype && bun dev
```

## Documentation

- [`CLAUDE.md`](./CLAUDE.md) - Developer guide
- [`spec/`](./spec/) - BDD feature specs and behaviour decisions
- [`prototype/`](./prototype/) - React visual design reference

## License

MIT
