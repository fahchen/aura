//! Theme system for Aura HUD
//!
//! Provides 4 theme styles plus System (auto-detect):
//! - Liquid Dark: transparent glass on dark backgrounds
//! - Liquid Light: transparent glass on light backgrounds
//! - Solid Dark: opaque VS Code / OLED style with shadows
//! - Solid Light: clean minimal light with shadows

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
    /// Solid opaque dark theme (VS Code style)
    SolidDark,
    /// Solid opaque light theme (clean minimal)
    SolidLight,
}

impl ThemeStyle {
    /// Resolve the effective style based on preference and system appearance
    pub fn resolve(&self, system_is_dark: bool) -> ResolvedStyle {
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
            ThemeStyle::SolidDark => ResolvedStyle::SolidDark,
            ThemeStyle::SolidLight => ResolvedStyle::SolidLight,
        }
    }

    /// Create from config string representation.
    pub fn from_config_str(s: &str) -> Self {
        match s {
            "liquid-dark" => Self::LiquidDark,
            "liquid-light" => Self::LiquidLight,
            "solid-dark" => Self::SolidDark,
            "solid-light" => Self::SolidLight,
            _ => Self::System,
        }
    }

    /// Convert to config string representation.
    pub fn to_config_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::LiquidDark => "liquid-dark",
            Self::LiquidLight => "liquid-light",
            Self::SolidDark => "solid-dark",
            Self::SolidLight => "solid-light",
        }
    }

    /// Cycle to next theme style
    pub fn next(&self) -> Self {
        match self {
            ThemeStyle::System => ThemeStyle::LiquidDark,
            ThemeStyle::LiquidDark => ThemeStyle::LiquidLight,
            ThemeStyle::LiquidLight => ThemeStyle::SolidDark,
            ThemeStyle::SolidDark => ThemeStyle::SolidLight,
            ThemeStyle::SolidLight => ThemeStyle::System,
        }
    }
}

/// Resolved style after applying system detection
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResolvedStyle {
    LiquidDark,
    LiquidLight,
    SolidDark,
    SolidLight,
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
/// Solid themes use opaque backgrounds with shadows for depth.
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

    // === Shadow ===
    /// Whether to use shadow (solid themes only)
    pub use_shadow: bool,
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
    // Shadow
    use_shadow: bool,
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
        use_shadow: p.use_shadow,
    }
}

/// Transparent black — used for disabled glass effects in solid themes.
const TRANSPARENT: (f32, f32) = (0.0, 0.0);

impl ThemeColors {
    /// Liquid Dark theme - very transparent glass on dark backgrounds
    ///
    /// Uses translucent white backgrounds for glass effect.
    /// Strong top highlight (40%) for depth.
    pub fn liquid_dark() -> Self {
        build_theme(&Palette {
            // Text: white at varying alpha
            text_primary:    (1.0, 0.95),
            text_secondary:  (1.0, 0.70),
            text_header:     (1.0, 0.60),
            // Icons: white at varying alpha
            icon_state:      (1.0, 0.85),
            icon_tool:       (1.0, 0.60),
            // Backgrounds: translucent white (1-4%)
            container_bg:    (1.0, 0.03),
            content_bg:      (1.0, 0.02),
            row_bg:          (1.0, 0.01),
            row_hover_bg:    (1.0, 0.04),
            indicator_bg:    (1.0, 0.04),
            // Borders: subtle white (6-10%)
            border:          (1.0, 0.10),
            content_highlight: (1.0, 0.40), // strong top highlight
            // Glass effects: white highlights
            glass_top_highlight: (1.0, 0.50),
            gloss:           (1.0, 0.30),
            // Indicator
            indicator_icon:  (1.0, 0.95),
            indicator_border: (1.0, 0.10),
            // Shadow for liquid themes
            use_shadow: true,
        })
    }

    /// Liquid Light theme - very transparent glass on light backgrounds
    ///
    /// Uses translucent black backgrounds for glass effect.
    /// Strong white top highlight (55%) for depth.
    pub fn liquid_light() -> Self {
        build_theme(&Palette {
            // Text: solid dark grays
            text_primary:    (0.10, 1.0),  // #1A1A1A
            text_secondary:  (0.32, 1.0),  // #525252
            text_header:     (0.32, 1.0),  // #525252
            // Icons: solid dark grays
            icon_state:      (0.25, 1.0),  // #404040
            icon_tool:       (0.45, 1.0),  // #737373
            // Backgrounds: translucent black (0.5-3%)
            container_bg:    (0.0, 0.02),
            content_bg:      (0.0, 0.015),
            row_bg:          (0.0, 0.005),
            row_hover_bg:    (0.0, 0.03),
            indicator_bg:    (0.0, 0.03),
            // Borders: subtle black (5-8%)
            border:          (0.0, 0.08),
            content_highlight: (1.0, 0.55), // strong white top highlight
            // Glass effects: white highlights for depth
            glass_top_highlight: (1.0, 0.70),
            gloss:           (1.0, 0.40),
            // Indicator
            indicator_icon:  (0.10, 1.0),  // #1A1A1A
            indicator_border: (0.0, 0.08),
            // Shadow for liquid themes
            use_shadow: true,
        })
    }

    /// Solid Dark theme - VS Code / OLED style with shadows
    ///
    /// Uses opaque backgrounds with box-shadow for depth.
    /// No glass effects.
    pub fn solid_dark() -> Self {
        build_theme(&Palette {
            // Text: white at varying alpha
            text_primary:    (1.0, 0.92),
            text_secondary:  (1.0, 0.65),
            text_header:     (1.0, 0.55),
            // Icons: white at varying alpha
            icon_state:      (1.0, 0.80),
            icon_tool:       (1.0, 0.55),
            // Backgrounds: opaque dark grays
            container_bg:    (0.118, 1.0), // #1E1E1E
            content_bg:      (0.145, 1.0), // #252526
            row_bg:          (0.176, 1.0), // #2D2D2D
            row_hover_bg:    (0.22, 1.0),  // #383838
            indicator_bg:    (0.176, 1.0), // #2D2D2D
            // Borders: opaque dark
            border:          (0.235, 1.0), // #3C3C3C
            content_highlight: (0.235, 1.0), // #3C3C3C (no glass, subtle)
            // Glass effects: transparent (no glass in solid themes)
            glass_top_highlight: TRANSPARENT,
            gloss:           TRANSPARENT,
            // Indicator
            indicator_icon:  (1.0, 0.92),
            indicator_border: (0.235, 1.0), // #3C3C3C
            // Shadow for solid themes
            use_shadow: true,
        })
    }

    /// Solid Light theme - clean minimal light with shadows
    ///
    /// Uses opaque light backgrounds with box-shadow for depth.
    /// No glass effects.
    pub fn solid_light() -> Self {
        build_theme(&Palette {
            // Text: solid dark colors
            text_primary:    (0.09, 1.0),  // #171717
            text_secondary:  (0.32, 1.0),  // #525252
            text_header:     (0.32, 1.0),  // #525252
            // Icons: solid dark
            icon_state:      (0.25, 1.0),  // #404040
            icon_tool:       (0.45, 1.0),  // #737373
            // Backgrounds: opaque light grays
            container_bg:    (0.96, 1.0),  // #F5F5F5
            content_bg:      (1.0, 1.0),   // #FFFFFF
            row_bg:          (0.98, 1.0),  // #FAFAFA
            row_hover_bg:    (0.94, 1.0),  // #F0F0F0
            indicator_bg:    (1.0, 1.0),   // #FFFFFF
            // Borders: opaque light
            border:          (0.90, 1.0),  // #E5E5E5
            content_highlight: (0.90, 1.0), // #E5E5E5 (no glass, subtle)
            // Glass effects: transparent (no glass in solid themes)
            glass_top_highlight: TRANSPARENT,
            gloss:           TRANSPARENT,
            // Indicator
            indicator_icon:  (0.09, 1.0),  // #171717
            indicator_border: (0.90, 1.0), // #E5E5E5
            // Shadow for solid themes
            use_shadow: true,
        })
    }

    /// Get theme colors for a resolved style
    pub fn for_style(style: ResolvedStyle) -> Self {
        match style {
            ResolvedStyle::LiquidDark => Self::liquid_dark(),
            ResolvedStyle::LiquidLight => Self::liquid_light(),
            ResolvedStyle::SolidDark => Self::solid_dark(),
            ResolvedStyle::SolidLight => Self::solid_light(),
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
            ThemeStyle::SolidDark,
            ThemeStyle::SolidLight,
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

