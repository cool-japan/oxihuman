// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Variance Shadow Maps parameters.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VSMShadow {
    pub min_variance: f32,
    pub light_bleed_reduction: f32,
}

#[allow(dead_code)]
pub fn new_vsm_shadow(min_variance: f32, light_bleed_reduction: f32) -> VSMShadow {
    VSMShadow { min_variance, light_bleed_reduction }
}

#[allow(dead_code)]
pub fn vsm_chebyshev(mean: f32, variance: f32, t: f32) -> f32 {
    if t <= mean {
        return 1.0;
    }
    let diff = t - mean;
    let p_max = variance / (variance + diff * diff);
    p_max.clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn vsm_evaluate(shadow: &VSMShadow, mean: f32, variance: f32, depth: f32) -> f32 {
    let v = variance.max(shadow.min_variance);
    let p = vsm_chebyshev(mean, v, depth);
    vsm_reduce_bleed(shadow, p)
}

#[allow(dead_code)]
pub fn vsm_reduce_bleed(shadow: &VSMShadow, p: f32) -> f32 {
    let lbr = shadow.light_bleed_reduction;
    /* smoothstep remapping */
    let t = ((p - lbr) / (1.0 - lbr)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[allow(dead_code)]
pub fn vsm_min_variance(shadow: &VSMShadow) -> f32 {
    shadow.min_variance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chebyshev_at_or_below_mean() {
        let p = vsm_chebyshev(0.5, 0.01, 0.5);
        assert!((p - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_chebyshev_above_mean_less_than_one() {
        let p = vsm_chebyshev(0.5, 0.01, 0.9);
        assert!(p < 1.0);
        assert!(p >= 0.0);
    }

    #[test]
    fn test_evaluate_returns_finite() {
        let s = new_vsm_shadow(0.00001, 0.1);
        let r = vsm_evaluate(&s, 0.5, 0.01, 0.8);
        assert!(r.is_finite());
    }

    #[test]
    fn test_reduce_bleed_clamped() {
        let s = new_vsm_shadow(0.00001, 0.2);
        let r = vsm_reduce_bleed(&s, 0.5);
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn test_reduce_bleed_full_lit() {
        let s = new_vsm_shadow(0.00001, 0.0);
        let r = vsm_reduce_bleed(&s, 1.0);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_min_variance_getter() {
        let s = new_vsm_shadow(0.0001, 0.1);
        assert!((vsm_min_variance(&s) - 0.0001).abs() < 1e-9);
    }

    #[test]
    fn test_reduce_bleed_zero_p() {
        let s = new_vsm_shadow(0.00001, 0.2);
        let r = vsm_reduce_bleed(&s, 0.0);
        assert!((r - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_in_range() {
        let s = new_vsm_shadow(0.001, 0.05);
        let r = vsm_evaluate(&s, 0.5, 0.001, 0.6);
        assert!((0.0..=1.0).contains(&r));
    }
}
