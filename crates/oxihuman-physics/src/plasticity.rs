// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Elastoplastic material with yield surface (von Mises).

#![allow(dead_code)]

/// Cauchy stress tensor (Voigt notation: [s11, s22, s33, s12, s13, s23]).
pub type StressTensor = [f64; 6];

/// Elastoplastic material state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlasticState {
    pub stress: StressTensor,
    pub plastic_strain: StressTensor,
    pub accumulated_plastic_strain: f64,
    pub yield_stress: f64,
    pub hardening_modulus: f64,
}

impl PlasticState {
    #[allow(dead_code)]
    pub fn new(yield_stress: f64, hardening_modulus: f64) -> Self {
        Self {
            stress: [0.0; 6],
            plastic_strain: [0.0; 6],
            accumulated_plastic_strain: 0.0,
            yield_stress,
            hardening_modulus,
        }
    }

    /// Current yield stress with isotropic hardening.
    #[allow(dead_code)]
    pub fn current_yield_stress(&self) -> f64 {
        self.yield_stress + self.hardening_modulus * self.accumulated_plastic_strain
    }

    /// Von Mises equivalent stress.
    #[allow(dead_code)]
    pub fn von_mises(&self) -> f64 {
        von_mises_stress(&self.stress)
    }

    /// Check if material has yielded.
    #[allow(dead_code)]
    pub fn has_yielded(&self) -> bool {
        self.von_mises() >= self.current_yield_stress()
    }

    /// Return mapping (radial return) for 3D isotropic hardening.
    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn return_mapping(&mut self, trial_stress: &StressTensor, elastic_modulus: f64) {
        let s_dev = deviatoric(trial_stress);
        let s_eq = von_mises_from_deviatoric(&s_dev);
        let sigma_y = self.current_yield_stress();

        if s_eq <= sigma_y {
            self.stress = *trial_stress;
            return;
        }

        let d_gamma = (s_eq - sigma_y) / (3.0 * elastic_modulus + self.hardening_modulus);
        let scale = 1.0 - 3.0 * elastic_modulus * d_gamma / s_eq;
        for i in 0..3 {
            self.stress[i] = hydrostatic(trial_stress) + scale * s_dev[i];
            self.plastic_strain[i] += d_gamma * s_dev[i] / s_eq;
        }
        for i in 3..6 {
            self.stress[i] = scale * s_dev[i];
            self.plastic_strain[i] += d_gamma * s_dev[i] / s_eq;
        }
        self.accumulated_plastic_strain += d_gamma * (2.0 / 3.0_f64).sqrt();
    }
}

/// Hydrostatic stress (mean normal stress).
#[allow(dead_code)]
pub fn hydrostatic(s: &StressTensor) -> f64 {
    (s[0] + s[1] + s[2]) / 3.0
}

/// Deviatoric stress.
#[allow(dead_code)]
pub fn deviatoric(s: &StressTensor) -> StressTensor {
    let p = hydrostatic(s);
    [s[0] - p, s[1] - p, s[2] - p, s[3], s[4], s[5]]
}

/// Von Mises equivalent stress.
#[allow(dead_code)]
pub fn von_mises_stress(s: &StressTensor) -> f64 {
    let dev = deviatoric(s);
    von_mises_from_deviatoric(&dev)
}

fn von_mises_from_deviatoric(dev: &StressTensor) -> f64 {
    let j2 = 0.5 * (dev[0] * dev[0] + dev[1] * dev[1] + dev[2] * dev[2])
        + dev[3] * dev[3] + dev[4] * dev[4] + dev[5] * dev[5];
    (3.0 * j2).sqrt()
}

/// Build uniaxial stress state.
#[allow(dead_code)]
pub fn uniaxial_stress(s11: f64) -> StressTensor {
    [s11, 0.0, 0.0, 0.0, 0.0, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_von_mises_uniaxial() {
        let s = uniaxial_stress(100.0);
        let vm = von_mises_stress(&s);
        assert!((vm - 100.0).abs() < 1e-6, "vm={vm}");
    }

    #[test]
    fn test_hydrostatic() {
        let s = [3.0, 6.0, 9.0, 0.0, 0.0, 0.0];
        let p = hydrostatic(&s);
        assert!((p - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_deviatoric_trace_zero() {
        let s = [1.0, 2.0, 3.0, 0.0, 0.0, 0.0];
        let dev = deviatoric(&s);
        assert!((dev[0] + dev[1] + dev[2]).abs() < 1e-9);
    }

    #[test]
    fn test_no_yield_below_threshold() {
        let state = PlasticState::new(200.0, 1000.0);
        assert!(!state.has_yielded());
    }

    #[test]
    fn test_yield_above_threshold() {
        let mut state = PlasticState::new(100.0, 0.0);
        state.stress = uniaxial_stress(150.0);
        assert!(state.has_yielded());
    }

    #[test]
    fn test_return_mapping_elastic() {
        let mut state = PlasticState::new(200.0, 0.0);
        let trial = uniaxial_stress(100.0);
        state.return_mapping(&trial, 200e9);
        assert!((state.stress[0] - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_return_mapping_plastic() {
        let mut state = PlasticState::new(100.0, 0.0);
        let trial = uniaxial_stress(200.0);
        state.return_mapping(&trial, 200.0);
        assert!(state.accumulated_plastic_strain > 0.0);
    }

    #[test]
    fn test_hardening_increases_yield() {
        let mut state = PlasticState::new(100.0, 50.0);
        state.accumulated_plastic_strain = 2.0;
        assert!((state.current_yield_stress() - 200.0).abs() < 1e-9);
    }

    #[test]
    fn test_von_mises_pure_shear() {
        let s: StressTensor = [0.0, 0.0, 0.0, 100.0, 0.0, 0.0];
        let vm = von_mises_stress(&s);
        assert!((vm - 100.0 * 3.0_f64.sqrt()).abs() < 1e-6, "vm={vm}");
    }

    #[test]
    fn test_new_state_zero_plastic_strain() {
        let state = PlasticState::new(250.0, 100.0);
        assert!((state.accumulated_plastic_strain).abs() < 1e-12);
    }
}
