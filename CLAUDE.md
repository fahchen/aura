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
├── main.rs                    # CLI parser, threading model
├── lib.rs                     # Crate root, module exports
├── event.rs                   # AgentEvent enum (10 variants)
├── session.rs                 # SessionState, SessionInfo, RunningTool
├── ipc.rs                     # Unix socket path utility
├── registry.rs                # SessionRegistry state machine + tool tracking
├── server.rs                  # Unix socket server (receives hook events)
├── agents/
│   ├── mod.rs                 # Helper functions: truncate(), short_path()
│   ├── claude_code.rs         # Hook parser + print_install_config()
│   └── codex.rs               # App-server JSON-RPC client
└── ui/
    ├── mod.rs                 # Two-window HUD driver
    ├── indicator.rs           # Collapsed 36×36 indicator window
    ├── session_list.rs        # Expanded session list window
    ├── animation.rs           # Tool cycling, breathe, shake, marquee animations
    ├── theme.rs               # Theme system + color palettes
    ├── icons.rs               # SVG paths + tool icon mapping
    ├── assets.rs              # SVG asset preloading
    └── glass.rs               # Glassmorphism rendering helper
```

### Session Sources

- **Claude Code**: Hooks system — `aura hook --agent claude-code` receives events via stdin, forwards to daemon over Unix socket
- **Gemini CLI**: Hooks system — `aura hook --agent gemini-cli` (same flow as Claude Code)
- **OpenCode**: Hooks system — `aura hook --agent open-code` (same flow as Claude Code)
- **Codex**: App-server JSON-RPC client — spawns `codex app-server` subprocess, communicates via stdio

### Threading Model

- **Main thread**: gpui runs the HUD windows (must not block)
- **Background thread**: tokio runtime handles IPC socket server, Codex client, and stale detection
- **Shared state**: `Arc<Mutex<SessionRegistry>>` bridges async and UI

### Event Flow

```
Claude Code / Gemini CLI / OpenCode hooks → aura hook --agent <type> → Unix socket → SessionRegistry
Codex app-server ← JSON-RPC (stdio) ← aura                                       → SessionRegistry
                                                                                          ↓
                                                                                 gpui polls each frame
                                                                                          ↓
                                                                            Indicator + SessionList windows
```

### Key Modules

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI parser, threading model |
| `src/lib.rs` | Crate root, module exports |
| `src/event.rs` | `AgentEvent` enum (10 variants) |
| `src/session.rs` | `SessionState`, `SessionInfo`, `RunningTool` |
| `src/ipc.rs` | Socket path utility |
| `src/server.rs` | Unix socket server |
| `src/registry.rs` | Session state machine, tool tracking |
| `src/agents/mod.rs` | Helper functions: `truncate()`, `short_path()` |
| `src/agents/claude_code.rs` | Hook handler (stdin JSON → Unix socket) |
| `src/agents/codex.rs` | Codex app-server JSON-RPC client |
| `src/ui/mod.rs` | Two-window HUD driver |
| `src/ui/indicator.rs` | Collapsed 36×36 indicator window |
| `src/ui/session_list.rs` | Expanded session list window |
| `src/ui/animation.rs` | Tool cycling, breathe, shake, marquee animations |
| `src/ui/theme.rs` | Theme system + color palettes |
| `src/ui/icons.rs` | SVG paths + tool icon mapping |
| `src/ui/assets.rs` | SVG asset preloading |
| `src/ui/glass.rs` | Glassmorphism rendering helper |

### Session States

Running → Idle → Stale (10min timeout), or Running → Attention (permission needed), or Running → Waiting (awaiting user input), or Running → Compacting (context compact).

### Design Reference

- `prototype/` - React implementation (source of truth for visual design)
- `docs/design-spec.md` - Detailed visual specs (colors, animations, dimensions)

## CLI Usage

```bash
# Start the HUD daemon
aura

# Set a custom session name (for HUD display)
aura set-name "fixing auth bug"

# Handle hook events (agents: claude-code, gemini-cli, open-code)
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

## Project Knowledge

**MUST read before coding:** Review [docs/agents/](docs/agents/) and follow the established patterns:

- `knowledge.md`: Naming conventions, design decisions
- `patterns.md`: Logging, testing, standard implementations
- `improvements.md`: Past mistakes to avoid

Use `/agent-docs:update-knowledge` to capture new learnings after a session.
