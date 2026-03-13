// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cable-driven actuator stub — models a cable reel/spool drive.

/// Cable-driven actuator parameters.
#[derive(Clone, Debug)]
pub struct CableDriveParams {
    /// Cable stiffness (N/m).
    pub stiffness: f32,
    /// Cable damping (N·s/m).
    pub damping: f32,
    /// Maximum cable tension (N).
    pub max_tension: f32,
    /// Spool radius (m).
    pub spool_radius: f32,
    /// Cable length at rest (m).
    pub rest_length: f32,
}

impl Default for CableDriveParams {
    fn default() -> Self {
        Self {
            stiffness: 5000.0,
            damping: 20.0,
            max_tension: 500.0,
            spool_radius: 0.02,
            rest_length: 0.5,
        }
    }
}

/// Cable-driven actuator state.
#[derive(Clone, Debug, Default)]
pub struct CableDriveState {
    /// Current cable length (m).
    pub cable_length: f32,
    /// Spool angle (rad) — positive = wind in (shorten cable).
    pub spool_angle: f32,
    /// Current tension in cable (N).
    pub tension: f32,
    /// Load position (m) along cable direction.
    pub load_position: f32,
}

/// Creates a new cable drive state.
pub fn new_cable_state(rest_length: f32) -> CableDriveState {
    CableDriveState {
        cable_length: rest_length,
        ..Default::default()
    }
}

/// Computes cable length from spool angle and rest length.
pub fn cable_length_from_spool(params: &CableDriveParams, spool_angle: f32) -> f32 {
    (params.rest_length - params.spool_radius * spool_angle).max(0.0)
}

/// Computes tension based on cable extension and velocity.
pub fn compute_cable_tension(
    params: &CableDriveParams,
    state: &CableDriveState,
    velocity: f32,
) -> f32 {
    let extension = (state.load_position - state.cable_length).max(0.0);
    let tension = params.stiffness * extension + params.damping * velocity.max(0.0);
    tension.min(params.max_tension)
}

/// Winds the spool by the given angle (positive = shorten cable).
pub fn wind_spool(params: &CableDriveParams, state: &mut CableDriveState, delta_angle: f32) {
    state.spool_angle += delta_angle;
    state.cable_length = cable_length_from_spool(params, state.spool_angle);
}

/// Steps the cable drive simulation by dt seconds.
pub fn step_cable_drive(
    params: &CableDriveParams,
    state: &mut CableDriveState,
    load_velocity: f32,
    dt: f32,
) {
    let tension = compute_cable_tension(params, state, load_velocity);
    state.tension = tension;
    state.load_position += load_velocity * dt;
}

/// Returns the torque at the spool due to cable tension (N·m).
pub fn spool_torque(params: &CableDriveParams, state: &CableDriveState) -> f32 {
    state.tension * params.spool_radius
}

/// Returns whether the cable is slack (no tension).
pub fn is_cable_slack(state: &CableDriveState) -> bool {
    state.tension < 1e-6
}

/// Cable drive stub struct.
pub struct CableDrive {
    pub params: CableDriveParams,
    pub state: CableDriveState,
}

impl CableDrive {
    /// Creates a new cable drive with default params.
    pub fn new(params: CableDriveParams) -> Self {
        let state = new_cable_state(params.rest_length);
        Self { state, params }
    }

    /// Winds the spool by delta_angle and steps simulation.
    pub fn actuate(&mut self, delta_angle: f32, load_velocity: f32, dt: f32) {
        wind_spool(&self.params, &mut self.state, delta_angle);
        step_cable_drive(&self.params, &mut self.state, load_velocity, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_drive() -> CableDrive {
        CableDrive::new(CableDriveParams::default())
    }

    #[test]
    fn test_initial_cable_length_is_rest_length() {
        let d = default_drive();
        assert!((d.state.cable_length - d.params.rest_length).abs() < 1e-6);
    }

    #[test]
    fn test_winding_shortens_cable() {
        let mut d = default_drive();
        let before = d.state.cable_length;
        wind_spool(&d.params, &mut d.state, 1.0); /* positive angle => shorten */
        assert!(d.state.cable_length < before);
    }

    #[test]
    fn test_cable_length_non_negative() {
        let mut d = default_drive();
        wind_spool(&d.params, &mut d.state, 1000.0); /* extreme winding */
        assert!(d.state.cable_length >= 0.0);
    }

    #[test]
    fn test_tension_zero_when_slack() {
        let d = default_drive();
        /* load at same position as cable length => no extension */
        let t = compute_cable_tension(&d.params, &d.state, 0.0);
        assert!(t < 1e-6);
    }

    #[test]
    fn test_tension_positive_when_loaded() {
        let mut d = default_drive();
        d.state.load_position = d.params.rest_length + 0.01;
        let t = compute_cable_tension(&d.params, &d.state, 0.0);
        assert!(t > 0.0);
    }

    #[test]
    fn test_tension_clamped_to_max() {
        let mut d = default_drive();
        d.state.load_position = d.params.rest_length + 10.0; /* huge load */
        let t = compute_cable_tension(&d.params, &d.state, 0.0);
        assert!(t <= d.params.max_tension);
    }

    #[test]
    fn test_spool_torque_positive_when_tensioned() {
        let mut d = default_drive();
        d.state.tension = 100.0;
        assert!(spool_torque(&d.params, &d.state) > 0.0);
    }

    #[test]
    fn test_is_cable_slack_at_rest() {
        let d = default_drive();
        assert!(is_cable_slack(&d.state));
    }

    #[test]
    fn test_step_updates_load_position() {
        let mut d = default_drive();
        let before = d.state.load_position;
        step_cable_drive(&d.params, &mut d.state, 1.0, 0.1);
        assert!((d.state.load_position - (before + 0.1)).abs() < 1e-5);
    }
}
