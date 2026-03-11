// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Adipose tissue jiggle simulation (overdamped oscillator).

/// An adipose tissue volume node.
#[derive(Debug, Clone)]
pub struct AdiposNode {
    /// Current position offset from rest.
    pub offset: [f32; 3],
    /// Velocity.
    pub velocity: [f32; 3],
    /// Spring stiffness.
    pub stiffness: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Mass (kg).
    pub mass: f32,
}

impl AdiposNode {
    pub fn new(mass: f32, stiffness: f32, damping: f32) -> Self {
        AdiposNode {
            offset: [0.0; 3],
            velocity: [0.0; 3],
            stiffness,
            damping,
            mass,
        }
    }
}

/// Create a new adipose simulation node.
pub fn new_adipos_node(mass: f32, stiffness: f32, damping: f32) -> AdiposNode {
    AdiposNode::new(mass, stiffness, damping)
}

/// Step the adipose jiggle simulation given external acceleration `acc`.
#[allow(clippy::needless_range_loop)]
pub fn adipos_step(node: &mut AdiposNode, acc: [f32; 3], dt: f32) {
    for k in 0..3 {
        /* spring force + damping + external drive */
        let f =
            -node.stiffness * node.offset[k] - node.damping * node.velocity[k] + node.mass * acc[k];
        let a = f / node.mass.max(1e-10);
        node.velocity[k] += a * dt;
        node.offset[k] += node.velocity[k] * dt;
    }
}

/// Return the magnitude of the current offset.
pub fn adipos_offset_mag(node: &AdiposNode) -> f32 {
    let o = node.offset;
    (o[0] * o[0] + o[1] * o[1] + o[2] * o[2]).sqrt()
}

/// Return the kinetic energy of the jiggle motion.
pub fn adipos_kinetic_energy(node: &AdiposNode) -> f32 {
    let v = node.velocity;
    0.5 * node.mass * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
}

/// Return the potential energy stored in the spring.
pub fn adipos_potential_energy(node: &AdiposNode) -> f32 {
    let o = node.offset;
    0.5 * node.stiffness * (o[0] * o[0] + o[1] * o[1] + o[2] * o[2])
}

/// Reset node to rest position.
pub fn adipos_reset(node: &mut AdiposNode) {
    node.offset = [0.0; 3];
    node.velocity = [0.0; 3];
}

/// Return `true` if the node is approximately at rest (offset < `tol`).
pub fn adipos_is_settled(node: &AdiposNode, tol: f32) -> bool {
    adipos_offset_mag(node) < tol
        && (node.velocity[0]
            .abs()
            .max(node.velocity[1].abs())
            .max(node.velocity[2].abs()))
            < tol
}

/// Apply a short impulse (delta-v) to the node.
#[allow(clippy::needless_range_loop)]
pub fn adipos_impulse(node: &mut AdiposNode, impulse: [f32; 3]) {
    let inv_m = 1.0 / node.mass.max(1e-10);
    for k in 0..3 {
        node.velocity[k] += impulse[k] * inv_m;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_node_at_rest() {
        let n = new_adipos_node(0.5, 100.0, 10.0);
        assert!(adipos_is_settled(&n, 1e-4));
    }

    #[test]
    fn test_impulse_moves_node() {
        let mut n = new_adipos_node(0.5, 100.0, 10.0);
        adipos_impulse(&mut n, [1.0, 0.0, 0.0]);
        adipos_step(&mut n, [0.0; 3], 0.01);
        assert!(adipos_offset_mag(&n) > 0.0);
    }

    #[test]
    fn test_kinetic_energy_after_impulse() {
        let mut n = new_adipos_node(0.5, 100.0, 10.0);
        adipos_impulse(&mut n, [2.0, 0.0, 0.0]);
        assert!(adipos_kinetic_energy(&n) > 0.0);
    }

    #[test]
    fn test_damping_settles_motion() {
        let mut n = new_adipos_node(1.0, 50.0, 20.0);
        adipos_impulse(&mut n, [1.0, 0.0, 0.0]);
        for _ in 0..500 {
            adipos_step(&mut n, [0.0; 3], 0.01);
        }
        assert!(adipos_is_settled(&n, 0.01));
    }

    #[test]
    fn test_potential_energy_after_offset() {
        let mut n = new_adipos_node(1.0, 100.0, 1.0);
        n.offset = [0.1, 0.0, 0.0];
        assert!(adipos_potential_energy(&n) > 0.0);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut n = new_adipos_node(1.0, 100.0, 10.0);
        adipos_impulse(&mut n, [5.0, 5.0, 5.0]);
        adipos_reset(&mut n);
        assert!(adipos_is_settled(&n, 1e-4));
    }

    #[test]
    fn test_external_acc_drives_motion() {
        let mut n = new_adipos_node(1.0, 10.0, 0.5);
        for _ in 0..20 {
            adipos_step(&mut n, [5.0, 0.0, 0.0], 0.01);
        }
        assert!(n.offset[0].abs() > 0.0);
    }

    #[test]
    fn test_offset_magnitude_nonnegative() {
        let n = new_adipos_node(1.0, 100.0, 10.0);
        assert!(adipos_offset_mag(&n) >= 0.0);
    }

    #[test]
    fn test_energy_conserved_approximately() {
        /* with very low damping, KE+PE should not grow spontaneously */
        let mut n = new_adipos_node(1.0, 100.0, 0.001);
        adipos_impulse(&mut n, [1.0, 0.0, 0.0]);
        let e0 = adipos_kinetic_energy(&n) + adipos_potential_energy(&n);
        for _ in 0..100 {
            adipos_step(&mut n, [0.0; 3], 0.001);
        }
        let e1 = adipos_kinetic_energy(&n) + adipos_potential_energy(&n);
        assert!(e1 <= e0 + 0.05); /* forward Euler adds a small amount; bounded */
    }
}
