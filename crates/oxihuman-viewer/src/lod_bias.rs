// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! LOD bias for quality vs performance trade-off.

#[allow(dead_code)]
pub struct LodBias {
    pub bias: f32,
    pub min_lod: u32,
    pub max_lod: u32,
}

#[allow(dead_code)]
pub fn new_lod_bias(bias: f32) -> LodBias {
    LodBias { bias, min_lod: 0, max_lod: 8 }
}

#[allow(dead_code)]
pub fn lb_effective_lod(b: &LodBias, base_lod: u32) -> u32 {
    let biased = base_lod as f32 + b.bias;
    (biased.round() as u32).clamp(b.min_lod, b.max_lod)
}

#[allow(dead_code)]
pub fn lb_set_bias(b: &mut LodBias, bias: f32) {
    b.bias = bias;
}

#[allow(dead_code)]
pub fn lb_bias(b: &LodBias) -> f32 {
    b.bias
}

#[allow(dead_code)]
pub fn lb_quality_hint(b: &LodBias) -> f32 {
    let max = b.max_lod as f32;
    if max < 1e-7 {
        return 1.0;
    }
    1.0 - b.bias.clamp(0.0, max) / max
}

#[allow(dead_code)]
pub fn lb_set_range(b: &mut LodBias, min: u32, max: u32) {
    b.min_lod = min;
    b.max_lod = max;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_lod_no_bias() {
        let b = new_lod_bias(0.0);
        assert_eq!(lb_effective_lod(&b, 3), 3);
    }

    #[test]
    fn test_effective_lod_bias_up() {
        let b = new_lod_bias(2.0);
        assert_eq!(lb_effective_lod(&b, 3), 5);
    }

    #[test]
    fn test_effective_lod_clamped() {
        let b = new_lod_bias(100.0);
        assert_eq!(lb_effective_lod(&b, 0), 8);
    }

    #[test]
    fn test_effective_lod_min_clamp() {
        let b = new_lod_bias(-10.0);
        assert_eq!(lb_effective_lod(&b, 0), 0);
    }

    #[test]
    fn test_set_bias() {
        let mut b = new_lod_bias(0.0);
        lb_set_bias(&mut b, 1.5);
        assert!((lb_bias(&b) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_bias_getter() {
        let b = new_lod_bias(2.0);
        assert!((lb_bias(&b) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_quality_hint_zero_bias() {
        let b = new_lod_bias(0.0);
        assert!((lb_quality_hint(&b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_range() {
        let mut b = new_lod_bias(0.0);
        lb_set_range(&mut b, 1, 4);
        assert_eq!(b.min_lod, 1);
        assert_eq!(b.max_lod, 4);
    }
}
