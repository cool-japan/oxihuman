// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! PID motor controller for joint drives.

// ─── Structures ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct PidParams {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub integral_limit: f32,
    pub output_limit: f32,
}

#[allow(dead_code)]
pub struct PidState {
    pub integral: f32,
    pub prev_error: f32,
    pub output: f32,
}

#[allow(dead_code)]
pub struct MotorController {
    pub id: u32,
    pub name: String,
    pub params: PidParams,
    pub state: PidState,
    pub target: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub struct MotorBank {
    pub motors: Vec<MotorController>,
    pub next_id: u32,
}

// ─── PID helpers ─────────────────────────────────────────────────────────────

fn clamp(value: f32, limit: f32) -> f32 {
    value.clamp(-limit, limit)
}

// ─── Functions ───────────────────────────────────────────────────────────────

/// Return sensible default PID parameters.
#[allow(dead_code)]
pub fn default_pid_params() -> PidParams {
    PidParams {
        kp: 1.0,
        ki: 0.0,
        kd: 0.0,
        integral_limit: 100.0,
        output_limit: 100.0,
    }
}

/// Create a new motor controller.
#[allow(dead_code)]
pub fn new_motor(name: &str, params: PidParams) -> MotorController {
    MotorController {
        id: 0,
        name: name.to_string(),
        params,
        state: PidState {
            integral: 0.0,
            prev_error: 0.0,
            output: 0.0,
        },
        target: 0.0,
        enabled: true,
    }
}

/// Run one PID update step. Returns the new output.
#[allow(dead_code)]
pub fn pid_update(
    params: &PidParams,
    state: &mut PidState,
    setpoint: f32,
    measured: f32,
    dt: f32,
) -> f32 {
    let error = setpoint - measured;

    // Integral with anti-windup clamping.
    state.integral = clamp(state.integral + error * dt, params.integral_limit);

    // Derivative (backward difference).
    let derivative = if dt > 1e-12 {
        (error - state.prev_error) / dt
    } else {
        0.0
    };

    state.prev_error = error;

    let raw_output = params.kp * error + params.ki * state.integral + params.kd * derivative;
    let output = clamp(raw_output, params.output_limit);
    state.output = output;
    output
}

/// Reset PID integrator and previous-error to zero.
#[allow(dead_code)]
pub fn reset_pid(state: &mut PidState) {
    state.integral = 0.0;
    state.prev_error = 0.0;
    state.output = 0.0;
}

/// Set the target (setpoint) of a motor.
#[allow(dead_code)]
pub fn set_motor_target(motor: &mut MotorController, target: f32) {
    motor.target = target;
}

/// Update a motor: run PID from its current target and measured value.
/// Returns the controller output (torque / force).
#[allow(dead_code)]
pub fn motor_update(motor: &mut MotorController, measured: f32, dt: f32) -> f32 {
    if !motor.enabled {
        return 0.0;
    }
    pid_update(&motor.params, &mut motor.state, motor.target, measured, dt)
}

/// Create an empty motor bank.
#[allow(dead_code)]
pub fn new_motor_bank() -> MotorBank {
    MotorBank {
        motors: Vec::new(),
        next_id: 0,
    }
}

/// Add a motor to the bank; returns its assigned id.
#[allow(dead_code)]
pub fn add_motor_to_bank(bank: &mut MotorBank, name: &str, params: PidParams) -> u32 {
    let id = bank.next_id;
    bank.next_id += 1;
    let mut motor = new_motor(name, params);
    motor.id = id;
    bank.motors.push(motor);
    id
}

/// Update all enabled motors with their respective measurements.
/// `measurements` length must equal `bank.motors.len()`.
#[allow(dead_code)]
pub fn update_all_motors(bank: &mut MotorBank, measurements: &[f32], dt: f32) -> Vec<f32> {
    let mut outputs = vec![0.0_f32; bank.motors.len()];
    for (i, motor) in bank.motors.iter_mut().enumerate() {
        let measured = measurements.get(i).copied().unwrap_or(0.0);
        outputs[i] = motor_update(motor, measured, dt);
    }
    outputs
}

/// Look up a motor by id; returns `None` if not found.
#[allow(dead_code)]
pub fn get_motor(bank: &MotorBank, id: u32) -> Option<&MotorController> {
    bank.motors.iter().find(|m| m.id == id)
}

/// Enable a motor by id.
#[allow(dead_code)]
pub fn enable_motor(bank: &mut MotorBank, id: u32) {
    for motor in bank.motors.iter_mut() {
        if motor.id == id {
            motor.enabled = true;
        }
    }
}

/// Disable a motor by id.
#[allow(dead_code)]
pub fn disable_motor(bank: &mut MotorBank, id: u32) {
    for motor in bank.motors.iter_mut() {
        if motor.id == id {
            motor.enabled = false;
        }
    }
}

/// Number of motors in the bank.
#[allow(dead_code)]
pub fn motor_count(bank: &MotorBank) -> usize {
    bank.motors.len()
}

/// Simple proportional controller: returns `kp * (setpoint − measured)`.
#[allow(dead_code)]
pub fn proportional_controller(kp: f32, setpoint: f32, measured: f32) -> f32 {
    kp * (setpoint - measured)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pid_params() {
        let p = default_pid_params();
        assert!(p.kp > 0.0);
        assert!(p.integral_limit > 0.0);
        assert!(p.output_limit > 0.0);
    }

    #[test]
    fn test_pid_update_proportional() {
        let params = PidParams {
            kp: 2.0,
            ki: 0.0,
            kd: 0.0,
            integral_limit: 1000.0,
            output_limit: 1000.0,
        };
        let mut state = PidState {
            integral: 0.0,
            prev_error: 0.0,
            output: 0.0,
        };
        let output = pid_update(&params, &mut state, 10.0, 0.0, 0.1);
        // error = 10, kp=2 → output = 20.
        assert!((output - 20.0).abs() < 1e-5, "output={}", output);
    }

    #[test]
    fn test_pid_update_output_limit() {
        let params = PidParams {
            kp: 1000.0,
            ki: 0.0,
            kd: 0.0,
            integral_limit: 1000.0,
            output_limit: 50.0,
        };
        let mut state = PidState {
            integral: 0.0,
            prev_error: 0.0,
            output: 0.0,
        };
        let output = pid_update(&params, &mut state, 1.0, 0.0, 0.1);
        assert!(output <= 50.0, "output={}", output);
    }

    #[test]
    fn test_reset_pid() {
        let mut state = PidState {
            integral: 99.0,
            prev_error: -5.0,
            output: 42.0,
        };
        reset_pid(&mut state);
        assert_eq!(state.integral, 0.0);
        assert_eq!(state.prev_error, 0.0);
        assert_eq!(state.output, 0.0);
    }

    #[test]
    fn test_set_motor_target() {
        let mut motor = new_motor("test", default_pid_params());
        set_motor_target(&mut motor, 2.78);
        assert!((motor.target - 2.78).abs() < 1e-6);
    }

    #[test]
    fn test_motor_update_enabled() {
        let params = PidParams {
            kp: 1.0,
            ki: 0.0,
            kd: 0.0,
            integral_limit: 100.0,
            output_limit: 100.0,
        };
        let mut motor = new_motor("m", params);
        set_motor_target(&mut motor, 5.0);
        let output = motor_update(&mut motor, 0.0, 0.1);
        // error=5, kp=1 → 5.0
        assert!((output - 5.0).abs() < 1e-5, "output={}", output);
    }

    #[test]
    fn test_motor_update_disabled() {
        let mut motor = new_motor("m", default_pid_params());
        motor.enabled = false;
        set_motor_target(&mut motor, 100.0);
        let output = motor_update(&mut motor, 0.0, 0.1);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_new_motor_bank_empty() {
        let bank = new_motor_bank();
        assert_eq!(motor_count(&bank), 0);
    }

    #[test]
    fn test_add_motor_to_bank() {
        let mut bank = new_motor_bank();
        add_motor_to_bank(&mut bank, "hip", default_pid_params());
        add_motor_to_bank(&mut bank, "knee", default_pid_params());
        assert_eq!(motor_count(&bank), 2);
    }

    #[test]
    fn test_motor_count() {
        let mut bank = new_motor_bank();
        for i in 0..6 {
            add_motor_to_bank(&mut bank, &format!("motor_{}", i), default_pid_params());
        }
        assert_eq!(motor_count(&bank), 6);
    }

    #[test]
    fn test_proportional_controller() {
        let out = proportional_controller(3.0, 10.0, 4.0);
        assert!((out - 18.0).abs() < 1e-5, "out={}", out);
    }

    #[test]
    fn test_update_all_motors() {
        let mut bank = new_motor_bank();
        let id0 = add_motor_to_bank(
            &mut bank,
            "a",
            PidParams {
                kp: 1.0,
                ki: 0.0,
                kd: 0.0,
                integral_limit: 100.0,
                output_limit: 100.0,
            },
        );
        let id1 = add_motor_to_bank(
            &mut bank,
            "b",
            PidParams {
                kp: 1.0,
                ki: 0.0,
                kd: 0.0,
                integral_limit: 100.0,
                output_limit: 100.0,
            },
        );
        bank.motors[id0 as usize].target = 10.0;
        bank.motors[id1 as usize].target = 5.0;
        let outputs = update_all_motors(&mut bank, &[0.0, 0.0], 0.1);
        assert!((outputs[id0 as usize] - 10.0).abs() < 1e-5);
        assert!((outputs[id1 as usize] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_enable_disable_motor() {
        let mut bank = new_motor_bank();
        let id = add_motor_to_bank(&mut bank, "m", default_pid_params());
        disable_motor(&mut bank, id);
        assert!(!bank.motors[0].enabled);
        enable_motor(&mut bank, id);
        assert!(bank.motors[0].enabled);
    }

    #[test]
    fn test_get_motor() {
        let mut bank = new_motor_bank();
        let id = add_motor_to_bank(&mut bank, "test_motor", default_pid_params());
        let motor = get_motor(&bank, id);
        assert!(motor.is_some());
        assert_eq!(motor.expect("should succeed").name, "test_motor");
    }
}
