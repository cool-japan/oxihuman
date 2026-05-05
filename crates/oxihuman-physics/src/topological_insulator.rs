// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Topological surface state physics stub.
//! Models topological insulator surface Dirac cone physics conceptually.

use std::f32::consts::PI;

/// Material parameters for a topological insulator surface state.
#[derive(Debug, Clone)]
pub struct TopoInsulatorConfig {
    pub fermi_velocity: f32,         /* v_F [m/s] */
    pub bulk_gap: f32,               /* E_g [eV] */
    pub surface_state_lifetime: f32, /* tau [s] */
    pub dirac_point_energy: f32,     /* E_D [eV] */
}

impl TopoInsulatorConfig {
    pub fn new(
        fermi_velocity: f32,
        bulk_gap: f32,
        surface_state_lifetime: f32,
        dirac_point_energy: f32,
    ) -> Self {
        TopoInsulatorConfig {
            fermi_velocity,
            bulk_gap,
            surface_state_lifetime,
            dirac_point_energy,
        }
    }

    pub fn bi2te3() -> Self {
        /* Bi2Te3 approximate values */
        TopoInsulatorConfig::new(4.0e5, 0.17, 1e-12, 0.0)
    }

    pub fn bi2se3() -> Self {
        /* Bi2Se3 approximate values */
        TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0)
    }
}

impl Default for TopoInsulatorConfig {
    fn default() -> Self {
        Self::bi2se3()
    }
}

/// Dirac cone energy dispersion: E(k) = hbar * v_F * |k|.
pub fn surface_dispersion(config: &TopoInsulatorConfig, k_magnitude: f32) -> f32 {
    /* Using hbar in eV*s: 6.582e-16 eV*s */
    let hbar_ev = 6.582e-16f32;
    hbar_ev * config.fermi_velocity * k_magnitude.abs()
}

/// Spin-momentum locking angle (helical spin texture): phi_spin = phi_k + pi/2.
pub fn spin_angle(k_angle: f32) -> f32 {
    k_angle + PI / 2.0
}

/// Topological Z2 invariant (stub: returns 1 for non-trivial topology).
pub fn z2_invariant(_config: &TopoInsulatorConfig) -> i32 {
    1 /* non-trivial by definition for this stub */
}

/// Surface state density of states at energy E.
pub fn surface_dos(config: &TopoInsulatorConfig, energy: f32) -> f32 {
    /* For 2D Dirac: DOS = |E| / (pi * hbar^2 * v_F^2) */
    let hbar_ev = 6.582e-16f32;
    let vf = config.fermi_velocity;
    let e = (energy - config.dirac_point_energy).abs();
    e / (PI * hbar_ev * hbar_ev * vf * vf)
}

/// Mean free path of surface Dirac electrons.
pub fn mean_free_path(config: &TopoInsulatorConfig) -> f32 {
    config.fermi_velocity * config.surface_state_lifetime
}

/// Fermi wavevector at given chemical potential (measured from Dirac point).
pub fn fermi_wavevector(config: &TopoInsulatorConfig, chemical_potential: f32) -> f32 {
    let hbar_ev = 6.582e-16f32;
    if config.fermi_velocity <= 0.0 || hbar_ev <= 0.0 {
        return 0.0;
    }
    chemical_potential.abs() / (hbar_ev * config.fermi_velocity)
}

/// Check if energy is within the bulk gap (surface state relevant).
pub fn is_in_bulk_gap(config: &TopoInsulatorConfig, energy: f32) -> bool {
    let half_gap = config.bulk_gap / 2.0;
    let e_rel = energy - config.dirac_point_energy;
    e_rel.abs() < half_gap
}

/// Quantum anomalous Hall conductance (sigma_xy in units of e^2/h) — stub.
pub fn anomalous_hall_conductance_stub(_config: &TopoInsulatorConfig) -> f32 {
    /* For a magnetically gapped TI surface: sigma_xy = e^2/(2h) = 0.5 in natural units */
    0.5
}

/// Chern number (topological invariant) — stub returns 1.
pub fn chern_number(_config: &TopoInsulatorConfig) -> i32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_dispersion_zero_k() {
        let c = TopoInsulatorConfig::default();
        assert_eq!(surface_dispersion(&c, 0.0), 0.0);
    }

    #[test]
    fn test_surface_dispersion_positive_k() {
        let c = TopoInsulatorConfig::default();
        assert!(surface_dispersion(&c, 1e9) > 0.0);
    }

    #[test]
    fn test_spin_angle_offset() {
        let k_angle = 0.0f32;
        assert!((spin_angle(k_angle) - PI / 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_z2_invariant_nontrivial() {
        let c = TopoInsulatorConfig::default();
        assert_eq!(z2_invariant(&c), 1);
    }

    #[test]
    fn test_surface_dos_positive() {
        let c = TopoInsulatorConfig::default();
        assert!(surface_dos(&c, 0.1) > 0.0);
    }

    #[test]
    fn test_surface_dos_at_dirac_point_zero() {
        let c = TopoInsulatorConfig::default();
        /* At Dirac point, DOS = 0 for linear dispersion */
        let dos = surface_dos(&c, c.dirac_point_energy);
        assert_eq!(dos, 0.0);
    }

    #[test]
    fn test_mean_free_path_positive() {
        let c = TopoInsulatorConfig::default();
        assert!(mean_free_path(&c) > 0.0);
    }

    #[test]
    fn test_fermi_wavevector_positive() {
        let c = TopoInsulatorConfig::default();
        assert!(fermi_wavevector(&c, 0.1) > 0.0);
    }

    #[test]
    fn test_is_in_bulk_gap() {
        let c = TopoInsulatorConfig::bi2se3();
        /* energy within gap */
        assert!(is_in_bulk_gap(&c, 0.05));
        /* energy outside gap */
        assert!(!is_in_bulk_gap(&c, 0.5));
    }

    #[test]
    fn test_anomalous_hall_conductance() {
        let c = TopoInsulatorConfig::default();
        assert!((anomalous_hall_conductance_stub(&c) - 0.5).abs() < 0.001);
    }
}
