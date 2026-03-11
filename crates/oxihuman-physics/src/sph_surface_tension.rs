// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct SphSurfaceTension {
    pub sigma: f32,
    pub threshold: f32,
}

#[allow(dead_code)]
pub fn new_sph_surface_tension(sigma: f32) -> SphSurfaceTension {
    SphSurfaceTension { sigma, threshold: 0.01 }
}

#[allow(dead_code)]
pub fn sst_color_field_force(st: &SphSurfaceTension, normal: [f32; 3], curvature: f32) -> [f32; 3] {
    [
        -st.sigma * curvature * normal[0],
        -st.sigma * curvature * normal[1],
        -st.sigma * curvature * normal[2],
    ]
}

#[allow(dead_code)]
pub fn sst_curvature_from_normal(normal: [f32; 3]) -> f32 {
    (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt()
}

#[allow(dead_code)]
pub fn sst_sigma(st: &SphSurfaceTension) -> f32 {
    st.sigma
}

#[allow(dead_code)]
pub fn sst_is_surface_particle(st: &SphSurfaceTension, normal_mag: f32) -> bool {
    normal_mag > st.threshold
}

#[allow(dead_code)]
pub fn sst_energy(st: &SphSurfaceTension, surface_area: f32) -> f32 {
    st.sigma * surface_area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_getter() {
        let st = new_sph_surface_tension(0.072);
        assert!((sst_sigma(&st) - 0.072).abs() < 1e-7);
    }

    #[test]
    fn test_color_field_force_direction() {
        let st = new_sph_surface_tension(1.0);
        let force = sst_color_field_force(&st, [1.0, 0.0, 0.0], 1.0);
        assert!(force[0] < 0.0);
    }

    #[test]
    fn test_color_field_force_zero_curvature() {
        let st = new_sph_surface_tension(1.0);
        let force = sst_color_field_force(&st, [1.0, 0.0, 0.0], 0.0);
        assert!((force[0]).abs() < 1e-7);
    }

    #[test]
    fn test_curvature_from_normal_unit() {
        let c = sst_curvature_from_normal([1.0, 0.0, 0.0]);
        assert!((c - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_curvature_from_normal_zero() {
        let c = sst_curvature_from_normal([0.0; 3]);
        assert!((c).abs() < 1e-6);
    }

    #[test]
    fn test_is_surface_particle_above() {
        let st = new_sph_surface_tension(1.0);
        assert!(sst_is_surface_particle(&st, 0.1));
    }

    #[test]
    fn test_is_surface_particle_below() {
        let st = new_sph_surface_tension(1.0);
        assert!(!sst_is_surface_particle(&st, 0.005));
    }

    #[test]
    fn test_energy() {
        let st = new_sph_surface_tension(0.5);
        assert!((sst_energy(&st, 2.0) - 1.0).abs() < 1e-6);
    }
}
