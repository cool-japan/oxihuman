// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Soft tissue visco-elastic deformation model stub.
//!
//! Models soft biological tissue using a Kelvin-Voigt visco-elastic element
//! with separate elastic and viscous stiffness parameters.

/// Parameters for the tissue material.
#[derive(Debug, Clone)]
pub struct TissueParams {
    /// Elastic (spring) stiffness `[Pa]`.
    pub elastic_modulus: f64,
    /// Viscous (dashpot) stiffness [Pa·s].
    pub viscous_coefficient: f64,
    /// Density [kg/m³].
    pub density: f64,
    /// Poisson ratio.
    pub poisson_ratio: f64,
}

impl Default for TissueParams {
    fn default() -> Self {
        Self {
            elastic_modulus: 3000.0,
            viscous_coefficient: 50.0,
            density: 1060.0,
            poisson_ratio: 0.45,
        }
    }
}

/// State of a single tissue node.
#[derive(Debug, Clone, Default)]
pub struct TissueNode {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub force: [f64; 3],
    pub mass: f64,
}

impl TissueNode {
    pub fn new(pos: [f64; 3], mass: f64) -> Self {
        Self {
            position: pos,
            velocity: [0.0; 3],
            force: [0.0; 3],
            mass,
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn apply_force(&mut self, f: [f64; 3]) {
        for i in 0..3 {
            self.force[i] += f[i];
        }
    }

    pub fn clear_forces(&mut self) {
        self.force = [0.0; 3];
    }

    pub fn kinetic_energy(&self) -> f64 {
        let v2: f64 = self.velocity.iter().map(|v| v * v).sum();
        0.5 * self.mass * v2
    }
}

/// A tissue mesh consisting of nodes and edges.
#[derive(Debug, Clone)]
pub struct TissueMesh {
    pub nodes: Vec<TissueNode>,
    pub edges: Vec<(usize, usize)>,
    pub params: TissueParams,
}

impl TissueMesh {
    pub fn new(params: TissueParams) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            params,
        }
    }

    pub fn add_node(&mut self, pos: [f64; 3], mass: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(TissueNode::new(pos, mass));
        idx
    }

    pub fn add_edge(&mut self, a: usize, b: usize) {
        self.edges.push((a, b));
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Total kinetic energy.
    pub fn kinetic_energy(&self) -> f64 {
        self.nodes.iter().map(|n| n.kinetic_energy()).sum()
    }
}

/// Compute the rest length of an edge.
pub fn rest_length(mesh: &TissueMesh, edge: (usize, usize)) -> f64 {
    let a = &mesh.nodes[edge.0].position;
    let b = &mesh.nodes[edge.1].position;
    let d: f64 = (0..3).map(|i| (b[i] - a[i]).powi(2)).sum::<f64>().sqrt();
    d.max(1e-12)
}

/// Apply visco-elastic spring forces between connected nodes.
pub fn apply_tissue_forces(mesh: &mut TissueMesh, rest_lengths: &[f64]) {
    let e_mod = mesh.params.elastic_modulus;
    let visc = mesh.params.viscous_coefficient;

    for (idx, &(a, b)) in mesh.edges.iter().enumerate() {
        let rest = rest_lengths.get(idx).copied().unwrap_or(1.0);
        let pa = mesh.nodes[a].position;
        let pb = mesh.nodes[b].position;
        let va = mesh.nodes[a].velocity;
        let vb = mesh.nodes[b].velocity;

        let mut diff = [0.0f64; 3];
        let mut vel_diff = [0.0f64; 3];
        for i in 0..3 {
            diff[i] = pb[i] - pa[i];
            vel_diff[i] = vb[i] - va[i];
        }
        let len: f64 = diff.iter().map(|d| d * d).sum::<f64>().sqrt().max(1e-12);
        let dir: [f64; 3] = [diff[0] / len, diff[1] / len, diff[2] / len];
        let strain = (len - rest) / rest;
        let strain_rate: f64 = vel_diff
            .iter()
            .zip(dir.iter())
            .map(|(v, d)| v * d)
            .sum::<f64>()
            / rest;

        let force_mag = e_mod * strain + visc * strain_rate;
        let force: [f64; 3] = [dir[0] * force_mag, dir[1] * force_mag, dir[2] * force_mag];

        mesh.nodes[a].apply_force(force);
        mesh.nodes[b].apply_force([-force[0], -force[1], -force[2]]);
    }
}

/// Integrate tissue nodes forward by `dt` seconds (semi-implicit Euler).
pub fn integrate_tissue(mesh: &mut TissueMesh, dt: f64) {
    for node in &mut mesh.nodes {
        if node.mass <= 0.0 {
            continue;
        }
        let inv_mass = 1.0 / node.mass;
        for i in 0..3 {
            node.velocity[i] += node.force[i] * inv_mass * dt;
            node.position[i] += node.velocity[i] * dt;
        }
        node.clear_forces();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> TissueMesh {
        let mut m = TissueMesh::new(TissueParams::default());
        let a = m.add_node([0.0, 0.0, 0.0], 1.0);
        let b = m.add_node([1.0, 0.0, 0.0], 1.0);
        m.add_edge(a, b);
        m
    }

    #[test]
    fn test_node_count() {
        let m = simple_mesh();
        assert_eq!(m.node_count(), 2);
    }

    #[test]
    fn test_edge_count() {
        let m = simple_mesh();
        assert_eq!(m.edge_count(), 1);
    }

    #[test]
    fn test_rest_length() {
        let m = simple_mesh();
        let rl = rest_length(&m, (0, 1));
        assert!((rl - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_kinetic_energy_zero_initially() {
        let m = simple_mesh();
        assert_eq!(m.kinetic_energy(), 0.0);
    }

    #[test]
    fn test_apply_forces_no_panic() {
        let mut m = simple_mesh();
        let rest = vec![1.0];
        apply_tissue_forces(&mut m, &rest);
    }

    #[test]
    fn test_integrate_moves_node() {
        let mut m = simple_mesh();
        m.nodes[0].force[0] = 10.0;
        let pos_before = m.nodes[0].position[0];
        integrate_tissue(&mut m, 0.01);
        assert!(m.nodes[0].position[0] != pos_before || m.nodes[0].velocity[0] != 0.0);
    }

    #[test]
    fn test_apply_force_accumulates() {
        let mut n = TissueNode::new([0.0; 3], 1.0);
        n.apply_force([1.0, 0.0, 0.0]);
        n.apply_force([2.0, 0.0, 0.0]);
        assert!((n.force[0] - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_clear_forces() {
        let mut n = TissueNode::new([0.0; 3], 1.0);
        n.apply_force([5.0, 5.0, 5.0]);
        n.clear_forces();
        assert_eq!(n.force, [0.0; 3]);
    }

    #[test]
    fn test_default_params() {
        let p = TissueParams::default();
        assert!(p.elastic_modulus > 0.0);
        assert!((0.0..=0.5).contains(&p.poisson_ratio));
    }
}
