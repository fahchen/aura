---
id: BDR-0001
title: Session naming via hook parsing instead of direct IPC
status: accepted
date: 2026-02-07
summary: Session names are extracted from Bash PreToolUse hook events rather than sent directly via IPC from the CLI
---

**Feature**: integration/claude-code.feature
**Rule**: Session naming is parsed from Bash tool hook events

## Context

Aura needs to display meaningful session names in the HUD. The `aura:session-naming` Claude Code skill instructs the agent to call `aura set-name "description"` as a Bash command when starting a task.

## Behaviours Considered

### Option A: Direct IPC from `aura set-name`
The `aura set-name` CLI command connects to the daemon's Unix socket and sends a `SessionNameUpdated` event directly.

### Option B: Hook parsing of Bash commands
The `aura set-name` CLI remains a stub (exit 0). When Claude Code executes `aura set-name "..."` as a Bash tool, the `PreToolUse` hook fires and delivers the full command via stdin. Aura's hook parser detects the `aura set-name` pattern and emits a `SessionNameUpdated` event.

### Option C: Auto-derive names from user prompts
Parse the first user prompt or turn preview to automatically generate a session name without any explicit naming step.

## Decision

Chose Option B (hook parsing). The hook infrastructure already exists and carries the full Bash command in `tool_input.command`. Parsing it avoids adding socket connection logic to the CLI, keeps the CLI simple, and reuses the established event flow.

## Rejected Alternatives

- **Option A** adds complexity to a CLI command that should be lightweight. It requires the CLI to locate the socket, connect, handle errors, and format the IPC message — all for a single string. The hook path already delivers this data.
- **Option C** has no reliable way to summarize a prompt into a good short name without LLM involvement. The skill-driven approach lets the agent itself choose a descriptive name.
