// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Ductile solid with plastic flow before fracture.

#![allow(dead_code)]

/// State of a ductile material element.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DuctileState {
    pub stress: f64,
    pub strain: f64,
    pub plastic_strain: f64,
    pub yield_stress: f64,
    pub ultimate_stress: f64,
    pub hardening_rate: f64,
    pub is_fractured: bool,
}

impl DuctileState {
    #[allow(dead_code)]
    pub fn new(yield_stress: f64, ultimate_stress: f64, hardening_rate: f64) -> Self {
        Self {
            stress: 0.0,
            strain: 0.0,
            plastic_strain: 0.0,
            yield_stress,
            ultimate_stress,
            hardening_rate,
            is_fractured: false,
        }
    }

    /// Current flow stress with linear hardening.
    #[allow(dead_code)]
    pub fn flow_stress(&self) -> f64 {
        (self.yield_stress + self.hardening_rate * self.plastic_strain).min(self.ultimate_stress)
    }

    /// Apply incremental strain. Returns stress increment.
    #[allow(dead_code)]
    pub fn apply_strain(&mut self, d_strain: f64, elastic_modulus: f64) {
        if self.is_fractured {
            return;
        }
        let trial_stress = self.stress + elastic_modulus * d_strain;
        let flow = self.flow_stress();
        if trial_stress.abs() <= flow {
            self.stress = trial_stress;
        } else {
            let sign = trial_stress.signum();
            self.stress = sign * flow;
            let d_plastic = (trial_stress.abs() - flow) / elastic_modulus;
            self.plastic_strain += d_plastic;
            self.strain += d_strain;
            if self.flow_stress() >= self.ultimate_stress {
                self.is_fractured = true;
            }
            return;
        }
        self.strain += d_strain;
    }

    #[allow(dead_code)]
    pub fn is_elastic(&self) -> bool {
        self.stress.abs() < self.yield_stress
    }

    #[allow(dead_code)]
    pub fn ductility(&self) -> f64 {
        if self.yield_stress > 0.0 {
            self.plastic_strain / (self.ultimate_stress / self.hardening_rate.max(1e-12))
        } else {
            0.0
        }
    }
}

/// Ductile body with multiple elements.
#[allow(dead_code)]
pub struct DuctileBody {
    pub elements: Vec<DuctileState>,
}

impl DuctileBody {
    #[allow(dead_code)]
    pub fn new(n: usize, yield_stress: f64, ultimate_stress: f64, hardening_rate: f64) -> Self {
        Self {
            elements: (0..n)
                .map(|_| DuctileState::new(yield_stress, ultimate_stress, hardening_rate))
                .collect(),
        }
    }

    #[allow(dead_code)]
    pub fn apply_strain_uniform(&mut self, d_strain: f64, elastic_modulus: f64) {
        for elem in &mut self.elements {
            elem.apply_strain(d_strain, elastic_modulus);
        }
    }

    #[allow(dead_code)]
    pub fn fractured_count(&self) -> usize {
        self.elements.iter().filter(|e| e.is_fractured).count()
    }

    #[allow(dead_code)]
    pub fn avg_plastic_strain(&self) -> f64 {
        if self.elements.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.elements.iter().map(|e| e.plastic_strain).sum();
        sum / self.elements.len() as f64
    }

    #[allow(dead_code)]
    pub fn max_stress(&self) -> f64 {
        self.elements.iter().map(|e| e.stress.abs()).fold(0.0_f64, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elastic_below_yield() {
        let mut state = DuctileState::new(100.0, 500.0, 200.0);
        state.apply_strain(0.0004, 200e3);
        assert!(state.is_elastic());
    }

    #[test]
    fn test_plastic_above_yield() {
        let mut state = DuctileState::new(100.0, 500.0, 100.0);
        state.apply_strain(0.01, 100.0);
        assert!(state.plastic_strain > 0.0);
    }

    #[test]
    fn test_flow_stress_increases_with_hardening() {
        let mut state = DuctileState::new(100.0, 500.0, 200.0);
        state.plastic_strain = 1.0;
        assert!(state.flow_stress() > 100.0);
    }

    #[test]
    fn test_flow_stress_capped_at_ultimate() {
        let mut state = DuctileState::new(100.0, 300.0, 10000.0);
        state.plastic_strain = 100.0;
        assert!(state.flow_stress() <= 300.0);
    }

    #[test]
    fn test_fracture_after_ultimate() {
        let mut state = DuctileState::new(100.0, 105.0, 1000.0);
        for _ in 0..100 {
            state.apply_strain(0.01, 100.0);
        }
        assert!(state.is_fractured);
    }

    #[test]
    fn test_no_change_after_fracture() {
        let mut state = DuctileState::new(100.0, 105.0, 1000.0);
        for _ in 0..100 {
            state.apply_strain(0.01, 100.0);
        }
        let ps = state.plastic_strain;
        state.apply_strain(1.0, 100.0);
        assert_eq!(state.plastic_strain, ps);
    }

    #[test]
    fn test_body_fractured_count() {
        let mut body = DuctileBody::new(5, 100.0, 105.0, 1000.0);
        for _ in 0..100 {
            body.apply_strain_uniform(0.01, 100.0);
        }
        assert!(body.fractured_count() > 0);
    }

    #[test]
    fn test_body_avg_plastic_strain() {
        let mut body = DuctileBody::new(3, 100.0, 500.0, 100.0);
        body.apply_strain_uniform(0.1, 100.0);
        assert!(body.avg_plastic_strain() >= 0.0);
    }

    #[test]
    fn test_body_max_stress() {
        let mut body = DuctileBody::new(3, 100.0, 500.0, 100.0);
        body.apply_strain_uniform(0.001, 100.0);
        assert!(body.max_stress() >= 0.0);
    }

    #[test]
    fn test_new_state_is_elastic() {
        let state = DuctileState::new(200.0, 600.0, 100.0);
        assert!(state.is_elastic());
    }
}
