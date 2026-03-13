// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hydraulic cylinder actuator stub.

/// Hydraulic cylinder parameters.
#[derive(Clone, Debug)]
pub struct HydraulicCylinderParams {
    /// Piston bore area (m²).
    pub bore_area: f32,
    /// Rod area (m²) — used for retraction side.
    pub rod_area: f32,
    /// Stroke length (m).
    pub stroke: f32,
    /// Supply pressure (Pa).
    pub supply_pressure: f32,
    /// Bulk modulus of hydraulic fluid (Pa).
    pub bulk_modulus: f32,
}

impl Default for HydraulicCylinderParams {
    fn default() -> Self {
        Self {
            bore_area: 0.002,
            rod_area: 0.001,
            stroke: 0.3,
            supply_pressure: 200_000.0,
            bulk_modulus: 1.5e9,
        }
    }
}

/// Hydraulic cylinder state.
#[derive(Clone, Debug, Default)]
pub struct HydraulicCylinderState {
    /// Current extension (m), 0 = fully retracted.
    pub extension: f32,
    /// Current velocity (m/s).
    pub velocity: f32,
    /// Pressure on extension side (Pa).
    pub pressure_extend: f32,
    /// Pressure on retraction side (Pa).
    pub pressure_retract: f32,
}

/// Computes the extension force of the cylinder.
pub fn extension_force(params: &HydraulicCylinderParams, state: &HydraulicCylinderState) -> f32 {
    state.pressure_extend * params.bore_area - state.pressure_retract * params.rod_area
}

/// Computes the retraction force.
pub fn retraction_force(params: &HydraulicCylinderParams, state: &HydraulicCylinderState) -> f32 {
    state.pressure_retract * params.rod_area - state.pressure_extend * params.bore_area
}

/// Sets the valve command: positive = extend, negative = retract, zero = hold.
pub fn set_valve_command(
    params: &HydraulicCylinderParams,
    state: &mut HydraulicCylinderState,
    command: f32,
) {
    let cmd = command.clamp(-1.0, 1.0);
    if cmd > 0.0 {
        state.pressure_extend = params.supply_pressure * cmd;
        state.pressure_retract = 0.0;
    } else if cmd < 0.0 {
        state.pressure_retract = params.supply_pressure * (-cmd);
        state.pressure_extend = 0.0;
    } else {
        /* hold */
    }
}

/// Steps the cylinder simulation by dt seconds under a given load force.
pub fn step_cylinder(
    params: &HydraulicCylinderParams,
    state: &mut HydraulicCylinderState,
    load_force: f32,
    mass_kg: f32,
    dt: f32,
) {
    let net_force = extension_force(params, state) - load_force;
    let accel = if mass_kg > 1e-9 {
        net_force / mass_kg
    } else {
        0.0
    };
    state.velocity += accel * dt;
    state.extension = (state.extension + state.velocity * dt).clamp(0.0, params.stroke);
    if (0.0..=1.0).contains(&(state.extension / params.stroke.max(1e-9))) {
        /* within stroke, no clamp needed */
    } else {
        state.velocity = 0.0;
    }
}

/// Returns the extension ratio (0 = retracted, 1 = fully extended).
pub fn extension_ratio(params: &HydraulicCylinderParams, state: &HydraulicCylinderState) -> f32 {
    (state.extension / params.stroke).clamp(0.0, 1.0)
}

/// Returns the hydraulic power consumed (W).
pub fn hydraulic_power(params: &HydraulicCylinderParams, state: &HydraulicCylinderState) -> f32 {
    extension_force(params, state) * state.velocity.abs()
}

/// Hydraulic cylinder stub struct.
pub struct HydraulicCylinder {
    pub params: HydraulicCylinderParams,
    pub state: HydraulicCylinderState,
}

impl HydraulicCylinder {
    /// Creates a new hydraulic cylinder with default params.
    pub fn new(params: HydraulicCylinderParams) -> Self {
        Self {
            state: HydraulicCylinderState::default(),
            params,
        }
    }

    /// Commands the cylinder and steps simulation.
    pub fn actuate(&mut self, command: f32, load_force: f32, mass_kg: f32, dt: f32) {
        set_valve_command(&self.params, &mut self.state, command);
        step_cylinder(&self.params, &mut self.state, load_force, mass_kg, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cylinder() -> HydraulicCylinder {
        HydraulicCylinder::new(HydraulicCylinderParams::default())
    }

    #[test]
    fn test_extend_command_sets_extend_pressure() {
        let mut c = default_cylinder();
        set_valve_command(&c.params, &mut c.state, 1.0);
        assert!(c.state.pressure_extend > 0.0);
        assert_eq!(c.state.pressure_retract, 0.0);
    }

    #[test]
    fn test_retract_command_sets_retract_pressure() {
        let mut c = default_cylinder();
        set_valve_command(&c.params, &mut c.state, -1.0);
        assert!(c.state.pressure_retract > 0.0);
        assert_eq!(c.state.pressure_extend, 0.0);
    }

    #[test]
    fn test_extension_force_positive_when_extending() {
        let mut c = default_cylinder();
        set_valve_command(&c.params, &mut c.state, 1.0);
        assert!(extension_force(&c.params, &c.state) > 0.0);
    }

    #[test]
    fn test_cylinder_extends_under_command() {
        let mut c = default_cylinder();
        c.actuate(1.0, 0.0, 10.0, 0.1);
        assert!(c.state.extension > 0.0);
    }

    #[test]
    fn test_extension_ratio_bounded() {
        let mut c = default_cylinder();
        c.state.extension = c.params.stroke;
        let ratio = extension_ratio(&c.params, &c.state);
        assert!((ratio - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_extension_ratio_zero_at_rest() {
        let c = default_cylinder();
        assert!((extension_ratio(&c.params, &c.state)).abs() < 1e-6);
    }

    #[test]
    fn test_hydraulic_power_zero_at_rest() {
        let c = default_cylinder();
        assert!((hydraulic_power(&c.params, &c.state)).abs() < 1e-6);
    }

    #[test]
    fn test_extension_clamped_at_stroke() {
        let mut c = default_cylinder();
        c.state.extension = c.params.stroke;
        c.state.pressure_extend = c.params.supply_pressure;
        step_cylinder(&c.params, &mut c.state, 0.0, 10.0, 1.0);
        assert!(c.state.extension <= c.params.stroke);
    }

    #[test]
    fn test_command_clamped_at_unit() {
        let mut c = default_cylinder();
        set_valve_command(&c.params, &mut c.state, 5.0); /* should clamp to 1.0 */
        assert!((c.state.pressure_extend - c.params.supply_pressure).abs() < 1.0);
    }
}
