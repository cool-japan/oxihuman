// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lattice spring model — a network of springs on a regular lattice used to
//! simulate elasticity and fracture via bond breaking.

/// A lattice node.
#[derive(Debug, Clone)]
pub struct LatticeNode {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub mass: f64,
    pub pinned: bool,
}

impl LatticeNode {
    pub fn new(x: f64, y: f64, mass: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            mass,
            pinned: false,
        }
    }
}

/// A spring bond between two lattice nodes.
#[derive(Debug, Clone)]
pub struct LatticeBond {
    pub i: usize,
    pub j: usize,
    pub rest_length: f64,
    pub stiffness: f64,
    pub broken: bool,
}

/// Lattice spring model.
pub struct LatticeSpringModel {
    pub nodes: Vec<LatticeNode>,
    pub bonds: Vec<LatticeBond>,
    pub damping: f64,
}

impl LatticeSpringModel {
    /// Create an empty lattice spring model.
    pub fn new(damping: f64) -> Self {
        Self {
            nodes: Vec::new(),
            bonds: Vec::new(),
            damping,
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, x: f64, y: f64, mass: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(LatticeNode::new(x, y, mass));
        idx
    }

    /// Add a spring bond.
    pub fn add_bond(&mut self, i: usize, j: usize, stiffness: f64) {
        let dx = self.nodes[j].x - self.nodes[i].x;
        let dy = self.nodes[j].y - self.nodes[i].y;
        let rest = (dx * dx + dy * dy).sqrt();
        self.bonds.push(LatticeBond {
            i,
            j,
            rest_length: rest,
            stiffness,
            broken: false,
        });
    }

    /// Apply spring forces and integrate (Euler step).
    pub fn step(&mut self, dt: f64, critical_strain: f64) {
        let mut fx = vec![0.0f64; self.nodes.len()];
        let mut fy = vec![0.0f64; self.nodes.len()];
        for bond in &mut self.bonds {
            if bond.broken {
                continue;
            }
            let dx = self.nodes[bond.j].x - self.nodes[bond.i].x;
            let dy = self.nodes[bond.j].y - self.nodes[bond.i].y;
            let dist = (dx * dx + dy * dy).sqrt();
            if bond.rest_length > 0.0 {
                let strain = (dist - bond.rest_length) / bond.rest_length;
                if strain.abs() > critical_strain {
                    bond.broken = true;
                    continue;
                }
                let f = bond.stiffness * (dist - bond.rest_length);
                if dist > 1e-14 {
                    let nx = dx / dist;
                    let ny = dy / dist;
                    fx[bond.i] += f * nx;
                    fy[bond.i] += f * ny;
                    fx[bond.j] -= f * nx;
                    fy[bond.j] -= f * ny;
                }
            }
        }
        let d = self.damping;
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.pinned || node.mass <= 0.0 {
                continue;
            }
            node.vx += (fx[i] / node.mass - d * node.vx) * dt;
            node.vy += (fy[i] / node.mass - d * node.vy) * dt;
            node.x += node.vx * dt;
            node.y += node.vy * dt;
        }
    }

    /// Number of active bonds.
    pub fn active_bond_count(&self) -> usize {
        self.bonds.iter().filter(|b| !b.broken).count()
    }

    /// Total strain energy.
    pub fn total_strain_energy(&self) -> f64 {
        self.bonds
            .iter()
            .filter(|b| !b.broken)
            .map(|b| {
                let dx = self.nodes[b.j].x - self.nodes[b.i].x;
                let dy = self.nodes[b.j].y - self.nodes[b.i].y;
                let dist = (dx * dx + dy * dy).sqrt();
                let delta = dist - b.rest_length;
                0.5 * b.stiffness * delta * delta
            })
            .sum()
    }
}

/// Create a new lattice spring model.
pub fn new_lattice_spring_model(damping: f64) -> LatticeSpringModel {
    LatticeSpringModel::new(damping)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_node_model() -> LatticeSpringModel {
        let mut m = LatticeSpringModel::new(0.0);
        m.add_node(0.0, 0.0, 1.0);
        m.add_node(1.0, 0.0, 1.0);
        m.add_bond(0, 1, 100.0);
        m
    }

    #[test]
    fn test_add_node() {
        let mut m = LatticeSpringModel::new(0.1);
        let idx = m.add_node(0.0, 0.0, 1.0);
        assert_eq!(idx, 0); /* first node index */
    }

    #[test]
    fn test_rest_length_correct() {
        let m = two_node_model();
        assert!((m.bonds[0].rest_length - 1.0).abs() < 1e-10); /* rest length = 1 */
    }

    #[test]
    fn test_no_force_at_rest() {
        let mut m = two_node_model();
        let x0 = m.nodes[1].x;
        m.step(0.01, 10.0);
        /* at rest length no force applied */
        assert!((m.nodes[1].x - x0).abs() < 1e-10); /* no movement */
    }

    #[test]
    fn test_compressed_spring_repels() {
        let mut m = LatticeSpringModel::new(0.0);
        m.add_node(0.0, 0.0, 1.0);
        m.add_node(1.0, 0.0, 1.0); /* rest_length = 1.0 */
        m.add_bond(0, 1, 100.0);
        /* compress: move node 1 to 0.5 (closer than rest) */
        m.nodes[1].x = 0.5;
        m.nodes[0].pinned = true; /* pin node 0 */
        m.step(0.001, 10.0);
        /* node 1 should move right (repulsion) */
        assert!(m.nodes[1].x > 0.5); /* pushed right */
    }

    #[test]
    fn test_active_bond_count() {
        let m = two_node_model();
        assert_eq!(m.active_bond_count(), 1); /* one active bond */
    }

    #[test]
    fn test_bond_breaking() {
        let mut m = LatticeSpringModel::new(0.0);
        m.add_node(0.0, 0.0, 1.0);
        m.add_node(1.0, 0.0, 1.0); /* rest length = 1.0 */
        m.add_bond(0, 1, 100.0);
        /* stretch node 1 far away so strain >> critical */
        m.nodes[1].x = 5.0;
        m.step(0.001, 0.5); /* critical strain 50% — bond at 400% strain breaks */
        assert_eq!(m.active_bond_count(), 0); /* bond broken */
    }

    #[test]
    fn test_strain_energy_initially_zero_for_rest() {
        let m = two_node_model();
        assert!(m.total_strain_energy().abs() < 1e-10); /* at rest, zero energy */
    }

    #[test]
    fn test_pinned_node_does_not_move() {
        let mut m = two_node_model();
        m.nodes[0].pinned = true;
        /* stretch: move node 1 away */
        m.nodes[1].x = 2.0;
        let x0 = m.nodes[0].x;
        m.step(0.01, 10.0);
        assert!((m.nodes[0].x - x0).abs() < 1e-10); /* pinned node stays */
    }

    #[test]
    fn test_new_helper() {
        let m = new_lattice_spring_model(0.05);
        assert_eq!(m.nodes.len(), 0); /* empty model */
    }

    #[test]
    fn test_strain_energy_stretched() {
        let mut m = LatticeSpringModel::new(0.0);
        m.add_node(0.0, 0.0, 1.0);
        m.add_node(2.0, 0.0, 1.0); /* rest=2, stretched to 2 => no strain initially */
        m.add_bond(0, 1, 100.0);
        /* manually stretch */
        m.nodes[1].x = 3.0;
        let energy = m.total_strain_energy();
        assert!(energy > 0.0); /* energy when stretched */
    }
}
