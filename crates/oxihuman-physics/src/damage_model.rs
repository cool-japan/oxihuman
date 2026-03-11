// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Continuum damage mechanics (isotropic scalar damage).

#![allow(dead_code)]

/// Isotropic scalar damage state variable.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DamageState {
    /// Damage variable D in [0, 1]. D=0: undamaged, D=1: fully damaged.
    pub damage: f64,
    /// History variable (max equivalent strain reached).
    pub kappa: f64,
    /// Damage threshold (initial).
    pub kappa_0: f64,
    /// Softening parameter.
    pub alpha: f64,
    /// Residual stiffness ratio at full damage.
    pub beta: f64,
}

impl DamageState {
    #[allow(dead_code)]
    pub fn new(kappa_0: f64, alpha: f64, beta: f64) -> Self {
        Self {
            damage: 0.0,
            kappa: kappa_0,
            kappa_0,
            alpha,
            beta,
        }
    }

    /// Equivalent strain from principal strains (Mazars criterion).
    #[allow(dead_code)]
    pub fn equivalent_strain(principal_strains: &[f64; 3]) -> f64 {
        principal_strains.iter().map(|&e| e.max(0.0).powi(2)).sum::<f64>().sqrt()
    }

    /// Update damage state given equivalent strain.
    #[allow(dead_code)]
    pub fn update(&mut self, eps_eq: f64) {
        if eps_eq > self.kappa {
            self.kappa = eps_eq;
            self.damage = self.damage_function(self.kappa);
        }
    }

    /// Exponential softening damage function.
    fn damage_function(&self, kappa: f64) -> f64 {
        if kappa <= self.kappa_0 {
            return 0.0;
        }
        let d = 1.0
            - (self.kappa_0 / kappa)
                * (1.0 - self.alpha + self.alpha * (-(kappa - self.kappa_0) / self.beta).exp());
        d.clamp(0.0, 1.0)
    }

    /// Effective stiffness factor (1 - D).
    #[allow(dead_code)]
    pub fn stiffness_factor(&self) -> f64 {
        1.0 - self.damage
    }

    /// Check if fully damaged.
    #[allow(dead_code)]
    pub fn is_failed(&self) -> bool {
        self.damage >= 1.0 - 1e-6
    }

    /// Apply damage to stress (effective stress = (1-D) * nominal stress).
    #[allow(dead_code)]
    pub fn apply_to_stress(&self, nominal_stress: f64) -> f64 {
        self.stiffness_factor() * nominal_stress
    }

    /// Energy dissipated (proportional to damage).
    #[allow(dead_code)]
    pub fn dissipated_energy(&self, fracture_energy: f64) -> f64 {
        self.damage * fracture_energy
    }
}

/// Multi-point damage body with per-node damage states.
#[allow(dead_code)]
pub struct DamageBody {
    pub states: Vec<DamageState>,
    pub kappa_0: f64,
    pub alpha: f64,
    pub beta: f64,
}

impl DamageBody {
    #[allow(dead_code)]
    pub fn new(n: usize, kappa_0: f64, alpha: f64, beta: f64) -> Self {
        Self {
            states: (0..n).map(|_| DamageState::new(kappa_0, alpha, beta)).collect(),
            kappa_0,
            alpha,
            beta,
        }
    }

    #[allow(dead_code)]
    pub fn update_node(&mut self, idx: usize, eps_eq: f64) {
        if let Some(s) = self.states.get_mut(idx) {
            s.update(eps_eq);
        }
    }

    #[allow(dead_code)]
    pub fn max_damage(&self) -> f64 {
        self.states.iter().map(|s| s.damage).fold(0.0f64, f64::max)
    }

    #[allow(dead_code)]
    pub fn failed_count(&self) -> usize {
        self.states.iter().filter(|s| s.is_failed()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_damage_below_threshold() {
        let mut state = DamageState::new(0.001, 0.99, 0.0005);
        state.update(0.0005);
        assert!((state.damage).abs() < 1e-9);
    }

    #[test]
    fn test_damage_increases_above_threshold() {
        let mut state = DamageState::new(0.001, 0.99, 0.0005);
        state.update(0.005);
        assert!(state.damage > 0.0);
    }

    #[test]
    fn test_damage_clamped_to_one() {
        let mut state = DamageState::new(0.0001, 0.99, 0.0001);
        state.update(1.0);
        assert!(state.damage <= 1.0);
    }

    #[test]
    fn test_is_failed() {
        let mut state = DamageState::new(0.0001, 0.999, 0.0001);
        state.update(10.0);
        assert!(state.is_failed());
    }

    #[test]
    fn test_apply_to_stress() {
        let mut state = DamageState::new(0.001, 0.99, 0.0005);
        state.damage = 0.5;
        let eff = state.apply_to_stress(100.0);
        assert!((eff - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_equivalent_strain() {
        let eps = [0.01, -0.005, 0.0];
        let eq = DamageState::equivalent_strain(&eps);
        assert!((eq - 0.01_f64).abs() < 1e-9);
    }

    #[test]
    fn test_kappa_monotone_increasing() {
        let mut state = DamageState::new(0.001, 0.99, 0.001);
        state.update(0.01);
        let k1 = state.kappa;
        state.update(0.005);
        assert_eq!(state.kappa, k1);
    }

    #[test]
    fn test_damage_body_max_damage() {
        let mut body = DamageBody::new(3, 0.001, 0.99, 0.0005);
        body.update_node(1, 0.01);
        assert!(body.max_damage() > 0.0);
    }

    #[test]
    fn test_damage_body_failed_count() {
        let mut body = DamageBody::new(3, 0.0001, 0.999, 0.0001);
        body.update_node(0, 10.0);
        body.update_node(1, 10.0);
        assert_eq!(body.failed_count(), 2);
    }

    #[test]
    fn test_stiffness_factor() {
        let mut state = DamageState::new(0.001, 0.99, 0.001);
        state.damage = 0.3;
        assert!((state.stiffness_factor() - 0.7).abs() < 1e-9);
    }
}
