// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Plastic deformation with yield surface stub.
//!
//! Implements a J2 (von Mises) plasticity model with isotropic hardening
//! using a radial return mapping algorithm.

/// Elastic-plastic material parameters.
#[derive(Debug, Clone)]
pub struct PlasticityParams {
    /// Young's modulus `[GPa]`.
    pub young_modulus: f64,
    /// Poisson ratio.
    pub poisson_ratio: f64,
    /// Initial yield stress `[MPa]`.
    pub yield_stress_0: f64,
    /// Isotropic hardening modulus H `[MPa]`.
    pub hardening_modulus: f64,
}

impl Default for PlasticityParams {
    fn default() -> Self {
        Self {
            young_modulus: 200.0,
            poisson_ratio: 0.3,
            yield_stress_0: 250.0,
            hardening_modulus: 2000.0,
        }
    }
}

/// Internal state of a plastic material point.
#[derive(Debug, Clone, Default)]
pub struct PlasticState {
    /// Accumulated equivalent plastic strain.
    pub equiv_plastic_strain: f64,
    /// Back-stress for kinematic hardening (currently unused — stub).
    pub back_stress: f64,
    /// Current yield stress.
    pub yield_stress: f64,
}

impl PlasticState {
    pub fn new(initial_yield: f64) -> Self {
        Self {
            equiv_plastic_strain: 0.0,
            back_stress: 0.0,
            yield_stress: initial_yield,
        }
    }

    pub fn is_yielded(&self) -> bool {
        self.equiv_plastic_strain > 0.0
    }
}

/// Compute von Mises equivalent stress from a stress tensor (Voigt 6-vector: `[σxx,σyy,σzz,σxy,σyz,σxz]`).
pub fn von_mises_stress(s: &[f64; 6]) -> f64 {
    let dev_xx = s[0] - (s[0] + s[1] + s[2]) / 3.0;
    let dev_yy = s[1] - (s[0] + s[1] + s[2]) / 3.0;
    let dev_zz = s[2] - (s[0] + s[1] + s[2]) / 3.0;
    let j2 = 0.5 * (dev_xx * dev_xx + dev_yy * dev_yy + dev_zz * dev_zz)
        + s[3] * s[3]
        + s[4] * s[4]
        + s[5] * s[5];
    (3.0 * j2).sqrt()
}

/// Compute the current yield stress with isotropic hardening.
pub fn current_yield_stress(state: &PlasticState, params: &PlasticityParams) -> f64 {
    params.yield_stress_0 + params.hardening_modulus * state.equiv_plastic_strain
}

/// Evaluate the yield function f = σ_eq - σ_y.
pub fn yield_function(sigma_eq: f64, state: &PlasticState, params: &PlasticityParams) -> f64 {
    sigma_eq - current_yield_stress(state, params)
}

/// Perform radial return mapping for J2 plasticity (1D simplification).
///
/// Returns the corrected stress and updates the plastic state.
pub fn radial_return(
    trial_stress: f64,
    state: &mut PlasticState,
    params: &PlasticityParams,
) -> f64 {
    let e = params.young_modulus * 1e3; /* GPa → MPa */
    let sigma_y = current_yield_stress(state, params);
    let f = trial_stress.abs() - sigma_y;

    if f <= 0.0 {
        return trial_stress; /* Elastic */
    }

    /* Plastic correction */
    let d_gamma = f / (e + params.hardening_modulus);
    let sign = if trial_stress >= 0.0 { 1.0 } else { -1.0 };
    let corrected = trial_stress - sign * e * d_gamma;
    state.equiv_plastic_strain += d_gamma;
    state.yield_stress = current_yield_stress(state, params);
    corrected
}

/// Check if a stress state causes yielding.
pub fn is_yielding(sigma_eq: f64, state: &PlasticState, params: &PlasticityParams) -> bool {
    yield_function(sigma_eq, state, params) > 0.0
}

/// Compute the plastic strain increment for a given stress excess.
pub fn plastic_strain_increment(stress_excess: f64, params: &PlasticityParams) -> f64 {
    let e = params.young_modulus * 1e3;
    stress_excess / (e + params.hardening_modulus)
}

/// Compute shear modulus G.
pub fn shear_modulus(params: &PlasticityParams) -> f64 {
    let e = params.young_modulus * 1e3;
    e / (2.0 * (1.0 + params.poisson_ratio))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> PlasticityParams {
        PlasticityParams::default()
    }

    #[test]
    fn test_von_mises_uniaxial() {
        let s = [100.0f64, 0.0, 0.0, 0.0, 0.0, 0.0];
        let vm = von_mises_stress(&s);
        assert!((vm - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_von_mises_pure_shear() {
        let s = [0.0f64, 0.0, 0.0, 100.0, 0.0, 0.0];
        let vm = von_mises_stress(&s);
        assert!((vm - 100.0 * 3.0_f64.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn test_yield_function_negative_below_yield() {
        let p = default_params();
        let state = PlasticState::new(p.yield_stress_0);
        assert!(yield_function(100.0, &state, &p) < 0.0);
    }

    #[test]
    fn test_yield_function_positive_above_yield() {
        let p = default_params();
        let state = PlasticState::new(p.yield_stress_0);
        assert!(yield_function(300.0, &state, &p) > 0.0);
    }

    #[test]
    fn test_radial_return_elastic() {
        let p = default_params();
        let mut state = PlasticState::new(p.yield_stress_0);
        let corrected = radial_return(100.0, &mut state, &p);
        assert_eq!(corrected, 100.0);
        assert!(!state.is_yielded());
    }

    #[test]
    fn test_radial_return_plastic() {
        let p = default_params();
        let mut state = PlasticState::new(p.yield_stress_0);
        let corrected = radial_return(400.0, &mut state, &p);
        assert!(corrected < 400.0);
        assert!(state.is_yielded());
    }

    #[test]
    fn test_shear_modulus_positive() {
        let p = default_params();
        assert!(shear_modulus(&p) > 0.0);
    }

    #[test]
    fn test_plastic_strain_increment() {
        let p = default_params();
        let inc = plastic_strain_increment(50.0, &p);
        assert!(inc > 0.0);
    }

    #[test]
    fn test_current_yield_hardening() {
        let p = default_params();
        let mut state = PlasticState::new(p.yield_stress_0);
        state.equiv_plastic_strain = 0.01;
        let sy = current_yield_stress(&state, &p);
        assert!(sy > p.yield_stress_0);
    }
}
