//! HUD UI module - gpui-based status display
//!
//! Architecture:
//! - assets.rs: SVG icon asset source
//! - indicator.rs: Collapsed single-icon view (Nerd Font glyph with gloss effect)
//! - session_list.rs: Expanded session row rendering
//! - animation.rs: Tool cycling, marquee, and shake animations
//! - icons.rs: Icon paths and colors

mod animation;
mod assets;
mod icons;
mod indicator;
mod session_list;

use animation::{
    calculate_animation_state, calculate_marquee_offset, MARQUEE_CHAR_WIDTH,
    MARQUEE_SPEED_PX_PER_SEC, RESET_DURATION_MS,
};
use assets::Assets;
use aura_common::SessionInfo;
use crate::registry::SessionRegistry;
use gpui::{
    actions, div, point, px, size, App, AppContext, Application, Bounds, Context, Entity,
    InteractiveElement, IntoElement, Menu, MenuItem, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, WindowBackgroundAppearance, WindowBounds,
    WindowKind, WindowOptions,
};
use session_list::{
    calculate_expanded_height, extract_session_name, ROW_GAP, SESSION_NAME_WIDTH,
    WIDTH as EXPANDED_WIDTH, MAX_SESSIONS,
};
use indicator::{HEIGHT as COLLAPSED_HEIGHT, WIDTH as COLLAPSED_WIDTH};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use unicode_width::UnicodeWidthStr;

// Define application actions
actions!(aura, [Quit]);

/// Collapse delay duration
const COLLAPSE_DELAY: Duration = Duration::from_millis(3000);


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
            (EXPANDED_WIDTH, calculate_expanded_height(num_sessions))
        } else {
            (COLLAPSED_WIDTH, COLLAPSED_HEIGHT)
        };

        // Resize window
        window.resize(size(px(width), px(height)));
        cx.notify();
    }

    /// Derive whether we should be expanded based on current state
    fn should_be_expanded(&self, has_sessions: bool) -> bool {
        if !has_sessions {
            return false;
        }

        // Hovered -> stay expanded
        if self.is_hovered {
            return true;
        }

        match self.hover_left_at {
            // Mouse left - check if delay expired
            Some(left_at) => left_at.elapsed() < COLLAPSE_DELAY,
            // Mouse never entered - stay expanded (expand on event)
            None => true,
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
    fn handle_row_hover(
        &mut self,
        session_id: String,
        needs_marquee: bool,
        hovered: bool,
        _cx: &mut Context<Self>,
    ) {
        if hovered && needs_marquee {
            // Only start scrolling if text actually overflows
            self.row_states.insert(
                session_id,
                RowMarqueeState::Scrolling {
                    start: Instant::now(),
                },
            );
        } else if !hovered {
            // Only animate reset if we were actually scrolling
            if let Some(RowMarqueeState::Scrolling { start }) = self.row_states.get(&session_id) {
                let from_offset = -(start.elapsed().as_secs_f32() * MARQUEE_SPEED_PX_PER_SEC);
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

    /// Calculate marquee offset and scrolling state for a session row
    fn calculate_marquee_state(
        &self,
        session_id: &str,
        session_name: &str,
    ) -> (f32, bool) {
        let marquee_state = self
            .row_states
            .get(session_id)
            .cloned()
            .unwrap_or_default();

        // Use unicode-width for accurate width calculation
        let display_width = session_name.width();
        let estimated_text_width = display_width as f32 * MARQUEE_CHAR_WIDTH;
        let needs_marquee = estimated_text_width > SESSION_NAME_WIDTH;

        match &marquee_state {
            RowMarqueeState::Idle => (0.0, false),
            RowMarqueeState::Scrolling { start } if needs_marquee => (
                calculate_marquee_offset(estimated_text_width, SESSION_NAME_WIDTH, *start),
                true,
            ),
            RowMarqueeState::Scrolling { .. } => (0.0, false),
            RowMarqueeState::Resetting { start, from_offset } => {
                let offset = session_list::calculate_reset_offset(
                    *from_offset,
                    start.elapsed().as_millis(),
                );
                (offset, false)
            }
        }
    }

    /// Render a session row with hover-based marquee scrolling
    fn render_session_row(
        &self,
        session: &SessionInfo,
        tool_index: usize,
        fade_progress: f32,
        animation_start: Instant,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let session_id = session.session_id.clone();
        let session_name = extract_session_name(&session.cwd);
        let (marquee_offset, is_scrolling) =
            self.calculate_marquee_state(&session_id, &session_name);

        // Calculate needs_marquee for hover handler
        let display_width = session_name.width();
        let estimated_text_width = display_width as f32 * MARQUEE_CHAR_WIDTH;
        let needs_marquee = estimated_text_width > SESSION_NAME_WIDTH;

        let session_id_for_hover = session_id.clone();

        div()
            .id(SharedString::from(format!("session-row-{}", session_id)))
            .on_hover(cx.listener(move |this, hovered: &bool, _window, cx| {
                this.handle_row_hover(session_id_for_hover.clone(), needs_marquee, *hovered, cx);
            }))
            .child(session_list::render_row_content(
                session.state,
                &session_name,
                &session.running_tools,
                tool_index,
                fade_progress,
                marquee_offset,
                is_scrolling,
                animation_start,
            ))
    }
}

impl Render for HudView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Request continuous animation frames (for smooth animation + registry polling)
        window.request_animation_frame();

        // Refresh sessions from registry on each frame
        self.state.update(cx, |state, _cx| {
            state.refresh_from_registry();
        });

        // Update hover state from window
        let window_hovered = window.is_window_hovered();
        if window_hovered != self.is_hovered {
            if window_hovered {
                // Mouse entered
                self.hover_left_at = None;
            } else {
                // Mouse left - start collapse delay
                self.hover_left_at = Some(Instant::now());
            }
            self.is_hovered = window_hovered;
        }

        // Derive and apply expansion state
        let has_sessions = !self.state.read(cx).sessions.is_empty();
        let should_expand = self.should_be_expanded(has_sessions);
        if should_expand != self.is_expanded {
            self.set_expanded(should_expand, window, cx);
        }

        // Clean up completed reset animations (Resetting -> Idle when done)
        self.cleanup_completed_resets();

        let hud_state = self.state.read(cx);
        let sessions = &hud_state.sessions;
        let is_expanded = self.is_expanded;

        // Resize window if session count changed while expanded
        let current_count = sessions.len().min(MAX_SESSIONS);
        if is_expanded && current_count != self.last_session_count {
            self.last_session_count = current_count;
            let height = calculate_expanded_height(current_count);
            window.resize(size(px(EXPANDED_WIDTH), px(height)));
        }

        // Calculate animation state (shared across all sessions for sync switching)
        let animation_start = hud_state.animation_start;
        let (tool_index, fade_progress) =
            calculate_animation_state(animation_start, hud_state.animation_seed);

        // Clone data needed for rendering (status_bar handles empty case with sleep icon)
        let sessions_for_render: Vec<_> = sessions.iter().take(MAX_SESSIONS).cloned().collect();

        // Container for HUD content
        div()
            .id("hud-container")
            .size_full()
            .cursor_pointer()
            .child(if is_expanded && !sessions_for_render.is_empty() {
                // Expanded view: full session list with glass background
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap(px(ROW_GAP))
                    .p(px(6.0))
                    .rounded(px(8.0))
                    .bg(gpui::rgba(0xFFFFFF40)) // Glass: semi-transparent white
                    .children(
                        sessions_for_render
                            .iter()
                            .map(|session| self.render_session_row(session, tool_index, fade_progress, animation_start, cx)),
                    )
                    .into_any_element()
            } else {
                // Collapsed view: single icon (also used when no sessions)
                indicator::render(&sessions_for_render, animation_start).into_any_element()
            })
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

        // Register quit action handler
        app.on_action(|_: &Quit, cx: &mut App| {
            cx.quit();
        });

        // Set up application menu
        app.set_menus(vec![Menu {
            name: "Aura".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        // Activate app to show menu bar
        app.activate(true);

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
