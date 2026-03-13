// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Full-pipeline colour grading: lift/gamma/gain, saturation, and LUT index.

use std::f32::consts::E;

/// Colour grade parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ColorGradeParams {
    /// Lift (shadows offset), −0.5..=0.5.
    pub lift: f32,
    /// Gamma exponent (reciprocal applied), 0.1..=5.0.
    pub gamma: f32,
    /// Gain (highlights scale), 0.0..=4.0.
    pub gain: f32,
    /// Saturation scale, 0.0..=3.0.
    pub saturation: f32,
    /// Global exposure in EV stops.
    pub exposure: f32,
}

impl Default for ColorGradeParams {
    fn default() -> Self {
        Self {
            lift: 0.0,
            gamma: 1.0,
            gain: 1.0,
            saturation: 1.0,
            exposure: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_color_grade() -> ColorGradeParams {
    ColorGradeParams::default()
}

#[allow(dead_code)]
pub fn cg_set_lift(p: &mut ColorGradeParams, v: f32) {
    p.lift = v.clamp(-0.5, 0.5);
}

#[allow(dead_code)]
pub fn cg_set_gamma(p: &mut ColorGradeParams, v: f32) {
    p.gamma = v.clamp(0.1, 5.0);
}

#[allow(dead_code)]
pub fn cg_set_gain(p: &mut ColorGradeParams, v: f32) {
    p.gain = v.clamp(0.0, 4.0);
}

#[allow(dead_code)]
pub fn cg_set_saturation(p: &mut ColorGradeParams, v: f32) {
    p.saturation = v.clamp(0.0, 3.0);
}

#[allow(dead_code)]
pub fn cg_set_exposure(p: &mut ColorGradeParams, ev: f32) {
    p.exposure = ev.clamp(-10.0, 10.0);
}

#[allow(dead_code)]
pub fn cg_reset(p: &mut ColorGradeParams) {
    *p = ColorGradeParams::default();
}

/// Apply lift/gamma/gain to a single channel.
#[allow(dead_code)]
pub fn cg_apply_channel(v: f32, p: &ColorGradeParams) -> f32 {
    let exposure_scale = E.powf(p.exposure * std::f32::consts::LN_2);
    let lifted = v + p.lift;
    let gained = (lifted * p.gain * exposure_scale).clamp(0.0, f32::MAX);
    gained.powf(1.0 / p.gamma.max(1e-6)).min(1.0)
}

/// Apply grading to an RGB pixel (linear light).
#[allow(dead_code)]
pub fn cg_apply_rgb(rgb: [f32; 3], p: &ColorGradeParams) -> [f32; 3] {
    let r = cg_apply_channel(rgb[0], p);
    let g = cg_apply_channel(rgb[1], p);
    let b = cg_apply_channel(rgb[2], p);
    // Saturation via luminance desaturation
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    [
        lum + (r - lum) * p.saturation,
        lum + (g - lum) * p.saturation,
        lum + (b - lum) * p.saturation,
    ]
}

/// Check whether the grade is effectively identity.
#[allow(dead_code)]
pub fn cg_is_identity(p: &ColorGradeParams) -> bool {
    p.lift.abs() < 1e-4
        && (p.gamma - 1.0).abs() < 1e-4
        && (p.gain - 1.0).abs() < 1e-4
        && (p.saturation - 1.0).abs() < 1e-4
        && p.exposure.abs() < 1e-4
}

#[allow(dead_code)]
pub fn cg_blend(a: &ColorGradeParams, b: &ColorGradeParams, t: f32) -> ColorGradeParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ColorGradeParams {
        lift: a.lift * inv + b.lift * t,
        gamma: a.gamma * inv + b.gamma * t,
        gain: a.gain * inv + b.gain * t,
        saturation: a.saturation * inv + b.saturation * t,
        exposure: a.exposure * inv + b.exposure * t,
    }
}

#[allow(dead_code)]
pub fn cg_to_json(p: &ColorGradeParams) -> String {
    format!(
        "{{\"lift\":{:.4},\"gamma\":{:.4},\"gain\":{:.4},\"saturation\":{:.4},\"exposure\":{:.4}}}",
        p.lift, p.gamma, p.gain, p.saturation, p.exposure
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_identity() {
        assert!(cg_is_identity(&new_color_grade()));
    }

    #[test]
    fn identity_channel_preserves_value() {
        let p = new_color_grade();
        assert!((cg_apply_channel(0.5, &p) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn gain_two_doubles_channel() {
        let mut p = new_color_grade();
        cg_set_gain(&mut p, 2.0);
        assert!((cg_apply_channel(0.25, &p) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn lift_clamps() {
        let mut p = new_color_grade();
        cg_set_lift(&mut p, 5.0);
        assert!((p.lift - 0.5).abs() < 1e-6);
    }

    #[test]
    fn gamma_clamps_low() {
        let mut p = new_color_grade();
        cg_set_gamma(&mut p, 0.0);
        assert!(p.gamma >= 0.1);
    }

    #[test]
    fn saturation_zero_gives_gray() {
        let mut p = new_color_grade();
        cg_set_saturation(&mut p, 0.0);
        let out = cg_apply_rgb([1.0, 0.0, 0.0], &p);
        // r, g, b should all equal luminance
        assert!((out[0] - out[1]).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_identity() {
        let mut p = new_color_grade();
        cg_set_lift(&mut p, 0.3);
        cg_reset(&mut p);
        assert!(cg_is_identity(&p));
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_color_grade();
        cg_set_gain(&mut b, 2.0);
        let r = cg_blend(&new_color_grade(), &b, 0.5);
        assert!((r.gain - 1.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_lift() {
        let j = cg_to_json(&new_color_grade());
        assert!(j.contains("lift") && j.contains("exposure"));
    }

    #[test]
    fn exposure_positive_brightens() {
        let mut p = new_color_grade();
        cg_set_exposure(&mut p, 1.0);
        let bright = cg_apply_channel(0.3, &p);
        assert!(bright > 0.3);
    }
}
