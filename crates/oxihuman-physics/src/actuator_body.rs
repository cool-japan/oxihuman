// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Linear pneumatic actuator with stroke and force control.

#![allow(dead_code)]

/// State of a linear pneumatic actuator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ActuatorBody {
    /// Maximum stroke length in meters.
    pub max_stroke: f32,
    /// Current extension in meters.
    pub extension: f32,
    /// Current velocity in m/s.
    pub velocity: f32,
    /// Maximum force in Newtons.
    pub max_force: f32,
    /// Current applied pressure (0.0..1.0 normalized).
    pub pressure: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Effective piston area (m²) — for pressure-to-force.
    pub piston_area: f32,
}

/// Create a new pneumatic actuator.
#[allow(dead_code)]
pub fn new_actuator(
    max_stroke: f32,
    max_force: f32,
    piston_area: f32,
    damping: f32,
) -> ActuatorBody {
    ActuatorBody {
        max_stroke: max_stroke.abs(),
        extension: 0.0,
        velocity: 0.0,
        max_force: max_force.abs(),
        pressure: 0.0,
        damping,
        piston_area: piston_area.max(1e-6),
    }
}

/// Compute force exerted by the actuator given current pressure.
/// F = pressure * max_force (normalized pressure 0..1).
#[allow(dead_code)]
pub fn actuator_force(act: &ActuatorBody) -> f32 {
    act.pressure.clamp(0.0, 1.0) * act.max_force
}

/// Set the pressure command (0.0 = fully retracted, 1.0 = fully extended).
#[allow(dead_code)]
pub fn actuator_set_pressure(act: &mut ActuatorBody, pressure: f32) {
    act.pressure = pressure.clamp(0.0, 1.0);
}

/// Step the actuator using F=ma model with mass and dt.
#[allow(dead_code)]
pub fn actuator_step(act: &mut ActuatorBody, mass: f32, dt: f32) {
    let mass = mass.max(1e-6);
    let net_force = actuator_force(act) - act.damping * act.velocity;
    let accel = net_force / mass;
    act.velocity += accel * dt;
    act.extension = (act.extension + act.velocity * dt).clamp(0.0, act.max_stroke);
    if act.extension <= 0.0 || act.extension >= act.max_stroke {
        act.velocity = 0.0;
    }
}

/// Stroke ratio: 0.0 = fully retracted, 1.0 = fully extended.
#[allow(dead_code)]
pub fn actuator_stroke_ratio(act: &ActuatorBody) -> f32 {
    if act.max_stroke < 1e-10 {
        return 0.0;
    }
    act.extension / act.max_stroke
}

/// Check if actuator is fully extended.
#[allow(dead_code)]
pub fn actuator_is_extended(act: &ActuatorBody) -> bool {
    act.extension >= act.max_stroke - 1e-5
}

/// Check if actuator is fully retracted.
#[allow(dead_code)]
pub fn actuator_is_retracted(act: &ActuatorBody) -> bool {
    act.extension <= 1e-5
}

/// Power output: F * v.
#[allow(dead_code)]
pub fn actuator_power(act: &ActuatorBody) -> f32 {
    actuator_force(act) * act.velocity
}

/// Reset the actuator to its initial state.
#[allow(dead_code)]
pub fn actuator_reset(act: &mut ActuatorBody) {
    act.extension = 0.0;
    act.velocity = 0.0;
    act.pressure = 0.0;
}

/// Set a direct position command (bypasses dynamics).
#[allow(dead_code)]
pub fn actuator_set_position(act: &mut ActuatorBody, pos: f32) {
    act.extension = pos.clamp(0.0, act.max_stroke);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_act() -> ActuatorBody {
        new_actuator(0.2, 1000.0, 0.01, 10.0)
    }

    #[test]
    fn test_initial_state() {
        let a = make_act();
        assert_eq!(a.extension, 0.0);
        assert_eq!(a.pressure, 0.0);
    }

    #[test]
    fn test_force_at_zero_pressure() {
        let a = make_act();
        assert_eq!(actuator_force(&a), 0.0);
    }

    #[test]
    fn test_force_at_full_pressure() {
        let mut a = make_act();
        actuator_set_pressure(&mut a, 1.0);
        assert!((actuator_force(&a) - 1000.0).abs() < 1e-4);
    }

    #[test]
    fn test_step_extends_actuator() {
        let mut a = make_act();
        actuator_set_pressure(&mut a, 1.0);
        actuator_step(&mut a, 1.0, 0.1);
        assert!(a.extension > 0.0);
    }

    #[test]
    fn test_extension_clamped_to_max_stroke() {
        let mut a = make_act();
        actuator_set_pressure(&mut a, 1.0);
        for _ in 0..1000 {
            actuator_step(&mut a, 1.0, 0.1);
        }
        assert!(actuator_is_extended(&a));
    }

    #[test]
    fn test_stroke_ratio() {
        let mut a = make_act();
        actuator_set_position(&mut a, 0.1);
        let r = actuator_stroke_ratio(&a);
        assert!((r - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_is_retracted_initially() {
        let a = make_act();
        assert!(actuator_is_retracted(&a));
    }

    #[test]
    fn test_reset() {
        let mut a = make_act();
        actuator_set_pressure(&mut a, 0.5);
        a.extension = 0.1;
        actuator_reset(&mut a);
        assert_eq!(a.extension, 0.0);
        assert_eq!(a.pressure, 0.0);
    }

    #[test]
    fn test_set_position() {
        let mut a = make_act();
        actuator_set_position(&mut a, 0.15);
        assert!((a.extension - 0.15).abs() < 1e-5);
    }

    #[test]
    fn test_pressure_clamped() {
        let mut a = make_act();
        actuator_set_pressure(&mut a, 2.0);
        assert!(a.pressure <= 1.0);
    }
}
