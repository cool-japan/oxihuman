// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Linear motor thrust model stub — voice-coil / linear induction motor.

/// Linear motor parameters.
#[derive(Clone, Debug)]
pub struct LinearMotorParams {
    /// Force constant (N/A).
    pub force_constant: f32,
    /// Back-EMF constant (V·s/m).
    pub back_emf_constant: f32,
    /// Coil resistance (Ohm).
    pub resistance: f32,
    /// Moving mass (kg).
    pub mover_mass: f32,
    /// Viscous friction (N·s/m).
    pub damping: f32,
    /// Travel limit in each direction (m).
    pub travel_limit: f32,
}

impl Default for LinearMotorParams {
    fn default() -> Self {
        Self {
            force_constant: 50.0,
            back_emf_constant: 0.05,
            resistance: 2.0,
            mover_mass: 0.5,
            damping: 5.0,
            travel_limit: 0.1,
        }
    }
}

/// Linear motor state.
#[derive(Clone, Debug, Default)]
pub struct LinearMotorState {
    /// Position (m).
    pub position: f32,
    /// Velocity (m/s).
    pub velocity: f32,
    /// Applied voltage (V).
    pub voltage: f32,
    /// Coil current (A).
    pub current: f32,
}

/// Computes coil current given voltage and mover velocity.
pub fn compute_linear_current(params: &LinearMotorParams, voltage: f32, velocity: f32) -> f32 {
    (voltage - params.back_emf_constant * velocity) / params.resistance
}

/// Computes the thrust force produced by the motor.
pub fn compute_thrust(params: &LinearMotorParams, current: f32) -> f32 {
    params.force_constant * current
}

/// Steps the linear motor by dt seconds under a given load.
pub fn step_linear_motor(
    params: &LinearMotorParams,
    state: &mut LinearMotorState,
    load_force: f32,
    dt: f32,
) {
    let current = compute_linear_current(params, state.voltage, state.velocity);
    let thrust = compute_thrust(params, current);
    let net_force = thrust - load_force - params.damping * state.velocity;
    state.velocity += (net_force / params.mover_mass) * dt;
    state.position =
        (state.position + state.velocity * dt).clamp(-params.travel_limit, params.travel_limit);
    state.current = current;
}

/// Returns the peak thrust at the given voltage (N).
pub fn peak_thrust(params: &LinearMotorParams, voltage: f32) -> f32 {
    params.force_constant * voltage / params.resistance
}

/// Returns the no-load velocity at the given voltage (m/s).
pub fn no_load_velocity(params: &LinearMotorParams, voltage: f32) -> f32 {
    if params.back_emf_constant.abs() < 1e-9 {
        0.0
    } else {
        voltage / params.back_emf_constant
    }
}

/// Returns the power dissipated in the coil (W).
pub fn coil_power_dissipation(params: &LinearMotorParams, state: &LinearMotorState) -> f32 {
    state.current * state.current * params.resistance
}

/// Linear motor stub struct.
pub struct LinearMotor {
    pub params: LinearMotorParams,
    pub state: LinearMotorState,
}

impl LinearMotor {
    /// Creates a new linear motor with given params.
    pub fn new(params: LinearMotorParams) -> Self {
        Self {
            state: LinearMotorState::default(),
            params,
        }
    }

    /// Applies voltage and steps simulation.
    pub fn apply_voltage(&mut self, voltage: f32, load_force: f32, dt: f32) {
        self.state.voltage = voltage;
        step_linear_motor(&self.params, &mut self.state, load_force, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_lm() -> LinearMotor {
        LinearMotor::new(LinearMotorParams::default())
    }

    #[test]
    fn test_thrust_positive_for_positive_current() {
        let p = LinearMotorParams::default();
        assert!(compute_thrust(&p, 5.0) > 0.0);
    }

    #[test]
    fn test_current_at_zero_velocity() {
        let p = LinearMotorParams::default();
        let i = compute_linear_current(&p, 10.0, 0.0);
        assert!((i - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_motor_accelerates_from_rest() {
        let mut m = default_lm();
        m.apply_voltage(20.0, 0.0, 0.01);
        assert!(m.state.velocity > 0.0);
    }

    #[test]
    fn test_position_clamped_to_travel_limit() {
        let mut m = default_lm();
        for _ in 0..500 {
            m.apply_voltage(100.0, 0.0, 0.01);
        }
        assert!(m.state.position <= m.params.travel_limit + 1e-5);
    }

    #[test]
    fn test_peak_thrust_proportional_to_voltage() {
        let p = LinearMotorParams::default();
        let t1 = peak_thrust(&p, 10.0);
        let t2 = peak_thrust(&p, 20.0);
        assert!((t2 - 2.0 * t1).abs() < 1e-4);
    }

    #[test]
    fn test_no_load_velocity_zero_bemf_returns_zero() {
        let p = LinearMotorParams {
            back_emf_constant: 0.0,
            ..LinearMotorParams::default()
        };
        assert_eq!(no_load_velocity(&p, 12.0), 0.0);
    }

    #[test]
    fn test_coil_dissipation_zero_at_rest() {
        let m = default_lm();
        assert!((coil_power_dissipation(&m.params, &m.state)).abs() < 1e-6);
    }

    #[test]
    fn test_negative_voltage_moves_negative() {
        let mut m = default_lm();
        m.apply_voltage(-20.0, 0.0, 0.1);
        assert!(m.state.velocity < 0.0);
    }

    #[test]
    fn test_position_negative_for_negative_command() {
        let mut m = default_lm();
        for _ in 0..100 {
            m.apply_voltage(-20.0, 0.0, 0.01);
        }
        assert!(m.state.position < 0.0);
    }
}
