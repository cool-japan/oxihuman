// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! FPS counter with rolling average and display formatting.
//!
//! Call [`tick_fps`] each frame with the delta time in seconds to update the
//! counter, then query [`current_fps`], [`average_fps`], [`min_fps`], etc.

#![allow(dead_code)]

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for the FPS counter.
#[derive(Debug, Clone)]
pub struct FpsCounterConfig {
    /// Number of samples used for the rolling average. Default: `60`.
    pub sample_window: usize,
    /// Minimum delta time (seconds) accepted — guards against division by zero.
    pub min_delta_secs: f32,
    /// FPS is considered "stable" when variance falls below this threshold.
    pub stability_threshold: f32,
}

impl Default for FpsCounterConfig {
    fn default() -> Self {
        Self {
            sample_window: 60,
            min_delta_secs: 1e-6,
            stability_threshold: 5.0,
        }
    }
}

/// FPS counter state.
#[derive(Debug, Clone)]
pub struct FpsCounter {
    /// User configuration.
    pub config: FpsCounterConfig,
    /// Ring buffer of recent instantaneous FPS values.
    samples: Vec<f32>,
    /// Total number of ticks ever recorded.
    total_frames: u64,
    /// All-time minimum FPS observed (since creation or last reset).
    min_observed: f32,
    /// All-time maximum FPS observed (since creation or last reset).
    max_observed: f32,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// JSON string representation type alias.
pub type FpsJson = String;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`FpsCounterConfig`].
#[allow(dead_code)]
pub fn default_fps_config() -> FpsCounterConfig {
    FpsCounterConfig::default()
}

/// Create a new [`FpsCounter`] with the given configuration.
#[allow(dead_code)]
pub fn new_fps_counter(config: FpsCounterConfig) -> FpsCounter {
    FpsCounter {
        config,
        samples: Vec::new(),
        total_frames: 0,
        min_observed: f32::MAX,
        max_observed: 0.0,
    }
}

/// Advance the counter with `delta_secs` elapsed since the last frame.
///
/// Returns the updated [`FpsCounter`].
#[allow(dead_code)]
pub fn tick_fps(mut counter: FpsCounter, delta_secs: f32) -> FpsCounter {
    let dt = delta_secs.max(counter.config.min_delta_secs);
    let fps = 1.0 / dt;

    if fps < counter.min_observed {
        counter.min_observed = fps;
    }
    if fps > counter.max_observed {
        counter.max_observed = fps;
    }

    counter.total_frames += 1;
    counter.samples.push(fps);

    let win = counter.config.sample_window;
    if counter.samples.len() > win {
        let excess = counter.samples.len() - win;
        counter.samples.drain(..excess);
    }
    counter
}

/// Return the most recent instantaneous FPS, or `0.0` if no ticks recorded.
#[allow(dead_code)]
pub fn current_fps(counter: &FpsCounter) -> f32 {
    counter.samples.last().copied().unwrap_or(0.0)
}

/// Return the rolling average FPS over the current sample window.
/// Returns `0.0` if no samples.
#[allow(dead_code)]
pub fn average_fps(counter: &FpsCounter) -> f32 {
    let n = counter.samples.len();
    if n == 0 {
        return 0.0;
    }
    counter.samples.iter().sum::<f32>() / n as f32
}

/// Return the all-time minimum FPS observed since creation or last reset.
/// Returns `0.0` if no ticks recorded.
#[allow(dead_code)]
pub fn min_fps(counter: &FpsCounter) -> f32 {
    if counter.total_frames == 0 {
        0.0
    } else {
        counter.min_observed
    }
}

/// Return the all-time maximum FPS observed since creation or last reset.
#[allow(dead_code)]
pub fn max_fps(counter: &FpsCounter) -> f32 {
    counter.max_observed
}

/// Format the current FPS as a display string, e.g. `"60.0 FPS"`.
#[allow(dead_code)]
pub fn fps_to_string(counter: &FpsCounter) -> String {
    format!("{:.1} FPS", current_fps(counter))
}

/// Reset the counter: clear all samples, min/max, and total frame count.
#[allow(dead_code)]
pub fn reset_fps_counter(mut counter: FpsCounter) -> FpsCounter {
    counter.samples.clear();
    counter.total_frames = 0;
    counter.min_observed = f32::MAX;
    counter.max_observed = 0.0;
    counter
}

/// Return the total number of ticks recorded since creation or last reset.
#[allow(dead_code)]
pub fn frame_count(counter: &FpsCounter) -> u64 {
    counter.total_frames
}

/// Serialise the counter state as a compact JSON string.
#[allow(dead_code)]
pub fn fps_counter_to_json(counter: &FpsCounter) -> FpsJson {
    let mn = if counter.total_frames == 0 {
        0.0
    } else {
        counter.min_observed
    };
    format!(
        "{{\"current_fps\":{:.2},\"average_fps\":{:.2},\"min_fps\":{:.2},\
         \"max_fps\":{:.2},\"frame_count\":{},\"sample_window\":{}}}",
        current_fps(counter),
        average_fps(counter),
        mn,
        counter.max_observed,
        counter.total_frames,
        counter.config.sample_window,
    )
}

/// Update the sample window size on `counter` and return it.
///
/// If the new window is smaller than the current sample count, older samples
/// are dropped.
#[allow(dead_code)]
pub fn set_sample_window(mut counter: FpsCounter, window: usize) -> FpsCounter {
    counter.config.sample_window = window;
    if counter.samples.len() > window {
        let excess = counter.samples.len() - window;
        counter.samples.drain(..excess);
    }
    counter
}

/// Return `true` if the FPS is stable, i.e., the standard deviation of the
/// rolling window is below `config.stability_threshold`.
///
/// Returns `false` if fewer than 2 samples are available.
#[allow(dead_code)]
pub fn is_fps_stable(counter: &FpsCounter) -> bool {
    let n = counter.samples.len();
    if n < 2 {
        return false;
    }
    let mean = average_fps(counter);
    let variance: f32 = counter
        .samples
        .iter()
        .map(|&s| (s - mean) * (s - mean))
        .sum::<f32>()
        / n as f32;
    variance.sqrt() < counter.config.stability_threshold
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Tick `n` frames all with the same delta time.
    fn tick_n(counter: FpsCounter, delta: f32, n: u32) -> FpsCounter {
        (0..n).fold(counter, |c, _| tick_fps(c, delta))
    }

    #[test]
    fn default_config_fields() {
        let cfg = default_fps_config();
        assert_eq!(cfg.sample_window, 60);
        assert!((cfg.min_delta_secs - 1e-6).abs() < 1e-9);
        assert!((cfg.stability_threshold - 5.0).abs() < 1e-4);
    }

    #[test]
    fn new_counter_zero_state() {
        let c = new_fps_counter(default_fps_config());
        assert_eq!(frame_count(&c), 0);
        assert!((current_fps(&c)).abs() < 1e-6);
        assert!((average_fps(&c)).abs() < 1e-6);
        assert!((min_fps(&c)).abs() < 1e-6);
        assert!((max_fps(&c)).abs() < 1e-6);
    }

    #[test]
    fn tick_one_frame_60fps() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_fps(c, 1.0 / 60.0);
        assert!((current_fps(&c) - 60.0).abs() < 0.01);
        assert_eq!(frame_count(&c), 1);
    }

    #[test]
    fn average_fps_over_uniform_ticks() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_n(c, 1.0 / 60.0, 10);
        assert!((average_fps(&c) - 60.0).abs() < 0.1);
    }

    #[test]
    fn min_fps_tracks_minimum() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_fps(c, 1.0 / 30.0); // 30 FPS
        let c = tick_fps(c, 1.0 / 60.0); // 60 FPS
        assert!((min_fps(&c) - 30.0).abs() < 0.1);
    }

    #[test]
    fn max_fps_tracks_maximum() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_fps(c, 1.0 / 30.0); // 30 FPS
        let c = tick_fps(c, 1.0 / 120.0); // 120 FPS
        assert!((max_fps(&c) - 120.0).abs() < 0.5);
    }

    #[test]
    fn fps_to_string_format() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_fps(c, 1.0 / 60.0);
        let s = fps_to_string(&c);
        assert!(s.ends_with(" FPS"));
    }

    #[test]
    fn reset_clears_all() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_n(c, 1.0 / 60.0, 5);
        let c = reset_fps_counter(c);
        assert_eq!(frame_count(&c), 0);
        assert!((current_fps(&c)).abs() < 1e-6);
    }

    #[test]
    fn frame_count_increments() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_n(c, 0.016, 7);
        assert_eq!(frame_count(&c), 7);
    }

    #[test]
    fn fps_counter_to_json_valid() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_fps(c, 0.016);
        let j = fps_counter_to_json(&c);
        assert!(j.starts_with('{'));
        assert!(j.ends_with('}'));
        assert!(j.contains("current_fps"));
    }

    #[test]
    fn set_sample_window_shrinks_samples() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_n(c, 0.016, 10);
        assert_eq!(c.samples.len(), 10);
        let c = set_sample_window(c, 5);
        assert_eq!(c.samples.len(), 5);
        assert_eq!(c.config.sample_window, 5);
    }

    #[test]
    fn is_fps_stable_true_for_uniform() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_n(c, 1.0 / 60.0, 30);
        assert!(is_fps_stable(&c));
    }

    #[test]
    fn is_fps_stable_false_for_varying() {
        let c = new_fps_counter(default_fps_config());
        // Alternate between very fast and very slow
        let c = tick_fps(c, 1.0 / 5.0);   // 5 fps
        let c = tick_fps(c, 1.0 / 120.0); // 120 fps
        let c = tick_fps(c, 1.0 / 5.0);
        let c = tick_fps(c, 1.0 / 120.0);
        assert!(!is_fps_stable(&c));
    }

    #[test]
    fn is_fps_stable_false_with_one_sample() {
        let c = new_fps_counter(default_fps_config());
        let c = tick_fps(c, 0.016);
        assert!(!is_fps_stable(&c));
    }

    #[test]
    fn window_limits_sample_count() {
        let mut cfg = default_fps_config();
        cfg.sample_window = 3;
        let c = new_fps_counter(cfg);
        let c = tick_n(c, 0.016, 10);
        assert_eq!(c.samples.len(), 3);
    }

    #[test]
    fn min_delta_guards_division_by_zero() {
        let c = new_fps_counter(default_fps_config());
        // Pass 0.0 — should clamp to min_delta_secs
        let c = tick_fps(c, 0.0);
        // Should produce a very large FPS but not panic
        assert!(current_fps(&c) > 0.0);
    }

    #[test]
    fn total_frames_survives_window_trim() {
        let mut cfg = default_fps_config();
        cfg.sample_window = 5;
        let c = new_fps_counter(cfg);
        let c = tick_n(c, 0.016, 20);
        assert_eq!(frame_count(&c), 20);
        assert_eq!(c.samples.len(), 5);
    }
}
