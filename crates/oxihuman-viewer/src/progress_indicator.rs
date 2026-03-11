// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! On-screen progress bar / spinner widget for long operations.

#![allow(dead_code)]

/// Configuration for the progress indicator.
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// Minimum progress value (usually 0).
    pub min: f64,
    /// Maximum progress value (usually 1 or 100).
    pub max: f64,
    /// Whether to show spinner animation.
    pub show_spinner: bool,
    /// Width of the progress bar in pixels.
    pub bar_width_px: u32,
}

/// Runtime state for the progress indicator.
#[derive(Debug, Clone)]
pub struct ProgressState {
    /// Configuration.
    pub config: ProgressConfig,
    /// Current progress value.
    pub current: f64,
    /// Display label.
    pub label: String,
    /// Whether the operation is complete.
    pub done: bool,
    /// Spinner tick counter (for animation).
    pub spinner_tick: u32,
}

/// Returns the default [`ProgressConfig`].
pub fn default_progress_config() -> ProgressConfig {
    ProgressConfig {
        min: 0.0,
        max: 1.0,
        show_spinner: true,
        bar_width_px: 200,
    }
}

/// Creates a new [`ProgressState`] at zero progress.
pub fn new_progress_indicator(config: ProgressConfig) -> ProgressState {
    ProgressState {
        config,
        current: 0.0,
        label: String::new(),
        done: false,
        spinner_tick: 0,
    }
}

/// Sets the current progress value (clamped to [min, max]).
pub fn progress_set(state: &mut ProgressState, value: f64) {
    state.current = value.clamp(state.config.min, state.config.max);
}

/// Advances progress by `delta` (clamped at max).
pub fn progress_advance(state: &mut ProgressState, delta: f64) {
    let new_val = (state.current + delta).min(state.config.max);
    state.current = new_val.max(state.config.min);
    state.spinner_tick = state.spinner_tick.wrapping_add(1);
}

/// Marks the operation as complete and sets progress to max.
pub fn progress_complete(state: &mut ProgressState) {
    state.current = state.config.max;
    state.done = true;
}

/// Returns whether the operation is done.
pub fn progress_is_done(state: &ProgressState) -> bool {
    state.done
}

/// Returns the progress as a fraction in [0.0, 1.0].
pub fn progress_fraction(state: &ProgressState) -> f64 {
    let range = state.config.max - state.config.min;
    if range <= 0.0 {
        return 1.0;
    }
    ((state.current - state.config.min) / range).clamp(0.0, 1.0)
}

/// Returns the current label string.
pub fn progress_label(state: &ProgressState) -> &str {
    &state.label
}

/// Serialises the progress state as JSON.
pub fn progress_to_json(state: &ProgressState) -> String {
    format!(
        "{{\"current\":{:.6},\"min\":{:.6},\"max\":{:.6},\
        \"fraction\":{:.6},\"done\":{},\"label\":\"{}\",\"spinner_tick\":{}}}",
        state.current,
        state.config.min,
        state.config.max,
        progress_fraction(state),
        state.done,
        state.label,
        state.spinner_tick,
    )
}

/// Resets progress to min and clears done flag.
pub fn progress_reset(state: &mut ProgressState) {
    state.current = state.config.min;
    state.done = false;
    state.label.clear();
    state.spinner_tick = 0;
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_state() -> ProgressState {
        new_progress_indicator(default_progress_config())
    }

    #[test]
    fn default_config_values() {
        let cfg = default_progress_config();
        assert!((cfg.min - 0.0).abs() < 1e-10);
        assert!((cfg.max - 1.0).abs() < 1e-10);
        assert!(cfg.show_spinner);
    }

    #[test]
    fn new_state_at_zero() {
        let s = default_state();
        assert!((progress_fraction(&s) - 0.0).abs() < 1e-10);
        assert!(!progress_is_done(&s));
    }

    #[test]
    fn set_progress_clamps_to_max() {
        let mut s = default_state();
        progress_set(&mut s, 2.0);
        assert!((s.current - 1.0).abs() < 1e-10);
    }

    #[test]
    fn set_progress_half() {
        let mut s = default_state();
        progress_set(&mut s, 0.5);
        assert!((progress_fraction(&s) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn advance_increments_progress() {
        let mut s = default_state();
        progress_advance(&mut s, 0.3);
        assert!((progress_fraction(&s) - 0.3).abs() < 1e-10);
    }

    #[test]
    fn advance_does_not_exceed_max() {
        let mut s = default_state();
        progress_advance(&mut s, 0.8);
        progress_advance(&mut s, 0.8);
        assert!((progress_fraction(&s) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn complete_marks_done() {
        let mut s = default_state();
        progress_complete(&mut s);
        assert!(progress_is_done(&s));
        assert!((progress_fraction(&s) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn reset_clears_state() {
        let mut s = default_state();
        progress_set(&mut s, 0.7);
        progress_complete(&mut s);
        progress_reset(&mut s);
        assert!(!progress_is_done(&s));
        assert!((progress_fraction(&s) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn to_json_contains_fraction_and_done() {
        let mut s = default_state();
        progress_set(&mut s, 0.5);
        let json = progress_to_json(&s);
        assert!(json.contains("\"fraction\""));
        assert!(json.contains("\"done\""));
        assert!(json.contains("\"current\""));
    }

    #[test]
    fn label_accessor() {
        let mut s = default_state();
        s.label = "Processing...".to_string();
        assert_eq!(progress_label(&s), "Processing...");
    }
}
