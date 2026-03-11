// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! An elastic mesh: a 2-D grid of particles connected by springs.

/// A particle in the elastic mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElasticParticle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub inv_mass: f32,
}

#[allow(dead_code)]
impl ElasticParticle {
    pub fn new(pos: [f32; 2], inv_mass: f32) -> Self {
        Self {
            pos,
            vel: [0.0; 2],
            inv_mass,
        }
    }

    pub fn is_fixed(&self) -> bool {
        self.inv_mass == 0.0
    }
}

/// A spring connecting two particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElasticSpring {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// An elastic mesh simulation.
#[allow(dead_code)]
pub struct ElasticMesh {
    pub particles: Vec<ElasticParticle>,
    pub springs: Vec<ElasticSpring>,
    pub rows: usize,
    pub cols: usize,
}

#[allow(dead_code)]
impl ElasticMesh {
    /// Create a regular grid mesh of `rows` x `cols` with given `spacing` and `stiffness`.
    pub fn new_grid(rows: usize, cols: usize, spacing: f32, stiffness: f32, damping: f32) -> Self {
        let mut particles = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            for c in 0..cols {
                let pos = [c as f32 * spacing, -(r as f32) * spacing];
                let inv_mass = if r == 0 { 0.0 } else { 1.0 };
                particles.push(ElasticParticle::new(pos, inv_mass));
            }
        }
        let mut springs = Vec::new();
        let idx = |r: usize, c: usize| r * cols + c;
        for r in 0..rows {
            for c in 0..cols {
                // Horizontal spring
                if c + 1 < cols {
                    let a = idx(r, c);
                    let b = idx(r, c + 1);
                    let rest_len = (particles[b].pos[0] - particles[a].pos[0]).abs();
                    springs.push(ElasticSpring {
                        a,
                        b,
                        rest_len,
                        stiffness,
                        damping,
                    });
                }
                // Vertical spring
                if r + 1 < rows {
                    let a = idx(r, c);
                    let b = idx(r + 1, c);
                    let rest_len = (particles[b].pos[1] - particles[a].pos[1]).abs();
                    springs.push(ElasticSpring {
                        a,
                        b,
                        rest_len,
                        stiffness,
                        damping,
                    });
                }
            }
        }
        Self {
            particles,
            springs,
            rows,
            cols,
        }
    }

    /// Integrate one timestep under gravity.
    pub fn step(&mut self, dt: f32, gravity: f32) {
        // Compute spring forces
        let mut forces = vec![[0.0_f32; 2]; self.particles.len()];
        for s in &self.springs {
            let pa = self.particles[s.a].pos;
            let pb = self.particles[s.b].pos;
            let va = self.particles[s.a].vel;
            let vb = self.particles[s.b].vel;
            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 1e-9 {
                continue;
            }
            let nx = dx / dist;
            let ny = dy / dist;
            let spring_f = s.stiffness * (dist - s.rest_len);
            let rel_vel = (vb[0] - va[0]) * nx + (vb[1] - va[1]) * ny;
            let f = spring_f + s.damping * rel_vel;
            forces[s.a][0] += f * nx;
            forces[s.a][1] += f * ny;
            forces[s.b][0] -= f * nx;
            forces[s.b][1] -= f * ny;
        }
        // Integrate
        for (i, p) in self.particles.iter_mut().enumerate() {
            if p.is_fixed() {
                continue;
            }
            p.vel[0] += forces[i][0] * p.inv_mass * dt;
            p.vel[1] += (forces[i][1] * p.inv_mass - gravity) * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }

    pub fn elastic_energy(&self) -> f32 {
        self.springs
            .iter()
            .map(|s| {
                let pa = self.particles[s.a].pos;
                let pb = self.particles[s.b].pos;
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                let dist = (dx * dx + dy * dy).sqrt();
                let deform = dist - s.rest_len;
                0.5 * s.stiffness * deform * deform
            })
            .sum()
    }

    pub fn center_of_mass(&self) -> [f32; 2] {
        let free: Vec<&ElasticParticle> = self.particles.iter().filter(|p| !p.is_fixed()).collect();
        if free.is_empty() {
            return [0.0; 2];
        }
        let sum_x: f32 = free.iter().map(|p| p.pos[0]).sum();
        let sum_y: f32 = free.iter().map(|p| p.pos[1]).sum();
        let n = free.len() as f32;
        [sum_x / n, sum_y / n]
    }
}

pub fn new_elastic_mesh(rows: usize, cols: usize, spacing: f32, k: f32, c: f32) -> ElasticMesh {
    ElasticMesh::new_grid(rows, cols, spacing, k, c)
}

pub fn em_step(mesh: &mut ElasticMesh, dt: f32, gravity: f32) {
    mesh.step(dt, gravity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_particle_count() {
        let m = new_elastic_mesh(3, 4, 1.0, 100.0, 0.5);
        assert_eq!(m.particle_count(), 12);
    }

    #[test]
    fn top_row_fixed() {
        let m = new_elastic_mesh(3, 4, 1.0, 100.0, 0.5);
        for c in 0..4 {
            assert!(m.particles[c].is_fixed());
        }
    }

    #[test]
    fn spring_count_correct() {
        // 3x4 grid: horizontal = 3*3=9, vertical = 2*4=8 → 17
        let m = new_elastic_mesh(3, 4, 1.0, 100.0, 0.5);
        assert_eq!(m.spring_count(), 17);
    }

    #[test]
    fn step_moves_free_particles() {
        let mut m = new_elastic_mesh(2, 2, 1.0, 0.0, 0.0);
        let old_y = m.particles[2].pos[1];
        em_step(&mut m, 0.01, 9.81);
        assert!(m.particles[2].pos[1] < old_y + 1e-9);
    }

    #[test]
    fn fixed_particles_dont_move() {
        let mut m = new_elastic_mesh(2, 2, 1.0, 100.0, 0.5);
        let old_p = m.particles[0].pos;
        em_step(&mut m, 0.01, 9.81);
        assert_eq!(m.particles[0].pos, old_p);
    }

    #[test]
    fn elastic_energy_zero_at_rest() {
        let m = new_elastic_mesh(2, 2, 1.0, 100.0, 0.0);
        assert!(m.elastic_energy() < 1e-8);
    }

    #[test]
    fn center_of_mass_computed() {
        let m = new_elastic_mesh(2, 2, 1.0, 0.0, 0.0);
        let cm = m.center_of_mass();
        // Free particles are row 1: (0,-1) and (1,-1)
        assert!((cm[0] - 0.5).abs() < 1e-5);
        assert!((cm[1] - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn rest_lengths_equal_spacing() {
        let m = new_elastic_mesh(2, 3, 2.0, 10.0, 0.0);
        for s in &m.springs {
            if (s.a as isize - s.b as isize).unsigned_abs() == 1 {
                // horizontal spring
                assert!((s.rest_len - 2.0).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn rows_and_cols_stored() {
        let m = new_elastic_mesh(4, 5, 1.0, 10.0, 0.0);
        assert_eq!(m.rows, 4);
        assert_eq!(m.cols, 5);
    }

    #[test]
    fn step_multiple_frames() {
        let mut m = new_elastic_mesh(2, 2, 1.0, 50.0, 1.0);
        for _ in 0..20 {
            em_step(&mut m, 0.01, 9.81);
        }
        // Just verify no NaN
        for p in &m.particles {
            assert!(!p.pos[0].is_nan());
            assert!(!p.pos[1].is_nan());
        }
    }
}
