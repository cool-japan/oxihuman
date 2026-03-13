// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Crushable foam material model stub.
//!
//! Models the highly nonlinear compressive response of cellular foams using
//! a three-region stress-strain curve: linear elastic, plateau (crushing),
//! and densification.

/// Foam material parameters.
#[derive(Debug, Clone)]
pub struct FoamParams {
    /// Initial Young's modulus (linear region) `[MPa]`.
    pub young_modulus: f64,
    /// Plateau stress (crush stress) `[MPa]`.
    pub plateau_stress: f64,
    /// Densification strain (onset of densification).
    pub densification_strain: f64,
    /// Relative density ρ/ρ_s.
    pub relative_density: f64,
    /// Solid material Young's modulus `[MPa]`.
    pub solid_modulus: f64,
}

impl Default for FoamParams {
    fn default() -> Self {
        Self {
            young_modulus: 5.0,
            plateau_stress: 0.5,
            densification_strain: 0.7,
            relative_density: 0.05,
            solid_modulus: 3000.0,
        }
    }
}

/// State of a foam element.
#[derive(Debug, Clone, Default)]
pub struct FoamState {
    pub strain: f64,
    pub stress: f64,
    pub plastic_strain: f64,
    pub densified: bool,
}

impl FoamState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn energy_absorbed(&self) -> f64 {
        /* Approximate area under stress-strain curve */
        self.stress * self.strain * 0.5
    }
}

/// Compute foam stress from strain using a three-region model.
pub fn foam_stress(strain: f64, params: &FoamParams) -> f64 {
    let eps = strain.clamp(0.0, 1.0);
    let eps_y = params.plateau_stress / params.young_modulus;
    let eps_d = params.densification_strain;

    if eps < eps_y {
        /* Linear elastic */
        params.young_modulus * eps
    } else if eps < eps_d {
        /* Plateau (crushing) */
        params.plateau_stress
    } else {
        /* Densification: rapid stiffening */
        let excess = eps - eps_d;
        params.plateau_stress + params.solid_modulus * excess * excess
    }
}

/// Compute the energy absorption capacity of a foam layer.
pub fn energy_absorption(params: &FoamParams) -> f64 {
    /* Area under the plateau region */
    let eps_y = params.plateau_stress / params.young_modulus;
    let plateau_len = (params.densification_strain - eps_y).max(0.0);
    0.5 * params.young_modulus * eps_y * eps_y + params.plateau_stress * plateau_len
}

/// Update foam state given a new applied strain.
pub fn update_foam_state(state: &mut FoamState, new_strain: f64, params: &FoamParams) {
    state.strain = new_strain.max(state.strain); /* Irreversible crush */
    state.stress = foam_stress(state.strain, params);
    let eps_y = params.plateau_stress / params.young_modulus;
    if state.strain > eps_y {
        state.plastic_strain = state.strain - eps_y;
    }
    state.densified = state.strain >= params.densification_strain;
}

/// Compute the specific energy absorption (per unit mass).
pub fn specific_energy_absorption(energy: f64, density: f64) -> f64 {
    if density <= 0.0 {
        return 0.0;
    }
    energy / density
}

/// Estimate the Gibson-Ashby scaling for plateau stress.
///
/// σ_pl / σ_y_s ≈ C * (ρ/ρ_s)^n  (n≈1.5 for open-cell, 2.0 for closed-cell foam)
pub fn gibson_ashby_plateau(params: &FoamParams, solid_yield: f64, n: f64, c: f64) -> f64 {
    solid_yield * c * params.relative_density.powf(n)
}

/// Compute the Poisson ratio of an open-cell foam.
pub fn open_cell_poisson() -> f64 {
    /* Near 1/3 for most open-cell foams at low relative density */
    1.0 / 3.0
}

/// Check if foam is in its elastic regime.
pub fn is_elastic(strain: f64, params: &FoamParams) -> bool {
    strain < params.plateau_stress / params.young_modulus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> FoamParams {
        FoamParams::default()
    }

    #[test]
    fn test_stress_zero_at_zero_strain() {
        assert_eq!(foam_stress(0.0, &default_params()), 0.0);
    }

    #[test]
    fn test_stress_elastic_region() {
        let p = default_params();
        let eps = 0.01; /* Below yield */
        let sigma = foam_stress(eps, &p);
        assert!((sigma - p.young_modulus * eps).abs() < 1e-9);
    }

    #[test]
    fn test_stress_plateau_region() {
        let p = default_params();
        let sigma = foam_stress(0.3, &p); /* Mid-plateau */
        assert!((sigma - p.plateau_stress).abs() < 1e-9);
    }

    #[test]
    fn test_stress_densification_higher() {
        let p = default_params();
        let s_plateau = foam_stress(0.5, &p);
        let s_dense = foam_stress(0.85, &p);
        assert!(s_dense > s_plateau);
    }

    #[test]
    fn test_energy_absorption_positive() {
        assert!(energy_absorption(&default_params()) > 0.0);
    }

    #[test]
    fn test_update_state_densified() {
        let p = default_params();
        let mut s = FoamState::new();
        update_foam_state(&mut s, p.densification_strain + 0.01, &p);
        assert!(s.densified);
    }

    #[test]
    fn test_gibson_ashby_positive() {
        let p = default_params();
        let v = gibson_ashby_plateau(&p, 300.0, 1.5, 0.3);
        assert!(v > 0.0);
    }

    #[test]
    fn test_is_elastic_true_small_strain() {
        let p = default_params();
        assert!(is_elastic(0.001, &p));
    }

    #[test]
    fn test_specific_energy() {
        let sea = specific_energy_absorption(100.0, 50.0);
        assert!((sea - 2.0).abs() < 1e-9);
    }
}
