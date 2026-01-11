//! Animation utilities for tool cycling and text marquee

use std::time::Instant;

/// Animation timing constants
pub const CYCLE_DURATION_MIN_MS: u64 = 1500; // Min time showing each tool
pub const CYCLE_DURATION_MAX_MS: u64 = 2000; // Max time showing each tool
pub const FADE_DURATION_MS: u64 = 500; // Cross-fade duration

/// Marquee animation constants
pub const MARQUEE_SPEED_PX_PER_SEC: f32 = 30.0; // Pixels per second scroll speed
pub const MARQUEE_PAUSE_MS: u64 = 1000; // Pause at each end before reversing

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

/// Reset animation duration in milliseconds
pub const RESET_DURATION_MS: u64 = 300;

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

/// Monaco 13px monospace font: base unit width
pub const MARQUEE_CHAR_WIDTH: f32 = 7.8;
/// Separator: 4 spaces
pub const MARQUEE_SEPARATOR_CHARS: usize = 4;
/// Separator width for seamless marquee
pub const MARQUEE_SEPARATOR_WIDTH: f32 = MARQUEE_SEPARATOR_CHARS as f32 * MARQUEE_CHAR_WIDTH;

/// Calculate marquee scroll offset for text that overflows its container.
/// Returns the x-offset in pixels (negative = scrolled left).
///
/// For seamless scrolling, the text is rendered twice with a separator.
/// We scroll by exactly (text_width + separator_width) to create a perfect loop.
///
/// Arguments:
/// - `text_width`: The width of ONE copy of the text
/// - `container_width`: The visible container width
/// - `start_time`: When the animation started
pub fn calculate_marquee_offset(text_width: f32, container_width: f32, start_time: Instant) -> f32 {
    let overflow = text_width - container_width;
    if overflow <= 0.0 {
        return 0.0; // No scrolling needed
    }

    let elapsed_ms = start_time.elapsed().as_millis() as u64;

    // Scroll distance = one copy + separator (seamless loop point)
    let scroll_distance = text_width + MARQUEE_SEPARATOR_WIDTH;
    let scroll_duration_ms = ((scroll_distance / MARQUEE_SPEED_PX_PER_SEC) * 1000.0) as u64;

    // Continuous scroll - no pause
    let pos_in_cycle = elapsed_ms % scroll_duration_ms;
    let scroll_progress = pos_in_cycle as f32 / scroll_duration_ms as f32;
    -scroll_progress * scroll_distance
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
    fn test_marquee_no_overflow() {
        let start = Instant::now();
        // Text fits in container - should always return 0
        assert_eq!(calculate_marquee_offset(50.0, 80.0, start), 0.0);
        assert_eq!(calculate_marquee_offset(80.0, 80.0, start), 0.0);
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
}
