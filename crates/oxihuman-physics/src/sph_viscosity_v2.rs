// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct SphViscosity {
    pub mu: f32,
    pub alpha: f32,
    pub beta: f32,
}

#[allow(dead_code)]
pub fn new_sph_viscosity(mu: f32) -> SphViscosity {
    SphViscosity { mu, alpha: 0.1, beta: 0.2 }
}

#[allow(dead_code)]
pub fn sv_dynamic_force(v: &SphViscosity, vel_diff: [f32; 3], laplacian_w: f32) -> [f32; 3] {
    [
        v.mu * laplacian_w * vel_diff[0],
        v.mu * laplacian_w * vel_diff[1],
        v.mu * laplacian_w * vel_diff[2],
    ]
}

#[allow(dead_code)]
pub fn sv_artificial_visc(
    v: &SphViscosity,
    vel_diff: [f32; 3],
    pos_diff: [f32; 3],
    c_sound: f32,
    density: f32,
) -> f32 {
    let vdp = vel_diff[0] * pos_diff[0] + vel_diff[1] * pos_diff[1] + vel_diff[2] * pos_diff[2];
    if vdp >= 0.0 {
        return 0.0;
    }
    let r2 = pos_diff[0].powi(2) + pos_diff[1].powi(2) + pos_diff[2].powi(2);
    let eta2 = 0.01 * r2;
    let mu_ij = r2.sqrt() * vdp / (r2 + eta2);
    let rho = density.max(1e-7);
    (-v.alpha * c_sound * mu_ij + v.beta * mu_ij.powi(2)) / rho
}

#[allow(dead_code)]
pub fn sv_mu(v: &SphViscosity) -> f32 {
    v.mu
}

#[allow(dead_code)]
pub fn sv_set_mu(v: &mut SphViscosity, mu: f32) {
    v.mu = mu;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_force_zero_vel_diff() {
        let v = new_sph_viscosity(0.5);
        let force = sv_dynamic_force(&v, [0.0; 3], 1.0);
        assert!((force[0]).abs() < 1e-7);
        assert!((force[1]).abs() < 1e-7);
        assert!((force[2]).abs() < 1e-7);
    }

    #[test]
    fn test_dynamic_force_proportional() {
        let v = new_sph_viscosity(1.0);
        let f1 = sv_dynamic_force(&v, [1.0, 0.0, 0.0], 2.0);
        assert!((f1[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_mu_getter() {
        let v = new_sph_viscosity(2.71);
        assert!((sv_mu(&v) - 2.71).abs() < 1e-6);
    }

    #[test]
    fn test_set_mu() {
        let mut v = new_sph_viscosity(1.0);
        sv_set_mu(&mut v, 5.0);
        assert!((sv_mu(&v) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_artificial_visc_approaching() {
        let v = new_sph_viscosity(0.01);
        let visc = sv_artificial_visc(&v, [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 343.0, 1000.0);
        assert!(visc >= 0.0);
    }

    #[test]
    fn test_artificial_visc_receding() {
        let v = new_sph_viscosity(0.01);
        let visc = sv_artificial_visc(&v, [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 343.0, 1000.0);
        assert!((visc).abs() < 1e-6);
    }

    #[test]
    fn test_alpha_beta_defaults() {
        let v = new_sph_viscosity(1.0);
        assert!((v.alpha - 0.1).abs() < 1e-6);
        assert!((v.beta - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_dynamic_force_direction() {
        let v = new_sph_viscosity(1.0);
        let f = sv_dynamic_force(&v, [0.0, 1.0, 0.0], 1.0);
        assert!((f[1] - 1.0).abs() < 1e-6);
    }
}
