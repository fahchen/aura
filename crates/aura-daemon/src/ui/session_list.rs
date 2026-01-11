//! Session list rendering - expanded view with session rows
//!
//! Each row displays:
//! - Status dot (colored by state)
//! - Session name (folder name from cwd, with marquee for long names)
//! - Current tool with cross-fade animation

use super::animation::{ease_in_out, ease_out, MARQUEE_CHAR_WIDTH, RESET_DURATION_MS};
use super::icons;
use aura_common::{RunningTool, SessionState};
use gpui::{div, px, svg, Div, ParentElement, Styled};
use unicode_width::UnicodeWidthStr;

/// Session list dimensions
pub const WIDTH: f32 = 200.0;
pub const ROW_HEIGHT: f32 = 32.0;
pub const ROW_GAP: f32 = 4.0;
pub const MAX_SESSIONS: usize = 5;

/// Column widths for table-style layout
pub const STATUS_DOT_WIDTH: f32 = 18.0;
pub const SESSION_NAME_WIDTH: f32 = 80.0;

/// Render the content of a session row (status dot + name + tool)
///
/// Note: The outer wrapper with hover handler is created in mod.rs since
/// it needs access to `cx.listener()` which is tied to HudView.
pub fn render_row_content(
    state: SessionState,
    session_name: &str,
    running_tools: &[RunningTool],
    tool_index: usize,
    fade_progress: f32,
    marquee_offset: f32,
    is_scrolling: bool,
) -> Div {
    let state_color = state_to_color(state);

    // Use unicode-width for accurate width calculation (CJK = 2 units, ASCII = 1 unit)
    let display_width = session_name.width();
    let estimated_text_width = display_width as f32 * MARQUEE_CHAR_WIDTH;
    let needs_marquee = estimated_text_width > SESSION_NAME_WIDTH;

    div()
        .w_full()
        .h(px(ROW_HEIGHT))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .px(px(8.0))
        .rounded(px(6.0))
        // Status dot column (fixed width)
        .child(
            div()
                .flex_shrink_0()
                .w(px(STATUS_DOT_WIDTH))
                .flex()
                .items_center()
                .justify_center()
                .child(div().size(px(10.0)).rounded_full().bg(state_color)),
        )
        // Session name column (fixed width with seamless marquee)
        .child(
            div()
                .flex_shrink_0()
                .w(px(SESSION_NAME_WIDTH))
                .h_full()
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .flex()
                        .items_center()
                        .ml(px(marquee_offset))
                        .font_family("Maple Mono NF CN")
                        .text_size(px(13.0))
                        .text_color(gpui::rgb(0xFFFFFF))
                        .whitespace_nowrap()
                        .child(if needs_marquee && is_scrolling {
                            // Two copies for seamless loop when scrolling
                            format!("{}    {}", session_name, session_name)
                        } else {
                            session_name.to_string()
                        }),
                ),
        )
        // Current tool (flex-1, takes remaining space)
        .child(render_current_tool(running_tools, tool_index, fade_progress))
}

/// Render current tool with cross-fade animation
/// Shows one tool at a time, cycling through the list
pub fn render_current_tool(
    tools: &[RunningTool],
    tool_index: usize,
    fade_progress: f32,
) -> Div {
    if tools.is_empty() {
        return div().flex_1();
    }

    // Get current and next tool indices
    let current_idx = tool_index % tools.len();
    let next_idx = (tool_index + 1) % tools.len();
    let current_tool = &tools[current_idx].tool_name;
    let next_tool = &tools[next_idx].tool_name;

    // Apply easing to fade progress
    let progress = ease_in_out(fade_progress);
    let current_opacity = 1.0 - progress; // fades out
    let next_opacity = progress; // fades in

    // Slide animation offsets
    let slide_distance = ROW_HEIGHT * 0.6;
    let current_y_offset = -progress * slide_distance; // slides up
    let next_y_offset = (1.0 - progress) * slide_distance; // slides down from above

    // Stack both tools with cross-fade opacity using a relative container
    div()
        .flex_1()
        .h_full()
        .relative()
        .overflow_hidden()
        .text_size(px(12.0))
        // Current tool (fading out, sliding up)
        .child(
            div()
                .absolute()
                .inset_0()
                .top(px(current_y_offset))
                .flex()
                .items_center()
                .child(render_tool_with_icon(current_tool, current_opacity * 0.8)),
        )
        // Next tool (fading in, sliding down from above)
        .child(
            div()
                .absolute()
                .inset_0()
                .top(px(next_y_offset))
                .flex()
                .items_center()
                .child(render_tool_with_icon(next_tool, next_opacity * 0.8)),
        )
}

/// Render a tool with its Lucide icon
pub fn render_tool_with_icon(tool_name: &str, opacity: f32) -> Div {
    let icon_path = icons::tool_icon_path(tool_name);
    let color = rgba_with_alpha(opacity);

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(svg().path(icon_path).size(px(14.0)).text_color(color))
        .child(div().text_color(color).child(tool_name.to_string()))
}

/// Create RGBA color with specified alpha (0.0 to 1.0)
fn rgba_with_alpha(alpha: f32) -> gpui::Rgba {
    gpui::rgba((0xFFFFFF00u32) | ((alpha * 255.0) as u32))
}

/// Extract session name from cwd (last folder name)
pub fn extract_session_name(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("session")
        .to_string()
}

/// Convert SessionState to color
pub fn state_to_color(state: SessionState) -> gpui::Hsla {
    match state {
        SessionState::Running => icons::colors::GREEN,
        SessionState::Idle => icons::colors::BLUE,
        SessionState::Attention => icons::colors::YELLOW,
        SessionState::Compacting => icons::colors::PURPLE,
        SessionState::Stale => icons::colors::GRAY,
    }
}

/// Calculate marquee offset during reset animation
pub fn calculate_reset_offset(from_offset: f32, elapsed_ms: u128) -> f32 {
    let progress = (elapsed_ms as f32 / RESET_DURATION_MS as f32).min(1.0);
    let eased = ease_out(progress);
    from_offset * (1.0 - eased)
}

/// Calculate expanded window height based on session count
pub fn calculate_expanded_height(session_count: usize) -> f32 {
    let count = session_count.min(MAX_SESSIONS);
    (ROW_HEIGHT + ROW_GAP) * count as f32 + 12.0
}
