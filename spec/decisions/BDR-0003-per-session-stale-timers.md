---
id: BDR-0003
title: Per-session stale timers instead of global polling
status: accepted
date: 2026-02-07
summary: Each session has its own stale timeout timer that resets on events, replacing the global 10-second polling interval
---

**Feature**: session/session-lifecycle.feature
**Rule**: Stale detection uses per-session timers

## Context

Stale detection marks sessions that have been inactive for 10 minutes. The original implementation uses a global polling loop that checks all sessions every 10 seconds.

## Behaviours Considered

### Option A: Global polling (current)
A background task runs every 10 seconds, iterates all sessions, and marks any session inactive for 10+ minutes as stale.

### Option B: Per-session timers (event-driven)
Each session has its own timer that starts when the session goes non-running. Any new event for that session resets the timer. When the timer fires, only that session transitions to Stale.

## Decision

Chose Option B (per-session timers). Event-driven stale detection is more precise — a session transitions to Stale exactly at the 10-minute mark rather than within a 10-second window. It also avoids unnecessary work: instead of checking all sessions every 10 seconds, only the sessions that actually need checking are evaluated.

## Rejected Alternatives

- **Option A** works but is imprecise (up to 10 seconds late) and does unnecessary work scanning active sessions that cannot be stale. For a small number of sessions the cost is negligible, but the event-driven model is conceptually cleaner and aligns with how the rest of the system already works (event → state change).
