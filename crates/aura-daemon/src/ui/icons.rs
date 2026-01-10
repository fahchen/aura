//! SVG icon paths and colors for HUD icons
//!
//! Icons from Lucide (https://lucide.dev), 24x24 viewBox
//! Rendered at 16x16px in the HUD

use gpui::Hsla;

/// Icon colors (from design spec)
pub mod colors {
    use gpui::Hsla;

    /// Green - Running state, Check icon (#22C55E)
    pub const GREEN: Hsla = Hsla {
        h: 142.0 / 360.0,
        s: 0.71,
        l: 0.45,
        a: 1.0,
    };

    /// Blue - Idle state (#3B82F6)
    pub const BLUE: Hsla = Hsla {
        h: 217.0 / 360.0,
        s: 0.91,
        l: 0.60,
        a: 1.0,
    };

    /// Yellow - Attention state (#EAB308)
    pub const YELLOW: Hsla = Hsla {
        h: 45.0 / 360.0,
        s: 0.93,
        l: 0.47,
        a: 1.0,
    };

    /// Purple - Compacting state (#A855F7)
    pub const PURPLE: Hsla = Hsla {
        h: 271.0 / 360.0,
        s: 0.91,
        l: 0.65,
        a: 1.0,
    };

    /// Gray - Stale state (#6B7280)
    pub const GRAY: Hsla = Hsla {
        h: 220.0 / 360.0,
        s: 0.09,
        l: 0.46,
        a: 1.0,
    };
}

/// SVG paths for icons (Lucide, 24x24 viewBox)
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

/// Get icon path for a tool name
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

/// Get SVG asset path for a tool name
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

/// Get left icon info (attention indicator)
pub fn left_icon(has_attention: bool) -> (&'static str, Hsla) {
    if has_attention {
        (paths::BELL, colors::YELLOW)
    } else {
        (paths::CHECK, colors::GREEN)
    }
}

/// Get right icon info (aggregate state)
pub fn right_icon(state: super::state::AggregateState) -> (&'static str, Hsla) {
    use super::state::AggregateState;

    match state {
        AggregateState::Running => (paths::SQUARE, colors::GREEN), // Placeholder, matrix used instead
        AggregateState::Compacting => (paths::ROTATE_CW, colors::PURPLE),
        AggregateState::Idle => (paths::SQUARE, colors::BLUE),
        AggregateState::Stale => (paths::PAUSE, colors::GRAY),
    }
}
