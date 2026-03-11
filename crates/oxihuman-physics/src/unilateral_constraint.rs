// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Unilateral constraint — inequality constraint C(q) >= 0.

/// A unilateral (inequality) constraint — e.g., non-penetration.
#[derive(Debug, Clone)]
pub struct UnilateralConstraint {
    pub body_a: usize,
    pub body_b: Option<usize>,
    pub jacobian_a: [f64; 6],
    pub jacobian_b: [f64; 6],
    pub bias: f64,
    pub lambda: f64,      /* accumulated impulse (must be >= 0) */
    pub restitution: f64, /* coefficient of restitution */
}

impl UnilateralConstraint {
    /// Create a new unilateral constraint.
    pub fn new(
        body_a: usize,
        body_b: Option<usize>,
        jacobian_a: [f64; 6],
        jacobian_b: [f64; 6],
        bias: f64,
        restitution: f64,
    ) -> Self {
        UnilateralConstraint {
            body_a,
            body_b,
            jacobian_a,
            jacobian_b,
            bias,
            lambda: 0.0,
            restitution: restitution.clamp(0.0, 1.0),
        }
    }

    /// Velocity error J * v.
    pub fn velocity_error(&self, va: &[f64; 6], vb: &[f64; 6]) -> f64 {
        let jva: f64 = (0..6).map(|i| self.jacobian_a[i] * va[i]).sum();
        let jvb: f64 = (0..6).map(|i| self.jacobian_b[i] * vb[i]).sum();
        jva + jvb
    }

    /// Effective mass.
    pub fn effective_mass(
        &self,
        inv_mass_a: f64,
        inv_inertia_a: f64,
        inv_mass_b: f64,
        inv_inertia_b: f64,
    ) -> f64 {
        let tr = |j: &[f64; 6], im: f64, ii: f64| -> f64 {
            im * (j[0] * j[0] + j[1] * j[1] + j[2] * j[2])
                + ii * (j[3] * j[3] + j[4] * j[4] + j[5] * j[5])
        };
        tr(&self.jacobian_a, inv_mass_a, inv_inertia_a)
            + tr(&self.jacobian_b, inv_mass_b, inv_inertia_b)
    }

    /// Solve with non-negativity clamping on lambda.
    pub fn solve(
        &mut self,
        va: &[f64; 6],
        vb: &[f64; 6],
        inv_mass_a: f64,
        inv_inertia_a: f64,
        inv_mass_b: f64,
        inv_inertia_b: f64,
    ) -> f64 {
        let jv = self.velocity_error(va, vb);
        let em = self.effective_mass(inv_mass_a, inv_inertia_a, inv_mass_b, inv_inertia_b);
        if em.abs() < 1e-12 {
            return 0.0;
        }
        let rhs = jv + self.bias + self.restitution * jv.min(0.0);
        let delta_lambda = -rhs / em;
        let old_lambda = self.lambda;
        self.lambda = (self.lambda + delta_lambda).max(0.0);
        self.lambda - old_lambda
    }

    /// True if the constraint is active (lambda > 0).
    pub fn is_active(&self) -> bool {
        self.lambda > 0.0
    }

    /// Reset lambda (for new time step).
    pub fn reset_lambda(&mut self) {
        self.lambda = 0.0;
    }

    /// Check if constraint is violated (jv < 0 means separating, >= 0 means violating).
    pub fn is_violated(&self, va: &[f64; 6], vb: &[f64; 6]) -> bool {
        self.velocity_error(va, vb) < 0.0
    }
}

/// Clamp a lambda array to non-negative values (unilateral constraint requirement).
pub fn clamp_lambdas(lambdas: &mut [f64]) {
    for l in lambdas.iter_mut() {
        if *l < 0.0 {
            *l = 0.0;
        }
    }
}

/// Total positive impulse from a set of unilateral constraints.
pub fn total_normal_impulse(constraints: &[UnilateralConstraint]) -> f64 {
    constraints.iter().map(|c| c.lambda).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_error_zero() {
        let c =
            UnilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0, 0.0);
        let va = [0.0f64; 6];
        let vb = [0.0f64; 6];
        assert_eq!(c.velocity_error(&va, &vb), 0.0);
    }

    #[test]
    fn test_lambda_non_negative_after_solve() {
        let mut c =
            UnilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0, 0.0);
        let va = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0]; /* separating */
        let vb = [0.0f64; 6];
        c.solve(&va, &vb, 1.0, 1.0, 0.0, 0.0);
        assert!(c.lambda >= 0.0 /* lambda always non-negative */);
    }

    #[test]
    fn test_reset_lambda() {
        let mut c = UnilateralConstraint::new(0, None, [1.0; 6], [0.0; 6], 0.0, 0.0);
        c.lambda = 5.0;
        c.reset_lambda();
        assert_eq!(c.lambda, 0.0);
    }

    #[test]
    fn test_is_active() {
        let mut c = UnilateralConstraint::new(0, None, [1.0; 6], [0.0; 6], 0.0, 0.0);
        assert!(!c.is_active() /* initially inactive */);
        c.lambda = 1.0;
        assert!(c.is_active() /* active when lambda > 0 */);
    }

    #[test]
    fn test_restitution_clamp() {
        let c = UnilateralConstraint::new(0, None, [1.0; 6], [0.0; 6], 0.0, 2.0);
        assert!(c.restitution <= 1.0 /* clamped to 1 */);
    }

    #[test]
    fn test_clamp_lambdas() {
        let mut lambdas = vec![-1.0, 0.5, -0.3, 1.0];
        clamp_lambdas(&mut lambdas);
        assert!(lambdas.iter().all(|&l| l >= 0.0) /* all non-negative */);
    }

    #[test]
    fn test_total_normal_impulse() {
        let mut c1 = UnilateralConstraint::new(0, None, [1.0; 6], [0.0; 6], 0.0, 0.0);
        let mut c2 = UnilateralConstraint::new(1, None, [1.0; 6], [0.0; 6], 0.0, 0.0);
        c1.lambda = 3.0;
        c2.lambda = 4.0;
        assert!((total_normal_impulse(&[c1, c2]) - 7.0).abs() < 1e-10 /* total = 7 */);
    }

    #[test]
    fn test_is_violated() {
        let c =
            UnilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0, 0.0);
        let va = [-1.0, 0.0, 0.0, 0.0, 0.0, 0.0]; /* approaching */
        let vb = [0.0f64; 6];
        assert!(c.is_violated(&va, &vb) /* approaching = violated */);
    }

    #[test]
    fn test_effective_mass_positive() {
        let c =
            UnilateralConstraint::new(0, None, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], [0.0; 6], 0.0, 0.0);
        let em = c.effective_mass(1.0, 0.0, 0.0, 0.0);
        assert!(em > 0.0);
    }
}
