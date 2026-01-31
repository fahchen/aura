//! Visual regression tests for HUD UI components
//!
//! These tests use `VisualTestAppContext` for real Metal rendering with screenshot capture.
//! Run with: `cargo test -p aura-daemon -- --ignored --test-threads=1`
//!
//! To update baselines: `UPDATE_BASELINES=1 cargo test -p aura-daemon -- --ignored --test-threads=1`

#[cfg(test)]
mod tests {
    use crate::ui::{
        assets::Assets, icons::colors, indicator, session_list,
        theme,
        visual_test_helpers::{
            attention_session, capture_and_save, idle_session, mock_sessions, running_session,
        },
    };
    use aura_common::{RunningTool, SessionInfo};
    use gpui::{div, px, size, AppContext as _, IntoElement, ParentElement, Render, Styled, VisualTestAppContext, Window};
    use std::sync::Arc;
    use std::time::Instant;

    /// Test view that wraps a single component for rendering
    struct TestView {
        content:
            Box<dyn Fn(&mut Window, &mut gpui::Context<Self>) -> gpui::AnyElement + Send + Sync>,
    }

    impl Render for TestView {
        fn render(
            &mut self,
            window: &mut Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            (self.content)(window, cx)
        }
    }

    // ============================================================================
    // Visual Tests
    // ============================================================================
    // Note: All tests are ignored by default because they require:
    // 1. macOS main thread (Metal rendering)
    // 2. Sequential execution (--test-threads=1)
    //
    // Run with: cargo test -p aura-daemon -- --ignored --test-threads=1
    // Update baselines: UPDATE_BASELINES=1 cargo test -p aura-daemon -- --ignored --test-threads=1

    /// Test indicator rendering with no sessions (Panda icon, dim state)
    #[test]
    #[ignore]
    fn test_indicator_no_sessions() {
        let mut cx = VisualTestAppContext::with_asset_source(Arc::new(Assets));
        let animation_start = Instant::now();
        let sessions: Vec<SessionInfo> = vec![];

        let window = cx
            .open_offscreen_window(size(px(36.0), px(36.0)), |_window, app| {
                app.new(|_cx| TestView {
                    content: Box::new(move |_window, _cx| {
                        indicator::render(&sessions, animation_start, false).into_any_element()
                    }),
                })
            })
            .expect("Failed to open window");

        capture_and_save(&mut cx, window.into(), "indicator_no_sessions")
            .expect("Screenshot comparison failed");
    }

    /// Test indicator rendering with running sessions (active state)
    #[test]
    #[ignore]
    fn test_indicator_running() {
        let mut cx = VisualTestAppContext::with_asset_source(Arc::new(Assets));
        let animation_start = Instant::now();
        let sessions = vec![running_session()];

        let window = cx
            .open_offscreen_window(size(px(36.0), px(36.0)), |_window, app| {
                app.new(|_cx| TestView {
                    content: Box::new(move |_window, _cx| {
                        indicator::render(&sessions, animation_start, false).into_any_element()
                    }),
                })
            })
            .expect("Failed to open window");

        capture_and_save(&mut cx, window.into(), "indicator_running")
            .expect("Screenshot comparison failed");
    }

    /// Test indicator rendering with attention state (Bell icon, shaking)
    #[test]
    #[ignore]
    fn test_indicator_attention() {
        let mut cx = VisualTestAppContext::with_asset_source(Arc::new(Assets));
        let animation_start = Instant::now();
        let sessions = vec![attention_session()];

        let window = cx
            .open_offscreen_window(size(px(36.0), px(36.0)), |_window, app| {
                app.new(|_cx| TestView {
                    content: Box::new(move |_window, _cx| {
                        indicator::render(&sessions, animation_start, false).into_any_element()
                    }),
                })
            })
            .expect("Failed to open window");

        capture_and_save(&mut cx, window.into(), "indicator_attention")
            .expect("Screenshot comparison failed");
    }

    /// Test single session row rendering
    #[test]
    #[ignore]
    fn test_session_row() {
        let mut cx = VisualTestAppContext::with_asset_source(Arc::new(Assets));
        let animation_start = Instant::now();
        let session = running_session();
        let theme = theme::ThemeColors::liquid_dark();

        let window = cx
            .open_offscreen_window(size(px(320.0), px(56.0)), |_window, app| {
                app.new(|_cx| TestView {
                    content: Box::new(move |_window, _cx| {
                        session_list::render_row_content(
                            &session,
                            "running",
                            &session_list::RowRenderArgs {
                                tool_index: 0,
                                fade_progress: 0.0,
                                animation_start,
                                state_opacity: 1.0,
                                state_x: 0.0,
                                remove_opacity: 0.0,
                                remove_x: -16.0,
                                theme: &theme,
                            },
                        )
                        .into_any_element()
                    }),
                })
            })
            .expect("Failed to open window");

        capture_and_save(&mut cx, window.into(), "session_row")
            .expect("Screenshot comparison failed");
    }

    /// Test MCP tool display format (server:function format)
    #[test]
    #[ignore]
    fn test_mcp_tool_display() {
        let mut cx = VisualTestAppContext::with_asset_source(Arc::new(Assets));
        let tool = RunningTool {
            tool_id: "test-tool".to_string(),
            tool_name: "mcp__memory__memory_search".to_string(),
            tool_label: Some("memory_search".to_string()),
        };

        let window = cx
            .open_offscreen_window(size(px(200.0), px(24.0)), |_window, app| {
                app.new(|_cx| TestView {
                    content: Box::new(move |_window, _cx| {
                        session_list::render_tool_with_icon(&tool).into_any_element()
                    }),
                })
            })
            .expect("Failed to open window");

        capture_and_save(&mut cx, window.into(), "mcp_tool_display")
            .expect("Screenshot comparison failed");
    }

    /// Test indicator hover state (enhanced visual effect)
    #[test]
    #[ignore]
    fn test_indicator_hovered() {
        let mut cx = VisualTestAppContext::with_asset_source(Arc::new(Assets));
        let animation_start = Instant::now();
        let sessions = mock_sessions();

        let window = cx
            .open_offscreen_window(size(px(36.0), px(36.0)), |_window, app| {
                app.new(|_cx| TestView {
                    content: Box::new(move |_window, _cx| {
                        // Render with hover state
                        indicator::render(&sessions, animation_start, true).into_any_element()
                    }),
                })
            })
            .expect("Failed to open window");

        capture_and_save(&mut cx, window.into(), "indicator_hovered")
            .expect("Screenshot comparison failed");
    }

    /// Test session row with Attention state (permission needed)
    #[test]
    #[ignore]
    fn test_session_row_attention() {
        let mut cx = VisualTestAppContext::with_asset_source(Arc::new(Assets));
        let animation_start = Instant::now();
        let session = attention_session();
        let theme = theme::ThemeColors::liquid_dark();

        let window = cx
            .open_offscreen_window(size(px(320.0), px(56.0)), |_window, app| {
                app.new(|_cx| TestView {
                    content: Box::new(move |_window, _cx| {
                        session_list::render_row_content(
                            &session,
                            "attention",
                            &session_list::RowRenderArgs {
                                tool_index: 0,
                                fade_progress: 0.0,
                                animation_start,
                                state_opacity: 1.0,
                                state_x: 0.0,
                                remove_opacity: 0.0,
                                remove_x: -16.0,
                                theme: &theme,
                            },
                        )
                        .into_any_element()
                    }),
                })
            })
            .expect("Failed to open window");

        capture_and_save(&mut cx, window.into(), "session_row_attention")
            .expect("Screenshot comparison failed");
    }

    /// Test complete session list window with multiple sessions
    #[test]
    #[ignore]
    fn test_session_list_window() {
        let mut cx = VisualTestAppContext::with_asset_source(Arc::new(Assets));
        let animation_start = Instant::now();
        let theme = theme::ThemeColors::liquid_dark();

        // Create 3 sessions with different states
        let sessions = vec![running_session(), attention_session(), idle_session()];

        // Calculate window height: (row_height + gap) * num_sessions + header_height + padding
        // Row height: 56px, gap: 4px, header: 28px, padding: 12px top + 12px bottom
        let num_sessions = sessions.len();
        let window_height = (56.0 + 4.0) * num_sessions as f32 + 52.0;

        let window = cx
            .open_offscreen_window(size(px(320.0), px(window_height)), |_window, app| {
                app.new(|_cx| TestView {
                    content: Box::new(move |_window, _cx| {
                        // Build the session list window layout
                        div()
                            .w(px(320.0))
                            .h_full()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .p(px(12.0))
                            .rounded(px(16.0))
                            .bg(colors::CONTAINER_BG)
                            // Header: "3 sessions"
                            .child(
                                div()
                                    .w_full()
                                    .h(px(28.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .font_family("Maple Mono NF CN")
                                    .text_size(px(12.0))
                                    .text_color(colors::TEXT_HEADER)
                                    .child(format!("{} sessions", sessions.len())),
                            )
                            // Session rows
                            .child(session_list::render_row_content(
                                &sessions[0],
                                sessions[0].name.as_deref().unwrap_or("running"),
                                &session_list::RowRenderArgs {
                                    tool_index: 0,
                                    fade_progress: 0.0,
                                    animation_start,
                                    state_opacity: 1.0,
                                    state_x: 0.0,
                                    remove_opacity: 0.0,
                                    remove_x: -16.0,
                                    theme: &theme,
                                },
                            ))
                            .child(session_list::render_row_content(
                                &sessions[1],
                                sessions[1].name.as_deref().unwrap_or("attention"),
                                &session_list::RowRenderArgs {
                                    tool_index: 0,
                                    fade_progress: 0.0,
                                    animation_start,
                                    state_opacity: 1.0,
                                    state_x: 0.0,
                                    remove_opacity: 0.0,
                                    remove_x: -16.0,
                                    theme: &theme,
                                },
                            ))
                            .child(session_list::render_row_content(
                                &sessions[2],
                                sessions[2].name.as_deref().unwrap_or("idle"),
                                &session_list::RowRenderArgs {
                                    tool_index: 0,
                                    fade_progress: 0.0,
                                    animation_start,
                                    state_opacity: 1.0,
                                    state_x: 0.0,
                                    remove_opacity: 0.0,
                                    remove_x: -16.0,
                                    theme: &theme,
                                },
                            ))
                            .into_any_element()
                    }),
                })
            })
            .expect("Failed to open window");

        capture_and_save(&mut cx, window.into(), "session_list_window")
            .expect("Screenshot comparison failed");
    }
}
