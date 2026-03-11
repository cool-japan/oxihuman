// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct TaaView {
    pub blend_factor: f32,
    pub jitter_scale: f32,
    pub history_rejection_threshold: f32,
}

pub fn new_taa_view() -> TaaView {
    TaaView {
        blend_factor: 0.1,
        jitter_scale: 1.0,
        history_rejection_threshold: 0.05,
    }
}

pub fn taa_set_blend_factor(v: &mut TaaView, f: f32) {
    v.blend_factor = f.clamp(0.0, 1.0);
}

pub fn taa_halton_jitter(frame: u32, base_x: u32, base_y: u32) -> (f32, f32) {
    fn halton(mut index: u32, base: u32) -> f32 {
        let mut result = 0.0f32;
        let mut denom = 1.0f32;
        while index > 0 {
            denom *= base as f32;
            result += (index % base) as f32 / denom;
            index /= base;
        }
        result
    }
    let n = (frame % 16) + 1;
    (halton(n, base_x) - 0.5, halton(n, base_y) - 0.5)
}

pub fn taa_is_aggressive(v: &TaaView) -> bool {
    v.blend_factor < 0.05
}

pub fn taa_blend(a: &TaaView, b: &TaaView, t: f32) -> TaaView {
    let t = t.clamp(0.0, 1.0);
    TaaView {
        blend_factor: a.blend_factor + (b.blend_factor - a.blend_factor) * t,
        jitter_scale: a.jitter_scale + (b.jitter_scale - a.jitter_scale) * t,
        history_rejection_threshold: a.history_rejection_threshold
            + (b.history_rejection_threshold - a.history_rejection_threshold) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default blend factor */
        let v = new_taa_view();
        assert!((v.blend_factor - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_set_blend_factor_clamped() {
        /* clamped to 1 */
        let mut v = new_taa_view();
        taa_set_blend_factor(&mut v, 2.0);
        assert!((v.blend_factor - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_halton_jitter_range() {
        /* jitter in [-0.5, 0.5] */
        let (jx, jy) = taa_halton_jitter(0, 2, 3);
        assert!((-0.5..=0.5).contains(&jx));
        assert!((-0.5..=0.5).contains(&jy));
    }

    #[test]
    fn test_not_aggressive_by_default() {
        /* blend 0.1 is not aggressive */
        let v = new_taa_view();
        assert!(!taa_is_aggressive(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = TaaView {
            blend_factor: 0.0,
            jitter_scale: 0.0,
            history_rejection_threshold: 0.0,
        };
        let b = TaaView {
            blend_factor: 1.0,
            jitter_scale: 1.0,
            history_rejection_threshold: 1.0,
        };
        let c = taa_blend(&a, &b, 0.5);
        assert!((c.blend_factor - 0.5).abs() < 1e-5);
    }
}
