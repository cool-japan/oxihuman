//! Jaw movement and phoneme-driven jaw opening for speech animation.

use std::collections::HashMap;

/// Configuration for jaw movement ranges and dynamics.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct JawConfig {
    /// Maximum jaw opening angle (normalized 0..1).
    pub max_open: f32,
    /// Minimum jaw opening angle (normalized 0..1).
    pub min_open: f32,
    /// Maximum lateral offset (normalized -1..1).
    pub max_lateral: f32,
    /// Smoothing factor for jaw transitions (higher = snappier).
    pub smoothing: f32,
    /// Maximum angular velocity (units per second).
    pub max_velocity: f32,
}

/// Runtime state of the jaw.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct JawState {
    /// Current jaw opening (0 = closed, 1 = fully open).
    pub current_open: f32,
    /// Target jaw opening to transition toward.
    pub target_open: f32,
    /// Current lateral offset (-1 left, 0 center, 1 right).
    pub lateral_offset: f32,
    /// Current velocity of jaw opening (units/s).
    pub velocity: f32,
}

/// Maps phoneme strings to jaw opening values.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PhonemeJawMap {
    /// Mapping from phoneme label to jaw opening amount (0..1).
    pub entries: HashMap<String, f32>,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a default jaw configuration with sensible defaults.
#[allow(dead_code)]
pub fn default_jaw_config() -> JawConfig {
    JawConfig {
        max_open: 1.0,
        min_open: 0.0,
        max_lateral: 0.5,
        smoothing: 10.0,
        max_velocity: 5.0,
    }
}

/// Create a new jaw state at rest (closed).
#[allow(dead_code)]
pub fn new_jaw_state() -> JawState {
    JawState {
        current_open: 0.0,
        target_open: 0.0,
        lateral_offset: 0.0,
        velocity: 0.0,
    }
}

// ---------------------------------------------------------------------------
// Phoneme map
// ---------------------------------------------------------------------------

/// Build a default phoneme-to-jaw-opening map.
#[allow(dead_code)]
pub fn build_default_phoneme_map() -> PhonemeJawMap {
    let mut entries = HashMap::new();
    // Vowels - wider openings
    entries.insert("AA".to_string(), 0.9);
    entries.insert("AE".to_string(), 0.8);
    entries.insert("AH".to_string(), 0.7);
    entries.insert("AO".to_string(), 0.85);
    entries.insert("AW".to_string(), 0.8);
    entries.insert("AY".to_string(), 0.75);
    entries.insert("EH".to_string(), 0.5);
    entries.insert("ER".to_string(), 0.4);
    entries.insert("EY".to_string(), 0.45);
    entries.insert("IH".to_string(), 0.3);
    entries.insert("IY".to_string(), 0.25);
    entries.insert("OW".to_string(), 0.6);
    entries.insert("OY".to_string(), 0.65);
    entries.insert("UH".to_string(), 0.35);
    entries.insert("UW".to_string(), 0.3);
    // Consonants - smaller openings
    entries.insert("B".to_string(), 0.05);
    entries.insert("CH".to_string(), 0.2);
    entries.insert("D".to_string(), 0.15);
    entries.insert("DH".to_string(), 0.15);
    entries.insert("F".to_string(), 0.1);
    entries.insert("G".to_string(), 0.2);
    entries.insert("HH".to_string(), 0.3);
    entries.insert("JH".to_string(), 0.25);
    entries.insert("K".to_string(), 0.2);
    entries.insert("L".to_string(), 0.2);
    entries.insert("M".to_string(), 0.0);
    entries.insert("N".to_string(), 0.1);
    entries.insert("NG".to_string(), 0.15);
    entries.insert("P".to_string(), 0.0);
    entries.insert("R".to_string(), 0.2);
    entries.insert("S".to_string(), 0.1);
    entries.insert("SH".to_string(), 0.15);
    entries.insert("T".to_string(), 0.1);
    entries.insert("TH".to_string(), 0.15);
    entries.insert("V".to_string(), 0.1);
    entries.insert("W".to_string(), 0.15);
    entries.insert("Y".to_string(), 0.15);
    entries.insert("Z".to_string(), 0.1);
    entries.insert("ZH".to_string(), 0.15);
    // Silence
    entries.insert("SIL".to_string(), 0.0);
    PhonemeJawMap { entries }
}

// ---------------------------------------------------------------------------
// Core operations
// ---------------------------------------------------------------------------

/// Set the target jaw opening, clamped to 0..1.
#[allow(dead_code)]
pub fn set_jaw_open(state: &mut JawState, config: &JawConfig, amount: f32) {
    state.target_open = amount.clamp(config.min_open, config.max_open);
}

/// Look up the jaw opening amount for a given phoneme string.
/// Returns 0.0 if the phoneme is not found.
#[allow(dead_code)]
pub fn jaw_open_for_phoneme(map: &PhonemeJawMap, phoneme: &str) -> f32 {
    map.entries.get(phoneme).copied().unwrap_or(0.0)
}

/// Smoothly update the jaw state toward its target over a time step `dt`.
#[allow(dead_code)]
pub fn update_jaw(state: &mut JawState, config: &JawConfig, dt: f32) {
    if dt <= 0.0 {
        return;
    }
    let diff = state.target_open - state.current_open;
    let raw_velocity = diff * config.smoothing;
    let clamped_velocity = raw_velocity.clamp(-config.max_velocity, config.max_velocity);
    state.velocity = clamped_velocity;
    let delta = clamped_velocity * dt;
    state.current_open = (state.current_open + delta).clamp(config.min_open, config.max_open);
}

/// Return the current jaw open amount.
#[allow(dead_code)]
pub fn jaw_open_amount(state: &JawState) -> f32 {
    state.current_open
}

/// Return the current lateral offset.
#[allow(dead_code)]
pub fn jaw_lateral_offset(state: &JawState) -> f32 {
    state.lateral_offset
}

/// Set the lateral jaw offset, clamped to [-max_lateral, max_lateral].
#[allow(dead_code)]
pub fn set_jaw_lateral(state: &mut JawState, config: &JawConfig, offset: f32) {
    state.lateral_offset = offset.clamp(-config.max_lateral, config.max_lateral);
}

/// Clamp jaw state values to the valid range specified by config.
#[allow(dead_code)]
pub fn clamp_jaw_range(state: &mut JawState, config: &JawConfig) {
    state.current_open = state.current_open.clamp(config.min_open, config.max_open);
    state.target_open = state.target_open.clamp(config.min_open, config.max_open);
    state.lateral_offset = state
        .lateral_offset
        .clamp(-config.max_lateral, config.max_lateral);
    state.velocity = state
        .velocity
        .clamp(-config.max_velocity, config.max_velocity);
}

/// Return the current jaw velocity.
#[allow(dead_code)]
pub fn jaw_velocity(state: &JawState) -> f32 {
    state.velocity
}

/// Reset the jaw to its closed rest position.
#[allow(dead_code)]
pub fn reset_jaw(state: &mut JawState) {
    state.current_open = 0.0;
    state.target_open = 0.0;
    state.lateral_offset = 0.0;
    state.velocity = 0.0;
}

/// Blend two jaw states by a factor `t` (0 = all `a`, 1 = all `b`).
#[allow(dead_code)]
pub fn blend_jaw_states(a: &JawState, b: &JawState, t: f32) -> JawState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    JawState {
        current_open: a.current_open * inv + b.current_open * t,
        target_open: a.target_open * inv + b.target_open * t,
        lateral_offset: a.lateral_offset * inv + b.lateral_offset * t,
        velocity: a.velocity * inv + b.velocity * t,
    }
}

/// Convert jaw state to morph target weights.
///
/// Returns a `HashMap` with keys like `"jaw_open"`, `"jaw_lateral"` mapped to
/// their current weight values.
#[allow(dead_code)]
pub fn jaw_to_morph_weights(state: &JawState) -> HashMap<String, f32> {
    let mut weights = HashMap::new();
    weights.insert("jaw_open".to_string(), state.current_open);
    weights.insert("jaw_lateral".to_string(), state.lateral_offset);
    // Derived weights for speech
    if state.current_open > 0.5 {
        weights.insert("mouth_wide".to_string(), (state.current_open - 0.5) * 2.0);
    } else {
        weights.insert("mouth_wide".to_string(), 0.0);
    }
    if state.current_open < 0.2 {
        weights.insert("lips_together".to_string(), 1.0 - state.current_open * 5.0);
    } else {
        weights.insert("lips_together".to_string(), 0.0);
    }
    weights
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_jaw_config();
        assert!(cfg.max_open > 0.0);
        assert!(cfg.smoothing > 0.0);
        assert!(cfg.max_velocity > 0.0);
    }

    #[test]
    fn test_new_jaw_state() {
        let s = new_jaw_state();
        assert_eq!(s.current_open, 0.0);
        assert_eq!(s.target_open, 0.0);
        assert_eq!(s.lateral_offset, 0.0);
        assert_eq!(s.velocity, 0.0);
    }

    #[test]
    fn test_set_jaw_open_clamps() {
        let cfg = default_jaw_config();
        let mut s = new_jaw_state();
        set_jaw_open(&mut s, &cfg, 1.5);
        assert!((s.target_open - 1.0).abs() < 1e-6);
        set_jaw_open(&mut s, &cfg, -0.5);
        assert!((s.target_open - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_jaw_open_for_phoneme_found() {
        let map = build_default_phoneme_map();
        let val = jaw_open_for_phoneme(&map, "AA");
        assert!(val > 0.8);
    }

    #[test]
    fn test_jaw_open_for_phoneme_missing() {
        let map = build_default_phoneme_map();
        let val = jaw_open_for_phoneme(&map, "ZZZZZ");
        assert_eq!(val, 0.0);
    }

    #[test]
    fn test_update_jaw_toward_target() {
        let cfg = default_jaw_config();
        let mut s = new_jaw_state();
        s.target_open = 1.0;
        update_jaw(&mut s, &cfg, 0.1);
        assert!(s.current_open > 0.0);
        assert!(s.current_open < 1.0);
    }

    #[test]
    fn test_update_jaw_zero_dt() {
        let cfg = default_jaw_config();
        let mut s = new_jaw_state();
        s.target_open = 1.0;
        update_jaw(&mut s, &cfg, 0.0);
        assert_eq!(s.current_open, 0.0);
    }

    #[test]
    fn test_jaw_open_amount() {
        let mut s = new_jaw_state();
        s.current_open = 0.42;
        assert!((jaw_open_amount(&s) - 0.42).abs() < 1e-6);
    }

    #[test]
    fn test_set_jaw_lateral() {
        let cfg = default_jaw_config();
        let mut s = new_jaw_state();
        set_jaw_lateral(&mut s, &cfg, 0.3);
        assert!((s.lateral_offset - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_jaw_lateral_clamps() {
        let cfg = default_jaw_config();
        let mut s = new_jaw_state();
        set_jaw_lateral(&mut s, &cfg, 10.0);
        assert!((s.lateral_offset - cfg.max_lateral).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_jaw_range() {
        let cfg = default_jaw_config();
        let mut s = JawState {
            current_open: 2.0,
            target_open: -1.0,
            lateral_offset: 5.0,
            velocity: 100.0,
        };
        clamp_jaw_range(&mut s, &cfg);
        assert!(s.current_open <= cfg.max_open);
        assert!(s.target_open >= cfg.min_open);
        assert!(s.lateral_offset <= cfg.max_lateral);
        assert!(s.velocity <= cfg.max_velocity);
    }

    #[test]
    fn test_jaw_velocity() {
        let cfg = default_jaw_config();
        let mut s = new_jaw_state();
        s.target_open = 1.0;
        update_jaw(&mut s, &cfg, 0.01);
        assert!(jaw_velocity(&s).abs() > 0.0);
    }

    #[test]
    fn test_reset_jaw() {
        let mut s = JawState {
            current_open: 0.5,
            target_open: 0.8,
            lateral_offset: 0.2,
            velocity: 1.0,
        };
        reset_jaw(&mut s);
        assert_eq!(s.current_open, 0.0);
        assert_eq!(s.target_open, 0.0);
        assert_eq!(s.lateral_offset, 0.0);
        assert_eq!(s.velocity, 0.0);
    }

    #[test]
    fn test_blend_jaw_states_zero() {
        let a = JawState {
            current_open: 0.0,
            target_open: 0.0,
            lateral_offset: 0.0,
            velocity: 0.0,
        };
        let b = JawState {
            current_open: 1.0,
            target_open: 1.0,
            lateral_offset: 0.5,
            velocity: 2.0,
        };
        let r = blend_jaw_states(&a, &b, 0.0);
        assert!((r.current_open - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_jaw_states_one() {
        let a = JawState {
            current_open: 0.0,
            target_open: 0.0,
            lateral_offset: 0.0,
            velocity: 0.0,
        };
        let b = JawState {
            current_open: 1.0,
            target_open: 1.0,
            lateral_offset: 0.5,
            velocity: 2.0,
        };
        let r = blend_jaw_states(&a, &b, 1.0);
        assert!((r.current_open - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_jaw_states_half() {
        let a = JawState {
            current_open: 0.0,
            target_open: 0.0,
            lateral_offset: 0.0,
            velocity: 0.0,
        };
        let b = JawState {
            current_open: 1.0,
            target_open: 1.0,
            lateral_offset: 0.5,
            velocity: 2.0,
        };
        let r = blend_jaw_states(&a, &b, 0.5);
        assert!((r.current_open - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_jaw_to_morph_weights_closed() {
        let s = new_jaw_state();
        let w = jaw_to_morph_weights(&s);
        assert_eq!(*w.get("jaw_open").unwrap(), 0.0);
        assert!(*w.get("lips_together").unwrap() > 0.9);
    }

    #[test]
    fn test_jaw_to_morph_weights_wide_open() {
        let s = JawState {
            current_open: 0.8,
            target_open: 0.8,
            lateral_offset: 0.0,
            velocity: 0.0,
        };
        let w = jaw_to_morph_weights(&s);
        assert!(*w.get("mouth_wide").unwrap() > 0.0);
        assert_eq!(*w.get("lips_together").unwrap(), 0.0);
    }

    #[test]
    fn test_phoneme_map_has_many_entries() {
        let map = build_default_phoneme_map();
        assert!(map.entries.len() >= 30);
    }

    #[test]
    fn test_update_jaw_converges() {
        let cfg = default_jaw_config();
        let mut s = new_jaw_state();
        s.target_open = 0.5;
        for _ in 0..200 {
            update_jaw(&mut s, &cfg, 0.016);
        }
        assert!((s.current_open - 0.5).abs() < 0.01);
    }
}
