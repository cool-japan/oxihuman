// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Joint damper: velocity-proportional damping applied at a joint.

use std::f32::consts::FRAC_PI_4;

/// Joint damper configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct JointDamper {
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub max_linear_force: f32,
    pub max_angular_torque: f32,
    pub enabled: bool,
}

/// Create a joint damper.
#[allow(dead_code)]
pub fn new_joint_damper(linear: f32, angular: f32) -> JointDamper {
    JointDamper {
        linear_damping: linear,
        angular_damping: angular,
        max_linear_force: f32::MAX,
        max_angular_torque: f32::MAX,
        enabled: true,
    }
}

/// Compute linear damping force for a given relative velocity.
#[allow(dead_code)]
pub fn damper_linear_force(d: &JointDamper, rel_vel: [f32; 3]) -> [f32; 3] {
    if !d.enabled {
        return [0.0; 3];
    }
    let f = [
        -d.linear_damping * rel_vel[0],
        -d.linear_damping * rel_vel[1],
        -d.linear_damping * rel_vel[2],
    ];
    let mag = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
    if mag > d.max_linear_force && mag > 1e-12 {
        let s = d.max_linear_force / mag;
        [f[0] * s, f[1] * s, f[2] * s]
    } else {
        f
    }
}

/// Compute angular damping torque for a given relative angular velocity.
#[allow(dead_code)]
pub fn damper_angular_torque(d: &JointDamper, rel_ang_vel: [f32; 3]) -> [f32; 3] {
    if !d.enabled {
        return [0.0; 3];
    }
    let t = [
        -d.angular_damping * rel_ang_vel[0],
        -d.angular_damping * rel_ang_vel[1],
        -d.angular_damping * rel_ang_vel[2],
    ];
    let mag = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
    if mag > d.max_angular_torque && mag > 1e-12 {
        let s = d.max_angular_torque / mag;
        [t[0] * s, t[1] * s, t[2] * s]
    } else {
        t
    }
}

/// Power dissipated by linear damping.
#[allow(dead_code)]
pub fn damper_linear_power(d: &JointDamper, rel_vel: [f32; 3]) -> f32 {
    let f = damper_linear_force(d, rel_vel);
    -(f[0] * rel_vel[0] + f[1] * rel_vel[1] + f[2] * rel_vel[2])
}

/// Damping energy lost in one timestep.
#[allow(dead_code)]
pub fn damper_energy_step(d: &JointDamper, rel_vel: [f32; 3], dt: f32) -> f32 {
    damper_linear_power(d, rel_vel) * dt
}

/// Enable or disable the damper.
#[allow(dead_code)]
pub fn damper_set_enabled(d: &mut JointDamper, enabled: bool) {
    d.enabled = enabled;
}

/// Set linear damping coefficient.
#[allow(dead_code)]
pub fn damper_set_linear(d: &mut JointDamper, c: f32) {
    d.linear_damping = c;
}

/// Set angular damping coefficient.
#[allow(dead_code)]
pub fn damper_set_angular(d: &mut JointDamper, c: f32) {
    d.angular_damping = c;
}

/// Decay factor per timestep for a given damping and mass.
#[allow(dead_code)]
pub fn damper_decay_factor(damping: f32, mass: f32, dt: f32) -> f32 {
    (1.0 - (damping / mass.max(1e-12)) * dt).max(0.0)
}

/// Dummy constant use.
#[allow(dead_code)]
pub fn joint_45_deg() -> f32 {
    FRAC_PI_4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_force_opposes_velocity() {
        let d = new_joint_damper(10.0, 1.0);
        let f = damper_linear_force(&d, [1.0, 0.0, 0.0]);
        assert!(f[0] < 0.0);
    }

    #[test]
    fn test_angular_torque_opposes_velocity() {
        let d = new_joint_damper(1.0, 5.0);
        let t = damper_angular_torque(&d, [0.0, 1.0, 0.0]);
        assert!(t[1] < 0.0);
    }

    #[test]
    fn test_disabled_zero_force() {
        let mut d = new_joint_damper(10.0, 10.0);
        damper_set_enabled(&mut d, false);
        let f = damper_linear_force(&d, [5.0, 0.0, 0.0]);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn test_max_force_clamp() {
        let mut d = new_joint_damper(1000.0, 1.0);
        d.max_linear_force = 1.0;
        let f = damper_linear_force(&d, [1.0, 0.0, 0.0]);
        let mag = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        assert!(mag <= 1.0 + 1e-5);
    }

    #[test]
    fn test_power_positive() {
        let d = new_joint_damper(5.0, 1.0);
        let p = damper_linear_power(&d, [2.0, 0.0, 0.0]);
        assert!(p > 0.0);
    }

    #[test]
    fn test_energy_step() {
        let d = new_joint_damper(10.0, 1.0);
        let e = damper_energy_step(&d, [1.0, 0.0, 0.0], 0.1);
        assert!((e - 1.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_set_damping() {
        let mut d = new_joint_damper(1.0, 1.0);
        damper_set_linear(&mut d, 20.0);
        let f = damper_linear_force(&d, [1.0, 0.0, 0.0]);
        assert!((f[0] - (-20.0_f32)).abs() < 1e-5);
    }

    #[test]
    fn test_decay_factor() {
        let df = damper_decay_factor(10.0, 1.0, 0.05);
        assert!((df - 0.5_f32).abs() < 1e-5);
    }

    #[test]
    fn test_joint_45_deg() {
        use std::f32::consts::FRAC_PI_4;
        assert!((joint_45_deg() - FRAC_PI_4).abs() < 1e-6);
    }
}
