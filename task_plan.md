# Task Plan: Aura MVP

## Goal
Build Aura - a real-time situational awareness HUD for AI code agents.

## Phased Approach

| Phase | Scope | Status |
|-------|-------|--------|
| **Phase 1** | Claude Code hooks plugin → Aura HUD | **Current** |
| **Phase 2** | `aura run` PTY wrapper for other CLIs | Future |

## Task Progress

- [x] Task 1: Research Claude Code Hooks
- [x] Task 2: Project Scaffolding
- [x] Task 2.5: Architecture Refactoring (agent-agnostic)
- [x] Task 3: Hook Handler
- [x] Task 4: Daemon & Task Registry
- [x] Task 5: HUD UI (gpui.rs)
- [ ] Task 6: Polish & Testing ← **CURRENT**

---

## Current: Task 6 - Polish & Testing

### Task 5 Completed (Commit: e40aa2e)
- [x] gpui dependency and basic window
- [x] Session row layout (status dot + name + tools)
- [x] Tool cycling with cross-fade animation (1.5-2s random)
- [x] Lucide SVG icons for all tool types
- [x] Synchronized switching across sessions
- [x] AssetSource for embedded SVG loading

### Task 6 TODO
- [ ] **UI Redesign**: Two-icon status bar with hover-to-expand
  - Default: Two small icons (attention + aggregate state)
  - Hover: Expand to show full session list
- [ ] Connect HUD to live registry (currently demo data)
- [ ] End-to-end test with Claude Code hooks
- [ ] Error handling and edge cases

### Notes
- HUD shows session rows in single window (pivoted from notch-flanking design)
- Time-based animation using `request_animation_frame()` + `Instant::now()`
- Random seed per session for varied timing

### Reference
- gpui source: https://github.com/zed-industries/zed/tree/main/crates/gpui
- UI design: `notes.md`

---

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│  Claude Code    │     │  Other Agents   │
│  (hooks API)    │     │  (Phase 2)      │
└────────┬────────┘     └────────┬────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│  aura-hook      │     │  aura run       │
│  (adapter)      │     │  (PTY wrapper)  │
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │
                     ▼
              AgentEvent (generic)
                     │ IPC
                     ▼
           ┌─────────────────┐
           │  Aura Daemon    │
           │  (HUD + gpui)   │
           └─────────────────┘
```

---

## State Model (5 States)

| State | Icon (SVG) | Color |
|-------|------------|-------|
| Running | play | #22C55E |
| Idle | stop | #3B82F6 |
| Attention | bell | #EAB308 |
| Compacting | refresh | #A855F7 |
| Stale | pause | #6B7280 |

---

## Tool Icons (SVG)

| Tool | Icon | Tool | Icon |
|------|------|------|------|
| Task | robot | Bash | terminal |
| Glob | folder | Grep | search |
| Read | book | Edit | pencil |
| Write | file | WebFetch | globe |
| WebSearch | search | mcp__* | plug |
| (other) | gear | | |

---

## References

- **Full plan:** `~/.claude/plans/dynamic-strolling-falcon.md`
- **Hook docs:** https://code.claude.com/docs/en/hooks
