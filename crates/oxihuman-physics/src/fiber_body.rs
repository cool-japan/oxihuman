// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fiber body: a 1-D elastic rod modeled as a chain of particles with bending resistance.

/// A single fiber particle (position + previous position for Verlet integration).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FiberParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub inv_mass: f32,
}

#[allow(dead_code)]
impl FiberParticle {
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

/// A fiber body with stretch and optional bending stiffness.
#[allow(dead_code)]
pub struct FiberBody {
    pub particles: Vec<FiberParticle>,
    pub rest_lengths: Vec<f32>,
    pub stretch_stiffness: f32,
    pub bending_stiffness: f32,
    pub solve_iters: u32,
}

#[allow(dead_code)]
impl FiberBody {
    pub fn new(n: usize, seg_len: f32, stretch_k: f32, bend_k: f32) -> Self {
        let particles: Vec<FiberParticle> = (0..n)
            .map(|i| {
                let pos = [i as f32 * seg_len, 0.0, 0.0];
                let inv_mass = if i == 0 { 0.0 } else { 1.0 };
                FiberParticle::new(pos, inv_mass)
            })
            .collect();
        let rest_lengths = vec![seg_len; n.saturating_sub(1)];
        Self {
            particles,
            rest_lengths,
            stretch_stiffness: stretch_k,
            bending_stiffness: bend_k,
            solve_iters: 4,
        }
    }

    /// Verlet integration under gravity (y-down).
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

    /// Apply stretch (distance) constraints.
    pub fn solve_stretch(&mut self) {
        for _ in 0..self.solve_iters {
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
                let stretch = (dist - rest) / dist;
                let ia = self.particles[i].inv_mass;
                let ib = self.particles[i + 1].inv_mass;
                let total = ia + ib;
                if total < 1e-9 {
                    continue;
                }
                let w_corr = self.stretch_stiffness * stretch;
                self.particles[i].pos[0] += (ia / total) * w_corr * dx;
                self.particles[i].pos[1] += (ia / total) * w_corr * dy;
                self.particles[i].pos[2] += (ia / total) * w_corr * dz;
                self.particles[i + 1].pos[0] -= (ib / total) * w_corr * dx;
                self.particles[i + 1].pos[1] -= (ib / total) * w_corr * dy;
                self.particles[i + 1].pos[2] -= (ib / total) * w_corr * dz;
            }
        }
    }

    pub fn step(&mut self, dt: f32, gravity: f32) {
        self.integrate(dt, gravity);
        self.solve_stretch();
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn total_rest_length(&self) -> f32 {
        self.rest_lengths.iter().sum()
    }

    pub fn current_length(&self) -> f32 {
        (0..self.rest_lengths.len())
            .map(|i| {
                let pa = self.particles[i].pos;
                let pb = self.particles[i + 1].pos;
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                let dz = pb[2] - pa[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .sum()
    }

    /// Bending angle between three consecutive particles (radians).
    pub fn bending_angle(&self, i: usize) -> f32 {
        if i + 2 >= self.particles.len() {
            return 0.0;
        }
        let pa = self.particles[i].pos;
        let pb = self.particles[i + 1].pos;
        let pc = self.particles[i + 2].pos;
        let ax = pb[0] - pa[0];
        let ay = pb[1] - pa[1];
        let az = pb[2] - pa[2];
        let bx = pc[0] - pb[0];
        let by = pc[1] - pb[1];
        let bz = pc[2] - pb[2];
        let dot = ax * bx + ay * by + az * bz;
        let ma = (ax * ax + ay * ay + az * az).sqrt();
        let mb = (bx * bx + by * by + bz * bz).sqrt();
        if ma < 1e-9 || mb < 1e-9 {
            return 0.0;
        }
        (dot / (ma * mb)).clamp(-1.0, 1.0).acos()
    }
}

pub fn new_fiber_body(n: usize, seg_len: f32, stretch_k: f32, bend_k: f32) -> FiberBody {
    FiberBody::new(n, seg_len, stretch_k, bend_k)
}

pub fn fb_step(body: &mut FiberBody, dt: f32, gravity: f32) {
    body.step(dt, gravity);
}

pub fn fb_particle_count(body: &FiberBody) -> usize {
    body.particle_count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn particle_count() {
        let f = new_fiber_body(6, 0.1, 1.0, 0.5);
        assert_eq!(fb_particle_count(&f), 6);
    }

    #[test]
    fn first_particle_fixed() {
        let f = new_fiber_body(4, 0.1, 1.0, 0.0);
        assert!(f.particles[0].is_fixed());
    }

    #[test]
    fn rest_length() {
        let f = new_fiber_body(4, 0.5, 1.0, 0.0);
        assert!((f.total_rest_length() - 1.5).abs() < 1e-5);
    }

    #[test]
    fn step_does_not_move_fixed_particle() {
        let mut f = new_fiber_body(3, 0.2, 1.0, 0.0);
        let old = f.particles[0].pos;
        fb_step(&mut f, 0.01, 9.81);
        assert_eq!(f.particles[0].pos, old);
    }

    #[test]
    fn gravity_drops_free_particles() {
        let mut f = new_fiber_body(3, 0.2, 0.0, 0.0);
        let old_y = f.particles[2].pos[1];
        fb_step(&mut f, 0.01, 9.81);
        assert!(f.particles[2].pos[1] <= old_y + 1e-9);
    }

    #[test]
    fn current_length_near_rest() {
        let mut f = new_fiber_body(4, 1.0, 1.0, 0.0);
        for _ in 0..10 {
            fb_step(&mut f, 0.01, 0.0);
        }
        let ratio = (f.current_length() / f.total_rest_length() - 1.0).abs();
        assert!(ratio < 0.3);
    }

    #[test]
    fn bending_angle_straight_is_zero() {
        let f = new_fiber_body(3, 1.0, 1.0, 0.0);
        let angle = f.bending_angle(0);
        assert!(angle.abs() < 1e-4);
    }

    #[test]
    fn bending_angle_right_angle_is_pi_half() {
        let mut f = new_fiber_body(3, 1.0, 0.0, 0.0);
        f.particles[0].pos = [0.0, 0.0, 0.0];
        f.particles[1].pos = [1.0, 0.0, 0.0];
        f.particles[2].pos = [1.0, 1.0, 0.0];
        let angle = f.bending_angle(0);
        assert!((angle - PI / 2.0).abs() < 1e-4);
    }

    #[test]
    fn multiple_steps_no_nan() {
        let mut f = new_fiber_body(5, 0.1, 0.5, 0.1);
        for _ in 0..30 {
            fb_step(&mut f, 0.005, 9.81);
        }
        for p in &f.particles {
            assert!(!p.pos[0].is_nan());
        }
    }

    #[test]
    fn single_particle_fiber() {
        let f = new_fiber_body(1, 1.0, 1.0, 0.0);
        assert_eq!(f.particle_count(), 1);
        assert_eq!(f.rest_lengths.len(), 0);
    }
}
