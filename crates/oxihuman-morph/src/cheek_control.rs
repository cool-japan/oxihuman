//! Cheek puff and hollow morph control for facial expressions.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Which side of the face to address.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheekSide {
    Left,
    Right,
    Both,
}

/// Configuration for cheek dynamics and limits.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CheekConfig {
    /// Maximum puff intensity (0 = neutral, 1 = fully puffed).
    pub max_puff: f32,
    /// Maximum hollow intensity (0 = neutral, -1 = fully hollowed).
    pub max_hollow: f32,
    /// Maximum raise amount (0..1).
    pub max_raise: f32,
    /// Smoothing factor for transitions (higher = snappier).
    pub smoothing: f32,
    /// How much a smile automatically raises/puffs the cheeks (0..1).
    pub smile_puff_factor: f32,
}

/// Runtime state of both cheeks.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CheekState {
    /// Left cheek puff amount (0..1).
    pub puff_left: f32,
    /// Right cheek puff amount (0..1).
    pub puff_right: f32,
    /// Left cheek hollow amount (-1..0).
    pub hollow_left: f32,
    /// Right cheek hollow amount (-1..0).
    pub hollow_right: f32,
    /// Left cheek raise amount (0..1).
    pub raise_left: f32,
    /// Right cheek raise amount (0..1).
    pub raise_right: f32,
    /// Target puff left (for smooth interpolation).
    pub target_puff_left: f32,
    /// Target puff right (for smooth interpolation).
    pub target_puff_right: f32,
}

/// Type alias for a list of morph-target weight pairs.
#[allow(dead_code)]
pub type CheekMorphWeights = Vec<(String, f32)>;

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Return a sensible default cheek configuration.
#[allow(dead_code)]
pub fn default_cheek_config() -> CheekConfig {
    CheekConfig {
        max_puff: 1.0,
        max_hollow: -1.0,
        max_raise: 1.0,
        smoothing: 8.0,
        smile_puff_factor: 0.3,
    }
}

/// Create a new cheek state at rest (all zeros).
#[allow(dead_code)]
pub fn new_cheek_state() -> CheekState {
    CheekState {
        puff_left: 0.0,
        puff_right: 0.0,
        hollow_left: 0.0,
        hollow_right: 0.0,
        raise_left: 0.0,
        raise_right: 0.0,
        target_puff_left: 0.0,
        target_puff_right: 0.0,
    }
}

// ---------------------------------------------------------------------------
// Setters
// ---------------------------------------------------------------------------

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn clamp_neg1_0(v: f32) -> f32 {
    v.clamp(-1.0, 0.0)
}

/// Set cheek puff amount (clamped 0..1) on the given side.
#[allow(dead_code)]
pub fn set_cheek_puff(state: &mut CheekState, side: CheekSide, amount: f32) {
    let v = clamp01(amount);
    match side {
        CheekSide::Left => {
            state.target_puff_left = v;
        }
        CheekSide::Right => {
            state.target_puff_right = v;
        }
        CheekSide::Both => {
            state.target_puff_left = v;
            state.target_puff_right = v;
        }
    }
}

/// Set cheek hollow amount (clamped -1..0) on the given side.
#[allow(dead_code)]
pub fn set_cheek_hollow(state: &mut CheekState, side: CheekSide, amount: f32) {
    let v = clamp_neg1_0(amount);
    match side {
        CheekSide::Left => state.hollow_left = v,
        CheekSide::Right => state.hollow_right = v,
        CheekSide::Both => {
            state.hollow_left = v;
            state.hollow_right = v;
        }
    }
}

/// Set cheek raise amount (clamped 0..1) on the given side.
#[allow(dead_code)]
pub fn set_cheek_raise(state: &mut CheekState, side: CheekSide, amount: f32) {
    let v = clamp01(amount);
    match side {
        CheekSide::Left => state.raise_left = v,
        CheekSide::Right => state.raise_right = v,
        CheekSide::Both => {
            state.raise_left = v;
            state.raise_right = v;
        }
    }
}

// ---------------------------------------------------------------------------
// Update / smoothing
// ---------------------------------------------------------------------------

/// Advance cheek state toward targets using exponential smoothing.
/// `dt` is the timestep in seconds.
#[allow(dead_code)]
pub fn update_cheeks(state: &mut CheekState, cfg: &CheekConfig, dt: f32) {
    let t = (cfg.smoothing * dt).min(1.0);
    state.puff_left += (state.target_puff_left - state.puff_left) * t;
    state.puff_right += (state.target_puff_right - state.puff_right) * t;
}

// ---------------------------------------------------------------------------
// Convenience accessors
// ---------------------------------------------------------------------------

/// Return the current left cheek puff value.
#[allow(dead_code)]
pub fn cheek_puff_left(state: &CheekState) -> f32 {
    state.puff_left
}

/// Return the current right cheek puff value.
#[allow(dead_code)]
pub fn cheek_puff_right(state: &CheekState) -> f32 {
    state.puff_right
}

/// Return the current left cheek hollow value.
#[allow(dead_code)]
pub fn cheek_hollow_left(state: &CheekState) -> f32 {
    state.hollow_left
}

/// Return the current right cheek hollow value.
#[allow(dead_code)]
pub fn cheek_hollow_right(state: &CheekState) -> f32 {
    state.hollow_right
}

// ---------------------------------------------------------------------------
// Blending and morph-weight output
// ---------------------------------------------------------------------------

/// Linearly blend two cheek states by weight `t` (0 = a, 1 = b).
#[allow(dead_code)]
pub fn blend_cheek_states(a: &CheekState, b: &CheekState, t: f32) -> CheekState {
    let t = clamp01(t);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    CheekState {
        puff_left: lerp(a.puff_left, b.puff_left),
        puff_right: lerp(a.puff_right, b.puff_right),
        hollow_left: lerp(a.hollow_left, b.hollow_left),
        hollow_right: lerp(a.hollow_right, b.hollow_right),
        raise_left: lerp(a.raise_left, b.raise_left),
        raise_right: lerp(a.raise_right, b.raise_right),
        target_puff_left: lerp(a.target_puff_left, b.target_puff_left),
        target_puff_right: lerp(a.target_puff_right, b.target_puff_right),
    }
}

/// Convert the current cheek state into a list of morph-target weights.
#[allow(dead_code)]
pub fn cheek_to_morph_weights(state: &CheekState) -> CheekMorphWeights {
    vec![
        ("cheek_puff_left".to_string(), state.puff_left),
        ("cheek_puff_right".to_string(), state.puff_right),
        ("cheek_hollow_left".to_string(), -state.hollow_left),
        ("cheek_hollow_right".to_string(), -state.hollow_right),
        ("cheek_raise_left".to_string(), state.raise_left),
        ("cheek_raise_right".to_string(), state.raise_right),
    ]
}

// ---------------------------------------------------------------------------
// Reset
// ---------------------------------------------------------------------------

/// Reset all cheek values to neutral.
#[allow(dead_code)]
pub fn reset_cheeks(state: &mut CheekState) {
    *state = new_cheek_state();
}

// ---------------------------------------------------------------------------
// Smile effect
// ---------------------------------------------------------------------------

/// Automatically puff and raise cheeks proportional to a smile intensity (0..1).
#[allow(dead_code)]
pub fn apply_smile_effect(state: &mut CheekState, cfg: &CheekConfig, smile: f32) {
    let smile = clamp01(smile);
    let puff = smile * cfg.smile_puff_factor;
    let raise = smile * cfg.smile_puff_factor;
    set_cheek_puff(state, CheekSide::Both, puff);
    set_cheek_raise(state, CheekSide::Both, raise);
    // Immediately apply (no separate update step needed for raise)
    state.puff_left = state.target_puff_left;
    state.puff_right = state.target_puff_right;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cheek_config_values() {
        let cfg = default_cheek_config();
        assert_eq!(cfg.max_puff, 1.0);
        assert_eq!(cfg.max_hollow, -1.0);
        assert!(cfg.smoothing > 0.0);
    }

    #[test]
    fn test_new_cheek_state_zeroed() {
        let s = new_cheek_state();
        assert_eq!(s.puff_left, 0.0);
        assert_eq!(s.puff_right, 0.0);
        assert_eq!(s.hollow_left, 0.0);
        assert_eq!(s.hollow_right, 0.0);
    }

    #[test]
    fn test_set_cheek_puff_left() {
        let mut s = new_cheek_state();
        set_cheek_puff(&mut s, CheekSide::Left, 0.7);
        assert!((s.target_puff_left - 0.7).abs() < 1e-5);
        assert_eq!(s.target_puff_right, 0.0);
    }

    #[test]
    fn test_set_cheek_puff_right() {
        let mut s = new_cheek_state();
        set_cheek_puff(&mut s, CheekSide::Right, 0.5);
        assert!((s.target_puff_right - 0.5).abs() < 1e-5);
        assert_eq!(s.target_puff_left, 0.0);
    }

    #[test]
    fn test_set_cheek_puff_both() {
        let mut s = new_cheek_state();
        set_cheek_puff(&mut s, CheekSide::Both, 0.9);
        assert!((s.target_puff_left - 0.9).abs() < 1e-5);
        assert!((s.target_puff_right - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_set_cheek_puff_clamped_above() {
        let mut s = new_cheek_state();
        set_cheek_puff(&mut s, CheekSide::Both, 2.0);
        assert!((s.target_puff_left - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_cheek_hollow_clamped() {
        let mut s = new_cheek_state();
        set_cheek_hollow(&mut s, CheekSide::Both, -0.5);
        assert!((s.hollow_left - (-0.5)).abs() < 1e-5);
        set_cheek_hollow(&mut s, CheekSide::Both, -2.0);
        assert!((s.hollow_left - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_set_cheek_raise() {
        let mut s = new_cheek_state();
        set_cheek_raise(&mut s, CheekSide::Left, 0.4);
        assert!((s.raise_left - 0.4).abs() < 1e-5);
        assert_eq!(s.raise_right, 0.0);
    }

    #[test]
    fn test_update_cheeks_smoothing() {
        let cfg = default_cheek_config();
        let mut s = new_cheek_state();
        set_cheek_puff(&mut s, CheekSide::Left, 1.0);
        update_cheeks(&mut s, &cfg, 1.0);
        assert!(s.puff_left > 0.0);
        assert!(s.puff_left <= 1.0);
    }

    #[test]
    fn test_cheek_puff_accessors() {
        let mut s = new_cheek_state();
        s.puff_left = 0.3;
        s.puff_right = 0.6;
        assert!((cheek_puff_left(&s) - 0.3).abs() < 1e-5);
        assert!((cheek_puff_right(&s) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_cheek_hollow_accessors() {
        let mut s = new_cheek_state();
        s.hollow_left = -0.4;
        s.hollow_right = -0.8;
        assert!((cheek_hollow_left(&s) - (-0.4)).abs() < 1e-5);
        assert!((cheek_hollow_right(&s) - (-0.8)).abs() < 1e-5);
    }

    #[test]
    fn test_blend_cheek_states_midpoint() {
        let mut a = new_cheek_state();
        let mut b = new_cheek_state();
        a.puff_left = 0.0;
        b.puff_left = 1.0;
        let blended = blend_cheek_states(&a, &b, 0.5);
        assert!((blended.puff_left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_cheek_states_at_zero() {
        let a = new_cheek_state();
        let b = new_cheek_state();
        let blended = blend_cheek_states(&a, &b, 0.0);
        assert_eq!(blended.puff_left, a.puff_left);
    }

    #[test]
    fn test_cheek_to_morph_weights_keys() {
        let s = new_cheek_state();
        let weights = cheek_to_morph_weights(&s);
        assert_eq!(weights.len(), 6);
        let keys: Vec<&str> = weights.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"cheek_puff_left"));
        assert!(keys.contains(&"cheek_hollow_left"));
        assert!(keys.contains(&"cheek_raise_right"));
    }

    #[test]
    fn test_reset_cheeks() {
        let mut s = new_cheek_state();
        s.puff_left = 0.9;
        s.hollow_right = -0.5;
        reset_cheeks(&mut s);
        assert_eq!(s.puff_left, 0.0);
        assert_eq!(s.hollow_right, 0.0);
    }

    #[test]
    fn test_apply_smile_effect_puffs_cheeks() {
        let cfg = default_cheek_config();
        let mut s = new_cheek_state();
        apply_smile_effect(&mut s, &cfg, 1.0);
        assert!(s.puff_left > 0.0);
        assert!(s.puff_right > 0.0);
    }

    #[test]
    fn test_apply_smile_effect_zero_smile() {
        let cfg = default_cheek_config();
        let mut s = new_cheek_state();
        apply_smile_effect(&mut s, &cfg, 0.0);
        assert_eq!(s.puff_left, 0.0);
        assert_eq!(s.puff_right, 0.0);
    }
}
