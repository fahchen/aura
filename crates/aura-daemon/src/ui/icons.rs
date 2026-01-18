//! SVG icon paths and colors for HUD icons
//!
//! Icons from Lucide (https://lucide.dev), 24x24 viewBox
//! Rendered at 16x16px in the HUD using gpui's svg() element

// Note: gpui::Hsla is used in the colors module but not directly imported here
// to avoid unused import warnings while maintaining the type for color definitions

/// State icon SVG paths (Lucide icons, 24x24 viewBox)
/// These paths are stored for future SVG rendering migration
#[allow(dead_code)]
pub mod state_icons {
    /// Cctv - Running state (monitoring/active)
    pub const RUNNING: &[&str] = &[
        "M16.75 12h3.632a1 1 0 0 1 .894 1.447l-2.034 4.069a1 1 0 0 1-1.708.134l-2.124-2.97",
        "M17.106 9.053a1 1 0 0 1 .447 1.341l-3.106 6.211a1 1 0 0 1-1.342.447L3.61 12.3a2.92 2.92 0 0 1-1.3-3.91L3.69 5.6a2.92 2.92 0 0 1 3.92-1.3z",
        "M2 19h3.76a2 2 0 0 0 1.8-1.1L9 15",
        "M2 21v-4",
        "M7 9h.01",
    ];

    /// MessageSquareCode - Idle state (waiting for input)
    pub const IDLE: &[&str] = &[
        "M22 17a2 2 0 0 1-2 2H6.828a2 2 0 0 0-1.414.586l-2.202 2.202A.71.71 0 0 1 2 21.286V5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2z",
        "m10 8-3 3 3 3",
        "m14 14 3-3-3-3",
    ];

    /// BellRing - Attention state (needs permission)
    pub const ATTENTION: &[&str] = &[
        "M10.268 21a2 2 0 0 0 3.464 0",
        "M22 8c0-2.3-.8-4.3-2-6",
        "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326",
        "M4 2C2.8 3.7 2 5.7 2 8",
    ];

    /// Cookie - Compacting state (processing/munching)
    pub const COMPACTING: &[&str] = &[
        "M12 2a10 10 0 1 0 10 10 4 4 0 0 1-5-5 4 4 0 0 1-5-5",
        "M8.5 8.5v.01",
        "M16 15.5v.01",
        "M12 12v.01",
        "M11 17v.01",
        "M7 14v.01",
    ];

    /// Ghost - Stale state (abandoned/inactive)
    pub const STALE: &[&str] = &[
        "M9 10h.01",
        "M15 10h.01",
        "M12 2a8 8 0 0 0-8 8v12l3-3 2.5 2.5L12 19l2.5 2.5L17 19l3 3V10a8 8 0 0 0-8-8z",
    ];

    /// Wind - Waiting state (Lucide wind icon)
    pub const WAITING: &[&str] = &[
        "M17.7 7.7a2.5 2.5 0 1 1 1.8 4.3H2",
        "M9.6 4.6A2 2 0 1 1 11 8H2",
        "M12.6 19.4A2 2 0 1 0 14 16H2",
    ];
}

/// Tool icon SVG paths (Lucide icons, 24x24 viewBox)
/// These paths are stored for future SVG rendering migration
#[allow(dead_code)]
pub mod tool_icons {
    /// Bot - Task/Agent tool
    pub const BOT: &[&str] = &[
        "M12 8V4H8",
        "M2 14h2",
        "M20 14h2",
        "M15 13v2",
        "M9 13v2",
    ];
    pub const BOT_RECT: (f32, f32, f32, f32, f32) = (4.0, 8.0, 16.0, 12.0, 2.0); // x, y, w, h, rx

    /// Terminal - Bash tool
    pub const TERMINAL: &[&str] = &["m4 17 6-6-6-6", "M12 19h8"];

    /// BookSearch - Glob tool
    pub const BOOK_SEARCH: &[&str] = &[
        "M11 22H5.5a1 1 0 0 1 0-5h4.501",
        "m21 22-1.879-1.878",
        "M3 19.5v-15A2.5 2.5 0 0 1 5.5 2H18a1 1 0 0 1 1 1v8",
    ];
    pub const BOOK_SEARCH_CIRCLE: (f32, f32, f32) = (17.0, 18.0, 3.0); // cx, cy, r

    /// FileSearchCorner - Grep tool
    pub const FILE_SEARCH: &[&str] = &[
        "M11.1 22H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.706.706l3.589 3.588A2.4 2.4 0 0 1 20 8v3.25",
        "M14 2v5a1 1 0 0 0 1 1h5",
        "m21 22-2.88-2.88",
    ];
    pub const FILE_SEARCH_CIRCLE: (f32, f32, f32) = (16.0, 17.0, 3.0); // cx, cy, r

    /// Newspaper - Read tool
    pub const NEWSPAPER: &[&str] = &[
        "M15 18h-5",
        "M18 14h-8",
        "M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-4 0v-9a2 2 0 0 1 2-2h2",
    ];
    pub const NEWSPAPER_RECT: (f32, f32, f32, f32, f32) = (10.0, 6.0, 8.0, 4.0, 1.0); // x, y, w, h, rx

    /// FilePenLine - Edit tool
    pub const FILE_PEN: &[&str] = &[
        "m18.226 5.226-2.52-2.52A2.4 2.4 0 0 0 14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-.351",
        "M21.378 12.626a1 1 0 0 0-3.004-3.004l-4.01 4.012a2 2 0 0 0-.506.854l-.837 2.87a.5.5 0 0 0 .62.62l2.87-.837a2 2 0 0 0 .854-.506z",
        "M8 18h1",
    ];

    /// FileBracesCorner - Write tool
    pub const FILE_BRACES: &[&str] = &[
        "M14 22h4a2 2 0 0 0 2-2V8a2.4 2.4 0 0 0-.706-1.706l-3.588-3.588A2.4 2.4 0 0 0 14 2H6a2 2 0 0 0-2 2v6",
        "M14 2v5a1 1 0 0 0 1 1h5",
        "M5 14a1 1 0 0 0-1 1v2a1 1 0 0 1-1 1 1 1 0 0 1 1 1v2a1 1 0 0 0 1 1",
        "M9 22a1 1 0 0 0 1-1v-2a1 1 0 0 1 1-1 1 1 0 0 1-1-1v-2a1 1 0 0 0-1-1",
    ];

    /// MonitorDown - WebFetch tool
    pub const MONITOR_DOWN: &[&str] = &["M12 13V7", "m15 10-3 3-3-3", "M12 17v4", "M8 21h8"];
    pub const MONITOR_DOWN_RECT: (f32, f32, f32, f32, f32) = (2.0, 3.0, 20.0, 14.0, 2.0); // x, y, w, h, rx

    /// Binoculars - WebSearch tool
    pub const BINOCULARS: &[&str] = &[
        "M10 10h4",
        "M19 7V4a1 1 0 0 0-1-1h-2a1 1 0 0 0-1 1v3",
        "M20 21a2 2 0 0 0 2-2v-3.851c0-1.39-2-2.962-2-4.829V8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v11a2 2 0 0 0 2 2z",
        "M22 16H2",
        "M4 21a2 2 0 0 1-2-2v-3.851c0-1.39 2-2.962 2-4.829V8a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v11a2 2 0 0 1-2 2z",
        "M9 7V4a1 1 0 0 0-1-1H6a1 1 0 0 0-1 1v3",
    ];

    /// Plug - MCP tools
    pub const PLUG: &[&str] = &[
        "M12 22v-5",
        "M15 8V2",
        "M17 8a1 1 0 0 1 1 1v4a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V9a1 1 0 0 1 1-1z",
        "M9 8V2",
    ];

    /// Ticket - Default/other tools
    pub const TICKET: &[&str] = &[
        "M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z",
        "M13 5v2",
        "M13 17v2",
        "M13 11v2",
    ];
}

/// Indicator cycling icons (Lucide, 24x24 viewBox)
/// These paths are stored for future SVG rendering migration
#[allow(dead_code)]
pub mod indicator_icons {
    /// Panda - Idle/no sessions
    pub const PANDA: &[&str] = &[
        "M11.25 17.25h1.5L12 18z",
        "m15 12 2 2",
        "M18 6.5a.5.5 0 0 0-.5-.5",
        "M20.69 9.67a4.5 4.5 0 1 0-7.04-5.5 8.35 8.35 0 0 0-3.3 0 a 4.5 4.5 0 1 0-7.04 5.5C2.49 11.2 2 12.88 2 14.5 2 19.47 6.48 22 12 22s10-2.53 10-7.5c0-1.62-.48-3.3-1.3-4.83",
        "M6 6.5a.495.495 0 0 1 .5-.5",
        "m9 12-2 2",
    ];

    /// WandSparkles
    pub const WAND_SPARKLES: &[&str] = &[
        "m21.64 3.64-1.28-1.28a1.21 1.21 0 0 0-1.72 0L2.36 18.64a1.21 1.21 0 0 0 0 1.72l1.28 1.28a1.2 1.2 0 0 0 1.72 0L21.64 5.36a1.2 1.2 0 0 0 0-1.72",
        "m14 7 3 3",
        "M5 6v4",
        "M19 14v4",
        "M10 2v2",
        "M7 8H3",
        "M21 16h-4",
        "M11 3H9",
    ];

    /// Sparkles
    pub const SPARKLES: &[&str] = &[
        "M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594z",
        "M20 2v4",
        "M22 4h-4",
    ];
    pub const SPARKLES_CIRCLE: (f32, f32, f32) = (4.0, 20.0, 2.0); // cx, cy, r

    /// Flame
    pub const FLAME: &[&str] =
        &["M12 3q1 4 4 6.5t3 5.5a1 1 0 0 1-14 0 5 5 0 0 1 1-3 1 1 0 0 0 5 0c0-2-1.5-3-1.5-5q0-2 2.5-4"];

    /// Zap
    pub const ZAP: &[&str] = &["M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z"];

    /// Brain
    pub const BRAIN: &[&str] = &[
        "M12 18V5",
        "M15 13a4.17 4.17 0 0 1-3-4 4.17 4.17 0 0 1-3 4",
        "M17.598 6.5A3 3 0 1 0 12 5a3 3 0 1 0-5.598 1.5",
        "M17.997 5.125a4 4 0 0 1 2.526 5.77",
        "M18 18a4 4 0 0 0 2-7.464",
        "M19.967 17.483A4 4 0 1 1 12 18a4 4 0 1 1-7.967-.517",
        "M6 18a4 4 0 0 1-2-7.464",
        "M6.003 5.125a4 4 0 0 0-2.526 5.77",
    ];

    /// Spotlight
    pub const SPOTLIGHT: &[&str] = &[
        "M15.295 19.562 16 22",
        "m17 16 3.758 2.098",
        "m19 12.5 3.026-.598",
        "M7.61 6.3a3 3 0 0 0-3.92 1.3l-1.38 2.79a3 3 0 0 0 1.3 3.91l6.89 3.597a1 1 0 0 0 1.342-.447l3.106-6.211a1 1 0 0 0-.447-1.341z",
        "M8 9V2",
    ];

    /// BicepsFlexed
    pub const BICEPS_FLEXED: &[&str] = &[
        "M12.409 13.017A5 5 0 0 1 22 15c0 3.866-4 7-9 7-4.077 0-8.153-.82-10.371-2.462-.426-.316-.631-.832-.62-1.362C2.118 12.723 2.627 2 10 2a3 3 0 0 1 3 3 2 2 0 0 1-2 2c-1.105 0-1.64-.444-2-1",
        "M15 14a5 5 0 0 0-7.584 2",
        "M9.964 6.825C8.019 7.977 9.5 13 8 15",
    ];

    /// Rocket
    pub const ROCKET: &[&str] = &[
        "M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z",
        "m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z",
        "M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0",
        "M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5",
    ];

    /// Cpu
    pub const CPU: &[&str] = &[
        "M12 20v2",
        "M12 2v2",
        "M17 20v2",
        "M17 2v2",
        "M2 12h2",
        "M2 17h2",
        "M2 7h2",
        "M20 12h2",
        "M20 17h2",
        "M20 7h2",
        "M7 20v2",
        "M7 2v2",
    ];
    pub const CPU_OUTER_RECT: (f32, f32, f32, f32, f32) = (4.0, 4.0, 16.0, 16.0, 2.0);
    pub const CPU_INNER_RECT: (f32, f32, f32, f32, f32) = (8.0, 8.0, 8.0, 8.0, 1.0);

    /// Puzzle
    pub const PUZZLE: &[&str] = &[
        "M15.39 4.39a1 1 0 0 0 1.68-.474 2.5 2.5 0 1 1 3.014 3.015 1 1 0 0 0-.474 1.68l1.683 1.682a2.414 2.414 0 0 1 0 3.414L19.61 15.39a1 1 0 0 1-1.68-.474 2.5 2.5 0 1 0-3.014 3.015 1 1 0 0 1 .474 1.68l-1.683 1.682a2.414 2.414 0 0 1-3.414 0L8.61 19.61a1 1 0 0 0-1.68.474 2.5 2.5 0 1 1-3.014-3.015 1 1 0 0 0 .474-1.68l-1.683-1.682a2.414 2.414 0 0 1 0-3.414L4.39 8.61a1 1 0 0 1 1.68.474 2.5 2.5 0 1 0 3.014-3.015 1 1 0 0 1-.474-1.68l1.683-1.682a2.414 2.414 0 0 1 3.414 0z",
    ];

    /// Orbit
    pub const ORBIT: &[&str] = &[
        "M20.341 6.484A10 10 0 0 1 10.266 21.85",
        "M3.659 17.516A10 10 0 0 1 13.74 2.152",
    ];
    pub const ORBIT_CIRCLES: [(f32, f32, f32); 3] = [
        (12.0, 12.0, 3.0), // center
        (19.0, 5.0, 2.0),  // top right
        (5.0, 19.0, 2.0),  // bottom left
    ];
}

/// Icon colors (from design spec - React prototype)
#[allow(dead_code)]
pub mod colors {
    use gpui::Hsla;

    // === Layout Constants ===

    /// macOS Tahoe window corner radius (16pt for TitleBar windows)
    pub const WINDOW_RADIUS: f32 = 16.0;

    // === Text Colors (solid grays, no opacity) ===

    /// Session name - #F2F2F2 (95% brightness)
    pub const TEXT_PRIMARY: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.95,
        a: 1.0,
    };

    /// Tool text - #999999 (60% brightness)
    pub const TEXT_SECONDARY: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.60,
        a: 1.0,
    };

    /// Placeholder text - #4D4D4D (30% brightness)
    pub const TEXT_MUTED: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.30,
        a: 1.0,
    };

    /// Header count - #808080 (50% brightness)
    pub const TEXT_HEADER: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.50,
        a: 1.0,
    };

    // === Icon Colors (solid grays, no opacity) ===

    /// State icon - #B3B3B3 (70% brightness)
    pub const ICON_STATE: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.70,
        a: 1.0,
    };

    /// Tool icon - #808080 (50% brightness)
    pub const ICON_TOOL: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.50,
        a: 1.0,
    };

    /// Default icon - #999999 (60% brightness)
    pub const ICON_DEFAULT: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.60,
        a: 1.0,
    };

    // === Background Colors ===

    /// Container background - translucent white for liquid glass
    /// Session list: linear-gradient with 0.12-0.10 alpha (use mid value)
    pub const CONTAINER_BG: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.08,
    };

    /// Session row background - rgba(255,255,255,0.06) for visible stacked cards
    pub const ROW_BG: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.06,
    };

    /// Session row hover background - rgba(255,255,255,0.12)
    pub const ROW_HOVER_BG: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.12,
    };

    /// Indicator background - rgba(255,255,255,0.10)
    pub const INDICATOR_BG: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.10,
    };

    /// Gloss overlay - rgba(255,255,255,0.15)
    pub const GLOSS: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.15,
    };

    // === Border Colors ===

    /// Standard border - rgba(255,255,255,0.2)
    pub const BORDER: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.2,
    };

    /// Content border - rgba(255,255,255,0.15)
    pub const BORDER_SUBTLE: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.15,
    };

    /// Content area background - rgba(255,255,255,0.07)
    pub const CONTENT_BG: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.07,
    };

    /// Content top highlight - rgba(255,255,255,0.25)
    pub const CONTENT_HIGHLIGHT: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.25,
    };

    /// Content area border - rgba(255,255,255,0.15)
    pub const CONTENT_BORDER: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.15,
    };

    // === Glass Highlight Colors (subtle inset glow simulation) ===

    /// Top highlight line - rgba(255,255,255,0.20) - simulates inset 0 1px glow
    pub const GLASS_TOP_HIGHLIGHT: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.20,
    };

    /// Row top highlight - rgba(255,255,255,0.10) - subtle row depth
    pub const ROW_HIGHLIGHT: Hsla = Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 0.10,
    };

    // === Legacy state colors (keep for backwards compatibility) ===

    /// Green - Running state (#22C55E)
    #[allow(dead_code)]
    pub const GREEN: Hsla = Hsla {
        h: 142.0 / 360.0,
        s: 0.71,
        l: 0.45,
        a: 1.0,
    };

    /// Blue - Idle state (#3B82F6)
    #[allow(dead_code)]
    pub const BLUE: Hsla = Hsla {
        h: 217.0 / 360.0,
        s: 0.91,
        l: 0.60,
        a: 1.0,
    };

    /// Yellow - Attention state (#EAB308)
    #[allow(dead_code)]
    pub const YELLOW: Hsla = Hsla {
        h: 45.0 / 360.0,
        s: 0.93,
        l: 0.47,
        a: 1.0,
    };

    /// Purple - Compacting state (#A855F7)
    #[allow(dead_code)]
    pub const PURPLE: Hsla = Hsla {
        h: 271.0 / 360.0,
        s: 0.91,
        l: 0.65,
        a: 1.0,
    };

    /// Gray - Stale state (#6B7280)
    #[allow(dead_code)]
    pub const GRAY: Hsla = Hsla {
        h: 220.0 / 360.0,
        s: 0.09,
        l: 0.46,
        a: 1.0,
    };

    /// Warm cream (#FFF8E7)
    #[allow(dead_code)]
    pub const CREAM: Hsla = Hsla {
        h: 43.0 / 360.0,
        s: 1.0,
        l: 0.96,
        a: 1.0,
    };
}

// Note: gpui's svg() requires an asset path, so we use embedded SVG assets
// For now, we keep the Nerd Font approach for session_list icons
// and will migrate to SVG assets when gpui supports inline SVG data

/// Get SVG asset path for a state
#[allow(dead_code)]
pub fn state_icon_path(state: aura_common::SessionState) -> &'static str {
    match state {
        aura_common::SessionState::Running => "icons/cctv.svg",
        aura_common::SessionState::Idle => "icons/message-square-code.svg",
        aura_common::SessionState::Attention => "icons/bell-ring.svg",
        aura_common::SessionState::Waiting => "icons/fan.svg",
        aura_common::SessionState::Compacting => "icons/cookie.svg",
        aura_common::SessionState::Stale => "icons/ghost.svg",
    }
}

/// Get SVG asset path for a tool name
#[allow(dead_code)]
pub fn tool_icon_asset(tool_name: &str) -> &'static str {
    match tool_name {
        "Task" => "icons/bot.svg",
        "Bash" => "icons/terminal.svg",
        "Glob" => "icons/book-search.svg",
        "Grep" => "icons/file-search.svg",
        "Read" => "icons/newspaper.svg",
        "Edit" => "icons/file-pen-line.svg",
        "Write" => "icons/file-braces.svg",
        "WebFetch" => "icons/monitor-down.svg",
        "WebSearch" => "icons/binoculars.svg",
        name if name.starts_with("mcp__") => "icons/plug.svg",
        _ => "icons/ticket.svg",
    }
}

/// Indicator icon asset paths for cycling
#[allow(dead_code)]
pub const INDICATOR_RUNNING_ASSETS: &[&str] = &[
    "icons/wand-sparkles.svg",
    "icons/sparkles.svg",
    "icons/flame.svg",
    "icons/zap.svg",
    "icons/brain.svg",
    "icons/spotlight.svg",
    "icons/biceps-flexed.svg",
    "icons/rocket.svg",
    "icons/cpu.svg",
    "icons/puzzle.svg",
    "icons/orbit.svg",
];

// Legacy Nerd Font support (for backwards compatibility during migration)

/// Nerd Font glyphs for session state icons (legacy)
pub mod nerd_state_icons {
    /// Keyboard icon - Running state (nf-fa-keyboard-o)
    pub const RUNNING: &str = "\u{f11c}";
    /// Bell icon - Attention state (nf-fa-bell)
    pub const ATTENTION: &str = "\u{f0a2}";
    /// Broom icon - Compacting state (nf-fa-broom)
    pub const COMPACTING: &str = "\u{f51a}";
    /// Coffee icon - Idle state (nf-fa-coffee)
    pub const IDLE: &str = "\u{f0f4}";
    /// Ghost icon - Stale state (nf-md-ghost)
    pub const STALE: &str = "\u{f02a0}";
}

/// Nerd Font glyphs for cycling indicator when sessions are running (legacy)
pub const RUNNING_ICONS: &[&str] = &[
    "\u{f0d0}",  // nf-fa-magic (wand)
    "\u{f005}",  // nf-fa-star
    "\u{f06d}",  // nf-fa-fire (flame)
    "\u{f0e7}",  // nf-fa-bolt (zap)
    "\u{f5dc}",  // nf-fa-brain
    "\u{f002}",  // nf-fa-search (spotlight)
    "\u{f21c}",  // nf-fa-heartbeat (pulse/energy - alternative to biceps)
    "\u{f135}",  // nf-fa-rocket
    "\u{f2db}",  // nf-fa-microchip (cpu)
    "\u{f12e}",  // nf-fa-puzzle_piece
    "\u{f1ce}",  // nf-fa-circle_o_notch (orbit/spinner)
];

/// Get Nerd Font glyph for a tool name (legacy)
pub fn tool_nerd_icon(tool_name: &str) -> &'static str {
    match tool_name {
        "Bash" => "\u{e795}",            // nf-dev-terminal
        "Read" => "\u{f441}",            // nf-oct-file_code
        "Edit" => "\u{f044}",            // nf-fa-pencil_square_o
        "Write" => "\u{f15c}",           // nf-fa-file_text (file-plus alternative)
        "Glob" => "\u{f413}",            // nf-oct-file_directory
        "Grep" => "\u{f002}",            // nf-fa-search
        "WebFetch" => "\u{f0ac}",        // nf-fa-globe
        "WebSearch" => "\u{eb8b}",       // nf-cod-search
        "Task" => "\u{f1b3}",            // nf-fa-cubes
        "TodoWrite" => "\u{f0ae}",       // nf-fa-tasks
        "LSP" => "\u{ea95}",             // nf-cod-symbol_method
        "NotebookEdit" => "\u{e606}",    // nf-seti-notebook
        "AskUserQuestion" => "\u{f128}", // nf-fa-question
        name if name.starts_with("mcp__") => "\u{f1e6}", // nf-fa-plug
        _ => "\u{f013}",                 // nf-fa-gear
    }
}

// Keep the old paths module for potential future use
#[allow(dead_code)]
pub mod paths {
    /// Check mark - no attention needed (lucide/check)
    pub const CHECK: &str = "M20 6 9 17l-5-5";

    /// Bell - attention needed (lucide/bell, combined paths)
    pub const BELL: &str = "M10.268 21a2 2 0 0 0 3.464 0 M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326";

    /// Square - idle state (simple rect, works as path)
    pub const SQUARE: &str = "M3 3h18v18H3z";

    /// Rotate CW - compacting state (lucide/rotate-cw, combined paths)
    pub const ROTATE_CW: &str = "M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8 M21 3v5h-5";

    /// Pause - stale state (two vertical bars)
    pub const PAUSE: &str = "M10 15V9 M14 15V9";

    // Tool icons (Lucide, 24x24 viewBox)

    /// Terminal - Bash tool (lucide/terminal)
    pub const TERMINAL: &str = "m4 17 6-6-6-6 M12 19h8";

    /// Book - Read tool (lucide/book-open)
    pub const BOOK: &str = "M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z";

    /// Pencil - Edit tool (lucide/pencil)
    pub const PENCIL: &str = "M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z M15 5l4 4";

    /// File - Write tool (lucide/file)
    pub const FILE: &str = "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z M14 2v4a2 2 0 0 0 2 2h4";

    /// Folder - Glob tool (lucide/folder)
    pub const FOLDER: &str = "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z";

    /// Search - Grep tool (lucide/search)
    pub const SEARCH: &str = "m21 21-4.3-4.3 M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z";

    /// Globe - WebFetch tool (lucide/globe)
    pub const GLOBE: &str = "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20Z M2 12h20 M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10Z";

    /// Plug - MCP tools (lucide/plug)
    pub const PLUG: &str = "M12 22v-5 M9 8V2 M15 8V2 M18 8v5a6 6 0 0 1-6 6 6 6 0 0 1-6-6V8Z";

    /// Robot - Task/Agent tool (lucide/bot)
    pub const ROBOT: &str = "M12 8V4H8 M8 8a4 4 0 0 0-4 4v4a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-4a4 4 0 0 0-4-4Z M9 14h.01 M15 14h.01";

    /// Gear - Default/other tools (lucide/settings)
    pub const GEAR: &str = "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z";
}

/// Get icon path for a tool name (legacy)
#[allow(dead_code)]
pub fn tool_icon(tool_name: &str) -> &'static str {
    match tool_name {
        "Bash" => paths::TERMINAL,
        "Read" => paths::BOOK,
        "Edit" => paths::PENCIL,
        "Write" => paths::FILE,
        "Glob" => paths::FOLDER,
        "Grep" => paths::SEARCH,
        "WebFetch" => paths::GLOBE,
        "WebSearch" => paths::SEARCH,
        "Task" => paths::ROBOT,
        name if name.starts_with("mcp__") => paths::PLUG,
        _ => paths::GEAR,
    }
}

/// Get SVG asset path for a tool name (legacy)
#[allow(dead_code)]
pub fn tool_icon_path(tool_name: &str) -> &'static str {
    match tool_name {
        "Bash" => "icons/terminal.svg",
        "Read" => "icons/book-open.svg",
        "Edit" => "icons/pencil.svg",
        "Write" => "icons/file.svg",
        "Glob" => "icons/folder.svg",
        "Grep" | "WebSearch" => "icons/search.svg",
        "WebFetch" => "icons/globe.svg",
        "Task" => "icons/bot.svg",
        name if name.starts_with("mcp__") => "icons/plug.svg",
        _ => "icons/settings.svg",
    }
}
