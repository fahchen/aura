# Aura HUD Design Specification

This document defines the visual design for the Aura HUD, based on the React prototype as the source of truth.

---

## Overview

Aura HUD is a floating status window displaying AI coding session states. It uses a **liquid glass aesthetic** with translucent white backgrounds and high contrast text for visibility on any desktop background.

---

## DOM Structure

```
.prototype-container
├── .indicator
│   └── .indicator-circle
│       ├── .indicator-gloss
│       └── .indicator-icon
└── .session-list
    ├── .session-list-header (background layer, z-index: 1)
    ├── .session-list-header-content (z-index: 3)
    │   └── .session-list-title
    │       ├── .session-list-title-icon
    │       └── .session-list-title-count
    └── .session-list-content (z-index: 2)
        └── .session-row[.running|.idle|.attention|.compacting|.stale]
            ├── .session-header
            │   ├── .state-indicator[.replaceable]
            │   │   ├── .state-icon-default
            │   │   └── .state-icon-remove
            │   └── .session-name
            │       └── .session-name-text
            └── .session-event
                ├── .tool-item
                │   ├── .tool-icon
                │   └── .tool-label
                └── .tool-placeholder
```

---

## Layout Structure

### Container
| Property | Value |
|----------|-------|
| Position | Fixed, top: 30px, centered horizontally |
| Width | **320px** |
| Layout | Flex column |
| Gap | **12px** (between indicator and session list) |
| Transform | `translateX(-50%)` + drag offset |

### Indicator (Collapsed View)
| Property | Value |
|----------|-------|
| Size | **36×36px** |
| Border-radius | **12px** (rounded square) |
| Background | Glass gradient (see Glass Effects) |
| Backdrop | `blur(24px) saturate(200%)` |
| Border | `1px solid rgba(255,255,255,0.2)` |
| Icon size | **16px** |
| Cursor | `grab` / `grabbing` when dragging |

### Session List (Expanded View)
| Property | Value |
|----------|-------|
| Width | **320px** |
| Border-radius | **16px** |
| Background | Liquid glass gradient |
| Backdrop | `blur(32px) saturate(200%)` |

### Header Bar
| Property | Value |
|----------|-------|
| Height | **28px** |
| Padding | **6px 12px** |
| Layout | Flex centered, "N session(s)" text only |
| Font size | **11px** |
| Font weight | 400 |
| Color | `rgba(255,255,255,0.5)` |
| Cursor | `grab` / `grabbing` when dragging |

### Session Content
| Property | Value |
|----------|-------|
| Padding | **10px** |
| Gap | **4px** |
| Max height | **320px** |
| Border-radius | **14px** |
| Overflow | Auto (scrollbar hidden) |

### Session Row
| Property | Value |
|----------|-------|
| Layout | **Flex column** (vertical) |
| Gap | **3px** |
| Padding | **10px 14px** |
| Border-radius | **12px** |
| Background | Subtle glass gradient |

### Session Header (Line 1)
| Property | Value |
|----------|-------|
| Layout | Flex row |
| Gap | **8px** |
| Align | Center |
| Contents | State icon (14×14) + Session name |

### Session Event (Line 2)
| Property | Value |
|----------|-------|
| Padding-left | **24px** (aligns under name, not icon) |
| Gap | **6px** |
| Min height | **18px** |
| Contents | Tool icon (12px) + tool label, OR placeholder text |

---

## Glass Effects

### Indicator Circle
```css
background: linear-gradient(
  135deg,
  rgba(255, 255, 255, 0.15) 0%,
  rgba(255, 255, 255, 0.05) 50%,
  rgba(255, 255, 255, 0.1) 100%
);
backdrop-filter: blur(24px) saturate(200%);
border: 1px solid rgba(255, 255, 255, 0.2);
box-shadow:
  0 8px 32px rgba(0, 0, 0, 0.2),
  0 2px 8px rgba(0, 0, 0, 0.1),
  inset 0 1px 1px rgba(255, 255, 255, 0.3),
  inset 0 -1px 1px rgba(0, 0, 0, 0.1);
```

### Indicator Gloss
```css
position: absolute;
top: 0; left: 0; right: 0;
height: 50%;
background: linear-gradient(
  180deg,
  rgba(255, 255, 255, 0.25) 0%,
  rgba(255, 255, 255, 0.05) 100%
);
border-radius: 12px 12px 50% 50%;
```

### Session List Background
```css
background: linear-gradient(
  165deg,
  rgba(255, 255, 255, 0.12) 0%,
  rgba(255, 255, 255, 0.05) 30%,
  rgba(255, 255, 255, 0.08) 70%,
  rgba(255, 255, 255, 0.1) 100%
);
backdrop-filter: blur(32px) saturate(200%);
border: 1px solid rgba(255, 255, 255, 0.2);
box-shadow:
  0 8px 40px rgba(0, 0, 0, 0.15),
  0 2px 12px rgba(0, 0, 0, 0.1),
  inset 0 1px 1px rgba(255, 255, 255, 0.3),
  inset 0 -1px 1px rgba(0, 0, 0, 0.05);
```

### Session Content Layer
```css
background: linear-gradient(
  180deg,
  rgba(255, 255, 255, 0.08) 0%,
  rgba(255, 255, 255, 0.04) 100%
);
backdrop-filter: blur(32px) saturate(200%);
border: 1px solid rgba(255, 255, 255, 0.15);
border-top: none;
box-shadow:
  0 4px 12px rgba(0, 0, 0, 0.15),
  0 8px 32px rgba(0, 0, 0, 0.2),
  0 16px 48px rgba(0, 0, 0, 0.15);
```

### Session Row
```css
background: linear-gradient(
  135deg,
  rgba(255, 255, 255, 0.06) 0%,
  rgba(255, 255, 255, 0.02) 100%
);
```

---

## Colors

### Text Colors
| Element | Color |
|---------|-------|
| Session name | `rgba(255,255,255,0.95)` |
| State icon | `rgba(255,255,255,0.7)` |
| Tool text | `rgba(255,255,255,0.6)` |
| Tool icon | `rgba(255,255,255,0.5)` |
| Header count | `rgba(255,255,255,0.5)` |
| Placeholder text | `rgba(255,255,255,0.3)` |
| Remove button | `rgba(255,255,255,0.3)` |

### Background Gradients
| Element | Gradient |
|---------|----------|
| Indicator circle | `linear-gradient(135deg, rgba(255,255,255,0.15) 0%, rgba(255,255,255,0.05) 50%, rgba(255,255,255,0.1) 100%)` |
| Indicator gloss | `linear-gradient(180deg, rgba(255,255,255,0.25) 0%, rgba(255,255,255,0.05) 100%)` |
| Session list header | `linear-gradient(165deg, rgba(255,255,255,0.12) 0%, rgba(255,255,255,0.05) 30%, rgba(255,255,255,0.08) 70%, rgba(255,255,255,0.1) 100%)` |
| Session content | `linear-gradient(180deg, rgba(255,255,255,0.08) 0%, rgba(255,255,255,0.04) 100%)` |
| Session row | `linear-gradient(135deg, rgba(255,255,255,0.06) 0%, rgba(255,255,255,0.02) 100%)` |
| Session row (hover) | `linear-gradient(135deg, rgba(255,255,255,0.15) 0%, rgba(255,255,255,0.08) 100%)` |

### Border Colors
| Element | Color |
|---------|-------|
| Indicator circle | `rgba(255,255,255,0.2)` |
| Session list header | `rgba(255,255,255,0.2)` |
| Session content | `rgba(255,255,255,0.15)` |

### Shadow Colors
| Element | Shadows |
|---------|---------|
| Indicator | `rgba(0,0,0,0.2)`, `rgba(0,0,0,0.1)`, `rgba(255,255,255,0.3)` inset, `rgba(0,0,0,0.1)` inset |
| Indicator (hover) | `rgba(0,0,0,0.25)`, `rgba(0,0,0,0.15)`, `rgba(255,255,255,0.4)` inset |
| Session list | `rgba(0,0,0,0.15)`, `rgba(0,0,0,0.1)`, `rgba(255,255,255,0.3)` inset, `rgba(0,0,0,0.05)` inset |
| Session content | `rgba(0,0,0,0.15)`, `rgba(0,0,0,0.2)`, `rgba(0,0,0,0.15)` |
| Row (hover) | `rgba(0,0,0,0.06)` |
| Attention row | `rgba(255,255,255,0.1)`, `rgba(255,255,255,0.2)` inset |

### Text Shadows
| Element | Shadow |
|---------|--------|
| Session name | `0 1px 3px rgba(0,0,0,0.4)` |
| Header count | `0 1px 2px rgba(0,0,0,0.3)` |
| State icon | `0 1px 2px rgba(0,0,0,0.3)` |
| Tool text | `0 1px 2px rgba(0,0,0,0.3)` |

### Attention Pulse Animation
| Keyframe | Shadows |
|----------|---------|
| 0%, 100% | `rgba(0,0,0,0.2)`, `rgba(255,255,255,0.15)` glow, `rgba(255,255,255,0.4)` inset |
| 50% | `rgba(0,0,0,0.25)`, `rgba(255,255,255,0.25)` glow, `rgba(255,255,255,0.5)` inset |

### Danger/Remove Colors
| Element | Color |
|---------|-------|
| Remove button (hover bg) | `rgba(239,68,68,0.2)` |
| Remove button (hover text) | `rgba(239,68,68,0.9)` |

---

## State Styles

### Row Opacity by State
| State | Opacity | Notes |
|-------|---------|-------|
| Running | 1.0 | Default |
| Idle | 0.7 | Reduced |
| Attention | 1.0 | + glow shadow |
| Compacting | 1.0 | Default (icon opacity 0.9) |
| Stale | 0.5↔0.3 | Animated breathe |

### State Icon Opacity (constants.ts)
| State | Opacity |
|-------|---------|
| Running | 1.0 |
| Idle | 0.8 |
| Attention | 1.0 |
| Compacting | 0.9 |
| Stale | 0.8 |

### Attention State Shadow
```css
box-shadow:
  0 0 16px rgba(255, 255, 255, 0.1),
  inset 0 1px 1px rgba(255, 255, 255, 0.2);
```

---

## Icons (Lucide)

### State Icons (14px in row, 16px in indicator)
| State | Lucide Icon |
|-------|-------------|
| Running | `Cctv` |
| Idle | `MessageSquareCode` |
| Attention | `BellRing` |
| Compacting | `Cookie` |
| Stale | `Ghost` |

### Tool Icons (12px)
| Tool | Lucide Icon |
|------|-------------|
| Task | `Bot` |
| Bash | `Terminal` |
| Glob | `BookSearch` |
| Grep | `FileSearchCorner` |
| Read | `Newspaper` |
| Edit | `FilePenLine` |
| Write | `FileBracesCorner` |
| WebFetch | `MonitorDown` |
| WebSearch | `Binoculars` |
| mcp__* | `Plug` |
| (default) | `Ticket` |

### Header Icons (12px)
| Element | Lucide Icon |
|---------|-------------|
| Title icon | `Layers` |

### Indicator Icons (16px)
| State | Icon |
|-------|------|
| Idle (no sessions) | `Panda` |
| Attention | `BellRing` |
| Running | Cycles through creative icons |

### Creative Icons (Running State Cycle)
`WandSparkles`, `Sparkles`, `Flame`, `Zap`, `Brain`, `Spotlight`, `BicepsFlexed`, `Rocket`, `Cpu`, `Puzzle`, `Orbit`

---

## Animations

### Summary Table
| Animation | Duration | Easing | Details |
|-----------|----------|--------|---------|
| Icon cycle interval | 2500ms | - | Random selection (different from current) |
| Icon slide transition | 400ms | ease-out | Slide left/right |
| Shake (attention) | 150ms | ease-in-out, infinite, alternate | ±1.5px X |
| Breathe (stale) | 4000ms | ease-in-out, infinite | 0.5↔0.3 opacity |
| Row slide-in | 350ms | cubic-bezier(0.4, 0, 0.2, 1) | translateX(-12px) + scale(0.98) |
| Row slide-out | 300ms | cubic-bezier(0.4, 0, 1, 1) | translateX(12px) + scale(0.98) |
| Expand in | 400ms | cubic-bezier(0.34, 1.56, 0.64, 1) | scale(0.9) + translateY(-12px) |
| Collapse out | 300ms | cubic-bezier(0.4, 0, 1, 1) | scale(0.9) + translateY(-12px) |
| Tool fade-in | 400ms | cubic-bezier(0.4, 0, 0.2, 1) | translateY(6px) |
| Icon swap (hover) | 300ms | cubic-bezier(0.4, 0, 0.2, 1) | Slide + fade |

---

### Indicator Icon Cycling
When in `running` state, the indicator cycles through creative icons:

| Property | Value |
|----------|-------|
| Interval | 2500ms |
| Selection | Random (always different from current) |
| Exit animation | `slideOutToLeft` - 400ms ease-out |
| Enter animation | `slideInFromRight` - 400ms ease-out |

```css
@keyframes slideOutToLeft {
  from {
    transform: translateX(0);
    opacity: 1;
  }
  to {
    transform: translateX(-100%);
    opacity: 0;
  }
}

@keyframes slideInFromRight {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
```

---

### State Icon ↔ Remove Icon Swap
On hover, the state icon swaps with the remove (Bomb) icon:

| Property | Value |
|----------|-------|
| Trigger | Hover on `.session-row` when `.state-indicator.replaceable` |
| Duration | 300ms |
| Easing | cubic-bezier(0.4, 0, 0.2, 1) |

| Element | Default | On Hover |
|---------|---------|----------|
| State icon | translateX(0), opacity 1 | translateX(16px), opacity 0 |
| Remove icon | translateX(-16px), opacity 0 | translateX(0), opacity 1 |

```css
.state-indicator.replaceable .state-icon-default {
  transform: translateX(0);
  opacity: 1;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.3s ease;
}

.state-indicator.replaceable .state-icon-remove {
  position: absolute;
  inset: 0;
  transform: translateX(-16px);
  opacity: 0;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.3s ease;
}

.session-row:hover .state-indicator.replaceable .state-icon-default {
  transform: translateX(16px);
  opacity: 0;
}

.session-row:hover .state-indicator.replaceable .state-icon-remove {
  transform: translateX(0);
  opacity: 1;
}
```

---

### Session List Expand/Collapse
```css
@keyframes expandIn-glass {
  from {
    opacity: 0;
    transform: scale(0.9) translateY(-12px);
    filter: blur(4px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
    filter: blur(0);
  }
}

@keyframes collapseOut-glass {
  to {
    opacity: 0;
    transform: scale(0.9) translateY(-12px);
    filter: blur(4px);
  }
}
```

---

### Session Row Animations
```css
/* Row appearing */
@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-12px) scale(0.98);
    filter: blur(2px);
  }
  to {
    opacity: 1;
    transform: translateX(0) scale(1);
    filter: blur(0);
  }
}

/* Row removing */
@keyframes slideOut {
  to {
    opacity: 0;
    transform: translateX(12px) scale(0.98);
    filter: blur(2px);
  }
}
```

---

### Tool Item Fade-In
```css
@keyframes fadeIn-glass {
  from {
    opacity: 0;
    transform: translateY(6px);
    filter: blur(2px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
    filter: blur(0);
  }
}
```

---

### State Animations

#### Attention - Shake + Pulse
```css
.indicator.attention .indicator-circle {
  animation: shake 0.15s ease-in-out infinite alternate,
             pulse-attention-glass 2s ease-in-out infinite;
}

@keyframes shake {
  0% { transform: translateX(-1.5px); }
  100% { transform: translateX(1.5px); }
}

@keyframes pulse-attention-glass {
  0%, 100% {
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.2),
      0 0 20px rgba(255, 255, 255, 0.15),
      inset 0 1px 1px rgba(255, 255, 255, 0.4);
  }
  50% {
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.25),
      0 0 30px rgba(255, 255, 255, 0.25),
      inset 0 1px 1px rgba(255, 255, 255, 0.5);
  }
}
```

#### Stale - Breathe
```css
.session-row.stale {
  animation: breathe-glass 4s ease-in-out infinite;
}

@keyframes breathe-glass {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 0.3; }
}
```

---

### Unused (Reserved)
```css
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
```

---

## Typography

| Element | Font Size | Weight | Style |
|---------|-----------|--------|-------|
| Session name | 14px | **500** | normal |
| Tool label | 12px | 400 | **italic** |
| Placeholder | 12px | 400 | italic |
| Header count | 11px | 400 | normal |
| State icon | 14px | - | - |

**Font Family:** `'Maple Mono NF CN', 'Monaco', monospace`

---

## Placeholder Text

State-specific placeholder when no running tools:

| State | Text |
|-------|------|
| Idle | `waiting since {datetime}` |
| Stale | `inactive since {datetime}` |
| Attention | `{tool} needs permission` |
| Compacting | `compacting context...` |
| Running | Random: "thinking...", "drafting...", "building...", "planning...", "analyzing...", "pondering...", "processing...", "reasoning..." |

**Datetime format:** "Jan 17, 14:30" (short month, day, 24h time)

---

## Interactions

### Indicator
| Event | Action |
|-------|--------|
| Click (when collapsed) | Open/expand session list |
| Click (when expanded) | Hide/collapse session list |
| Mouse down | Initiate drag |
| Hover | Scale 1.08, enhanced shadow |
| Cursor | `grab` → `grabbing` when dragging |

### Indicator Hover Effect
```css
.indicator:hover .indicator-circle {
  transform: scale(1.08);
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.25),
    0 4px 12px rgba(0, 0, 0, 0.15),
    inset 0 1px 2px rgba(255, 255, 255, 0.4),
    inset 0 -1px 1px rgba(0, 0, 0, 0.1);
}
```

### Session List Header
| Element | Event | Action |
|---------|-------|--------|
| Header content | Mouse down | Initiate drag (session list only) |

### Session Row
| Element | Event | Action |
|---------|-------|--------|
| Row | Hover | Enhanced glass background |
| State indicator (.replaceable) | Hover | Bomb icon slides in, state icon slides out |
| State indicator (.replaceable) | Click | Remove session |

### Row Hover Effect
```css
.session-row:hover {
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.15) 0%,
    rgba(255, 255, 255, 0.08) 100%
  );
  backdrop-filter: blur(40px) saturate(180%);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}
```

### State Icon Replacement (Hover)
```css
/* Default icon slides right and fades */
.session-row:hover .state-indicator.replaceable .state-icon-default {
  transform: translateX(16px);
  opacity: 0;
}
/* Remove icon slides in from left */
.session-row:hover .state-indicator.replaceable .state-icon-remove {
  transform: translateX(0);
  opacity: 1;
}
```

---

## Expand/Collapse Behavior

| Trigger | Action |
|---------|--------|
| Sessions: 0 → >0 | Auto-expand |
| Sessions: >0 → 0 | Auto-collapse |
| Indicator click (no sessions) | No action (list stays hidden) |
| Indicator click (collapsed, has sessions) | Expand session list |
| Indicator click (expanded) | Collapse session list |

---

## Drag Behavior

### Draggable Elements
| Element | Trigger |
|---------|---------|
| Indicator | `mousedown` on `.indicator` |
| Session list header | `mousedown` on `.session-list-header-content` |

### Cursor States
| State | Cursor |
|-------|--------|
| Idle (hovering) | `grab` |
| Active (dragging) | `grabbing` |

### Position Calculation
- **Initial position**: `{ x: 0, y: 0 }`
- **Base transform**: `translateX(-50%)` (centers container horizontally)
- **Drag offset**: Added to base transform as `translate(${x}px, ${y}px)`
- **Final transform**: `translateX(-50%) translate(${position.x}px, ${position.y}px)`

### Drag Implementation
1. **mousedown**:
   - `preventDefault()` to avoid text selection
   - Record start mouse position (`clientX`, `clientY`)
   - Record current element position
   - Set `isDragging = true`

2. **mousemove** (while dragging):
   - Calculate delta: `dx = clientX - startX`, `dy = clientY - startY`
   - Update position: `{ x: startPos.x + dx, y: startPos.y + dy }`

3. **mouseup**:
   - Set `isDragging = false`

### Event Handling
- Events (`mousemove`, `mouseup`) are attached to `document` during drag
- This ensures smooth dragging even when cursor moves outside the element
- Events are removed on `mouseup` or component cleanup

### Position Persistence
- Indicator and session list have independent positions
- Each position persists across expand/collapse cycles (stored in memory)
- Both positions reset to center on program restart (not persisted to disk)
- Session list starts offset below indicator, then moves independently once dragged

### Drag vs Click Discrimination
- Dragging MUST NOT trigger click behavior (toggle session list)
- If mouse moves during drag, suppress the click event on mouseup
- Only pure clicks (mousedown + mouseup with no/minimal movement) toggle the session list

---

## Visual States

### Session Row States
| State | Visual Effect |
|-------|---------------|
| Running | Full opacity, white icon |
| Idle | Reduced opacity (70%), muted icon color |
| Attention | Shake animation + pulsing glow shadow |
| Compacting | Normal appearance with progress icon |
| Stale | Breathing opacity animation (50%↔30%) |

### UI Variants
| Variant | Visual Effect |
|---------|---------------|
| Removable sessions | State icon swaps to remove icon on row hover |

---

## Layer Stacking

The session list uses three visual layers:

1. **Background layer** - Glass effect behind the header
2. **Content layer** - Scrollable container for session rows
3. **Header content** - Title and session count (always visible on top)

---

## Dimensions Summary

| Element | Value |
|---------|-------|
| Container width | 320px |
| Container gap | 12px |
| Indicator size | 36×36px |
| Indicator border-radius | 12px |
| Session list border-radius | 16px |
| Header height | 28px |
| Header padding | 6px 12px |
| Content padding | 10px |
| Content gap | 4px |
| Content max-height | 320px |
| Content border-radius | 14px |
| Row padding | 10px 14px |
| Row border-radius | 12px |
| Row internal gap | 3px |
| Event padding-left | 24px |
| Event gap | 6px |
| Event min-height | 18px |
| Tool item gap | 8px |
| State icon size | 14px (row), 16px (indicator) |
| Tool icon size | 12px |
| Max sessions displayed | ~6 (based on 320px height) |

---

## Reference

- **Prototype:** `prototype/` directory
- **Lucide icons:** https://lucide.dev/icons
- **gpui docs:** https://github.com/zed-industries/zed/tree/main/crates/gpui
