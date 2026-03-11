// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Maxwell/Kelvin-Voigt viscoelastic element stub.
//!
//! Implements the standard linear solid (SLS) model combining spring and
//! dashpot elements in series (Maxwell) and parallel (Kelvin-Voigt) arrangements.

/// A single viscoelastic element (spring or dashpot).
#[derive(Debug, Clone, PartialEq)]
pub enum ElementKind {
    Spring { stiffness: f64 },
    Dashpot { viscosity: f64 },
}

/// Maxwell model: spring + dashpot in series.
#[derive(Debug, Clone)]
pub struct MaxwellModel {
    pub spring_stiffness: f64,
    pub dashpot_viscosity: f64,
    /// Internal spring strain.
    pub spring_strain: f64,
    /// Internal dashpot strain rate (stored as strain).
    pub dashpot_strain: f64,
}

impl MaxwellModel {
    pub fn new(k: f64, eta: f64) -> Self {
        Self {
            spring_stiffness: k,
            dashpot_viscosity: eta,
            spring_strain: 0.0,
            dashpot_strain: 0.0,
        }
    }

    pub fn relaxation_time(&self) -> f64 {
        if self.spring_stiffness == 0.0 {
            return f64::INFINITY;
        }
        self.dashpot_viscosity / self.spring_stiffness
    }

    /// Compute stress from spring strain.
    pub fn stress(&self) -> f64 {
        self.spring_stiffness * self.spring_strain
    }

    /// Integrate one time step at a given total strain rate.
    pub fn integrate(&mut self, strain_rate: f64, dt: f64) {
        let tau = self.relaxation_time();
        if tau <= 0.0 {
            return;
        }
        let total_strain = self.spring_strain + self.dashpot_strain;
        let d_spring = strain_rate - self.spring_strain / tau;
        self.spring_strain += d_spring * dt;
        self.dashpot_strain += (strain_rate - d_spring) * dt;
        let _ = total_strain;
    }
}

/// Kelvin-Voigt model: spring + dashpot in parallel.
#[derive(Debug, Clone)]
pub struct KelvinVoigtModel {
    pub spring_stiffness: f64,
    pub dashpot_viscosity: f64,
    /// Current strain.
    pub strain: f64,
    /// Current strain rate.
    pub strain_rate: f64,
}

impl KelvinVoigtModel {
    pub fn new(k: f64, eta: f64) -> Self {
        Self {
            spring_stiffness: k,
            dashpot_viscosity: eta,
            strain: 0.0,
            strain_rate: 0.0,
        }
    }

    pub fn retardation_time(&self) -> f64 {
        if self.spring_stiffness == 0.0 {
            return f64::INFINITY;
        }
        self.dashpot_viscosity / self.spring_stiffness
    }

    /// Compute stress from current strain and strain rate.
    pub fn stress(&self) -> f64 {
        self.spring_stiffness * self.strain + self.dashpot_viscosity * self.strain_rate
    }

    /// Integrate strain response under constant stress.
    pub fn integrate_under_stress(&mut self, applied_stress: f64, dt: f64) {
        let tau = self.retardation_time();
        if tau <= 0.0 {
            return;
        }
        /* ε(t+dt) ≈ ε(t) + dt * [σ/η - ε(t)/τ] */
        let d_strain = dt * (applied_stress / self.dashpot_viscosity - self.strain / tau);
        self.strain_rate = d_strain / dt;
        self.strain += d_strain;
    }
}

/// Standard Linear Solid (SLS / Zener) model.
#[derive(Debug, Clone)]
pub struct SlsModel {
    pub e_0: f64, /* Equilibrium spring stiffness */
    pub e_1: f64, /* Maxwell spring stiffness */
    pub eta: f64, /* Maxwell dashpot viscosity */
    pub internal_strain: f64,
}

impl SlsModel {
    pub fn new(e0: f64, e1: f64, eta: f64) -> Self {
        Self {
            e_0: e0,
            e_1: e1,
            eta,
            internal_strain: 0.0,
        }
    }

    pub fn relaxation_time(&self) -> f64 {
        if self.e_1 <= 0.0 {
            return f64::INFINITY;
        }
        self.eta / self.e_1
    }

    /// Stress from equilibrium spring + Maxwell element.
    pub fn stress(&self, total_strain: f64) -> f64 {
        self.e_0 * total_strain + self.e_1 * (total_strain - self.internal_strain)
    }

    pub fn integrate(&mut self, total_strain: f64, dt: f64) {
        let tau = self.relaxation_time();
        if tau <= 0.0 {
            return;
        }
        let d_int = dt * (total_strain - self.internal_strain) / tau;
        self.internal_strain += d_int;
    }
}

/// Compute the storage modulus E'(ω) for a Maxwell model.
pub fn maxwell_storage_modulus(model: &MaxwellModel, omega: f64) -> f64 {
    let tau = model.relaxation_time();
    let wt = omega * tau;
    model.spring_stiffness * wt * wt / (1.0 + wt * wt)
}

/// Compute the loss modulus E''(ω) for a Maxwell model.
pub fn maxwell_loss_modulus(model: &MaxwellModel, omega: f64) -> f64 {
    let tau = model.relaxation_time();
    let wt = omega * tau;
    model.spring_stiffness * wt / (1.0 + wt * wt)
}

/// Compute tan δ (loss tangent) = E'' / E'.
pub fn loss_tangent(storage: f64, loss: f64) -> f64 {
    if storage == 0.0 {
        return f64::INFINITY;
    }
    loss / storage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maxwell_relaxation_time() {
        let m = MaxwellModel::new(1000.0, 10.0);
        assert!((m.relaxation_time() - 0.01).abs() < 1e-12);
    }

    #[test]
    fn test_kelvin_retardation_time() {
        let kv = KelvinVoigtModel::new(1000.0, 10.0);
        assert!((kv.retardation_time() - 0.01).abs() < 1e-12);
    }

    #[test]
    fn test_kelvin_stress_zero_at_zero_strain() {
        let kv = KelvinVoigtModel::new(1000.0, 10.0);
        assert_eq!(kv.stress(), 0.0);
    }

    #[test]
    fn test_sls_stress_proportional() {
        let sls = SlsModel::new(1000.0, 2000.0, 1.0);
        let s = sls.stress(0.001);
        assert!(s > 0.0);
    }

    #[test]
    fn test_maxwell_storage_modulus() {
        let m = MaxwellModel::new(1000.0, 1.0);
        let e_prime = maxwell_storage_modulus(&m, 1.0);
        assert!(e_prime > 0.0 && e_prime < 1000.0);
    }

    #[test]
    fn test_maxwell_loss_modulus() {
        let m = MaxwellModel::new(1000.0, 1.0);
        let e_double_prime = maxwell_loss_modulus(&m, 1.0);
        assert!(e_double_prime > 0.0);
    }

    #[test]
    fn test_loss_tangent() {
        let tan_d = loss_tangent(500.0, 500.0);
        assert!((tan_d - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_maxwell_integrate_changes_strain() {
        let mut m = MaxwellModel::new(1000.0, 1.0);
        m.integrate(0.01, 0.001);
        assert!(m.spring_strain != 0.0 || m.dashpot_strain != 0.0);
    }

    #[test]
    fn test_kelvin_integrate_under_stress() {
        let mut kv = KelvinVoigtModel::new(1000.0, 10.0);
        kv.integrate_under_stress(100.0, 0.001);
        assert!(kv.strain > 0.0);
    }
}
