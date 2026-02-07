# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build --release   # Build
cargo run               # Run daemon
cargo run -- -vv        # Run with debug logging
cargo test              # Run all tests
./scripts/bundle-macos.sh  # Build macOS app bundle
```

### Prototype (React reference implementation)

```bash
cd prototype
bun install && bun dev  # Dev server at localhost:5173
```

## Architecture

Aura is a floating HUD that monitors AI coding sessions via hooks and app-server integration. Single crate: `src/` with `agents/` (hook parsers, Codex JSON-RPC client), `ui/` (two-window HUD, animations, themes), and core modules (event types, session state machine, registry, IPC server).

## CLI Usage

```bash
aura                           # Start HUD daemon
aura set-name "fixing auth"   # Set session name (stub — update via hook parsing)
aura hook --agent claude-code  # Handle hook events from stdin
```

## Claude Code Integration

```bash
/plugin marketplace add fahchen/skills
/plugin install aura@fahchen-skills
```

## Project Knowledge

**MUST read before coding:** Review [docs/agents/](docs/agents/) for architecture details, design decisions, and established patterns.

- `knowledge.md`: Architecture, session design, naming flow, design references
- `patterns.md`: Standard implementations
- `improvements.md`: Past mistakes to avoid

Feature specs: `spec/features/` (9 features), decisions: `spec/decisions/` (BDRs), terminology: `spec/glossary.md`.

Use `/agent-docs:update-knowledge` to capture new learnings after a session.
