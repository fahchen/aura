//! Shared helpers for visual regression tests
//!
//! This module provides common utilities used by both the `#[test]` module
//! and the standalone visual test runner binary.

use aura_common::{RunningTool, SessionInfo, SessionState};
use gpui::{AnyWindowHandle, VisualTestAppContext};
use image::RgbaImage;
use std::path::PathBuf;

/// Test result for reporting
pub enum TestResult {
    Pass(String),
    Fail(String, String),
}

/// Get the baselines directory path
pub fn baselines_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/baselines")
}

/// Capture screenshot and compare/save against baseline
///
/// - If `UPDATE_BASELINES=1`: saves screenshot as new baseline
/// - Otherwise: compares with existing baseline using pixel comparison
pub fn capture_and_save(
    cx: &mut VisualTestAppContext,
    window: AnyWindowHandle,
    test_name: &str,
) -> anyhow::Result<()> {
    // Let the UI settle
    cx.run_until_parked();

    // Capture screenshot
    let screenshot = cx.capture_screenshot(window)?;

    let baselines = baselines_dir();
    let baseline_path = baselines.join(format!("{}.png", test_name));

    // Check if we should update baselines
    if std::env::var("UPDATE_BASELINES").is_ok() {
        // Create baselines directory if it doesn't exist
        std::fs::create_dir_all(&baselines)?;
        screenshot.save(&baseline_path)?;
        println!("Updated baseline: {}", baseline_path.display());
        return Ok(());
    }

    // Compare with existing baseline
    if !baseline_path.exists() {
        // Save actual image for inspection
        let actual_path = baselines.join(format!("{}_actual.png", test_name));
        std::fs::create_dir_all(&baselines)?;
        screenshot.save(&actual_path)?;
        anyhow::bail!(
            "Baseline not found: {}. Run with UPDATE_BASELINES=1 to create it. Actual saved to: {}",
            baseline_path.display(),
            actual_path.display()
        );
    }

    let baseline = image::open(&baseline_path)?.to_rgba8();
    compare_images(&screenshot, &baseline, test_name)
}

/// Compare two images with tolerance, save diff on mismatch
pub fn compare_images(
    actual: &RgbaImage,
    expected: &RgbaImage,
    test_name: &str,
) -> anyhow::Result<()> {
    // Tolerance per channel (0-255)
    const TOLERANCE: u8 = 2;

    // Check dimensions
    if actual.dimensions() != expected.dimensions() {
        let baselines = baselines_dir();
        let actual_path = baselines.join(format!("{}_actual.png", test_name));
        actual.save(&actual_path)?;
        anyhow::bail!(
            "Image dimensions mismatch: actual {:?} vs expected {:?}. Actual saved to: {}",
            actual.dimensions(),
            expected.dimensions(),
            actual_path.display()
        );
    }

    let (width, height) = actual.dimensions();
    let mut diff_count = 0u64;
    let mut diff_image = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let actual_pixel = actual.get_pixel(x, y);
            let expected_pixel = expected.get_pixel(x, y);

            let mut is_diff = false;
            for i in 0..4 {
                let diff = (actual_pixel[i] as i16 - expected_pixel[i] as i16).unsigned_abs();
                if diff > TOLERANCE as u16 {
                    is_diff = true;
                    break;
                }
            }

            if is_diff {
                diff_count += 1;
                // Mark differences in red
                diff_image.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            } else {
                // Keep original pixel (dimmed)
                let dimmed = image::Rgba([
                    actual_pixel[0] / 3,
                    actual_pixel[1] / 3,
                    actual_pixel[2] / 3,
                    actual_pixel[3],
                ]);
                diff_image.put_pixel(x, y, dimmed);
            }
        }
    }

    if diff_count > 0 {
        let baselines = baselines_dir();
        std::fs::create_dir_all(&baselines)?;

        let actual_path = baselines.join(format!("{}_actual.png", test_name));
        let diff_path = baselines.join(format!("{}_diff.png", test_name));

        actual.save(&actual_path)?;
        diff_image.save(&diff_path)?;

        let total_pixels = (width * height) as f64;
        let diff_percent = (diff_count as f64 / total_pixels) * 100.0;

        anyhow::bail!(
            "Visual regression: {} pixels differ ({:.2}%).\nActual: {}\nDiff: {}",
            diff_count,
            diff_percent,
            actual_path.display(),
            diff_path.display()
        );
    }

    Ok(())
}

/// Create mock sessions for testing
pub fn mock_sessions() -> Vec<SessionInfo> {
    vec![
        SessionInfo {
            session_id: "test-session-1".to_string(),
            cwd: "/Users/test/projects/aura".to_string(),
            state: SessionState::Running,
            running_tools: vec![RunningTool {
                tool_id: "tool-1".to_string(),
                tool_name: "mcp__memory__memory_search".to_string(),
                tool_label: Some("memory_search".to_string()),
            }],
            name: Some("aura".to_string()),
            stopped_at: None,
            stale_at: None,
            permission_tool: None,
            git_branch: None,
            message_count: None,
            last_prompt_preview: None,
        },
        SessionInfo {
            session_id: "test-session-2".to_string(),
            cwd: "/Users/test/projects/other".to_string(),
            state: SessionState::Attention,
            running_tools: vec![],
            name: Some("other-project".to_string()),
            stopped_at: None,
            stale_at: None,
            permission_tool: Some("Bash".to_string()),
            git_branch: None,
            message_count: None,
            last_prompt_preview: None,
        },
    ]
}

/// Create a session with Attention state
pub fn attention_session() -> SessionInfo {
    SessionInfo {
        session_id: "attention-session".to_string(),
        cwd: "/Users/test/projects/attention".to_string(),
        state: SessionState::Attention,
        running_tools: vec![],
        name: Some("attention".to_string()),
        stopped_at: None,
        stale_at: None,
        permission_tool: Some("Bash".to_string()),
        git_branch: None,
        message_count: None,
        last_prompt_preview: None,
    }
}

/// Create a session with Running state and tools
pub fn running_session() -> SessionInfo {
    SessionInfo {
        session_id: "running-session".to_string(),
        cwd: "/Users/test/projects/running".to_string(),
        state: SessionState::Running,
        running_tools: vec![RunningTool {
            tool_id: "tool-1".to_string(),
            tool_name: "Read".to_string(),
            tool_label: Some("/path/to/file.rs".to_string()),
        }],
        name: Some("running".to_string()),
        stopped_at: None,
        stale_at: None,
        permission_tool: None,
        git_branch: None,
        message_count: None,
        last_prompt_preview: None,
    }
}

/// Create a session with Idle state (no tools)
pub fn idle_session() -> SessionInfo {
    SessionInfo {
        session_id: "idle-session".to_string(),
        cwd: "/Users/test/projects/idle".to_string(),
        state: SessionState::Idle,
        running_tools: vec![],
        name: Some("idle".to_string()),
        stopped_at: Some(1705500000), // Fixed timestamp for deterministic display
        stale_at: None,
        permission_tool: None,
        git_branch: None,
        message_count: None,
        last_prompt_preview: None,
    }
}
