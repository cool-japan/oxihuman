// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Gyroscopic torque calculation.

#![allow(dead_code)]

/// Compute the angular momentum L = I * ω where I is a diagonal inertia tensor.
#[allow(dead_code)]
pub fn angular_momentum(inertia: [f32; 3], omega: [f32; 3]) -> [f32; 3] {
    [
        inertia[0] * omega[0],
        inertia[1] * omega[1],
        inertia[2] * omega[2],
    ]
}

/// Compute gyroscopic torque τ = ω × (I·ω).
#[allow(dead_code)]
pub fn gyroscopic_torque(inertia: [f32; 3], omega: [f32; 3]) -> [f32; 3] {
    let l = angular_momentum(inertia, omega);
    // Cross product ω × L
    [
        omega[1] * l[2] - omega[2] * l[1],
        omega[2] * l[0] - omega[0] * l[2],
        omega[0] * l[1] - omega[1] * l[0],
    ]
}

/// Compute the precession rate of a spinning top.
/// Rate = |torque| / (|spin_axis| * angular_momentum_magnitude)
#[allow(dead_code)]
pub fn precession_rate(torque: [f32; 3], spin_axis: [f32; 3], angular_momentum: f32) -> f32 {
    let t_mag = (torque[0] * torque[0] + torque[1] * torque[1] + torque[2] * torque[2]).sqrt();
    let s_mag =
        (spin_axis[0] * spin_axis[0] + spin_axis[1] * spin_axis[1] + spin_axis[2] * spin_axis[2])
            .sqrt();
    if s_mag < 1e-9 || angular_momentum.abs() < 1e-9 {
        return 0.0;
    }
    t_mag / (s_mag * angular_momentum.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angular_momentum_basic() {
        let l = angular_momentum([1.0, 2.0, 3.0], [1.0, 1.0, 1.0]);
        assert!((l[0] - 1.0).abs() < 1e-6);
        assert!((l[1] - 2.0).abs() < 1e-6);
        assert!((l[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_angular_momentum_zero_omega() {
        let l = angular_momentum([5.0, 5.0, 5.0], [0.0; 3]);
        for v in &l {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn test_gyroscopic_torque_isotropic_is_zero() {
        // If I is isotropic (Ixx=Iyy=Izz), τ = ω × (I·ω) = I * (ω × ω) = 0
        let tau = gyroscopic_torque([2.0, 2.0, 2.0], [1.0, 2.0, 3.0]);
        for v in &tau {
            assert!(v.abs() < 1e-5, "got {v}");
        }
    }

    #[test]
    fn test_gyroscopic_torque_nonzero_for_anisotropic() {
        // Non-isotropic inertia + rotation should produce nonzero torque
        let tau = gyroscopic_torque([1.0, 2.0, 4.0], [1.0, 1.0, 1.0]);
        let mag = (tau[0] * tau[0] + tau[1] * tau[1] + tau[2] * tau[2]).sqrt();
        assert!(mag > 0.0);
    }

    #[test]
    fn test_precession_rate_zero_spin() {
        let rate = precession_rate([0.0, 9.8, 0.0], [0.0; 3], 10.0);
        assert!(rate.abs() < 1e-9);
    }

    #[test]
    fn test_precession_rate_zero_angular_momentum() {
        let rate = precession_rate([0.0, 9.8, 0.0], [0.0, 1.0, 0.0], 0.0);
        assert!(rate.abs() < 1e-9);
    }

    #[test]
    fn test_precession_rate_positive() {
        let rate = precession_rate([0.0, 1.0, 0.0], [0.0, 1.0, 0.0], 1.0);
        assert!(rate > 0.0);
    }

    #[test]
    fn test_angular_momentum_scales_with_inertia() {
        let l1 = angular_momentum([1.0, 1.0, 1.0], [2.0, 0.0, 0.0]);
        let l2 = angular_momentum([3.0, 1.0, 1.0], [2.0, 0.0, 0.0]);
        assert!((l2[0] - 3.0 * l1[0]).abs() < 1e-6);
    }

    #[test]
    fn test_gyroscopic_torque_length_is_6() {
        let tau = gyroscopic_torque([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        assert_eq!(tau.len(), 3);
    }
}
