//! SVG icon paths and colors for HUD icons
//!
//! Icons from Lucide (https://lucide.dev), 24x24 viewBox
//! Rendered at 16x16px in the HUD using gpui's svg() element

/// Get SVG asset path for a state
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
