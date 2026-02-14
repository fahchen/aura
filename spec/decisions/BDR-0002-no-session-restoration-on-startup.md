---
id: BDR-0002
title: No session restoration on startup
status: accepted
date: 2026-02-07
summary: Daemon starts with empty registry; sessions are driven by external event sources, not persisted Aura state
---

**Feature**: session/session-lifecycle.feature
**Rule**: Sessions start fresh on daemon launch

## Context

When the Aura daemon restarts, it could attempt to restore previously known sessions from persisted state (files, socket reconnection, transcript scanning). The question is whether stale session data from a previous daemon run should carry over.

## Behaviours Considered

### Option A: Clean start (no restoration)
Start with an empty session registry. Sessions only appear when external sources (hook events, Codex rollouts) emit events.

### Option B: Restore from persisted state
Save session state to disk on shutdown and reload on startup, showing previously active sessions.

## Decision

Chose Option A (clean start). Restored sessions would likely be stale or inaccurate since the daemon has no way to verify whether an agent is still running. Showing phantom sessions undermines the core value of ambient awareness — what you see should reflect reality.

## Rejected Alternatives

- **Option B** introduces complexity (serialization, file management, state reconciliation) and the restored sessions would immediately need validation against live agent state — which isn't available at startup. The result would be ghost sessions that confuse more than help.
