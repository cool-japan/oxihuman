// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Muscle body: Hill-type muscle model with activation dynamics.

use std::f32::consts::E;

/// Hill-type muscle body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MuscleBody {
    /// Optimal fiber length (m).
    pub l_opt: f32,
    /// Maximum isometric force (N).
    pub f_max: f32,
    /// Current activation level [0, 1].
    pub activation: f32,
    /// Current fiber length.
    pub fiber_length: f32,
    /// Current fiber velocity (m/s).
    pub fiber_velocity: f32,
    /// Pennation angle (rad).
    pub pennation: f32,
    /// Tendon slack length.
    pub tendon_slack: f32,
}

/// Create a new `MuscleBody`.
#[allow(dead_code)]
pub fn new_muscle_body(l_opt: f32, f_max: f32) -> MuscleBody {
    MuscleBody {
        l_opt: l_opt.max(1e-4),
        f_max: f_max.max(0.0),
        activation: 0.0,
        fiber_length: l_opt,
        fiber_velocity: 0.0,
        pennation: 0.0,
        tendon_slack: l_opt * 0.1,
    }
}

/// Force-length relationship (Gaussian).
#[allow(dead_code)]
pub fn muscle_fl(l: f32, l_opt: f32) -> f32 {
    let norm = l / l_opt.max(1e-9);
    let exponent = -4.0 * (norm - 1.0) * (norm - 1.0);
    E.powf(exponent)
}

/// Force-velocity relationship (Hill equation normalized).
#[allow(dead_code)]
pub fn muscle_fv(v: f32, v_max: f32) -> f32 {
    let v_max = v_max.max(1e-9);
    if v <= 0.0 {
        let a = 0.25;
        ((v_max - v) / (v_max + v / a)).clamp(0.0, 1.5)
    } else {
        ((v_max - v) / (v_max + v)).clamp(0.0, 1.5)
    }
}

/// Compute active force.
#[allow(dead_code)]
pub fn muscle_active_force(body: &MuscleBody) -> f32 {
    let v_max = body.l_opt * 10.0;
    let fl = muscle_fl(body.fiber_length, body.l_opt);
    let fv = muscle_fv(body.fiber_velocity, v_max);
    body.activation * body.f_max * fl * fv
}

/// Set activation (clamped to [0, 1]).
#[allow(dead_code)]
pub fn mb_set_activation(body: &mut MuscleBody, a: f32) {
    body.activation = a.clamp(0.0, 1.0);
}

/// Step activation dynamics (first-order rise/fall).
#[allow(dead_code)]
pub fn mb_step_activation(body: &mut MuscleBody, target: f32, dt: f32) {
    let tau = if target > body.activation { 0.01 } else { 0.04 };
    body.activation += (target - body.activation) * dt / tau;
    body.activation = body.activation.clamp(0.0, 1.0);
}

/// Update fiber kinematics.
#[allow(dead_code)]
pub fn mb_step_fiber(body: &mut MuscleBody, new_length: f32, dt: f32) {
    if dt > 1e-9 {
        body.fiber_velocity = (new_length - body.fiber_length) / dt;
    }
    body.fiber_length = new_length.max(1e-4);
}

/// Passive elastic force (exponential).
#[allow(dead_code)]
pub fn muscle_passive_force(body: &MuscleBody) -> f32 {
    let norm = body.fiber_length / body.l_opt.max(1e-9);
    if norm <= 1.0 {
        return 0.0;
    }
    body.f_max * 0.01 * (E.powf(10.0 * (norm - 1.0)) - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_muscle_body() {
        let mb = new_muscle_body(0.1, 1000.0);
        assert!((mb.l_opt - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_fl_at_optimal() {
        let fl = muscle_fl(0.1, 0.1);
        assert!((fl - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fl_decreases_off_optimal() {
        let fl_opt = muscle_fl(0.1, 0.1);
        let fl_off = muscle_fl(0.2, 0.1);
        assert!(fl_opt > fl_off);
    }

    #[test]
    fn test_activation_zero_gives_zero_force() {
        let mb = new_muscle_body(0.1, 1000.0);
        assert!((muscle_active_force(&mb)).abs() < 1e-6);
    }

    #[test]
    fn test_activation_one_gives_positive_force() {
        let mut mb = new_muscle_body(0.1, 1000.0);
        mb_set_activation(&mut mb, 1.0);
        assert!(muscle_active_force(&mb) > 0.0);
    }

    #[test]
    fn test_activation_clamp() {
        let mut mb = new_muscle_body(0.1, 1000.0);
        mb_set_activation(&mut mb, 2.0);
        assert!((mb.activation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_step_activation_rises() {
        let mut mb = new_muscle_body(0.1, 1000.0);
        mb_step_activation(&mut mb, 1.0, 0.1);
        assert!(mb.activation > 0.0);
    }

    #[test]
    fn test_passive_force_at_rest() {
        let mb = new_muscle_body(0.1, 1000.0);
        assert!((muscle_passive_force(&mb)).abs() < 1e-6);
    }

    #[test]
    fn test_fiber_velocity_computed() {
        let mut mb = new_muscle_body(0.1, 1000.0);
        mb_step_fiber(&mut mb, 0.12, 0.01);
        assert!(mb.fiber_velocity.abs() > 0.0);
    }

    #[test]
    fn test_e_constant_used() {
        let val = E.ln();
        assert!((val - 1.0).abs() < 1e-5);
    }
}
