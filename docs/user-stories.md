# Feature Specification: Aura HUD

**Feature Branch**: `main`
**Created**: 2025-01-17
**Status**: Implemented (React Prototype)
**Input**: Real-time situational awareness HUD for AI code agents

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Monitor Active Sessions (Priority: P1)

As a developer using multiple AI coding agents, I want to see at-a-glance status of all active sessions so I know which ones need attention without switching contexts.

**Why this priority**: Core value proposition - without visibility into session states, the HUD provides no value.

**Independent Test**: Launch multiple Claude Code sessions in different terminals, verify all appear in the HUD with correct states.

**Acceptance Scenarios**:

1. **Given** no sessions are active, **When** I start a Claude Code session, **Then** the session appears in the HUD with Running state
2. **Given** a session is running, **When** it stops, **Then** the session shows Idle state with "waiting since {time}"
3. **Given** a session is idle for 60+ seconds, **When** viewing the HUD, **Then** the session shows Stale state with breathing animation
4. **Given** a session requests permission, **When** viewing the HUD, **Then** it shows Attention state with shake animation

---

### User Story 2 - Toggle Session List Visibility (Priority: P1)

As a developer, I want to collapse the session list to minimize screen clutter, and expand it when I need to check on sessions.

**Why this priority**: Essential for HUD usability - must be able to show/hide without obstruction.

**Independent Test**: Click indicator to toggle session list visibility, verify expand/collapse animations.

**Acceptance Scenarios**:

1. **Given** the session list is collapsed and sessions exist, **When** I click the indicator, **Then** the session list expands with animation
2. **Given** the session list is expanded, **When** I click the indicator, **Then** the session list collapses with animation
3. **Given** there are no sessions, **When** I click the indicator, **Then** nothing happens (list stays hidden)
4. **Given** the session list is expanded and all sessions end, **When** the last session is removed, **Then** the session list automatically closes

---

### User Story 3 - Reposition HUD (Priority: P1)

As a developer, I want to drag the HUD to any position on screen so it doesn't block my work area.

**Why this priority**: Must not obstruct IDE/editor - unusable otherwise.

**Independent Test**: Drag indicator or header, verify HUD moves and position persists across expand/collapse.

**Acceptance Scenarios**:

1. **Given** the HUD is at default position, **When** I drag the indicator, **Then** only the indicator moves with my cursor
2. **Given** I have dragged the indicator, **When** I collapse and expand the list, **Then** the indicator remains at the dragged position
3. **Given** I have positioned the indicator, **When** the program restarts, **Then** the indicator returns to default centered position
4. **Given** the session list is expanded, **When** I drag the indicator and release, **Then** the session list remains expanded (click not triggered)
5. **Given** the session list is expanded, **When** I drag the session list header, **Then** only the session list moves (independently from indicator)
6. **Given** I have dragged the session list, **When** I collapse and expand the list, **Then** the session list remains at its dragged position

---

### User Story 4 - View Running Tools (Priority: P2)

As a developer, I want to see which tools a session is currently using so I understand what it's doing.

**Why this priority**: Provides context for what's happening, but not critical for basic awareness.

**Independent Test**: Run a session that uses multiple tools, verify tool icons and names cycle in the HUD.

**Acceptance Scenarios**:

1. **Given** a session is running tools, **When** I view its row, **Then** I see the tool icon and name
2. **Given** a session is running multiple tools, **When** watching its row, **Then** the displayed tool cycles every 2 seconds
3. **Given** a session is running but no tools active, **When** viewing its row, **Then** I see a placeholder like "thinking..." or "planning..."

---

### User Story 5 - Auto-Expand/Collapse on Session Changes (Priority: P2)

As a developer, I want the session list to automatically appear when sessions start and disappear when all sessions end.

**Why this priority**: Reduces manual interaction, but not blocking for core functionality.

**Independent Test**: Start first session (list expands), end all sessions (list collapses).

**Acceptance Scenarios**:

1. **Given** there are no sessions, **When** a new session starts, **Then** the session list automatically expands
2. **Given** there is one session remaining, **When** that session ends, **Then** the session list automatically collapses
3. **Given** sessions already exist, **When** a new session starts, **Then** the list remains expanded (no re-animation)

---

### User Story 6 - Identify Session by Name (Priority: P2)

As a developer with multiple sessions, I want to see meaningful session names so I can distinguish between them.

**Why this priority**: Important for multi-session use but system works with default names.

**Independent Test**: Set session name via skill, verify it appears in HUD row.

**Acceptance Scenarios**:

1. **Given** a session has a custom name, **When** viewing its row, **Then** I see the session name
2. **Given** a session has no name, **When** viewing its row, **Then** I see the last segment of the working directory path
3. **Given** a session name is very long, **When** viewing its row, **Then** the name is truncated with ellipsis

---

### User Story 7 - Remove Session Manually (Priority: P3)

As a developer, I want to remove a stale or stuck session from the HUD without restarting.

**Why this priority**: Nice-to-have for cleanup, not essential for monitoring.

**Independent Test**: Hover over row, click remove icon, verify row disappears with animation.

**Acceptance Scenarios**:

1. **Given** I hover over a session row, **When** viewing the state icon area, **Then** it swaps to a remove (Bomb) icon
2. **Given** the remove icon is visible, **When** I click it, **Then** the session row slides out and is removed
3. **Given** I am not hovering, **When** viewing the row, **Then** I see the normal state icon (not remove)

---

### User Story 8 - Visual State Feedback on Indicator (Priority: P2)

As a developer, I want the indicator to show aggregate status at a glance without expanding the list.

**Why this priority**: Enables quick status check, but full list provides complete info.

**Independent Test**: Create sessions in different states, verify indicator icon changes accordingly.

**Acceptance Scenarios**:

1. **Given** no sessions exist, **When** viewing the indicator, **Then** it shows Panda icon with reduced opacity
2. **Given** at least one session needs attention, **When** viewing the indicator, **Then** it shows BellRing icon with shake + pulse
3. **Given** sessions are running (no attention), **When** watching the indicator, **Then** the icon cycles through creative icons every 2.5s

---

### Edge Cases

- What happens when a session name contains special characters or is extremely long? → Truncated with ellipsis
- What happens when many sessions exist (>6)? → List becomes scrollable (hidden scrollbar)
- What happens when a tool name is very long? → Truncated with ellipsis
- What happens when dragging and cursor leaves the window? → Drag continues (events on document)
- What happens when multiple sessions need attention simultaneously? → Indicator shows attention state, all rows shake
- What happens when I drag the indicator slightly and release? → Treated as drag (no click), session list state unchanged
- What happens when session list is open and all sessions are removed? → Session list automatically closes

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display all active AI coding sessions in a floating HUD
- **FR-002**: System MUST show session state (Running, Idle, Attention, Compacting, Stale) with distinct visual treatment
- **FR-003**: System MUST allow expanding/collapsing the session list via indicator click
- **FR-004**: System MUST allow repositioning the HUD via drag on indicator or header
- **FR-005**: System MUST persist drag position in memory across expand/collapse cycles
- **FR-006**: System MUST auto-expand when first session appears (0→1 transition)
- **FR-007**: System MUST auto-collapse when last session ends (1→0 transition)
- **FR-008**: System MUST display running tool(s) with icon and name, cycling if multiple
- **FR-009**: System MUST show state-appropriate placeholder text when no tools running
- **FR-010**: System MUST animate row entry/exit (slide-in from left, slide-out to right)
- **FR-011**: System MUST show remove icon on row hover, allowing session removal
- **FR-012**: System MUST cycle indicator icon through creative icons in running state
- **FR-013**: System MUST show shake + pulse animation in attention state
- **FR-014**: System MUST show breathing opacity animation in stale state
- **FR-015**: System MUST NOT open session list when clicking indicator with no sessions
- **FR-016**: System MUST distinguish drag from click - dragging MUST NOT trigger click behavior

### Key Entities

- **Session**: Represents an AI coding session with id, cwd, name, state, runningTools[], timestamps
- **State**: Enum of Running, Idle, Attention, Compacting, Stale with associated icons and animations
- **Tool**: Name string mapped to Lucide icon (Task→Bot, Bash→Terminal, etc.)

### Non-Requirements (Rejected Decisions)

These document decisions we explicitly made NOT to implement:

- **NR-001**: System MUST NOT persist HUD position to disk - resets on restart is acceptable
- **NR-002**: System MUST NOT show context menu on right-click - removed as unimplemented feature
- **NR-003**: System MUST NOT use Font Awesome or Nerd Fonts - Lucide icons only for consistency
- **NR-004**: System MUST NOT show tooltip on long session names - truncation with ellipsis is sufficient
- **NR-005**: System MUST NOT allow clicking indicator when no sessions exist - prevents empty list confusion
- **NR-006**: System MUST NOT persist session list expanded/collapsed state - auto-expand/collapse handles this

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 5 session states display with distinct visual treatment (icon, opacity, animation)
- **SC-002**: HUD responds to session changes within 100ms (perceived real-time)
- **SC-003**: Drag latency < 16ms (60fps smooth)
- **SC-004**: Session list expand/collapse animation completes in <500ms
- **SC-005**: Tool cycling updates every 2s without visual glitches
- **SC-006**: Indicator icon cycling updates every 2.5s with smooth transition
- **SC-007**: Position persists across 10+ expand/collapse cycles without drift
- **SC-008**: HUD supports 20+ concurrent sessions without performance degradation
