// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ClearcoatView {
    pub show_normals: bool,
    pub show_factor: bool,
    pub show_roughness: bool,
}

pub fn new_clearcoat_view() -> ClearcoatView {
    ClearcoatView {
        show_normals: false,
        show_factor: true,
        show_roughness: false,
    }
}

pub fn clearcoat_factor_color(factor: f32) -> [f32; 3] {
    let f = factor.clamp(0.0, 1.0);
    [f, f, f]
}

pub fn clearcoat_roughness_color(roughness: f32) -> [f32; 3] {
    let r = roughness.clamp(0.0, 1.0);
    [r, 1.0 - r, 0.0]
}

pub fn clearcoat_fresnel(cos_theta: f32, ior: f32) -> f32 {
    /* Schlick approximation */
    let r0 = ((1.0 - ior) / (1.0 + ior)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos_theta.clamp(0.0, 1.0)).powi(5)
}

pub fn clearcoat_is_visible(factor: f32) -> bool {
    factor > 0.01
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clearcoat_view() {
        /* show_factor defaults to true */
        let v = new_clearcoat_view();
        assert!(v.show_factor);
    }

    #[test]
    fn test_clearcoat_factor_color_black() {
        /* factor=0 -> black */
        let c = clearcoat_factor_color(0.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_clearcoat_roughness_color() {
        /* roughness=0 -> [0,1,0] */
        let c = clearcoat_roughness_color(0.0);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clearcoat_fresnel_normal() {
        /* at cos_theta=1 (normal incidence) fresnel should equal r0 */
        let f = clearcoat_fresnel(1.0, 1.5);
        let r0 = ((1.0 - 1.5f32) / (1.0 + 1.5f32)).powi(2);
        assert!((f - r0).abs() < 1e-5);
    }

    #[test]
    fn test_clearcoat_is_visible() {
        /* factor > 0.01 is visible */
        assert!(clearcoat_is_visible(0.5));
        assert!(!clearcoat_is_visible(0.0));
    }
}
