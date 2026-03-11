// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Explicit Euler integration step: pos += vel * dt.
#[allow(dead_code)]
pub fn euler_step(pos: [f32; 3], vel: [f32; 3], dt: f32) -> [f32; 3] {
    [
        pos[0] + vel[0] * dt,
        pos[1] + vel[1] * dt,
        pos[2] + vel[2] * dt,
    ]
}

/// Semi-implicit Euler: update velocity first, then position.
#[allow(dead_code)]
pub fn semi_implicit_euler(pos: &mut [f32; 3], vel: &mut [f32; 3], acc: [f32; 3], dt: f32) {
    vel[0] += acc[0] * dt;
    vel[1] += acc[1] * dt;
    vel[2] += acc[2] * dt;
    pos[0] += vel[0] * dt;
    pos[1] += vel[1] * dt;
    pos[2] += vel[2] * dt;
}

/// Classic 4th-order Runge-Kutta step for position/velocity under constant acceleration.
#[allow(dead_code)]
pub fn runge_kutta4_step(
    pos: [f32; 3],
    vel: [f32; 3],
    acc: [f32; 3],
    dt: f32,
) -> ([f32; 3], [f32; 3]) {
    // For constant acceleration, RK4 simplifies significantly.
    let half_dt = dt * 0.5;
    // k1
    let k1_v = vel;
    let k1_a = acc;
    // k2
    let k2_v = [vel[0] + k1_a[0] * half_dt, vel[1] + k1_a[1] * half_dt, vel[2] + k1_a[2] * half_dt];
    let k2_a = acc;
    // k3
    let k3_v = [vel[0] + k2_a[0] * half_dt, vel[1] + k2_a[1] * half_dt, vel[2] + k2_a[2] * half_dt];
    let k3_a = acc;
    // k4
    let k4_v = [vel[0] + k3_a[0] * dt, vel[1] + k3_a[1] * dt, vel[2] + k3_a[2] * dt];
    let k4_a = acc;

    let inv6 = 1.0 / 6.0;
    let new_pos = [
        pos[0] + inv6 * dt * (k1_v[0] + 2.0 * k2_v[0] + 2.0 * k3_v[0] + k4_v[0]),
        pos[1] + inv6 * dt * (k1_v[1] + 2.0 * k2_v[1] + 2.0 * k3_v[1] + k4_v[1]),
        pos[2] + inv6 * dt * (k1_v[2] + 2.0 * k2_v[2] + 2.0 * k3_v[2] + k4_v[2]),
    ];
    let new_vel = [
        vel[0] + inv6 * dt * (k1_a[0] + 2.0 * k2_a[0] + 2.0 * k3_a[0] + k4_a[0]),
        vel[1] + inv6 * dt * (k1_a[1] + 2.0 * k2_a[1] + 2.0 * k3_a[1] + k4_a[1]),
        vel[2] + inv6 * dt * (k1_a[2] + 2.0 * k2_a[2] + 2.0 * k3_a[2] + k4_a[2]),
    ];
    (new_pos, new_vel)
}

/// Leapfrog (Verlet) integration step.
#[allow(dead_code)]
pub fn leapfrog_step(pos: &mut [f32; 3], vel: &mut [f32; 3], acc: [f32; 3], dt: f32) {
    let half_dt = dt * 0.5;
    vel[0] += acc[0] * half_dt;
    vel[1] += acc[1] * half_dt;
    vel[2] += acc[2] * half_dt;
    pos[0] += vel[0] * dt;
    pos[1] += vel[1] * dt;
    pos[2] += vel[2] * dt;
    vel[0] += acc[0] * half_dt;
    vel[1] += acc[1] * half_dt;
    vel[2] += acc[2] * half_dt;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euler_step_zero_velocity() {
        let p = euler_step([1.0, 2.0, 3.0], [0.0, 0.0, 0.0], 0.1);
        assert!((p[0] - 1.0).abs() < 1e-6);
        assert!((p[1] - 2.0).abs() < 1e-6);
        assert!((p[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn euler_step_uniform_velocity() {
        let p = euler_step([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn semi_implicit_euler_no_acceleration() {
        let mut pos = [0.0f32; 3];
        let mut vel = [1.0, 0.0, 0.0];
        semi_implicit_euler(&mut pos, &mut vel, [0.0; 3], 1.0);
        assert!((pos[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn semi_implicit_euler_with_acceleration() {
        let mut pos = [0.0f32; 3];
        let mut vel = [0.0f32; 3];
        semi_implicit_euler(&mut pos, &mut vel, [2.0, 0.0, 0.0], 1.0);
        // vel becomes 2.0, pos becomes 2.0
        assert!((pos[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn rk4_zero_acceleration() {
        let pos = [1.0f32, 0.0, 0.0];
        let vel = [1.0f32, 0.0, 0.0];
        let (np, nv) = runge_kutta4_step(pos, vel, [0.0; 3], 1.0);
        assert!((np[0] - 2.0).abs() < 1e-5);
        assert!((nv[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn rk4_produces_new_position() {
        let pos = [0.0f32; 3];
        let vel = [0.0f32; 3];
        let (np, _nv) = runge_kutta4_step(pos, vel, [1.0, 0.0, 0.0], 0.1);
        assert!(np[0] > 0.0);
    }

    #[test]
    fn leapfrog_zero_acceleration_moves_by_velocity() {
        let mut pos = [0.0f32; 3];
        let mut vel = [1.0, 0.0, 0.0];
        leapfrog_step(&mut pos, &mut vel, [0.0; 3], 1.0);
        assert!((pos[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn leapfrog_acceleration_changes_velocity() {
        let mut pos = [0.0f32; 3];
        let mut vel = [0.0f32; 3];
        leapfrog_step(&mut pos, &mut vel, [2.0, 0.0, 0.0], 1.0);
        assert!((vel[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn euler_step_3d() {
        let p = euler_step([1.0, 2.0, 3.0], [1.0, -1.0, 0.5], 2.0);
        assert!((p[0] - 3.0).abs() < 1e-6);
        assert!((p[1] - 0.0).abs() < 1e-6);
        assert!((p[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn leapfrog_reversible() {
        let mut pos = [5.0f32, 0.0, 0.0];
        let mut vel = [1.0f32, 0.0, 0.0];
        leapfrog_step(&mut pos, &mut vel, [0.0; 3], 1.0);
        assert!((pos[0] - 6.0).abs() < 1e-6);
    }
}
