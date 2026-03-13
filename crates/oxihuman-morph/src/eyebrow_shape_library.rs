// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Library of eyebrow shape presets (flat, arched, peaked, etc.).

/// Eyebrow shape preset names.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowShapePreset {
    Flat,
    Arched,
    Peaked,
    Rounded,
    Straight,
    Angled,
    SoftArch,
    Bushy,
}

/// Eyebrow shape control parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BrowShapeParams {
    /// Overall arch height 0..=1.
    pub arch_height: f32,
    /// Peak position along brow 0..=1 (0 = inner, 1 = outer).
    pub peak_position: f32,
    /// Brow thickness 0..=1.
    pub thickness: f32,
    /// Tail angle (outer end lift/drop) -1..=1.
    pub tail_angle: f32,
    /// Inner corner angle -1..=1.
    pub inner_angle: f32,
    /// Width scale 0..=2.
    pub width: f32,
}

impl Default for BrowShapeParams {
    fn default() -> Self {
        Self {
            arch_height: 0.4,
            peak_position: 0.6,
            thickness: 0.5,
            tail_angle: 0.0,
            inner_angle: 0.0,
            width: 1.0,
        }
    }
}

/// Return preset params for a given preset.
#[allow(dead_code)]
pub fn preset_params(preset: BrowShapePreset) -> BrowShapeParams {
    match preset {
        BrowShapePreset::Flat => BrowShapeParams {
            arch_height: 0.1,
            peak_position: 0.5,
            thickness: 0.5,
            tail_angle: 0.0,
            inner_angle: 0.0,
            width: 1.0,
        },
        BrowShapePreset::Arched => BrowShapeParams {
            arch_height: 0.7,
            peak_position: 0.6,
            thickness: 0.4,
            tail_angle: -0.2,
            inner_angle: 0.1,
            width: 1.0,
        },
        BrowShapePreset::Peaked => BrowShapeParams {
            arch_height: 0.9,
            peak_position: 0.65,
            thickness: 0.35,
            tail_angle: -0.3,
            inner_angle: 0.0,
            width: 1.0,
        },
        BrowShapePreset::Rounded => BrowShapeParams {
            arch_height: 0.5,
            peak_position: 0.5,
            thickness: 0.6,
            tail_angle: -0.1,
            inner_angle: -0.1,
            width: 1.05,
        },
        BrowShapePreset::Straight => BrowShapeParams {
            arch_height: 0.05,
            peak_position: 0.5,
            thickness: 0.55,
            tail_angle: 0.0,
            inner_angle: 0.0,
            width: 1.0,
        },
        BrowShapePreset::Angled => BrowShapeParams {
            arch_height: 0.6,
            peak_position: 0.7,
            thickness: 0.4,
            tail_angle: -0.4,
            inner_angle: 0.2,
            width: 1.0,
        },
        BrowShapePreset::SoftArch => BrowShapeParams {
            arch_height: 0.45,
            peak_position: 0.58,
            thickness: 0.5,
            tail_angle: -0.1,
            inner_angle: 0.0,
            width: 1.0,
        },
        BrowShapePreset::Bushy => BrowShapeParams {
            arch_height: 0.35,
            peak_position: 0.5,
            thickness: 0.85,
            tail_angle: 0.0,
            inner_angle: 0.0,
            width: 1.1,
        },
    }
}

/// List all preset names.
#[allow(dead_code)]
pub fn all_presets() -> [BrowShapePreset; 8] {
    [
        BrowShapePreset::Flat,
        BrowShapePreset::Arched,
        BrowShapePreset::Peaked,
        BrowShapePreset::Rounded,
        BrowShapePreset::Straight,
        BrowShapePreset::Angled,
        BrowShapePreset::SoftArch,
        BrowShapePreset::Bushy,
    ]
}

/// Blend two brow shape params.
#[allow(dead_code)]
pub fn blend_brow_shape(a: &BrowShapeParams, b: &BrowShapeParams, t: f32) -> BrowShapeParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    BrowShapeParams {
        arch_height: a.arch_height * inv + b.arch_height * t,
        peak_position: a.peak_position * inv + b.peak_position * t,
        thickness: a.thickness * inv + b.thickness * t,
        tail_angle: a.tail_angle * inv + b.tail_angle * t,
        inner_angle: a.inner_angle * inv + b.inner_angle * t,
        width: a.width * inv + b.width * t,
    }
}

/// Evaluate brow height at a normalized x position.
#[allow(dead_code)]
pub fn brow_height_at(x: f32, params: &BrowShapeParams) -> f32 {
    let x = x.clamp(0.0, 1.0);
    let peak = params.peak_position.clamp(0.0, 1.0);
    let arch = params.arch_height.clamp(0.0, 1.0);
    let sigma = 0.25f32;
    let gaussian = (-(x - peak).powi(2) / (2.0 * sigma * sigma)).exp();
    arch * gaussian + params.tail_angle * (x - 0.5) + params.inner_angle * (0.5 - x).max(0.0)
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_brow_shape(params: &mut BrowShapeParams) {
    *params = BrowShapeParams::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn brow_shape_to_json(params: &BrowShapeParams) -> String {
    format!(
        r#"{{"arch_height":{:.4},"peak_position":{:.4},"thickness":{:.4},"tail_angle":{:.4},"inner_angle":{:.4},"width":{:.4}}}"#,
        params.arch_height,
        params.peak_position,
        params.thickness,
        params.tail_angle,
        params.inner_angle,
        params.width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = BrowShapeParams::default();
        assert!((0.0..=1.0).contains(&p.arch_height));
    }

    #[test]
    fn test_preset_flat_low_arch() {
        let p = preset_params(BrowShapePreset::Flat);
        assert!(p.arch_height < 0.2);
    }

    #[test]
    fn test_preset_peaked_high_arch() {
        let p = preset_params(BrowShapePreset::Peaked);
        assert!(p.arch_height > 0.8);
    }

    #[test]
    fn test_all_presets_count() {
        assert_eq!(all_presets().len(), 8);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = BrowShapeParams {
            arch_height: 0.0,
            ..Default::default()
        };
        let b = BrowShapeParams {
            arch_height: 1.0,
            ..Default::default()
        };
        let r = blend_brow_shape(&a, &b, 0.5);
        assert!((r.arch_height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_brow_height_at_peak() {
        let p = BrowShapeParams {
            arch_height: 1.0,
            peak_position: 0.5,
            tail_angle: 0.0,
            inner_angle: 0.0,
            ..Default::default()
        };
        let h_peak = brow_height_at(0.5, &p);
        let h_edge = brow_height_at(0.0, &p);
        assert!(h_peak > h_edge);
    }

    #[test]
    fn test_brow_height_clamped_x() {
        let p = BrowShapeParams::default();
        let h1 = brow_height_at(-1.0, &p);
        let h2 = brow_height_at(0.0, &p);
        assert!((h1 - h2).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut p = BrowShapeParams {
            arch_height: 0.99,
            ..Default::default()
        };
        reset_brow_shape(&mut p);
        assert!((p.arch_height - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = brow_shape_to_json(&BrowShapeParams::default());
        assert!(j.contains("arch_height"));
        assert!(j.contains("thickness"));
    }

    #[test]
    fn test_preset_bushy_thick() {
        let p = preset_params(BrowShapePreset::Bushy);
        assert!(p.thickness > 0.8);
    }
}
