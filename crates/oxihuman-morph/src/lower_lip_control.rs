// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Lower lip morphology control.

/// Lower lip parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LowerLipParams {
    pub thickness: f32,
    pub protrusion: f32,
    pub droop: f32,
    pub width: f32,
}

/// Lower lip result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LowerLipResult {
    pub thickness_weight: f32,
    pub protrusion_weight: f32,
    pub droop_weight: f32,
    pub overall_weight: f32,
}

/// Default lower lip parameters.
#[allow(dead_code)]
pub fn default_lower_lip() -> LowerLipParams {
    LowerLipParams {
        thickness: 0.5,
        protrusion: 0.3,
        droop: 0.0,
        width: 0.5,
    }
}

/// Evaluate lower lip morph.
#[allow(dead_code)]
pub fn evaluate_lower_lip(params: &LowerLipParams) -> LowerLipResult {
    let th = params.thickness.clamp(0.0, 1.0);
    let pr = params.protrusion.clamp(0.0, 1.0);
    let dr = params.droop.clamp(0.0, 1.0);
    LowerLipResult {
        thickness_weight: th,
        protrusion_weight: pr,
        droop_weight: dr,
        overall_weight: th * 0.4 + pr * 0.3 + dr * 0.3,
    }
}

/// Blend lower lip params.
#[allow(dead_code)]
pub fn blend_lower_lip(a: &LowerLipParams, b: &LowerLipParams, t: f32) -> LowerLipParams {
    let t = t.clamp(0.0, 1.0);
    LowerLipParams {
        thickness: a.thickness + (b.thickness - a.thickness) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        droop: a.droop + (b.droop - a.droop) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

/// Set lower lip thickness.
#[allow(dead_code)]
pub fn set_lower_lip_thickness(params: &mut LowerLipParams, value: f32) {
    params.thickness = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_lower_lip(params: &LowerLipParams) -> bool {
    (0.0..=1.0).contains(&params.thickness)
        && (0.0..=1.0).contains(&params.protrusion)
        && (0.0..=1.0).contains(&params.droop)
        && (0.0..=1.0).contains(&params.width)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_lower_lip(params: &mut LowerLipParams) {
    *params = default_lower_lip();
}

/// Lip fullness factor.
#[allow(dead_code)]
pub fn lip_fullness(params: &LowerLipParams) -> f32 {
    (params.thickness * 0.6 + params.protrusion * 0.4).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_lower_lip();
        assert!((p.thickness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_lower_lip();
        let r = evaluate_lower_lip(&p);
        assert!((0.0..=1.0).contains(&r.overall_weight));
    }

    #[test]
    fn test_blend() {
        let a = default_lower_lip();
        let mut b = default_lower_lip();
        b.thickness = 1.0;
        let c = blend_lower_lip(&a, &b, 0.5);
        assert!((c.thickness - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_thickness() {
        let mut p = default_lower_lip();
        set_lower_lip_thickness(&mut p, 0.9);
        assert!((p.thickness - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_lower_lip(&default_lower_lip()));
    }

    #[test]
    fn test_invalid() {
        let p = LowerLipParams { thickness: 2.0, protrusion: 0.5, droop: 0.0, width: 0.5 };
        assert!(!is_valid_lower_lip(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = LowerLipParams { thickness: 0.9, protrusion: 0.1, droop: 0.2, width: 0.3 };
        reset_lower_lip(&mut p);
        assert!((p.thickness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_fullness() {
        let p = LowerLipParams { thickness: 1.0, protrusion: 1.0, droop: 0.0, width: 0.5 };
        assert!((lip_fullness(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_fullness() {
        let p = LowerLipParams { thickness: 0.0, protrusion: 0.0, droop: 0.0, width: 0.5 };
        assert!(lip_fullness(&p).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = default_lower_lip();
        let c = blend_lower_lip(&a, &a, 0.5);
        assert!((c.thickness - a.thickness).abs() < 1e-6);
    }
}
