//! Collapsed indicator view - single centered icon showing aggregate state
//!
//! The indicator shows a dark circle with a light cream icon:
//! - Attention: bell_ring (bright icon, shaking animation)
//! - Normal (sessions exist): robot (medium brightness, static)
//! - No sessions: sleep (dim, static)
//!
//! Visual design:
//! - Dark semi-transparent circle background (high contrast)
//! - Gloss overlay: top half of circle for depth
//! - Icon: Nerd Font glyph in warm cream color

use super::animation::calculate_shake_offset;
use super::icons;
use aura_common::{SessionInfo, SessionState};
use gpui::{div, px, Div, ParentElement, SharedString, Styled};
use std::time::Instant;

/// Circle size for the indicator
const CIRCLE_SIZE: f32 = 28.0;

/// Indicator dimensions (matches circle size exactly)
pub const WIDTH: f32 = CIRCLE_SIZE;
pub const HEIGHT: f32 = CIRCLE_SIZE;

/// Icon font size within the circle (even number for proper centering)
const ICON_FONT_SIZE: f32 = 12.0;

/// Nerd Font glyphs
const GLYPH_BELL_RING: &str = "\u{f009e}"; // 󰂞
const GLYPH_ROBOT: &str = "\u{f06a9}"; // 󰚩
const GLYPH_SLEEP: &str = "\u{f04b2}"; // 󰒲

/// Indicator state
#[derive(Clone, Copy, PartialEq, Eq)]
enum IndicatorState {
    /// Any session needs attention - shaking circle
    Attention,
    /// Sessions exist, no attention needed - static circle
    Normal,
    /// No sessions - low opacity circle
    NoSessions,
}

/// Determine the current indicator state from sessions
fn determine_state(sessions: &[SessionInfo]) -> IndicatorState {
    if sessions.is_empty() {
        IndicatorState::NoSessions
    } else if sessions.iter().any(|s| s.state == SessionState::Attention) {
        IndicatorState::Attention
    } else {
        IndicatorState::Normal
    }
}

/// Render the indicator with dark circle background and light cream icon
pub fn render(sessions: &[SessionInfo], animation_start: Instant) -> Div {
    let state = determine_state(sessions);

    // Select glyph and opacity values based on state
    // Dark circle background, light cream icon for high contrast
    let (glyph, circle_bg_alpha, icon_alpha, gloss_alpha) = match state {
        IndicatorState::Attention => (GLYPH_BELL_RING, 0.75, 1.0, 0.15),
        IndicatorState::Normal => (GLYPH_ROBOT, 0.65, 0.95, 0.12),
        IndicatorState::NoSessions => (GLYPH_SLEEP, 0.55, 0.75, 0.08),
    };

    // Calculate shake offset for attention state
    let shake_offset = if state == IndicatorState::Attention {
        calculate_shake_offset(animation_start)
    } else {
        0.0
    };

    // Glass background (semi-transparent white)
    let circle_bg_color = gpui::Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0, // White
        a: circle_bg_alpha * 0.4, // More transparent for glass effect
    };
    // Dark icon for contrast on glass
    let icon_color = gpui::Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.15, // Dark gray
        a: icon_alpha,
    };
    // Subtle white gloss for depth
    let gloss_color = gpui::Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0, // White
        a: gloss_alpha,
    };

    // Circle indicator (no outer container to avoid border artifacts)
    div()
        .w(px(CIRCLE_SIZE))
        .h(px(CIRCLE_SIZE))
        .rounded_full()
        .bg(circle_bg_color)
        .relative()
        .overflow_hidden()
        // Gloss highlight (top half)
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h(px(CIRCLE_SIZE / 2.0))
                .bg(gloss_color),
        )
        // Icon (centered using explicit size + flex, with shake animation)
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .w(px(ICON_FONT_SIZE))
                        .h(px(ICON_FONT_SIZE))
                        .flex()
                        .items_center()
                        .justify_center()
                        .ml(px(shake_offset)) // Apply horizontal shake for attention
                        .font_family("Maple Mono NF CN")
                        .text_size(px(ICON_FONT_SIZE))
                        .text_color(icon_color)
                        .child(SharedString::from(glyph)),
                ),
        )
}
