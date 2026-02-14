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
use crate::{PLACEHOLDER_TEXTS, RunningTool, SessionInfo, SessionState};
use chrono::{DateTime, Local, Utc};
use gpui::{
    Div, Hsla, InteractiveElement, ParentElement, Styled, Transformation, div, px, radians, svg,
};
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

/// Shared render arguments for a session row.
pub(crate) struct RowRenderArgs<'a> {
    pub(crate) tool_index: usize,
    pub(crate) fade_progress: f32,
    pub(crate) animation_start: Instant,
    pub(crate) state_opacity: f32,
    pub(crate) state_x: f32,
    pub(crate) remove_opacity: f32,
    pub(crate) remove_x: f32,
    pub(crate) theme: &'a ThemeColors,
}

/// Render the content of a session row (two-line vertical layout)
///
/// Layout:
/// ```text
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
pub(crate) fn render_row_content(
    session: &SessionInfo,
    session_name: &str,
    args: &RowRenderArgs<'_>,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(3.0))
        .px(px(14.0))
        .py(px(10.0))
        .rounded(px(WINDOW_RADIUS))
        .bg(args.theme.row_bg)
        .hover(|style| style.bg(args.theme.row_hover_bg))
        // Session header (Line 1): icon + name
        .child(render_session_header(session.state, session_name, args))
        // Session event (Line 2): tool or placeholder
        .child(render_session_event(session, args))
}

/// Render the session header (Line 1): state icon + session name
fn render_session_header(state: SessionState, session_name: &str, args: &RowRenderArgs<'_>) -> Div {
    div()
        .w_full()
        .h(px(18.0)) // Explicit height for h_full children
        .flex()
        .flex_row()
        .items_center()
        .gap(px(HEADER_GAP))
        // State icon (fixed width, with opacity + shake)
        .child(render_state_indicator(
            state,
            args.animation_start,
            args.state_opacity,
            args.state_x,
            args.remove_opacity,
            args.remove_x,
            args.theme,
        ))
        // Session name (with ellipsis truncation)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .font_family("Maple Mono NF CN")
                .text_size(px(14.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(args.theme.text_primary)
                .whitespace_nowrap()
                .text_ellipsis()
                .child(session_name.to_string()),
        )
}

/// Render the session event (Line 2): tool or placeholder
fn render_session_event(session: &SessionInfo, args: &RowRenderArgs<'_>) -> Div {
    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .pl(px(EVENT_PADDING_LEFT)) // Align under session name
        .h(px(18.0)) // Fixed height to prevent layout jumps
        .child(render_tool_or_placeholder(
            session,
            args.tool_index,
            args.fade_progress,
            args.animation_start,
            args.theme,
        ))
}

/// Format a Unix timestamp as "Jan 17, 14:30"
pub(crate) fn format_datetime(unix_ts: u64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(unix_ts as i64, 0).unwrap_or_else(Utc::now);
    let local: DateTime<Local> = datetime.into();
    local.format("%b %d, %H:%M").to_string()
}

/// Get a stable placeholder text for Running state based on session_id hash
/// This prevents flickering that would occur if we used time-based random selection
pub(crate) fn get_stable_placeholder(session_id: &str) -> &'static str {
    // Use session_id hash for stable but varied selection
    let hash = session_id.bytes().fold(0usize, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as usize)
    });
    let idx = hash % PLACEHOLDER_TEXTS.len();
    PLACEHOLDER_TEXTS[idx]
}

/// Get state-specific placeholder text based on session state
pub(crate) fn get_placeholder_text(session: &SessionInfo) -> String {
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
        SessionState::Waiting => "waiting for input".to_string(),
        SessionState::Compacting => "compacting context...".to_string(),
        SessionState::Running => get_stable_placeholder(&session.session_id).to_string(),
    }
}

/// Get placeholder icon path for a state
fn get_placeholder_icon(state: SessionState) -> &'static str {
    match state {
        SessionState::Waiting => "icons/wind.svg",
        _ => "icons/audio-lines.svg",
    }
}

/// Render placeholder text with state-specific icon (italic per design spec)
fn render_placeholder(text: &str, icon_path: &'static str, theme: &ThemeColors) -> Div {
    div()
        .w_full() // Fill parent container width
        .h(px(18.0)) // Fixed height for consistent layout
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .overflow_hidden()
        .min_w_0()
        // State-specific icon
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
    animation_start: Instant,
    theme: &ThemeColors,
) -> Div {
    if session.running_tools.is_empty() {
        if let Some(activity_text) = get_recent_activity_text(session, animation_start) {
            return div()
                .flex_1()
                .min_w_0()
                .h(px(18.0))
                .overflow_hidden()
                .child(render_activity_text(&activity_text, theme));
        }

        // Show state-specific placeholder
        let placeholder_text = get_placeholder_text(session);
        let icon_path = get_placeholder_icon(session.state);
        return div()
            .flex_1()
            .min_w_0() // Allow shrinking for text ellipsis
            .h(px(18.0)) // Fixed height to match tool display
            .overflow_hidden()
            .child(render_placeholder(&placeholder_text, icon_path, theme));
    }

    // Render tools with vertical slide (ticker) animation
    render_current_tool(&session.running_tools, tool_index, fade_progress, theme)
}

/// Render current tool with vertical slide (ticker) animation
/// Shows one tool at a time, cycling through the list
fn render_current_tool(
    tools: &[RunningTool],
    tool_index: usize,
    fade_progress: f32,
    theme: &ThemeColors,
) -> Div {
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

    // Stack both tools with vertical slide (ticker) using a relative container
    div()
        .flex_1()
        .min_w_0() // Allow shrinking for text ellipsis
        .h(px(18.0))
        .relative()
        .overflow_hidden()
        // Current tool (sliding up and out)
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
        // Next tool (sliding up from below)
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

/// Rotate through recent activity without repeats.
pub(crate) fn get_recent_activity_text(
    session: &SessionInfo,
    animation_start: Instant,
) -> Option<String> {
    let items = &session.recent_activity;
    if items.is_empty() {
        return None;
    }
    let seconds = animation_start.elapsed().as_secs();
    let idx = (seconds / 3) as usize % items.len();
    items.get(idx).cloned()
}

/// Render recent activity text (non-italic, neutral icon)
fn render_activity_text(text: &str, theme: &ThemeColors) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .child(
            svg()
                .path("icons/spotlight.svg")
                .size(px(10.0))
                .text_color(theme.text_secondary),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family("Maple Mono NF CN")
                .text_size(px(12.0))
                .text_color(theme.text_secondary)
                .whitespace_nowrap()
                .text_ellipsis()
                .child(text.to_string()),
        )
}

/// Icon width for consistent alignment
const TOOL_ICON_WIDTH: f32 = 12.0;

/// Format the display text for a tool, handling MCP server prefixes and special cases.
pub(crate) fn format_tool_display_text(tool_name: &str, tool_label: Option<&str>) -> String {
    if tool_name.starts_with("mcp__") {
        let parts: Vec<&str> = tool_name.split("__").collect();
        if parts.len() >= 3 {
            let server = parts[1];
            let func = tool_label.unwrap_or(parts[2]);
            format!("{}: {}", server, func)
        } else {
            tool_label.unwrap_or(tool_name).to_string()
        }
    } else if tool_name == "WebFetch" && tool_label.is_none() {
        "fetching...".to_string()
    } else {
        tool_label.unwrap_or(tool_name).to_string()
    }
}

/// Render a tool with its SVG icon (using theme colors)
pub(crate) fn render_tool_with_icon(tool: &RunningTool, theme: &ThemeColors) -> Div {
    let icon_path = icons::tool_icon_asset(&tool.tool_name);
    let display_text = format_tool_display_text(&tool.tool_name, tool.tool_label.as_deref());

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
pub(crate) fn extract_session_name(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("session")
        .to_string()
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

    // Calculate rotation for Waiting state (2 second full rotation, counter-clockwise)
    let rotation_radians = if state == SessionState::Waiting {
        let elapsed_ms = animation_start.elapsed().as_millis() as f32;
        let rotation_period_ms = 2000.0;
        -((elapsed_ms / rotation_period_ms) * std::f32::consts::TAU) // Negative for counter-clockwise
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

    // Build state icon SVG (with rotation for Waiting state)
    let state_icon_svg = svg()
        .path(icon_path)
        .size(px(STATE_ICON_SIZE))
        .text_color(state_icon_color);

    let state_icon_svg = if rotation_radians != 0.0 {
        state_icon_svg.with_transformation(Transformation::rotate(radians(rotation_radians)))
    } else {
        state_icon_svg
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
                .child(state_icon_svg),
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
pub(crate) fn state_to_opacity(state: SessionState) -> f32 {
    match state {
        SessionState::Running | SessionState::Attention | SessionState::Waiting => 1.0,
        SessionState::Compacting => 0.9,
        SessionState::Idle | SessionState::Stale => 0.8,
    }
}

/// Header bar height (28px per prototype)
pub const HEADER_HEIGHT: f32 = 28.0;

/// Calculate expanded window height based on session count
pub(crate) fn calculate_expanded_height(session_count: usize) -> f32 {
    let count = session_count.min(MAX_SESSIONS);
    // Header (28px) + rows + container padding (10px top + 10px bottom)
    HEADER_HEIGHT + (ROW_HEIGHT + ROW_GAP) * count as f32 + 20.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SessionInfo, SessionState};

    fn make_session(state: SessionState) -> SessionInfo {
        SessionInfo {
            session_id: "test-session".into(),
            cwd: "/home/user/project".into(),
            state,
            running_tools: vec![],
            name: None,
            stopped_at: None,
            stale_at: None,
            permission_tool: None,
            recent_activity: vec![],
        }
    }

    // --- extract_session_name tests ---

    #[test]
    fn session_name_from_path() {
        assert_eq!(extract_session_name("/home/user/project"), "project");
    }

    #[test]
    fn session_name_from_root() {
        assert_eq!(extract_session_name("/"), "session");
    }

    #[test]
    fn session_name_from_empty() {
        assert_eq!(extract_session_name(""), "session");
    }

    // --- get_placeholder_text tests ---

    #[test]
    fn placeholder_running() {
        let session = make_session(SessionState::Running);
        let text = get_placeholder_text(&session);
        assert!(
            crate::PLACEHOLDER_TEXTS.contains(&text.as_str()),
            "got: {}",
            text
        );
    }

    #[test]
    fn placeholder_idle_with_timestamp() {
        let mut session = make_session(SessionState::Idle);
        session.stopped_at = Some(1705500600); // Jan 17, 2024, 14:30 UTC
        let text = get_placeholder_text(&session);
        assert!(text.starts_with("waiting since "), "got: {}", text);
    }

    #[test]
    fn placeholder_idle_without_timestamp() {
        let session = make_session(SessionState::Idle);
        assert_eq!(get_placeholder_text(&session), "waiting...");
    }

    #[test]
    fn placeholder_stale_with_timestamp() {
        let mut session = make_session(SessionState::Stale);
        session.stale_at = Some(1705500600);
        let text = get_placeholder_text(&session);
        assert!(text.starts_with("inactive since "), "got: {}", text);
    }

    #[test]
    fn placeholder_attention_with_tool() {
        let mut session = make_session(SessionState::Attention);
        session.permission_tool = Some("Read".into());
        assert_eq!(get_placeholder_text(&session), "Read needs permission");
    }

    #[test]
    fn placeholder_waiting() {
        let session = make_session(SessionState::Waiting);
        assert_eq!(get_placeholder_text(&session), "waiting for input");
    }

    #[test]
    fn placeholder_compacting() {
        let session = make_session(SessionState::Compacting);
        assert_eq!(get_placeholder_text(&session), "compacting context...");
    }

    // --- get_stable_placeholder tests ---

    #[test]
    fn stable_placeholder_deterministic() {
        let a = get_stable_placeholder("session-abc");
        let b = get_stable_placeholder("session-abc");
        assert_eq!(a, b);
    }

    #[test]
    fn stable_placeholder_varies() {
        let a = get_stable_placeholder("session-abc");
        let b = get_stable_placeholder("session-xyz-different");
        // Both must be valid placeholders regardless of whether they happen to match
        assert!(crate::PLACEHOLDER_TEXTS.contains(&a));
        assert!(crate::PLACEHOLDER_TEXTS.contains(&b));
    }

    // --- state_to_opacity tests ---

    #[test]
    fn opacity_running_states() {
        assert_eq!(state_to_opacity(SessionState::Running), 1.0);
        assert_eq!(state_to_opacity(SessionState::Attention), 1.0);
        assert_eq!(state_to_opacity(SessionState::Waiting), 1.0);
    }

    #[test]
    fn opacity_compacting() {
        assert_eq!(state_to_opacity(SessionState::Compacting), 0.9);
    }

    #[test]
    fn opacity_idle_stale() {
        assert_eq!(state_to_opacity(SessionState::Idle), 0.8);
        assert_eq!(state_to_opacity(SessionState::Stale), 0.8);
    }

    // --- get_recent_activity_text tests ---

    #[test]
    fn recent_activity_empty() {
        let session = make_session(SessionState::Running);
        let text = get_recent_activity_text(&session, Instant::now());
        assert!(text.is_none());
    }

    #[test]
    fn recent_activity_single_item() {
        let mut session = make_session(SessionState::Running);
        session.recent_activity = vec!["main.rs".into()];
        let text = get_recent_activity_text(&session, Instant::now());
        assert_eq!(text, Some("main.rs".into()));
    }

    #[test]
    fn recent_activity_cycles() {
        let mut session = make_session(SessionState::Running);
        session.recent_activity = vec!["a".into(), "b".into(), "c".into()];
        // At t=0, idx = (0/3) % 3 = 0 -> "a"
        let text = get_recent_activity_text(&session, Instant::now());
        assert_eq!(text, Some("a".into()));
        // At t=3s, idx = (3/3) % 3 = 1 -> "b"
        let start = Instant::now() - std::time::Duration::from_secs(3);
        let text = get_recent_activity_text(&session, start);
        assert_eq!(text, Some("b".into()));
    }

    // --- calculate_expanded_height tests ---

    #[test]
    fn expanded_height_one_session() {
        let expected = HEADER_HEIGHT + (ROW_HEIGHT + ROW_GAP) * 1.0 + 20.0;
        assert_eq!(calculate_expanded_height(1), expected);
    }

    #[test]
    fn expanded_height_max_sessions() {
        let expected = HEADER_HEIGHT + (ROW_HEIGHT + ROW_GAP) * 5.0 + 20.0;
        assert_eq!(calculate_expanded_height(5), expected);
    }

    #[test]
    fn expanded_height_capped() {
        // 10 sessions should be capped at MAX_SESSIONS (5)
        assert_eq!(calculate_expanded_height(10), calculate_expanded_height(5));
    }

    // --- format_tool_display_text tests ---

    #[test]
    fn format_mcp_tool_with_label() {
        assert_eq!(
            format_tool_display_text("mcp__github__search", Some("react")),
            "github: react"
        );
    }

    #[test]
    fn format_mcp_tool_without_label() {
        assert_eq!(
            format_tool_display_text("mcp__memory__create_entities", None),
            "memory: create_entities"
        );
    }

    #[test]
    fn format_regular_tool_with_label() {
        assert_eq!(format_tool_display_text("Read", Some("main.rs")), "main.rs");
    }

    #[test]
    fn format_regular_tool_without_label() {
        assert_eq!(format_tool_display_text("Read", None), "Read");
    }

    #[test]
    fn format_webfetch_without_label() {
        assert_eq!(format_tool_display_text("WebFetch", None), "fetching...");
    }
}
