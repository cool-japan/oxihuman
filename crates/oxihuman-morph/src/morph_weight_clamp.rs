#![allow(dead_code)]
//! Morph weight clamp: restricts morph weights to a configurable range.

/// A clamp for morph weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphWeightClamp {
    min: f32,
    max: f32,
}

/// Create a new clamp with the given range.
#[allow(dead_code)]
pub fn new_weight_clamp(min: f32, max: f32) -> MorphWeightClamp {
    MorphWeightClamp { min, max }
}

/// Clamp a weight value.
#[allow(dead_code)]
pub fn clamp_weight(clamp: &MorphWeightClamp, value: f32) -> f32 {
    value.clamp(clamp.min, clamp.max)
}

/// Return the minimum.
#[allow(dead_code)]
pub fn clamp_min(clamp: &MorphWeightClamp) -> f32 {
    clamp.min
}

/// Return the maximum.
#[allow(dead_code)]
pub fn clamp_max(clamp: &MorphWeightClamp) -> f32 {
    clamp.max
}

/// Check if a value would be clamped.
#[allow(dead_code)]
pub fn is_clamped(clamp: &MorphWeightClamp, value: f32) -> bool {
    !(clamp.min..=clamp.max).contains(&value)
}

/// Return the clamp range as (min, max).
#[allow(dead_code)]
pub fn clamp_range(clamp: &MorphWeightClamp) -> (f32, f32) {
    (clamp.min, clamp.max)
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn clamp_to_json(clamp: &MorphWeightClamp) -> String {
    format!("{{\"min\":{},\"max\":{}}}", clamp.min, clamp.max)
}

/// Reset to default range [0, 1].
#[allow(dead_code)]
pub fn reset_clamp(clamp: &mut MorphWeightClamp) {
    clamp.min = 0.0;
    clamp.max = 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clamp() {
        let c = new_weight_clamp(0.0, 1.0);
        assert!((clamp_min(&c) - 0.0).abs() < 1e-6);
        assert!((clamp_max(&c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_weight_within() {
        let c = new_weight_clamp(0.0, 1.0);
        assert!((clamp_weight(&c, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_weight_below() {
        let c = new_weight_clamp(0.0, 1.0);
        assert!((clamp_weight(&c, -1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_weight_above() {
        let c = new_weight_clamp(0.0, 1.0);
        assert!((clamp_weight(&c, 2.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_clamped() {
        let c = new_weight_clamp(0.0, 1.0);
        assert!(is_clamped(&c, 1.5));
        assert!(!is_clamped(&c, 0.5));
    }

    #[test]
    fn test_clamp_range() {
        let c = new_weight_clamp(-1.0, 2.0);
        assert_eq!(clamp_range(&c), (-1.0, 2.0));
    }

    #[test]
    fn test_clamp_to_json() {
        let c = new_weight_clamp(0.0, 1.0);
        let json = clamp_to_json(&c);
        assert!(json.contains("\"min\":0"));
    }

    #[test]
    fn test_reset_clamp() {
        let mut c = new_weight_clamp(-5.0, 5.0);
        reset_clamp(&mut c);
        assert!((clamp_min(&c) - 0.0).abs() < 1e-6);
        assert!((clamp_max(&c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_boundary_not_clamped() {
        let c = new_weight_clamp(0.0, 1.0);
        assert!(!is_clamped(&c, 0.0));
        assert!(!is_clamped(&c, 1.0));
    }

    #[test]
    fn test_negative_range() {
        let c = new_weight_clamp(-2.0, -1.0);
        assert!((clamp_weight(&c, 0.0) - (-1.0)).abs() < 1e-6);
    }
}
