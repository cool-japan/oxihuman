#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contact velocity and restitution helpers.

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn relative_contact_vel(
    va: [f32; 3],
    vb: [f32; 3],
    ra: [f32; 3],
    rb: [f32; 3],
    wa: [f32; 3],
    wb: [f32; 3],
    n: [f32; 3],
) -> f32 {
    let vel_a = [
        va[0] + cross(wa, ra)[0],
        va[1] + cross(wa, ra)[1],
        va[2] + cross(wa, ra)[2],
    ];
    let vel_b = [
        vb[0] + cross(wb, rb)[0],
        vb[1] + cross(wb, rb)[1],
        vb[2] + cross(wb, rb)[2],
    ];
    let rel_vel = [
        vel_a[0] - vel_b[0],
        vel_a[1] - vel_b[1],
        vel_a[2] - vel_b[2],
    ];
    dot(rel_vel, n)
}

#[allow(dead_code)]
pub fn restitution_target_vel(vn: f32, restitution: f32) -> f32 {
    if vn < 0.0 {
        -restitution * vn
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn velocity_bias(depth: f32, dt: f32, slop: f32, beta: f32) -> f32 {
    let excess = (depth - slop).max(0.0);
    if dt > 1e-12 {
        beta * excess / dt
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn contact_lambda(vn: f32, bias: f32, eff_mass: f32) -> f32 {
    if eff_mass < 1e-12 {
        return 0.0;
    }
    -(vn + bias) / eff_mass
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_vel_approaching() {
        // Body A moving towards B along Y axis
        let va = [0.0f32, -2.0, 0.0];
        let vb = [0.0f32, 0.0, 0.0];
        let n = [0.0f32, 1.0, 0.0];
        let vn = relative_contact_vel(va, vb, [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], n);
        assert!(vn < 0.0);
    }

    #[test]
    fn relative_vel_separating() {
        let va = [0.0f32, 2.0, 0.0];
        let vb = [0.0f32, 0.0, 0.0];
        let n = [0.0f32, 1.0, 0.0];
        let vn = relative_contact_vel(va, vb, [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], n);
        assert!(vn > 0.0);
    }

    #[test]
    fn restitution_zero_for_separating() {
        assert_eq!(restitution_target_vel(1.0, 0.8), 0.0);
    }

    #[test]
    fn restitution_nonzero_for_approaching() {
        let target = restitution_target_vel(-2.0, 0.5);
        assert!((target - 1.0).abs() < 1e-5);
    }

    #[test]
    fn velocity_bias_zero_no_penetration() {
        let bias = velocity_bias(0.0, 0.016, 0.01, 0.2);
        assert_eq!(bias, 0.0);
    }

    #[test]
    fn velocity_bias_nonzero_with_penetration() {
        let bias = velocity_bias(0.05, 0.016, 0.01, 0.2);
        assert!(bias > 0.0);
    }

    #[test]
    fn velocity_bias_zero_dt() {
        let bias = velocity_bias(0.1, 0.0, 0.01, 0.2);
        assert_eq!(bias, 0.0);
    }

    #[test]
    fn contact_lambda_zero_mass() {
        assert_eq!(contact_lambda(-1.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn contact_lambda_basic() {
        let lambda = contact_lambda(-1.0, 0.0, 2.0);
        assert!((lambda - 0.5).abs() < 1e-5);
    }

    #[test]
    fn relative_vel_both_zero() {
        let vn = relative_contact_vel([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0, 1.0, 0.0]);
        assert_eq!(vn, 0.0);
    }
}
