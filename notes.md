# Aura HUD UI Design

## Design Goals
- Ultra-minimal notch-hugging icons
- At-a-glance status without taking screen space
- Native macOS menu bar integration feel
- Click to expand full details (future)

---

## Layout: Notch-Flanking Icons

Two small status icons positioned on either side of the macOS notch:

```
    ┌────────────────────────────────────────────────────┐
    │  🔔    ┌─────────┐    ⏹   │  ← menu bar
    │  LEFT  │  notch  │  RIGHT │
    │        └─────────┘        │
    └────────────────────────────────────────────────────┘
```

### Icon Logic (Symmetric - Always 2 or 0)

| Left Icon | Condition |
|-----------|-----------|
| 🔔 Bell (yellow) | Any session needs Attention |
| ✓ Check (green) | No attention needed |

| Right Icon | Condition |
|------------|-----------|
| ⣿ Matrix (green, animated) | At least one session Running |
| ↻ Refresh (purple) | Any session Compacting (and none Running) |
| ⏹ Stop (blue) | All sessions Idle |
| ⏸ Pause (gray) | All sessions Stale |

**Rule: If no sessions exist, hide both icons. Otherwise, always show both.**

### Priority (Right Icon)
1. Running (green) - highest priority
2. Compacting (purple)
3. Idle (blue)
4. Stale (gray) - lowest priority

---

## State Matrix (Symmetric)

**Always 2 icons or 0 icons - never 1.**

| Sessions State | Has Attention? | Left Icon | Right Icon |
|----------------|----------------|-----------|------------|
| No sessions | - | (hidden) | (hidden) |
| All Stale | No | ✓ Green | ⏸ Gray |
| All Stale | Yes | 🔔 Yellow | ⏸ Gray |
| All Idle | No | ✓ Green | ⏹ Blue |
| All Idle | Yes | 🔔 Yellow | ⏹ Blue |
| Any Compacting | No | ✓ Green | ↻ Purple |
| Any Compacting | Yes | 🔔 Yellow | ↻ Purple |
| Any Running | No | ✓ Green | ⣿ Matrix (animated) |
| Any Running | Yes | 🔔 Yellow | ⣿ Matrix (animated) |

### Visual Examples

```
No sessions (both hidden):
═══════════════════╡    ┃████┃    ╞═══════════════════

All idle, no attention:
═══════════════════╡ ✓ ┃████┃ ⏹ ╞═══════════════════
                 green      blue

All idle, has attention:
═══════════════════╡ 🔔 ┃████┃ ⏹ ╞═══════════════════
                 yellow      blue

Running, no attention:
═══════════════════╡ ✓ ┃████┃ ⣿ ╞═══════════════════
                 green    green
                         animated

Running, has attention:
═══════════════════╡ 🔔 ┃████┃ ⣿ ╞═══════════════════
                 yellow   green
                         animated
```

---

## Dimensions

### Icon Size
- Icon: 16x16px (menu bar standard)
- Padding: 4px around icon
- Total clickable area: 24x24px

### Positioning
- Left icon: Right edge of left menu bar area (hugging notch)
- Right icon: Left edge of right menu bar area (hugging notch)
- Y position: Centered in menu bar (~22px height)

```
Menu bar items ──→  🔔 ┃notch┃ ⏹  ←── System icons
                      ↑      ↑
                   ~4px gap from notch edges
```

---

## Visual Style (Liquid Glass Pill)

Each icon sits in a subtle liquid glass pill:

```
    ╭───────╮
    │░░ 🔔 ░│  ← frosted glass background
    ╰───────╯
```

### Styling
- Background: rgba(255, 255, 255, 0.1) with blur
- Border: 1px rgba(255, 255, 255, 0.2)
- Corner radius: 6px (pill shape)
- Icon color: State-specific (see below)

### State Colors

| State | Color | Hex |
|-------|-------|-----|
| Running | Green | #22C55E |
| Idle | Blue | #3B82F6 |
| Attention | Yellow | #EAB308 |
| Compacting | Purple | #A855F7 |
| Stale | Gray | #6B7280 |

---

## State Icons

### Static Icons (SVG Paths, 24x24 viewBox)

| Icon | SVG Path |
|------|----------|
| ✓ Check | `M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z` |
| 🔔 Bell | `M12 22c1.1 0 2-.9 2-2h-4c0 1.1.9 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2z` |
| ⏹ Stop | `M6 6h12v12H6z` |
| ↻ Refresh | `M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z` |
| ⏸ Pause | `M6 19h4V5H6v14zm8-14v14h4V5h-4z` |

### Matrix Animation (Running State)

Inspired by [ElevenLabs Matrix](https://ui.elevenlabs.io/docs/components/matrix).

```
Dot-matrix grid: 3x3 (compact for 16px icon)

  ● ○ ●      ○ ● ○      ● ● ○
  ○ ● ○  →   ● ○ ●  →   ○ ● ●  → ...
  ● ○ ●      ○ ● ○      ● ○ ●

Animation: Wave pattern, 60fps
Dot size: 2px with 1px gap
Color: #22C55E (green)
```

#### Animation Frames (Wave Pattern)
- 3x3 grid of circular dots
- Each dot has opacity 0.3 (off) or 1.0 (on)
- Wave sweeps diagonally across grid
- ~200ms per cycle (5 frames/second feel)

#### Implementation
```rust
struct MatrixIcon {
    frame: usize,
    dots: [[bool; 3]; 3],
}

// Wave pattern frames
const WAVE_FRAMES: [[[bool; 3]; 3]; 4] = [
    [[1,0,0], [0,1,0], [0,0,1]],  // diagonal \
    [[0,1,0], [1,0,1], [0,1,0]],  // diamond
    [[0,0,1], [0,1,0], [1,0,0]],  // diagonal /
    [[1,0,1], [0,1,0], [1,0,1]],  // X pattern
];
```

---

## Interaction

### Hover
- Slight scale up (1.05x)
- Increased background opacity

### Click (Future)
- Expand to show full session list panel
- Panel drops down below the icons
- Shows all sessions with details

### Auto-Hide
- Icons visible when: Any session exists
- Icons hidden when: No sessions for 30s

---

## Mockup (ASCII)

### State: No running sessions, has attention
```
═══════════════════╡ 🔔 ┃████┃ ⏹ ╞═══════════════════
                        notch
```

### State: Running session, has attention
```
═══════════════════╡ 🔔 ┃████┃ ▶ ╞═══════════════════
                        notch
```

### State: Running session, no attention
```
═══════════════════╡    ┃████┃ ▶ ╞═══════════════════
                        notch
```

### State: All idle, no attention
```
═══════════════════╡    ┃████┃ ⏹ ╞═══════════════════
                        notch
```

---

## Implementation Plan
1. Create two small windows positioned at notch edges
2. Implement icon rendering with SVG paths
3. Add liquid glass pill styling
4. Connect to registry for state updates
5. Add hover/click interactions
6. Future: Expandable dropdown panel

---

## Known Issues

### gpui WindowBackgroundAppearance Toggle (macOS)

**Issue**: Toggling `WindowBackgroundAppearance` between `Blurred` and `Transparent` at runtime doesn't work reliably on macOS.

**Root cause**: On macOS 12+, gpui uses `NSVisualEffectView` for blur. The view creation works, but removal/toggle at runtime doesn't properly update the window.

**Attempted fixes**:
- `window.refresh()` after `set_background_appearance()` - no effect
- Setting `Opaque` then `Transparent` to force view removal - no effect
- Reordering resize/appearance calls - no effect

**Workaround**: Use static `Transparent` (glass effect) or static `Blurred`. Don't try to toggle at runtime.

**Status**: Unresolved. May require gpui fix or different approach (e.g., overlay elements instead of window-level blur).

### gpui on_hover Not Firing on Mouse Leave

**Issue**: The `on_hover` callback sometimes doesn't fire `false` when mouse leaves the window, especially after window resize.

**Status**: Investigated but not fully resolved. The hover event fires intermittently.
