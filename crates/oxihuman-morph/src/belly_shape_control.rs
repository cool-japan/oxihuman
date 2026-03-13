// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Abdominal convexity and bloat morph control.

#![allow(dead_code)]

/// Belly shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BellyShapeParams {
    /// Convexity: 0.0 = flat, 1.0 = maximum protrusion.
    pub convexity: f32,
    /// Bloat: 0.0 = no bloat, 1.0 = maximum distension.
    pub bloat: f32,
    /// Love handles: 0.0 = none, 1.0 = prominent.
    pub love_handles: f32,
    /// Upper vs lower distribution: -1.0 = upper, 1.0 = lower.
    pub upper_lower: f32,
}

#[allow(dead_code)]
impl Default for BellyShapeParams {
    fn default() -> Self {
        Self {
            convexity: 0.0,
            bloat: 0.0,
            love_handles: 0.0,
            upper_lower: 0.0,
        }
    }
}

/// Resulting morph weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BellyShapeWeights {
    pub convexity_w: f32,
    pub bloat_w: f32,
    pub love_handles_w: f32,
    pub distribution_w: f32,
}

/// Create default belly shape params.
#[allow(dead_code)]
pub fn default_belly_shape() -> BellyShapeParams {
    BellyShapeParams::default()
}

/// Evaluate morph weights.
#[allow(dead_code)]
pub fn evaluate_belly_shape(p: &BellyShapeParams) -> BellyShapeWeights {
    BellyShapeWeights {
        convexity_w: p.convexity.clamp(0.0, 1.0),
        bloat_w: p.bloat.clamp(0.0, 1.0),
        love_handles_w: p.love_handles.clamp(0.0, 1.0),
        distribution_w: p.upper_lower.clamp(-1.0, 1.0),
    }
}

/// Blend two param sets.
#[allow(dead_code)]
pub fn blend_belly_shape(a: &BellyShapeParams, b: &BellyShapeParams, t: f32) -> BellyShapeParams {
    let t = t.clamp(0.0, 1.0);
    BellyShapeParams {
        convexity: a.convexity + (b.convexity - a.convexity) * t,
        bloat: a.bloat + (b.bloat - a.bloat) * t,
        love_handles: a.love_handles + (b.love_handles - a.love_handles) * t,
        upper_lower: a.upper_lower + (b.upper_lower - a.upper_lower) * t,
    }
}

/// Set convexity.
#[allow(dead_code)]
pub fn set_belly_convexity(p: &mut BellyShapeParams, value: f32) {
    p.convexity = value.clamp(0.0, 1.0);
}

/// Set bloat level.
#[allow(dead_code)]
pub fn set_belly_bloat(p: &mut BellyShapeParams, value: f32) {
    p.bloat = value.clamp(0.0, 1.0);
}

/// Check validity.
#[allow(dead_code)]
pub fn is_valid_belly_shape(p: &BellyShapeParams) -> bool {
    (0.0..=1.0).contains(&p.convexity)
        && (0.0..=1.0).contains(&p.bloat)
        && (0.0..=1.0).contains(&p.love_handles)
        && (-1.0..=1.0).contains(&p.upper_lower)
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_belly_shape(p: &mut BellyShapeParams) {
    *p = BellyShapeParams::default();
}

/// Approximate volume change relative to flat belly.
#[allow(dead_code)]
pub fn belly_volume_factor(p: &BellyShapeParams) -> f32 {
    1.0 + p.convexity * 0.3 + p.bloat * 0.2
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn belly_shape_to_json(p: &BellyShapeParams) -> String {
    format!(
        r#"{{"convexity":{:.4},"bloat":{:.4},"love_handles":{:.4},"upper_lower":{:.4}}}"#,
        p.convexity, p.bloat, p.love_handles, p.upper_lower
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_zero() {
        let p = BellyShapeParams::default();
        assert!(p.convexity.abs() < 1e-6);
        assert!(p.bloat.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_clamps() {
        let p = BellyShapeParams {
            convexity: 2.0,
            bloat: -1.0,
            love_handles: 0.5,
            upper_lower: 0.0,
        };
        let w = evaluate_belly_shape(&p);
        assert!((w.convexity_w - 1.0).abs() < 1e-6);
        assert!(w.bloat_w < 1e-6);
    }

    #[test]
    fn test_blend() {
        let a = BellyShapeParams {
            convexity: 0.0,
            bloat: 0.0,
            love_handles: 0.0,
            upper_lower: 0.0,
        };
        let b = BellyShapeParams {
            convexity: 1.0,
            bloat: 1.0,
            love_handles: 1.0,
            upper_lower: 1.0,
        };
        let m = blend_belly_shape(&a, &b, 0.5);
        assert!((m.convexity - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_convexity() {
        let mut p = BellyShapeParams::default();
        set_belly_convexity(&mut p, 0.7);
        assert!((p.convexity - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_bloat_clamped() {
        let mut p = BellyShapeParams::default();
        set_belly_bloat(&mut p, -5.0);
        assert!(p.bloat < 1e-6);
    }

    #[test]
    fn test_volume_factor_default() {
        let p = BellyShapeParams::default();
        assert!((belly_volume_factor(&p) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_valid_default() {
        assert!(is_valid_belly_shape(&BellyShapeParams::default()));
    }

    #[test]
    fn test_reset() {
        let mut p = BellyShapeParams {
            convexity: 0.8,
            bloat: 0.5,
            love_handles: 0.3,
            upper_lower: -0.5,
        };
        reset_belly_shape(&mut p);
        assert!(p.convexity.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = belly_shape_to_json(&BellyShapeParams::default());
        assert!(j.contains("convexity"));
        assert!(j.contains("bloat"));
    }
}
