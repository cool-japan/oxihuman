// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Gluteal region shape morph control.

#![allow(dead_code)]

/// Gluteal morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlutealParams {
    /// Volume: 0.0 = flat, 1.0 = full.
    pub volume: f32,
    /// Projection (posterior protrusion): 0.0 = flat, 1.0 = prominent.
    pub projection: f32,
    /// Width: 0.0 = narrow, 1.0 = wide.
    pub width: f32,
    /// Lift (superior position): 0.0 = low, 1.0 = high.
    pub lift: f32,
}

#[allow(dead_code)]
impl Default for GlutealParams {
    fn default() -> Self {
        Self {
            volume: 0.5,
            projection: 0.5,
            width: 0.5,
            lift: 0.5,
        }
    }
}

/// Resulting morph weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlutealWeights {
    pub volume_w: f32,
    pub projection_w: f32,
    pub width_w: f32,
    pub lift_w: f32,
}

/// Create default gluteal params.
#[allow(dead_code)]
pub fn default_gluteal() -> GlutealParams {
    GlutealParams::default()
}

/// Evaluate morph weights.
#[allow(dead_code)]
pub fn evaluate_gluteal(p: &GlutealParams) -> GlutealWeights {
    GlutealWeights {
        volume_w: p.volume.clamp(0.0, 1.0),
        projection_w: p.projection.clamp(0.0, 1.0),
        width_w: p.width.clamp(0.0, 1.0),
        lift_w: p.lift.clamp(0.0, 1.0),
    }
}

/// Blend two param sets.
#[allow(dead_code)]
pub fn blend_gluteal(a: &GlutealParams, b: &GlutealParams, t: f32) -> GlutealParams {
    let t = t.clamp(0.0, 1.0);
    GlutealParams {
        volume: a.volume + (b.volume - a.volume) * t,
        projection: a.projection + (b.projection - a.projection) * t,
        width: a.width + (b.width - a.width) * t,
        lift: a.lift + (b.lift - a.lift) * t,
    }
}

/// Set gluteal volume.
#[allow(dead_code)]
pub fn set_gluteal_volume(p: &mut GlutealParams, value: f32) {
    p.volume = value.clamp(0.0, 1.0);
}

/// Set gluteal projection.
#[allow(dead_code)]
pub fn set_gluteal_projection(p: &mut GlutealParams, value: f32) {
    p.projection = value.clamp(0.0, 1.0);
}

/// Set gluteal lift.
#[allow(dead_code)]
pub fn set_gluteal_lift(p: &mut GlutealParams, value: f32) {
    p.lift = value.clamp(0.0, 1.0);
}

/// Check validity.
#[allow(dead_code)]
pub fn is_valid_gluteal(p: &GlutealParams) -> bool {
    (0.0..=1.0).contains(&p.volume)
        && (0.0..=1.0).contains(&p.projection)
        && (0.0..=1.0).contains(&p.width)
        && (0.0..=1.0).contains(&p.lift)
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_gluteal(p: &mut GlutealParams) {
    *p = GlutealParams::default();
}

/// Approximate cross-section area index.
#[allow(dead_code)]
pub fn gluteal_area_index(p: &GlutealParams) -> f32 {
    p.volume * p.projection * p.width
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn gluteal_to_json(p: &GlutealParams) -> String {
    format!(
        r#"{{"volume":{:.4},"projection":{:.4},"width":{:.4},"lift":{:.4}}}"#,
        p.volume, p.projection, p.width, p.lift
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = GlutealParams::default();
        assert!((p.volume - 0.5).abs() < 1e-6);
        assert!((p.lift - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_clamps() {
        let p = GlutealParams {
            volume: 2.0,
            projection: -1.0,
            width: 0.5,
            lift: 0.5,
        };
        let w = evaluate_gluteal(&p);
        assert!((w.volume_w - 1.0).abs() < 1e-6);
        assert!(w.projection_w < 1e-6);
    }

    #[test]
    fn test_blend() {
        let a = GlutealParams {
            volume: 0.0,
            projection: 0.0,
            width: 0.0,
            lift: 0.0,
        };
        let b = GlutealParams {
            volume: 1.0,
            projection: 1.0,
            width: 1.0,
            lift: 1.0,
        };
        let m = blend_gluteal(&a, &b, 0.5);
        assert!((m.volume - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_volume() {
        let mut p = GlutealParams::default();
        set_gluteal_volume(&mut p, 0.9);
        assert!((p.volume - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection_clamped() {
        let mut p = GlutealParams::default();
        set_gluteal_projection(&mut p, 5.0);
        assert!((p.projection - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_area_index() {
        let p = GlutealParams {
            volume: 1.0,
            projection: 1.0,
            width: 1.0,
            lift: 0.5,
        };
        assert!((gluteal_area_index(&p) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_valid_default() {
        assert!(is_valid_gluteal(&GlutealParams::default()));
    }

    #[test]
    fn test_reset() {
        let mut p = GlutealParams {
            volume: 0.9,
            projection: 0.8,
            width: 0.7,
            lift: 0.6,
        };
        reset_gluteal(&mut p);
        assert!((p.volume - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = gluteal_to_json(&GlutealParams::default());
        assert!(j.contains("projection"));
        assert!(j.contains("lift"));
    }
}
