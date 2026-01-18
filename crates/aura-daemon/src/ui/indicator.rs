//! Collapsed indicator view - single centered icon showing aggregate state
//!
//! The indicator shows a liquid glass square with Lucide SVG icons:
//! - Attention: bell_ring (bright icon, shaking animation)
//! - Running (sessions exist): cycles through 11 creative icons every 2500ms
//! - No sessions: panda (dim, static)
//!
//! Visual design:
//! - Liquid glass: translucent background with border
//! - Gloss overlay: top half for depth
//! - Icon: themed color (white for dark, black for light)

use super::animation::{calculate_shake_offset, ease_out};
use super::icons;
use super::theme::ThemeColors;
use aura_common::{SessionInfo, SessionState};
use gpui::{div, prelude::FluentBuilder, px, radians, svg, Div, Hsla, ParentElement, Styled, Transformation};
use std::time::Instant;

/// Indicator dimensions (matching React prototype: 36x36px rounded square)
const INDICATOR_SIZE: f32 = 36.0;
const INDICATOR_BORDER_RADIUS: f32 = 12.0;

/// Indicator dimensions (exported for window sizing)
pub const WIDTH: f32 = INDICATOR_SIZE;
pub const HEIGHT: f32 = INDICATOR_SIZE;

/// Icon font size within the indicator (16px per prototype)
const ICON_FONT_SIZE: f32 = 16.0;

/// Icon cycle interval in milliseconds (matches prototype: 2500ms)
const ICON_CYCLE_MS: u64 = 2500;

/// Icon transition duration in milliseconds (slide animation)
const ICON_TRANSITION_MS: u64 = 400;

/// SVG asset paths for static states
const ICON_ATTENTION: &str = "icons/bell-ring.svg";
const ICON_WAITING: &str = "icons/fan.svg";
const ICON_NO_SESSIONS: &str = "icons/panda.svg";

/// Indicator state
#[derive(Clone, Copy, PartialEq, Eq)]
enum IndicatorState {
    /// Any session needs attention - shaking circle with bell icon
    Attention,
    /// Any session waiting for user input
    Waiting,
    /// Sessions exist, no attention needed - cycling creative icons
    Running,
    /// No sessions - low opacity circle with sleep icon
    NoSessions,
}

/// Determine the current indicator state from sessions
fn determine_state(sessions: &[SessionInfo]) -> IndicatorState {
    if sessions.is_empty() {
        IndicatorState::NoSessions
    } else if sessions.iter().any(|s| s.state == SessionState::Attention) {
        IndicatorState::Attention
    } else if sessions.iter().any(|s| s.state == SessionState::Waiting) {
        IndicatorState::Waiting
    } else {
        IndicatorState::Running
    }
}

/// Get icon state for running animation - returns (current_icon, prev_icon, transition_progress)
/// transition_progress: 0.0-1.0 during first 400ms of cycle, 1.0 after transition complete
fn get_running_icon_state(animation_start: Instant) -> (&'static str, &'static str, f32) {
    let elapsed_ms = animation_start.elapsed().as_millis() as u64;
    let icon_count = icons::INDICATOR_RUNNING_ASSETS.len();

    let cycle = (elapsed_ms / ICON_CYCLE_MS) as usize;
    let pos_in_cycle = elapsed_ms % ICON_CYCLE_MS;

    let current_idx = cycle % icon_count;
    let prev_idx = if cycle == 0 {
        icon_count - 1
    } else {
        (cycle - 1) % icon_count
    };

    let current_icon = icons::INDICATOR_RUNNING_ASSETS[current_idx];
    let prev_icon = icons::INDICATOR_RUNNING_ASSETS[prev_idx];

    let transition_progress = if pos_in_cycle < ICON_TRANSITION_MS {
        pos_in_cycle as f32 / ICON_TRANSITION_MS as f32
    } else {
        1.0
    };

    (current_icon, prev_icon, transition_progress)
}

/// Render the indicator with liquid glass background and Lucide SVG icon
///
/// When `is_hovered` is true, applies enhanced visual effect:
/// - Increased background opacity
/// - Brighter gloss highlight
pub fn render(sessions: &[SessionInfo], animation_start: Instant, is_hovered: bool, theme: &ThemeColors) -> Div {
    let state = determine_state(sessions);

    // Get running icon state (may include transition)
    let running_state = if state == IndicatorState::Running {
        Some(get_running_icon_state(animation_start))
    } else {
        None
    };

    // Select SVG icon path and opacity values based on state
    // Icon colors: theme-based with varying alpha (Attention 0.95, Running 1.0, NoSessions 0.5)
    // Background alpha: averaged from CSS gradient (0.15/0.05/0.1 -> ~0.10)
    // On hover: enhance background +0.05, gloss +0.10 for visual feedback
    let hover_bg_boost = if is_hovered { 0.05 } else { 0.0 };
    let hover_gloss_boost = if is_hovered { 0.10 } else { 0.0 };

    let (icon_path, bg_alpha_boost, icon_alpha, gloss_alpha_boost) = match state {
        IndicatorState::Attention => (ICON_ATTENTION, 0.02 + hover_bg_boost, 0.95, 0.01 + hover_gloss_boost),
        IndicatorState::Waiting => (ICON_WAITING, hover_bg_boost, 0.9, hover_gloss_boost),
        IndicatorState::Running => {
            let (current, _, _) = running_state.unwrap();
            (current, hover_bg_boost, 1.0, hover_gloss_boost)
        }
        IndicatorState::NoSessions => (ICON_NO_SESSIONS, -0.02 + hover_bg_boost, 0.5, -0.02 + hover_gloss_boost),
    };

    // Calculate shake offset for attention state
    let shake_offset = if state == IndicatorState::Attention {
        calculate_shake_offset(animation_start)
    } else {
        0.0
    };

    // Calculate rotation for waiting state (2 second full rotation, counter-clockwise)
    let rotation_radians = if state == IndicatorState::Waiting {
        let elapsed_ms = animation_start.elapsed().as_millis() as f32;
        let rotation_period_ms = 2000.0; // 2 seconds per full rotation
        -((elapsed_ms / rotation_period_ms) * std::f32::consts::TAU) // Negative for counter-clockwise
    } else {
        0.0
    };

    // Use theme colors with state-based adjustments
    let circle_bg_color = Hsla {
        a: (theme.indicator_bg.a + bg_alpha_boost).clamp(0.0, 1.0),
        ..theme.indicator_bg
    };
    let border_color = theme.indicator_border;
    // Icon color from theme with state-based alpha
    let icon_color = Hsla {
        a: theme.indicator_icon.a * icon_alpha,
        ..theme.indicator_icon
    };
    // Gloss color from theme with state adjustments
    let gloss_color = Hsla {
        a: (theme.gloss.a + gloss_alpha_boost).clamp(0.0, 1.0),
        ..theme.gloss
    };

    // Rounded square indicator (36x36px with 12px border-radius per prototype)
    div()
        .w(px(INDICATOR_SIZE))
        .h(px(INDICATOR_SIZE))
        .rounded(px(INDICATOR_BORDER_RADIUS))
        .bg(circle_bg_color)
        .border_1()
        .border_color(border_color)
        .when(theme.use_shadow, |this| this.shadow_md())
        .relative()
        .overflow_hidden()
        // Gloss highlight (top half) - use explicit size, not w_full
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w(px(INDICATOR_SIZE))
                .h(px(INDICATOR_SIZE / 2.0))
                .bg(gloss_color),
        )
        // SVG Icon (centered, with shake animation for attention, slide for running)
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w(px(INDICATOR_SIZE))
                .h(px(INDICATOR_SIZE))
                .flex()
                .items_center()
                .justify_center()
                .ml(px(shake_offset)) // Apply horizontal shake for attention
                .child(
                    if let Some((current_icon, prev_icon, transition_progress)) = running_state {
                        if transition_progress < 1.0 {
                            // During transition: show both icons sliding
                            let eased = ease_out(transition_progress);
                            let prev_offset = -eased * ICON_FONT_SIZE;
                            let current_offset = (1.0 - eased) * ICON_FONT_SIZE;

                            {
                                let prev_alpha = icon_color.a * (1.0 - eased);
                                let curr_alpha = icon_color.a * eased;

                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .relative()
                                    .w(px(ICON_FONT_SIZE))
                                    .h(px(ICON_FONT_SIZE))
                                    .overflow_hidden()
                                    // Previous icon sliding out
                                    .child(
                                        div()
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .w(px(ICON_FONT_SIZE))
                                            .h(px(ICON_FONT_SIZE))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .ml(px(prev_offset))
                                            .child(
                                                svg()
                                                    .path(prev_icon)
                                                    .size(px(ICON_FONT_SIZE))
                                                    .text_color(Hsla {
                                                        a: prev_alpha,
                                                        ..icon_color
                                                    }),
                                            ),
                                    )
                                    // Current icon sliding in
                                    .child(
                                        div()
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .w(px(ICON_FONT_SIZE))
                                            .h(px(ICON_FONT_SIZE))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .ml(px(current_offset))
                                            .child(
                                                svg()
                                                    .path(current_icon)
                                                    .size(px(ICON_FONT_SIZE))
                                                    .text_color(Hsla {
                                                        a: curr_alpha,
                                                        ..icon_color
                                                    }),
                                            ),
                                    )
                            }
                        } else {
                            // No transition: single icon
                            div()
                                .w(px(ICON_FONT_SIZE))
                                .h(px(ICON_FONT_SIZE))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    svg()
                                        .path(icon_path)
                                        .size(px(ICON_FONT_SIZE))
                                        .text_color(icon_color),
                                )
                        }
                    } else {
                        // Non-running states: single icon (with rotation for Waiting)
                        let icon_svg = svg()
                            .path(icon_path)
                            .size(px(ICON_FONT_SIZE))
                            .text_color(icon_color);

                        div()
                            .w(px(ICON_FONT_SIZE))
                            .h(px(ICON_FONT_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(if rotation_radians != 0.0 {
                                icon_svg.with_transformation(Transformation::rotate(radians(rotation_radians)))
                            } else {
                                icon_svg
                            })
                    },
                ),
        )
}
