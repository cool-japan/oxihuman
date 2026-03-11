// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SsgiView {
    pub sample_count: u32,
    pub radius: f32,
    pub intensity: f32,
    pub falloff_exponent: f32,
}

pub fn new_ssgi_view() -> SsgiView {
    SsgiView {
        sample_count: 8,
        radius: 1.0,
        intensity: 1.0,
        falloff_exponent: 2.0,
    }
}

pub fn ssgi_set_sample_count(v: &mut SsgiView, n: u32) {
    v.sample_count = n.clamp(1, 64);
}

pub fn ssgi_irradiance_weight(v: &SsgiView, distance: f32) -> f32 {
    let d = distance.clamp(0.0, v.radius);
    v.intensity * (1.0 - (d / v.radius).powf(v.falloff_exponent))
}

pub fn ssgi_is_high_quality(v: &SsgiView) -> bool {
    v.sample_count >= 32
}

pub fn ssgi_blend(a: &SsgiView, b: &SsgiView, t: f32) -> SsgiView {
    let t = t.clamp(0.0, 1.0);
    let sc = (a.sample_count as f32 + (b.sample_count as f32 - a.sample_count as f32) * t).round()
        as u32;
    SsgiView {
        sample_count: sc.clamp(1, 64),
        radius: a.radius + (b.radius - a.radius) * t,
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        falloff_exponent: a.falloff_exponent + (b.falloff_exponent - a.falloff_exponent) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default sample count */
        let v = new_ssgi_view();
        assert_eq!(v.sample_count, 8);
    }

    #[test]
    fn test_set_sample_count_clamped() {
        /* clamped to 64 */
        let mut v = new_ssgi_view();
        ssgi_set_sample_count(&mut v, 128);
        assert_eq!(v.sample_count, 64);
    }

    #[test]
    fn test_irradiance_weight_at_origin() {
        /* full intensity at distance 0 */
        let v = new_ssgi_view();
        let w = ssgi_irradiance_weight(&v, 0.0);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_not_high_quality_by_default() {
        /* 8 samples is not high quality */
        let v = new_ssgi_view();
        assert!(!ssgi_is_high_quality(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint radius */
        let a = SsgiView {
            sample_count: 8,
            radius: 0.0,
            intensity: 0.0,
            falloff_exponent: 1.0,
        };
        let b = SsgiView {
            sample_count: 8,
            radius: 2.0,
            intensity: 2.0,
            falloff_exponent: 1.0,
        };
        let c = ssgi_blend(&a, &b, 0.5);
        assert!((c.radius - 1.0).abs() < 1e-5);
    }
}
