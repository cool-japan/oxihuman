// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Nose columella shape and angle morph control.

#![allow(dead_code)]

use std::f32::consts::FRAC_PI_4;

/// Columella morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColumellaParams {
    /// Angle in radians: negative = retracted, positive = projected.
    pub angle_rad: f32,
    /// Width: 0.0 = narrow, 1.0 = wide.
    pub width: f32,
    /// Length: 0.0 = short, 1.0 = long.
    pub length: f32,
}

#[allow(dead_code)]
impl Default for ColumellaParams {
    fn default() -> Self {
        Self {
            angle_rad: 0.0,
            width: 0.5,
            length: 0.5,
        }
    }
}

/// Morph weight output.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColumellaWeights {
    pub angle_w: f32,
    pub width_w: f32,
    pub length_w: f32,
}

/// Create default columella params.
#[allow(dead_code)]
pub fn default_columella() -> ColumellaParams {
    ColumellaParams::default()
}

/// Evaluate morph weights from params.
#[allow(dead_code)]
pub fn evaluate_columella(p: &ColumellaParams) -> ColumellaWeights {
    let max_angle = FRAC_PI_4;
    ColumellaWeights {
        angle_w: (p.angle_rad / max_angle).clamp(-1.0, 1.0),
        width_w: p.width.clamp(0.0, 1.0),
        length_w: p.length.clamp(0.0, 1.0),
    }
}

/// Blend two param sets.
#[allow(dead_code)]
pub fn blend_columella(a: &ColumellaParams, b: &ColumellaParams, t: f32) -> ColumellaParams {
    let t = t.clamp(0.0, 1.0);
    ColumellaParams {
        angle_rad: a.angle_rad + (b.angle_rad - a.angle_rad) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
    }
}

/// Set columella angle.
#[allow(dead_code)]
pub fn set_columella_angle(p: &mut ColumellaParams, rad: f32) {
    p.angle_rad = rad.clamp(-FRAC_PI_4, FRAC_PI_4);
}

/// Set columella width.
#[allow(dead_code)]
pub fn set_columella_width(p: &mut ColumellaParams, value: f32) {
    p.width = value.clamp(0.0, 1.0);
}

/// Set columella length.
#[allow(dead_code)]
pub fn set_columella_length(p: &mut ColumellaParams, value: f32) {
    p.length = value.clamp(0.0, 1.0);
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_columella(p: &mut ColumellaParams) {
    *p = ColumellaParams::default();
}

/// Check validity.
#[allow(dead_code)]
pub fn is_valid_columella(p: &ColumellaParams) -> bool {
    p.angle_rad.abs() <= FRAC_PI_4
        && (0.0..=1.0).contains(&p.width)
        && (0.0..=1.0).contains(&p.length)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn columella_to_json(p: &ColumellaParams) -> String {
    format!(
        r#"{{"angle_rad":{:.6},"width":{:.4},"length":{:.4}}}"#,
        p.angle_rad, p.width, p.length
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = ColumellaParams::default();
        assert!(p.angle_rad.abs() < 1e-6);
        assert!((p.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_angle() {
        let p = ColumellaParams {
            angle_rad: FRAC_PI_4,
            width: 0.5,
            length: 0.5,
        };
        let w = evaluate_columella(&p);
        assert!((w.angle_w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_clamps() {
        let p = ColumellaParams {
            angle_rad: 10.0,
            width: 2.0,
            length: -1.0,
        };
        let w = evaluate_columella(&p);
        assert!((w.angle_w - 1.0).abs() < 1e-5);
        assert!((w.width_w - 1.0).abs() < 1e-5);
        assert!(w.length_w < 1e-5);
    }

    #[test]
    fn test_blend() {
        let a = ColumellaParams {
            angle_rad: 0.0,
            width: 0.0,
            length: 0.0,
        };
        let b = ColumellaParams {
            angle_rad: 0.4,
            width: 1.0,
            length: 1.0,
        };
        let m = blend_columella(&a, &b, 0.5);
        assert!((m.width - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_angle_clamped() {
        let mut p = ColumellaParams::default();
        set_columella_angle(&mut p, 10.0);
        assert!((p.angle_rad - FRAC_PI_4).abs() < 1e-5);
    }

    #[test]
    fn test_set_width() {
        let mut p = ColumellaParams::default();
        set_columella_width(&mut p, 0.8);
        assert!((p.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_is_valid_default() {
        assert!(is_valid_columella(&ColumellaParams::default()));
    }

    #[test]
    fn test_reset() {
        let mut p = ColumellaParams {
            angle_rad: 0.3,
            width: 0.9,
            length: 0.1,
        };
        reset_columella(&mut p);
        assert!(p.angle_rad.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = columella_to_json(&ColumellaParams::default());
        assert!(j.contains("angle_rad"));
        assert!(j.contains("width"));
    }
}
