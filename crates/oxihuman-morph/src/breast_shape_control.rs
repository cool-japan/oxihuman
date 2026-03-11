// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Breast shape, ptosis and volume morphs.

#![allow(dead_code)]

/// Breast shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BreastShapeParams {
    /// Volume: 0.0 = minimal, 1.0 = maximum.
    pub volume: f32,
    /// Ptosis (droop): 0.0 = none, 1.0 = maximum droop.
    pub ptosis: f32,
    /// Projection (forward protrusion): 0.0 = flat, 1.0 = high.
    pub projection: f32,
    /// Lateral spread: 0.0 = medial, 1.0 = lateral.
    pub spread: f32,
    /// Upper pole fullness: 0.0 = deflated, 1.0 = full.
    pub upper_pole: f32,
}

#[allow(dead_code)]
impl Default for BreastShapeParams {
    fn default() -> Self {
        Self {
            volume: 0.5,
            ptosis: 0.0,
            projection: 0.5,
            spread: 0.5,
            upper_pole: 0.5,
        }
    }
}

/// Resulting morph target weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BreastShapeWeights {
    pub volume_w: f32,
    pub ptosis_w: f32,
    pub projection_w: f32,
    pub spread_w: f32,
    pub upper_pole_w: f32,
}

/// Create default breast shape params.
#[allow(dead_code)]
pub fn default_breast_shape() -> BreastShapeParams {
    BreastShapeParams::default()
}

/// Evaluate morph weights.
#[allow(dead_code)]
pub fn evaluate_breast_shape(p: &BreastShapeParams) -> BreastShapeWeights {
    BreastShapeWeights {
        volume_w: p.volume.clamp(0.0, 1.0),
        ptosis_w: p.ptosis.clamp(0.0, 1.0),
        projection_w: p.projection.clamp(0.0, 1.0),
        spread_w: p.spread.clamp(0.0, 1.0),
        upper_pole_w: p.upper_pole.clamp(0.0, 1.0),
    }
}

/// Blend two param sets.
#[allow(dead_code)]
pub fn blend_breast_shape(
    a: &BreastShapeParams,
    b: &BreastShapeParams,
    t: f32,
) -> BreastShapeParams {
    let t = t.clamp(0.0, 1.0);
    BreastShapeParams {
        volume: a.volume + (b.volume - a.volume) * t,
        ptosis: a.ptosis + (b.ptosis - a.ptosis) * t,
        projection: a.projection + (b.projection - a.projection) * t,
        spread: a.spread + (b.spread - a.spread) * t,
        upper_pole: a.upper_pole + (b.upper_pole - a.upper_pole) * t,
    }
}

/// Set volume.
#[allow(dead_code)]
pub fn set_breast_volume(p: &mut BreastShapeParams, value: f32) {
    p.volume = value.clamp(0.0, 1.0);
}

/// Set ptosis level.
#[allow(dead_code)]
pub fn set_breast_ptosis(p: &mut BreastShapeParams, value: f32) {
    p.ptosis = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_breast_shape(p: &BreastShapeParams) -> bool {
    (0.0..=1.0).contains(&p.volume)
        && (0.0..=1.0).contains(&p.ptosis)
        && (0.0..=1.0).contains(&p.projection)
        && (0.0..=1.0).contains(&p.spread)
        && (0.0..=1.0).contains(&p.upper_pole)
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_breast_shape(p: &mut BreastShapeParams) {
    *p = BreastShapeParams::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn breast_shape_to_json(p: &BreastShapeParams) -> String {
    format!(
        r#"{{"volume":{:.4},"ptosis":{:.4},"projection":{:.4},"spread":{:.4},"upper_pole":{:.4}}}"#,
        p.volume, p.ptosis, p.projection, p.spread, p.upper_pole
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = BreastShapeParams::default();
        assert!(p.ptosis.abs() < 1e-6);
        assert!((p.volume - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_clamps() {
        let p = BreastShapeParams {
            volume: 2.0,
            ptosis: -1.0,
            projection: 0.5,
            spread: 0.5,
            upper_pole: 0.5,
        };
        let w = evaluate_breast_shape(&p);
        assert!((w.volume_w - 1.0).abs() < 1e-6);
        assert!(w.ptosis_w < 1e-6);
    }

    #[test]
    fn test_blend() {
        let a = BreastShapeParams {
            volume: 0.0,
            ptosis: 0.0,
            projection: 0.0,
            spread: 0.0,
            upper_pole: 0.0,
        };
        let b = BreastShapeParams {
            volume: 1.0,
            ptosis: 1.0,
            projection: 1.0,
            spread: 1.0,
            upper_pole: 1.0,
        };
        let m = blend_breast_shape(&a, &b, 0.5);
        assert!((m.volume - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_volume() {
        let mut p = BreastShapeParams::default();
        set_breast_volume(&mut p, 0.8);
        assert!((p.volume - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_ptosis_clamped() {
        let mut p = BreastShapeParams::default();
        set_breast_ptosis(&mut p, 5.0);
        assert!((p.ptosis - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_valid_default() {
        assert!(is_valid_breast_shape(&BreastShapeParams::default()));
    }

    #[test]
    fn test_is_invalid() {
        let p = BreastShapeParams {
            volume: 1.5,
            ptosis: 0.0,
            projection: 0.5,
            spread: 0.5,
            upper_pole: 0.5,
        };
        assert!(!is_valid_breast_shape(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = BreastShapeParams {
            volume: 0.9,
            ptosis: 0.9,
            projection: 0.9,
            spread: 0.9,
            upper_pole: 0.9,
        };
        reset_breast_shape(&mut p);
        assert!(p.ptosis.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = breast_shape_to_json(&BreastShapeParams::default());
        assert!(j.contains("ptosis"));
        assert!(j.contains("volume"));
    }
}
