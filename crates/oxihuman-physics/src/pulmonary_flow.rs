// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Pulmonary alveolar capillary circuit (Poiseuille flow).
pub struct PulmonaryCircuit {
    pub driving_pressure_pa: f32,
    pub total_resistance: f32,
    pub num_capillaries: u32,
    pub capillary_radius: f32,
    pub capillary_length: f32,
    pub viscosity: f32,
}

impl PulmonaryCircuit {
    pub fn new() -> Self {
        PulmonaryCircuit {
            driving_pressure_pa: 1200.0, // ~9 mmHg
            total_resistance: 0.0,       // computed
            num_capillaries: 280_000_000,
            capillary_radius: 3.5e-6, // 3.5 μm
            capillary_length: 600e-6, // 600 μm
            viscosity: 0.0027,        // blood viscosity Pa·s
        }
    }
}

impl Default for PulmonaryCircuit {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_pulmonary_circuit() -> PulmonaryCircuit {
    PulmonaryCircuit::new()
}

/// Poiseuille resistance for a single capillary: 8ηL / (π r^4)
/// Total resistance for n parallel capillaries: R_single / n
pub fn pulmonary_resistance(p: &PulmonaryCircuit) -> f32 {
    let r_single = 8.0 * p.viscosity * p.capillary_length / (PI * p.capillary_radius.powi(4));
    let n = p.num_capillaries.max(1) as f32;
    r_single / n
}

pub fn pulmonary_flow_rate(p: &PulmonaryCircuit) -> f32 {
    let r = pulmonary_resistance(p);
    if r <= 0.0 {
        return 0.0;
    }
    p.driving_pressure_pa / r
}

/// Transit time = capillary length / (average flow velocity)
/// v_avg = Q / (n * π r²)
pub fn pulmonary_transit_time(p: &PulmonaryCircuit) -> f32 {
    let q = pulmonary_flow_rate(p);
    let n = p.num_capillaries.max(1) as f32;
    let area = PI * p.capillary_radius * p.capillary_radius;
    let v_avg = q / (n * area);
    if v_avg <= 0.0 {
        return f32::INFINITY;
    }
    p.capillary_length / v_avg
}

pub fn pulmonary_update_radius(p: &mut PulmonaryCircuit, r: f32) {
    p.capillary_radius = r;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new circuit has defaults */
        let p = new_pulmonary_circuit();
        assert!(p.capillary_radius > 0.0);
        assert!(p.driving_pressure_pa > 0.0);
    }

    #[test]
    fn test_resistance_positive() {
        /* Poiseuille resistance is positive */
        let p = new_pulmonary_circuit();
        assert!(pulmonary_resistance(&p) > 0.0);
    }

    #[test]
    fn test_flow_rate_positive() {
        /* flow rate is positive */
        let p = new_pulmonary_circuit();
        assert!(pulmonary_flow_rate(&p) > 0.0);
    }

    #[test]
    fn test_transit_time_positive() {
        /* transit time is positive */
        let p = new_pulmonary_circuit();
        assert!(pulmonary_transit_time(&p) > 0.0);
    }

    #[test]
    fn test_resistance_decreases_with_radius() {
        /* larger radius → lower resistance */
        let mut p1 = new_pulmonary_circuit();
        let mut p2 = new_pulmonary_circuit();
        pulmonary_update_radius(&mut p1, 3.5e-6);
        pulmonary_update_radius(&mut p2, 7.0e-6);
        assert!(pulmonary_resistance(&p2) < pulmonary_resistance(&p1));
    }

    #[test]
    fn test_update_radius() {
        /* update_radius changes the radius */
        let mut p = new_pulmonary_circuit();
        pulmonary_update_radius(&mut p, 5e-6);
        assert!((p.capillary_radius - 5e-6).abs() < 1e-12);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let p = PulmonaryCircuit::default();
        assert!(p.num_capillaries > 0);
    }
}
