// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Trait-based constraint system for XPBD with compliance.

/// Core trait for all XPBD constraints.
///
/// Constraints correct particle positions to satisfy some geometric condition.
/// The `compliance` parameter controls stiffness — zero means perfectly rigid,
/// larger values mean softer/more compliant behavior.
///
/// XPBD uses the effective compliance `α_tilde = α / dt²` where `α` is the raw
/// compliance and `dt` is the (sub)step time.  The solver accumulates Lagrange
/// multipliers across Gauss-Seidel iterations to avoid over-correction.
pub trait XpbdConstraint: Send + Sync {
    /// Project the constraint by adjusting `positions` in-place.
    ///
    /// Returns the scalar constraint violation **before** correction so the
    /// solver can track convergence.
    ///
    /// # Arguments
    /// * `positions`  – mutable slice of all particle positions
    /// * `inv_masses` – inverse masses (0.0 ⇒ infinite mass / fixed)
    /// * `dt`         – timestep of the current substep
    fn project(
        &self,
        positions: &mut [[f64; 3]],
        inv_masses: &[f64],
        dt: f64,
    ) -> f64;

    /// Raw compliance value `α` (units: 1/stiffness).
    fn compliance(&self) -> f64;

    /// Particle indices this constraint references.
    fn particle_indices(&self) -> &[usize];

    /// Human-readable label (for debugging / profiling).
    fn label(&self) -> &str {
        "xpbd_constraint"
    }
}

/// Unique identifier for a constraint inside the solver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstraintId(pub(crate) u64);

impl ConstraintId {
    /// Create a new constraint id from a raw `u64`.
    pub fn raw(id: u64) -> Self {
        Self(id)
    }

    /// Return the inner value.
    pub fn value(self) -> u64 {
        self.0
    }
}

/// Wrapper that pairs a boxed constraint with its solver-assigned id and
/// the accumulated Lagrange multiplier `λ` (reset each timestep).
pub(crate) struct ConstraintEntry {
    pub id: ConstraintId,
    pub constraint: Box<dyn XpbdConstraint>,
    /// Accumulated Lagrange multiplier — reset to 0 at the start of each step.
    pub lambda: f64,
}

impl ConstraintEntry {
    pub fn new(id: ConstraintId, constraint: Box<dyn XpbdConstraint>) -> Self {
        Self {
            id,
            constraint,
            lambda: 0.0,
        }
    }

    /// Reset the multiplier for a new timestep.
    pub fn reset_lambda(&mut self) {
        self.lambda = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal dummy constraint for testing the trait machinery.
    struct DummyConstraint {
        indices: Vec<usize>,
        compliance_val: f64,
    }

    impl XpbdConstraint for DummyConstraint {
        fn project(
            &self,
            _positions: &mut [[f64; 3]],
            _inv_masses: &[f64],
            _dt: f64,
        ) -> f64 {
            0.0
        }

        fn compliance(&self) -> f64 {
            self.compliance_val
        }

        fn particle_indices(&self) -> &[usize] {
            &self.indices
        }
    }

    #[test]
    fn test_constraint_entry_reset() {
        let c = DummyConstraint {
            indices: vec![0, 1],
            compliance_val: 1e-6,
        };
        let mut entry = ConstraintEntry::new(ConstraintId(0), Box::new(c));
        entry.lambda = 42.0;
        entry.reset_lambda();
        assert!((entry.lambda).abs() < f64::EPSILON);
    }

    #[test]
    fn test_constraint_id() {
        let id = ConstraintId::raw(99);
        assert_eq!(id.value(), 99);
    }

    #[test]
    fn test_default_label() {
        let c = DummyConstraint {
            indices: vec![0],
            compliance_val: 0.0,
        };
        assert_eq!(c.label(), "xpbd_constraint");
    }
}
