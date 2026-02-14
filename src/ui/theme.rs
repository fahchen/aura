//! Theme system for Aura HUD
//!
//! Provides 2 liquid glass theme styles plus System (auto-detect):
//! - Liquid Dark: transparent glass on dark backgrounds
//! - Liquid Light: transparent glass on light backgrounds

use gpui::{Hsla, WindowAppearance};

/// Theme style preference
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ThemeStyle {
    /// Follow system appearance (default) - uses Liquid variant
    #[default]
    System,
    /// Liquid glass dark theme
    LiquidDark,
    /// Liquid glass light theme
    LiquidLight,
}

impl ThemeStyle {
    /// Resolve the effective style based on preference and system appearance
    pub fn resolve(self, system_is_dark: bool) -> ResolvedStyle {
        match self {
            ThemeStyle::System => {
                if system_is_dark {
                    ResolvedStyle::LiquidDark
                } else {
                    ResolvedStyle::LiquidLight
                }
            }
            ThemeStyle::LiquidDark => ResolvedStyle::LiquidDark,
            ThemeStyle::LiquidLight => ResolvedStyle::LiquidLight,
        }
    }

    /// Create from config string representation.
    pub fn from_config_str(s: &str) -> Self {
        match s {
            "liquid-dark" => Self::LiquidDark,
            "liquid-light" => Self::LiquidLight,
            _ => Self::System,
        }
    }

    /// Convert to config string representation.
    pub fn to_config_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::LiquidDark => "liquid-dark",
            Self::LiquidLight => "liquid-light",
        }
    }

    /// Cycle to next theme style
    pub fn next(self) -> Self {
        match self {
            ThemeStyle::System => ThemeStyle::LiquidDark,
            ThemeStyle::LiquidDark => ThemeStyle::LiquidLight,
            ThemeStyle::LiquidLight => ThemeStyle::System,
        }
    }
}

/// Resolved style after applying system detection
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResolvedStyle {
    LiquidDark,
    LiquidLight,
}

/// Check if the system appearance is dark
pub fn is_system_dark(appearance: WindowAppearance) -> bool {
    matches!(
        appearance,
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    )
}

/// Theme color palette
///
/// All colors are defined as HSLA for consistency with gpui.
/// Liquid themes use transparent backgrounds for glass effect.
#[derive(Clone, Copy)]
pub struct ThemeColors {
    // === Text Colors ===
    /// Primary text (session names, important labels)
    pub text_primary: Hsla,
    /// Secondary text (tool labels, descriptions)
    pub text_secondary: Hsla,
    /// Header text (session count)
    pub text_header: Hsla,

    // === Icon Colors ===
    /// State icon color (running, idle, etc.)
    pub icon_state: Hsla,
    /// Tool icon color
    pub icon_tool: Hsla,

    // === Background Colors ===
    /// Container background (outer window)
    pub container_bg: Hsla,
    /// Content area background
    pub content_bg: Hsla,
    /// Session row background
    pub row_bg: Hsla,
    /// Session row hover background
    pub row_hover_bg: Hsla,
    /// Indicator background
    pub indicator_bg: Hsla,

    // === Border Colors ===
    /// Standard border
    pub border: Hsla,
    /// Content top highlight
    pub content_highlight: Hsla,

    // === Glass Effect Colors ===
    /// Top highlight for glass effect
    pub glass_top_highlight: Hsla,
    /// Gloss overlay
    pub gloss: Hsla,

    // === Indicator-specific ===
    /// Indicator icon color (white for dark, dark for light)
    pub indicator_icon: Hsla,
    /// Indicator border
    pub indicator_border: Hsla,
}

/// All theme colors are achromatic (h=0, s=0), so each color is fully
/// described by a (lightness, alpha) pair.
struct Palette {
    // Text
    text_primary: (f32, f32),
    text_secondary: (f32, f32),
    text_header: (f32, f32),
    // Icons
    icon_state: (f32, f32),
    icon_tool: (f32, f32),
    // Backgrounds
    container_bg: (f32, f32),
    content_bg: (f32, f32),
    row_bg: (f32, f32),
    row_hover_bg: (f32, f32),
    indicator_bg: (f32, f32),
    // Borders
    border: (f32, f32),
    content_highlight: (f32, f32),
    // Glass effects
    glass_top_highlight: (f32, f32),
    gloss: (f32, f32),
    // Indicator
    indicator_icon: (f32, f32),
    indicator_border: (f32, f32),
}

/// Construct an achromatic Hsla from (lightness, alpha).
const fn gray(l: f32, a: f32) -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l,
        a,
    }
}

fn build_theme(p: &Palette) -> ThemeColors {
    ThemeColors {
        text_primary: gray(p.text_primary.0, p.text_primary.1),
        text_secondary: gray(p.text_secondary.0, p.text_secondary.1),
        text_header: gray(p.text_header.0, p.text_header.1),
        icon_state: gray(p.icon_state.0, p.icon_state.1),
        icon_tool: gray(p.icon_tool.0, p.icon_tool.1),
        container_bg: gray(p.container_bg.0, p.container_bg.1),
        content_bg: gray(p.content_bg.0, p.content_bg.1),
        row_bg: gray(p.row_bg.0, p.row_bg.1),
        row_hover_bg: gray(p.row_hover_bg.0, p.row_hover_bg.1),
        indicator_bg: gray(p.indicator_bg.0, p.indicator_bg.1),
        border: gray(p.border.0, p.border.1),
        content_highlight: gray(p.content_highlight.0, p.content_highlight.1),
        glass_top_highlight: gray(p.glass_top_highlight.0, p.glass_top_highlight.1),
        gloss: gray(p.gloss.0, p.gloss.1),
        indicator_icon: gray(p.indicator_icon.0, p.indicator_icon.1),
        indicator_border: gray(p.indicator_border.0, p.indicator_border.1),
    }
}

impl ThemeColors {
    /// Liquid Dark theme - very transparent glass on dark backgrounds
    ///
    /// Uses translucent white backgrounds for glass effect.
    /// Strong top highlight (40%) for depth.
    pub fn liquid_dark() -> Self {
        build_theme(&Palette {
            // Text: white at varying alpha
            text_primary: (1.0, 0.95),
            text_secondary: (1.0, 0.70),
            text_header: (1.0, 0.60),
            // Icons: white at varying alpha
            icon_state: (1.0, 0.85),
            icon_tool: (1.0, 0.60),
            // Backgrounds: translucent white (1-4%)
            container_bg: (1.0, 0.03),
            content_bg: (1.0, 0.02),
            row_bg: (1.0, 0.01),
            row_hover_bg: (1.0, 0.04),
            indicator_bg: (1.0, 0.04),
            // Borders: subtle white (6-10%)
            border: (1.0, 0.10),
            content_highlight: (1.0, 0.40), // strong top highlight
            // Glass effects: white highlights
            glass_top_highlight: (1.0, 0.50),
            gloss: (1.0, 0.30),
            // Indicator
            indicator_icon: (1.0, 0.95),
            indicator_border: (1.0, 0.10),
        })
    }

    /// Liquid Light theme - very transparent glass on light backgrounds
    ///
    /// Uses translucent black backgrounds for glass effect.
    /// Strong white top highlight (55%) for depth.
    pub fn liquid_light() -> Self {
        build_theme(&Palette {
            // Text: solid dark grays
            text_primary: (0.10, 1.0),   // #1A1A1A
            text_secondary: (0.32, 1.0), // #525252
            text_header: (0.32, 1.0),    // #525252
            // Icons: solid dark grays
            icon_state: (0.25, 1.0), // #404040
            icon_tool: (0.45, 1.0),  // #737373
            // Backgrounds: translucent black (0.5-3%)
            container_bg: (0.0, 0.02),
            content_bg: (0.0, 0.015),
            row_bg: (0.0, 0.005),
            row_hover_bg: (0.0, 0.03),
            indicator_bg: (0.0, 0.03),
            // Borders: subtle black (5-8%)
            border: (0.0, 0.08),
            content_highlight: (1.0, 0.55), // strong white top highlight
            // Glass effects: white highlights for depth
            glass_top_highlight: (1.0, 0.70),
            gloss: (1.0, 0.40),
            // Indicator
            indicator_icon: (0.10, 1.0), // #1A1A1A
            indicator_border: (0.0, 0.08),
        })
    }

    /// Get theme colors for a resolved style
    pub fn for_style(style: ResolvedStyle) -> Self {
        match style {
            ResolvedStyle::LiquidDark => Self::liquid_dark(),
            ResolvedStyle::LiquidLight => Self::liquid_light(),
        }
    }
}

// === Layout Constants (theme-independent) ===

/// macOS window corner radius
pub const WINDOW_RADIUS: f32 = 16.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_str_roundtrip() {
        for style in [
            ThemeStyle::System,
            ThemeStyle::LiquidDark,
            ThemeStyle::LiquidLight,
        ] {
            let s = style.to_config_str();
            let back = ThemeStyle::from_config_str(s);
            assert_eq!(back, style);
        }
    }

    #[test]
    fn config_str_unknown_defaults_to_system() {
        assert_eq!(ThemeStyle::from_config_str("unknown"), ThemeStyle::System);
        assert_eq!(ThemeStyle::from_config_str(""), ThemeStyle::System);
    }
}
