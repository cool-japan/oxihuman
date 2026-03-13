// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XPBD cloth simulation — distance constraints on a particle grid.

/// A cloth particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
}

#[allow(dead_code)]
impl ClothParticle {
    pub fn new(pos: [f32; 3], mass: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        Self {
            pos,
            prev_pos: pos,
            vel: [0.0; 3],
            inv_mass,
        }
    }

    pub fn is_static(&self) -> bool {
        self.inv_mass < 1e-9
    }
}

/// A distance constraint between two particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothConstraint {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub compliance: f32,
}

/// XPBD cloth simulator.
#[allow(dead_code)]
pub struct XpbdCloth {
    pub particles: Vec<ClothParticle>,
    pub constraints: Vec<ClothConstraint>,
    pub gravity: [f32; 3],
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl XpbdCloth {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            constraints: Vec::new(),
            gravity: [0.0, -9.81, 0.0],
            time: 0.0,
            steps: 0,
        }
    }

    pub fn add_particle(&mut self, p: ClothParticle) -> usize {
        let id = self.particles.len();
        self.particles.push(p);
        id
    }

    pub fn add_constraint(&mut self, a: usize, b: usize, compliance: f32) {
        let rest_len = {
            let pa = &self.particles[a];
            let pb = &self.particles[b];
            let dx = pb.pos[0] - pa.pos[0];
            let dy = pb.pos[1] - pa.pos[1];
            let dz = pb.pos[2] - pa.pos[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        };
        self.constraints.push(ClothConstraint {
            a,
            b,
            rest_len,
            compliance,
        });
    }

    pub fn step(&mut self, dt: f32, sub_steps: u32) {
        let sub_dt = dt / sub_steps as f32;
        for _ in 0..sub_steps {
            self.substep(sub_dt);
        }
        self.time += dt;
        self.steps += 1;
    }

    fn substep(&mut self, dt: f32) {
        // Predict positions.
        for p in &mut self.particles {
            if p.is_static() {
                continue;
            }
            p.prev_pos = p.pos;
            p.vel[0] += self.gravity[0] * dt;
            p.vel[1] += self.gravity[1] * dt;
            p.vel[2] += self.gravity[2] * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
            p.pos[2] += p.vel[2] * dt;
        }
        // Solve distance constraints.
        let alpha_dt2 = 0.0f32; // compliance / dt^2 (stiff for now)
        let _ = alpha_dt2;
        let constraints = self.constraints.clone();
        for c in &constraints {
            let pa = self.particles[c.a].pos;
            let pb = self.particles[c.b].pos;
            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let dz = pb[2] - pa[2];
            let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
            let err = len - c.rest_len;
            let w_a = self.particles[c.a].inv_mass;
            let w_b = self.particles[c.b].inv_mass;
            let w_sum = w_a + w_b;
            if w_sum < 1e-9 {
                continue;
            }
            let alpha = c.compliance / (dt * dt);
            let lagrange = -err / (w_sum + alpha);
            let corr = [
                dx / len * lagrange,
                dy / len * lagrange,
                dz / len * lagrange,
            ];
            if !self.particles[c.a].is_static() {
                self.particles[c.a].pos[0] -= w_a * corr[0];
                self.particles[c.a].pos[1] -= w_a * corr[1];
                self.particles[c.a].pos[2] -= w_a * corr[2];
            }
            if !self.particles[c.b].is_static() {
                self.particles[c.b].pos[0] += w_b * corr[0];
                self.particles[c.b].pos[1] += w_b * corr[1];
                self.particles[c.b].pos[2] += w_b * corr[2];
            }
        }
        // Update velocities.
        let inv_dt = 1.0 / dt.max(1e-9);
        for p in &mut self.particles {
            if p.is_static() {
                continue;
            }
            p.vel[0] = (p.pos[0] - p.prev_pos[0]) * inv_dt;
            p.vel[1] = (p.pos[1] - p.prev_pos[1]) * inv_dt;
            p.vel[2] = (p.pos[2] - p.prev_pos[2]) * inv_dt;
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    pub fn total_kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .map(|p| {
                let v2 = p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1] + p.vel[2] * p.vel[2];
                0.5 * v2 / p.inv_mass.max(1e-9)
            })
            .sum()
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.constraints.clear();
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for XpbdCloth {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_xpbd_cloth() -> XpbdCloth {
    XpbdCloth::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_falls_under_gravity() {
        let mut c = new_xpbd_cloth();
        c.add_particle(ClothParticle::new([0.0, 10.0, 0.0], 1.0));
        c.step(0.5, 5);
        assert!(c.particles[0].pos[1] < 10.0);
    }

    #[test]
    fn static_particle_does_not_move() {
        let mut c = new_xpbd_cloth();
        c.add_particle(ClothParticle::new([0.0, 5.0, 0.0], 0.0));
        c.step(1.0, 5);
        assert!((c.particles[0].pos[1] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn constraint_limits_stretch() {
        let mut c = new_xpbd_cloth();
        let a = c.add_particle(ClothParticle::new([0.0, 5.0, 0.0], 0.0));
        let b = c.add_particle(ClothParticle::new([0.0, 4.0, 0.0], 1.0));
        c.add_constraint(a, b, 0.0);
        let rest = c.constraints[0].rest_len;
        c.step(0.5, 10);
        let dx = c.particles[b].pos[0] - c.particles[a].pos[0];
        let dy = c.particles[b].pos[1] - c.particles[a].pos[1];
        let dz = c.particles[b].pos[2] - c.particles[a].pos[2];
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        assert!((len - rest).abs() < 0.2);
    }

    #[test]
    fn particle_count() {
        let mut c = new_xpbd_cloth();
        c.add_particle(ClothParticle::new([0.0; 3], 1.0));
        c.add_particle(ClothParticle::new([1.0, 0.0, 0.0], 1.0));
        assert_eq!(c.particle_count(), 2);
    }

    #[test]
    fn constraint_count() {
        let mut c = new_xpbd_cloth();
        c.add_particle(ClothParticle::new([0.0; 3], 1.0));
        c.add_particle(ClothParticle::new([1.0, 0.0, 0.0], 1.0));
        c.add_constraint(0, 1, 0.001);
        assert_eq!(c.constraint_count(), 1);
    }

    #[test]
    fn step_count_increments() {
        let mut c = new_xpbd_cloth();
        c.add_particle(ClothParticle::new([0.0; 3], 1.0));
        c.step(0.016, 2);
        c.step(0.016, 2);
        assert_eq!(c.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut c = new_xpbd_cloth();
        c.step(0.1, 1);
        assert!((c.time - 0.1).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut c = new_xpbd_cloth();
        c.add_particle(ClothParticle::new([0.0, 5.0, 0.0], 1.0));
        c.step(0.5, 5);
        assert!(c.total_kinetic_energy() >= 0.0);
    }

    #[test]
    fn clear_resets() {
        let mut c = new_xpbd_cloth();
        c.add_particle(ClothParticle::new([0.0; 3], 1.0));
        c.step(0.1, 1);
        c.clear();
        assert_eq!(c.particle_count(), 0);
    }

    #[test]
    fn static_particle_detection() {
        let p = ClothParticle::new([0.0; 3], 0.0);
        assert!(p.is_static());
        let p2 = ClothParticle::new([0.0; 3], 1.0);
        assert!(!p2.is_static());
    }
}
