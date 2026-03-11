// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rack-and-pinion actuator stub — converts rotary to linear motion.

/// Rack-and-pinion parameters.
#[derive(Clone, Debug)]
pub struct RackPinionParams {
    /// Pinion pitch radius (m).
    pub pitch_radius: f32,
    /// Rack length (m).
    pub rack_length: f32,
    /// Efficiency (0–1).
    pub efficiency: f32,
    /// Rack mass (kg).
    pub rack_mass: f32,
    /// Viscous friction coefficient (N·s/m).
    pub friction: f32,
}

impl Default for RackPinionParams {
    fn default() -> Self {
        Self {
            pitch_radius: 0.02,
            rack_length: 0.5,
            efficiency: 0.92,
            rack_mass: 1.0,
            friction: 15.0,
        }
    }
}

/// Rack-and-pinion state.
#[derive(Clone, Debug, Default)]
pub struct RackPinionState {
    /// Pinion angular velocity (rad/s).
    pub pinion_omega: f32,
    /// Rack linear position (m).
    pub rack_position: f32,
    /// Rack linear velocity (m/s).
    pub rack_velocity: f32,
    /// Pinion torque (N·m).
    pub pinion_torque: f32,
}

/// Converts pinion angular velocity to rack linear velocity.
pub fn omega_to_rack_velocity(params: &RackPinionParams, omega: f32) -> f32 {
    omega * params.pitch_radius
}

/// Converts pinion torque to rack force (with efficiency).
pub fn torque_to_rack_force(params: &RackPinionParams, torque: f32) -> f32 {
    torque * params.efficiency / params.pitch_radius
}

/// Converts rack force to required pinion torque.
pub fn rack_force_to_torque(params: &RackPinionParams, force: f32) -> f32 {
    force * params.pitch_radius / params.efficiency
}

/// Steps the rack-and-pinion by dt seconds under a given external load.
pub fn step_rack_pinion(
    params: &RackPinionParams,
    state: &mut RackPinionState,
    external_load: f32,
    dt: f32,
) {
    let rack_force = torque_to_rack_force(params, state.pinion_torque);
    let net_force = rack_force - external_load - params.friction * state.rack_velocity;
    let accel = net_force / params.rack_mass;
    state.rack_velocity += accel * dt;
    state.rack_position =
        (state.rack_position + state.rack_velocity * dt).clamp(0.0, params.rack_length);
    state.pinion_omega = state.rack_velocity / params.pitch_radius;
}

/// Returns the rack's travel ratio (0 = start, 1 = end of rack).
pub fn rack_travel_ratio(params: &RackPinionParams, state: &RackPinionState) -> f32 {
    (state.rack_position / params.rack_length).clamp(0.0, 1.0)
}

/// Returns the mechanical advantage (force per unit torque).
pub fn mechanical_advantage(params: &RackPinionParams) -> f32 {
    params.efficiency / params.pitch_radius
}

/// Rack-and-pinion actuator stub struct.
pub struct RackPinionActuator {
    pub params: RackPinionParams,
    pub state: RackPinionState,
}

impl RackPinionActuator {
    /// Creates a new rack-and-pinion actuator with default params.
    pub fn new(params: RackPinionParams) -> Self {
        Self {
            state: RackPinionState::default(),
            params,
        }
    }

    /// Applies pinion torque and steps simulation.
    pub fn apply_torque(&mut self, torque: f32, load: f32, dt: f32) {
        self.state.pinion_torque = torque;
        step_rack_pinion(&self.params, &mut self.state, load, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_rp() -> RackPinionActuator {
        RackPinionActuator::new(RackPinionParams::default())
    }

    #[test]
    fn test_omega_to_rack_velocity_positive() {
        let p = RackPinionParams::default();
        assert!(omega_to_rack_velocity(&p, 10.0) > 0.0);
    }

    #[test]
    fn test_torque_to_force_positive() {
        let p = RackPinionParams::default();
        assert!(torque_to_rack_force(&p, 1.0) > 0.0);
    }

    #[test]
    fn test_force_torque_roundtrip() {
        let p = RackPinionParams::default();
        let t = 2.0_f32;
        let f = torque_to_rack_force(&p, t);
        let t2 = rack_force_to_torque(&p, f);
        assert!((t2 - t).abs() < 1e-5);
    }

    #[test]
    fn test_rack_moves_with_positive_torque() {
        let mut rp = default_rp();
        rp.apply_torque(5.0, 0.0, 0.05);
        assert!(rp.state.rack_position > 0.0);
    }

    #[test]
    fn test_rack_position_clamped_to_length() {
        let mut rp = default_rp();
        for _ in 0..500 {
            rp.apply_torque(100.0, 0.0, 0.01);
        }
        assert!(rp.state.rack_position <= rp.params.rack_length + 1e-5);
    }

    #[test]
    fn test_rack_travel_ratio_bounded() {
        let rp = default_rp();
        let r = rack_travel_ratio(&rp.params, &rp.state);
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn test_mechanical_advantage_positive() {
        let p = RackPinionParams::default();
        assert!(mechanical_advantage(&p) > 0.0);
    }

    #[test]
    fn test_negative_torque_negative_velocity() {
        let mut rp = RackPinionActuator::new(RackPinionParams {
            rack_length: 10.0,
            ..Default::default()
        });
        rp.state.rack_position = 5.0; /* start in the middle */
        for _ in 0..20 {
            rp.apply_torque(-5.0, 0.0, 0.01);
        }
        assert!(rp.state.rack_velocity < 0.0);
    }

    #[test]
    fn test_pinion_omega_updated_after_step() {
        let mut rp = default_rp();
        rp.apply_torque(5.0, 0.0, 0.05);
        assert!(rp.state.pinion_omega > 0.0);
    }
}
