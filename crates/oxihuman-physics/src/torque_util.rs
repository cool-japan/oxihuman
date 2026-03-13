// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Compute torque τ = r × F (cross product).
#[allow(dead_code)]
pub fn torque_from_force(r: [f32; 3], f: [f32; 3]) -> [f32; 3] {
    [
        r[1] * f[2] - r[2] * f[1],
        r[2] * f[0] - r[0] * f[2],
        r[0] * f[1] - r[1] * f[0],
    ]
}

/// Compute angular acceleration α = τ / I for diagonal inertia tensor.
#[allow(dead_code)]
pub fn angular_accel(torque: [f32; 3], inertia_diag: [f32; 3]) -> [f32; 3] {
    [
        if inertia_diag[0].abs() < 1e-12 { 0.0 } else { torque[0] / inertia_diag[0] },
        if inertia_diag[1].abs() < 1e-12 { 0.0 } else { torque[1] / inertia_diag[1] },
        if inertia_diag[2].abs() < 1e-12 { 0.0 } else { torque[2] / inertia_diag[2] },
    ]
}

/// Sum multiple torque vectors.
#[allow(dead_code)]
pub fn net_torque(torques: &[[f32; 3]]) -> [f32; 3] {
    torques.iter().fold([0.0f32; 3], |acc, t| {
        [acc[0] + t[0], acc[1] + t[1], acc[2] + t[2]]
    })
}

/// Magnitude of a torque vector.
#[allow(dead_code)]
pub fn torque_magnitude(t: [f32; 3]) -> f32 {
    (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torque_basic_cross_product() {
        // r = (1,0,0), F = (0,1,0) => τ = (0,0,1)
        let t = torque_from_force([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((t[2] - 1.0).abs() < 1e-6);
        assert!(t[0].abs() < 1e-6);
        assert!(t[1].abs() < 1e-6);
    }

    #[test]
    fn torque_zero_force() {
        let t = torque_from_force([1.0, 2.0, 3.0], [0.0, 0.0, 0.0]);
        assert_eq!(t, [0.0; 3]);
    }

    #[test]
    fn torque_parallel_zero() {
        // Parallel r and F produce zero torque
        let t = torque_from_force([1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!(torque_magnitude(t) < 1e-6);
    }

    #[test]
    fn angular_accel_basic() {
        let a = angular_accel([2.0, 4.0, 6.0], [2.0, 2.0, 2.0]);
        assert!((a[0] - 1.0).abs() < 1e-6);
        assert!((a[1] - 2.0).abs() < 1e-6);
        assert!((a[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn angular_accel_zero_inertia_returns_zero() {
        let a = angular_accel([5.0, 5.0, 5.0], [0.0, 0.0, 0.0]);
        assert_eq!(a, [0.0; 3]);
    }

    #[test]
    fn net_torque_empty() {
        let t = net_torque(&[]);
        assert_eq!(t, [0.0; 3]);
    }

    #[test]
    fn net_torque_sums_correctly() {
        let torques = [[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]];
        let t = net_torque(&torques);
        assert!((t[0] - 1.0).abs() < 1e-6);
        assert!((t[1] - 2.0).abs() < 1e-6);
        assert!((t[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn torque_magnitude_basic() {
        let t = [3.0f32, 4.0, 0.0];
        assert!((torque_magnitude(t) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn torque_magnitude_zero() {
        assert_eq!(torque_magnitude([0.0; 3]), 0.0);
    }

    #[test]
    fn torque_anticommutative() {
        // τ(r,F) = -τ(F,r)
        let r = [1.0f32, 2.0, 3.0];
        let f = [4.0f32, 5.0, 6.0];
        let t1 = torque_from_force(r, f);
        let t2 = torque_from_force(f, r);
        for i in 0..3 {
            assert!((t1[i] + t2[i]).abs() < 1e-5);
        }
    }
}
