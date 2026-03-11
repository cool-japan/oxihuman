// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Velocity-level constraint (e.g., no-penetration).

#![allow(dead_code)]

/// A velocity-level constraint between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VelocityConstraint {
    pub body_a: u32,
    pub body_b: u32,
    pub jacobian: [f32; 3],
    pub bias: f32,
    pub lambda: f32,
    pub lo: f32,
    pub hi: f32,
}

/// Creates a new velocity constraint with clamped lambda range [lo=0, hi=inf].
#[allow(dead_code)]
pub fn new_velocity_constraint(a: u32, b: u32, jac: [f32; 3], bias: f32) -> VelocityConstraint {
    VelocityConstraint {
        body_a: a,
        body_b: b,
        jacobian: jac,
        bias,
        lambda: 0.0,
        lo: 0.0,
        hi: f32::INFINITY,
    }
}

/// Computes the effective mass for the constraint.
#[allow(dead_code)]
pub fn vc_effective_mass(vc: &VelocityConstraint, inv_mass_a: f32, inv_mass_b: f32) -> f32 {
    let j = vc.jacobian;
    let j_dot_j = j[0] * j[0] + j[1] * j[1] + j[2] * j[2];
    let denom = (inv_mass_a + inv_mass_b) * j_dot_j;
    if denom.abs() < 1e-12 {
        0.0
    } else {
        1.0 / denom
    }
}

/// Solves the constraint and returns the delta lambda.
#[allow(dead_code)]
pub fn vc_solve(
    vc: &mut VelocityConstraint,
    va: [f32; 3],
    vb: [f32; 3],
    eff_mass: f32,
) -> f32 {
    let j = vc.jacobian;
    let jv = j[0] * (va[0] - vb[0]) + j[1] * (va[1] - vb[1]) + j[2] * (va[2] - vb[2]);
    let delta_lambda = -(jv + vc.bias) * eff_mass;
    let new_lambda = (vc.lambda + delta_lambda).clamp(vc.lo, vc.hi);
    let actual_delta = new_lambda - vc.lambda;
    vc.lambda = new_lambda;
    actual_delta
}

/// Applies the impulse delta to body velocities.
#[allow(dead_code)]
pub fn vc_apply(
    va: &mut [f32; 3],
    vb: &mut [f32; 3],
    vc: &VelocityConstraint,
    delta: f32,
    im_a: f32,
    im_b: f32,
) {
    let j = vc.jacobian;
    va[0] += im_a * delta * j[0];
    va[1] += im_a * delta * j[1];
    va[2] += im_a * delta * j[2];
    vb[0] -= im_b * delta * j[0];
    vb[1] -= im_b * delta * j[1];
    vb[2] -= im_b * delta * j[2];
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    #[test]
    fn test_new_defaults() {
        let vc = new_velocity_constraint(0, 1, [0.0, 1.0, 0.0], 0.0);
        assert_eq!(vc.body_a, 0);
        assert_eq!(vc.body_b, 1);
        assert!((vc.lambda).abs() < EPS);
    }

    #[test]
    fn test_effective_mass_unit() {
        let vc = new_velocity_constraint(0, 1, [1.0, 0.0, 0.0], 0.0);
        let em = vc_effective_mass(&vc, 1.0, 1.0);
        assert!((em - 0.5).abs() < EPS);
    }

    #[test]
    fn test_effective_mass_zero_masses() {
        let vc = new_velocity_constraint(0, 1, [1.0, 0.0, 0.0], 0.0);
        let em = vc_effective_mass(&vc, 0.0, 0.0);
        assert!(em.abs() < EPS);
    }

    #[test]
    fn test_solve_clamped_lo() {
        let mut vc = new_velocity_constraint(0, 1, [1.0, 0.0, 0.0], 0.0);
        // If approaching, lambda should stay >= lo=0
        let delta = vc_solve(&mut vc, [0.0; 3], [0.0; 3], 1.0);
        assert!(vc.lambda >= vc.lo - EPS);
        let _ = delta;
    }

    #[test]
    fn test_apply_changes_velocity() {
        let vc = new_velocity_constraint(0, 1, [1.0, 0.0, 0.0], 0.0);
        let mut va = [0.0f32; 3];
        let mut vb = [0.0f32; 3];
        vc_apply(&mut va, &mut vb, &vc, 1.0, 1.0, 1.0);
        assert!((va[0] - 1.0).abs() < EPS);
        assert!((vb[0] - (-1.0)).abs() < EPS);
    }

    #[test]
    fn test_apply_zero_delta() {
        let vc = new_velocity_constraint(0, 1, [1.0, 0.0, 0.0], 0.0);
        let mut va = [5.0f32, 0.0, 0.0];
        let mut vb = [0.0f32; 3];
        vc_apply(&mut va, &mut vb, &vc, 0.0, 1.0, 1.0);
        assert!((va[0] - 5.0).abs() < EPS);
    }

    #[test]
    fn test_body_ids() {
        let vc = new_velocity_constraint(10, 20, [0.0, 0.0, 1.0], 0.1);
        assert_eq!(vc.body_a, 10);
        assert_eq!(vc.body_b, 20);
        assert!((vc.bias - 0.1).abs() < EPS);
    }

    #[test]
    fn test_effective_mass_equal_masses() {
        let vc = new_velocity_constraint(0, 1, [0.0, 0.0, 1.0], 0.0);
        let em = vc_effective_mass(&vc, 0.5, 0.5);
        assert!((em - 1.0).abs() < EPS);
    }

    #[test]
    fn test_lambda_clamp_hi() {
        let mut vc = new_velocity_constraint(0, 1, [1.0, 0.0, 0.0], -100.0);
        vc.lo = f32::NEG_INFINITY;
        vc.hi = 1.0;
        let _ = vc_solve(&mut vc, [0.0; 3], [0.0; 3], 1.0);
        assert!(vc.lambda <= vc.hi + EPS);
    }
}
