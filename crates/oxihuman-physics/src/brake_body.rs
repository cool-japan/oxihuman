// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Friction brake with clamping force and slip velocity model.

#![allow(dead_code)]

/// A friction brake.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrakeBody {
    /// Maximum clamping force in Newtons.
    pub clamp_force: f32,
    /// Coefficient of kinetic friction.
    pub mu_k: f32,
    /// Coefficient of static friction.
    pub mu_s: f32,
    /// Effective brake radius in meters.
    pub radius: f32,
    /// Current slip velocity (rad/s of disc).
    pub slip_velocity: f32,
    /// Engaged flag.
    pub engaged: bool,
    /// Accumulated heat (proportional to energy dissipated).
    pub heat: f32,
}

/// Create a new brake body.
#[allow(dead_code)]
pub fn new_brake(clamp_force: f32, mu_k: f32, mu_s: f32, radius: f32) -> BrakeBody {
    BrakeBody {
        clamp_force: clamp_force.abs(),
        mu_k,
        mu_s,
        radius: radius.abs(),
        slip_velocity: 0.0,
        engaged: false,
        heat: 0.0,
    }
}

/// Engage or disengage the brake.
#[allow(dead_code)]
pub fn brake_set_engaged(brake: &mut BrakeBody, engaged: bool) {
    brake.engaged = engaged;
}

/// Compute brake torque for a given disc angular velocity.
/// Returns the braking torque (magnitude, opposing rotation).
#[allow(dead_code)]
pub fn brake_torque(brake: &BrakeBody, omega: f32) -> f32 {
    if !brake.engaged {
        return 0.0;
    }
    let mu = if omega.abs() < 1e-4 {
        brake.mu_s
    } else {
        brake.mu_k
    };
    brake.clamp_force * mu * brake.radius
}

/// Step the brake: update slip velocity and heat given disc omega.
#[allow(dead_code)]
pub fn brake_step(brake: &mut BrakeBody, omega: f32, inertia: f32, dt: f32) {
    if !brake.engaged || inertia < 1e-10 {
        brake.slip_velocity = omega;
        return;
    }
    let torque = brake_torque(brake, omega);
    let alpha = -torque * omega.signum() / inertia;
    let new_omega = omega + alpha * dt;
    brake.slip_velocity = if omega * new_omega < 0.0 {
        0.0
    } else {
        new_omega
    };
    let power = torque * omega.abs();
    brake.heat += power * dt;
}

/// Power dissipated by braking.
#[allow(dead_code)]
pub fn brake_power(brake: &BrakeBody, omega: f32) -> f32 {
    brake_torque(brake, omega) * omega.abs()
}

/// Reset brake heat (cooling).
#[allow(dead_code)]
pub fn brake_cool(brake: &mut BrakeBody, amount: f32) {
    brake.heat = (brake.heat - amount.abs()).max(0.0);
}

/// Reset brake state.
#[allow(dead_code)]
pub fn brake_reset(brake: &mut BrakeBody) {
    brake.slip_velocity = 0.0;
    brake.heat = 0.0;
    brake.engaged = false;
}

/// Is the disc locked (slip velocity ≈ 0 while engaged)?
#[allow(dead_code)]
pub fn brake_is_locked(brake: &BrakeBody) -> bool {
    brake.engaged && brake.slip_velocity.abs() < 1e-4
}

/// Set the clamping force.
#[allow(dead_code)]
pub fn brake_set_clamp(brake: &mut BrakeBody, clamp: f32) {
    brake.clamp_force = clamp.abs();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_brake() -> BrakeBody {
        new_brake(1000.0, 0.4, 0.5, 0.15)
    }

    #[test]
    fn test_initial_state() {
        let b = make_brake();
        assert!(!b.engaged);
        assert_eq!(b.heat, 0.0);
    }

    #[test]
    fn test_no_torque_when_disengaged() {
        let b = make_brake();
        assert_eq!(brake_torque(&b, 10.0), 0.0);
    }

    #[test]
    fn test_torque_when_engaged() {
        let mut b = make_brake();
        brake_set_engaged(&mut b, true);
        assert!(brake_torque(&b, 10.0) > 0.0);
    }

    #[test]
    fn test_torque_uses_static_mu_at_zero() {
        let mut b = make_brake();
        brake_set_engaged(&mut b, true);
        let ts = brake_torque(&b, 0.0);
        let tk = brake_torque(&b, 1.0);
        assert!(ts >= tk);
    }

    #[test]
    fn test_heat_accumulates() {
        let mut b = make_brake();
        brake_set_engaged(&mut b, true);
        brake_step(&mut b, 10.0, 1.0, 0.1);
        assert!(b.heat > 0.0);
    }

    #[test]
    fn test_cooling() {
        let mut b = make_brake();
        brake_set_engaged(&mut b, true);
        brake_step(&mut b, 10.0, 1.0, 1.0);
        let heat_before = b.heat;
        brake_cool(&mut b, heat_before * 0.5);
        assert!(b.heat < heat_before);
    }

    #[test]
    fn test_reset() {
        let mut b = make_brake();
        brake_set_engaged(&mut b, true);
        b.heat = 100.0;
        brake_reset(&mut b);
        assert!(!b.engaged);
        assert_eq!(b.heat, 0.0);
    }

    #[test]
    fn test_power() {
        let mut b = make_brake();
        brake_set_engaged(&mut b, true);
        let p = brake_power(&b, 10.0);
        assert!(p > 0.0);
    }

    #[test]
    fn test_set_clamp() {
        let mut b = make_brake();
        brake_set_clamp(&mut b, 2000.0);
        assert_eq!(b.clamp_force, 2000.0);
    }

    #[test]
    fn test_locked_when_stopped() {
        let mut b = make_brake();
        brake_set_engaged(&mut b, true);
        b.slip_velocity = 0.0;
        assert!(brake_is_locked(&b));
    }
}
