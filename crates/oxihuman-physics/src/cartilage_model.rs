// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cartilage compression model (biphasic poroelastic stub).

/// A cartilage layer element.
#[derive(Debug, Clone)]
pub struct Cartilage {
    /// Thickness at rest (m).
    pub thickness: f32,
    /// Current compressed thickness (m).
    pub current_thickness: f32,
    /// Aggregate modulus (Pa).
    pub modulus: f32,
    /// Permeability (m^4/N·s) for fluid flow.
    pub permeability: f32,
    /// Fluid pressure (Pa).
    pub fluid_pressure: f32,
    /// Damage level [0, 1]; 0 = healthy.
    pub damage: f32,
}

impl Cartilage {
    pub fn new(thickness: f32, modulus: f32) -> Self {
        Cartilage {
            thickness,
            current_thickness: thickness,
            modulus,
            permeability: 1e-15,
            fluid_pressure: 0.0,
            damage: 0.0,
        }
    }
}

/// Create a new cartilage element.
pub fn new_cartilage(thickness: f32, modulus: f32) -> Cartilage {
    Cartilage::new(thickness, modulus)
}

/// Apply a compressive load `load` (N/m²) and advance by `dt`.
pub fn cartilage_step(c: &mut Cartilage, load: f32, dt: f32) {
    let eff_mod = c.modulus * (1.0 - c.damage);
    let strain = load / eff_mod.max(1e-3);
    let target = c.thickness * (1.0 - strain.clamp(0.0, 0.9));
    /* creep toward target */
    c.current_thickness += (target - c.current_thickness) * (dt / 0.1).min(1.0);
    c.current_thickness = c.current_thickness.max(c.thickness * 0.1);
    /* fluid pressure rises under compression */
    c.fluid_pressure = load * (1.0 - c.permeability.min(1.0));
    /* accumulate damage under high load */
    if load > eff_mod * 0.5 {
        c.damage = (c.damage + 0.0001 * dt).min(1.0);
    }
}

/// Return the compressive strain.
pub fn cartilage_strain(c: &Cartilage) -> f32 {
    (c.thickness - c.current_thickness) / c.thickness.max(1e-10)
}

/// Return `true` if the cartilage is compressed.
pub fn cartilage_is_compressed(c: &Cartilage) -> bool {
    c.current_thickness < c.thickness - 1e-6
}

/// Return the contact stress (Pa) given compressive load.
pub fn cartilage_contact_stress(c: &Cartilage, load: f32) -> f32 {
    /* effective stress = total load - fluid pressure share */
    (load - c.fluid_pressure).max(0.0)
}

/// Reset cartilage to unloaded state.
pub fn cartilage_reset(c: &mut Cartilage) {
    c.current_thickness = c.thickness;
    c.fluid_pressure = 0.0;
}

/// Return remaining functional modulus (reduced by damage).
pub fn cartilage_effective_modulus(c: &Cartilage) -> f32 {
    c.modulus * (1.0 - c.damage)
}

/// Apply repair: reduce damage by `amount`.
pub fn cartilage_repair(c: &mut Cartilage, amount: f32) {
    c.damage = (c.damage - amount.abs()).max(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cartilage_uncompressed() {
        let c = new_cartilage(0.004, 1_000_000.0);
        assert!(!cartilage_is_compressed(&c));
    }

    #[test]
    fn test_step_compresses_under_load() {
        let mut c = new_cartilage(0.004, 1_000_000.0);
        cartilage_step(&mut c, 500_000.0, 0.1);
        assert!(cartilage_is_compressed(&c));
    }

    #[test]
    fn test_strain_positive_under_load() {
        let mut c = new_cartilage(0.004, 1_000_000.0);
        cartilage_step(&mut c, 500_000.0, 0.1);
        assert!(cartilage_strain(&c) > 0.0);
    }

    #[test]
    fn test_reset_restores_thickness() {
        let mut c = new_cartilage(0.004, 1_000_000.0);
        cartilage_step(&mut c, 500_000.0, 0.5);
        cartilage_reset(&mut c);
        assert!(!cartilage_is_compressed(&c));
    }

    #[test]
    fn test_effective_modulus_decreases_with_damage() {
        let mut c = new_cartilage(0.004, 1_000_000.0);
        c.damage = 0.3;
        let eff = cartilage_effective_modulus(&c);
        assert!(eff < c.modulus);
    }

    #[test]
    fn test_repair_reduces_damage() {
        let mut c = new_cartilage(0.004, 1_000_000.0);
        c.damage = 0.5;
        cartilage_repair(&mut c, 0.2);
        assert!((c.damage - 0.3).abs() < 1e-4);
    }

    #[test]
    fn test_fluid_pressure_rises_under_load() {
        let mut c = new_cartilage(0.004, 1_000_000.0);
        cartilage_step(&mut c, 200_000.0, 0.05);
        assert!(c.fluid_pressure > 0.0);
    }

    #[test]
    fn test_contact_stress_nonnegative() {
        let mut c = new_cartilage(0.004, 1_000_000.0);
        cartilage_step(&mut c, 300_000.0, 0.1);
        assert!(cartilage_contact_stress(&c, 300_000.0) >= 0.0);
    }

    #[test]
    fn test_damage_does_not_exceed_one() {
        let mut c = new_cartilage(0.004, 100_000.0);
        for _ in 0..10000 {
            cartilage_step(&mut c, 2_000_000.0, 0.1);
        }
        assert!(c.damage <= 1.0);
    }
}
