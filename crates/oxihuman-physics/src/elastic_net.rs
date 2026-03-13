#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Elastic net: a network of elastic edges connecting particle positions.

/// A single elastic edge between two particles.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ElasticEdge {
    pub particle_a: usize,
    pub particle_b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
}

/// An elastic net with particles and edges.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ElasticNet {
    pub positions: Vec<[f32; 3]>,
    pub edges: Vec<ElasticEdge>,
}

/// Create a new empty `ElasticNet`.
#[allow(dead_code)]
pub fn new_elastic_net() -> ElasticNet {
    ElasticNet::default()
}

/// Add an elastic edge.
#[allow(dead_code)]
pub fn add_elastic_edge(net: &mut ElasticNet, pa: usize, pb: usize, rest_length: f32, stiffness: f32) {
    net.edges.push(ElasticEdge { particle_a: pa, particle_b: pb, rest_length, stiffness });
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute the net elastic force magnitude on all edges.
#[allow(dead_code)]
pub fn elastic_net_force(net: &ElasticNet) -> f32 {
    net.edges.iter().map(|e| {
        if e.particle_a < net.positions.len() && e.particle_b < net.positions.len() {
            let d = dist3(net.positions[e.particle_a], net.positions[e.particle_b]);
            e.stiffness * (d - e.rest_length).abs()
        } else {
            0.0
        }
    }).sum()
}

/// Return the number of edges.
#[allow(dead_code)]
pub fn edge_count(net: &ElasticNet) -> usize {
    net.edges.len()
}

/// Compute total elastic potential energy.
#[allow(dead_code)]
pub fn net_energy(net: &ElasticNet) -> f32 {
    net.edges.iter().map(|e| {
        if e.particle_a < net.positions.len() && e.particle_b < net.positions.len() {
            let d = dist3(net.positions[e.particle_a], net.positions[e.particle_b]);
            let stretch = d - e.rest_length;
            0.5 * e.stiffness * stretch * stretch
        } else {
            0.0
        }
    }).sum()
}

/// Return the current length of an edge.
#[allow(dead_code)]
pub fn edge_length(net: &ElasticNet, edge_idx: usize) -> f32 {
    if let Some(e) = net.edges.get(edge_idx) {
        if e.particle_a < net.positions.len() && e.particle_b < net.positions.len() {
            return dist3(net.positions[e.particle_a], net.positions[e.particle_b]);
        }
    }
    0.0
}

/// Return the rest length of an edge.
#[allow(dead_code)]
pub fn edge_rest_length(net: &ElasticNet, edge_idx: usize) -> f32 {
    net.edges.get(edge_idx).map(|e| e.rest_length).unwrap_or(0.0)
}

/// Apply a single Euler step to settle the net (moves particles along force gradient).
#[allow(dead_code)]
pub fn net_step(net: &mut ElasticNet, dt: f32) {
    let n = net.positions.len();
    let mut forces = vec![[0.0f32; 3]; n];
    for e in &net.edges {
        if e.particle_a >= n || e.particle_b >= n { continue; }
        let pa = net.positions[e.particle_a];
        let pb = net.positions[e.particle_b];
        let dx = pb[0] - pa[0];
        let dy = pb[1] - pa[1];
        let dz = pb[2] - pa[2];
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
        let stretch = len - e.rest_length;
        let f = e.stiffness * stretch / len;
        forces[e.particle_a][0] += f * dx;
        forces[e.particle_a][1] += f * dy;
        forces[e.particle_a][2] += f * dz;
        forces[e.particle_b][0] -= f * dx;
        forces[e.particle_b][1] -= f * dy;
        forces[e.particle_b][2] -= f * dz;
    }
    for (i, p) in net.positions.iter_mut().enumerate() {
        p[0] += forces[i][0] * dt;
        p[1] += forces[i][1] * dt;
        p[2] += forces[i][2] * dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_elastic_net() {
        let net = new_elastic_net();
        assert_eq!(edge_count(&net), 0);
    }

    #[test]
    fn test_add_elastic_edge() {
        let mut net = new_elastic_net();
        add_elastic_edge(&mut net, 0, 1, 1.0, 10.0);
        assert_eq!(edge_count(&net), 1);
    }

    #[test]
    fn test_edge_rest_length() {
        let mut net = new_elastic_net();
        add_elastic_edge(&mut net, 0, 1, 2.5, 1.0);
        assert!((edge_rest_length(&net, 0) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_at_rest_no_energy() {
        let mut net = new_elastic_net();
        net.positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        add_elastic_edge(&mut net, 0, 1, 1.0, 10.0);
        assert!(net_energy(&net) < 1e-6);
    }

    #[test]
    fn test_stretched_has_energy() {
        let mut net = new_elastic_net();
        net.positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        add_elastic_edge(&mut net, 0, 1, 1.0, 10.0);
        assert!(net_energy(&net) > 0.0);
    }

    #[test]
    fn test_edge_length() {
        let mut net = new_elastic_net();
        net.positions = vec![[0.0, 0.0, 0.0], [3.0, 4.0, 0.0]];
        add_elastic_edge(&mut net, 0, 1, 1.0, 1.0);
        assert!((edge_length(&net, 0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_net_step_reduces_energy() {
        let mut net = new_elastic_net();
        net.positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        add_elastic_edge(&mut net, 0, 1, 1.0, 1.0);
        let e0 = net_energy(&net);
        net_step(&mut net, 0.1);
        let e1 = net_energy(&net);
        assert!(e1 < e0);
    }

    #[test]
    fn test_elastic_net_force_zero_at_rest() {
        let mut net = new_elastic_net();
        net.positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        add_elastic_edge(&mut net, 0, 1, 1.0, 5.0);
        assert!(elastic_net_force(&net) < 1e-5);
    }
}
