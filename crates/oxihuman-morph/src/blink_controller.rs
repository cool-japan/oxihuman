//! Eye blink animation morph controller — drives upper/lower eyelid morph weights
//! with natural blink timing using a deterministic phase-based model.

// ── Types ────────────────────────────────────────────────────────────────────

/// Configuration for the blink controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlinkConfig {
    /// Target blinks per minute.
    pub blinks_per_minute: f32,
    /// Duration of the closing phase in seconds.
    pub close_duration: f32,
    /// Duration of the closed phase in seconds (hold time).
    pub closed_duration: f32,
    /// Duration of the opening phase in seconds.
    pub open_duration: f32,
    /// Weight applied to the upper eyelid morph at full close.
    pub upper_lid_max: f32,
    /// Weight applied to the lower eyelid morph at full close.
    pub lower_lid_max: f32,
}

/// Phase of the blink cycle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlinkPhase {
    /// Eyes fully open; waiting for the next blink.
    Open,
    /// Upper/lower lids moving toward fully closed.
    Closing,
    /// Lids fully closed.
    Closed,
    /// Lids moving back toward fully open.
    Opening,
}

/// Runtime state of the blink controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlinkState {
    /// Active configuration.
    pub config: BlinkConfig,
    /// Current phase.
    pub phase: BlinkPhase,
    /// Time elapsed within the current phase.
    pub phase_time: f32,
    /// Countdown until the next spontaneous blink (seconds).
    pub next_blink_timer: f32,
    /// Normalised eye-open amount: 0 = closed, 1 = open.
    pub open_amount: f32,
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Convert blinks-per-minute to the average interval between blinks in seconds.
fn bpm_to_interval(bpm: f32) -> f32 {
    if bpm <= 0.0 {
        4.0
    } else {
        60.0 / bpm
    }
}

/// Smoothstep easing to make lid motion feel natural.
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default `BlinkConfig` (~15 blinks/min, typical human rate).
#[allow(dead_code)]
pub fn default_blink_config() -> BlinkConfig {
    BlinkConfig {
        blinks_per_minute: 15.0,
        close_duration: 0.06,
        closed_duration: 0.02,
        open_duration: 0.10,
        upper_lid_max: 1.0,
        lower_lid_max: 0.4,
    }
}

/// Construct a fresh `BlinkState` from the given config.
#[allow(dead_code)]
pub fn new_blink_state(cfg: &BlinkConfig) -> BlinkState {
    BlinkState {
        config: cfg.clone(),
        phase: BlinkPhase::Open,
        phase_time: 0.0,
        next_blink_timer: bpm_to_interval(cfg.blinks_per_minute),
        open_amount: 1.0,
    }
}

/// Advance the blink simulation by `dt` seconds.
#[allow(dead_code)]
pub fn step_blink(state: &mut BlinkState, dt: f32) {
    match state.phase {
        BlinkPhase::Open => {
            state.open_amount = 1.0;
            state.next_blink_timer -= dt;
            if state.next_blink_timer <= 0.0 {
                state.phase = BlinkPhase::Closing;
                state.phase_time = 0.0;
            }
        }
        BlinkPhase::Closing => {
            state.phase_time += dt;
            let t = if state.config.close_duration > 0.0 {
                (state.phase_time / state.config.close_duration).clamp(0.0, 1.0)
            } else {
                1.0
            };
            state.open_amount = 1.0 - smoothstep(t);
            if state.phase_time >= state.config.close_duration {
                state.phase = BlinkPhase::Closed;
                state.phase_time = 0.0;
                state.open_amount = 0.0;
            }
        }
        BlinkPhase::Closed => {
            state.open_amount = 0.0;
            state.phase_time += dt;
            if state.phase_time >= state.config.closed_duration {
                state.phase = BlinkPhase::Opening;
                state.phase_time = 0.0;
            }
        }
        BlinkPhase::Opening => {
            state.phase_time += dt;
            let t = if state.config.open_duration > 0.0 {
                (state.phase_time / state.config.open_duration).clamp(0.0, 1.0)
            } else {
                1.0
            };
            state.open_amount = smoothstep(t);
            if state.phase_time >= state.config.open_duration {
                state.phase = BlinkPhase::Open;
                state.phase_time = 0.0;
                state.open_amount = 1.0;
                state.next_blink_timer = bpm_to_interval(state.config.blinks_per_minute);
            }
        }
    }
}

/// Current morph weight for the upper eyelid (0 = open, 1 = closed).
#[allow(dead_code)]
pub fn blink_upper_lid_weight(state: &BlinkState) -> f32 {
    ((1.0 - state.open_amount) * state.config.upper_lid_max).clamp(0.0, 1.0)
}

/// Current morph weight for the lower eyelid (0 = open, 1 = raised).
#[allow(dead_code)]
pub fn blink_lower_lid_weight(state: &BlinkState) -> f32 {
    ((1.0 - state.open_amount) * state.config.lower_lid_max).clamp(0.0, 1.0)
}

/// Force an immediate blink by entering the Closing phase now.
#[allow(dead_code)]
pub fn trigger_blink(state: &mut BlinkState) {
    state.phase = BlinkPhase::Closing;
    state.phase_time = 0.0;
}

/// Update the spontaneous blink rate without resetting the current phase.
#[allow(dead_code)]
pub fn set_blink_rate(state: &mut BlinkState, blinks_per_minute: f32) {
    state.config.blinks_per_minute = blinks_per_minute.max(0.0);
}

/// Human-readable name for a `BlinkPhase`.
#[allow(dead_code)]
pub fn blink_phase_name(phase: BlinkPhase) -> &'static str {
    match phase {
        BlinkPhase::Open => "open",
        BlinkPhase::Closing => "closing",
        BlinkPhase::Closed => "closed",
        BlinkPhase::Opening => "opening",
    }
}

/// Returns `true` when the eye is fully open (not in a blink).
#[allow(dead_code)]
pub fn blink_is_open(state: &BlinkState) -> bool {
    state.phase == BlinkPhase::Open
}

/// Reset the blink controller to a fully-open, waiting state.
#[allow(dead_code)]
pub fn reset_blink(state: &mut BlinkState) {
    state.phase = BlinkPhase::Open;
    state.phase_time = 0.0;
    state.next_blink_timer = bpm_to_interval(state.config.blinks_per_minute);
    state.open_amount = 1.0;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> BlinkState {
        new_blink_state(&default_blink_config())
    }

    #[test]
    fn test_default_config_positive_rate() {
        let cfg = default_blink_config();
        assert!(cfg.blinks_per_minute > 0.0);
        assert!(cfg.close_duration > 0.0);
        assert!(cfg.open_duration > 0.0);
    }

    #[test]
    fn test_new_state_starts_open() {
        let s = make_state();
        assert_eq!(s.phase, BlinkPhase::Open);
        assert_eq!(s.open_amount, 1.0);
        assert!(s.next_blink_timer > 0.0);
    }

    #[test]
    fn test_trigger_blink_enters_closing() {
        let mut s = make_state();
        trigger_blink(&mut s);
        assert_eq!(s.phase, BlinkPhase::Closing);
    }

    #[test]
    fn test_step_through_full_blink() {
        let mut s = make_state();
        trigger_blink(&mut s);
        // Drive through closing → closed → opening → open.
        for _ in 0..200 {
            step_blink(&mut s, 0.01);
        }
        assert_eq!(s.phase, BlinkPhase::Open);
        assert!((s.open_amount - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_upper_lid_weight_range() {
        let mut s = make_state();
        trigger_blink(&mut s);
        for _ in 0..100 {
            step_blink(&mut s, 0.005);
            let w = blink_upper_lid_weight(&s);
            assert!((0.0..=1.0).contains(&w));
        }
    }

    #[test]
    fn test_lower_lid_weight_range() {
        let mut s = make_state();
        trigger_blink(&mut s);
        for _ in 0..100 {
            step_blink(&mut s, 0.005);
            let w = blink_lower_lid_weight(&s);
            assert!((0.0..=1.0).contains(&w));
        }
    }

    #[test]
    fn test_phase_name_all_variants() {
        assert_eq!(blink_phase_name(BlinkPhase::Open), "open");
        assert_eq!(blink_phase_name(BlinkPhase::Closing), "closing");
        assert_eq!(blink_phase_name(BlinkPhase::Closed), "closed");
        assert_eq!(blink_phase_name(BlinkPhase::Opening), "opening");
    }

    #[test]
    fn test_blink_is_open_true_at_start() {
        let s = make_state();
        assert!(blink_is_open(&s));
    }

    #[test]
    fn test_blink_is_open_false_during_blink() {
        let mut s = make_state();
        trigger_blink(&mut s);
        assert!(!blink_is_open(&s));
    }

    #[test]
    fn test_reset_blink_restores_state() {
        let mut s = make_state();
        trigger_blink(&mut s);
        step_blink(&mut s, 0.05);
        reset_blink(&mut s);
        assert_eq!(s.phase, BlinkPhase::Open);
        assert_eq!(s.open_amount, 1.0);
        assert!(s.next_blink_timer > 0.0);
    }

    #[test]
    fn test_set_blink_rate_clamps_zero() {
        let mut s = make_state();
        set_blink_rate(&mut s, -5.0);
        assert_eq!(s.config.blinks_per_minute, 0.0);
    }

    #[test]
    fn test_spontaneous_blink_fires() {
        let cfg = BlinkConfig {
            blinks_per_minute: 120.0, // very fast — 0.5 s interval
            close_duration: 0.01,
            closed_duration: 0.01,
            open_duration: 0.01,
            upper_lid_max: 1.0,
            lower_lid_max: 0.4,
        };
        let mut s = new_blink_state(&cfg);
        // Step enough to trigger the spontaneous blink.
        for _ in 0..100 {
            step_blink(&mut s, 0.01);
        }
        // After 1 s the blink should have completed and returned to Open.
        assert_eq!(s.phase, BlinkPhase::Open);
    }
}
