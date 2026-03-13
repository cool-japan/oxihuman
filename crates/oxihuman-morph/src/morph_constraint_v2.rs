// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Morph constraints v2: range, sum, and dependency constraints.

#[allow(dead_code)]
pub enum MorphConstraintV2Kind {
    Range(f32, f32),
    MaxSum(f32),
    DrivenBy(usize, f32),
}

#[allow(dead_code)]
pub struct MorphConstraintV2 {
    pub morph_idx: usize,
    pub kind: MorphConstraintV2Kind,
}

#[allow(dead_code)]
pub fn mc2_apply_range(weights: &mut [f32], idx: usize, lo: f32, hi: f32) {
    if idx < weights.len() {
        weights[idx] = weights[idx].clamp(lo, hi);
    }
}

#[allow(dead_code)]
pub fn mc2_apply_max_sum(weights: &mut [f32], max_sum: f32) {
    let sum: f32 = weights.iter().sum();
    if sum > max_sum && sum > 1e-7 {
        let scale = max_sum / sum;
        for w in weights.iter_mut() {
            *w *= scale;
        }
    }
}

#[allow(dead_code)]
pub fn mc2_apply_driven(weights: &mut [f32], driven_idx: usize, driver_idx: usize, gain: f32) {
    if driver_idx < weights.len() && driven_idx < weights.len() {
        let driver_val = weights[driver_idx];
        weights[driven_idx] = driver_val * gain;
    }
}

#[allow(dead_code)]
pub fn mc2_apply_all(weights: &mut [f32], constraints: &[MorphConstraintV2]) {
    for c in constraints {
        match &c.kind {
            MorphConstraintV2Kind::Range(lo, hi) => mc2_apply_range(weights, c.morph_idx, *lo, *hi),
            MorphConstraintV2Kind::MaxSum(max) => mc2_apply_max_sum(weights, *max),
            MorphConstraintV2Kind::DrivenBy(driver_idx, gain) => {
                mc2_apply_driven(weights, c.morph_idx, *driver_idx, *gain)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_range_clamps_high() {
        let mut w = vec![1.5, 0.5];
        mc2_apply_range(&mut w, 0, 0.0, 1.0);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_range_clamps_low() {
        let mut w = vec![-0.5, 0.5];
        mc2_apply_range(&mut w, 0, 0.0, 1.0);
        assert!(w[0].abs() < 1e-5);
    }

    #[test]
    fn test_apply_max_sum_scales() {
        let mut w = vec![0.6, 0.6];
        mc2_apply_max_sum(&mut w, 1.0);
        let sum: f32 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_apply_max_sum_under_limit() {
        let mut w = vec![0.3, 0.3];
        mc2_apply_max_sum(&mut w, 1.0);
        assert!((w[0] - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_apply_driven() {
        let mut w = vec![0.5, 0.0];
        mc2_apply_driven(&mut w, 1, 0, 2.0);
        assert!((w[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_all_range() {
        let mut w = vec![1.5];
        let c = vec![MorphConstraintV2 { morph_idx: 0, kind: MorphConstraintV2Kind::Range(0.0, 1.0) }];
        mc2_apply_all(&mut w, &c);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_all_driven() {
        let mut w = vec![0.8, 0.0];
        let c = vec![MorphConstraintV2 { morph_idx: 1, kind: MorphConstraintV2Kind::DrivenBy(0, 0.5) }];
        mc2_apply_all(&mut w, &c);
        assert!((w[1] - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_apply_range_out_of_bounds_safe() {
        let mut w = vec![0.5];
        mc2_apply_range(&mut w, 99, 0.0, 1.0); /* should not panic */
    }
}
