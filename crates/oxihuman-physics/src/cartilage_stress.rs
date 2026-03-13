// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Cartilage biphasic layer (simplified).
pub struct CartilageLayer {
    pub thickness: f32,
    pub modulus: f32,
    pub permeability: f32,
    pub fluid_fraction: f32,
    pub strain: f32,
}

impl CartilageLayer {
    pub fn new(thickness: f32) -> Self {
        CartilageLayer {
            thickness,
            modulus: 1.0e6, // 1 MPa
            permeability: 1e-15,
            fluid_fraction: 0.75,
            strain: 0.0,
        }
    }
}

pub fn new_cartilage_layer(thickness: f32) -> CartilageLayer {
    CartilageLayer::new(thickness)
}

/// Total stress = E * strain
pub fn cartilage_total_stress(c: &CartilageLayer, strain: f32) -> f32 {
    c.modulus * strain
}

/// Fluid pressure contribution proportional to strain rate and permeability.
pub fn cartilage_fluid_pressure(c: &CartilageLayer, strain_rate: f32) -> f32 {
    c.fluid_fraction * strain_rate / c.permeability * c.thickness * 1e-10
}

/// Apply a compressive load; update strain.
pub fn cartilage_apply_load(c: &mut CartilageLayer, load: f32, dt: f32) {
    let stress_rate = load / c.thickness;
    c.strain += (stress_rate / c.modulus) * dt;
    c.strain = c.strain.clamp(0.0, 0.5);
}

/// Recovery: strain decays exponentially toward zero.
pub fn cartilage_recovery(c: &mut CartilageLayer, dt: f32) {
    let tau = 1.0; // relaxation time (s)
    c.strain *= (-dt / tau).exp();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new layer has correct defaults */
        let c = new_cartilage_layer(2e-3);
        assert!((c.thickness - 2e-3).abs() < 1e-6);
        assert!((c.fluid_fraction - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_total_stress() {
        /* total stress = E * strain */
        let c = new_cartilage_layer(2e-3);
        let stress = cartilage_total_stress(&c, 0.1);
        assert!((stress - 1e5).abs() < 1.0);
    }

    #[test]
    fn test_apply_load_increases_strain() {
        /* applying a load increases strain */
        let mut c = new_cartilage_layer(2e-3);
        cartilage_apply_load(&mut c, 1000.0, 0.1);
        assert!(c.strain > 0.0);
    }

    #[test]
    fn test_strain_clamp() {
        /* strain is clamped at 0.5 */
        let mut c = new_cartilage_layer(1e-4);
        for _ in 0..1000 {
            cartilage_apply_load(&mut c, 1e8, 0.1);
        }
        assert!(c.strain <= 0.5 + 1e-6);
    }

    #[test]
    fn test_recovery() {
        /* recovery reduces strain */
        let mut c = new_cartilage_layer(2e-3);
        c.strain = 0.2;
        cartilage_recovery(&mut c, 1.0);
        assert!(c.strain < 0.2);
    }

    #[test]
    fn test_fluid_pressure_positive() {
        /* fluid pressure is positive for positive strain rate */
        let c = new_cartilage_layer(2e-3);
        let p = cartilage_fluid_pressure(&c, 1.0);
        assert!(p >= 0.0);
    }

    #[test]
    fn test_initial_strain_zero() {
        /* initial strain is zero */
        let c = new_cartilage_layer(3e-3);
        assert!((c.strain - 0.0).abs() < 1e-9);
    }
}
