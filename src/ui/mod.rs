//! HUD UI module - gpui-based status display
//!
//! Architecture:
//! - Two separate popup windows: Indicator (36x36) and Session List (320xN)
//! - assets.rs: SVG icon asset source
//! - indicator.rs: Single centered icon showing aggregate state
//! - session_list.rs: Expanded session row rendering
//! - animation.rs: Tool cycling, marquee, and shake animations
//! - icons.rs: Icon paths and colors
//! - theme.rs: Theme system with Dark, Light, and System modes

mod animation;
pub(crate) mod assets;
mod glass;
pub(crate) mod icons;
pub(crate) mod indicator;
pub(crate) mod session_list;
pub(crate) mod theme;


use animation::{
    calculate_animation_state, calculate_breathe_opacity, calculate_icon_swap,
    calculate_row_slide_in, calculate_row_slide_out,
};
use assets::Assets;
use crate::{SessionInfo, SessionState};
use crate::registry::SessionRegistry;
use gpui::{
    actions, div, point, px, size, uniform_list, App, AppContext, Application, Bounds, Context,
    Entity, InteractiveElement, IntoElement, Menu, MenuItem, ParentElement, Pixels, Point, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, WindowBackgroundAppearance,
    WindowBounds, WindowHandle, WindowKind, WindowOptions,
    prelude::FluentBuilder,
};
use session_list::{
    calculate_expanded_height, extract_session_name, ROW_GAP,
    WIDTH as EXPANDED_WIDTH, MAX_SESSIONS,
};
use indicator::{HEIGHT as COLLAPSED_HEIGHT, WIDTH as COLLAPSED_WIDTH};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Instant;


// Define application actions
actions!(aura, [Quit, SetThemeSystem, SetThemeLiquidDark, SetThemeLiquidLight, SetThemeSolidDark, SetThemeSolidLight]);

/// Gap between indicator and session list windows
const WINDOW_GAP: f32 = 4.0;



/// Shared HUD state between indicator and session list windows
pub(crate) struct SharedHudState {
    /// Current sessions to display (refreshed from registry)
    sessions: Vec<SessionInfo>,
    /// Animation start time for time-based tool cycling
    animation_start: Instant,
    /// Random seed for animation timing (fixed per session)
    animation_seed: u64,
    /// Registry reference for refreshing session data
    registry: Arc<Mutex<SessionRegistry>>,
    /// Whether the session list window should be visible
    session_list_visible: bool,
    /// Session list window handle (for tracking open/close state)
    session_list_window: Option<WindowHandle<SessionListView>>,
    /// Session list window origin position (for reopening at same location)
    session_list_origin: Point<Pixels>,
    /// Indicator window handle (for getting current position)
    indicator_window: Option<WindowHandle<IndicatorView>>,
    /// Theme style preference (System, LiquidDark, LiquidLight, SolidDark, SolidLight)
    theme_style: theme::ThemeStyle,
    /// Whether the system is currently in dark mode (detected from OS)
    system_is_dark: bool,
    /// Whether registry data changed and needs refresh
    registry_dirty: Arc<AtomicBool>,
}

#[cfg(test)]
impl SharedHudState {
    /// Create a SharedHudState for testing with given sessions
    pub(crate) fn new_for_test(sessions: Vec<SessionInfo>) -> Self {
        let registry = Arc::new(Mutex::new(SessionRegistry::new()));
        let registry_dirty = Arc::new(AtomicBool::new(false));
        Self {
            sessions,
            animation_start: Instant::now(),
            animation_seed: 42,
            registry,
            session_list_visible: false,
            session_list_window: None,
            session_list_origin: point(px(0.0), px(0.0)),
            indicator_window: None,
            theme_style: theme::ThemeStyle::System,
            system_is_dark: true,
            registry_dirty,
        }
    }
}

impl SharedHudState {
    /// Refresh sessions from registry
    /// - Shows all sessions (including Idle and Stale)
    fn refresh_from_registry(&mut self) {
        if let Ok(registry) = self.registry.lock() {
            self.sessions = registry.get_all();
        }
    }

    /// Get the current resolved theme colors
    fn theme_colors(&self) -> theme::ThemeColors {
        let resolved = self.theme_style.resolve(self.system_is_dark);
        theme::ThemeColors::for_style(resolved)
    }

    /// Update system appearance from window
    fn update_system_appearance(&mut self, appearance: gpui::WindowAppearance) {
        self.system_is_dark = theme::is_system_dark(appearance);
    }
}

/// Indicator window view (36x36px, always visible)
struct IndicatorView {
    state: Entity<SharedHudState>,
    /// Track hover state for enhanced visual effect
    is_hovered: bool,
    /// Track window position at mouse down (for drag detection)
    window_pos_at_mouse_down: Option<Point<Pixels>>,
}

impl Render for IndicatorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Request continuous animation frames
        window.request_animation_frame();

        // Update system appearance and refresh sessions from registry on each frame
        let appearance = window.appearance();
        self.state.update(cx, |state, _cx| {
            state.update_system_appearance(appearance);
            if state.registry_dirty.swap(false, Ordering::Relaxed) {
                state.refresh_from_registry();
            }
        });

        let hud_state = self.state.read(cx);
        let sessions = &hud_state.sessions;
        let animation_start = hud_state.animation_start;
        let theme_colors = hud_state.theme_colors();
        let sessions_for_render: Vec<_> = sessions.iter().take(MAX_SESSIONS).cloned().collect();

        let is_hovered = self.is_hovered;

        // Indicator container with click and drag support
        div()
            .id("indicator-container")
            .size_full()
            .cursor(gpui::CursorStyle::OpenHand)
            .on_hover(cx.listener(|this, hovered: &bool, _window, _cx| {
                this.is_hovered = *hovered;
            }))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _event: &gpui::MouseDownEvent, window, _cx| {
                    // Record window position before drag starts
                    this.window_pos_at_mouse_down = Some(window.bounds().origin);
                    // Start window drag on mouse down
                    window.start_window_move();
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Right,
                cx.listener({
                    let state = self.state.clone();
                    move |_this, _event: &gpui::MouseDownEvent, _window, app| {
                        state.update(app, |state, _cx| {
                            state.theme_style = state.theme_style.next();
                            save_theme(state.theme_style);
                        });
                    }
                }),
            )
            .on_click({
                let state_for_click = self.state.clone();
                cx.listener(move |this, _event: &gpui::ClickEvent, window, app| {
                    // Check if window moved (was dragged)
                    if let Some(start_pos) = this.window_pos_at_mouse_down.take() {
                        let current_pos = window.bounds().origin;
                        let threshold = px(5.0);
                        let dx = current_pos.x - start_pos.x;
                        let dy = current_pos.y - start_pos.y;
                        if dx > threshold || dx < -threshold || dy > threshold || dy < -threshold {
                            // Window was dragged — save new position
                            let pos = current_pos;
                            let state = crate::config::State {
                                indicator_x: Some(f32::from(pos.x) as f64),
                                indicator_y: Some(f32::from(pos.y) as f64),
                            };
                            let _ = crate::config::save_state(&state);
                            return;
                        }
                    }

                    // Toggle session list immediately
                    state_for_click.update(app, |state, _cx| {
                        state.registry_dirty.store(true, Ordering::Relaxed);
                    });
                    let hud_state = state_for_click.read(app);
                    let was_visible = hud_state.session_list_visible;
                    let window_handle = hud_state.session_list_window;

                    let should_open = !was_visible;
                    let should_close = was_visible && window_handle.is_some();

                    // NOTE: When the session list window is moved and then closed, gpui logs
                    // "window not found" errors. This is a known gpui limitation (v0.2.2):
                    // - When a window is moved, gpui registers internal callbacks for position tracking
                    // - When remove_window() is called, these callbacks still fire
                    // - The callbacks fail to find the window -> "window not found" is logged
                    // - Error locations in gpui: app.rs:1388, app.rs:2201, window.rs:4725
                    // - This is benign: the window closes correctly, no functional impact
                    // - Fix requires changes to gpui's callback cleanup logic
                    if should_close {
                        let handle = window_handle.unwrap();
                        let _ = handle.update(app, |_view, window, _cx| {
                            window.remove_window();
                        });

                        state_for_click.update(app, |state, _cx| {
                            state.session_list_visible = false;
                            state.session_list_window = None;
                        });
                    } else if should_open {
                        let indicator_origin = window.bounds().origin;
                        let session_list_origin = point(
                            indicator_origin.x - px((EXPANDED_WIDTH - COLLAPSED_WIDTH) / 2.0),
                            indicator_origin.y + px(COLLAPSED_HEIGHT + WINDOW_GAP),
                        );
                        let session_count = hud_state.sessions.len().clamp(1, MAX_SESSIONS);
                        let height = calculate_expanded_height(session_count);

                        state_for_click.update(app, |state, _cx| {
                            state.session_list_visible = true;
                            state.session_list_origin = session_list_origin;
                        });
                        if window_handle.is_none() {
                            open_session_list_window_sync(app, state_for_click.clone());
                        } else if let Some(handle) = window_handle {
                            let _ = handle.update(app, |_view, window, _cx| {
                                window.resize(size(px(EXPANDED_WIDTH), px(height)));
                            });
                        }
                    }
                })
            })
            .child(indicator::render(&sessions_for_render, animation_start, is_hovered, &theme_colors))
    }
}

/// Session list window view (320px wide, shows/hides on demand)
struct SessionListView {
    state: Entity<SharedHudState>,
    /// Last known session count (for resize detection)
    last_session_count: usize,
    /// Track when each session was first seen (for slide-in animation)
    appeared_at: HashMap<String, Instant>,
    /// Track hover start time per row for icon swap animation
    icon_hover_at: HashMap<String, (Instant, bool)>,
    /// Track sessions being removed with slide-out animation
    /// Maps session_id -> (SessionInfo snapshot, removal start time)
    removing: HashMap<String, (SessionInfo, Instant)>,
    /// Cache of last known session info (for exit animation)
    session_cache: HashMap<String, SessionInfo>,
}

impl SessionListView {
    /// Render a session row with hover-based marquee scrolling and slide-in animation
    fn render_session_row(
        &mut self,
        session: &SessionInfo,
        tool_index: usize,
        fade_progress: f32,
        animation_start: Instant,
        theme_colors: &theme::ThemeColors,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let session_id = session.session_id.clone();

        // Track first appearance time for slide-in animation
        let appeared_at = *self
            .appeared_at
            .entry(session_id.clone())
            .or_insert_with(Instant::now);
        let (slide_opacity, slide_x_offset) = calculate_row_slide_in(appeared_at);

        let session_name = session
            .name
            .clone()
            .unwrap_or_else(|| extract_session_name(&session.cwd));

        let session_id_for_icon = session_id.clone();

        // Get icon swap hover state - pass None when no hover history exists
        let (icon_hover_start, icon_is_hovered) = self
            .icon_hover_at
            .get(&session_id)
            .copied()
            .map(|(start, hovered)| (Some(start), hovered))
            .unwrap_or((None, false));
        let (state_opacity, state_x, remove_opacity, remove_x) =
            calculate_icon_swap(icon_hover_start, icon_is_hovered);

        // State-based row opacity for visual hierarchy
        let row_opacity = match session.state {
            SessionState::Running => 1.0,
            SessionState::Attention => 1.0,
            SessionState::Waiting => 1.0,
            SessionState::Compacting => 0.9,
            SessionState::Idle => 0.7,
            SessionState::Stale => calculate_breathe_opacity(animation_start),
        };

        // Session ID for remove click handler
        let session_id_for_remove = session_id.clone();
        let state_for_remove = self.state.clone();

        // Check if remove icon is visible enough to be clickable
        let remove_clickable = remove_opacity > 0.5;

        div()
            .id(SharedString::from(format!("session-row-{}", session_id)))
            .relative() // For absolute positioning of remove overlay
            .opacity(row_opacity * slide_opacity) // Combine state opacity with slide-in
            .ml(px(slide_x_offset)) // Slide from left
            .on_hover(cx.listener(move |this, hovered: &bool, _window, _cx| {
                // Track hover timing for icon swap animation
                let now = Instant::now();
                if *hovered {
                    // Starting hover
                    this.icon_hover_at.insert(session_id_for_icon.clone(), (now, true));
                } else {
                    // Ending hover - keep the timestamp but mark as not hovered for reverse animation
                    this.icon_hover_at.insert(session_id_for_icon.clone(), (now, false));
                }
            }))
            .child(session_list::render_row_content(
                session,
                &session_name,
                &session_list::RowRenderArgs {
                    tool_index,
                    fade_progress,
                    animation_start,
                    state_opacity,
                    state_x,
                    remove_opacity,
                    remove_x,
                    theme: theme_colors,
                },
            ))
            // Remove button overlay - positioned over the state icon area
            // Only clickable when remove icon is visible (hover state)
            .when(remove_clickable, |this| {
                this.child(
                    div()
                        .id(SharedString::from(format!("remove-btn-{}", session_id_for_remove)))
                        .absolute()
                        .left(px(14.0)) // Row left padding
                        .top(px(10.0)) // Row top padding
                        .w(px(14.0)) // State icon width
                        .h(px(14.0)) // State icon height
                        .cursor(gpui::CursorStyle::PointingHand)
                        .on_click(move |_event, _window, app| {
                            // Remove session from registry
                            state_for_remove.update(app, |state, _cx| {
                                if let Ok(mut registry) = state.registry.lock() {
                                    registry.remove_session(&session_id_for_remove);
                                }
                                state.registry_dirty.store(true, Ordering::Relaxed);
                            });
                        }),
                )
            })
    }

    /// Render a session row that is being removed (slide-out animation)
    fn render_removing_row(
        &self,
        session: &SessionInfo,
        removed_at: Instant,
        tool_index: usize,
        fade_progress: f32,
        animation_start: Instant,
        theme_colors: &theme::ThemeColors,
    ) -> gpui::Div {
        let _session_id = session.session_id.clone();
        let session_name = session
            .name
            .clone()
            .unwrap_or_else(|| extract_session_name(&session.cwd));

        // Calculate slide-out animation
        let (slide_opacity, slide_x_offset, _) = calculate_row_slide_out(removed_at);

        // State-based row opacity
        let row_opacity = match session.state {
            SessionState::Running => 1.0,
            SessionState::Attention => 1.0,
            SessionState::Waiting => 1.0,
            SessionState::Compacting => 0.9,
            SessionState::Idle => 0.7,
            SessionState::Stale => calculate_breathe_opacity(animation_start),
        };

        div()
            .opacity(row_opacity * slide_opacity)
            .ml(px(slide_x_offset)) // Slide right
            .child(session_list::render_row_content(
                session,
                &session_name,
                &session_list::RowRenderArgs {
                    tool_index,
                    fade_progress,
                    animation_start,
                    state_opacity: 1.0,   // State icon visible
                    state_x: 0.0,         // No x offset
                    remove_opacity: 0.0,  // Remove icon hidden
                    remove_x: -16.0,      // Remove icon off-screen
                    theme: theme_colors,
                },
            ))
    }
}

impl Render for SessionListView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check visibility FIRST - if not visible, return empty (window closing handled by click handler)
        let is_visible = self.state.read(cx).session_list_visible;
        if !is_visible {
            window.resize(size(px(1.0), px(1.0)));
            return div().size_full().opacity(0.0).into_any_element();
        }

        // Request continuous animation frames (only for visible windows)
        window.request_animation_frame();

        // Update system appearance and save current position (for reopening at same location)
        let appearance = window.appearance();
        let current_origin = window.bounds().origin;
        self.state.update(cx, |state, _cx| {
            state.update_system_appearance(appearance);
            state.session_list_origin = current_origin;
        });

        let hud_state = self.state.read(cx);
        let sessions = &hud_state.sessions;
        let total_count = sessions.len();
        let animation_start = hud_state.animation_start;
        let theme_colors = hud_state.theme_colors();

        // Resize window if session count changed
        // Include removing sessions in count to prevent height jump during exit animation
        let visible_count = (total_count + self.removing.len()).min(MAX_SESSIONS);
        if visible_count != self.last_session_count && visible_count > 0 {
            self.last_session_count = visible_count;
            let height = calculate_expanded_height(visible_count);
            window.resize(size(px(EXPANDED_WIDTH), px(height)));
        }

        // Handle empty sessions case - show placeholder
        if sessions.is_empty() {
            let height = calculate_expanded_height(1);
            window.resize(size(px(EXPANDED_WIDTH), px(height)));
            return div()
                .id("session-list-container")
                .size_full()
                .relative()
                .rounded(px(theme::WINDOW_RADIUS))
                .overflow_hidden()
                .bg(theme_colors.container_bg)
                .border_1()
                .border_color(theme_colors.border)
                .when(theme_colors.use_shadow, |this| this.shadow_md())
                .child(glass::render_container_highlight(
                    theme::WINDOW_RADIUS,
                    &theme_colors,
                ))
                .child(
                    div()
                        .relative()
                        .size_full()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .id("session-list-header")
                                .w_full()
                                .h(px(28.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .font_family("Maple Mono NF CN")
                                .text_size(px(11.0))
                                .font_weight(gpui::FontWeight::NORMAL)
                                .text_color(theme_colors.text_header)
                                .child("0 sessions".to_string()),
                        )
                        .child(
                            div()
                                .id("session-list-content")
                                .p(px(10.0))
                                .flex_1()
                                .rounded(px(theme::WINDOW_RADIUS))
                                .bg(theme_colors.content_bg)
                                .border_t_1()
                                .border_color(theme_colors.content_highlight)
                                .flex()
                                .items_center()
                                .justify_center()
                                .font_family("Maple Mono NF CN")
                                .text_size(px(11.0))
                                .text_color(theme_colors.text_secondary)
                                .child("No active sessions".to_string()),
                        ),
                )
                .into_any_element();
        }

        // Calculate animation state
        let (tool_index, fade_progress) =
            calculate_animation_state(animation_start, hud_state.animation_seed);

        let sessions_for_render: Vec<_> = sessions.to_vec();
        let session_count = total_count;
        let list_theme_colors = theme_colors;

        // Build current session IDs set
        let current_ids: std::collections::HashSet<_> = sessions_for_render
            .iter()
            .map(|s| s.session_id.clone())
            .collect();

        // Update session cache with current sessions (so we have info for exit animation)
        for session in &sessions_for_render {
            self.session_cache
                .insert(session.session_id.clone(), session.clone());
        }

        // Detect sessions that have been removed (were in appeared_at but not in current)
        // and add them to the removing list for exit animation
        for session_id in self.appeared_at.keys() {
            if !current_ids.contains(session_id) && !self.removing.contains_key(session_id) {
                // Get cached session info for exit animation
                if let Some(session_info) = self.session_cache.get(session_id).cloned() {
                    self.removing
                        .insert(session_id.clone(), (session_info, Instant::now()));
                }
            }
        }

        // Clean up completed exit animations and their cache entries
        let completed_removals: Vec<_> = self
            .removing
            .iter()
            .filter(|(_, (_, removed_at))| calculate_row_slide_out(*removed_at).2)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &completed_removals {
            self.removing.remove(id);
            self.session_cache.remove(id);
        }

        let session_list = uniform_list(
            "sessions",
            sessions_for_render.len(),
            cx.processor(move |this, range, _window, cx| {
                let mut items = Vec::new();
                for ix in range {
                    if let Some(session) = sessions_for_render.get(ix) {
                        items.push(this.render_session_row(
                            session,
                            tool_index,
                            fade_progress,
                            animation_start,
                            &list_theme_colors,
                            cx,
                        ));
                    }
                }
                items
            }),
        )
        .h_full();

        // Build removing session rows (exit animation)
        let removing_rows: Vec<_> = self
            .removing
            .iter()
            .map(|(_, (session, removed_at))| {
                self.render_removing_row(session, *removed_at, tool_index, fade_progress, animation_start, &theme_colors)
            })
            .collect();

        // Clean up appeared_at and icon_hover_at for sessions that no longer exist
        self.appeared_at.retain(|id, _| current_ids.contains(id));
        self.icon_hover_at.retain(|id, _| current_ids.contains(id));

        // Session list container with liquid glass effect
        div()
            .id("session-list-container")
            .size_full()
            .relative()
            .rounded(px(theme::WINDOW_RADIUS))
            .overflow_hidden()
            .bg(theme_colors.container_bg)
            .border_1()
            .border_color(theme_colors.border)
            .when(theme_colors.use_shadow, |this| this.shadow_md())
            // Top highlight for glass effect
            .child(glass::render_container_highlight(theme::WINDOW_RADIUS, &theme_colors))
            // Header + Content
            .child(
                div()
                    .relative()
                    .size_full()
                    .flex()
                    .flex_col()
                    // Header: "N sessions" text at top
                    // Note: Drag disabled due to gpui "window not found" error after move+close
                    .child(
                        div()
                            .id("session-list-header")
                            .w_full()
                            .h(px(28.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_family("Maple Mono NF CN")
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::NORMAL)
                            .text_color(theme_colors.text_header)
                            .child(format!(
                                "{} session{}",
                                session_count,
                                if session_count == 1 { "" } else { "s" }
                            )),
                    )
                    // Content: session rows below header
                    .child(
                        div()
                            .id("session-list-content")
                            .p(px(10.0))
                            .flex_1()
                            .rounded(px(theme::WINDOW_RADIUS))
                            .bg(theme_colors.content_bg)
                            .border_t_1()
                            .border_color(theme_colors.content_highlight)
                            .flex()
                            .flex_col()
                            .gap(px(ROW_GAP))
                            .overflow_y_scroll()
                            .scrollbar_width(px(0.0))
                            .child(session_list)
                            .children(removing_rows),
                    ),
            )
            .into_any_element()
    }
}

/// Open the session list window synchronously
///
/// Called when user clicks indicator to show session list.
/// Creates a new window positioned below the indicator's current position.
fn open_session_list_window_sync(app: &mut App, state: Entity<SharedHudState>) {
    // Use saved session list origin (persists across open/close cycles)
    // Initial value is calculated relative to indicator at startup
    let origin = state.read(app).session_list_origin;

    let initial_height = calculate_expanded_height(1);
    let list_bounds = Bounds {
        origin,
        size: size(px(EXPANDED_WIDTH), px(initial_height)),
    };

    let state_for_list = state.clone();
    let window_handle = app
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(list_bounds)),
                titlebar: None,
                focus: false,
                show: true,
                kind: WindowKind::PopUp,
                is_movable: true,
                is_resizable: false,
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_window, app| {
                app.new(|_cx| SessionListView {
                    state: state_for_list,
                    last_session_count: 0,
                    appeared_at: HashMap::new(),
                    icon_hover_at: HashMap::new(),
                    removing: HashMap::new(),
                    session_cache: HashMap::new(),
                })
            },
        )
        .ok();

    // Store window handle in shared state
    if let Some(handle) = window_handle {
        state.update(app, |state, _cx| {
            state.session_list_window = Some(handle);
        });
    }
}

/// Persist the current theme preference to config.json.
fn save_theme(style: theme::ThemeStyle) {
    let mut config = crate::config::load_config();
    config.theme = style.to_config_str().to_string();
    let _ = crate::config::save_config(&config);
}

/// Run the HUD application with two separate windows
///
/// This function blocks and runs the gpui event loop.
/// Call from main thread only.
pub fn run_hud(registry: Arc<Mutex<SessionRegistry>>, registry_dirty: Arc<AtomicBool>) {
    Application::new().with_assets(Assets).run(|app: &mut App| {
        // Load saved theme preference from config.json
        let saved_config = crate::config::load_config();
        let initial_theme = theme::ThemeStyle::from_config_str(&saved_config.theme);

        // Register embedded Maple Mono font for consistent marquee rendering
        let font_data = include_bytes!("../../assets/fonts/MapleMono-NF-CN-Regular.ttf");
        app.text_system()
            .add_fonts(vec![Cow::Borrowed(font_data.as_slice())])
            .expect("Failed to load Maple Mono font");

        // Register quit action handler
        app.on_action(|_: &Quit, cx: &mut App| {
            cx.quit();
        });

        // Set up application menu with theme submenu
        app.set_menus(vec![Menu {
            name: "Aura".into(),
            items: vec![
                MenuItem::submenu(Menu {
                    name: "Theme".into(),
                    items: vec![
                        MenuItem::action("System", SetThemeSystem),
                        MenuItem::separator(),
                        MenuItem::action("Liquid Dark", SetThemeLiquidDark),
                        MenuItem::action("Liquid Light", SetThemeLiquidLight),
                        MenuItem::separator(),
                        MenuItem::action("Solid Dark", SetThemeSolidDark),
                        MenuItem::action("Solid Light", SetThemeSolidLight),
                    ],
                }),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
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

        // Load saved indicator position, clamped to visible display bounds
        let saved_state = crate::config::load_state();
        let default_x = (screen_width - px(EXPANDED_WIDTH)) / 2.0 + px((EXPANDED_WIDTH - COLLAPSED_WIDTH) / 2.0);
        let default_y = px(30.0);
        let (indicator_x, indicator_y) = if let (Some(x), Some(y)) = (saved_state.indicator_x, saved_state.indicator_y) {
            let x = px(x as f32);
            let y = px(y as f32);
            let db = display_bounds;
            // Clamp so the indicator stays fully on-screen
            let min_x = db.origin.x;
            let max_x = db.origin.x + db.size.width - px(COLLAPSED_WIDTH);
            let min_y = db.origin.y;
            let max_y = db.origin.y + db.size.height - px(COLLAPSED_HEIGHT);
            if x >= min_x && x <= max_x && y >= min_y && y <= max_y {
                (x, y)
            } else {
                (default_x, default_y)
            }
        } else {
            (default_x, default_y)
        };

        // Calculate session list origin (below indicator)
        let session_list_origin = point(
            indicator_x - px((EXPANDED_WIDTH - COLLAPSED_WIDTH) / 2.0),
            indicator_y + px(COLLAPSED_HEIGHT + WINDOW_GAP),
        );

        // Detect initial system appearance
        let initial_system_is_dark = app
            .displays()
            .first()
            .map(|_| {
                // Default to dark on macOS since we can't query appearance without a window
                // The actual appearance will be detected when windows are created
                true
            })
            .unwrap_or(true);

        // Create shared state between both windows
        let shared_state = app.new(|_cx| SharedHudState {
            sessions: initial_sessions,
            animation_start: Instant::now(),
            animation_seed,
            registry,
            session_list_visible: false,
            session_list_window: None,
            session_list_origin,
            indicator_window: None, // Will be set after window creation
            theme_style: initial_theme,
            system_is_dark: initial_system_is_dark,
            registry_dirty,
        });

        // Register theme action handlers
        let state_for_system = shared_state.clone();
        app.on_action(move |_: &SetThemeSystem, cx: &mut App| {
            state_for_system.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::System;
                save_theme(state.theme_style);
            });
        });

        let state_for_liquid_dark = shared_state.clone();
        app.on_action(move |_: &SetThemeLiquidDark, cx: &mut App| {
            state_for_liquid_dark.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::LiquidDark;
                save_theme(state.theme_style);
            });
        });

        let state_for_liquid_light = shared_state.clone();
        app.on_action(move |_: &SetThemeLiquidLight, cx: &mut App| {
            state_for_liquid_light.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::LiquidLight;
                save_theme(state.theme_style);
            });
        });

        let state_for_solid_dark = shared_state.clone();
        app.on_action(move |_: &SetThemeSolidDark, cx: &mut App| {
            state_for_solid_dark.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::SolidDark;
                save_theme(state.theme_style);
            });
        });

        let state_for_solid_light = shared_state.clone();
        app.on_action(move |_: &SetThemeSolidLight, cx: &mut App| {
            state_for_solid_light.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::SolidLight;
                save_theme(state.theme_style);
            });
        });

        // Create indicator window (always visible, 36x36)
        let indicator_bounds = Bounds {
            origin: point(indicator_x, indicator_y),
            size: size(px(COLLAPSED_WIDTH), px(COLLAPSED_HEIGHT)),
        };

        let state_for_indicator = shared_state.clone();
        let indicator_handle = app
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(indicator_bounds)),
                    titlebar: None,
                    focus: false,
                    show: true,
                    kind: WindowKind::PopUp,
                    is_movable: true,
                    is_resizable: false,
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_window, app| {
                    app.new(|_cx| IndicatorView {
                        state: state_for_indicator,
                        is_hovered: false,
                        window_pos_at_mouse_down: None,
                    })
                },
            )
            .expect("Failed to open indicator window");

        // Store indicator window handle in shared state
        shared_state.update(app, |state, _cx| {
            state.indicator_window = Some(indicator_handle);
        });

        // Session list window is opened on demand when user clicks indicator
        // (see open_session_list_window function)

        // Keep shared state alive
        let _ = shared_state;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    fn make_session(id: &str, state: SessionState) -> SessionInfo {
        SessionInfo {
            session_id: id.to_string(),
            cwd: "/test/project".to_string(),
            state,
            running_tools: vec![],
            name: None,
            stopped_at: None,
            stale_at: None,
            permission_tool: None,
            recent_activity: vec![],
        }
    }

    // --- 2.1: SharedHudState session refresh ---

    #[gpui::test]
    async fn shared_state_refresh_from_registry(cx: &mut TestAppContext) {
        let registry = Arc::new(Mutex::new(SessionRegistry::new()));
        let registry_dirty = Arc::new(AtomicBool::new(false));

        // Seed registry with a session
        {
            let mut reg = registry.lock().unwrap();
            reg.process_event(crate::AgentEvent::SessionStarted {
                session_id: "s1".into(),
                agent: crate::AgentType::ClaudeCode,
                cwd: "/test/project".into(),
            });
        }

        let state = cx.new(|_cx| {
            let mut s = SharedHudState::new_for_test(vec![]);
            s.registry = registry.clone();
            s.registry_dirty = registry_dirty.clone();
            s
        });

        // Initially empty
        state.read_with(cx, |s, _| {
            assert_eq!(s.sessions.len(), 0);
        });

        // After refresh
        state.update(cx, |s, _| {
            s.refresh_from_registry();
        });

        state.read_with(cx, |s, _| {
            assert_eq!(s.sessions.len(), 1);
            assert_eq!(s.sessions[0].session_id, "s1");
        });
    }

    #[gpui::test]
    async fn shared_state_starts_with_list_hidden(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| SharedHudState::new_for_test(vec![]));

        state.read_with(cx, |s, _| {
            assert!(!s.session_list_visible);
            assert!(s.session_list_window.is_none());
        });
    }

    // --- 2.2: Theme cycling ---

    #[gpui::test]
    async fn theme_cycle_system_to_liquid_dark(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| SharedHudState::new_for_test(vec![]));

        state.read_with(cx, |s, _| {
            assert_eq!(s.theme_style, theme::ThemeStyle::System);
        });

        state.update(cx, |s, _| {
            s.theme_style = s.theme_style.next();
        });

        state.read_with(cx, |s, _| {
            assert_eq!(s.theme_style, theme::ThemeStyle::LiquidDark);
        });
    }

    #[gpui::test]
    async fn theme_cycle_full_round(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| SharedHudState::new_for_test(vec![]));

        let expected = [
            theme::ThemeStyle::LiquidDark,
            theme::ThemeStyle::LiquidLight,
            theme::ThemeStyle::SolidDark,
            theme::ThemeStyle::SolidLight,
            theme::ThemeStyle::System, // wraps back
        ];

        for &exp in &expected {
            state.update(cx, |s, _| {
                s.theme_style = s.theme_style.next();
            });
            state.read_with(cx, |s, _| {
                assert_eq!(s.theme_style, exp);
            });
        }
    }

    // --- 2.3: Theme colors resolve correctly ---

    #[gpui::test]
    async fn theme_colors_dark_system(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| {
            let mut s = SharedHudState::new_for_test(vec![]);
            s.system_is_dark = true;
            s
        });

        state.read_with(cx, |s, _| {
            let colors = s.theme_colors();
            // Dark system should use shadow (liquid dark)
            assert!(colors.use_shadow);
        });
    }

    #[gpui::test]
    async fn theme_colors_light_system(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| {
            let mut s = SharedHudState::new_for_test(vec![]);
            s.system_is_dark = false;
            s
        });

        state.read_with(cx, |s, _| {
            let colors = s.theme_colors();
            // Light system should use shadow (liquid light)
            assert!(colors.use_shadow);
        });
    }

    #[gpui::test]
    async fn theme_colors_solid_uses_shadow(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| {
            let mut s = SharedHudState::new_for_test(vec![]);
            s.theme_style = theme::ThemeStyle::SolidDark;
            s
        });

        state.read_with(cx, |s, _| {
            let colors = s.theme_colors();
            // All themes use shadow (implemented in Phase 3.1)
            assert!(colors.use_shadow);
        });
    }

    // --- 2.4: Right-click cycles theme without toggling session list ---

    #[gpui::test]
    async fn right_click_cycles_theme(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| SharedHudState::new_for_test(vec![]));

        state.read_with(cx, |s, _| {
            assert_eq!(s.theme_style, theme::ThemeStyle::System);
        });

        // Simulate what the right-click handler does
        state.update(cx, |s, _| {
            s.theme_style = s.theme_style.next();
        });

        state.read_with(cx, |s, _| {
            assert_eq!(s.theme_style, theme::ThemeStyle::LiquidDark);
        });
    }

    #[gpui::test]
    async fn right_click_does_not_toggle_session_list(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| SharedHudState::new_for_test(vec![]));

        state.read_with(cx, |s, _| {
            assert!(!s.session_list_visible);
        });

        // Simulate right-click: only cycles theme, does not touch session list
        state.update(cx, |s, _| {
            s.theme_style = s.theme_style.next();
        });

        state.read_with(cx, |s, _| {
            assert!(!s.session_list_visible);
            assert_eq!(s.theme_style, theme::ThemeStyle::LiquidDark);
        });
    }

    // --- 2.5: Session list visibility toggle ---

    #[gpui::test]
    async fn toggle_session_list_visibility(cx: &mut TestAppContext) {
        let sessions = vec![make_session("s1", SessionState::Running)];
        let state = cx.new(|_cx| SharedHudState::new_for_test(sessions));

        // Initially hidden
        state.read_with(cx, |s, _| {
            assert!(!s.session_list_visible);
        });

        // Toggle on
        state.update(cx, |s, _| {
            s.session_list_visible = true;
        });

        state.read_with(cx, |s, _| {
            assert!(s.session_list_visible);
        });

        // Toggle off
        state.update(cx, |s, _| {
            s.session_list_visible = false;
        });

        state.read_with(cx, |s, _| {
            assert!(!s.session_list_visible);
        });
    }

    // --- 2.5: Registry dirty flag ---

    #[gpui::test]
    async fn registry_dirty_flag_swap(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| SharedHudState::new_for_test(vec![]));

        // Set dirty
        state.read_with(cx, |s, _| {
            s.registry_dirty.store(true, Ordering::Relaxed);
        });

        // Swap should return true and clear it
        state.update(cx, |s, _| {
            let was_dirty = s.registry_dirty.swap(false, Ordering::Relaxed);
            assert!(was_dirty);
        });

        // Should now be clean
        state.read_with(cx, |s, _| {
            assert!(!s.registry_dirty.load(Ordering::Relaxed));
        });
    }

    // --- 2.6: Indicator view window creation ---

    #[gpui::test]
    async fn indicator_view_initial_state(cx: &mut TestAppContext) {
        let sessions = vec![make_session("s1", SessionState::Running)];
        let state = cx.new(|_cx| SharedHudState::new_for_test(sessions));

        let window = cx.add_window(|_window, _cx| IndicatorView {
            state: state.clone(),
            is_hovered: false,
            window_pos_at_mouse_down: None,
        });

        let view = window.root(cx).unwrap();

        view.read_with(cx, |v, _| {
            assert!(!v.is_hovered);
            assert!(v.window_pos_at_mouse_down.is_none());
        });
    }

    // --- 2.7: Session list view creation ---

    #[gpui::test]
    async fn session_list_view_initial_state(cx: &mut TestAppContext) {
        let sessions = vec![make_session("s1", SessionState::Running)];
        let state = cx.new(|_cx| SharedHudState::new_for_test(sessions));

        let window = cx.add_window(|_window, _cx| SessionListView {
            state: state.clone(),
            last_session_count: 0,
            appeared_at: HashMap::new(),
            icon_hover_at: HashMap::new(),
            removing: HashMap::new(),
            session_cache: HashMap::new(),
        });

        let view = window.root(cx).unwrap();

        view.read_with(cx, |v, _| {
            assert_eq!(v.last_session_count, 0);
            assert!(v.appeared_at.is_empty());
            assert!(v.removing.is_empty());
        });
    }

    // --- 2.8: Session removal from registry via state ---

    #[gpui::test]
    async fn session_removal_from_registry(cx: &mut TestAppContext) {
        let registry = Arc::new(Mutex::new(SessionRegistry::new()));

        // Add two sessions
        {
            let mut reg = registry.lock().unwrap();
            reg.process_event(crate::AgentEvent::SessionStarted {
                session_id: "s1".into(),
                agent: crate::AgentType::ClaudeCode,
                cwd: "/test/a".into(),
            });
            reg.process_event(crate::AgentEvent::SessionStarted {
                session_id: "s2".into(),
                agent: crate::AgentType::ClaudeCode,
                cwd: "/test/b".into(),
            });
        }

        let state = cx.new(|_cx| {
            let mut s = SharedHudState::new_for_test(vec![]);
            s.registry = registry.clone();
            s
        });

        // Refresh to populate
        state.update(cx, |s, _| {
            s.refresh_from_registry();
        });

        state.read_with(cx, |s, _| {
            assert_eq!(s.sessions.len(), 2);
        });

        // Remove one session
        state.update(cx, |s, _| {
            if let Ok(mut reg) = s.registry.lock() {
                reg.remove_session("s1");
            }
            s.refresh_from_registry();
        });

        state.read_with(cx, |s, _| {
            assert_eq!(s.sessions.len(), 1);
            assert_eq!(s.sessions[0].session_id, "s2");
        });
    }

    // --- 2.9: System appearance detection ---

    #[gpui::test]
    async fn system_appearance_dark_detection(cx: &mut TestAppContext) {
        let state = cx.new(|_cx| {
            let mut s = SharedHudState::new_for_test(vec![]);
            s.system_is_dark = false;
            s
        });

        state.read_with(cx, |s, _| {
            assert!(!s.system_is_dark);
        });

        state.update(cx, |s, _| {
            s.system_is_dark = true;
        });

        state.read_with(cx, |s, _| {
            assert!(s.system_is_dark);
            let colors = s.theme_colors();
            // System + dark → liquid dark → uses shadow
            assert!(colors.use_shadow);
        });
    }

}
