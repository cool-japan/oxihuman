//! Jaw open/close/shift morph control for speech and expressions.

/// Configuration for the jaw controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawConfig {
    /// Maximum open amount in normalised units [0..1].
    pub max_open: f32,
    /// Maximum lateral shift in either direction [0..1].
    pub max_shift: f32,
    /// Maximum forward protrusion [0..1].
    pub max_forward: f32,
    /// Damping factor applied when blending states [0..1].
    pub damping: f32,
}

/// Runtime state for the jaw controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawState {
    /// Active configuration.
    pub config: JawConfig,
    /// Current open amount [0..1].
    pub open: f32,
    /// Current left-shift amount [0..1].
    pub shift_left: f32,
    /// Current right-shift amount [0..1].
    pub shift_right: f32,
    /// Current forward protrusion [0..1].
    pub forward: f32,
}

// ── public API ───────────────────────────────────────────────────────────────

/// Return a sensible default `JawConfig`.
#[allow(dead_code)]
pub fn default_jaw_config() -> JawConfig {
    JawConfig {
        max_open: 1.0,
        max_shift: 0.5,
        max_forward: 0.3,
        damping: 0.1,
    }
}

/// Construct a fresh `JawState` with all values at zero.
#[allow(dead_code)]
pub fn new_jaw_state(cfg: &JawConfig) -> JawState {
    JawState {
        config: cfg.clone(),
        open: 0.0,
        shift_left: 0.0,
        shift_right: 0.0,
        forward: 0.0,
    }
}

/// Set the jaw open amount (clamped to `[0..max_open]`).
#[allow(dead_code)]
pub fn set_jaw_open(state: &mut JawState, amount: f32) {
    state.open = amount.clamp(0.0, state.config.max_open);
}

/// Set the jaw left-shift amount (clamped to `[0..max_shift]`).
#[allow(dead_code)]
pub fn set_jaw_shift_left(state: &mut JawState, amount: f32) {
    state.shift_left = amount.clamp(0.0, state.config.max_shift);
    // Shifting left cancels right shift.
    if state.shift_left > 0.0 {
        state.shift_right = 0.0;
    }
}

/// Set the jaw right-shift amount (clamped to `[0..max_shift]`).
#[allow(dead_code)]
pub fn set_jaw_shift_right(state: &mut JawState, amount: f32) {
    state.shift_right = amount.clamp(0.0, state.config.max_shift);
    // Shifting right cancels left shift.
    if state.shift_right > 0.0 {
        state.shift_left = 0.0;
    }
}

/// Set the jaw forward protrusion amount (clamped to `[0..max_forward]`).
#[allow(dead_code)]
pub fn set_jaw_forward(state: &mut JawState, amount: f32) {
    state.forward = amount.clamp(0.0, state.config.max_forward);
}

/// Return the effective jaw open weight [0..1].
#[allow(dead_code)]
pub fn jaw_open_weight(state: &JawState) -> f32 {
    if state.config.max_open > 0.0 {
        (state.open / state.config.max_open).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Return all four morph weights as `[open, shift_left, shift_right, forward]`.
#[allow(dead_code)]
pub fn jaw_morph_weights(state: &JawState) -> [f32; 4] {
    let open = jaw_open_weight(state);
    let sl = if state.config.max_shift > 0.0 {
        (state.shift_left / state.config.max_shift).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let sr = if state.config.max_shift > 0.0 {
        (state.shift_right / state.config.max_shift).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fwd = if state.config.max_forward > 0.0 {
        (state.forward / state.config.max_forward).clamp(0.0, 1.0)
    } else {
        0.0
    };
    [open, sl, sr, fwd]
}

/// Reset all jaw values to zero.
#[allow(dead_code)]
pub fn reset_jaw(state: &mut JawState) {
    state.open = 0.0;
    state.shift_left = 0.0;
    state.shift_right = 0.0;
    state.forward = 0.0;
}

/// Blend two `JawState` values by factor `t` (0 → a, 1 → b).
/// The result uses the config of `a`.
#[allow(dead_code)]
pub fn blend_jaw_states(a: &JawState, b: &JawState, t: f32) -> JawState {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    JawState {
        config: a.config.clone(),
        open: lerp(a.open, b.open),
        shift_left: lerp(a.shift_left, b.shift_left),
        shift_right: lerp(a.shift_right, b.shift_right),
        forward: lerp(a.forward, b.forward),
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_jaw_config();
        assert!(cfg.max_open > 0.0);
        assert!(cfg.max_shift > 0.0);
        assert!(cfg.max_forward > 0.0);
    }

    #[test]
    fn test_new_jaw_state_zero() {
        let cfg = default_jaw_config();
        let state = new_jaw_state(&cfg);
        assert_eq!(state.open, 0.0);
        assert_eq!(state.shift_left, 0.0);
        assert_eq!(state.shift_right, 0.0);
        assert_eq!(state.forward, 0.0);
    }

    #[test]
    fn test_set_jaw_open_clamped() {
        let cfg = default_jaw_config();
        let mut state = new_jaw_state(&cfg);
        set_jaw_open(&mut state, 2.0);
        assert_eq!(state.open, cfg.max_open);
        set_jaw_open(&mut state, -0.5);
        assert_eq!(state.open, 0.0);
    }

    #[test]
    fn test_shift_left_clears_right() {
        let cfg = default_jaw_config();
        let mut state = new_jaw_state(&cfg);
        set_jaw_shift_right(&mut state, 0.3);
        set_jaw_shift_left(&mut state, 0.2);
        assert_eq!(state.shift_right, 0.0);
        assert!(state.shift_left > 0.0);
    }

    #[test]
    fn test_morph_weights_range() {
        let cfg = default_jaw_config();
        let mut state = new_jaw_state(&cfg);
        set_jaw_open(&mut state, 0.5);
        set_jaw_shift_left(&mut state, 0.1);
        set_jaw_forward(&mut state, 0.2);
        let w = jaw_morph_weights(&state);
        for weight in w {
            assert!((0.0..=1.0).contains(&weight));
        }
    }

    #[test]
    fn test_reset_jaw() {
        let cfg = default_jaw_config();
        let mut state = new_jaw_state(&cfg);
        set_jaw_open(&mut state, 0.8);
        set_jaw_forward(&mut state, 0.3);
        reset_jaw(&mut state);
        let w = jaw_morph_weights(&state);
        for weight in w {
            assert_eq!(weight, 0.0);
        }
    }

    #[test]
    fn test_blend_jaw_states() {
        let cfg = default_jaw_config();
        let mut a = new_jaw_state(&cfg);
        let mut b = new_jaw_state(&cfg);
        set_jaw_open(&mut a, 0.0);
        set_jaw_open(&mut b, 1.0);
        let mid = blend_jaw_states(&a, &b, 0.5);
        assert!((mid.open - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_jaw_open_weight_normalised() {
        let cfg = default_jaw_config();
        let mut state = new_jaw_state(&cfg);
        set_jaw_open(&mut state, cfg.max_open);
        assert!((jaw_open_weight(&state) - 1.0).abs() < 1e-5);
    }
}
