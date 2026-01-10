//! HUD window management - session status display

use super::icons;
use aura_common::{RunningTool, SessionInfo, SessionState};
use crate::registry::SessionRegistry;
use gpui::{
    div, point, px, size, svg, App, AppContext, Application, AssetSource, Bounds, Context, Div,
    Entity, IntoElement, ParentElement, Render, Result as GpuiResult, SharedString, Styled, Window,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};
use std::borrow::Cow;
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
/// Window width (enough for session name + state + tools)
const WINDOW_WIDTH: f32 = 320.0;
/// Max sessions to display
const MAX_SESSIONS: usize = 5;

/// HUD view showing session rows
struct HudView {
    state: Entity<HudStateModel>,
}

impl Render for HudView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let hud_state = self.state.read(cx);
        let sessions = &hud_state.sessions;

        // Calculate animation state (shared across all sessions for sync switching)
        let (tool_index, fade_progress) =
            calculate_animation_state(hud_state.animation_start, hud_state.animation_seed);

        // Request continuous animation frames (for smooth animation)
        window.request_animation_frame();

        if sessions.is_empty() {
            return div().size_full();
        }

        // Vertical stack of session rows
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(ROW_GAP))
            .p(px(6.0))
            .rounded(px(8.0))
            .bg(gpui::rgba(0x000000CC)) // Dark semi-transparent background
            .children(
                sessions
                    .iter()
                    .take(MAX_SESSIONS)
                    .map(|session| render_session_row(session, tool_index, fade_progress)),
            )
    }
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
        // Session name
        .child(
            div()
                .flex_shrink_0()
                .w(px(100.0))
                .text_size(px(13.0))
                .text_color(gpui::rgb(0xFFFFFF))
                .overflow_hidden()
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
    /// Current sessions to display
    sessions: Vec<SessionInfo>,
    /// Animation start time for time-based tool cycling
    animation_start: Instant,
    /// Random seed for animation timing (fixed per session)
    animation_seed: u64,
    #[allow(dead_code)]
    registry: Arc<Mutex<SessionRegistry>>,
}

impl HudStateModel {
    #[allow(dead_code)]
    fn update_from_registry(&mut self) {
        if let Ok(registry) = self.registry.lock() {
            self.sessions = registry.get_all();
        }
    }
}

/// Create demo sessions for testing
fn create_demo_sessions() -> Vec<SessionInfo> {
    vec![
        SessionInfo {
            session_id: "session-1".into(),
            cwd: "/Users/dev/projects/aura".into(),
            state: SessionState::Running,
            running_tools: vec![
                RunningTool { tool_id: "t1".into(), tool_name: "Bash".into() },
                RunningTool { tool_id: "t2".into(), tool_name: "Read".into() },
                RunningTool { tool_id: "t3".into(), tool_name: "Grep".into() },
                RunningTool { tool_id: "t4".into(), tool_name: "Edit".into() },
                RunningTool { tool_id: "t5".into(), tool_name: "Write".into() },
            ],
        },
        SessionInfo {
            session_id: "session-2".into(),
            cwd: "/Users/dev/projects/my-app".into(),
            state: SessionState::Attention,
            running_tools: vec![
                RunningTool { tool_id: "t6".into(), tool_name: "Task".into() },
                RunningTool { tool_id: "t7".into(), tool_name: "WebFetch".into() },
                RunningTool { tool_id: "t8".into(), tool_name: "mcp__notion".into() },
            ],
        },
        SessionInfo {
            session_id: "session-3".into(),
            cwd: "/Users/dev/work/api-server".into(),
            state: SessionState::Compacting,
            running_tools: vec![
                RunningTool { tool_id: "t9".into(), tool_name: "Glob".into() },
                RunningTool { tool_id: "t10".into(), tool_name: "WebSearch".into() },
            ],
        },
        SessionInfo {
            session_id: "session-4".into(),
            cwd: "/Users/dev/old-project".into(),
            state: SessionState::Idle,
            running_tools: vec![
                RunningTool { tool_id: "t11".into(), tool_name: "TodoWrite".into() },
            ],
        },
    ]
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

        // Create demo sessions for testing
        let demo_sessions = create_demo_sessions();
        let num_sessions = demo_sessions.len();

        // Generate random seed from system time for varied animation timing
        let animation_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        // Create shared state model with demo data
        let hud_state = app.new(|_cx| HudStateModel {
            sessions: demo_sessions,
            animation_start: Instant::now(),
            animation_seed,
            registry,
        });

        // Calculate window height based on number of sessions
        let window_height = (ROW_HEIGHT + ROW_GAP) * num_sessions as f32 + 12.0; // +padding

        // Position window at top-right of screen
        let window_x = screen_width - px(WINDOW_WIDTH + 20.0); // 20px from right edge
        let window_y = px(4.0); // Small offset from top

        let window_bounds = Bounds {
            origin: point(window_x, window_y),
            size: size(px(WINDOW_WIDTH), px(window_height)),
        };

        // Create single HUD window with vertical stack
        let state_for_window = hud_state.clone();
        app.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: None,
                focus: false,
                show: true,
                kind: WindowKind::PopUp,
                is_movable: false,
                is_resizable: false,
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_window, app| app.new(|_cx| HudView { state: state_for_window }),
        )
        .expect("Failed to open HUD window");

        // Note: Animation is time-based, calculated from animation_start in render.
        // The UI will update when events occur. For continuous animation during
        // fade transitions, we'd need to implement a render loop trigger.
        // For now, the animation state is calculated on each render.
        let _ = hud_state;
    });
}
