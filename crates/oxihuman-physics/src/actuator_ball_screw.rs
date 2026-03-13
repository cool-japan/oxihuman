// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ball screw linear actuator stub.

/// Ball screw parameters.
#[derive(Clone, Debug)]
pub struct BallScrewParams {
    /// Lead (linear advance per full revolution, m).
    pub lead: f32,
    /// Efficiency (0–1).
    pub efficiency: f32,
    /// Maximum axial load (N).
    pub max_axial_load: f32,
    /// Dynamic load rating (N).
    pub dynamic_load_rating: f32,
    /// Nut mass (kg).
    pub nut_mass: f32,
    /// Viscous friction coefficient (N·s/m).
    pub friction: f32,
}

impl Default for BallScrewParams {
    fn default() -> Self {
        Self {
            lead: 0.005, /* 5 mm lead */
            efficiency: 0.90,
            max_axial_load: 10_000.0,
            dynamic_load_rating: 15_000.0,
            nut_mass: 0.5,
            friction: 20.0,
        }
    }
}

/// Ball screw state.
#[derive(Clone, Debug, Default)]
pub struct BallScrewState {
    /// Motor angular velocity (rad/s).
    pub motor_omega: f32,
    /// Nut linear position (m).
    pub position: f32,
    /// Nut linear velocity (m/s).
    pub velocity: f32,
    /// Applied motor torque (N·m).
    pub motor_torque: f32,
    /// Resulting axial force (N).
    pub axial_force: f32,
}

/// Converts motor angular velocity to nut linear velocity.
pub fn omega_to_linear_velocity(params: &BallScrewParams, omega: f32) -> f32 {
    omega * params.lead / (2.0 * std::f32::consts::PI)
}

/// Converts motor torque to axial force (accounting for efficiency).
pub fn torque_to_axial_force(params: &BallScrewParams, torque: f32) -> f32 {
    let force = torque * 2.0 * std::f32::consts::PI * params.efficiency / params.lead;
    force.clamp(-params.max_axial_load, params.max_axial_load)
}

/// Converts axial force to required motor torque.
pub fn axial_force_to_torque(params: &BallScrewParams, force: f32) -> f32 {
    force * params.lead / (2.0 * std::f32::consts::PI * params.efficiency)
}

/// Steps the ball screw state forward by dt seconds.
pub fn step_ball_screw(
    params: &BallScrewParams,
    state: &mut BallScrewState,
    load_force: f32,
    dt: f32,
) {
    let axial_force = torque_to_axial_force(params, state.motor_torque);
    let net_force = axial_force - load_force - params.friction * state.velocity;
    state.velocity += (net_force / params.nut_mass) * dt;
    state.position += state.velocity * dt;
    state.motor_omega = state.velocity * 2.0 * std::f32::consts::PI / params.lead;
    state.axial_force = axial_force;
}

/// Returns true if the applied load is within rated capacity.
pub fn load_within_rating(params: &BallScrewParams, axial_load: f32) -> bool {
    axial_load.abs() <= params.dynamic_load_rating
}

/// Returns the mechanical advantage (force amplification).
pub fn mechanical_advantage(params: &BallScrewParams) -> f32 {
    2.0 * std::f32::consts::PI * params.efficiency / params.lead
}

/// Ball screw actuator stub struct.
pub struct BallScrewActuator {
    pub params: BallScrewParams,
    pub state: BallScrewState,
}

impl BallScrewActuator {
    /// Creates a new ball screw actuator with default params.
    pub fn new(params: BallScrewParams) -> Self {
        Self {
            state: BallScrewState::default(),
            params,
        }
    }

    /// Applies motor torque and steps simulation.
    pub fn apply_torque(&mut self, torque: f32, load_force: f32, dt: f32) {
        self.state.motor_torque = torque;
        step_ball_screw(&self.params, &mut self.state, load_force, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_bs() -> BallScrewActuator {
        BallScrewActuator::new(BallScrewParams::default())
    }

    #[test]
    fn test_omega_to_linear_velocity_positive() {
        let p = BallScrewParams::default();
        let v = omega_to_linear_velocity(&p, 10.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_torque_to_force_positive() {
        let p = BallScrewParams::default();
        let f = torque_to_axial_force(&p, 1.0);
        assert!(f > 0.0);
    }

    #[test]
    fn test_axial_force_clamped_to_max() {
        let p = BallScrewParams::default();
        let f = torque_to_axial_force(&p, 1_000_000.0);
        assert!(f <= p.max_axial_load);
    }

    #[test]
    fn test_force_torque_roundtrip() {
        let p = BallScrewParams::default();
        let torque = 5.0_f32;
        let force = torque_to_axial_force(&p, torque);
        let back_torque = axial_force_to_torque(&p, force);
        assert!((back_torque - torque).abs() < 1e-3);
    }

    #[test]
    fn test_motor_accelerates_nut() {
        let mut bs = default_bs();
        bs.apply_torque(2.0, 0.0, 0.05);
        assert!(bs.state.velocity > 0.0);
    }

    #[test]
    fn test_position_increases_with_positive_torque() {
        let mut bs = default_bs();
        for _ in 0..20 {
            bs.apply_torque(2.0, 0.0, 0.01);
        }
        assert!(bs.state.position > 0.0);
    }

    #[test]
    fn test_mechanical_advantage_positive() {
        let p = BallScrewParams::default();
        assert!(mechanical_advantage(&p) > 0.0);
    }

    #[test]
    fn test_load_within_rating() {
        let p = BallScrewParams::default();
        assert!(load_within_rating(&p, 10_000.0));
        assert!(!load_within_rating(&p, 20_000.0));
    }

    #[test]
    fn test_negative_torque_negative_velocity() {
        let mut bs = default_bs();
        for _ in 0..20 {
            bs.apply_torque(-2.0, 0.0, 0.01);
        }
        assert!(bs.state.velocity < 0.0);
    }
}
