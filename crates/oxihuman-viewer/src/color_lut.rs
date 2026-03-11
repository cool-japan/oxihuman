// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Color LUT — 3-D color look-up table for color grading post-process.

/// Size of each LUT dimension (e.g., 32 → 32×32×32 entries).
pub const LUT_DIM: usize = 16;
pub const LUT_SIZE: usize = LUT_DIM * LUT_DIM * LUT_DIM;

/// A 3-D color LUT.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorLut {
    /// Flattened RGB data in [0.0, 1.0]. Length must be LUT_SIZE * 3.
    pub data: Vec<f32>,
    pub name: String,
    pub intensity: f32,
}

#[allow(dead_code)]
pub fn identity_color_lut(name: &str) -> ColorLut {
    let mut data = Vec::with_capacity(LUT_SIZE * 3);
    for b in 0..LUT_DIM {
        for g in 0..LUT_DIM {
            for r in 0..LUT_DIM {
                data.push(r as f32 / (LUT_DIM - 1) as f32);
                data.push(g as f32 / (LUT_DIM - 1) as f32);
                data.push(b as f32 / (LUT_DIM - 1) as f32);
            }
        }
    }
    ColorLut {
        data,
        name: name.to_string(),
        intensity: 1.0,
    }
}

#[allow(dead_code)]
pub fn lut_set_intensity(lut: &mut ColorLut, v: f32) {
    lut.intensity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn lut_sample(lut: &ColorLut, r: f32, g: f32, b: f32) -> [f32; 3] {
    let ri = (r.clamp(0.0, 1.0) * (LUT_DIM - 1) as f32) as usize;
    let gi = (g.clamp(0.0, 1.0) * (LUT_DIM - 1) as f32) as usize;
    let bi = (b.clamp(0.0, 1.0) * (LUT_DIM - 1) as f32) as usize;
    let ri = ri.min(LUT_DIM - 1);
    let gi = gi.min(LUT_DIM - 1);
    let bi = bi.min(LUT_DIM - 1);
    let idx = (bi * LUT_DIM * LUT_DIM + gi * LUT_DIM + ri) * 3;
    if idx + 2 < lut.data.len() {
        [lut.data[idx], lut.data[idx + 1], lut.data[idx + 2]]
    } else {
        [r, g, b]
    }
}

#[allow(dead_code)]
pub fn lut_apply_with_intensity(lut: &ColorLut, color: [f32; 3]) -> [f32; 3] {
    let sampled = lut_sample(lut, color[0], color[1], color[2]);
    let t = lut.intensity;
    [
        color[0] + (sampled[0] - color[0]) * t,
        color[1] + (sampled[1] - color[1]) * t,
        color[2] + (sampled[2] - color[2]) * t,
    ]
}

#[allow(dead_code)]
pub fn lut_is_identity(lut: &ColorLut) -> bool {
    if lut.data.len() != LUT_SIZE * 3 {
        return false;
    }
    for b in 0..LUT_DIM {
        for g in 0..LUT_DIM {
            for r in 0..LUT_DIM {
                let idx = (b * LUT_DIM * LUT_DIM + g * LUT_DIM + r) * 3;
                let er = r as f32 / (LUT_DIM - 1) as f32;
                let eg = g as f32 / (LUT_DIM - 1) as f32;
                let eb = b as f32 / (LUT_DIM - 1) as f32;
                if (lut.data[idx] - er).abs() > 1e-5
                    || (lut.data[idx + 1] - eg).abs() > 1e-5
                    || (lut.data[idx + 2] - eb).abs() > 1e-5
                {
                    return false;
                }
            }
        }
    }
    true
}

#[allow(dead_code)]
pub fn lut_data_len(lut: &ColorLut) -> usize {
    lut.data.len()
}

#[allow(dead_code)]
pub fn lut_to_json(lut: &ColorLut) -> String {
    format!(
        r#"{{"name":"{}","intensity":{:.4},"size":{}}}"#,
        lut.name, lut.intensity, LUT_DIM
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_lut_is_identity() {
        let lut = identity_color_lut("identity");
        assert!(lut_is_identity(&lut));
    }

    #[test]
    fn identity_sample_passthrough() {
        let lut = identity_color_lut("id");
        let out = lut_sample(&lut, 0.5, 0.25, 0.75);
        assert!((out[0] - 0.5).abs() < 0.1);
    }

    #[test]
    fn lut_data_len_correct() {
        let lut = identity_color_lut("id");
        assert_eq!(lut_data_len(&lut), LUT_SIZE * 3);
    }

    #[test]
    fn set_intensity_clamps() {
        let mut lut = identity_color_lut("id");
        lut_set_intensity(&mut lut, 5.0);
        assert!((lut.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn apply_at_zero_intensity_returns_original() {
        let mut lut = identity_color_lut("id");
        lut_set_intensity(&mut lut, 0.0);
        let color = [0.3, 0.6, 0.9];
        let out = lut_apply_with_intensity(&lut, color);
        assert!((out[0] - color[0]).abs() < 1e-6);
    }

    #[test]
    fn sample_corner_black() {
        let lut = identity_color_lut("id");
        let out = lut_sample(&lut, 0.0, 0.0, 0.0);
        assert!(out.iter().all(|&v| v.abs() < 0.1));
    }

    #[test]
    fn sample_corner_white() {
        let lut = identity_color_lut("id");
        let out = lut_sample(&lut, 1.0, 1.0, 1.0);
        assert!(out.iter().all(|&v| (v - 1.0).abs() < 0.1));
    }

    #[test]
    fn to_json_name() {
        let lut = identity_color_lut("cinematic");
        let j = lut_to_json(&lut);
        assert!(j.contains("cinematic"));
        assert!(j.contains("intensity"));
    }

    #[test]
    fn name_stored() {
        let lut = identity_color_lut("vivid");
        assert_eq!(lut.name, "vivid".to_string());
    }
}
