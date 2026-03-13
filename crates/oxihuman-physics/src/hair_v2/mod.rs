// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hair strand dynamics v2 — Cosserat rod model with XPBD solver.
//!
//! Each hair strand is modeled as a discrete Cosserat rod where every node
//! carries both a position `[f64; 3]` and an orientation quaternion `[f64; 4]`.
//! The solver applies stretch-twist, bend-twist, and length-preservation
//! constraints via eXtended Position-Based Dynamics (XPBD), and supports
//! shape matching for rest-pose recovery plus capsule-based hair-body collision.

pub mod collision;
pub mod constraints;
pub mod solver;
pub mod strand;

pub use collision::{BodyCapsule, HairCollisionConfig};
pub use constraints::{BendTwistConstraint, StretchTwistConstraint};
pub use solver::XpbdHairSolver;
pub use strand::{HairConfigV2, HairNode, HairStrand, HairSystemV2};
