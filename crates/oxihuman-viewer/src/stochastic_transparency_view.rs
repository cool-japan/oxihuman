// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct StochasticTransparencyView {
    pub sample_count: u32,
    pub alpha_correction: bool,
    pub stencil_mask_bits: u32,
}

pub fn new_stochastic_transparency_view() -> StochasticTransparencyView {
    StochasticTransparencyView {
        sample_count: 8,
        alpha_correction: true,
        stencil_mask_bits: 8,
    }
}

pub fn st_set_sample_count(v: &mut StochasticTransparencyView, n: u32) {
    v.sample_count = n.clamp(1, 256);
}

/// Stochastic alpha test: compare random [0,1) against alpha.
pub fn st_alpha_test(alpha: f32, random: f32) -> bool {
    random < alpha.clamp(0.0, 1.0)
}

pub fn st_effective_alpha(v: &StochasticTransparencyView, alpha: f32) -> f32 {
    let samples = v.sample_count as f32;
    let covered = (alpha * samples).round();
    covered / samples
}

pub fn st_is_high_sample(v: &StochasticTransparencyView) -> bool {
    v.sample_count >= 64
}

pub fn st_blend(
    a: &StochasticTransparencyView,
    b: &StochasticTransparencyView,
    t: f32,
) -> StochasticTransparencyView {
    let t = t.clamp(0.0, 1.0);
    let sc = (a.sample_count as f32 + (b.sample_count as f32 - a.sample_count as f32) * t).round()
        as u32;
    StochasticTransparencyView {
        sample_count: sc.clamp(1, 256),
        alpha_correction: if t < 0.5 {
            a.alpha_correction
        } else {
            b.alpha_correction
        },
        stencil_mask_bits: if t < 0.5 {
            a.stencil_mask_bits
        } else {
            b.stencil_mask_bits
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default sample count */
        let v = new_stochastic_transparency_view();
        assert_eq!(v.sample_count, 8);
    }

    #[test]
    fn test_alpha_test_pass() {
        /* random below alpha passes */
        assert!(st_alpha_test(0.9, 0.1));
    }

    #[test]
    fn test_alpha_test_fail() {
        /* random above alpha fails */
        assert!(!st_alpha_test(0.1, 0.9));
    }

    #[test]
    fn test_effective_alpha_range() {
        /* effective alpha in [0, 1] */
        let v = new_stochastic_transparency_view();
        let ea = st_effective_alpha(&v, 0.5);
        assert!((0.0..=1.0).contains(&ea));
    }

    #[test]
    fn test_not_high_sample_by_default() {
        /* 8 samples is not high sample */
        let v = new_stochastic_transparency_view();
        assert!(!st_is_high_sample(&v));
    }
}
