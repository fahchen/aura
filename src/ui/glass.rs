//! Liquid glass effect helpers for gpui
//!
//! Since gpui doesn't support CSS gradients or backdrop-filter, we add
//! subtle highlight/shadow lines to simulate the glass inset effect:
//!
//! - **Top highlight** - Thin bright line at top (simulates inset 0 1px glow)
//!
//! The main background colors are applied by the parent elements.
//! These helpers just add the subtle glass "shine" details.

use super::theme::ThemeColors;
use gpui::{Div, Styled, div, px};

/// Render glass highlight for container (just top edge glow)
///
/// Simulates the CSS inset box-shadow:
/// ```css
/// box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.3);
/// ```
pub fn render_container_highlight(border_radius: f32, theme: &ThemeColors) -> Div {
    // Top highlight line (simulates inset 0 1px glow)
    div()
        .absolute()
        .top_0()
        .left(px(border_radius / 2.0)) // Inset from rounded corners
        .right(px(border_radius / 2.0))
        .h(px(1.0))
        .bg(theme.glass_top_highlight)
}
