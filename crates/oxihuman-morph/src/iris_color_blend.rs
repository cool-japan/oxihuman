// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Iris color blend target — RGB tone weights for eye color variation.

/// Iris color preset indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrisColorPreset {
    Brown,
    Blue,
    Green,
    Hazel,
    Gray,
    Amber,
}

/// Iris color blend parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IrisColorBlendParams {
    /// RGB weight for base tone (0..=1 each).
    pub base_rgb: [f32; 3],
    /// Limbal ring darkness 0..=1.
    pub limbal_ring: f32,
    /// Iris pattern radial frequency.
    pub pattern_freq: f32,
    /// Iris pattern amplitude 0..=1.
    pub pattern_amp: f32,
    /// Blend weight to heterochromia target (right eye shift).
    pub heterochromia: f32,
}

impl Default for IrisColorBlendParams {
    fn default() -> Self {
        Self {
            base_rgb: [0.45, 0.30, 0.18],
            limbal_ring: 0.6,
            pattern_freq: 12.0,
            pattern_amp: 0.15,
            heterochromia: 0.0,
        }
    }
}

/// Return base RGB for a preset.
#[allow(dead_code)]
pub fn preset_rgb(preset: IrisColorPreset) -> [f32; 3] {
    match preset {
        IrisColorPreset::Brown => [0.45, 0.30, 0.18],
        IrisColorPreset::Blue => [0.35, 0.55, 0.80],
        IrisColorPreset::Green => [0.30, 0.55, 0.30],
        IrisColorPreset::Hazel => [0.55, 0.45, 0.20],
        IrisColorPreset::Gray => [0.55, 0.56, 0.58],
        IrisColorPreset::Amber => [0.75, 0.55, 0.10],
    }
}

/// Apply preset to params.
#[allow(dead_code)]
pub fn apply_preset(params: &mut IrisColorBlendParams, preset: IrisColorPreset) {
    params.base_rgb = preset_rgb(preset);
}

/// Blend two iris color params.
#[allow(dead_code)]
pub fn blend_iris_color(
    a: &IrisColorBlendParams,
    b: &IrisColorBlendParams,
    t: f32,
) -> IrisColorBlendParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    IrisColorBlendParams {
        base_rgb: [
            a.base_rgb[0] * inv + b.base_rgb[0] * t,
            a.base_rgb[1] * inv + b.base_rgb[1] * t,
            a.base_rgb[2] * inv + b.base_rgb[2] * t,
        ],
        limbal_ring: a.limbal_ring * inv + b.limbal_ring * t,
        pattern_freq: a.pattern_freq * inv + b.pattern_freq * t,
        pattern_amp: a.pattern_amp * inv + b.pattern_amp * t,
        heterochromia: a.heterochromia * inv + b.heterochromia * t,
    }
}

/// Set limbal ring strength.
#[allow(dead_code)]
pub fn set_limbal_ring(params: &mut IrisColorBlendParams, value: f32) {
    params.limbal_ring = value.clamp(0.0, 1.0);
}

/// Set heterochromia blend weight.
#[allow(dead_code)]
pub fn set_heterochromia(params: &mut IrisColorBlendParams, value: f32) {
    params.heterochromia = value.clamp(0.0, 1.0);
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_iris_color(params: &mut IrisColorBlendParams) {
    *params = IrisColorBlendParams::default();
}

/// Compute luminance of iris base color.
#[allow(dead_code)]
pub fn iris_luminance(params: &IrisColorBlendParams) -> f32 {
    0.2126 * params.base_rgb[0] + 0.7152 * params.base_rgb[1] + 0.0722 * params.base_rgb[2]
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn iris_color_blend_to_json(params: &IrisColorBlendParams) -> String {
    format!(
        r#"{{"r":{:.4},"g":{:.4},"b":{:.4},"limbal_ring":{:.4},"heterochromia":{:.4}}}"#,
        params.base_rgb[0],
        params.base_rgb[1],
        params.base_rgb[2],
        params.limbal_ring,
        params.heterochromia
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = IrisColorBlendParams::default();
        assert!((0.0..=1.0).contains(&p.limbal_ring));
    }

    #[test]
    fn test_preset_brown() {
        let rgb = preset_rgb(IrisColorPreset::Brown);
        assert!(rgb[0] > rgb[2]);
    }

    #[test]
    fn test_preset_blue() {
        let rgb = preset_rgb(IrisColorPreset::Blue);
        assert!(rgb[2] > rgb[0]);
    }

    #[test]
    fn test_apply_preset() {
        let mut p = IrisColorBlendParams::default();
        apply_preset(&mut p, IrisColorPreset::Blue);
        let rgb = preset_rgb(IrisColorPreset::Blue);
        assert!((p.base_rgb[0] - rgb[0]).abs() < 1e-6);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = IrisColorBlendParams {
            base_rgb: [0.0, 0.0, 0.0],
            ..Default::default()
        };
        let b = IrisColorBlendParams {
            base_rgb: [1.0, 1.0, 1.0],
            ..Default::default()
        };
        let r = blend_iris_color(&a, &b, 0.5);
        assert!((r.base_rgb[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_limbal_ring_clamped() {
        let mut p = IrisColorBlendParams::default();
        set_limbal_ring(&mut p, 5.0);
        assert!((p.limbal_ring - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_heterochromia() {
        let mut p = IrisColorBlendParams::default();
        set_heterochromia(&mut p, 0.3);
        assert!((p.heterochromia - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_luminance_positive() {
        let p = IrisColorBlendParams::default();
        assert!(iris_luminance(&p) > 0.0);
    }

    #[test]
    fn test_to_json() {
        let j = iris_color_blend_to_json(&IrisColorBlendParams::default());
        assert!(j.contains("limbal_ring"));
        assert!(j.contains("heterochromia"));
    }

    #[test]
    fn test_reset() {
        let mut p = IrisColorBlendParams::default();
        apply_preset(&mut p, IrisColorPreset::Blue);
        reset_iris_color(&mut p);
        let def_r = IrisColorBlendParams::default().base_rgb[0];
        assert!((p.base_rgb[0] - def_r).abs() < 1e-6);
    }
}
