// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Inverted pendulum balance control with PD controller.

use std::f32::consts::PI;

pub struct BalancePendulum {
    pub angle_rad: f32,
    pub angular_vel: f32,
    pub kp: f32,
    pub kd: f32,
    pub length: f32,
    pub g: f32,
}

pub fn new_balance_pendulum(length: f32) -> BalancePendulum {
    BalancePendulum {
        angle_rad: 0.0,
        angular_vel: 0.0,
        kp: 50.0,
        kd: 10.0,
        length: length.max(0.001),
        g: 9.81,
    }
}

pub fn balance_control_torque(p: &BalancePendulum) -> f32 {
    /* PD controller: torque = -kp*angle - kd*angular_vel */
    -p.kp * p.angle_rad - p.kd * p.angular_vel
}

pub fn balance_step(p: &mut BalancePendulum, dt: f32) {
    /* inverted pendulum: d²θ/dt² = (g/l)*sin(θ) - torque/(m*l²)
    Using m=1, simplified as angular_accel = g/l * sin(θ) + torque/l² */
    let gravity_torque = (p.g / p.length) * p.angle_rad.sin();
    let control = balance_control_torque(p);
    let angular_accel = gravity_torque + control / (p.length * p.length);
    p.angular_vel += angular_accel * dt;
    p.angle_rad += p.angular_vel * dt;
}

pub fn balance_is_stable(p: &BalancePendulum) -> bool {
    p.angle_rad.abs() < 0.1
}

pub fn balance_angle_deg(p: &BalancePendulum) -> f32 {
    p.angle_rad * 180.0 / PI
}

pub fn balance_apply_perturbation(p: &mut BalancePendulum, impulse: f32) {
    p.angular_vel += impulse;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_balance_pendulum() {
        /* new pendulum starts at zero angle */
        let p = new_balance_pendulum(1.0);
        assert_eq!(p.angle_rad, 0.0);
        assert_eq!(p.angular_vel, 0.0);
    }

    #[test]
    fn test_balance_step_stable_when_zero() {
        /* at zero angle, pendulum remains stable */
        let mut p = new_balance_pendulum(1.0);
        for _ in 0..100 {
            balance_step(&mut p, 0.01);
        }
        assert!(balance_is_stable(&p));
    }

    #[test]
    fn test_balance_is_stable_at_origin() {
        /* zero angle and velocity is stable */
        let p = new_balance_pendulum(1.0);
        assert!(balance_is_stable(&p));
    }

    #[test]
    fn test_balance_angle_deg() {
        /* angle in degrees conversion */
        let mut p = new_balance_pendulum(1.0);
        p.angle_rad = PI / 4.0;
        assert!((balance_angle_deg(&p) - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_balance_apply_perturbation() {
        /* perturbation changes angular velocity */
        let mut p = new_balance_pendulum(1.0);
        balance_apply_perturbation(&mut p, 1.0);
        assert!((p.angular_vel - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_balance_control_torque_zero_at_origin() {
        /* no torque needed at equilibrium */
        let p = new_balance_pendulum(1.0);
        assert_eq!(balance_control_torque(&p), 0.0);
    }

    #[test]
    fn test_balance_control_torque_corrects_angle() {
        /* control torque opposes positive angle */
        let mut p = new_balance_pendulum(1.0);
        p.angle_rad = 0.1;
        let torque = balance_control_torque(&p);
        assert!(torque < 0.0);
    }
}
