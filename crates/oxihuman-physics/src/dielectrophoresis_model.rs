// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dielectrophoresis (DEP) force model stub.

/// DEP configuration for a spherical particle.
#[derive(Debug, Clone)]
pub struct DepConfig {
    pub particle_radius: f32,       /* m */
    pub medium_permittivity: f32,   /* F/m */
    pub particle_permittivity: f32, /* F/m */
    pub medium_conductivity: f32,   /* S/m */
    pub particle_conductivity: f32, /* S/m */
}

impl DepConfig {
    pub fn new(
        particle_radius: f32,
        medium_permittivity: f32,
        particle_permittivity: f32,
        medium_conductivity: f32,
        particle_conductivity: f32,
    ) -> Self {
        DepConfig {
            particle_radius,
            medium_permittivity,
            particle_permittivity,
            medium_conductivity,
            particle_conductivity,
        }
    }

    pub fn polystyrene_in_water() -> Self {
        DepConfig::new(5e-6, 80.0 * 8.854e-12, 2.5 * 8.854e-12, 0.01, 1e-4)
    }
}

impl Default for DepConfig {
    fn default() -> Self {
        Self::polystyrene_in_water()
    }
}

/// Clausius-Mossotti factor (real part) at a given frequency.
pub fn clausius_mossotti_real(config: &DepConfig, frequency: f32) -> f32 {
    let omega = 2.0 * std::f32::consts::PI * frequency.max(0.0);
    let eps_p = config.particle_permittivity;
    let eps_m = config.medium_permittivity;
    let sigma_p = config.particle_conductivity;
    let sigma_m = config.medium_conductivity;
    /* Complex permittivity: eps* = eps - j*sigma/omega */
    if omega < 1e-10 {
        /* DC limit */
        if (sigma_p + 2.0 * sigma_m).abs() < 1e-30 {
            return 0.0;
        }
        return (sigma_p - sigma_m) / (sigma_p + 2.0 * sigma_m);
    }
    let num_re = eps_p - eps_m + (sigma_p - sigma_m) / omega;
    let denom_re = eps_p + 2.0 * eps_m + (sigma_p + 2.0 * sigma_m) / omega;
    if denom_re.abs() < 1e-30 {
        0.0
    } else {
        num_re / denom_re
    }
}

/// Time-averaged DEP force on a sphere in a non-uniform field.
/// F_DEP = 2*pi*r^3*eps_m*Re`[K]`*grad(|E|^2)
pub fn dep_force(config: &DepConfig, frequency: f32, grad_e_sq: f32) -> f32 {
    let k = clausius_mossotti_real(config, frequency);
    let r = config.particle_radius;
    2.0 * std::f32::consts::PI * r * r * r * config.medium_permittivity * k * grad_e_sq
}

/// Is particle undergoing positive DEP (attracted to field maxima)?
pub fn is_positive_dep(config: &DepConfig, frequency: f32) -> bool {
    clausius_mossotti_real(config, frequency) > 0.0
}

/// DEP crossover frequency (stub: geometric mean of sigma-controlled and eps-controlled regimes).
pub fn crossover_frequency(config: &DepConfig) -> f32 {
    let sigma_p = config.particle_conductivity;
    let sigma_m = config.medium_conductivity;
    let eps_p = config.particle_permittivity;
    let eps_m = config.medium_permittivity;
    /* f_c ≈ (1/(2*pi)) * sqrt((sigma_p - sigma_m)(sigma_p + 2*sigma_m) / ((eps_p - eps_m)(eps_p + 2*eps_m))) */
    let num = (sigma_p - sigma_m) * (sigma_p + 2.0 * sigma_m);
    let denom = (eps_p - eps_m) * (eps_p + 2.0 * eps_m);
    if denom.abs() < 1e-30 || num / denom < 0.0 {
        return 0.0;
    }
    (num / denom).sqrt() / (2.0 * std::f32::consts::PI)
}

/// Particle velocity under DEP in a viscous medium (Stokes drag balance).
pub fn dep_velocity(config: &DepConfig, frequency: f32, grad_e_sq: f32, viscosity: f32) -> f32 {
    if viscosity <= 0.0 {
        return 0.0;
    }
    let f = dep_force(config, frequency, grad_e_sq);
    let r = config.particle_radius;
    f / (6.0 * std::f32::consts::PI * viscosity * r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clausius_mossotti_range() {
        let c = DepConfig::default();
        let k = clausius_mossotti_real(&c, 1e6);
        assert!((-0.5..=1.0).contains(&k));
    }

    #[test]
    fn test_dep_force_sign_positive_dep() {
        let c = DepConfig::default();
        /* At high frequency, polystyrene (eps < water) => negative DEP */
        let k = clausius_mossotti_real(&c, 1e9);
        let f = dep_force(&c, 1e9, 1e12);
        assert_eq!(f >= 0.0, k >= 0.0);
    }

    #[test]
    fn test_dep_force_zero_grad() {
        let c = DepConfig::default();
        assert_eq!(dep_force(&c, 1e6, 0.0), 0.0);
    }

    #[test]
    fn test_crossover_frequency_non_negative() {
        let c = DepConfig::default();
        assert!(crossover_frequency(&c) >= 0.0);
    }

    #[test]
    fn test_dep_velocity_zero_viscosity() {
        let c = DepConfig::default();
        assert_eq!(dep_velocity(&c, 1e6, 1e12, 0.0), 0.0);
    }

    #[test]
    fn test_dep_velocity_finite() {
        let c = DepConfig::default();
        let v = dep_velocity(&c, 1e6, 1e12, 0.001);
        assert!(v.is_finite());
    }

    #[test]
    fn test_is_positive_dep_type() {
        let c = DepConfig::default();
        /* Just call to ensure no panic; result direction depends on material */
        let _pos = is_positive_dep(&c, 1e3);
    }

    #[test]
    fn test_dep_force_scales_with_radius_cubed() {
        let c1 = DepConfig::polystyrene_in_water();
        let mut c2 = c1.clone();
        c2.particle_radius *= 2.0;
        let f1 = dep_force(&c1, 1e6, 1e10).abs();
        let f2 = dep_force(&c2, 1e6, 1e10).abs();
        /* Force scales as r^3, so ratio should be ~8 */
        if f1 > 1e-30 {
            assert!((f2 / f1 - 8.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_clausius_mossotti_dc_limit() {
        let c = DepConfig::default();
        let k_dc = clausius_mossotti_real(&c, 0.0);
        assert!(k_dc.is_finite());
    }

    #[test]
    fn test_default_config() {
        let c = DepConfig::default();
        assert!(c.particle_radius > 0.0);
    }
}
