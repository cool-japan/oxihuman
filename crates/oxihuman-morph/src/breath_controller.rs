//! Breathing animation morph controller that drives chest/belly expansion morphs.

/// Configuration for the breath controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BreathConfig {
    /// Breaths per minute at rest.
    pub breaths_per_min: f32,
    /// Duration of inhale phase as fraction of total cycle [0..1].
    pub inhale_fraction: f32,
    /// Duration of hold phase as fraction of total cycle [0..1].
    pub hold_fraction: f32,
    /// Peak chest morph weight during full inhale.
    pub chest_amplitude: f32,
    /// Peak belly morph weight during full inhale.
    pub belly_amplitude: f32,
    /// Peak shoulder morph weight during full inhale.
    pub shoulder_amplitude: f32,
}

/// Current phase of the breath cycle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreathPhase {
    /// Breathing in.
    Inhale,
    /// Breathing out.
    Exhale,
    /// Momentary pause between inhale and exhale.
    Hold,
}

/// Runtime state for the breath controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BreathState {
    /// Active configuration.
    pub config: BreathConfig,
    /// Current phase.
    pub phase: BreathPhase,
    /// Time elapsed within the current cycle [0..cycle_duration].
    pub cycle_time: f32,
    /// Total duration of one breath cycle in seconds.
    pub cycle_duration: f32,
    /// Normalised phase progress [0..1].
    pub phase_progress: f32,
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn cycle_duration_from_bpm(bpm: f32) -> f32 {
    if bpm <= 0.0 { 4.0 } else { 60.0 / bpm }
}

/// Smoothstep ramp used to avoid click artefacts.
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ── public API ───────────────────────────────────────────────────────────────

/// Return a sensible default `BreathConfig` (adult at rest, ~15 bpm).
#[allow(dead_code)]
pub fn default_breath_config() -> BreathConfig {
    BreathConfig {
        breaths_per_min: 15.0,
        inhale_fraction: 0.4,
        hold_fraction: 0.1,
        chest_amplitude: 1.0,
        belly_amplitude: 0.8,
        shoulder_amplitude: 0.3,
    }
}

/// Construct a fresh `BreathState` from the given config.
#[allow(dead_code)]
pub fn new_breath_state(cfg: &BreathConfig) -> BreathState {
    let cycle_duration = cycle_duration_from_bpm(cfg.breaths_per_min);
    BreathState {
        config: cfg.clone(),
        phase: BreathPhase::Inhale,
        cycle_time: 0.0,
        cycle_duration,
        phase_progress: 0.0,
    }
}

/// Advance the breath simulation by `dt` seconds.
#[allow(dead_code)]
pub fn step_breath(state: &mut BreathState, dt: f32) {
    state.cycle_time = (state.cycle_time + dt) % state.cycle_duration;

    let t = state.cycle_time / state.cycle_duration; // [0..1]
    let inhale_end = state.config.inhale_fraction;
    let hold_end = inhale_end + state.config.hold_fraction;

    if t < inhale_end {
        state.phase = BreathPhase::Inhale;
        state.phase_progress = if inhale_end > 0.0 { t / inhale_end } else { 0.0 };
    } else if t < hold_end {
        state.phase = BreathPhase::Hold;
        let hold_len = hold_end - inhale_end;
        state.phase_progress = if hold_len > 0.0 { (t - inhale_end) / hold_len } else { 0.0 };
    } else {
        state.phase = BreathPhase::Exhale;
        let exhale_len = 1.0 - hold_end;
        state.phase_progress = if exhale_len > 0.0 { (t - hold_end) / exhale_len } else { 0.0 };
    }
}

/// Morph weight for chest expansion [0..1].
#[allow(dead_code)]
pub fn breath_chest_weight(state: &BreathState) -> f32 {
    let raw = match state.phase {
        BreathPhase::Inhale => smoothstep(state.phase_progress),
        BreathPhase::Hold => 1.0,
        BreathPhase::Exhale => smoothstep(1.0 - state.phase_progress),
    };
    (raw * state.config.chest_amplitude).clamp(0.0, 1.0)
}

/// Morph weight for belly expansion [0..1].
#[allow(dead_code)]
pub fn breath_belly_weight(state: &BreathState) -> f32 {
    let raw = match state.phase {
        BreathPhase::Inhale => smoothstep(state.phase_progress),
        BreathPhase::Hold => 1.0,
        BreathPhase::Exhale => smoothstep(1.0 - state.phase_progress),
    };
    (raw * state.config.belly_amplitude).clamp(0.0, 1.0)
}

/// Morph weight for shoulder rise [0..1].
#[allow(dead_code)]
pub fn breath_shoulder_weight(state: &BreathState) -> f32 {
    let raw = match state.phase {
        BreathPhase::Inhale => smoothstep(state.phase_progress),
        BreathPhase::Hold => 1.0,
        BreathPhase::Exhale => smoothstep(1.0 - state.phase_progress),
    };
    (raw * state.config.shoulder_amplitude).clamp(0.0, 1.0)
}

/// Update the breathing rate without resetting the cycle position.
#[allow(dead_code)]
pub fn set_breath_rate(state: &mut BreathState, breaths_per_min: f32) {
    let bpm = breaths_per_min.max(0.1);
    state.config.breaths_per_min = bpm;
    let old_dur = state.cycle_duration;
    let new_dur = cycle_duration_from_bpm(bpm);
    // Preserve the relative position in the cycle.
    if old_dur > 0.0 {
        state.cycle_time = state.cycle_time / old_dur * new_dur;
    }
    state.cycle_duration = new_dur;
}

/// Return a human-readable name for a `BreathPhase`.
#[allow(dead_code)]
pub fn breath_phase_name(phase: BreathPhase) -> &'static str {
    match phase {
        BreathPhase::Inhale => "inhale",
        BreathPhase::Hold => "hold",
        BreathPhase::Exhale => "exhale",
    }
}

/// Normalised time within the overall cycle [0..1].
#[allow(dead_code)]
pub fn breath_normalized_time(state: &BreathState) -> f32 {
    if state.cycle_duration > 0.0 {
        (state.cycle_time / state.cycle_duration).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Reset the breath controller back to the start of an inhale.
#[allow(dead_code)]
pub fn reset_breath(state: &mut BreathState) {
    state.phase = BreathPhase::Inhale;
    state.cycle_time = 0.0;
    state.phase_progress = 0.0;
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_reasonable() {
        let cfg = default_breath_config();
        assert!(cfg.breaths_per_min > 0.0);
        assert!(cfg.inhale_fraction > 0.0);
        assert!(cfg.hold_fraction >= 0.0);
        assert!(cfg.inhale_fraction + cfg.hold_fraction < 1.0);
    }

    #[test]
    fn test_new_breath_state_inhale_phase() {
        let cfg = default_breath_config();
        let state = new_breath_state(&cfg);
        assert_eq!(state.phase, BreathPhase::Inhale);
        assert_eq!(state.cycle_time, 0.0);
    }

    #[test]
    fn test_step_breath_advances_time() {
        let cfg = default_breath_config();
        let mut state = new_breath_state(&cfg);
        step_breath(&mut state, 0.5);
        assert!(state.cycle_time > 0.0);
    }

    #[test]
    fn test_weights_in_range() {
        let cfg = default_breath_config();
        let mut state = new_breath_state(&cfg);
        let dt = state.cycle_duration / 100.0;
        for _ in 0..100 {
            step_breath(&mut state, dt);
            let chest = breath_chest_weight(&state);
            let belly = breath_belly_weight(&state);
            let shoulder = breath_shoulder_weight(&state);
            assert!((0.0..=1.0).contains(&chest));
            assert!((0.0..=1.0).contains(&belly));
            assert!((0.0..=1.0).contains(&shoulder));
        }
    }

    #[test]
    fn test_phase_name() {
        assert_eq!(breath_phase_name(BreathPhase::Inhale), "inhale");
        assert_eq!(breath_phase_name(BreathPhase::Hold), "hold");
        assert_eq!(breath_phase_name(BreathPhase::Exhale), "exhale");
    }

    #[test]
    fn test_reset_breath() {
        let cfg = default_breath_config();
        let mut state = new_breath_state(&cfg);
        step_breath(&mut state, 2.0);
        reset_breath(&mut state);
        assert_eq!(state.phase, BreathPhase::Inhale);
        assert_eq!(state.cycle_time, 0.0);
        assert_eq!(state.phase_progress, 0.0);
    }

    #[test]
    fn test_set_breath_rate_updates_duration() {
        let cfg = default_breath_config();
        let mut state = new_breath_state(&cfg);
        set_breath_rate(&mut state, 30.0);
        let expected = 60.0_f32 / 30.0;
        assert!((state.cycle_duration - expected).abs() < 1e-5);
    }

    #[test]
    fn test_normalized_time_range() {
        let cfg = default_breath_config();
        let mut state = new_breath_state(&cfg);
        for i in 0..=10 {
            state.cycle_time = state.cycle_duration * (i as f32 / 10.0);
            let t = breath_normalized_time(&state);
            assert!((0.0..=1.0).contains(&t));
        }
    }
}
