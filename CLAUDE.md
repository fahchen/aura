# Aura

Real-time situational awareness HUD for AI code agents and CLIs.

## Project Overview

Aura provides a floating HUD window (gpui.rs) that displays status of multiple AI coding sessions. It supports various AI code agents through different integration methods:

| Phase | Agent | Integration |
|-------|-------|-------------|
| **Phase 1** | Claude Code | Hooks API |
| **Phase 2** | Any CLI | PTY wrapper (`aura run <cmd>`) |

The HUD shows:
- Session state (Running, Idle, Attention, Compacting, Stale)
- Active tools with marquee display
- Multiple concurrent sessions from different agents

## Architecture

### Phase 1: Claude Code Hooks
```
Claude Code ──(10 hooks)──▶ hooks/aura-hook ──(IPC)──▶ Aura Daemon (gpui HUD)
```

### Phase 2: PTY Wrapper (Future)
```
aura run <cmd> ──(PTY)──▶ Child Process
       │
       └──(IPC)──▶ Aura Daemon (gpui HUD)
```

### Components

| Component | Location | Purpose |
|-----------|----------|---------|
| Hook Handler | `hooks/aura-hook` | Receives hook events, sends to daemon |
| Daemon | `src/` | IPC server + gpui HUD |

## Tech Stack

- **Language:** Rust
- **UI:** gpui.rs (GPU-accelerated)
- **IPC:** interprocess crate (Unix socket / Windows named pipe)
- **Async:** tokio

## States

| State | Icon | Color | Trigger |
|-------|------|-------|---------|
| Running | `▶` | Green #22C55E | SessionStart, tool hooks |
| Idle | `◼` | Blue #3B82F6 | Stop |
| Attention | `🔔` | Yellow #EAB308 | PermissionRequest |
| Compacting | `⟳` | Purple #A855F7 | PreCompact |
| Stale | `⏸` | Gray #6B7280 | 60s timeout |

## Hook Events (All 10)

| Hook | Action |
|------|--------|
| SessionStart | Register session → Running |
| UserPromptSubmit | Health check → Running |
| PreToolUse | Add tool to running_tools |
| PostToolUse | Remove tool from running_tools |
| PermissionRequest | → Attention |
| Notification | → Running (or Attention if permission) |
| Stop | → Idle, clear tools |
| SubagentStop | Health check → Running |
| PreCompact | → Compacting |
| SessionEnd | Remove session |

## Tool Icons

| Tool | Icon | Tool | Icon |
|------|------|------|------|
| Task | 🤖 | Bash | >_ |
| Glob | 📂 | Grep | 🔍 |
| Read | 📖 | Edit | ✏️ |
| Write | 📝 | WebFetch | 🌐 |
| WebSearch | 🔎 | mcp__* | 🔌 |
| (other) | ⚙️ | | |

## Commands

```bash
# Build
cargo build --release

# Run daemon
cargo run

# Run tests
cargo test
```

## Target Agents

| Agent | Status | Integration |
|-------|--------|-------------|
| Claude Code | Phase 1 | Hooks API |
| Gemini CLI | Phase 2 | PTY wrapper |
| OpenCode | Phase 2 | PTY wrapper |
| Codex CLI | Phase 2 | PTY wrapper |
| Any CLI | Phase 2 | PTY wrapper |

## References

- [Claude Code Hooks](https://code.claude.com/docs/en/hooks)
- [gpui.rs](https://github.com/zed-industries/zed/tree/main/crates/gpui)
- [interprocess](https://docs.rs/interprocess)
