# Project Knowledge

Implementation patterns and design decisions.

## Architecture

### Threading Model

**Rule:** Main thread runs gpui (HUD windows, must not block). Background thread runs tokio runtime (IPC server, agent integrations). Shared state via `Arc<Mutex<SessionRegistry>>`.

**Why:** gpui requires main thread ownership. Async I/O must not block the UI.

**Reference:** `src/main.rs` - thread spawning and runtime setup

### Session Sources

**Rule:** Two integration patterns:
- **Hooks** (Claude Code): `aura hook --agent claude-code` receives JSON via stdin, forwards to daemon over Unix socket
- **Rollouts** (Codex): Daemon watches `~/.codex/sessions/**.jsonl` (or `$CODEX_HOME/sessions`) and tails appended JSONL events

**Reference:** `src/agents/claude_code.rs`, `src/agents/codex/mod.rs`, `src/agents/codex/sessions.rs`

### Event Flow

**Rule:** Hook events → Unix socket → `SessionRegistry`. Codex rollouts → filesystem watcher + JSONL tailer → `SessionRegistry`. gpui polls registry each frame → renders Indicator + SessionList windows.

**Reference:** `src/server.rs` (IPC), `src/registry.rs` (state machine)

## Session Design

### Session States

**Rule:** Six states: Running, Idle, Attention, Waiting, Compacting, Stale.
- **Running** → Idle (Stop), Attention (permission needed), Waiting (idle_prompt), Compacting (PreCompact)
- **Idle / Attention / Waiting / Compacting** → Running (on new activity)
- **Idle** → Stale (10min per-session timer, resets on events)
- Daemon starts with empty registry — Aura does not restore its own session state from disk
- Stale sessions are never auto-removed (user removes manually)

**Why:** Each state maps to a distinct user action (or non-action). See `spec/decisions/BDR-0002` and `BDR-0003`.

**Reference:** `src/registry.rs`, `src/session.rs`

### Session Naming

**Rule:** Names are set via the `aura:session-naming` Claude Code skill, which calls `aura set-name "name"` as a Bash command. The PreToolUse hook intercepts this command and extracts the name. The `aura set-name` CLI is a stub (exit 0) — the actual name update flows through the hook parser.

**Why:** Avoids adding socket connection logic to the CLI. Reuses the existing hook event flow. See `spec/decisions/BDR-0001`.

**Reference:** `src/agents/claude_code.rs` - hook parser, `src/main.rs` - CLI stub

## Persistence

### Config vs State Files

**Rule:** User preferences (theme) go in config file. Runtime state (indicator position) goes in a separate state file. Both under `dirs::data_dir()` / `dirs::config_dir()` (same path on macOS: `~/Library/Application Support/aura/`).

- `config.json` — theme preference
- `state.json` — indicator position

**Why:** Conceptual separation: config is what the user chose, state is derived from usage. A user might want to reset state without losing preferences, or share config across machines.

**Reference:** `dirs` crate for paths, `serde_json` for serialization

## Design Reference

### Spec as Source of Truth

**Rule:** BDD feature specs in `spec/` are the source of truth for behaviour. Prototype in `prototype/` is the visual design reference.

- `spec/features/` - 9 feature files organized by domain
- `spec/decisions/` - Behaviour Decision Records (BDRs)
- `spec/glossary.md` - Domain terminology
