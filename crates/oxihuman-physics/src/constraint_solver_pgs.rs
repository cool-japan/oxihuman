#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Projected Gauss-Seidel constraint solver.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PgsConstraint {
    pub jacobian_a: [f32; 3],
    pub jacobian_b: [f32; 3],
    pub bias: f32,
    pub lambda: f32,
    pub lo: f32,
    pub hi: f32,
    pub eff_mass: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PgsSolver {
    pub constraints: Vec<PgsConstraint>,
    pub iterations: u32,
}

#[allow(dead_code)]
pub fn new_pgs_solver(iterations: u32) -> PgsSolver {
    PgsSolver {
        constraints: Vec::new(),
        iterations,
    }
}

#[allow(dead_code)]
pub fn add_pgs_constraint(
    solver: &mut PgsSolver,
    ja: [f32; 3],
    jb: [f32; 3],
    bias: f32,
    lo: f32,
    hi: f32,
    eff_mass: f32,
) {
    solver.constraints.push(PgsConstraint {
        jacobian_a: ja,
        jacobian_b: jb,
        bias,
        lambda: 0.0,
        lo,
        hi,
        eff_mass,
    });
}

/// Perform one PGS iteration step for a single constraint.
/// Returns the delta lambda applied.
#[allow(dead_code)]
pub fn pgs_solve_step(c: &mut PgsConstraint, va: &[f32; 3], vb: &[f32; 3]) -> f32 {
    let jv_a = c.jacobian_a[0] * va[0] + c.jacobian_a[1] * va[1] + c.jacobian_a[2] * va[2];
    let jv_b = c.jacobian_b[0] * vb[0] + c.jacobian_b[1] * vb[1] + c.jacobian_b[2] * vb[2];
    let cdot = jv_a + jv_b + c.bias;
    let inv_eff = if c.eff_mass.abs() > 1e-10 {
        1.0 / c.eff_mass
    } else {
        0.0
    };
    let delta = -cdot * inv_eff;
    let old = c.lambda;
    c.lambda = (c.lambda + delta).clamp(c.lo, c.hi);
    c.lambda - old
}

#[allow(dead_code)]
pub fn pgs_constraint_count(solver: &PgsSolver) -> usize {
    solver.constraints.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_solver_empty() {
        let s = new_pgs_solver(10);
        assert_eq!(pgs_constraint_count(&s), 0);
        assert_eq!(s.iterations, 10);
    }

    #[test]
    fn test_add_constraint() {
        let mut s = new_pgs_solver(5);
        add_pgs_constraint(&mut s, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0, -10.0, 10.0, 1.0);
        assert_eq!(pgs_constraint_count(&s), 1);
    }

    #[test]
    fn test_solve_step_clamp_lo() {
        let mut c = PgsConstraint {
            jacobian_a: [1.0, 0.0, 0.0],
            jacobian_b: [0.0, 0.0, 0.0],
            bias: 0.0,
            lambda: 0.0,
            lo: 0.0,
            hi: 10.0,
            eff_mass: 1.0,
        };
        // jv_a = 1.0, cdot = 1.0, delta = -1.0, new lambda clamped to 0
        let dl = pgs_solve_step(&mut c, &[1.0, 0.0, 0.0], &[0.0, 0.0, 0.0]);
        assert!(c.lambda >= c.lo);
        assert!(dl <= 0.0);
    }

    #[test]
    fn test_solve_step_zero_velocity() {
        let mut c = PgsConstraint {
            jacobian_a: [1.0, 0.0, 0.0],
            jacobian_b: [0.0, 0.0, 0.0],
            bias: 0.0,
            lambda: 5.0,
            lo: 0.0,
            hi: 10.0,
            eff_mass: 1.0,
        };
        let dl = pgs_solve_step(&mut c, &[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]);
        // cdot = bias = 0, delta = 0
        assert!(dl.abs() < 1e-6);
    }

    #[test]
    fn test_solve_step_clamp_hi() {
        let mut c = PgsConstraint {
            jacobian_a: [-1.0, 0.0, 0.0],
            jacobian_b: [0.0, 0.0, 0.0],
            bias: 0.0,
            lambda: 9.0,
            lo: 0.0,
            hi: 10.0,
            eff_mass: 1.0,
        };
        pgs_solve_step(&mut c, &[1.0, 0.0, 0.0], &[0.0, 0.0, 0.0]);
        assert!(c.lambda <= c.hi);
    }

    #[test]
    fn test_eff_mass_zero() {
        let mut c = PgsConstraint {
            jacobian_a: [1.0, 0.0, 0.0],
            jacobian_b: [0.0, 0.0, 0.0],
            bias: 1.0,
            lambda: 0.0,
            lo: -100.0,
            hi: 100.0,
            eff_mass: 0.0,
        };
        let dl = pgs_solve_step(&mut c, &[1.0, 0.0, 0.0], &[0.0, 0.0, 0.0]);
        assert!(dl.abs() < 1e-6);
    }

    #[test]
    fn test_multiple_constraints() {
        let mut s = new_pgs_solver(10);
        for i in 0..5 {
            add_pgs_constraint(&mut s, [i as f32, 0.0, 0.0], [0.0, 0.0, 0.0], 0.0, -10.0, 10.0, 1.0);
        }
        assert_eq!(pgs_constraint_count(&s), 5);
    }

    #[test]
    fn test_lambda_initialized_zero() {
        let mut s = new_pgs_solver(1);
        add_pgs_constraint(&mut s, [1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.0, -10.0, 10.0, 2.0);
        assert!(s.constraints[0].lambda.abs() < 1e-6);
    }

    #[test]
    fn test_bias_effect() {
        let mut c = PgsConstraint {
            jacobian_a: [0.0, 0.0, 0.0],
            jacobian_b: [0.0, 0.0, 0.0],
            bias: 2.0,
            lambda: 0.0,
            lo: -100.0,
            hi: 100.0,
            eff_mass: 1.0,
        };
        let dl = pgs_solve_step(&mut c, &[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]);
        assert!((dl + 2.0).abs() < 1e-6);
    }
}
