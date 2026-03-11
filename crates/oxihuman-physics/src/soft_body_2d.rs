// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D soft body simulation using mass-spring model.

#[derive(Debug, Clone)]
pub struct SoftNode2d {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub mass: f32,
    pub fixed: bool,
}

impl SoftNode2d {
    pub fn new(x: f32, y: f32, mass: f32) -> Self {
        SoftNode2d {
            position: [x, y],
            velocity: [0.0; 2],
            mass,
            fixed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SoftEdge2d {
    pub i: usize,
    pub j: usize,
    pub rest: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl SoftEdge2d {
    pub fn new(i: usize, j: usize, rest: f32, stiffness: f32) -> Self {
        SoftEdge2d {
            i,
            j,
            rest,
            stiffness,
            damping: 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SoftBody2d {
    pub nodes: Vec<SoftNode2d>,
    pub edges: Vec<SoftEdge2d>,
    pub gravity: f32,
}

impl SoftBody2d {
    pub fn new(gravity: f32) -> Self {
        SoftBody2d {
            nodes: Vec::new(),
            edges: Vec::new(),
            gravity,
        }
    }

    pub fn add_node(&mut self, n: SoftNode2d) -> usize {
        self.nodes.push(n);
        self.nodes.len() - 1
    }

    pub fn add_edge(&mut self, e: SoftEdge2d) {
        self.edges.push(e);
    }

    pub fn step(&mut self, dt: f32) {
        let n = self.nodes.len();
        let mut forces = vec![[0.0f32; 2]; n];
        for (k, nd) in self.nodes.iter().enumerate() {
            if !nd.fixed {
                forces[k][1] -= nd.mass * self.gravity;
            }
        }
        let edges = self.edges.clone();
        for e in &edges {
            let pa = self.nodes[e.i].position;
            let pb = self.nodes[e.j].position;
            let va = self.nodes[e.i].velocity;
            let vb = self.nodes[e.j].velocity;
            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let dist = (dx * dx + dy * dy).sqrt().max(1e-8);
            let nx = dx / dist;
            let ny = dy / dist;
            let stretch = dist - e.rest;
            let rel_vn = (vb[0] - va[0]) * nx + (vb[1] - va[1]) * ny;
            let fmag = e.stiffness * stretch + e.damping * rel_vn;
            forces[e.i][0] += fmag * nx;
            forces[e.i][1] += fmag * ny;
            forces[e.j][0] -= fmag * nx;
            forces[e.j][1] -= fmag * ny;
        }
        for (nd, f) in self.nodes.iter_mut().zip(forces.iter()) {
            if nd.fixed {
                continue;
            }
            nd.velocity[0] += f[0] / nd.mass * dt;
            nd.velocity[1] += f[1] / nd.mass * dt;
            nd.position[0] += nd.velocity[0] * dt;
            nd.position[1] += nd.velocity[1] * dt;
        }
    }

    pub fn total_kinetic_energy(&self) -> f32 {
        self.nodes
            .iter()
            .map(|n| 0.5 * n.mass * (n.velocity[0].powi(2) + n.velocity[1].powi(2)))
            .sum()
    }

    pub fn center_of_mass(&self) -> [f32; 2] {
        let total_mass: f32 = self.nodes.iter().map(|n| n.mass).sum();
        if total_mass < f32::EPSILON {
            return [0.0; 2];
        }
        let cx = self
            .nodes
            .iter()
            .map(|n| n.mass * n.position[0])
            .sum::<f32>()
            / total_mass;
        let cy = self
            .nodes
            .iter()
            .map(|n| n.mass * n.position[1])
            .sum::<f32>()
            / total_mass;
        [cx, cy]
    }
}

pub fn make_soft_square_2d(cx: f32, cy: f32, half: f32, mass: f32, stiffness: f32) -> SoftBody2d {
    let mut body = SoftBody2d::new(9.81);
    let corners = [[-half, -half], [half, -half], [half, half], [-half, half]];
    for [ox, oy] in corners {
        body.add_node(SoftNode2d::new(cx + ox, cy + oy, mass));
    }
    let edges = [(0, 1), (1, 2), (2, 3), (3, 0), (0, 2), (1, 3)];
    for (i, j) in edges {
        let pa = body.nodes[i].position;
        let pb = body.nodes[j].position;
        let rest = ((pb[0] - pa[0]).powi(2) + (pb[1] - pa[1]).powi(2)).sqrt();
        body.add_edge(SoftEdge2d::new(i, j, rest, stiffness));
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_creation() {
        let body = make_soft_square_2d(0.0, 0.0, 1.0, 1.0, 100.0);
        assert_eq!(body.nodes.len(), 4);
        assert_eq!(body.edges.len(), 6);
    }

    #[test]
    fn test_step_runs() {
        let mut body = make_soft_square_2d(0.0, 0.0, 1.0, 1.0, 100.0);
        body.step(0.01);
        assert!(body.nodes[0].position[1].is_finite(), /* finite position */);
    }

    #[test]
    fn test_gravity_moves_nodes_down() {
        let mut body = make_soft_square_2d(0.0, 10.0, 1.0, 1.0, 100.0);
        let y0 = body.nodes[0].position[1];
        body.step(0.1);
        assert!(body.nodes[0].position[1] < y0 /* gravity pulls down */,);
    }

    #[test]
    fn test_fixed_node_stable() {
        let mut body = SoftBody2d::new(9.81);
        let mut n = SoftNode2d::new(0.0, 0.0, 1.0);
        n.fixed = true;
        body.add_node(n);
        body.step(1.0);
        assert!((body.nodes[0].position[1] - 0.0).abs() < 1e-6, /* fixed node stays */);
    }

    #[test]
    fn test_center_of_mass() {
        let body = make_soft_square_2d(0.0, 0.0, 1.0, 1.0, 100.0);
        let com = body.center_of_mass();
        assert!(com[0].abs() < 1e-4 && com[1].abs() < 1e-4, /* COM at center */);
    }

    #[test]
    fn test_kinetic_energy_at_rest() {
        let body = make_soft_square_2d(0.0, 0.0, 1.0, 1.0, 100.0);
        assert!((body.total_kinetic_energy() - 0.0).abs() < 1e-6, /* initially at rest */);
    }

    #[test]
    fn test_kinetic_energy_after_step() {
        let mut body = make_soft_square_2d(0.0, 10.0, 1.0, 1.0, 100.0);
        body.step(0.1);
        assert!(body.total_kinetic_energy() >= 0.0, /* KE non-negative */);
    }

    #[test]
    fn test_add_node_and_edge() {
        let mut body = SoftBody2d::new(9.81);
        let i = body.add_node(SoftNode2d::new(0.0, 0.0, 1.0));
        let j = body.add_node(SoftNode2d::new(1.0, 0.0, 1.0));
        body.add_edge(SoftEdge2d::new(i, j, 1.0, 100.0));
        assert_eq!(body.edges.len(), 1);
    }

    #[test]
    fn test_multi_step() {
        let mut body = make_soft_square_2d(0.0, 0.0, 1.0, 1.0, 100.0);
        for _ in 0..50 {
            body.step(0.01);
        }
        for n in &body.nodes {
            assert!(
                n.position[0].is_finite() && n.position[1].is_finite(),
                /* all positions remain finite */
            );
        }
    }
}
