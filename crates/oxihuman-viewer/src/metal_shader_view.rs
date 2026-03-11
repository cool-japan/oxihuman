// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct MetalShaderView {
    pub show_fresnel: bool,
    pub ior_n: [f32; 3],
    pub ior_k: [f32; 3],
    pub roughness: f32,
}

pub fn new_metal_shader_view(ior_n: [f32; 3], ior_k: [f32; 3]) -> MetalShaderView {
    MetalShaderView {
        show_fresnel: true,
        ior_n,
        ior_k,
        roughness: 0.1,
    }
}

/// Conductor Fresnel for a single channel.
pub fn metal_fresnel_conductor(cos_theta: f32, n: f32, k: f32) -> f32 {
    let cos2 = cos_theta * cos_theta;
    let sin2 = (1.0 - cos2).max(0.0);
    let n2_k2 = n * n + k * k;
    let a2_plus_b2 =
        (n2_k2 * n2_k2 - 2.0 * n2_k2 * sin2 + sin2 * sin2 + 4.0 * n * n * k * k).sqrt();
    let a = ((a2_plus_b2 + n2_k2 - sin2) * 0.5).sqrt();
    let rs_num = a2_plus_b2 - 2.0 * a * cos_theta + cos2;
    let rs_den = a2_plus_b2 + 2.0 * a * cos_theta + cos2;
    let rs = rs_num / rs_den.max(1e-9);
    let rp_num = rs * (cos2 * a2_plus_b2 - 2.0 * a * sin2 + sin2 * sin2 / cos2.max(1e-9));
    let rp_den = cos2 * a2_plus_b2 + 2.0 * a * sin2 + sin2 * sin2 / cos2.max(1e-9);
    let rp = rp_num / rp_den.max(1e-9);
    ((rs + rp) * 0.5).clamp(0.0, 1.0)
}

pub fn metal_fresnel_rgb(cos_theta: f32, v: &MetalShaderView) -> [f32; 3] {
    [
        metal_fresnel_conductor(cos_theta, v.ior_n[0], v.ior_k[0]),
        metal_fresnel_conductor(cos_theta, v.ior_n[1], v.ior_k[1]),
        metal_fresnel_conductor(cos_theta, v.ior_n[2], v.ior_k[2]),
    ]
}

pub fn metal_is_mirror(roughness: f32) -> bool {
    roughness < 0.01
}

pub fn metal_reflectance_at_normal(v: &MetalShaderView) -> [f32; 3] {
    metal_fresnel_rgb(1.0, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metal_shader_view() {
        /* roughness defaults to 0.1 */
        let v = new_metal_shader_view([0.2, 0.2, 0.2], [3.0, 3.0, 3.0]);
        assert!((v.roughness - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_metal_fresnel_conductor_range() {
        /* fresnel should be in [0,1] */
        let f = metal_fresnel_conductor(0.5, 0.2, 3.0);
        assert!((0.0..=1.0).contains(&f));
    }

    #[test]
    fn test_metal_fresnel_rgb() {
        /* RGB fresnel should all be in [0,1] */
        let v = new_metal_shader_view([0.2, 0.2, 0.2], [3.0, 3.0, 3.0]);
        let c = metal_fresnel_rgb(0.7, &v);
        for ch in c.iter() {
            assert!(*ch >= 0.0 && *ch <= 1.0);
        }
    }

    #[test]
    fn test_metal_is_mirror() {
        /* roughness < 0.01 is mirror */
        assert!(metal_is_mirror(0.005));
        assert!(!metal_is_mirror(0.5));
    }

    #[test]
    fn test_metal_reflectance_at_normal() {
        /* reflectance at normal incidence is valid */
        let v = new_metal_shader_view([0.2, 0.2, 0.2], [3.0, 3.0, 3.0]);
        let r = metal_reflectance_at_normal(&v);
        assert!(r[0] >= 0.0);
    }
}
