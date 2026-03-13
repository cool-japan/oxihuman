// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Piezoelectric actuator model (stack actuator stub).

/// A piezoelectric stack actuator.
#[derive(Debug, Clone)]
pub struct PiezoActuator {
    /// Maximum free stroke (m) at rated voltage.
    pub max_stroke: f32,
    /// Blocking force (N) at rated voltage.
    pub blocking_force: f32,
    /// Rated voltage (V).
    pub rated_voltage: f32,
    /// Current applied voltage (V).
    pub voltage: f32,
    /// Stiffness of the actuator (N/m).
    pub stiffness: f32,
    /// Current extension (m).
    pub extension: f32,
}

impl PiezoActuator {
    pub fn new(max_stroke: f32, blocking_force: f32, rated_voltage: f32) -> Self {
        let stiffness = blocking_force / max_stroke.max(1e-12);
        PiezoActuator {
            max_stroke,
            blocking_force,
            rated_voltage,
            voltage: 0.0,
            stiffness,
            extension: 0.0,
        }
    }
}

/// Create a new piezo actuator.
pub fn new_piezo(max_stroke: f32, blocking_force: f32, rated_voltage: f32) -> PiezoActuator {
    PiezoActuator::new(max_stroke, blocking_force, rated_voltage)
}

/// Set the drive voltage.
pub fn piezo_set_voltage(p: &mut PiezoActuator, v: f32) {
    p.voltage = v.clamp(0.0, p.rated_voltage);
}

/// Compute the free stroke at current voltage.
pub fn piezo_free_stroke(p: &PiezoActuator) -> f32 {
    p.max_stroke * (p.voltage / p.rated_voltage.max(1e-10))
}

/// Compute force output against an external load (spring constant `k_load`, displacement `x`).
pub fn piezo_force(p: &PiezoActuator, k_load: f32, x: f32) -> f32 {
    /* Force from actuator minus spring load force */
    let free = piezo_free_stroke(p);
    let f_act = p.stiffness * (free - x);
    let f_load = k_load * x;
    (f_act - f_load).max(0.0)
}

/// Update extension under a load `k_load` (N/m) in series.
pub fn piezo_update_extension(p: &mut PiezoActuator, k_load: f32) {
    let free = piezo_free_stroke(p);
    /* equilibrium: k_p * (x0 - x) = k_load * x  => x = k_p*x0 / (k_p + k_load) */
    p.extension = p.stiffness * free / (p.stiffness + k_load).max(1e-10);
}

/// Return the electrical power (W) consumed (simplified).
pub fn piezo_power(p: &PiezoActuator, frequency: f32, capacitance: f32) -> f32 {
    /* P = C V² f for capacitive load */
    capacitance * p.voltage * p.voltage * frequency
}

/// Return the electromechanical coupling coefficient k² (simplified).
pub fn piezo_coupling_k2(p: &PiezoActuator) -> f32 {
    /* k² = U_mech / U_total; simplified: proportional to voltage ratio */
    (p.voltage / p.rated_voltage.max(1e-10)).powi(2)
}

/// Return `true` if the actuator is at full extension.
pub fn piezo_is_full_stroke(p: &PiezoActuator) -> bool {
    (p.extension - p.max_stroke).abs() < 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_voltage_zero_stroke() {
        let p = new_piezo(100e-6, 1000.0, 150.0);
        assert!(piezo_free_stroke(&p).abs() < 1e-12);
    }

    #[test]
    fn test_full_voltage_max_stroke() {
        let mut p = new_piezo(100e-6, 1000.0, 150.0);
        piezo_set_voltage(&mut p, 150.0);
        let s = piezo_free_stroke(&p);
        assert!((s - 100e-6).abs() < 1e-12);
    }

    #[test]
    fn test_voltage_clamped_to_rated() {
        let mut p = new_piezo(100e-6, 1000.0, 150.0);
        piezo_set_voltage(&mut p, 300.0);
        assert!(p.voltage <= 150.0);
    }

    #[test]
    fn test_update_extension_under_load() {
        let mut p = new_piezo(100e-6, 1000.0, 150.0);
        piezo_set_voltage(&mut p, 150.0);
        piezo_update_extension(&mut p, 500.0);
        assert!(p.extension > 0.0 && p.extension < 100e-6);
    }

    #[test]
    fn test_power_consumption() {
        let mut p = new_piezo(100e-6, 1000.0, 150.0);
        piezo_set_voltage(&mut p, 100.0);
        let power = piezo_power(&p, 1000.0, 1e-9);
        assert!(power > 0.0);
    }

    #[test]
    fn test_coupling_k2_at_full_voltage() {
        let mut p = new_piezo(100e-6, 1000.0, 150.0);
        piezo_set_voltage(&mut p, 150.0);
        assert!((piezo_coupling_k2(&p) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_force_positive_under_small_load() {
        let mut p = new_piezo(100e-6, 1000.0, 150.0);
        piezo_set_voltage(&mut p, 150.0);
        let f = piezo_force(&p, 100.0, 10e-6);
        assert!(f >= 0.0);
    }

    #[test]
    fn test_stiffness_computed() {
        let p = new_piezo(100e-6, 1000.0, 150.0);
        assert!((p.stiffness - 1000.0 / 100e-6).abs() < 1.0);
    }

    #[test]
    fn test_voltage_zero_no_power() {
        let p = new_piezo(100e-6, 1000.0, 150.0);
        assert!(piezo_power(&p, 1000.0, 1e-9).abs() < 1e-12);
    }
}
