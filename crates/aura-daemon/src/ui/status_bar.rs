//! Collapsed status bar view - two icons showing aggregate state
//!
//! The status bar shows:
//! - Left icon: attention indicator (bell if any session needs attention, check otherwise)
//! - Right icon: aggregate state icon (Running > Compacting > Idle > Stale)

use super::icons;
use aura_common::{SessionInfo, SessionState};
use gpui::{div, px, svg, Div, ParentElement, Styled};

/// Status bar dimensions
pub const WIDTH: f32 = 60.0;
pub const HEIGHT: f32 = 28.0;
pub const ICON_SIZE: f32 = 16.0;

/// Render the collapsed status bar with two icons
pub fn render(sessions: &[SessionInfo]) -> Div {
    let has_attention = sessions.iter().any(|s| s.state == SessionState::Attention);
    let aggregate_state = aggregate_state(sessions);

    div()
        .size_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(8.0))
        .p(px(6.0))
        .rounded(px(8.0))
        .bg(gpui::rgba(0x000000BB))
        // Left icon: attention indicator
        .child(attention_icon(has_attention))
        // Right icon: aggregate state
        .child(state_icon(aggregate_state))
}

/// Get aggregate state from sessions (priority: Running > Compacting > Idle > Stale)
pub fn aggregate_state(sessions: &[SessionInfo]) -> SessionState {
    if sessions.iter().any(|s| s.state == SessionState::Running) {
        SessionState::Running
    } else if sessions.iter().any(|s| s.state == SessionState::Compacting) {
        SessionState::Compacting
    } else if sessions.iter().any(|s| s.state == SessionState::Idle) {
        SessionState::Idle
    } else {
        SessionState::Stale
    }
}

/// Render attention indicator icon (bell if attention needed, check otherwise)
pub fn attention_icon(has_attention: bool) -> Div {
    let (icon_path, color) = if has_attention {
        ("icons/bell.svg", icons::colors::YELLOW)
    } else {
        ("icons/check.svg", icons::colors::GREEN)
    };

    div()
        .size(px(ICON_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(icon_path).size(px(ICON_SIZE)).text_color(color))
}

/// Render aggregate state icon
pub fn state_icon(state: SessionState) -> Div {
    let (icon_path, color) = match state {
        SessionState::Running => ("icons/terminal.svg", icons::colors::GREEN),
        SessionState::Compacting => ("icons/settings.svg", icons::colors::PURPLE),
        SessionState::Idle => ("icons/file.svg", icons::colors::BLUE),
        SessionState::Attention => ("icons/bell.svg", icons::colors::YELLOW),
        SessionState::Stale => ("icons/file.svg", icons::colors::GRAY),
    };

    div()
        .size(px(ICON_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(icon_path).size(px(ICON_SIZE)).text_color(color))
}
