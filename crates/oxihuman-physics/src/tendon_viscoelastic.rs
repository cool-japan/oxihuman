// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Kelvin-Voigt viscoelastic tendon model.
pub struct Tendon {
    pub stiffness: f32,
    pub viscosity: f32,
    pub rest_length: f32,
    pub current_length: f32,
}

impl Tendon {
    pub fn new(stiffness: f32, viscosity: f32, rest_length: f32) -> Self {
        Tendon {
            stiffness,
            viscosity,
            rest_length,
            current_length: rest_length,
        }
    }
}

pub fn new_tendon(stiffness: f32, viscosity: f32, rest_length: f32) -> Tendon {
    Tendon::new(stiffness, viscosity, rest_length)
}

pub fn tendon_strain(t: &Tendon) -> f32 {
    (t.current_length - t.rest_length) / t.rest_length
}

/// F = k * strain + η * strain_rate  (velocity is strain rate here)
pub fn tendon_force(t: &Tendon, velocity: f32) -> f32 {
    let strain = tendon_strain(t);
    t.stiffness * strain + t.viscosity * velocity / t.rest_length
}

pub fn tendon_elongate(t: &mut Tendon, delta: f32) {
    t.current_length += delta;
    if t.current_length < 0.0 {
        t.current_length = 0.0;
    }
}

/// Exponential relaxation: strain decays toward zero.
pub fn tendon_relax(t: &mut Tendon, dt: f32) {
    let tau = t.viscosity / t.stiffness;
    if tau > 0.0 {
        let excess = t.current_length - t.rest_length;
        t.current_length = t.rest_length + excess * (-dt / tau).exp();
    }
}

/// Elastic potential energy: 0.5 * k * strain²  * rest_length
pub fn tendon_energy(t: &Tendon) -> f32 {
    let strain = tendon_strain(t);
    0.5 * t.stiffness * strain * strain * t.rest_length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new tendon starts at rest length */
        let t = new_tendon(1000.0, 10.0, 0.1);
        assert!((t.current_length - t.rest_length).abs() < 1e-7);
    }

    #[test]
    fn test_strain_at_rest() {
        /* strain is zero at rest length */
        let t = new_tendon(1000.0, 10.0, 0.1);
        assert!(tendon_strain(&t).abs() < 1e-7);
    }

    #[test]
    fn test_elongate() {
        /* elongation increases current length */
        let mut t = new_tendon(1000.0, 10.0, 0.1);
        tendon_elongate(&mut t, 0.01);
        assert!((t.current_length - 0.11).abs() < 1e-7);
    }

    #[test]
    fn test_force_elastic() {
        /* elastic component of force */
        let mut t = new_tendon(1000.0, 10.0, 0.1);
        tendon_elongate(&mut t, 0.01); // strain = 0.1
        let f = tendon_force(&t, 0.0);
        assert!((f - 100.0).abs() < 1e-3);
    }

    #[test]
    fn test_relax() {
        /* relaxation reduces strain */
        let mut t = new_tendon(100.0, 10.0, 0.1);
        tendon_elongate(&mut t, 0.05);
        tendon_relax(&mut t, 0.1);
        assert!(tendon_strain(&t) < 0.5);
    }

    #[test]
    fn test_energy_zero_at_rest() {
        /* energy is zero at rest */
        let t = new_tendon(1000.0, 10.0, 0.1);
        assert!(tendon_energy(&t) < 1e-10);
    }

    #[test]
    fn test_energy_positive_when_stretched() {
        /* energy positive when stretched */
        let mut t = new_tendon(1000.0, 10.0, 0.1);
        tendon_elongate(&mut t, 0.01);
        assert!(tendon_energy(&t) > 0.0);
    }
}
