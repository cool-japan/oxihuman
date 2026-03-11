// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Articulated rigid-body tree using Featherstone-style recursive dynamics stub.

/// A single body in the articulated tree.
#[derive(Debug, Clone)]
pub struct ArtBody {
    pub mass: f32,
    pub inertia: [f32; 6],
    pub parent: Option<usize>,
    pub joint_axis: [f32; 3],
}

/// Articulated rigid-body tree.
#[derive(Debug, Clone, Default)]
pub struct ArtBodyTree {
    pub bodies: Vec<ArtBody>,
    pub positions: Vec<f32>,
    pub velocities: Vec<f32>,
}

impl ArtBodyTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_body(&mut self, body: ArtBody) -> usize {
        let idx = self.bodies.len();
        self.positions.push(0.0);
        self.velocities.push(0.0);
        self.bodies.push(body);
        idx
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
}

/// Perform one step of forward dynamics (stub: apply trivial integration).
#[allow(clippy::needless_range_loop)]
pub fn forward_dynamics_step(tree: &mut ArtBodyTree, tau: &[f32], dt: f32) {
    /* stub: each body's velocity += tau * dt / mass */
    for i in 0..tree.bodies.len().min(tau.len()) {
        let mass = tree.bodies[i].mass.max(1e-6);
        tree.velocities[i] += tau[i] / mass * dt;
        tree.positions[i] += tree.velocities[i] * dt;
    }
}

/// Compute the total kinetic energy of the tree (stub).
pub fn total_kinetic_energy(tree: &ArtBodyTree) -> f32 {
    /* 0.5 * sum(m * v^2) */
    tree.bodies
        .iter()
        .zip(tree.velocities.iter())
        .map(|(b, &v)| 0.5 * b.mass * v * v)
        .sum()
}

/// Return the index of the root body (the first body with no parent).
pub fn find_root(tree: &ArtBodyTree) -> Option<usize> {
    tree.bodies.iter().position(|b| b.parent.is_none())
}

/// Count the number of leaf bodies (no body has them as a parent).
pub fn count_leaves(tree: &ArtBodyTree) -> usize {
    let parents: std::collections::HashSet<usize> =
        tree.bodies.iter().filter_map(|b| b.parent).collect();
    (0..tree.bodies.len())
        .filter(|i| !parents.contains(i))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tree() -> ArtBodyTree {
        let mut t = ArtBodyTree::new();
        t.add_body(ArtBody {
            mass: 1.0,
            inertia: [1.0; 6],
            parent: None,
            joint_axis: [0.0, 1.0, 0.0],
        });
        t.add_body(ArtBody {
            mass: 2.0,
            inertia: [1.0; 6],
            parent: Some(0),
            joint_axis: [0.0, 1.0, 0.0],
        });
        t
    }

    #[test]
    fn test_body_count() {
        /* two bodies added */
        let t = simple_tree();
        assert_eq!(t.body_count(), 2);
    }

    #[test]
    fn test_find_root() {
        /* root has no parent */
        let t = simple_tree();
        assert_eq!(find_root(&t), Some(0));
    }

    #[test]
    fn test_count_leaves() {
        /* only body 1 is a leaf */
        let t = simple_tree();
        assert_eq!(count_leaves(&t), 1);
    }

    #[test]
    fn test_forward_dynamics_step() {
        /* velocities change after step */
        let mut t = simple_tree();
        forward_dynamics_step(&mut t, &[1.0, 1.0], 0.01);
        assert!(t.velocities[0] > 0.0);
    }

    #[test]
    fn test_kinetic_energy_zero_initially() {
        /* all velocities are zero initially */
        let t = simple_tree();
        assert_eq!(total_kinetic_energy(&t), 0.0);
    }

    #[test]
    fn test_kinetic_energy_nonzero_after_step() {
        /* energy increases after applying force */
        let mut t = simple_tree();
        forward_dynamics_step(&mut t, &[10.0, 10.0], 0.1);
        assert!(total_kinetic_energy(&t) > 0.0);
    }

    #[test]
    fn test_add_body_positions_len() {
        /* positions vector grows with each body */
        let t = simple_tree();
        assert_eq!(t.positions.len(), 2);
    }

    #[test]
    fn test_empty_tree_no_root() {
        /* empty tree has no root */
        let t = ArtBodyTree::new();
        assert!(find_root(&t).is_none());
    }

    #[test]
    fn test_empty_tree_no_leaves() {
        /* empty tree has no leaves */
        let t = ArtBodyTree::new();
        assert_eq!(count_leaves(&t), 0);
    }
}
