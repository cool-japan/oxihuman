// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! DC motor torque/speed model stub.

/// DC motor parameters.
#[derive(Clone, Debug)]
pub struct DcMotorParams {
    /// Motor resistance (ohms).
    pub resistance: f32,
    /// Back-EMF constant (V·s/rad).
    pub ke: f32,
    /// Torque constant (N·m/A).
    pub kt: f32,
    /// Rotor inertia (kg·m²).
    pub inertia: f32,
    /// Viscous friction coefficient.
    pub damping: f32,
}

impl Default for DcMotorParams {
    fn default() -> Self {
        Self {
            resistance: 1.0,
            ke: 0.05,
            kt: 0.05,
            inertia: 0.001,
            damping: 0.001,
        }
    }
}

/// DC motor state.
#[derive(Clone, Debug, Default)]
pub struct DcMotorState {
    /// Angular velocity (rad/s).
    pub omega: f32,
    /// Armature current (A).
    pub current: f32,
    /// Applied voltage (V).
    pub voltage: f32,
}

/// Computes the armature current given voltage and back-EMF.
pub fn compute_current(params: &DcMotorParams, voltage: f32, omega: f32) -> f32 {
    (voltage - params.ke * omega) / params.resistance
}

/// Computes the electromagnetic torque produced by the motor.
pub fn compute_torque(params: &DcMotorParams, current: f32) -> f32 {
    params.kt * current
}

/// Steps the motor state forward by dt seconds under the given load torque.
pub fn step_motor(params: &DcMotorParams, state: &mut DcMotorState, load_torque: f32, dt: f32) {
    let current = compute_current(params, state.voltage, state.omega);
    let torque = compute_torque(params, current);
    let net_torque = torque - load_torque - params.damping * state.omega;
    state.omega += (net_torque / params.inertia) * dt;
    state.current = current;
}

/// Returns the motor's no-load speed at the given voltage (rad/s).
pub fn no_load_speed(params: &DcMotorParams, voltage: f32) -> f32 {
    if params.ke.abs() < 1e-9 {
        0.0
    } else {
        voltage / params.ke
    }
}

/// Returns the stall torque at the given voltage.
pub fn stall_torque(params: &DcMotorParams, voltage: f32) -> f32 {
    params.kt * voltage / params.resistance
}

/// Clamps voltage to the motor's rated range.
pub fn clamp_voltage(voltage: f32, max_voltage: f32) -> f32 {
    voltage.clamp(-max_voltage, max_voltage)
}

/// DC motor stub struct.
pub struct DcMotor {
    pub params: DcMotorParams,
    pub state: DcMotorState,
}

impl DcMotor {
    /// Creates a new DC motor with default parameters.
    pub fn new(params: DcMotorParams) -> Self {
        Self {
            params,
            state: DcMotorState::default(),
        }
    }

    /// Applies voltage and steps simulation by dt.
    pub fn apply_voltage(&mut self, voltage: f32, load_torque: f32, dt: f32) {
        self.state.voltage = voltage;
        step_motor(&self.params, &mut self.state, load_torque, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_motor() -> DcMotor {
        DcMotor::new(DcMotorParams::default())
    }

    #[test]
    fn test_compute_current_zero_omega() {
        let p = DcMotorParams::default();
        /* at zero speed, current = V/R */
        let i = compute_current(&p, 12.0, 0.0);
        assert!((i - 12.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_torque_positive() {
        let p = DcMotorParams::default();
        let torque = compute_torque(&p, 10.0);
        assert!(torque > 0.0);
    }

    #[test]
    fn test_no_load_speed_proportional_to_voltage() {
        let p = DcMotorParams::default();
        let s1 = no_load_speed(&p, 6.0);
        let s2 = no_load_speed(&p, 12.0);
        assert!((s2 - 2.0 * s1).abs() < 1e-4);
    }

    #[test]
    fn test_stall_torque_positive() {
        let p = DcMotorParams::default();
        assert!(stall_torque(&p, 12.0) > 0.0);
    }

    #[test]
    fn test_clamp_voltage() {
        assert_eq!(clamp_voltage(15.0, 12.0), 12.0);
        assert_eq!(clamp_voltage(-15.0, 12.0), -12.0);
        assert_eq!(clamp_voltage(5.0, 12.0), 5.0);
    }

    #[test]
    fn test_motor_accelerates_from_rest() {
        let mut m = default_motor();
        m.apply_voltage(12.0, 0.0, 0.01);
        assert!(m.state.omega > 0.0);
    }

    #[test]
    fn test_motor_step_increases_omega_over_time() {
        let mut m = default_motor();
        for _ in 0..100 {
            m.apply_voltage(12.0, 0.0, 0.01);
        }
        let omega_100 = m.state.omega;
        assert!(omega_100 > 0.0);
    }

    #[test]
    fn test_zero_voltage_decelerates_motor() {
        let params = DcMotorParams::default();
        let mut state = DcMotorState {
            omega: 100.0,
            current: 0.0,
            voltage: 0.0,
        };
        step_motor(&params, &mut state, 0.0, 0.1);
        /* back-EMF + damping should reduce speed */
        assert!(state.omega < 100.0);
    }

    #[test]
    fn test_no_load_speed_zero_ke_returns_zero() {
        let p = DcMotorParams {
            ke: 0.0,
            ..DcMotorParams::default()
        };
        assert_eq!(no_load_speed(&p, 12.0), 0.0);
    }
}
