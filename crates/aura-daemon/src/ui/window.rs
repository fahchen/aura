//! HUD window management - session status display

use super::animation::{
    calculate_animation_state, calculate_marquee_offset, ease_in_out, ease_out,
    MARQUEE_CHAR_WIDTH, RESET_DURATION_MS,
};
use super::icons;
use aura_common::{RunningTool, SessionInfo, SessionState};
use crate::registry::SessionRegistry;
use unicode_width::UnicodeWidthStr;
use gpui::{
    div, point, px, size, svg, App, AppContext, Application, AssetSource, Bounds, Context, Div,
    Entity, InteractiveElement, IntoElement, ParentElement, Render, Result as GpuiResult,
    SharedString, StatefulInteractiveElement, Styled, Window, WindowBackgroundAppearance,
    WindowBounds, WindowKind, WindowOptions,
};
use std::borrow::Cow;
use std::collections::HashMap;
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

/// Animation state for a session row's marquee
#[derive(Clone)]
enum RowMarqueeState {
    /// Not scrolling, at offset 0
    Idle,
    /// Scrolling while hovered
    Scrolling { start: Instant },
    /// Resetting back to offset 0 with ease animation
    Resetting { start: Instant, from_offset: f32 },
}

impl Default for RowMarqueeState {
    fn default() -> Self {
        Self::Idle
    }
}

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
    /// Per-row marquee animation state (session_id -> state)
    row_states: HashMap<String, RowMarqueeState>,
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

    /// Clean up completed reset animations by transitioning Resetting -> Idle
    fn cleanup_completed_resets(&mut self) {
        let reset_duration = std::time::Duration::from_millis(RESET_DURATION_MS);
        for state in self.row_states.values_mut() {
            if let RowMarqueeState::Resetting { start, .. } = state {
                if start.elapsed() >= reset_duration {
                    *state = RowMarqueeState::Idle;
                }
            }
        }
    }

    /// Handle hover state change for a session row's marquee animation
    fn handle_row_hover(&mut self, session_id: String, hovered: bool, _cx: &mut Context<Self>) {
        if hovered {
            // Start scrolling
            self.row_states.insert(
                session_id,
                RowMarqueeState::Scrolling {
                    start: Instant::now(),
                },
            );
        } else {
            // Calculate current offset and start reset animation
            if let Some(state) = self.row_states.get(&session_id) {
                let from_offset = match state {
                    RowMarqueeState::Scrolling { start } => {
                        // Need to calculate where we are in the scroll
                        // We don't have text_width here, so we'll store a sentinel
                        // and calculate the actual offset in render
                        let elapsed = start.elapsed();
                        // Store elapsed time as a proxy - we'll calculate real offset in render
                        -(elapsed.as_secs_f32() * MARQUEE_SPEED_PX_PER_SEC)
                    }
                    RowMarqueeState::Resetting { from_offset, .. } => *from_offset,
                    RowMarqueeState::Idle => 0.0,
                };
                self.row_states.insert(
                    session_id,
                    RowMarqueeState::Resetting {
                        start: Instant::now(),
                        from_offset,
                    },
                );
            }
        }
    }

    /// Render a session row with hover-based marquee scrolling
    fn render_session_row(
        &self,
        session: &SessionInfo,
        tool_index: usize,
        fade_progress: f32,
        marquee_state: RowMarqueeState,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<Div> {
        let state_color = state_to_color(session.state);
        let session_name = extract_session_name(&session.cwd);
        let session_id = session.session_id.clone();

        // Use unicode-width for accurate width calculation (CJK = 2 units, ASCII = 1 unit)
        let display_width = session_name.width();
        let estimated_text_width = display_width as f32 * MARQUEE_CHAR_WIDTH;
        let needs_marquee = estimated_text_width > SESSION_NAME_WIDTH;

        // Calculate marquee offset based on state
        let (marquee_offset, is_scrolling) = match &marquee_state {
            RowMarqueeState::Idle => (0.0, false),
            RowMarqueeState::Scrolling { start } if needs_marquee => (
                calculate_marquee_offset(estimated_text_width, SESSION_NAME_WIDTH, *start),
                true,
            ),
            RowMarqueeState::Scrolling { .. } => (0.0, false),
            RowMarqueeState::Resetting { start, from_offset } => {
                let elapsed = start.elapsed().as_millis() as f32;
                let progress = (elapsed / RESET_DURATION_MS as f32).min(1.0);
                let eased = ease_out(progress);
                (from_offset * (1.0 - eased), false)
            }
        };

        div()
            .id(SharedString::from(format!("session-row-{}", session_id)))
            .w_full()
            .h(px(ROW_HEIGHT))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .px(px(8.0))
            .rounded(px(6.0))
            .on_hover(cx.listener(move |this, hovered: &bool, _window, cx| {
                this.handle_row_hover(session_id.clone(), *hovered, cx);
            }))
            // Status dot column (fixed width)
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(STATUS_DOT_WIDTH))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(div().size(px(10.0)).rounded_full().bg(state_color)),
            )
            // Session name column (fixed width with seamless marquee)
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(SESSION_NAME_WIDTH))
                    .h_full()
                    .overflow_hidden()
                    .child(
                        div()
                            .h_full()
                            .flex()
                            .items_center()
                            .ml(px(marquee_offset))
                            .font_family("Maple Mono NF CN")
                            .text_size(px(13.0))
                            .text_color(gpui::rgb(0xFFFFFF))
                            .whitespace_nowrap()
                            .child(if needs_marquee && is_scrolling {
                                // Two copies for seamless loop when scrolling
                                format!("{}    {}", session_name, session_name)
                            } else {
                                session_name
                            }),
                    ),
            )
            // Current tool (flex-1, takes remaining space)
            .child(render_current_tool(
                &session.running_tools,
                tool_index,
                fade_progress,
            ))
    }
}

/// Constant for marquee scroll speed (imported from animation module)
const MARQUEE_SPEED_PX_PER_SEC: f32 = super::animation::MARQUEE_SPEED_PX_PER_SEC;

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

        // Clean up completed reset animations (Resetting -> Idle when done)
        self.cleanup_completed_resets();

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
        let animation_start = hud_state.animation_start;
        let (tool_index, fade_progress) =
            calculate_animation_state(animation_start, hud_state.animation_seed);

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
                    .rounded(px(8.0))
                    .bg(gpui::rgba(0x000000BB))
                    .children(sessions_for_render.iter().map(|session| {
                        let session_id = session.session_id.clone();
                        let marquee_state = self
                            .row_states
                            .get(&session_id)
                            .cloned()
                            .unwrap_or_default();
                        self.render_session_row(session, tool_index, fade_progress, marquee_state, cx)
                    }))
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
                    .rounded(px(8.0))
                    .bg(gpui::rgba(0x000000BB))
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

/// Fixed column widths for table-style layout
const STATUS_DOT_WIDTH: f32 = 18.0; // Status dot column (10px dot + padding)
const SESSION_NAME_WIDTH: f32 = 80.0; // Fixed session name column


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


use std::time::Instant;

/// Run the HUD application
///
/// This function blocks and runs the gpui event loop.
/// Call from main thread only.
pub fn run_hud(registry: Arc<Mutex<SessionRegistry>>) {
    Application::new().with_assets(Assets).run(|app: &mut App| {
        // Register embedded Maple Mono font for consistent marquee rendering
        let font_data = include_bytes!("../../assets/fonts/MapleMono-NF-CN-Regular.ttf");
        app.text_system()
            .add_fonts(vec![Cow::Borrowed(font_data.as_slice())])
            .expect("Failed to load Maple Mono font");
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
                window_background: WindowBackgroundAppearance::Transparent,
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
                    row_states: HashMap::new(),
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
