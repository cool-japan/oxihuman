// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deformable solid with linear elastic energy.

#![allow(dead_code)]

/// A deformable solid node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SolidNode {
    pub position: [f64; 3],
    pub rest_position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass: f64,
    pub force: [f64; 3],
}

/// Linear elastic deformable solid.
#[allow(dead_code)]
pub struct DeformableSolid {
    pub nodes: Vec<SolidNode>,
    pub young_modulus: f64,
    pub poisson_ratio: f64,
    pub damping: f64,
}

impl SolidNode {
    #[allow(dead_code)]
    pub fn new(position: [f64; 3], mass: f64) -> Self {
        Self {
            rest_position: position,
            position,
            velocity: [0.0; 3],
            mass,
            force: [0.0; 3],
        }
    }

    /// Displacement from rest.
    #[allow(dead_code)]
    pub fn displacement(&self) -> [f64; 3] {
        [
            self.position[0] - self.rest_position[0],
            self.position[1] - self.rest_position[1],
            self.position[2] - self.rest_position[2],
        ]
    }

    /// Displacement magnitude.
    #[allow(dead_code)]
    pub fn displacement_mag(&self) -> f64 {
        let d = self.displacement();
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    }
}

impl DeformableSolid {
    #[allow(dead_code)]
    pub fn new(young: f64, poisson: f64, damping: f64) -> Self {
        Self {
            nodes: Vec::new(),
            young_modulus: young,
            poisson_ratio: poisson.clamp(-1.0, 0.5),
            damping,
        }
    }

    #[allow(dead_code)]
    pub fn add_node(&mut self, pos: [f64; 3], mass: f64) -> usize {
        let id = self.nodes.len();
        self.nodes.push(SolidNode::new(pos, mass));
        id
    }

    /// Apply restoring elastic force to each node (linear spring to rest).
    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn apply_elastic_forces(&mut self) {
        let stiffness = self.young_modulus;
        let damping = self.damping;
        for node in &mut self.nodes {
            let d = node.displacement();
            for axis in 0..3 {
                node.force[axis] += -stiffness * d[axis] - damping * node.velocity[axis];
            }
        }
    }

    /// Integrate forces with explicit Euler step.
    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f64) {
        self.apply_elastic_forces();
        for node in &mut self.nodes {
            if node.mass > 0.0 {
                for axis in 0..3 {
                    let acc = node.force[axis] / node.mass;
                    node.velocity[axis] += acc * dt;
                    node.position[axis] += node.velocity[axis] * dt;
                    node.force[axis] = 0.0;
                }
            }
        }
    }

    /// Total elastic potential energy (sum of 0.5 * k * d^2).
    #[allow(dead_code)]
    pub fn elastic_energy(&self) -> f64 {
        let k = self.young_modulus;
        self.nodes.iter().map(|n| {
            let d2: f64 = n.displacement().iter().map(|&x| x * x).sum();
            0.5 * k * d2
        }).sum()
    }

    /// Total kinetic energy.
    #[allow(dead_code)]
    pub fn kinetic_energy(&self) -> f64 {
        self.nodes.iter().map(|n| {
            let v2: f64 = n.velocity.iter().map(|&x| x * x).sum();
            0.5 * n.mass * v2
        }).sum()
    }

    #[allow(dead_code)]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Compute Lame parameters from Young's modulus and Poisson's ratio.
#[allow(dead_code)]
pub fn lame_parameters(young: f64, poisson: f64) -> (f64, f64) {
    let mu = young / (2.0 * (1.0 + poisson));
    let lambda = young * poisson / ((1.0 + poisson) * (1.0 - 2.0 * poisson));
    (lambda, mu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut solid = DeformableSolid::new(1000.0, 0.3, 0.1);
        solid.add_node([0.0, 0.0, 0.0], 1.0);
        assert_eq!(solid.node_count(), 1);
    }

    #[test]
    fn test_displacement_zero_at_rest() {
        let node = SolidNode::new([1.0, 2.0, 3.0], 1.0);
        let d = node.displacement();
        assert!(d.iter().all(|&x| x.abs() < 1e-12));
    }

    #[test]
    fn test_displacement_magnitude() {
        let mut node = SolidNode::new([0.0, 0.0, 0.0], 1.0);
        node.position = [3.0, 4.0, 0.0];
        assert!((node.displacement_mag() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_elastic_energy_zero_at_rest() {
        let mut solid = DeformableSolid::new(100.0, 0.3, 0.0);
        solid.add_node([0.0, 0.0, 0.0], 1.0);
        assert!((solid.elastic_energy()).abs() < 1e-12);
    }

    #[test]
    fn test_elastic_energy_nonzero_displaced() {
        let mut solid = DeformableSolid::new(100.0, 0.3, 0.0);
        let id = solid.add_node([0.0, 0.0, 0.0], 1.0);
        solid.nodes[id].position = [1.0, 0.0, 0.0];
        assert!(solid.elastic_energy() > 0.0);
    }

    #[test]
    fn test_step_restores_displaced_node() {
        let mut solid = DeformableSolid::new(10.0, 0.3, 0.5);
        let id = solid.add_node([0.0, 0.0, 0.0], 1.0);
        solid.nodes[id].position = [1.0, 0.0, 0.0];
        for _ in 0..100 {
            solid.step(0.01);
        }
        assert!(solid.nodes[id].displacement_mag() < 1.0);
    }

    #[test]
    fn test_lame_parameters() {
        let (lambda, mu) = lame_parameters(200e9, 0.3);
        assert!(lambda > 0.0 && mu > 0.0);
    }

    #[test]
    fn test_poisson_clamped() {
        let solid = DeformableSolid::new(100.0, 0.6, 0.0);
        assert!(solid.poisson_ratio <= 0.5);
    }

    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let mut solid = DeformableSolid::new(100.0, 0.3, 0.0);
        solid.add_node([0.0, 0.0, 0.0], 1.0);
        assert!((solid.kinetic_energy()).abs() < 1e-12);
    }

    #[test]
    fn test_multiple_nodes() {
        let mut solid = DeformableSolid::new(100.0, 0.3, 0.1);
        for i in 0..5 {
            solid.add_node([i as f64, 0.0, 0.0], 1.0);
        }
        assert_eq!(solid.node_count(), 5);
    }
}
