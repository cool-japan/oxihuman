// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Palm width and arch morph control.

/// Palm morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PalmParams {
    pub width: f32,
    pub arch: f32,
    pub thickness: f32,
}

/// Palm morph weights for shader binding.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PalmWeights {
    pub narrow: f32,
    pub wide: f32,
    pub arched: f32,
    pub flat: f32,
}

#[allow(dead_code)]
pub fn default_palm_params() -> PalmParams {
    PalmParams::default()
}

#[allow(dead_code)]
pub fn palm_set_width(p: &mut PalmParams, v: f32) {
    p.width = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn palm_set_arch(p: &mut PalmParams, v: f32) {
    p.arch = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn palm_set_thickness(p: &mut PalmParams, v: f32) {
    p.thickness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn palm_reset(p: &mut PalmParams) {
    *p = PalmParams::default();
}

#[allow(dead_code)]
pub fn palm_is_neutral(p: &PalmParams) -> bool {
    p.width.abs() < 1e-6 && p.arch.abs() < 1e-6 && p.thickness.abs() < 1e-6
}

#[allow(dead_code)]
pub fn palm_to_weights(p: &PalmParams) -> PalmWeights {
    PalmWeights {
        narrow: (-p.width).max(0.0),
        wide: p.width.max(0.0),
        arched: p.arch.max(0.0),
        flat: (-p.arch).max(0.0),
    }
}

#[allow(dead_code)]
pub fn palm_blend(a: &PalmParams, b: &PalmParams, t: f32) -> PalmParams {
    let t = t.clamp(0.0, 1.0);
    PalmParams {
        width: a.width + (b.width - a.width) * t,
        arch: a.arch + (b.arch - a.arch) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
    }
}

#[allow(dead_code)]
pub fn palm_surface_area_estimate(p: &PalmParams) -> f32 {
    let base = 1.0_f32;
    let w_factor = 1.0 + p.width * 0.2;
    base * w_factor
}

#[allow(dead_code)]
pub fn palm_to_json(p: &PalmParams) -> String {
    format!(
        r#"{{"width":{:.4},"arch":{:.4},"thickness":{:.4}}}"#,
        p.width, p.arch, p.thickness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(palm_is_neutral(&default_palm_params()));
    }

    #[test]
    fn set_width_clamps() {
        let mut p = default_palm_params();
        palm_set_width(&mut p, 3.0);
        assert!((p.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_arch_negative() {
        let mut p = default_palm_params();
        palm_set_arch(&mut p, -0.5);
        assert!((p.arch - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn reset_clears_all() {
        let mut p = default_palm_params();
        palm_set_width(&mut p, 0.8);
        palm_reset(&mut p);
        assert!(palm_is_neutral(&p));
    }

    #[test]
    fn weights_wide_only_when_positive() {
        let mut p = default_palm_params();
        palm_set_width(&mut p, 0.6);
        let w = palm_to_weights(&p);
        assert!(w.wide > 0.0 && w.narrow < 1e-6);
    }

    #[test]
    fn weights_narrow_only_when_negative() {
        let mut p = default_palm_params();
        palm_set_width(&mut p, -0.6);
        let w = palm_to_weights(&p);
        assert!(w.narrow > 0.0 && w.wide < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_palm_params();
        let mut b = default_palm_params();
        palm_set_width(&mut b, 1.0);
        let m = palm_blend(&a, &b, 0.5);
        assert!((m.width - 0.5).abs() < 1e-5);
    }

    #[test]
    fn surface_area_increases_with_width() {
        let p0 = default_palm_params();
        let mut p1 = default_palm_params();
        palm_set_width(&mut p1, 1.0);
        assert!(palm_surface_area_estimate(&p1) > palm_surface_area_estimate(&p0));
    }

    #[test]
    fn to_json_has_arch() {
        let p = default_palm_params();
        assert!(palm_to_json(&p).contains("arch"));
    }
}
