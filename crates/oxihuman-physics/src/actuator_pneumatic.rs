// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pneumatic piston actuator stub.

/// Pneumatic cylinder parameters.
#[derive(Clone, Debug)]
pub struct PneumaticCylinderParams {
    /// Bore area (m²).
    pub bore_area: f32,
    /// Stroke length (m).
    pub stroke: f32,
    /// Supply pressure (Pa).
    pub supply_pressure: f32,
    /// Piston mass (kg).
    pub piston_mass: f32,
    /// Viscous friction coefficient (N·s/m).
    pub friction: f32,
}

impl Default for PneumaticCylinderParams {
    fn default() -> Self {
        Self {
            bore_area: 0.005,
            stroke: 0.2,
            supply_pressure: 600_000.0, /* 6 bar */
            piston_mass: 0.3,
            friction: 10.0,
        }
    }
}

/// Pneumatic cylinder state.
#[derive(Clone, Debug, Default)]
pub struct PneumaticCylinderState {
    /// Current extension (m).
    pub extension: f32,
    /// Velocity (m/s).
    pub velocity: f32,
    /// Gauge pressure inside cylinder (Pa).
    pub chamber_pressure: f32,
    /// Solenoid valve open (true = flow to extend).
    pub valve_open: bool,
}

/// Opens or closes the solenoid valve and sets pressure.
pub fn set_valve(params: &PneumaticCylinderParams, state: &mut PneumaticCylinderState, open: bool) {
    state.valve_open = open;
    state.chamber_pressure = if open { params.supply_pressure } else { 0.0 };
}

/// Computes the piston force (extension direction positive).
pub fn piston_force(params: &PneumaticCylinderParams, state: &PneumaticCylinderState) -> f32 {
    state.chamber_pressure * params.bore_area - params.friction * state.velocity
}

/// Steps the cylinder by dt seconds under a given load.
pub fn step_pneumatic(
    params: &PneumaticCylinderParams,
    state: &mut PneumaticCylinderState,
    load_force: f32,
    dt: f32,
) {
    let force = piston_force(params, state) - load_force;
    let accel = force / params.piston_mass;
    state.velocity += accel * dt;
    state.extension = (state.extension + state.velocity * dt).clamp(0.0, params.stroke);
}

/// Returns the extension ratio (0 = retracted, 1 = fully extended).
pub fn extension_ratio(params: &PneumaticCylinderParams, state: &PneumaticCylinderState) -> f32 {
    (state.extension / params.stroke).clamp(0.0, 1.0)
}

/// Returns the pneumatic power consumed (W).
pub fn pneumatic_power(params: &PneumaticCylinderParams, state: &PneumaticCylinderState) -> f32 {
    piston_force(params, state).max(0.0) * state.velocity.max(0.0)
}

/// Returns whether the cylinder is fully extended.
pub fn is_fully_extended(params: &PneumaticCylinderParams, state: &PneumaticCylinderState) -> bool {
    state.extension >= params.stroke - 1e-6
}

/// Pneumatic cylinder stub struct.
pub struct PneumaticCylinder {
    pub params: PneumaticCylinderParams,
    pub state: PneumaticCylinderState,
}

impl PneumaticCylinder {
    /// Creates a new pneumatic cylinder with default params.
    pub fn new(params: PneumaticCylinderParams) -> Self {
        Self {
            state: PneumaticCylinderState::default(),
            params,
        }
    }

    /// Actuates the cylinder valve and steps simulation.
    pub fn actuate(&mut self, open: bool, load_force: f32, dt: f32) {
        set_valve(&self.params, &mut self.state, open);
        step_pneumatic(&self.params, &mut self.state, load_force, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cyl() -> PneumaticCylinder {
        PneumaticCylinder::new(PneumaticCylinderParams::default())
    }

    #[test]
    fn test_valve_open_sets_pressure() {
        let mut c = default_cyl();
        set_valve(&c.params, &mut c.state, true);
        assert!(c.state.chamber_pressure > 0.0);
    }

    #[test]
    fn test_valve_closed_clears_pressure() {
        let mut c = default_cyl();
        set_valve(&c.params, &mut c.state, true);
        set_valve(&c.params, &mut c.state, false);
        assert_eq!(c.state.chamber_pressure, 0.0);
    }

    #[test]
    fn test_piston_force_positive_when_valve_open() {
        let mut c = default_cyl();
        set_valve(&c.params, &mut c.state, true);
        assert!(piston_force(&c.params, &c.state) > 0.0);
    }

    #[test]
    fn test_cylinder_extends_when_valve_open() {
        let mut c = default_cyl();
        c.actuate(true, 0.0, 0.05);
        assert!(c.state.extension > 0.0);
    }

    #[test]
    fn test_extension_ratio_bounded_01() {
        let mut c = default_cyl();
        c.state.extension = c.params.stroke;
        let r = extension_ratio(&c.params, &c.state);
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn test_extension_clamped_to_stroke() {
        let mut c = default_cyl();
        set_valve(&c.params, &mut c.state, true);
        for _ in 0..200 {
            step_pneumatic(&c.params, &mut c.state, 0.0, 0.01);
        }
        assert!(c.state.extension <= c.params.stroke + 1e-5);
    }

    #[test]
    fn test_power_zero_at_rest() {
        let c = default_cyl();
        assert!((pneumatic_power(&c.params, &c.state)).abs() < 1e-6);
    }

    #[test]
    fn test_fully_extended_detection() {
        let mut c = default_cyl();
        c.state.extension = c.params.stroke;
        assert!(is_fully_extended(&c.params, &c.state));
    }

    #[test]
    fn test_not_fully_extended_at_rest() {
        let c = default_cyl();
        assert!(!is_fully_extended(&c.params, &c.state));
    }
}
