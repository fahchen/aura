//! Animation utilities for tool cycling and text marquee

use std::time::Instant;

/// Animation timing constants
pub const CYCLE_DURATION_MIN_MS: u64 = 1500; // Min time showing each tool
pub const CYCLE_DURATION_MAX_MS: u64 = 2000; // Max time showing each tool
pub const FADE_DURATION_MS: u64 = 500; // Cross-fade duration


/// Simple hash function for deterministic pseudo-random per cycle
pub fn cycle_hash(cycle: u64, seed: u64) -> u64 {
    // Combine cycle and seed, then apply multiplicative hash
    let mut x = (cycle ^ seed).wrapping_mul(0x517cc1b727220a95);
    x ^= x >> 32;
    x
}

/// Get randomized cycle duration for a specific cycle number and seed
pub fn get_cycle_duration(cycle: u64, seed: u64) -> u64 {
    let hash = cycle_hash(cycle, seed);
    let range = CYCLE_DURATION_MAX_MS - CYCLE_DURATION_MIN_MS;
    CYCLE_DURATION_MIN_MS + (hash % (range + 1))
}

/// Calculate animation state based on elapsed time
/// Returns (tool_index, fade_progress)
pub fn calculate_animation_state(start_time: Instant, seed: u64) -> (usize, f32) {
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

/// Quadratic ease-in-out function
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Quadratic ease-out: decelerates to zero velocity
pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(2)
}

/// Shake animation constants
const SHAKE_PERIOD_MS: f32 = 150.0; // Oscillation period in milliseconds
const SHAKE_AMPLITUDE: f32 = 1.5; // Maximum horizontal displacement in pixels

/// Calculate horizontal shake offset for attention animation.
/// Returns x-offset in pixels oscillating between -amplitude and +amplitude.
pub fn calculate_shake_offset(start_time: Instant) -> f32 {
    let elapsed_ms = start_time.elapsed().as_millis() as f32;
    let phase = (elapsed_ms / SHAKE_PERIOD_MS) * std::f32::consts::TAU;
    phase.sin() * SHAKE_AMPLITUDE
}

/// Breathe animation constants
const BREATHE_CYCLE_MS: f32 = 4000.0; // Full cycle duration in milliseconds
const BREATHE_MIN_OPACITY: f32 = 0.3; // Minimum opacity
const BREATHE_MAX_OPACITY: f32 = 0.5; // Maximum opacity

/// Calculate breathe animation opacity for stale sessions.
/// Cycles between 0.3 and 0.5 over 4 seconds using sine wave.
pub fn calculate_breathe_opacity(start_time: Instant) -> f32 {
    let elapsed_ms = start_time.elapsed().as_millis() as f32;
    let t = (elapsed_ms % BREATHE_CYCLE_MS) / BREATHE_CYCLE_MS;
    let sine = (t * std::f32::consts::TAU).sin();
    // Map sine [-1, 1] to [0.3, 0.5]: center = 0.4, amplitude = 0.1
    let center = (BREATHE_MIN_OPACITY + BREATHE_MAX_OPACITY) / 2.0;
    let amplitude = (BREATHE_MAX_OPACITY - BREATHE_MIN_OPACITY) / 2.0;
    center + amplitude * sine
}


/// Row slide-in animation duration in milliseconds
pub const ROW_SLIDE_IN_MS: u64 = 350;

/// Calculate row slide-in animation state.
/// Returns (opacity, x_offset) for the slide-in animation.
/// - opacity: 0.0 → 1.0
/// - x_offset: -12.0 → 0.0
pub fn calculate_row_slide_in(appeared_at: Instant) -> (f32, f32) {
    let elapsed_ms = appeared_at.elapsed().as_millis() as u64;

    if elapsed_ms >= ROW_SLIDE_IN_MS {
        return (1.0, 0.0); // Animation complete
    }

    let progress = elapsed_ms as f32 / ROW_SLIDE_IN_MS as f32;
    let eased = ease_out(progress);

    let opacity = eased;
    let x_offset = -12.0 * (1.0 - eased);

    (opacity, x_offset)
}

/// Row slide-out animation duration in milliseconds
pub const ROW_SLIDE_OUT_MS: u64 = 300;

/// Calculate row slide-out animation state (when session is removed).
/// Returns (opacity, x_offset) for the slide-out animation.
/// - opacity: 1.0 → 0.0
/// - x_offset: 0.0 → 12.0 (slides right)
pub fn calculate_row_slide_out(removed_at: Instant) -> (f32, f32, bool) {
    let elapsed_ms = removed_at.elapsed().as_millis() as u64;

    if elapsed_ms >= ROW_SLIDE_OUT_MS {
        return (0.0, 12.0, true); // Animation complete, should be removed
    }

    let progress = elapsed_ms as f32 / ROW_SLIDE_OUT_MS as f32;
    // Use accelerate-out easing (approximation of cubic-bezier(0.4, 0, 1, 1))
    let eased = progress * progress;

    let opacity = 1.0 - eased;
    let x_offset = 12.0 * eased;

    (opacity, x_offset, false)
}

/// Icon swap animation duration in milliseconds
pub const ICON_SWAP_MS: u64 = 300;

/// Calculate icon swap animation state.
/// Returns (state_opacity, state_x, remove_opacity, remove_x) for the swap animation.
/// - state_icon: opacity 1→0, x 0→16
/// - remove_icon: opacity 0→1, x -16→0
pub fn calculate_icon_swap(hover_started: Option<Instant>, is_hovered: bool) -> (f32, f32, f32, f32) {
    let (progress, reverse) = match (hover_started, is_hovered) {
        (Some(start), true) => {
            // Hovering - animate state icon out, remove icon in
            let elapsed = start.elapsed().as_millis() as u64;
            let p = (elapsed as f32 / ICON_SWAP_MS as f32).min(1.0);
            (ease_out(p), false)
        }
        (Some(start), false) => {
            // Not hovering but have a start time - animating back
            let elapsed = start.elapsed().as_millis() as u64;
            let p = (elapsed as f32 / ICON_SWAP_MS as f32).min(1.0);
            (ease_out(p), true)
        }
        (None, _) => (0.0, false),
    };

    let (state_opacity, state_x, remove_opacity, remove_x) = if reverse {
        // Reverse animation (unhover): state fades in, remove fades out
        (progress, 16.0 * (1.0 - progress), 1.0 - progress, -16.0 * progress)
    } else {
        // Forward animation (hover): state fades out, remove fades in
        (1.0 - progress, 16.0 * progress, progress, -16.0 * (1.0 - progress))
    };

    (state_opacity, state_x, remove_opacity, remove_x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_in_out() {
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        // At midpoint should be 0.5
        assert!((ease_in_out(0.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cycle_hash_deterministic() {
        let hash1 = cycle_hash(5, 12345);
        let hash2 = cycle_hash(5, 12345);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_cycle_duration_in_range() {
        for cycle in 0..100 {
            let duration = get_cycle_duration(cycle, 42);
            assert!(duration >= CYCLE_DURATION_MIN_MS);
            assert!(duration <= CYCLE_DURATION_MAX_MS);
        }
    }

    #[test]
    fn test_shake_offset_bounds() {
        let start = Instant::now();
        // At t=0, offset should be near 0 (sin(0) = 0)
        let offset = calculate_shake_offset(start);
        assert!(offset.abs() < 0.5, "Initial offset should be near 0");

        // Offset should always be within amplitude bounds
        std::thread::sleep(std::time::Duration::from_millis(50));
        let offset = calculate_shake_offset(start);
        assert!(
            offset.abs() <= SHAKE_AMPLITUDE + 0.1,
            "Offset should be within amplitude bounds"
        );
    }

    #[test]
    fn test_breathe_opacity_bounds() {
        let start = Instant::now();
        // At t=0, opacity should be at center (sin(0) = 0 -> 0.4)
        let opacity = calculate_breathe_opacity(start);
        assert!(
            (opacity - 0.4).abs() < 0.05,
            "Initial opacity should be near center (0.4), got {}",
            opacity
        );

        // After some time, opacity should always be within bounds [0.3, 0.5]
        std::thread::sleep(std::time::Duration::from_millis(100));
        let opacity = calculate_breathe_opacity(start);
        assert!(
            (BREATHE_MIN_OPACITY - 0.01..=BREATHE_MAX_OPACITY + 0.01).contains(&opacity),
            "Opacity should be in range [0.3, 0.5], got {}",
            opacity
        );
    }
}
