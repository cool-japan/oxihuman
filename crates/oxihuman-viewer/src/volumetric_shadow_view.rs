// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct VolShadowView {
    pub extinction: f32,
    pub scattering_albedo: f32,
    pub march_steps: u32,
}

pub fn new_vol_shadow_view() -> VolShadowView {
    VolShadowView {
        extinction: 0.1,
        scattering_albedo: 0.5,
        march_steps: 32,
    }
}

pub fn vol_shadow_set_extinction(v: &mut VolShadowView, e: f32) {
    v.extinction = e.clamp(0.0, 10.0);
}

/// Beer-Lambert transmittance over distance d.
pub fn vol_shadow_transmittance(v: &VolShadowView, d: f32) -> f32 {
    (-v.extinction * d).exp()
}

pub fn vol_shadow_is_dense(v: &VolShadowView) -> bool {
    v.extinction > 1.0
}

pub fn vol_shadow_blend(a: &VolShadowView, b: &VolShadowView, t: f32) -> VolShadowView {
    let t = t.clamp(0.0, 1.0);
    let ms =
        (a.march_steps as f32 + (b.march_steps as f32 - a.march_steps as f32) * t).round() as u32;
    VolShadowView {
        extinction: a.extinction + (b.extinction - a.extinction) * t,
        scattering_albedo: a.scattering_albedo + (b.scattering_albedo - a.scattering_albedo) * t,
        march_steps: ms.max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default march steps */
        let v = new_vol_shadow_view();
        assert_eq!(v.march_steps, 32);
    }

    #[test]
    fn test_transmittance_at_zero() {
        /* transmittance is 1 at distance 0 */
        let v = new_vol_shadow_view();
        assert!((vol_shadow_transmittance(&v, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_transmittance_decreases() {
        /* transmittance decreases with distance */
        let v = new_vol_shadow_view();
        let t1 = vol_shadow_transmittance(&v, 1.0);
        let t2 = vol_shadow_transmittance(&v, 2.0);
        assert!(t2 < t1);
    }

    #[test]
    fn test_not_dense_by_default() {
        /* default extinction 0.1 is not dense */
        let v = new_vol_shadow_view();
        assert!(!vol_shadow_is_dense(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint extinction */
        let a = VolShadowView {
            extinction: 0.0,
            scattering_albedo: 0.0,
            march_steps: 16,
        };
        let b = VolShadowView {
            extinction: 2.0,
            scattering_albedo: 1.0,
            march_steps: 16,
        };
        let c = vol_shadow_blend(&a, &b, 0.5);
        assert!((c.extinction - 1.0).abs() < 1e-5);
    }
}
