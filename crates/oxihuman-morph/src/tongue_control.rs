//! Tongue movement for speech and expression animation.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Tongue shape categories.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TongueShape {
    Neutral,
    Pointed,
    Wide,
    Curled,
    Retracted,
}

/// Configuration for tongue movement ranges and dynamics.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TongueConfig {
    /// Maximum extension (0 = retracted, 1 = fully protruded).
    pub max_extension: f32,
    /// Maximum elevation angle in degrees.
    pub max_elevation_deg: f32,
    /// Maximum lateral offset (-1 left, 1 right).
    pub max_lateral: f32,
    /// Smoothing factor for transitions (higher = snappier).
    pub smoothing: f32,
}

/// Runtime state of the tongue.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TongueState {
    /// Current tongue shape.
    pub shape: TongueShape,
    /// Extension amount (0 = retracted, 1 = fully protruded).
    pub extension: f32,
    /// Target extension to transition toward.
    pub target_extension: f32,
    /// Elevation angle in degrees (positive = up, negative = down).
    pub elevation: f32,
    /// Lateral offset (-1 left, 0 center, 1 right).
    pub lateral: f32,
}

/// Result type for phoneme-to-tongue mapping.
#[allow(dead_code)]
pub type PhonemeMap = HashMap<String, (TongueShape, f32, f32)>;

/// Result type for morph weight outputs.
#[allow(dead_code)]
pub type MorphWeights = Vec<(String, f32)>;

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a default tongue configuration with sensible defaults.
#[allow(dead_code)]
pub fn default_tongue_config() -> TongueConfig {
    TongueConfig {
        max_extension: 1.0,
        max_elevation_deg: 45.0,
        max_lateral: 1.0,
        smoothing: 8.0,
    }
}

/// Create a new tongue state at rest (neutral, retracted).
#[allow(dead_code)]
pub fn new_tongue_state() -> TongueState {
    TongueState {
        shape: TongueShape::Neutral,
        extension: 0.0,
        target_extension: 0.0,
        elevation: 0.0,
        lateral: 0.0,
    }
}

// ---------------------------------------------------------------------------
// Setters
// ---------------------------------------------------------------------------

/// Set the tongue shape.
#[allow(dead_code)]
pub fn set_tongue_shape(state: &mut TongueState, shape: TongueShape) {
    state.shape = shape;
}

/// Set the tongue extension (protrusion) target, clamped to 0..1.
#[allow(dead_code)]
pub fn set_tongue_extension(state: &mut TongueState, config: &TongueConfig, amount: f32) {
    state.target_extension = amount.clamp(0.0, config.max_extension);
}

/// Set the tongue elevation angle, clamped to config limits.
#[allow(dead_code)]
pub fn set_tongue_elevation(state: &mut TongueState, config: &TongueConfig, degrees: f32) {
    state.elevation = degrees.clamp(-config.max_elevation_deg, config.max_elevation_deg);
}

/// Set the tongue lateral offset, clamped to config limits.
#[allow(dead_code)]
pub fn set_tongue_lateral(state: &mut TongueState, config: &TongueConfig, offset: f32) {
    state.lateral = offset.clamp(-config.max_lateral, config.max_lateral);
}

// ---------------------------------------------------------------------------
// Update / Transition
// ---------------------------------------------------------------------------

/// Smoothly transition the tongue extension toward its target.
/// `dt` is the delta time in seconds.
#[allow(dead_code)]
pub fn update_tongue(state: &mut TongueState, config: &TongueConfig, dt: f32) {
    let diff = state.target_extension - state.extension;
    let step = diff * (config.smoothing * dt).min(1.0);
    state.extension += step;
    state.extension = state.extension.clamp(0.0, config.max_extension);
}

// ---------------------------------------------------------------------------
// Phoneme mapping
// ---------------------------------------------------------------------------

/// Build a default phoneme-to-tongue-position map.
/// Each entry maps a phoneme label to `(shape, extension, elevation)`.
#[allow(dead_code)]
pub fn build_tongue_phoneme_map() -> PhonemeMap {
    let mut m = HashMap::new();
    // Alveolars: tongue tip up
    m.insert("T".to_string(), (TongueShape::Pointed, 0.3, 30.0));
    m.insert("D".to_string(), (TongueShape::Pointed, 0.3, 28.0));
    m.insert("N".to_string(), (TongueShape::Pointed, 0.25, 25.0));
    m.insert("L".to_string(), (TongueShape::Pointed, 0.35, 30.0));
    // Dental fricatives
    m.insert("TH".to_string(), (TongueShape::Wide, 0.5, 10.0));
    m.insert("DH".to_string(), (TongueShape::Wide, 0.45, 8.0));
    // Retroflex
    m.insert("R".to_string(), (TongueShape::Curled, 0.2, 15.0));
    // Velar
    m.insert("K".to_string(), (TongueShape::Retracted, 0.0, -10.0));
    m.insert("G".to_string(), (TongueShape::Retracted, 0.0, -8.0));
    m.insert("NG".to_string(), (TongueShape::Retracted, 0.0, -5.0));
    // Neutral vowels
    m.insert("AH".to_string(), (TongueShape::Neutral, 0.1, -5.0));
    m.insert("EH".to_string(), (TongueShape::Wide, 0.15, 5.0));
    m
}

/// Map a phoneme to a tongue position, applying to the given state.
/// Returns `true` if the phoneme was found in the map.
#[allow(dead_code)]
pub fn tongue_for_phoneme(state: &mut TongueState, map: &PhonemeMap, phoneme: &str) -> bool {
    if let Some(&(shape, ext, elev)) = map.get(phoneme) {
        state.shape = shape;
        state.target_extension = ext;
        state.elevation = elev;
        true
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Conversion
// ---------------------------------------------------------------------------

/// Convert tongue state to morph target weights.
#[allow(dead_code)]
pub fn tongue_to_morph_weights(state: &TongueState) -> MorphWeights {
    let mut weights = Vec::new();
    weights.push(("tongue_extension".to_string(), state.extension));
    weights.push(("tongue_elevation".to_string(), state.elevation / 45.0));
    weights.push(("tongue_lateral".to_string(), state.lateral));

    // Shape blends
    let shape_weight = match state.shape {
        TongueShape::Neutral => ("tongue_neutral", 1.0_f32),
        TongueShape::Pointed => ("tongue_pointed", 1.0),
        TongueShape::Wide => ("tongue_wide", 1.0),
        TongueShape::Curled => ("tongue_curled", 1.0),
        TongueShape::Retracted => ("tongue_retracted", 1.0),
    };
    weights.push((shape_weight.0.to_string(), shape_weight.1));
    weights
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

/// Reset the tongue to its default neutral position.
#[allow(dead_code)]
pub fn reset_tongue(state: &mut TongueState) {
    state.shape = TongueShape::Neutral;
    state.extension = 0.0;
    state.target_extension = 0.0;
    state.elevation = 0.0;
    state.lateral = 0.0;
}

/// Blend between two tongue states by factor `t` in [0, 1].
#[allow(dead_code)]
pub fn blend_tongue_states(a: &TongueState, b: &TongueState, t: f32) -> TongueState {
    let t = t.clamp(0.0, 1.0);
    TongueState {
        shape: if t < 0.5 { a.shape } else { b.shape },
        extension: a.extension + (b.extension - a.extension) * t,
        target_extension: a.target_extension + (b.target_extension - a.target_extension) * t,
        elevation: a.elevation + (b.elevation - a.elevation) * t,
        lateral: a.lateral + (b.lateral - a.lateral) * t,
    }
}

/// Return the current tongue extension amount.
#[allow(dead_code)]
pub fn tongue_extension_amount(state: &TongueState) -> f32 {
    state.extension
}

/// Return a human-readable name for the current tongue shape.
#[allow(dead_code)]
pub fn tongue_shape_name(shape: TongueShape) -> &'static str {
    match shape {
        TongueShape::Neutral => "Neutral",
        TongueShape::Pointed => "Pointed",
        TongueShape::Wide => "Wide",
        TongueShape::Curled => "Curled",
        TongueShape::Retracted => "Retracted",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_tongue_config();
        assert!(cfg.max_extension > 0.0);
        assert!(cfg.smoothing > 0.0);
    }

    #[test]
    fn test_new_state_is_neutral() {
        let s = new_tongue_state();
        assert_eq!(s.shape, TongueShape::Neutral);
        assert_eq!(s.extension, 0.0);
        assert_eq!(s.elevation, 0.0);
        assert_eq!(s.lateral, 0.0);
    }

    #[test]
    fn test_set_shape() {
        let mut s = new_tongue_state();
        set_tongue_shape(&mut s, TongueShape::Pointed);
        assert_eq!(s.shape, TongueShape::Pointed);
    }

    #[test]
    fn test_set_extension_clamps() {
        let cfg = default_tongue_config();
        let mut s = new_tongue_state();
        set_tongue_extension(&mut s, &cfg, 2.0);
        assert_eq!(s.target_extension, cfg.max_extension);
        set_tongue_extension(&mut s, &cfg, -1.0);
        assert_eq!(s.target_extension, 0.0);
    }

    #[test]
    fn test_set_elevation_clamps() {
        let cfg = default_tongue_config();
        let mut s = new_tongue_state();
        set_tongue_elevation(&mut s, &cfg, 999.0);
        assert_eq!(s.elevation, cfg.max_elevation_deg);
        set_tongue_elevation(&mut s, &cfg, -999.0);
        assert_eq!(s.elevation, -cfg.max_elevation_deg);
    }

    #[test]
    fn test_set_lateral_clamps() {
        let cfg = default_tongue_config();
        let mut s = new_tongue_state();
        set_tongue_lateral(&mut s, &cfg, 5.0);
        assert_eq!(s.lateral, cfg.max_lateral);
        set_tongue_lateral(&mut s, &cfg, -5.0);
        assert_eq!(s.lateral, -cfg.max_lateral);
    }

    #[test]
    fn test_update_moves_toward_target() {
        let cfg = default_tongue_config();
        let mut s = new_tongue_state();
        s.target_extension = 1.0;
        update_tongue(&mut s, &cfg, 0.1);
        assert!(s.extension > 0.0);
        assert!(s.extension < 1.0);
    }

    #[test]
    fn test_update_large_dt_reaches_target() {
        let cfg = default_tongue_config();
        let mut s = new_tongue_state();
        s.target_extension = 0.6;
        update_tongue(&mut s, &cfg, 10.0);
        assert!((s.extension - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_phoneme_map_not_empty() {
        let map = build_tongue_phoneme_map();
        assert!(!map.is_empty());
    }

    #[test]
    fn test_tongue_for_phoneme_found() {
        let map = build_tongue_phoneme_map();
        let mut s = new_tongue_state();
        assert!(tongue_for_phoneme(&mut s, &map, "T"));
        assert_eq!(s.shape, TongueShape::Pointed);
        assert!(s.target_extension > 0.0);
    }

    #[test]
    fn test_tongue_for_phoneme_not_found() {
        let map = build_tongue_phoneme_map();
        let mut s = new_tongue_state();
        assert!(!tongue_for_phoneme(&mut s, &map, "ZZZ"));
    }

    #[test]
    fn test_tongue_to_morph_weights() {
        let s = new_tongue_state();
        let weights = tongue_to_morph_weights(&s);
        assert!(!weights.is_empty());
        assert!(weights.iter().any(|(n, _)| n == "tongue_neutral"));
    }

    #[test]
    fn test_reset_tongue() {
        let mut s = new_tongue_state();
        s.shape = TongueShape::Curled;
        s.extension = 0.8;
        s.elevation = 30.0;
        s.lateral = -0.5;
        reset_tongue(&mut s);
        assert_eq!(s.shape, TongueShape::Neutral);
        assert_eq!(s.extension, 0.0);
        assert_eq!(s.elevation, 0.0);
        assert_eq!(s.lateral, 0.0);
    }

    #[test]
    fn test_blend_tongue_states_at_zero() {
        let a = new_tongue_state();
        let mut b = new_tongue_state();
        b.extension = 1.0;
        b.shape = TongueShape::Wide;
        let result = blend_tongue_states(&a, &b, 0.0);
        assert_eq!(result.extension, 0.0);
        assert_eq!(result.shape, TongueShape::Neutral);
    }

    #[test]
    fn test_blend_tongue_states_at_one() {
        let a = new_tongue_state();
        let mut b = new_tongue_state();
        b.extension = 1.0;
        b.shape = TongueShape::Curled;
        let result = blend_tongue_states(&a, &b, 1.0);
        assert!((result.extension - 1.0).abs() < 1e-6);
        assert_eq!(result.shape, TongueShape::Curled);
    }

    #[test]
    fn test_tongue_extension_amount() {
        let mut s = new_tongue_state();
        s.extension = 0.42;
        assert!((tongue_extension_amount(&s) - 0.42).abs() < 1e-6);
    }

    #[test]
    fn test_tongue_shape_name() {
        assert_eq!(tongue_shape_name(TongueShape::Neutral), "Neutral");
        assert_eq!(tongue_shape_name(TongueShape::Pointed), "Pointed");
        assert_eq!(tongue_shape_name(TongueShape::Wide), "Wide");
        assert_eq!(tongue_shape_name(TongueShape::Curled), "Curled");
        assert_eq!(tongue_shape_name(TongueShape::Retracted), "Retracted");
    }
}
