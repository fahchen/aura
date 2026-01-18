//! Session list rendering - expanded view with session rows
//!
//! Each row displays in two-line vertical layout:
//! - Line 1 (header): State icon (16x16) + session name
//! - Line 2 (event): Current tool with icon (or state-specific placeholder)
//!
//! Uses liquid glass theme with themed text colors on translucent backgrounds.

use super::animation::{calculate_shake_offset, ease_in_out};
use super::icons;
use super::theme::{ThemeColors, WINDOW_RADIUS};
use aura_common::{RunningTool, SessionInfo, SessionState, PLACEHOLDER_TEXTS};
use chrono::{DateTime, Local, Utc};
use gpui::{div, px, svg, Div, Hsla, InteractiveElement, ParentElement, Styled};
use std::time::Instant;

/// Session list dimensions
pub const WIDTH: f32 = 320.0; // Match prototype width
pub const ROW_HEIGHT: f32 = 56.0; // Two-line layout needs more height
pub const ROW_GAP: f32 = 4.0; // Gap between session rows
pub const MAX_SESSIONS: usize = 5;

/// Layout constants (matching React prototype)
const STATE_ICON_SIZE: f32 = 14.0; // State icon in session row
const HEADER_GAP: f32 = 8.0;
const EVENT_PADDING_LEFT: f32 = 24.0; // Icon width (14) + gap (8) + 2 = align under name


/// Render the content of a session row (two-line vertical layout)
///
/// Layout:
/// ```
/// .session-row {
///   flex-direction: column;
///   gap: 3px;
///   padding: 10px 14px;
/// }
/// .session-header { // Line 1
///   flex-direction: row;
///   gap: 8px;
///   // icon (16x16) + name
/// }
/// .session-event { // Line 2
///   padding-left: 24px;  // Align under name (icon 16px + gap 8px)
///   // tool or placeholder
/// }
/// ```
///
/// Note: The outer wrapper with hover handler is created in mod.rs since
/// it needs access to `cx.listener()` which is tied to HudView.
pub fn render_row_content(
    session: &SessionInfo,
    session_name: &str,
    tool_index: usize,
    fade_progress: f32,
    animation_start: Instant,
    // Icon swap animation parameters
    state_opacity: f32,
    state_x: f32,
    remove_opacity: f32,
    remove_x: f32,
    theme: &ThemeColors,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(3.0))
        .px(px(14.0))
        .py(px(10.0))
        .rounded(px(WINDOW_RADIUS))
        .bg(theme.row_bg)
        .hover(|style| style.bg(theme.row_hover_bg))
        // Session header (Line 1): icon + name
        .child(render_session_header(
            session.state,
            session_name,
            animation_start,
            state_opacity,
            state_x,
            remove_opacity,
            remove_x,
            theme,
        ))
        // Session event (Line 2): tool or placeholder
        .child(render_session_event(session, tool_index, fade_progress, theme))
}

/// Render the session header (Line 1): state icon + session name
fn render_session_header(
    state: SessionState,
    session_name: &str,
    animation_start: Instant,
    state_opacity: f32,
    state_x: f32,
    remove_opacity: f32,
    remove_x: f32,
    theme: &ThemeColors,
) -> Div {
    div()
        .w_full()
        .h(px(18.0)) // Explicit height for h_full children
        .flex()
        .flex_row()
        .items_center()
        .gap(px(HEADER_GAP))
        // State icon (fixed width, with opacity + shake)
        .child(render_state_indicator(state, animation_start, state_opacity, state_x, remove_opacity, remove_x, theme))
        // Session name (with ellipsis truncation)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .font_family("Maple Mono NF CN")
                .text_size(px(14.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme.text_primary)
                .whitespace_nowrap()
                .text_ellipsis()
                .child(session_name.to_string()),
        )
}

/// Render the session event (Line 2): tool or placeholder
fn render_session_event(
    session: &SessionInfo,
    tool_index: usize,
    fade_progress: f32,
    theme: &ThemeColors,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .pl(px(EVENT_PADDING_LEFT)) // Align under session name
        .h(px(18.0)) // Fixed height to prevent layout jumps
        .child(render_tool_or_placeholder(session, tool_index, fade_progress, theme))
}

/// Format a Unix timestamp as "Jan 17, 14:30"
fn format_datetime(unix_ts: u64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(unix_ts as i64, 0).unwrap_or_else(Utc::now);
    let local: DateTime<Local> = datetime.into();
    local.format("%b %d, %H:%M").to_string()
}

/// Get a stable placeholder text for Running state based on session_id hash
/// This prevents flickering that would occur if we used time-based random selection
fn get_stable_placeholder(session_id: &str) -> &'static str {
    // Use session_id hash for stable but varied selection
    let hash = session_id.bytes().fold(0usize, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as usize)
    });
    let idx = hash % PLACEHOLDER_TEXTS.len();
    PLACEHOLDER_TEXTS[idx]
}

/// Get state-specific placeholder text based on session state
fn get_placeholder_text(session: &SessionInfo) -> String {
    match session.state {
        SessionState::Idle => {
            if let Some(ts) = session.stopped_at {
                format!("waiting since {}", format_datetime(ts))
            } else {
                "waiting...".to_string()
            }
        }
        SessionState::Stale => {
            if let Some(ts) = session.stale_at {
                format!("inactive since {}", format_datetime(ts))
            } else {
                "inactive".to_string()
            }
        }
        SessionState::Attention => {
            let tool = session.permission_tool.as_deref().unwrap_or("Tool");
            format!("{} needs permission", tool)
        }
        SessionState::Compacting => "compacting context...".to_string(),
        SessionState::Running => get_stable_placeholder(&session.session_id).to_string(),
    }
}

/// Render placeholder text with AudioLines icon (italic per design spec)
fn render_placeholder(text: &str, theme: &ThemeColors) -> Div {
    div()
        .w_full() // Fill parent container width
        .h(px(18.0)) // Fixed height for consistent layout
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .overflow_hidden()
        .min_w_0()
        // AudioLines icon
        .child(
            div()
                .flex_shrink_0()
                .w(px(TOOL_ICON_WIDTH))
                .h(px(TOOL_ICON_WIDTH))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    svg()
                        .path("icons/audio-lines.svg")
                        .size(px(TOOL_ICON_WIDTH))
                        .text_color(theme.icon_tool),
                ),
        )
        // Placeholder text
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .font_family("Maple Mono NF CN")
                .text_size(px(12.0))
                .italic()
                .text_color(theme.text_secondary)
                .whitespace_nowrap()
                .text_ellipsis()
                .child(text.to_string()),
        )
}

/// Render tool or state-specific placeholder
/// Shows tools if available, otherwise shows state-specific placeholder text
fn render_tool_or_placeholder(
    session: &SessionInfo,
    tool_index: usize,
    fade_progress: f32,
    theme: &ThemeColors,
) -> Div {
    if session.running_tools.is_empty() {
        // Show state-specific placeholder
        let placeholder_text = get_placeholder_text(session);
        return div()
            .flex_1()
            .min_w_0() // Allow shrinking for text ellipsis
            .h(px(18.0)) // Fixed height to match tool display
            .overflow_hidden()
            .child(render_placeholder(&placeholder_text, theme));
    }

    // Render tools with cross-fade animation
    render_current_tool(&session.running_tools, tool_index, fade_progress, theme)
}

/// Render current tool with cross-fade animation
/// Shows one tool at a time, cycling through the list
fn render_current_tool(tools: &[RunningTool], tool_index: usize, fade_progress: f32, theme: &ThemeColors) -> Div {
    // Get current and next tool indices
    let current_idx = tool_index % tools.len();
    let next_idx = (tool_index + 1) % tools.len();
    let current_tool = &tools[current_idx];
    let next_tool = &tools[next_idx];

    // Apply easing to fade progress
    let progress = ease_in_out(fade_progress);
    let current_opacity = 1.0 - progress; // fades out
    let next_opacity = progress; // fades in

    // Slide animation offsets
    let slide_distance = 12.0;
    let current_y_offset = -progress * slide_distance; // slides up
    let next_y_offset = (1.0 - progress) * slide_distance; // slides down from above

    // Stack both tools with cross-fade opacity using a relative container
    div()
        .flex_1()
        .min_w_0() // Allow shrinking for text ellipsis
        .h(px(18.0))
        .relative()
        .overflow_hidden()
        // Current tool (fading out, sliding up)
        .child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .h(px(18.0))
                .top(px(current_y_offset))
                .flex()
                .items_center()
                .overflow_hidden()
                .opacity(current_opacity)
                .child(render_tool_with_icon(current_tool, theme)),
        )
        // Next tool (fading in, sliding down from above)
        .child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .h(px(18.0))
                .top(px(next_y_offset))
                .flex()
                .items_center()
                .overflow_hidden()
                .opacity(next_opacity)
                .child(render_tool_with_icon(next_tool, theme)),
        )
}

/// Icon width for consistent alignment
const TOOL_ICON_WIDTH: f32 = 12.0;

/// Render a tool with its SVG icon (using theme colors)
pub fn render_tool_with_icon(tool: &RunningTool, theme: &ThemeColors) -> Div {
    let icon_path = icons::tool_icon_asset(&tool.tool_name);
    let display_text = if tool.tool_name.starts_with("mcp__") {
        // Extract server name from mcp__server__function format
        let parts: Vec<&str> = tool.tool_name.split("__").collect();
        if parts.len() >= 3 {
            let server = parts[1];
            let func = tool.tool_label.as_deref().unwrap_or(parts[2]);
            format!("{}: {}", server, func)
        } else {
            tool.tool_label.clone().unwrap_or_else(|| tool.tool_name.clone())
        }
    } else {
        tool.tool_label.as_deref().unwrap_or(&tool.tool_name).to_string()
    };

    div()
        .w_full() // Fill parent container width
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0)) // Per design spec: event gap = 6px
        .overflow_hidden()
        .min_w_0()
        // Tool icon (SVG)
        .child(
            div()
                .flex_shrink_0()
                .w(px(TOOL_ICON_WIDTH))
                .h(px(TOOL_ICON_WIDTH))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    svg()
                        .path(icon_path)
                        .size(px(TOOL_ICON_WIDTH))
                        .text_color(theme.icon_tool),
                ),
        )
        // Tool label (italic per design spec, with ellipsis)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .font_family("Maple Mono NF CN")
                .text_size(px(12.0))
                .italic()
                .text_color(theme.text_secondary)
                .whitespace_nowrap()
                .text_ellipsis()
                .child(display_text),
        )
}

/// Extract session name from cwd (last folder name)
pub fn extract_session_name(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("session")
        .to_string()
}

/// Convert SessionState to color (used by indicator.rs)
#[allow(dead_code)]
pub fn state_to_color(state: SessionState) -> Hsla {
    use super::theme;
    match state {
        SessionState::Running => theme::GREEN,
        SessionState::Idle => theme::BLUE,
        SessionState::Attention => theme::YELLOW,
        SessionState::Compacting => theme::PURPLE,
        SessionState::Stale => theme::GRAY,
    }
}

/// Render state indicator with SVG icon and opacity
///
/// Uses themed icon color with varying opacity based on state urgency.
/// On hover, swaps to remove (X) icon with slide animation.
fn render_state_indicator(
    state: SessionState,
    animation_start: Instant,
    state_opacity: f32,
    state_x: f32,
    remove_opacity: f32,
    remove_x: f32,
    theme: &ThemeColors,
) -> Div {
    let icon_path = icons::state_icon_path(state);
    let base_opacity = state_to_opacity(state);

    // Calculate shake offset for Attention state
    let shake_offset = if state == SessionState::Attention {
        calculate_shake_offset(animation_start)
    } else {
        0.0
    };

    // Themed icon with state-based opacity
    let state_icon_color = Hsla {
        a: base_opacity * state_opacity,
        ..theme.icon_state
    };

    let remove_icon_color = Hsla {
        a: 0.9 * remove_opacity,
        ..theme.icon_state
    };

    div()
        .flex_shrink_0()
        .w(px(STATE_ICON_SIZE))
        .h(px(STATE_ICON_SIZE))
        .relative()
        .overflow_hidden()
        // State icon (slides right, fades out on hover)
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w(px(STATE_ICON_SIZE))
                .h(px(STATE_ICON_SIZE))
                .flex()
                .items_center()
                .justify_center()
                .ml(px(shake_offset + state_x))
                .opacity(state_opacity)
                .child(
                    svg()
                        .path(icon_path)
                        .size(px(STATE_ICON_SIZE))
                        .text_color(state_icon_color),
                ),
        )
        // Remove icon (slides in from left, fades in on hover)
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w(px(STATE_ICON_SIZE))
                .h(px(STATE_ICON_SIZE))
                .flex()
                .items_center()
                .justify_center()
                .ml(px(remove_x))
                .opacity(remove_opacity)
                .child(
                    svg()
                        .path("icons/bomb.svg")
                        .size(px(STATE_ICON_SIZE))
                        .text_color(remove_icon_color),
                ),
        )
}

/// Get opacity for session state (from prototype)
fn state_to_opacity(state: SessionState) -> f32 {
    match state {
        SessionState::Running | SessionState::Attention => 1.0,
        SessionState::Compacting => 0.9,
        SessionState::Idle | SessionState::Stale => 0.8,
    }
}

/// Header bar height (28px per prototype)
pub const HEADER_HEIGHT: f32 = 28.0;

/// Calculate expanded window height based on session count
pub fn calculate_expanded_height(session_count: usize) -> f32 {
    let count = session_count.min(MAX_SESSIONS);
    // Header (28px) + rows + container padding (10px top + 10px bottom)
    HEADER_HEIGHT + (ROW_HEIGHT + ROW_GAP) * count as f32 + 20.0
}
