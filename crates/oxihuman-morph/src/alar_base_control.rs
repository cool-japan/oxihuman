// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Nostril width (alar base) morph control.

#![allow(dead_code)]

/// Alar base parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlarBaseParams {
    /// Width: 0.0 = narrow, 1.0 = wide.
    pub width: f32,
    /// Flare: 0.0 = no flare, 1.0 = maximum flare.
    pub flare: f32,
    /// Height offset: -1.0 = low insertion, 1.0 = high insertion.
    pub height: f32,
}

#[allow(dead_code)]
impl Default for AlarBaseParams {
    fn default() -> Self {
        Self {
            width: 0.5,
            flare: 0.0,
            height: 0.0,
        }
    }
}

/// Morph weight output.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlarBaseWeights {
    pub width_w: f32,
    pub flare_w: f32,
    pub height_w: f32,
}

/// Create default alar base params.
#[allow(dead_code)]
pub fn default_alar_base() -> AlarBaseParams {
    AlarBaseParams::default()
}

/// Evaluate morph weights.
#[allow(dead_code)]
pub fn evaluate_alar_base(p: &AlarBaseParams) -> AlarBaseWeights {
    AlarBaseWeights {
        width_w: p.width.clamp(0.0, 1.0),
        flare_w: p.flare.clamp(0.0, 1.0),
        height_w: p.height.clamp(-1.0, 1.0),
    }
}

/// Blend two param sets.
#[allow(dead_code)]
pub fn blend_alar_base(a: &AlarBaseParams, b: &AlarBaseParams, t: f32) -> AlarBaseParams {
    let t = t.clamp(0.0, 1.0);
    AlarBaseParams {
        width: a.width + (b.width - a.width) * t,
        flare: a.flare + (b.flare - a.flare) * t,
        height: a.height + (b.height - a.height) * t,
    }
}

/// Set alar width.
#[allow(dead_code)]
pub fn set_alar_width(p: &mut AlarBaseParams, value: f32) {
    p.width = value.clamp(0.0, 1.0);
}

/// Set alar flare.
#[allow(dead_code)]
pub fn set_alar_flare(p: &mut AlarBaseParams, value: f32) {
    p.flare = value.clamp(0.0, 1.0);
}

/// Check validity.
#[allow(dead_code)]
pub fn is_valid_alar_base(p: &AlarBaseParams) -> bool {
    (0.0..=1.0).contains(&p.width)
        && (0.0..=1.0).contains(&p.flare)
        && (-1.0..=1.0).contains(&p.height)
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_alar_base(p: &mut AlarBaseParams) {
    *p = AlarBaseParams::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn alar_base_to_json(p: &AlarBaseParams) -> String {
    format!(
        r#"{{"width":{:.4},"flare":{:.4},"height":{:.4}}}"#,
        p.width, p.flare, p.height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = AlarBaseParams::default();
        assert!((p.width - 0.5).abs() < 1e-6);
        assert!(p.flare.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_clamps() {
        let p = AlarBaseParams {
            width: 2.0,
            flare: -1.0,
            height: 0.0,
        };
        let w = evaluate_alar_base(&p);
        assert!((w.width_w - 1.0).abs() < 1e-6);
        assert!(w.flare_w < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = AlarBaseParams::default();
        let b = AlarBaseParams::default();
        let m = blend_alar_base(&a, &b, 0.5);
        assert!((m.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = AlarBaseParams {
            width: 0.0,
            flare: 0.0,
            height: 0.0,
        };
        let b = AlarBaseParams {
            width: 1.0,
            flare: 1.0,
            height: 1.0,
        };
        let m = blend_alar_base(&a, &b, 0.5);
        assert!((m.width - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_width_clamped() {
        let mut p = AlarBaseParams::default();
        set_alar_width(&mut p, -5.0);
        assert!(p.width < 1e-6);
    }

    #[test]
    fn test_set_flare_clamped() {
        let mut p = AlarBaseParams::default();
        set_alar_flare(&mut p, 5.0);
        assert!((p.flare - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_valid_default() {
        assert!(is_valid_alar_base(&AlarBaseParams::default()));
    }

    #[test]
    fn test_reset() {
        let mut p = AlarBaseParams {
            width: 0.9,
            flare: 0.8,
            height: 0.5,
        };
        reset_alar_base(&mut p);
        assert!((p.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let j = alar_base_to_json(&AlarBaseParams::default());
        assert!(j.contains("width"));
        assert!(j.contains("flare"));
    }
}
