// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2-D membrane tension model — simulates a thin elastic membrane under
//! uniform tension using discrete mass-spring nodes.

/// A 2-D membrane node.
#[derive(Debug, Clone)]
pub struct MemNode {
    pub x: f64,
    pub y: f64,
    pub z: f64, /* out-of-plane displacement */
    pub vz: f64,
    pub mass: f64,
    pub pinned: bool,
}

impl MemNode {
    pub fn new(x: f64, y: f64, mass: f64) -> Self {
        Self {
            x,
            y,
            z: 0.0,
            vz: 0.0,
            mass,
            pinned: false,
        }
    }
}

/// A membrane spring between two nodes.
#[derive(Debug, Clone)]
pub struct MemSpring {
    pub i: usize,
    pub j: usize,
    pub rest_length: f64,
    pub tension: f64,
}

/// 2-D membrane tension model.
pub struct MembraneTension {
    pub nodes: Vec<MemNode>,
    pub springs: Vec<MemSpring>,
    pub damping: f64,
}

impl MembraneTension {
    /// Create a membrane.
    pub fn new(damping: f64) -> Self {
        Self {
            nodes: Vec::new(),
            springs: Vec::new(),
            damping,
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, x: f64, y: f64, mass: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(MemNode::new(x, y, mass));
        idx
    }

    /// Add a spring between two nodes.
    pub fn add_spring(&mut self, i: usize, j: usize, tension: f64) {
        let dx = self.nodes[j].x - self.nodes[i].x;
        let dy = self.nodes[j].y - self.nodes[i].y;
        let rest = (dx * dx + dy * dy).sqrt();
        self.springs.push(MemSpring {
            i,
            j,
            rest_length: rest,
            tension,
        });
    }

    /// Build a regular grid membrane of `nx` x `ny` nodes.
    pub fn build_grid(&mut self, nx: usize, ny: usize, spacing: f64, tension: f64, mass: f64) {
        let offset = self.nodes.len();
        for j in 0..ny {
            for i in 0..nx {
                self.add_node(i as f64 * spacing, j as f64 * spacing, mass);
            }
        }
        for j in 0..ny {
            for i in 0..nx {
                let idx = offset + j * nx + i;
                if i + 1 < nx {
                    self.add_spring(idx, idx + 1, tension);
                }
                if j + 1 < ny {
                    self.add_spring(idx, idx + nx, tension);
                }
            }
        }
    }

    /// Apply out-of-plane restoring forces and integrate.
    pub fn step(&mut self, dt: f64) {
        let mut fz = vec![0.0f64; self.nodes.len()];
        for s in &self.springs {
            let dz = self.nodes[s.j].z - self.nodes[s.i].z;
            let dx = self.nodes[s.j].x - self.nodes[s.i].x;
            let dy = self.nodes[s.j].y - self.nodes[s.i].y;
            let horiz = (dx * dx + dy * dy).sqrt().max(1e-14);
            /* transverse force due to tension */
            let f = s.tension * dz / horiz;
            fz[s.i] += f;
            fz[s.j] -= f;
        }
        let d = self.damping;
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.pinned || node.mass <= 0.0 {
                continue;
            }
            node.vz += (fz[i] / node.mass - d * node.vz) * dt;
            node.z += node.vz * dt;
        }
    }

    /// Maximum out-of-plane displacement.
    pub fn max_displacement(&self) -> f64 {
        self.nodes.iter().map(|n| n.z.abs()).fold(0.0f64, f64::max)
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of springs.
    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }
}

/// Create a new membrane tension model.
pub fn new_membrane_tension(damping: f64) -> MembraneTension {
    MembraneTension::new(damping)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut m = MembraneTension::new(0.1);
        let idx = m.add_node(0.0, 0.0, 1.0);
        assert_eq!(idx, 0); /* first node index */
    }

    #[test]
    fn test_build_grid_node_count() {
        let mut m = MembraneTension::new(0.1);
        m.build_grid(3, 3, 1.0, 1.0, 1.0);
        assert_eq!(m.node_count(), 9); /* 3x3 grid */
    }

    #[test]
    fn test_build_grid_spring_count() {
        let mut m = MembraneTension::new(0.1);
        m.build_grid(3, 3, 1.0, 1.0, 1.0);
        /* 3*2 horizontal + 2*3 vertical = 12 springs */
        assert_eq!(m.spring_count(), 12); /* correct spring count */
    }

    #[test]
    fn test_no_displacement_at_rest() {
        let mut m = MembraneTension::new(0.1);
        m.build_grid(3, 3, 1.0, 1.0, 1.0);
        m.step(0.01);
        assert_eq!(m.max_displacement(), 0.0); /* flat membrane stays flat */
    }

    #[test]
    fn test_pinned_node_no_movement() {
        let mut m = MembraneTension::new(0.0);
        m.add_node(0.0, 0.0, 1.0);
        m.add_node(1.0, 0.0, 1.0);
        m.add_spring(0, 1, 10.0);
        m.nodes[0].pinned = true;
        m.nodes[1].z = 0.5; /* perturb */
        let z0 = m.nodes[0].z;
        m.step(0.01);
        assert_eq!(m.nodes[0].z, z0); /* pinned node unchanged */
    }

    #[test]
    fn test_displaced_node_moves() {
        let mut m = MembraneTension::new(0.0);
        m.add_node(0.0, 0.0, 1.0);
        m.add_node(1.0, 0.0, 1.0);
        m.add_spring(0, 1, 10.0);
        m.nodes[0].pinned = true;
        m.nodes[1].z = 0.1; /* displaced */
        m.step(0.01);
        /* node 1 should move toward flat */
        assert!(m.nodes[1].z < 0.1); /* restoring force applied */
    }

    #[test]
    fn test_max_displacement() {
        let mut m = MembraneTension::new(0.0);
        m.add_node(0.0, 0.0, 1.0);
        m.nodes[0].z = 0.7;
        assert!((m.max_displacement() - 0.7).abs() < 1e-10); /* correct max */
    }

    #[test]
    fn test_spring_count() {
        let mut m = MembraneTension::new(0.0);
        m.add_node(0.0, 0.0, 1.0);
        m.add_node(1.0, 0.0, 1.0);
        m.add_spring(0, 1, 1.0);
        assert_eq!(m.spring_count(), 1); /* one spring added */
    }

    #[test]
    fn test_new_helper() {
        let m = new_membrane_tension(0.05);
        assert_eq!(m.node_count(), 0); /* empty model */
    }

    #[test]
    fn test_damping_reduces_velocity() {
        /* test that a node with an initial velocity and no spring force decays */
        let mut m = MembraneTension::new(5.0); /* damping coefficient */
        m.add_node(0.0, 0.0, 1.0);
        /* give node 0 an initial out-of-plane velocity, no springs */
        m.nodes[0].vz = 1.0;
        let v0 = m.nodes[0].vz.abs();
        m.step(0.1);
        let v1 = m.nodes[0].vz.abs();
        assert!(v1 < v0); /* damping reduces velocity */
    }
}
