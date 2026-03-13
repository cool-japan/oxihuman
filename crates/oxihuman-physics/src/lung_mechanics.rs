// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lung volume/pressure mechanics model (respiratory physiology stub).

/// A simple single-compartment lung model.
#[derive(Debug, Clone)]
pub struct Lung {
    /// Functional residual capacity (FRC) — resting volume (m³).
    pub frc: f32,
    /// Current lung volume (m³).
    pub volume: f32,
    /// Elastic recoil pressure constant (Pa/m³).
    pub compliance_inv: f32, /* 1 / compliance */
    /// Airway resistance (Pa·s/m³).
    pub resistance: f32,
    /// Pleural pressure (Pa, negative = subatmospheric).
    pub pleural_pressure: f32,
}

impl Lung {
    pub fn new() -> Self {
        Lung {
            frc: 2.5e-3, /* 2.5 L in SI */
            volume: 2.5e-3,
            compliance_inv: 200.0, /* 1/(5 mL/cmH2O) */
            resistance: 200.0,
            pleural_pressure: -500.0, /* -5 cmH2O */
        }
    }
}

impl Default for Lung {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new default lung model.
pub fn new_lung() -> Lung {
    Lung::new()
}

/// Elastic recoil pressure (Pa).
pub fn lung_elastic_pressure(lung: &Lung) -> f32 {
    lung.compliance_inv * (lung.volume - lung.frc)
}

/// Simulate airflow into/out of the lung given driving pressure `delta_p` (Pa) for `dt` s.
pub fn lung_step(lung: &mut Lung, delta_p: f32, dt: f32) {
    let flow = delta_p / lung.resistance.max(1e-10); /* m³/s */
    lung.volume += flow * dt;
    lung.volume = lung.volume.max(0.0);
}

/// Return tidal volume relative to FRC.
pub fn lung_tidal_volume(lung: &Lung) -> f32 {
    (lung.volume - lung.frc).abs()
}

/// Return `true` if lung is at or above functional residual capacity.
pub fn lung_above_frc(lung: &Lung) -> bool {
    lung.volume >= lung.frc
}

/// Simulate one breath cycle (inspiration + expiration).
pub fn lung_breathe(lung: &mut Lung, tidal_v: f32) {
    /* inspire */
    lung.volume += tidal_v;
    /* expire passively */
    lung.volume -= tidal_v;
}

/// Return ventilation (flow) for given respiratory rate (breaths/min) and tidal volume.
pub fn lung_ventilation(resp_rate: f32, tidal_v: f32) -> f32 {
    resp_rate * tidal_v / 60.0
}

/// Return compliance (m³/Pa).
pub fn lung_compliance(lung: &Lung) -> f32 {
    1.0 / lung.compliance_inv.max(1e-10)
}

/// Set lung volume (clamped to ≥ 0).
pub fn lung_set_volume(lung: &mut Lung, v: f32) {
    lung.volume = v.max(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lung_at_frc() {
        let lung = new_lung();
        assert!((lung.volume - lung.frc).abs() < 1e-8);
    }

    #[test]
    fn test_elastic_pressure_zero_at_frc() {
        let lung = new_lung();
        assert!(lung_elastic_pressure(&lung).abs() < 1e-5);
    }

    #[test]
    fn test_step_inflates_under_pressure() {
        let mut lung = new_lung();
        let v0 = lung.volume;
        lung_step(&mut lung, 1000.0, 0.01);
        assert!(lung.volume > v0);
    }

    #[test]
    fn test_step_deflates_under_negative_pressure() {
        let mut lung = new_lung();
        lung.volume = 3.0e-3; /* above FRC */
        let v0 = lung.volume;
        lung_step(&mut lung, -500.0, 0.05);
        assert!(lung.volume < v0);
    }

    #[test]
    fn test_tidal_volume_zero_at_frc() {
        let lung = new_lung();
        assert!(lung_tidal_volume(&lung) < 1e-8);
    }

    #[test]
    fn test_above_frc_true() {
        let mut lung = new_lung();
        lung.volume = lung.frc + 0.001;
        assert!(lung_above_frc(&lung));
    }

    #[test]
    fn test_ventilation_calculation() {
        let v = lung_ventilation(15.0, 0.5e-3); /* 15 breaths/min, 500 mL */
        assert!(v > 0.0);
    }

    #[test]
    fn test_compliance_positive() {
        let lung = new_lung();
        assert!(lung_compliance(&lung) > 0.0);
    }

    #[test]
    fn test_volume_nonnegative() {
        let mut lung = new_lung();
        for _ in 0..1000 {
            lung_step(&mut lung, -100_000.0, 0.01);
        }
        assert!(lung.volume >= 0.0);
    }
}
