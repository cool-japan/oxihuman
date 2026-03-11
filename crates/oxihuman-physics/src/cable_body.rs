// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cable (inextensible constraint) body: a chain of particles connected by distance constraints.

/// A single cable particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CableParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub inv_mass: f32,
}

#[allow(dead_code)]
impl CableParticle {
    pub fn new(pos: [f32; 3], inv_mass: f32) -> Self {
        Self {
            pos,
            prev_pos: pos,
            inv_mass,
        }
    }

    pub fn is_fixed(&self) -> bool {
        self.inv_mass == 0.0
    }
}

/// A cable body: a sequence of particles with rest-length constraints between neighbours.
#[allow(dead_code)]
pub struct CableBody {
    pub particles: Vec<CableParticle>,
    pub rest_lengths: Vec<f32>,
    pub total_length: f32,
    pub solve_iterations: u32,
}

#[allow(dead_code)]
impl CableBody {
    pub fn new(n: usize, segment_length: f32) -> Self {
        let particles: Vec<CableParticle> = (0..n)
            .map(|i| {
                let y = -(i as f32) * segment_length;
                let inv_mass = if i == 0 { 0.0 } else { 1.0 };
                CableParticle::new([0.0, y, 0.0], inv_mass)
            })
            .collect();
        let rest_lengths = vec![segment_length; n.saturating_sub(1)];
        let total_length = segment_length * n.saturating_sub(1) as f32;
        Self {
            particles,
            rest_lengths,
            total_length,
            solve_iterations: 4,
        }
    }

    /// Verlet integrate all free particles under gravity.
    pub fn integrate(&mut self, dt: f32, gravity: f32) {
        for p in &mut self.particles {
            if p.is_fixed() {
                continue;
            }
            let vx = p.pos[0] - p.prev_pos[0];
            let vy = p.pos[1] - p.prev_pos[1];
            let vz = p.pos[2] - p.prev_pos[2];
            p.prev_pos = p.pos;
            p.pos[0] += vx;
            p.pos[1] += vy - gravity * dt * dt;
            p.pos[2] += vz;
        }
    }

    /// Apply distance constraints (PBD style).
    pub fn solve_constraints(&mut self) {
        for _ in 0..self.solve_iterations {
            for i in 0..self.rest_lengths.len() {
                let rest = self.rest_lengths[i];
                let pa = self.particles[i].pos;
                let pb = self.particles[i + 1].pos;
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                let dz = pb[2] - pa[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist < 1e-9 {
                    continue;
                }
                let correction = (dist - rest) / dist;
                let ia = self.particles[i].inv_mass;
                let ib = self.particles[i + 1].inv_mass;
                let total_inv = ia + ib;
                if total_inv < 1e-9 {
                    continue;
                }
                let w_a = ia / total_inv;
                let w_b = ib / total_inv;
                self.particles[i].pos[0] += w_a * correction * dx;
                self.particles[i].pos[1] += w_a * correction * dy;
                self.particles[i].pos[2] += w_a * correction * dz;
                self.particles[i + 1].pos[0] -= w_b * correction * dx;
                self.particles[i + 1].pos[1] -= w_b * correction * dy;
                self.particles[i + 1].pos[2] -= w_b * correction * dz;
            }
        }
    }

    pub fn step(&mut self, dt: f32, gravity: f32) {
        self.integrate(dt, gravity);
        self.solve_constraints();
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn segment_length(&self, i: usize) -> f32 {
        if i + 1 >= self.particles.len() {
            return 0.0;
        }
        let pa = self.particles[i].pos;
        let pb = self.particles[i + 1].pos;
        let dx = pb[0] - pa[0];
        let dy = pb[1] - pa[1];
        let dz = pb[2] - pa[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn current_length(&self) -> f32 {
        (0..self.rest_lengths.len())
            .map(|i| self.segment_length(i))
            .sum()
    }
}

pub fn new_cable_body(n: usize, segment_length: f32) -> CableBody {
    CableBody::new(n, segment_length)
}

pub fn cb_step(body: &mut CableBody, dt: f32, gravity: f32) {
    body.step(dt, gravity);
}

pub fn cb_particle_count(body: &CableBody) -> usize {
    body.particle_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cable_has_correct_particle_count() {
        let c = new_cable_body(5, 1.0);
        assert_eq!(cb_particle_count(&c), 5);
    }

    #[test]
    fn first_particle_is_fixed() {
        let c = new_cable_body(3, 1.0);
        assert!(c.particles[0].is_fixed());
    }

    #[test]
    fn non_first_particles_are_free() {
        let c = new_cable_body(3, 1.0);
        assert!(!c.particles[1].is_fixed());
    }

    #[test]
    fn initial_total_length() {
        let c = new_cable_body(4, 2.0);
        assert!((c.total_length - 6.0).abs() < 1e-5);
    }

    #[test]
    fn step_moves_free_particles() {
        let mut c = new_cable_body(3, 1.0);
        let old_y = c.particles[2].pos[1];
        cb_step(&mut c, 0.016, 9.81);
        // free particle should have moved
        assert!(c.particles[2].pos[1] < old_y + 1e-6);
    }

    #[test]
    fn fixed_particle_does_not_move() {
        let mut c = new_cable_body(3, 1.0);
        let old_pos = c.particles[0].pos;
        cb_step(&mut c, 0.016, 9.81);
        assert_eq!(c.particles[0].pos, old_pos);
    }

    #[test]
    fn constraints_preserve_approx_length() {
        let mut c = new_cable_body(4, 1.0);
        for _ in 0..50 {
            cb_step(&mut c, 0.01, 9.81);
        }
        let len = c.current_length();
        // constraints should keep length close to rest length (within 20%)
        assert!((len - c.total_length).abs() / c.total_length < 0.2);
    }

    #[test]
    fn segment_length_computation() {
        let c = new_cable_body(2, 1.5);
        let l = c.segment_length(0);
        assert!((l - 1.5).abs() < 1e-5);
    }

    #[test]
    fn single_particle_cable() {
        let c = new_cable_body(1, 1.0);
        assert_eq!(c.particle_count(), 1);
        assert_eq!(c.rest_lengths.len(), 0);
    }

    #[test]
    fn current_length_matches_sum_of_segments() {
        let c = new_cable_body(3, 2.0);
        let manual: f32 = (0..2).map(|i| c.segment_length(i)).sum();
        assert!((c.current_length() - manual).abs() < 1e-6);
    }
}
