// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Morph value clipping.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphClip {
    pub min_val: f32,
    pub max_val: f32,
}

#[allow(dead_code)]
pub fn new_morph_clip(min_val: f32, max_val: f32) -> MorphClip {
    MorphClip { min_val, max_val }
}

#[allow(dead_code)]
pub fn mc_clip(clip: &MorphClip, value: f32) -> f32 {
    value.clamp(clip.min_val, clip.max_val)
}

#[allow(dead_code)]
pub fn mc_clip_slice(clip: &MorphClip, values: &[f32]) -> Vec<f32> {
    values.iter().map(|&v| mc_clip(clip, v)).collect()
}

#[allow(dead_code)]
pub fn mc_range(clip: &MorphClip) -> f32 {
    clip.max_val - clip.min_val
}

#[allow(dead_code)]
pub fn mc_normalize(clip: &MorphClip, value: f32) -> f32 {
    let r = mc_range(clip);
    if r == 0.0 { return 0.0; }
    (value - clip.min_val) / r
}

#[allow(dead_code)]
pub fn mc_denormalize(clip: &MorphClip, t: f32) -> f32 {
    clip.min_val + t * mc_range(clip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_below_min() {
        let c = new_morph_clip(0.0, 1.0);
        assert!((mc_clip(&c, -0.5) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_clip_above_max() {
        let c = new_morph_clip(0.0, 1.0);
        assert!((mc_clip(&c, 1.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clip_within_range() {
        let c = new_morph_clip(0.0, 1.0);
        assert!((mc_clip(&c, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clip_slice() {
        let c = new_morph_clip(0.0, 1.0);
        let r = mc_clip_slice(&c, &[-1.0, 0.5, 2.0]);
        assert!((r[0] - 0.0).abs() < 1e-6);
        assert!((r[1] - 0.5).abs() < 1e-6);
        assert!((r[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_range() {
        let c = new_morph_clip(0.2, 0.8);
        assert!((mc_range(&c) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_denormalize_roundtrip() {
        let c = new_morph_clip(0.2, 0.8);
        let v = 0.5;
        let t = mc_normalize(&c, v);
        let back = mc_denormalize(&c, t);
        assert!((back - v).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_at_min() {
        let c = new_morph_clip(0.0, 1.0);
        assert!((mc_normalize(&c, 0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_at_max() {
        let c = new_morph_clip(0.0, 1.0);
        assert!((mc_normalize(&c, 1.0) - 1.0).abs() < 1e-6);
    }
}
