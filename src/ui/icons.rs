//! SVG icon paths and colors for HUD icons
//!
//! Icons from Lucide (https://lucide.dev), 24x24 viewBox
//! Rendered at 16x16px in the HUD using gpui's svg() element

/// Get SVG asset path for a state
pub fn state_icon_path(state: crate::SessionState) -> &'static str {
    match state {
        crate::SessionState::Running => "icons/cctv.svg",
        crate::SessionState::Idle => "icons/message-square-code.svg",
        crate::SessionState::Attention => "icons/bell-ring.svg",
        crate::SessionState::Waiting => "icons/fan.svg",
        crate::SessionState::Compacting => "icons/cookie.svg",
        crate::SessionState::Stale => "icons/ghost.svg",
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
