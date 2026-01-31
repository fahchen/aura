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
pub mod assets;
mod glass;
pub mod icons;
pub mod indicator;
pub mod session_list;
pub mod theme;

// Visual tests disabled - needs update for gpui API changes
// #[cfg(test)]
// mod visual_tests;

/// Visual test helpers - available when `visual-tests` feature is enabled
#[cfg(feature = "visual-tests")]
pub mod visual_test_helpers;

use animation::{
    calculate_animation_state, calculate_breathe_opacity, calculate_icon_swap,
    calculate_row_slide_in, calculate_row_slide_out,
};
use assets::Assets;
use aura_common::{SessionInfo, SessionState};
use crate::registry::SessionRegistry;
use gpui::{
    actions, div, point, px, size, App, AppContext, Application, Bounds, Context, Entity,
    InteractiveElement, IntoElement, Menu, MenuItem, ParentElement, Pixels, Point, Render,
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
use std::sync::{Arc, Mutex};
use std::time::Instant;


// Define application actions
actions!(aura, [Quit, SetThemeSystem, SetThemeLiquidDark, SetThemeLiquidLight, SetThemeSolidDark, SetThemeSolidLight]);

/// Gap between indicator and session list windows
const WINDOW_GAP: f32 = 4.0;


/// Shared HUD state between indicator and session list windows
struct SharedHudState {
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
}

/// Grace period to keep showing idle sessions (5 minutes)
const IDLE_GRACE_PERIOD_SECS: u64 = 300;

impl SharedHudState {
    /// Refresh sessions from registry
    /// - Always shows active sessions (Running, Attention, Waiting, Compacting)
    /// - Shows Idle sessions for a grace period after they stop
    /// - Hides Stale sessions
    fn refresh_from_registry(&mut self) {
        if let Ok(registry) = self.registry.lock() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            self.sessions = registry
                .get_all()
                .into_iter()
                .filter(|s| match s.state {
                    SessionState::Stale => false,
                    SessionState::Idle => {
                        // Show idle sessions within grace period
                        s.stopped_at
                            .map(|stopped| now.saturating_sub(stopped) < IDLE_GRACE_PERIOD_SECS)
                            .unwrap_or(false)
                    }
                    _ => true,
                })
                .collect();
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
            state.refresh_from_registry();
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
            .on_click({
                let state_for_click = self.state.clone();
                cx.listener(move |this, event: &gpui::ClickEvent, window, app| {
                    // Check if window moved (was dragged)
                    if let Some(start_pos) = this.window_pos_at_mouse_down.take() {
                        let current_pos = window.bounds().origin;
                        let threshold = px(5.0);
                        let dx = current_pos.x - start_pos.x;
                        let dy = current_pos.y - start_pos.y;
                        if dx > threshold || dx < -threshold || dy > threshold || dy < -threshold {
                            // Window was dragged, don't toggle
                            return;
                        }
                    }

                    match event.click_count() {
                        3 => {
                            // Triple-click: cycle theme
                            state_for_click.update(app, |state, _cx| {
                                state.theme_style = state.theme_style.next();
                            });
                        }
                        1 => {
                            // Single-click: toggle session list
                            let hud_state = state_for_click.read(app);
                            let has_sessions = !hud_state.sessions.is_empty();
                            let was_visible = hud_state.session_list_visible;
                            let window_handle = hud_state.session_list_window;

                            // Only allow opening if there are sessions
                            if !was_visible && !has_sessions {
                                return; // No sessions, don't open
                            }

                            let should_open = !was_visible && window_handle.is_none();
                            let should_close = was_visible && window_handle.is_some();

                            // NOTE: When the session list window is moved and then closed, gpui logs
                            // "window not found" errors. This is a known gpui limitation (v0.2.2):
                            // - When a window is moved, gpui registers internal callbacks for position tracking
                            // - When remove_window() is called, these callbacks still fire
                            // - The callbacks fail to find the window → "window not found" is logged
                            // - Error locations in gpui: app.rs:1388, app.rs:2201, window.rs:4725
                            // - This is benign: the window closes correctly, no functional impact
                            // - Fix requires changes to gpui's callback cleanup logic
                            if should_close {
                                // Remove window FIRST, before state update
                                let handle = window_handle.unwrap();
                                let _ = handle.update(app, |_view, window, _cx| {
                                    window.remove_window();
                                });

                                // Then update state (window is already gone)
                                state_for_click.update(app, |state, _cx| {
                                    state.session_list_visible = false;
                                    state.session_list_window = None;
                                });
                            } else if should_open {
                                state_for_click.update(app, |state, _cx| {
                                    state.session_list_visible = true;
                                });
                                open_session_list_window_sync(app, state_for_click.clone());
                            }
                        }
                        _ => {
                            // Double-click or other: ignore
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
            return div().size_full().into_any_element();
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
        let animation_start = hud_state.animation_start;
        let theme_colors = hud_state.theme_colors();

        // Resize window if session count changed
        // Include removing sessions in count to prevent height jump during exit animation
        let visible_count = (sessions.len() + self.removing.len()).min(MAX_SESSIONS);
        if visible_count != self.last_session_count && visible_count > 0 {
            self.last_session_count = visible_count;
            let height = calculate_expanded_height(visible_count);
            window.resize(size(px(EXPANDED_WIDTH), px(height)));
        }

        // Handle empty sessions case - close the window
        if sessions.is_empty() {
            // Save position and close window
            let current_origin = window.bounds().origin;
            self.state.update(cx, |state, _cx| {
                state.session_list_origin = current_origin;
                state.session_list_visible = false;
                state.session_list_window = None;
            });
            window.remove_window();
            return div().size_full().into_any_element();
        }

        // Calculate animation state
        let (tool_index, fade_progress) =
            calculate_animation_state(animation_start, hud_state.animation_seed);

        let sessions_for_render: Vec<_> = sessions.iter().take(MAX_SESSIONS).cloned().collect();
        let session_count = sessions_for_render.len();

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

        // Build session rows (needs mutable self for appeared_at tracking)
        let session_rows: Vec<_> = sessions_for_render
            .iter()
            .map(|session| {
                self.render_session_row(session, tool_index, fade_progress, animation_start, &theme_colors, cx)
            })
            .collect();

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
                            .p(px(10.0))
                            .flex_1()
                            .rounded(px(theme::WINDOW_RADIUS))
                            .bg(theme_colors.content_bg)
                            .border_t_1()
                            .border_color(theme_colors.content_highlight)
                            .flex()
                            .flex_col()
                            .gap(px(ROW_GAP))
                            .children(session_rows)
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
                window_background: WindowBackgroundAppearance::Transparent,
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

/// Run the HUD application with two separate windows
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

        // Position windows centered under notch
        let window_x = (screen_width - px(EXPANDED_WIDTH)) / 2.0;
        let window_y = px(30.0); // Just below menu bar

        // Calculate session list origin (below indicator)
        let session_list_origin = point(window_x, window_y + px(COLLAPSED_HEIGHT + WINDOW_GAP));

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
            theme_style: theme::ThemeStyle::System,
            system_is_dark: initial_system_is_dark,
        });

        // Register theme action handlers
        let state_for_system = shared_state.clone();
        app.on_action(move |_: &SetThemeSystem, cx: &mut App| {
            state_for_system.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::System;
            });
        });

        let state_for_liquid_dark = shared_state.clone();
        app.on_action(move |_: &SetThemeLiquidDark, cx: &mut App| {
            state_for_liquid_dark.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::LiquidDark;
            });
        });

        let state_for_liquid_light = shared_state.clone();
        app.on_action(move |_: &SetThemeLiquidLight, cx: &mut App| {
            state_for_liquid_light.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::LiquidLight;
            });
        });

        let state_for_solid_dark = shared_state.clone();
        app.on_action(move |_: &SetThemeSolidDark, cx: &mut App| {
            state_for_solid_dark.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::SolidDark;
            });
        });

        let state_for_solid_light = shared_state.clone();
        app.on_action(move |_: &SetThemeSolidLight, cx: &mut App| {
            state_for_solid_light.update(cx, |state, _cx| {
                state.theme_style = theme::ThemeStyle::SolidLight;
            });
        });

        // Create indicator window (always visible, 36x36)
        let indicator_bounds = Bounds {
            origin: point(window_x + px((EXPANDED_WIDTH - COLLAPSED_WIDTH) / 2.0), window_y),
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
                    window_background: WindowBackgroundAppearance::Transparent,
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
