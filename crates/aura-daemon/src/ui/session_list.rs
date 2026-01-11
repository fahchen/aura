//! Session list rendering - expanded view with session rows
//!
//! Each row displays:
//! - Status dot (colored by state)
//! - Session name (folder name from cwd, with marquee for long names)
//! - Current tool with cross-fade animation

use super::animation::{ease_in_out, ease_out, MARQUEE_CHAR_WIDTH, RESET_DURATION_MS};
use super::icons;
use aura_common::{RunningTool, SessionState};
use gpui::{div, px, Div, ParentElement, Styled};
use unicode_width::UnicodeWidthStr;

/// Session list dimensions
pub const WIDTH: f32 = 242.0; // STATUS_DOT + NAME + TOOL + gaps(16) + padding(16)
pub const ROW_HEIGHT: f32 = 32.0;
pub const ROW_GAP: f32 = 4.0;
pub const MAX_SESSIONS: usize = 5;

/// Column widths for table-style layout
pub const STATUS_DOT_WIDTH: f32 = 18.0;
pub const SESSION_NAME_WIDTH: f32 = 80.0;
pub const TOOL_COLUMN_WIDTH: f32 = 112.0; // ~√2× SESSION_NAME_WIDTH

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
                        .text_size(px(14.0))
                        .text_color(gpui::rgb(0x1a1a1a)) // Dark text for glass background
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
        return div().flex_shrink_0().w(px(TOOL_COLUMN_WIDTH));
    }

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
    let slide_distance = ROW_HEIGHT * 0.6;
    let current_y_offset = -progress * slide_distance; // slides up
    let next_y_offset = (1.0 - progress) * slide_distance; // slides down from above

    // Stack both tools with cross-fade opacity using a relative container
    div()
        .flex_shrink_0()
        .w(px(TOOL_COLUMN_WIDTH))
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
                .opacity(current_opacity)
                .child(render_tool_with_icon(current_tool)),
        )
        // Next tool (fading in, sliding down from above)
        .child(
            div()
                .absolute()
                .inset_0()
                .top(px(next_y_offset))
                .flex()
                .items_center()
                .opacity(next_opacity)
                .child(render_tool_with_icon(next_tool)),
        )
}

/// Icon width for consistent alignment (Nerd Font icons vary in width)
const TOOL_ICON_WIDTH: f32 = 16.0;
/// Max characters for tool label display (approximate fit for column width)
const TOOL_LABEL_MAX_CHARS: usize = 12;

/// Truncate string intelligently based on content type
/// - Paths: truncate prefix ("...filename.rs")
/// - Commands/text: truncate suffix ("cargo bui...")
fn truncate_label(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }

    // Path detection: contains path separator
    let is_path = s.contains('/') || s.contains('\\');

    if is_path {
        // For paths: truncate prefix, keep filename/end portion
        let chars: Vec<char> = s.chars().collect();
        let keep_chars = max_chars.saturating_sub(3); // Reserve 3 for "..."
        let start = char_count - keep_chars;
        format!("...{}", chars[start..].iter().collect::<String>())
    } else {
        // For commands/text: truncate suffix
        format!(
            "{}...",
            s.chars()
                .take(max_chars.saturating_sub(3))
                .collect::<String>()
        )
    }
}

/// Dimmed color for tool display
const TOOL_COLOR: u32 = 0x303030FF; // Dark gray

/// Render a tool with its Nerd Font icon
pub fn render_tool_with_icon(tool: &RunningTool) -> Div {
    let icon = icons::tool_nerd_icon(&tool.tool_name);
    let display_text = tool.tool_label.as_deref().unwrap_or(&tool.tool_name);
    let truncated = truncate_label(display_text, TOOL_LABEL_MAX_CHARS);

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(1.0))
        .max_w(px(TOOL_COLUMN_WIDTH))
        .overflow_hidden()
        .font_family("Maple Mono NF CN")
        .text_color(gpui::rgba(TOOL_COLOR))
        .child(
            div()
                .flex_shrink_0()
                .w(px(TOOL_ICON_WIDTH))
                .text_size(px(12.0))
                .child(icon),
        )
        .child(
            div()
                .text_size(px(12.0))
                .whitespace_nowrap()
                .child(truncated),
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
