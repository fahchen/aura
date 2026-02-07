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
    /// Muted text (placeholders, hints)
    pub text_muted: Hsla,
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
    /// Subtle border
    pub border_subtle: Hsla,
    /// Content top highlight
    pub content_highlight: Hsla,

    // === Glass Effect Colors ===
    /// Top highlight for glass effect
    pub glass_top_highlight: Hsla,
    /// Row highlight for depth
    pub row_highlight: Hsla,
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
    /// Shadow opacity (0.0 for liquid, 0.12-0.4 for solid)
    pub shadow_opacity: f32,
}

impl ThemeColors {
    /// Liquid Dark theme - very transparent glass on dark backgrounds
    ///
    /// Uses translucent white backgrounds for glass effect.
    /// Strong top highlight (40%) for depth.
    pub fn liquid_dark() -> Self {
        Self {
            // Text: light colors for dark backgrounds
            text_primary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.95,
            }, // White 95%
            text_secondary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.70,
            }, // White 70%
            text_muted: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.50,
            }, // White 50%
            text_header: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.60,
            }, // White 60%

            // Icons: light colors for dark backgrounds
            icon_state: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.85,
            }, // White 85%
            icon_tool: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.60,
            }, // White 60%

            // Backgrounds: extremely transparent (1-4%) - increased see-through
            container_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.03,
            }, // 3%
            content_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.02,
            }, // 2%
            row_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.01,
            }, // 1%
            row_hover_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.04,
            }, // 4%
            indicator_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.04,
            }, // 4%

            // Borders: very subtle (6-10%)
            border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.10,
            }, // 10%
            border_subtle: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.06,
            }, // 6%
            content_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.40,
            }, // 40% - STRONG TOP

            // Glass effects
            glass_top_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.50,
            }, // 50%
            row_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.15,
            },
            gloss: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.30,
            }, // 30%

            // Indicator
            indicator_icon: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.95,
            }, // White 95%
            indicator_border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.10,
            }, // 10%

            // No shadow for liquid themes
            use_shadow: false,
            shadow_opacity: 0.0,
        }
    }

    /// Liquid Light theme - very transparent glass on light backgrounds
    ///
    /// Uses translucent black backgrounds for glass effect.
    /// Strong white top highlight (55%) for depth.
    pub fn liquid_light() -> Self {
        Self {
            // Text: solid dark grays
            text_primary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.10,
                a: 1.0,
            }, // #1A1A1A
            text_secondary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.32,
                a: 1.0,
            }, // #525252 (slate 600)
            text_muted: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.45,
                a: 1.0,
            }, // #737373 (slate 500)
            text_header: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.32,
                a: 1.0,
            }, // #525252

            // Icons: solid dark grays
            icon_state: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.25,
                a: 1.0,
            }, // #404040 (neutral 700)
            icon_tool: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.45,
                a: 1.0,
            }, // #737373

            // Backgrounds: extremely transparent black (1-3%) - increased see-through
            container_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.02,
            }, // 2%
            content_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.015,
            }, // 1.5%
            row_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.005,
            }, // 0.5%
            row_hover_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.03,
            }, // 3%
            indicator_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.03,
            }, // 3%

            // Borders: very subtle black (5-8%)
            border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.08,
            }, // 8%
            border_subtle: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.05,
            }, // 5%
            content_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.55,
            }, // 55% - STRONG TOP (white)

            // Glass effects: white highlights for depth
            glass_top_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.70,
            }, // 70%
            row_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.20,
            },
            gloss: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.40,
            }, // 40%

            // Indicator
            indicator_icon: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.10,
                a: 1.0,
            }, // #1A1A1A
            indicator_border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.08,
            }, // 8%

            // No shadow for liquid themes
            use_shadow: false,
            shadow_opacity: 0.0,
        }
    }

    /// Solid Dark theme - VS Code / OLED style with shadows
    ///
    /// Uses opaque backgrounds with box-shadow for depth.
    /// No glass effects.
    pub fn solid_dark() -> Self {
        Self {
            // Text: light colors for dark backgrounds
            text_primary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.92,
            }, // White 92%
            text_secondary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.65,
            }, // White 65%
            text_muted: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.45,
            }, // White 45%
            text_header: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.55,
            }, // White 55%

            // Icons
            icon_state: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.80,
            }, // White 80%
            icon_tool: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.55,
            }, // White 55%

            // Backgrounds: opaque dark grays
            container_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.118,
                a: 1.0,
            }, // #1E1E1E
            content_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.145,
                a: 1.0,
            }, // #252526
            row_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.176,
                a: 1.0,
            }, // #2D2D2D
            row_hover_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.22,
                a: 1.0,
            }, // #383838
            indicator_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.176,
                a: 1.0,
            }, // #2D2D2D

            // Borders: opaque dark
            border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.235,
                a: 1.0,
            }, // #3C3C3C
            border_subtle: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.2,
                a: 1.0,
            }, // #333333
            content_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.235,
                a: 1.0,
            }, // #3C3C3C (no glass effect, subtle)

            // Glass effects: transparent (no glass in solid themes)
            glass_top_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.0,
            }, // transparent
            row_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.0,
            },
            gloss: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.0,
            }, // transparent

            // Indicator
            indicator_icon: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.92,
            }, // White 92%
            indicator_border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.235,
                a: 1.0,
            }, // #3C3C3C

            // Shadow for solid themes
            use_shadow: true,
            shadow_opacity: 0.4,
        }
    }

    /// Solid Light theme - clean minimal light with shadows
    ///
    /// Uses opaque light backgrounds with box-shadow for depth.
    /// No glass effects.
    pub fn solid_light() -> Self {
        Self {
            // Text: solid dark colors
            text_primary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.09,
                a: 1.0,
            }, // #171717
            text_secondary: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.32,
                a: 1.0,
            }, // #525252
            text_muted: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.45,
                a: 1.0,
            }, // #737373
            text_header: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.32,
                a: 1.0,
            }, // #525252

            // Icons
            icon_state: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.25,
                a: 1.0,
            }, // #404040
            icon_tool: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.45,
                a: 1.0,
            }, // #737373

            // Backgrounds: opaque light grays
            container_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.96,
                a: 1.0,
            }, // #F5F5F5
            content_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 1.0,
            }, // #FFFFFF
            row_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.98,
                a: 1.0,
            }, // #FAFAFA
            row_hover_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.94,
                a: 1.0,
            }, // #F0F0F0
            indicator_bg: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 1.0,
            }, // #FFFFFF

            // Borders: opaque light
            border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.90,
                a: 1.0,
            }, // #E5E5E5
            border_subtle: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.92,
                a: 1.0,
            }, // #EBEBEB
            content_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.90,
                a: 1.0,
            }, // #E5E5E5 (no glass effect, subtle)

            // Glass effects: transparent (no glass in solid themes)
            glass_top_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.0,
            }, // transparent
            row_highlight: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.0,
            },
            gloss: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.0,
            }, // transparent

            // Indicator
            indicator_icon: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.09,
                a: 1.0,
            }, // #171717
            indicator_border: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.90,
                a: 1.0,
            }, // #E5E5E5

            // Shadow for solid themes
            use_shadow: true,
            shadow_opacity: 0.12,
        }
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

