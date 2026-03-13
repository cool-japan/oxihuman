// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Material Point Method (MPM) simulation stub — particles carry state and
//! transfer to/from a background grid for force evaluation.

/// A material point.
#[derive(Debug, Clone)]
pub struct MaterialPoint {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub mass: f64,
    pub volume: f64,
}

impl MaterialPoint {
    pub fn new(x: f64, y: f64, mass: f64, volume: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            mass,
            volume,
        }
    }
}

/// A 2-D background grid node.
#[derive(Debug, Default, Clone)]
pub struct GridNode {
    pub mass: f64,
    pub momentum_x: f64,
    pub momentum_y: f64,
    pub force_x: f64,
    pub force_y: f64,
}

/// MPM simulation on a 2-D grid.
pub struct MaterialPointMethod {
    pub particles: Vec<MaterialPoint>,
    pub grid: Vec<GridNode>,
    pub grid_w: usize,
    pub grid_h: usize,
    pub cell_size: f64,
    pub gravity: f64,
}

impl MaterialPointMethod {
    /// Create a new MPM simulation.
    pub fn new(grid_w: usize, grid_h: usize, cell_size: f64, gravity: f64) -> Self {
        let grid = vec![GridNode::default(); grid_w * grid_h];
        Self {
            particles: Vec::new(),
            grid,
            grid_w,
            grid_h,
            cell_size,
            gravity,
        }
    }

    /// Add a material point.
    pub fn add_point(&mut self, x: f64, y: f64, mass: f64, volume: f64) {
        self.particles.push(MaterialPoint::new(x, y, mass, volume));
    }

    fn node_idx(&self, gx: usize, gy: usize) -> usize {
        gy * self.grid_w + gx
    }

    /// Reset grid nodes.
    pub fn clear_grid(&mut self) {
        for node in &mut self.grid {
            *node = GridNode::default();
        }
    }

    /// Transfer particle mass and momentum to the grid (P2G).
    pub fn p2g(&mut self) {
        for p in &self.particles {
            let gx = (p.x / self.cell_size) as usize;
            let gy = (p.y / self.cell_size) as usize;
            if gx < self.grid_w && gy < self.grid_h {
                let idx = self.node_idx(gx, gy);
                self.grid[idx].mass += p.mass;
                self.grid[idx].momentum_x += p.mass * p.vx;
                self.grid[idx].momentum_y += p.mass * p.vy;
            }
        }
    }

    /// Apply gravity forces on grid nodes.
    pub fn apply_gravity(&mut self) {
        for node in &mut self.grid {
            node.force_y -= node.mass * self.gravity;
        }
    }

    /// Update grid velocities from forces (dt step).
    pub fn grid_update(&mut self, dt: f64) {
        for node in &mut self.grid {
            if node.mass > 0.0 {
                node.momentum_x += node.force_x * dt;
                node.momentum_y += node.force_y * dt;
            }
        }
    }

    /// Transfer grid velocities back to particles (G2P).
    pub fn g2p(&mut self, dt: f64) {
        for p in &mut self.particles {
            let gx = (p.x / self.cell_size) as usize;
            let gy = (p.y / self.cell_size) as usize;
            if gx < self.grid_w && gy < self.grid_h {
                let idx = gy * self.grid_w + gx;
                let node = &self.grid[idx];
                if node.mass > 0.0 {
                    p.vx = node.momentum_x / node.mass;
                    p.vy = node.momentum_y / node.mass;
                }
            }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
    }

    /// Number of particles.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Total grid node count.
    pub fn node_count(&self) -> usize {
        self.grid.len()
    }
}

/// Create a new MPM simulation.
pub fn new_mpm(grid_w: usize, grid_h: usize, cell_size: f64, gravity: f64) -> MaterialPointMethod {
    MaterialPointMethod::new(grid_w, grid_h, cell_size, gravity)
}

pub struct MpmParticle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub mass: f32,
    pub volume: f32,
    pub stress: [f32; 3],
}

pub fn new_mpm_particle(pos: [f32; 2], mass: f32) -> MpmParticle {
    MpmParticle {
        pos,
        vel: [0.0, 0.0],
        mass,
        volume: 1.0,
        stress: [0.0, 0.0, 0.0],
    }
}

pub fn mpm_particle_momentum(p: &MpmParticle) -> [f32; 2] {
    [p.mass * p.vel[0], p.mass * p.vel[1]]
}

pub fn mpm_particle_kinetic_energy(p: &MpmParticle) -> f32 {
    0.5 * p.mass * (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1])
}

pub fn mpm_particle_step(p: &mut MpmParticle, force: [f32; 2], dt: f32) {
    let inv_mass = if p.mass > 0.0 { 1.0 / p.mass } else { 0.0 };
    p.vel[0] += force[0] * inv_mass * dt;
    p.vel[1] += force[1] * inv_mass * dt;
    p.pos[0] += p.vel[0] * dt;
    p.pos[1] += p.vel[1] * dt;
}

pub fn mpm_von_mises_stress(p: &MpmParticle) -> f32 {
    let s11 = p.stress[0];
    let s22 = p.stress[1];
    let s12 = p.stress[2];
    let ds = s11 - s22;
    (0.5 * (ds * ds + (s11 * s11) + (s22 * s22)) + 3.0 * s12 * s12).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_point() {
        let mut mpm = MaterialPointMethod::new(10, 10, 1.0, 9.81);
        mpm.add_point(2.0, 2.0, 1.0, 1.0);
        assert_eq!(mpm.particle_count(), 1); /* one point added */
    }

    #[test]
    fn test_node_count() {
        let mpm = MaterialPointMethod::new(4, 4, 1.0, 9.81);
        assert_eq!(mpm.node_count(), 16); /* 4x4 grid */
    }

    #[test]
    fn test_p2g_mass_transfer() {
        let mut mpm = MaterialPointMethod::new(10, 10, 1.0, 9.81);
        mpm.add_point(1.0, 1.0, 2.0, 1.0);
        mpm.p2g();
        /* particle at (1,1) maps to node (1,1) = idx 1*10+1 = 11 */
        assert!((mpm.grid[11].mass - 2.0).abs() < 1e-10); /* mass transferred */
    }

    #[test]
    fn test_clear_grid() {
        let mut mpm = MaterialPointMethod::new(4, 4, 1.0, 9.81);
        mpm.add_point(1.0, 1.0, 1.0, 1.0);
        mpm.p2g();
        mpm.clear_grid();
        let total_mass: f64 = mpm.grid.iter().map(|n| n.mass).sum();
        assert_eq!(total_mass, 0.0); /* grid cleared */
    }

    #[test]
    fn test_gravity_reduces_momentum_y() {
        let mut mpm = MaterialPointMethod::new(10, 10, 1.0, 9.81);
        mpm.add_point(1.0, 1.0, 1.0, 1.0);
        mpm.p2g();
        mpm.apply_gravity();
        let node = &mpm.grid[11];
        assert!(node.force_y < 0.0); /* gravity acts downward */
    }

    #[test]
    fn test_grid_update_changes_momentum() {
        let mut mpm = MaterialPointMethod::new(10, 10, 1.0, 9.81);
        mpm.add_point(1.0, 1.0, 1.0, 1.0);
        mpm.p2g();
        mpm.apply_gravity();
        let mom_before = mpm.grid[11].momentum_y;
        mpm.grid_update(0.01);
        let mom_after = mpm.grid[11].momentum_y;
        assert!(mom_after < mom_before); /* gravity reduces y momentum */
    }

    #[test]
    fn test_g2p_moves_particle() {
        let mut mpm = MaterialPointMethod::new(10, 10, 1.0, 9.81);
        mpm.add_point(1.5, 1.5, 1.0, 1.0);
        let y0 = mpm.particles[0].y;
        mpm.p2g();
        mpm.apply_gravity();
        mpm.grid_update(0.01);
        mpm.g2p(0.01);
        /* with gravity the particle should move (direction depends on sign) */
        assert_ne!(mpm.particles[0].y, y0); /* particle moved */
    }

    #[test]
    fn test_new_helper() {
        let mpm = new_mpm(8, 8, 0.5, 9.81);
        assert_eq!(mpm.particle_count(), 0); /* empty on creation */
    }

    #[test]
    fn test_two_particles_total_mass() {
        let mut mpm = MaterialPointMethod::new(10, 10, 1.0, 9.81);
        mpm.add_point(1.0, 1.0, 2.0, 1.0);
        mpm.add_point(3.0, 3.0, 3.0, 1.0);
        mpm.p2g();
        let total_grid_mass: f64 = mpm.grid.iter().map(|n| n.mass).sum();
        assert!((total_grid_mass - 5.0).abs() < 1e-10); /* mass conserved */
    }
}
