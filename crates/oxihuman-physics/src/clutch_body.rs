// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Friction clutch with engagement and slip torque model.

#![allow(dead_code)]

/// State of the clutch engagement.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClutchState {
    /// Fully disengaged, no torque transfer.
    Disengaged,
    /// Slipping: partial torque transfer based on friction.
    Slipping,
    /// Locked: input and output rotate at same speed.
    Locked,
}

/// A friction clutch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClutchBody {
    /// Maximum static friction torque (N·m).
    pub torque_capacity: f32,
    /// Kinetic friction coefficient (relative to static).
    pub mu_k_ratio: f32,
    /// Engagement level (0.0 = disengaged, 1.0 = fully engaged).
    pub engagement: f32,
    /// Input shaft angular velocity (rad/s).
    pub omega_in: f32,
    /// Output shaft angular velocity (rad/s).
    pub omega_out: f32,
    /// Current clutch state.
    pub state: ClutchState,
    /// Heat accumulated (proportional to slip energy).
    pub heat: f32,
    /// Output shaft inertia (kg·m²).
    pub inertia_out: f32,
}

/// Create a new clutch body.
#[allow(dead_code)]
pub fn new_clutch(torque_capacity: f32, mu_k_ratio: f32, inertia_out: f32) -> ClutchBody {
    ClutchBody {
        torque_capacity: torque_capacity.abs(),
        mu_k_ratio: mu_k_ratio.clamp(0.0, 1.0),
        engagement: 0.0,
        omega_in: 0.0,
        omega_out: 0.0,
        state: ClutchState::Disengaged,
        heat: 0.0,
        inertia_out: inertia_out.max(1e-6),
    }
}

/// Set the engagement level (0.0..1.0).
#[allow(dead_code)]
pub fn clutch_set_engagement(cl: &mut ClutchBody, engagement: f32) {
    cl.engagement = engagement.clamp(0.0, 1.0);
    if cl.engagement < 1e-3 {
        cl.state = ClutchState::Disengaged;
    } else if cl.state == ClutchState::Disengaged {
        cl.state = ClutchState::Slipping;
    }
}

/// Compute the transmissible torque given engagement.
#[allow(dead_code)]
pub fn clutch_max_torque(cl: &ClutchBody) -> f32 {
    cl.torque_capacity * cl.engagement
}

/// Step the clutch simulation.
#[allow(dead_code)]
pub fn clutch_step(cl: &mut ClutchBody, omega_in: f32, torque_in: f32, dt: f32) {
    cl.omega_in = omega_in;
    let max_t = clutch_max_torque(cl);

    if cl.state == ClutchState::Disengaged || max_t < 1e-6 {
        cl.state = ClutchState::Disengaged;
        return;
    }

    let slip = cl.omega_in - cl.omega_out;
    let kinetic_torque = cl.mu_k_ratio * max_t;

    if slip.abs() < 0.01 && torque_in.abs() <= max_t {
        cl.state = ClutchState::Locked;
        cl.omega_out = cl.omega_in;
    } else {
        cl.state = ClutchState::Slipping;
        let transfer_torque = if slip > 0.0 {
            kinetic_torque
        } else {
            -kinetic_torque
        };
        let alpha_out = transfer_torque / cl.inertia_out;
        cl.omega_out += alpha_out * dt;
        cl.heat += kinetic_torque * slip.abs() * dt;
    }
}

/// Power transferred through the clutch.
#[allow(dead_code)]
pub fn clutch_power_transfer(cl: &ClutchBody) -> f32 {
    clutch_max_torque(cl) * cl.omega_out
}

/// Slip speed (rad/s).
#[allow(dead_code)]
pub fn clutch_slip_speed(cl: &ClutchBody) -> f32 {
    cl.omega_in - cl.omega_out
}

/// Is the clutch fully locked?
#[allow(dead_code)]
pub fn clutch_is_locked(cl: &ClutchBody) -> bool {
    cl.state == ClutchState::Locked
}

/// Is the clutch slipping?
#[allow(dead_code)]
pub fn clutch_is_slipping(cl: &ClutchBody) -> bool {
    cl.state == ClutchState::Slipping
}

/// Cool the clutch.
#[allow(dead_code)]
pub fn clutch_cool(cl: &mut ClutchBody, amount: f32) {
    cl.heat = (cl.heat - amount.abs()).max(0.0);
}

/// Reset the clutch state.
#[allow(dead_code)]
pub fn clutch_reset(cl: &mut ClutchBody) {
    cl.omega_in = 0.0;
    cl.omega_out = 0.0;
    cl.heat = 0.0;
    cl.engagement = 0.0;
    cl.state = ClutchState::Disengaged;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_clutch() -> ClutchBody {
        new_clutch(100.0, 0.8, 0.5)
    }

    #[test]
    fn test_initial_state() {
        let cl = make_clutch();
        assert_eq!(cl.state, ClutchState::Disengaged);
        assert_eq!(cl.engagement, 0.0);
    }

    #[test]
    fn test_max_torque_zero_at_zero_engagement() {
        let cl = make_clutch();
        assert_eq!(clutch_max_torque(&cl), 0.0);
    }

    #[test]
    fn test_max_torque_at_full_engagement() {
        let mut cl = make_clutch();
        clutch_set_engagement(&mut cl, 1.0);
        assert!((clutch_max_torque(&cl) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_no_power_when_disengaged() {
        let cl = make_clutch();
        assert_eq!(clutch_power_transfer(&cl), 0.0);
    }

    #[test]
    fn test_slipping_when_speed_diff() {
        let mut cl = make_clutch();
        clutch_set_engagement(&mut cl, 1.0);
        cl.omega_in = 100.0;
        cl.omega_out = 0.0;
        cl.state = ClutchState::Slipping;
        clutch_step(&mut cl, 100.0, 50.0, 0.1);
        assert!(cl.omega_out > 0.0);
    }

    #[test]
    fn test_heat_on_slip() {
        let mut cl = make_clutch();
        clutch_set_engagement(&mut cl, 1.0);
        cl.omega_in = 100.0;
        cl.state = ClutchState::Slipping;
        clutch_step(&mut cl, 100.0, 50.0, 0.1);
        assert!(cl.heat >= 0.0);
    }

    #[test]
    fn test_lock_when_speeds_match() {
        let mut cl = make_clutch();
        clutch_set_engagement(&mut cl, 1.0);
        cl.omega_in = 50.0;
        cl.omega_out = 50.0;
        clutch_step(&mut cl, 50.0, 5.0, 0.01);
        assert!(clutch_is_locked(&cl));
    }

    #[test]
    fn test_cool() {
        let mut cl = make_clutch();
        cl.heat = 100.0;
        clutch_cool(&mut cl, 50.0);
        assert!((cl.heat - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_reset() {
        let mut cl = make_clutch();
        clutch_set_engagement(&mut cl, 1.0);
        cl.omega_in = 50.0;
        cl.heat = 100.0;
        clutch_reset(&mut cl);
        assert_eq!(cl.state, ClutchState::Disengaged);
        assert_eq!(cl.heat, 0.0);
    }

    #[test]
    fn test_slip_speed() {
        let mut cl = make_clutch();
        cl.omega_in = 10.0;
        cl.omega_out = 5.0;
        assert!((clutch_slip_speed(&cl) - 5.0).abs() < 1e-5);
    }
}
