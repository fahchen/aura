//! HUD window management - session status display

use super::icons;
use aura_common::{RunningTool, SessionInfo, SessionState};
use crate::registry::SessionRegistry;
use gpui::{
    div, point, px, size, svg, App, AppContext, Application, AssetSource, Bounds, Context, Div,
    Entity, InteractiveElement, IntoElement, ParentElement, Render, Result as GpuiResult,
    SharedString, StatefulInteractiveElement, Styled, Window, WindowBackgroundAppearance,
    WindowBounds, WindowKind, WindowOptions,
};
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Asset source for loading icons from the assets directory
struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> GpuiResult<Option<Cow<'static, [u8]>>> {
        // Load from the assets directory embedded at compile time
        let content = match path {
            "icons/terminal.svg" => include_bytes!("../../assets/icons/terminal.svg").as_slice(),
            "icons/book-open.svg" => include_bytes!("../../assets/icons/book-open.svg").as_slice(),
            "icons/pencil.svg" => include_bytes!("../../assets/icons/pencil.svg").as_slice(),
            "icons/file.svg" => include_bytes!("../../assets/icons/file.svg").as_slice(),
            "icons/folder.svg" => include_bytes!("../../assets/icons/folder.svg").as_slice(),
            "icons/search.svg" => include_bytes!("../../assets/icons/search.svg").as_slice(),
            "icons/globe.svg" => include_bytes!("../../assets/icons/globe.svg").as_slice(),
            "icons/plug.svg" => include_bytes!("../../assets/icons/plug.svg").as_slice(),
            "icons/bot.svg" => include_bytes!("../../assets/icons/bot.svg").as_slice(),
            "icons/settings.svg" => include_bytes!("../../assets/icons/settings.svg").as_slice(),
            "icons/check.svg" => include_bytes!("../../assets/icons/check.svg").as_slice(),
            "icons/bell.svg" => include_bytes!("../../assets/icons/bell.svg").as_slice(),
            _ => return Ok(None),
        };
        Ok(Some(Cow::Borrowed(content)))
    }

    fn list(&self, _path: &str) -> GpuiResult<Vec<SharedString>> {
        Ok(vec![])
    }
}

/// Row dimensions
const ROW_HEIGHT: f32 = 32.0;
const ROW_GAP: f32 = 4.0;
/// Window dimensions
const COLLAPSED_WIDTH: f32 = 60.0; // Two icons side by side
const COLLAPSED_HEIGHT: f32 = 28.0; // Single row height
const EXPANDED_WIDTH: f32 = 200.0; // Compact session list width
/// Max sessions to display
const MAX_SESSIONS: usize = 5;
/// Icon size for collapsed view
const ICON_SIZE: f32 = 16.0;

/// Collapse delay in milliseconds
const COLLAPSE_DELAY_MS: u128 = 3000;

/// HUD view showing session rows
struct HudView {
    state: Entity<HudStateModel>,
    is_expanded: bool,
    is_hovered: bool,
    /// Time when mouse left the window (for delayed collapse)
    hover_left_at: Option<Instant>,
    /// Shared flag set by background thread when collapse should happen
    collapse_signal: Arc<AtomicBool>,
    /// Last known session count (for resize detection)
    last_session_count: usize,
}

impl HudView {
    fn set_expanded(&mut self, expanded: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_expanded == expanded {
            return;
        }
        self.is_expanded = expanded;

        // Calculate new window size
        let num_sessions = self.state.read(cx).sessions.len().min(MAX_SESSIONS);
        let (width, height) = if expanded {
            let h = (ROW_HEIGHT + ROW_GAP) * num_sessions as f32 + 12.0;
            (EXPANDED_WIDTH, h)
        } else {
            (COLLAPSED_WIDTH, COLLAPSED_HEIGHT)
        };

        // Resize window (keeps top-left position, so expanded will grow to the right)
        window.resize(size(px(width), px(height)));
        cx.notify();
    }

    fn set_hovered(&mut self, hovered: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.is_hovered = hovered;

        if hovered {
            // Mouse entered - expand immediately and clear any pending collapse
            self.hover_left_at = None;
            self.collapse_signal.store(false, Ordering::SeqCst);
            self.set_expanded(true, window, cx);
        } else {
            // Mouse left - start collapse timer
            self.hover_left_at = Some(Instant::now());

            // Spawn background thread to signal collapse after delay
            let signal = self.collapse_signal.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(COLLAPSE_DELAY_MS as u64));
                signal.store(true, Ordering::SeqCst);
            });
        }
    }

}

impl Render for HudView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Request continuous animation frames (for smooth animation + registry polling)
        window.request_animation_frame();

        // Check if background thread signaled collapse
        if self.collapse_signal.swap(false, Ordering::SeqCst) {
            // Only collapse if we're still in the "left" state (not re-hovered)
            if self.hover_left_at.is_some() {
                self.hover_left_at = None;
                self.set_expanded(false, window, cx);
            }
        }

        // Refresh sessions from registry on each frame
        self.state.update(cx, |state, _cx| {
            state.refresh_from_registry();
        });

        let hud_state = self.state.read(cx);
        let sessions = &hud_state.sessions;
        let is_expanded = self.is_expanded;

        // Resize window if session count changed while expanded
        let current_count = sessions.len().min(MAX_SESSIONS);
        if is_expanded && current_count != self.last_session_count {
            self.last_session_count = current_count;
            let height = (ROW_HEIGHT + ROW_GAP) * current_count as f32 + 12.0;
            window.resize(size(px(EXPANDED_WIDTH), px(height)));
        }

        // Calculate animation state (shared across all sessions for sync switching)
        let (tool_index, fade_progress) =
            calculate_animation_state(hud_state.animation_start, hud_state.animation_seed);

        if sessions.is_empty() {
            return div().id("hud-empty").size_full();
        }

        // Calculate aggregate state for collapsed view
        let has_attention = sessions.iter().any(|s| s.state == SessionState::Attention);
        let aggregate_state = get_aggregate_state(sessions);

        // Clone data needed for closures
        let sessions_for_render: Vec<_> = sessions
            .iter()
            .take(MAX_SESSIONS)
            .cloned()
            .collect();

        // Container for HUD content
        div()
            .id("hud-container")
            .size_full()
            .cursor_pointer()
            .on_hover(cx.listener(|this, hovered: &bool, window, cx| {
                this.set_hovered(*hovered, window, cx);
            }))
            .child(if is_expanded {
                // Expanded view: full session list
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap(px(ROW_GAP))
                    .p(px(6.0))
                    .rounded_bl(px(8.0))
                    .rounded_br(px(8.0))
                    .bg(gpui::rgba(0x000000CC))
                    .children(
                        sessions_for_render
                            .iter()
                            .map(|session| render_session_row(session, tool_index, fade_progress)),
                    )
            } else {
                // Collapsed view: two icons
                div()
                    .size_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .gap(px(8.0))
                    .p(px(6.0))
                    .rounded_bl(px(8.0))
                    .rounded_br(px(8.0))
                    .bg(gpui::rgba(0x000000CC))
                    // Left icon: attention indicator
                    .child(render_status_icon(has_attention))
                    // Right icon: aggregate state
                    .child(render_aggregate_icon(aggregate_state))
            })
    }
}

/// Get aggregate state from sessions (priority: Running > Compacting > Idle > Stale)
fn get_aggregate_state(sessions: &[SessionInfo]) -> SessionState {
    if sessions.iter().any(|s| s.state == SessionState::Running) {
        SessionState::Running
    } else if sessions.iter().any(|s| s.state == SessionState::Compacting) {
        SessionState::Compacting
    } else if sessions.iter().any(|s| s.state == SessionState::Idle) {
        SessionState::Idle
    } else {
        SessionState::Stale
    }
}

/// Render left status icon (attention indicator)
fn render_status_icon(has_attention: bool) -> Div {
    let (icon_path, color) = if has_attention {
        ("icons/bell.svg", icons::colors::YELLOW)
    } else {
        ("icons/check.svg", icons::colors::GREEN)
    };

    div()
        .size(px(ICON_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(icon_path).size(px(ICON_SIZE)).text_color(color))
}

/// Render right aggregate state icon
fn render_aggregate_icon(state: SessionState) -> Div {
    let (icon_path, color) = match state {
        SessionState::Running => ("icons/terminal.svg", icons::colors::GREEN),
        SessionState::Compacting => ("icons/settings.svg", icons::colors::PURPLE), // rotating gear
        SessionState::Idle => ("icons/file.svg", icons::colors::BLUE),
        SessionState::Attention => ("icons/bell.svg", icons::colors::YELLOW),
        SessionState::Stale => ("icons/file.svg", icons::colors::GRAY),
    };

    div()
        .size(px(ICON_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(icon_path).size(px(ICON_SIZE)).text_color(color))
}

/// Render a session row: status dot + session name + current tool
fn render_session_row(session: &SessionInfo, tool_index: usize, fade_progress: f32) -> Div {
    let state_color = state_to_color(session.state);
    let session_name = extract_session_name(&session.cwd);

    div()
        .w_full()
        .h(px(ROW_HEIGHT))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .px(px(8.0))
        .rounded(px(6.0))
        .bg(gpui::rgba(0xFFFFFF11)) // Subtle row background
        // Status dot
        .child(
            div()
                .size(px(10.0))
                .rounded_full()
                .bg(state_color),
        )
        // Session name (auto-width, max 80px)
        .child(
            div()
                .flex_shrink_0()
                .max_w(px(80.0))
                .text_size(px(13.0))
                .text_color(gpui::rgb(0xFFFFFF))
                .overflow_hidden()
                .whitespace_nowrap()
                .child(session_name),
        )
        // Current tool (cycles through tools with cross-fade)
        .child(render_current_tool(&session.running_tools, tool_index, fade_progress))
}

/// Quadratic ease-in-out function
fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Create RGBA color with specified alpha (0.0 to 1.0)
fn rgba_with_alpha(alpha: f32) -> gpui::Rgba {
    gpui::rgba((0xFFFFFF00u32) | ((alpha * 255.0) as u32))
}

/// Render a tool with its Lucide icon
fn render_tool_with_icon(tool_name: &str, opacity: f32) -> Div {
    let icon_path = icons::tool_icon_path(tool_name);
    let color = rgba_with_alpha(opacity);

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0)) // Space between icon and name
        .child(
            svg()
                .path(icon_path)
                .size(px(14.0))
                .text_color(color),
        )
        .child(
            div()
                .text_color(color)
                .child(tool_name.to_string()),
        )
}

/// Render current tool with cross-fade animation
/// Shows one tool at a time, cycling through the list
fn render_current_tool(tools: &[RunningTool], tool_index: usize, fade_progress: f32) -> Div {
    if tools.is_empty() {
        return div().flex_1();
    }

    // Get current and next tool indices
    let current_idx = tool_index % tools.len();
    let next_idx = (tool_index + 1) % tools.len();
    let current_tool = &tools[current_idx].tool_name;
    let next_tool = &tools[next_idx].tool_name;

    // Apply easing to fade progress
    let progress = ease_in_out(fade_progress);
    let current_opacity = 1.0 - progress; // fades out
    let next_opacity = progress; // fades in

    // Stack both tools with cross-fade opacity using a relative container
    div()
        .flex_1()
        .h_full()
        .relative()
        .text_size(px(12.0))
        // Current tool (fading out)
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .child(render_tool_with_icon(current_tool, current_opacity * 0.8)),
        )
        // Next tool (fading in)
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .child(render_tool_with_icon(next_tool, next_opacity * 0.8)),
        )
}

/// Extract session name from cwd (last folder name)
fn extract_session_name(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("session")
        .to_string()
}

/// Convert SessionState to color
fn state_to_color(state: SessionState) -> gpui::Hsla {
    match state {
        SessionState::Running => icons::colors::GREEN,
        SessionState::Idle => icons::colors::BLUE,
        SessionState::Attention => icons::colors::YELLOW,
        SessionState::Compacting => icons::colors::PURPLE,
        SessionState::Stale => icons::colors::GRAY,
    }
}

/// Shared HUD state model
struct HudStateModel {
    /// Current sessions to display (refreshed from registry)
    sessions: Vec<SessionInfo>,
    /// Animation start time for time-based tool cycling
    animation_start: Instant,
    /// Random seed for animation timing (fixed per session)
    animation_seed: u64,
    /// Registry reference for refreshing session data
    registry: Arc<Mutex<SessionRegistry>>,
}

impl HudStateModel {
    /// Refresh sessions from registry
    fn refresh_from_registry(&mut self) {
        if let Ok(registry) = self.registry.lock() {
            self.sessions = registry.get_all();
        }
    }
}


/// Animation timing constants
const CYCLE_DURATION_MIN_MS: u64 = 1500; // Min time showing each tool
const CYCLE_DURATION_MAX_MS: u64 = 2000; // Max time showing each tool
const FADE_DURATION_MS: u64 = 500; // Cross-fade duration

use std::time::Instant;

/// Simple hash function for deterministic pseudo-random per cycle
fn cycle_hash(cycle: u64, seed: u64) -> u64 {
    // Combine cycle and seed, then apply multiplicative hash
    let mut x = (cycle ^ seed).wrapping_mul(0x517cc1b727220a95);
    x ^= x >> 32;
    x
}

/// Get randomized cycle duration for a specific cycle number and seed
fn get_cycle_duration(cycle: u64, seed: u64) -> u64 {
    let hash = cycle_hash(cycle, seed);
    let range = CYCLE_DURATION_MAX_MS - CYCLE_DURATION_MIN_MS;
    CYCLE_DURATION_MIN_MS + (hash % (range + 1))
}

/// Calculate animation state based on elapsed time
/// Returns (tool_index, fade_progress)
fn calculate_animation_state(start_time: Instant, seed: u64) -> (usize, f32) {
    let elapsed_ms = start_time.elapsed().as_millis() as u64;

    // Find which cycle we're in by iterating (since durations vary)
    let mut accumulated_ms: u64 = 0;
    let mut cycle: u64 = 0;

    loop {
        let cycle_duration = get_cycle_duration(cycle, seed);
        let total_cycle_ms = cycle_duration + FADE_DURATION_MS;

        if accumulated_ms + total_cycle_ms > elapsed_ms {
            // We're in this cycle
            let pos_in_cycle = elapsed_ms - accumulated_ms;

            if pos_in_cycle < cycle_duration {
                // Showing current tool (no fade)
                return (cycle as usize, 0.0);
            } else {
                // Fading to next tool
                let fade_elapsed = pos_in_cycle - cycle_duration;
                let progress = (fade_elapsed as f32) / (FADE_DURATION_MS as f32);
                return (cycle as usize, progress.min(1.0));
            }
        }

        accumulated_ms += total_cycle_ms;
        cycle += 1;

        // Safety limit to prevent infinite loop
        if cycle > 10000 {
            return (cycle as usize, 0.0);
        }
    }
}

/// Run the HUD application
///
/// This function blocks and runs the gpui event loop.
/// Call from main thread only.
pub fn run_hud(registry: Arc<Mutex<SessionRegistry>>) {
    Application::new().with_assets(Assets).run(|app: &mut App| {
        // Get primary display for positioning
        let displays = app.displays();
        let primary = displays.first().expect("No display found");
        let display_bounds = primary.bounds();

        let screen_width = display_bounds.size.width;

        // Get initial sessions from registry
        let initial_sessions = registry
            .lock()
            .map(|r| r.get_all())
            .unwrap_or_default();

        // Generate random seed from system time for varied animation timing
        let animation_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        // Create shared state model with registry data
        let hud_state = app.new(|_cx| HudStateModel {
            sessions: initial_sessions,
            animation_start: Instant::now(),
            animation_seed,
            registry,
        });

        // Start with collapsed view, centered under notch
        // Position so that when expanded, the window grows to the right
        let window_x = (screen_width - px(EXPANDED_WIDTH)) / 2.0;
        let window_y = px(30.0); // Just below menu bar

        let window_bounds = Bounds {
            origin: point(window_x, window_y),
            size: size(px(COLLAPSED_WIDTH), px(COLLAPSED_HEIGHT)),
        };

        // Create HUD window (starts collapsed)
        let state_for_window = hud_state.clone();
        app.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: None,
                focus: true,
                show: true,
                kind: WindowKind::PopUp,
                is_movable: true,
                is_resizable: false,
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_window, app| {
                app.new(|_cx| HudView {
                    state: state_for_window,
                    is_expanded: false,
                    is_hovered: false,
                    hover_left_at: None,
                    collapse_signal: Arc::new(AtomicBool::new(false)),
                    last_session_count: 0,
                })
            },
        )
        .expect("Failed to open HUD window");

        // Note: Animation is time-based, calculated from animation_start in render.
        // The UI will update when events occur. For continuous animation during
        // fade transitions, we'd need to implement a render loop trigger.
        // For now, the animation state is calculated on each render.
        let _ = hud_state;
    });
}
