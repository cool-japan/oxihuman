// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Maxwell viscoelastic model (spring + dashpot in series).

#![allow(dead_code)]

/// Maxwell element: spring (stiffness k) and dashpot (viscosity eta) in series.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaxwellElement {
    pub stiffness: f64,
    pub viscosity: f64,
    /// Internal stress variable.
    pub stress: f64,
    /// Strain applied to this element.
    pub strain: f64,
}

impl MaxwellElement {
    #[allow(dead_code)]
    pub fn new(stiffness: f64, viscosity: f64) -> Self {
        Self { stiffness, viscosity, stress: 0.0, strain: 0.0 }
    }

    /// Relaxation time tau = eta / k.
    #[allow(dead_code)]
    pub fn relaxation_time(&self) -> f64 {
        if self.stiffness > 0.0 {
            self.viscosity / self.stiffness
        } else {
            f64::INFINITY
        }
    }

    /// Update stress given strain rate and time step.
    /// Maxwell ODE: d_sigma/dt + sigma/tau = k * d_epsilon/dt
    #[allow(dead_code)]
    pub fn update(&mut self, strain_rate: f64, dt: f64) {
        let tau = self.relaxation_time();
        let d_sigma = self.stiffness * strain_rate - self.stress / tau;
        self.stress += d_sigma * dt;
        self.strain += strain_rate * dt;
    }

    /// Steady-state stress for constant strain rate.
    #[allow(dead_code)]
    pub fn steady_state_stress(&self, strain_rate: f64) -> f64 {
        self.viscosity * strain_rate
    }
}

/// Generalized Maxwell model (multiple Maxwell elements in parallel).
#[allow(dead_code)]
pub struct GeneralizedMaxwell {
    pub elements: Vec<MaxwellElement>,
    pub equilibrium_stiffness: f64,
    pub total_strain: f64,
}

impl GeneralizedMaxwell {
    #[allow(dead_code)]
    pub fn new(equilibrium_stiffness: f64) -> Self {
        Self {
            elements: Vec::new(),
            equilibrium_stiffness,
            total_strain: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn add_element(&mut self, stiffness: f64, viscosity: f64) {
        self.elements.push(MaxwellElement::new(stiffness, viscosity));
    }

    #[allow(dead_code)]
    pub fn update(&mut self, strain_rate: f64, dt: f64) {
        self.total_strain += strain_rate * dt;
        for elem in &mut self.elements {
            elem.update(strain_rate, dt);
        }
    }

    /// Total stress = equilibrium + sum of element stresses.
    #[allow(dead_code)]
    pub fn total_stress(&self) -> f64 {
        let elem_stress: f64 = self.elements.iter().map(|e| e.stress).sum();
        self.equilibrium_stiffness * self.total_strain + elem_stress
    }

    #[allow(dead_code)]
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        for e in &mut self.elements {
            e.stress = 0.0;
            e.strain = 0.0;
        }
        self.total_strain = 0.0;
    }

    /// Instantaneous (glassy) stiffness = sum of all element stiffnesses + equilibrium.
    #[allow(dead_code)]
    pub fn instantaneous_stiffness(&self) -> f64 {
        self.equilibrium_stiffness
            + self.elements.iter().map(|e| e.stiffness).sum::<f64>()
    }
}

/// Kelvin-Voigt model: spring and dashpot in parallel.
#[allow(dead_code)]
pub struct KelvinVoigt {
    pub stiffness: f64,
    pub viscosity: f64,
    pub strain: f64,
}

impl KelvinVoigt {
    #[allow(dead_code)]
    pub fn new(stiffness: f64, viscosity: f64) -> Self {
        Self { stiffness, viscosity, strain: 0.0 }
    }

    #[allow(dead_code)]
    pub fn stress(&self, strain_rate: f64) -> f64 {
        self.stiffness * self.strain + self.viscosity * strain_rate
    }

    #[allow(dead_code)]
    pub fn update(&mut self, strain_rate: f64, dt: f64) {
        self.strain += strain_rate * dt;
    }

    #[allow(dead_code)]
    pub fn relaxation_time(&self) -> f64 {
        if self.stiffness > 0.0 {
            self.viscosity / self.stiffness
        } else {
            f64::INFINITY
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relaxation_time() {
        let elem = MaxwellElement::new(100.0, 50.0);
        assert!((elem.relaxation_time() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_stress_increases_with_strain() {
        let mut elem = MaxwellElement::new(100.0, 10.0);
        for _ in 0..10 {
            elem.update(0.1, 0.01);
        }
        assert!(elem.stress > 0.0);
    }

    #[test]
    fn test_stress_relaxes_without_straining() {
        let mut elem = MaxwellElement::new(100.0, 1.0);
        elem.stress = 100.0;
        for _ in 0..1000 {
            elem.update(0.0, 0.01);
        }
        assert!(elem.stress.abs() < 1.0, "stress={}", elem.stress);
    }

    #[test]
    fn test_steady_state_stress() {
        let elem = MaxwellElement::new(100.0, 50.0);
        let ss = elem.steady_state_stress(2.0);
        assert!((ss - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_generalized_maxwell_total_stress() {
        let mut gm = GeneralizedMaxwell::new(50.0);
        gm.add_element(100.0, 10.0);
        gm.update(1.0, 0.01);
        assert!(gm.total_stress() > 0.0);
    }

    #[test]
    fn test_generalized_maxwell_reset() {
        let mut gm = GeneralizedMaxwell::new(50.0);
        gm.add_element(100.0, 10.0);
        gm.update(1.0, 0.1);
        gm.reset();
        assert!((gm.total_stress()).abs() < 1e-12);
    }

    #[test]
    fn test_instantaneous_stiffness() {
        let mut gm = GeneralizedMaxwell::new(50.0);
        gm.add_element(100.0, 10.0);
        assert!((gm.instantaneous_stiffness() - 150.0).abs() < 1e-9);
    }

    #[test]
    fn test_kelvin_voigt_stress() {
        let mut kv = KelvinVoigt::new(100.0, 10.0);
        kv.update(1.0, 0.1);
        let s = kv.stress(1.0);
        assert!(s > 0.0);
    }

    #[test]
    fn test_kelvin_voigt_relaxation_time() {
        let kv = KelvinVoigt::new(50.0, 100.0);
        assert!((kv.relaxation_time() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_element_count() {
        let mut gm = GeneralizedMaxwell::new(0.0);
        gm.add_element(1.0, 1.0);
        gm.add_element(2.0, 2.0);
        assert_eq!(gm.element_count(), 2);
    }
}
