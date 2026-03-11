// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! XPBD Unified Solver — Extended Position-Based Dynamics with compliance.
//!
//! This module provides a complete XPBD simulation framework with:
//!
//! - A trait-based constraint system ([`XpbdConstraint`])
//! - Built-in constraints (distance, volume, shape-matching, collision)
//! - A Gauss-Seidel iterative solver ([`XpbdSolver`])
//! - Dynamic constraint addition and removal
//!
//! # Algorithm
//!
//! XPBD extends classical Position-Based Dynamics (PBD) by introducing
//! *compliance* — a physically meaningful stiffness parameter that makes
//! constraint behaviour independent of iteration count and timestep.
//!
//! Each step:
//! 1. Apply external forces (gravity) → update velocities
//! 2. Predict positions: `x_pred = x + v * dt`
//! 3. Gauss-Seidel iterate over all constraints
//! 4. Update velocities: `v = (x_new - x_old) / dt`
//! 5. Apply velocity damping
//!
//! # Example
//!
//! ```rust
//! use oxihuman_physics::xpbd_unified::{XpbdSolver, XpbdConfig, DistanceConstraint};
//!
//! let mut solver = XpbdSolver::new(XpbdConfig {
//!     dt: 1.0 / 60.0,
//!     gravity: [0.0, -9.81, 0.0],
//!     iterations: 10,
//!     damping: 0.01,
//!     substeps: 1,
//! });
//!
//! let a = solver.add_particle([0.0, 2.0, 0.0], 0.0);  // fixed
//! let b = solver.add_particle([0.0, 1.0, 0.0], 1.0);
//! solver.add_constraint(Box::new(DistanceConstraint::new(a, b, 1.0, 0.0)));
//!
//! solver.step().unwrap();
//! ```

pub mod builtin_constraints;
pub mod constraint;
pub mod solver;

// ── Re-exports for convenience ──────────────────────────────────────────────

pub use builtin_constraints::{
    CollisionConstraint, DistanceConstraint, ParticleCollisionConstraint,
    ShapeMatchingConstraint, VolumeConstraint,
};
pub use constraint::{ConstraintId, XpbdConstraint};
pub use solver::{XpbdConfig, XpbdSolver};
