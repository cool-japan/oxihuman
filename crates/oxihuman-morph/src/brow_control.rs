//! Eyebrow control system for inner/outer raise, furrow, and arch.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Which side of the face the brow operation applies to.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowSide {
    Left,
    Right,
    Both,
}

/// Configuration for brow movement ranges and dynamics.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BrowConfig {
    /// Maximum inner raise (0..1).
    pub max_inner_raise: f32,
    /// Maximum outer raise (0..1).
    pub max_outer_raise: f32,
    /// Maximum lower amount (0..1).
    pub max_lower: f32,
    /// Maximum furrow/scrunch amount (0..1).
    pub max_furrow: f32,
    /// Maximum arch amount (0..1).
    pub max_arch: f32,
    /// Smoothing factor for transitions (higher = snappier).
    pub smoothing: f32,
}

/// Runtime state for both eyebrows.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BrowState {
    /// Left inner brow raise (0..1).
    pub inner_raise_left: f32,
    /// Right inner brow raise (0..1).
    pub inner_raise_right: f32,
    /// Left outer brow raise (0..1).
    pub outer_raise_left: f32,
    /// Right outer brow raise (0..1).
    pub outer_raise_right: f32,
    /// Left brow lower amount (0..1).
    pub lower_left: f32,
    /// Right brow lower amount (0..1).
    pub lower_right: f32,
    /// Furrow / scrunch intensity (0..1, bilateral).
    pub furrow: f32,
    /// Arch intensity (0..1, per side).
    pub arch_left: f32,
    /// Arch intensity right side.
    pub arch_right: f32,
    /// Target inner raise left (for smooth interpolation).
    pub target_inner_left: f32,
    /// Target inner raise right.
    pub target_inner_right: f32,
}

/// Type alias for morph-weight output pairs.
#[allow(dead_code)]
pub type BrowMorphWeights = Vec<(String, f32)>;

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Return a sensible default brow configuration.
#[allow(dead_code)]
pub fn default_brow_config() -> BrowConfig {
    BrowConfig {
        max_inner_raise: 1.0,
        max_outer_raise: 1.0,
        max_lower: 1.0,
        max_furrow: 1.0,
        max_arch: 1.0,
        smoothing: 10.0,
    }
}

/// Create a new brow state at rest (all zeros).
#[allow(dead_code)]
pub fn new_brow_state() -> BrowState {
    BrowState {
        inner_raise_left: 0.0,
        inner_raise_right: 0.0,
        outer_raise_left: 0.0,
        outer_raise_right: 0.0,
        lower_left: 0.0,
        lower_right: 0.0,
        furrow: 0.0,
        arch_left: 0.0,
        arch_right: 0.0,
        target_inner_left: 0.0,
        target_inner_right: 0.0,
    }
}

// ---------------------------------------------------------------------------
// Setters
// ---------------------------------------------------------------------------

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Set inner or outer brow raise on the given side.
/// `inner` selects inner raise when `true`, outer raise when `false`.
#[allow(dead_code)]
pub fn set_brow_raise(state: &mut BrowState, side: BrowSide, amount: f32, inner: bool) {
    let v = clamp01(amount);
    match (side, inner) {
        (BrowSide::Left, true) => state.target_inner_left = v,
        (BrowSide::Right, true) => state.target_inner_right = v,
        (BrowSide::Both, true) => {
            state.target_inner_left = v;
            state.target_inner_right = v;
        }
        (BrowSide::Left, false) => state.outer_raise_left = v,
        (BrowSide::Right, false) => state.outer_raise_right = v,
        (BrowSide::Both, false) => {
            state.outer_raise_left = v;
            state.outer_raise_right = v;
        }
    }
}

/// Lower brows on the given side (0 = neutral, 1 = fully lowered).
#[allow(dead_code)]
pub fn set_brow_lower(state: &mut BrowState, side: BrowSide, amount: f32) {
    let v = clamp01(amount);
    match side {
        BrowSide::Left => state.lower_left = v,
        BrowSide::Right => state.lower_right = v,
        BrowSide::Both => {
            state.lower_left = v;
            state.lower_right = v;
        }
    }
}

/// Set bilateral brow furrow / scrunch amount (0..1).
#[allow(dead_code)]
pub fn set_brow_furrow(state: &mut BrowState, amount: f32) {
    state.furrow = clamp01(amount);
}

/// Set brow arch amount on the given side (0..1).
#[allow(dead_code)]
pub fn set_brow_arch(state: &mut BrowState, side: BrowSide, amount: f32) {
    let v = clamp01(amount);
    match side {
        BrowSide::Left => state.arch_left = v,
        BrowSide::Right => state.arch_right = v,
        BrowSide::Both => {
            state.arch_left = v;
            state.arch_right = v;
        }
    }
}

// ---------------------------------------------------------------------------
// Update / smoothing
// ---------------------------------------------------------------------------

/// Advance brow state toward targets using exponential smoothing.
/// `dt` is the timestep in seconds.
#[allow(dead_code)]
pub fn update_brows(state: &mut BrowState, cfg: &BrowConfig, dt: f32) {
    let t = (cfg.smoothing * dt).min(1.0);
    state.inner_raise_left += (state.target_inner_left - state.inner_raise_left) * t;
    state.inner_raise_right += (state.target_inner_right - state.inner_raise_right) * t;
}

// ---------------------------------------------------------------------------
// Convenience accessors
// ---------------------------------------------------------------------------

/// Return the current left brow raise (average of inner + outer).
#[allow(dead_code)]
pub fn brow_raise_left(state: &BrowState) -> f32 {
    (state.inner_raise_left + state.outer_raise_left) * 0.5
}

/// Return the current right brow raise (average of inner + outer).
#[allow(dead_code)]
pub fn brow_raise_right(state: &BrowState) -> f32 {
    (state.inner_raise_right + state.outer_raise_right) * 0.5
}

/// Return the current furrow amount.
#[allow(dead_code)]
pub fn brow_furrow_amount(state: &BrowState) -> f32 {
    state.furrow
}

// ---------------------------------------------------------------------------
// Blending and morph-weight output
// ---------------------------------------------------------------------------

/// Linearly blend two brow states by `t` (0 = a, 1 = b).
#[allow(dead_code)]
pub fn blend_brow_states(a: &BrowState, b: &BrowState, t: f32) -> BrowState {
    let t = clamp01(t);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    BrowState {
        inner_raise_left: lerp(a.inner_raise_left, b.inner_raise_left),
        inner_raise_right: lerp(a.inner_raise_right, b.inner_raise_right),
        outer_raise_left: lerp(a.outer_raise_left, b.outer_raise_left),
        outer_raise_right: lerp(a.outer_raise_right, b.outer_raise_right),
        lower_left: lerp(a.lower_left, b.lower_left),
        lower_right: lerp(a.lower_right, b.lower_right),
        furrow: lerp(a.furrow, b.furrow),
        arch_left: lerp(a.arch_left, b.arch_left),
        arch_right: lerp(a.arch_right, b.arch_right),
        target_inner_left: lerp(a.target_inner_left, b.target_inner_left),
        target_inner_right: lerp(a.target_inner_right, b.target_inner_right),
    }
}

/// Convert the current brow state into morph-target weights.
#[allow(dead_code)]
pub fn brow_to_morph_weights(state: &BrowState) -> BrowMorphWeights {
    vec![
        ("brow_inner_raise_left".to_string(), state.inner_raise_left),
        (
            "brow_inner_raise_right".to_string(),
            state.inner_raise_right,
        ),
        ("brow_outer_raise_left".to_string(), state.outer_raise_left),
        (
            "brow_outer_raise_right".to_string(),
            state.outer_raise_right,
        ),
        ("brow_lower_left".to_string(), state.lower_left),
        ("brow_lower_right".to_string(), state.lower_right),
        ("brow_furrow".to_string(), state.furrow),
        ("brow_arch_left".to_string(), state.arch_left),
        ("brow_arch_right".to_string(), state.arch_right),
    ]
}

// ---------------------------------------------------------------------------
// Reset
// ---------------------------------------------------------------------------

/// Reset all brow values to neutral.
#[allow(dead_code)]
pub fn reset_brows(state: &mut BrowState) {
    *state = new_brow_state();
}

// ---------------------------------------------------------------------------
// Emotion mapping
// ---------------------------------------------------------------------------

/// Map an emotion label to a preset brow configuration.
/// Supported labels: "angry", "sad", "surprised", "happy", "fearful", "neutral".
#[allow(dead_code)]
pub fn emotion_to_brow(emotion: &str, cfg: &BrowConfig) -> BrowState {
    let mut s = new_brow_state();
    match emotion {
        "angry" => {
            set_brow_lower(&mut s, BrowSide::Both, 0.6 * cfg.max_lower);
            set_brow_furrow(&mut s, 0.8 * cfg.max_furrow);
        }
        "sad" => {
            set_brow_raise(&mut s, BrowSide::Both, 0.5 * cfg.max_inner_raise, true);
            set_brow_lower(&mut s, BrowSide::Both, 0.2 * cfg.max_lower);
        }
        "surprised" => {
            set_brow_raise(&mut s, BrowSide::Both, 0.9 * cfg.max_inner_raise, true);
            set_brow_raise(&mut s, BrowSide::Both, 0.9 * cfg.max_outer_raise, false);
            // Immediately apply targets
            s.inner_raise_left = s.target_inner_left;
            s.inner_raise_right = s.target_inner_right;
        }
        "happy" => {
            set_brow_raise(&mut s, BrowSide::Both, 0.2 * cfg.max_outer_raise, false);
            s.arch_left = 0.3 * cfg.max_arch;
            s.arch_right = 0.3 * cfg.max_arch;
        }
        "fearful" => {
            set_brow_raise(&mut s, BrowSide::Both, 0.7 * cfg.max_inner_raise, true);
            set_brow_furrow(&mut s, 0.4 * cfg.max_furrow);
            s.inner_raise_left = s.target_inner_left;
            s.inner_raise_right = s.target_inner_right;
        }
        _ => {} // "neutral" or unknown → rest
    }
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_brow_config_non_zero() {
        let cfg = default_brow_config();
        assert!(cfg.max_inner_raise > 0.0);
        assert!(cfg.max_furrow > 0.0);
        assert!(cfg.smoothing > 0.0);
    }

    #[test]
    fn test_new_brow_state_zeroed() {
        let s = new_brow_state();
        assert_eq!(s.inner_raise_left, 0.0);
        assert_eq!(s.furrow, 0.0);
        assert_eq!(s.arch_right, 0.0);
    }

    #[test]
    fn test_set_brow_raise_inner_left() {
        let mut s = new_brow_state();
        set_brow_raise(&mut s, BrowSide::Left, 0.8, true);
        assert!((s.target_inner_left - 0.8).abs() < 1e-5);
        assert_eq!(s.target_inner_right, 0.0);
    }

    #[test]
    fn test_set_brow_raise_outer_both() {
        let mut s = new_brow_state();
        set_brow_raise(&mut s, BrowSide::Both, 0.6, false);
        assert!((s.outer_raise_left - 0.6).abs() < 1e-5);
        assert!((s.outer_raise_right - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_set_brow_raise_clamped() {
        let mut s = new_brow_state();
        set_brow_raise(&mut s, BrowSide::Both, 2.0, true);
        assert!((s.target_inner_left - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_brow_lower() {
        let mut s = new_brow_state();
        set_brow_lower(&mut s, BrowSide::Right, 0.5);
        assert!((s.lower_right - 0.5).abs() < 1e-5);
        assert_eq!(s.lower_left, 0.0);
    }

    #[test]
    fn test_set_brow_furrow() {
        let mut s = new_brow_state();
        set_brow_furrow(&mut s, 0.7);
        assert!((s.furrow - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_set_brow_arch() {
        let mut s = new_brow_state();
        set_brow_arch(&mut s, BrowSide::Left, 0.4);
        assert!((s.arch_left - 0.4).abs() < 1e-5);
        assert_eq!(s.arch_right, 0.0);
    }

    #[test]
    fn test_update_brows_smoothing() {
        let cfg = default_brow_config();
        let mut s = new_brow_state();
        set_brow_raise(&mut s, BrowSide::Left, 1.0, true);
        update_brows(&mut s, &cfg, 1.0);
        assert!(s.inner_raise_left > 0.0);
    }

    #[test]
    fn test_brow_raise_left_accessor() {
        let mut s = new_brow_state();
        s.inner_raise_left = 0.4;
        s.outer_raise_left = 0.6;
        let avg = brow_raise_left(&s);
        assert!((avg - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_brow_furrow_amount_accessor() {
        let mut s = new_brow_state();
        s.furrow = 0.3;
        assert!((brow_furrow_amount(&s) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_blend_brow_states_midpoint() {
        let mut a = new_brow_state();
        let mut b = new_brow_state();
        a.furrow = 0.0;
        b.furrow = 1.0;
        let blended = blend_brow_states(&a, &b, 0.5);
        assert!((blended.furrow - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_brow_to_morph_weights_count() {
        let s = new_brow_state();
        let w = brow_to_morph_weights(&s);
        assert_eq!(w.len(), 9);
    }

    #[test]
    fn test_brow_to_morph_weights_keys_present() {
        let s = new_brow_state();
        let w = brow_to_morph_weights(&s);
        let keys: Vec<&str> = w.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"brow_furrow"));
        assert!(keys.contains(&"brow_arch_left"));
    }

    #[test]
    fn test_reset_brows() {
        let mut s = new_brow_state();
        s.furrow = 0.9;
        s.arch_left = 0.5;
        reset_brows(&mut s);
        assert_eq!(s.furrow, 0.0);
        assert_eq!(s.arch_left, 0.0);
    }

    #[test]
    fn test_emotion_angry_sets_lower_and_furrow() {
        let cfg = default_brow_config();
        let s = emotion_to_brow("angry", &cfg);
        assert!(s.lower_left > 0.0);
        assert!(s.furrow > 0.0);
    }

    #[test]
    fn test_emotion_surprised_raises_brows() {
        let cfg = default_brow_config();
        let s = emotion_to_brow("surprised", &cfg);
        assert!(s.inner_raise_left > 0.0);
        assert!(s.outer_raise_right > 0.0);
    }

    #[test]
    fn test_emotion_neutral_zeroed() {
        let cfg = default_brow_config();
        let s = emotion_to_brow("neutral", &cfg);
        assert_eq!(s.furrow, 0.0);
        assert_eq!(s.inner_raise_left, 0.0);
    }
}
