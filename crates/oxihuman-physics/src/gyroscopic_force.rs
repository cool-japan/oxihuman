// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gyroscopic force / torque computation for rotating rigid bodies.

use std::f32::consts::PI;

#[allow(dead_code)]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_dot(v, v).sqrt()
}

/// Compute gyroscopic torque: tau = omega x (I * omega)
/// where I is a diagonal inertia tensor.
#[allow(dead_code)]
pub fn gyroscopic_torque(omega: [f32; 3], inertia: [f32; 3]) -> [f32; 3] {
    let i_omega = [
        inertia[0] * omega[0],
        inertia[1] * omega[1],
        inertia[2] * omega[2],
    ];
    vec3_cross(omega, i_omega)
}

/// Angular momentum L = I * omega (diagonal inertia).
#[allow(dead_code)]
pub fn angular_momentum(omega: [f32; 3], inertia: [f32; 3]) -> [f32; 3] {
    [
        inertia[0] * omega[0],
        inertia[1] * omega[1],
        inertia[2] * omega[2],
    ]
}

/// Rotational kinetic energy: E = 0.5 * omega . (I * omega).
#[allow(dead_code)]
pub fn rotational_kinetic_energy(omega: [f32; 3], inertia: [f32; 3]) -> f32 {
    let i_omega = angular_momentum(omega, inertia);
    0.5 * vec3_dot(omega, i_omega)
}

/// Precession rate for a symmetric top: omega_p = tau / (I * omega_spin).
#[allow(dead_code)]
pub fn precession_rate(torque_magnitude: f32, spin_inertia: f32, spin_rate: f32) -> f32 {
    let denom = spin_inertia * spin_rate;
    if denom.abs() < 1e-12 {
        return 0.0;
    }
    torque_magnitude / denom
}

/// Convert RPM to radians per second.
#[allow(dead_code)]
pub fn rpm_to_rad_s(rpm: f32) -> f32 {
    rpm * 2.0 * PI / 60.0
}

/// Convert radians per second to RPM.
#[allow(dead_code)]
pub fn rad_s_to_rpm(rad_s: f32) -> f32 {
    rad_s * 60.0 / (2.0 * PI)
}

/// Apply gyroscopic torque to angular velocity for one timestep.
#[allow(dead_code)]
pub fn apply_gyroscopic_step(
    omega: [f32; 3],
    inertia: [f32; 3],
    dt: f32,
) -> [f32; 3] {
    let tau = gyroscopic_torque(omega, inertia);
    let inv_i = [
        if inertia[0].abs() > 1e-12 { 1.0 / inertia[0] } else { 0.0 },
        if inertia[1].abs() > 1e-12 { 1.0 / inertia[1] } else { 0.0 },
        if inertia[2].abs() > 1e-12 { 1.0 / inertia[2] } else { 0.0 },
    ];
    let alpha = [
        -tau[0] * inv_i[0],
        -tau[1] * inv_i[1],
        -tau[2] * inv_i[2],
    ];
    vec3_add(omega, vec3_scale(alpha, dt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gyroscopic_torque_uniform() {
        // Uniform inertia => gyroscopic torque is zero
        let tau = gyroscopic_torque([1.0, 2.0, 3.0], [1.0, 1.0, 1.0]);
        assert!(vec3_len(tau) < 1e-5);
    }

    #[test]
    fn test_gyroscopic_torque_asymmetric() {
        let tau = gyroscopic_torque([0.0, 0.0, 10.0], [1.0, 2.0, 3.0]);
        // omega = (0,0,10), I*omega = (0,0,30)
        // cross = (0*30 - 10*0, 10*0 - 0*30, 0*0 - 0*0) = (0,0,0)
        assert!(vec3_len(tau) < 1e-5);
    }

    #[test]
    fn test_angular_momentum() {
        let l = angular_momentum([1.0, 2.0, 3.0], [2.0, 3.0, 4.0]);
        assert!((l[0] - 2.0).abs() < 1e-5);
        assert!((l[1] - 6.0).abs() < 1e-5);
        assert!((l[2] - 12.0).abs() < 1e-5);
    }

    #[test]
    fn test_rotational_energy() {
        let e = rotational_kinetic_energy([0.0, 0.0, 10.0], [0.0, 0.0, 2.0]);
        assert!((e - 100.0).abs() < 1e-3);
    }

    #[test]
    fn test_precession_rate() {
        let p = precession_rate(10.0, 2.0, 5.0);
        assert!((p - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_precession_zero_spin() {
        assert!((precession_rate(10.0, 2.0, 0.0)).abs() < 1e-5);
    }

    #[test]
    fn test_rpm_conversion() {
        let rad = rpm_to_rad_s(60.0);
        assert!((rad - 2.0 * PI).abs() < 1e-3);
        let rpm = rad_s_to_rpm(rad);
        assert!((rpm - 60.0).abs() < 1e-3);
    }

    #[test]
    fn test_apply_step_uniform() {
        let omega = [1.0, 2.0, 3.0];
        let inertia = [1.0, 1.0, 1.0];
        let new_omega = apply_gyroscopic_step(omega, inertia, 0.01);
        // Uniform inertia => no change
        assert!((new_omega[0] - omega[0]).abs() < 1e-4);
    }

    #[test]
    fn test_apply_step_zero_dt() {
        let omega = [5.0, 3.0, 1.0];
        let new_omega = apply_gyroscopic_step(omega, [1.0, 2.0, 3.0], 0.0);
        assert!((new_omega[0] - omega[0]).abs() < 1e-5);
    }

    #[test]
    fn test_zero_omega() {
        let tau = gyroscopic_torque([0.0, 0.0, 0.0], [1.0, 2.0, 3.0]);
        assert!(vec3_len(tau) < 1e-10);
    }
}
