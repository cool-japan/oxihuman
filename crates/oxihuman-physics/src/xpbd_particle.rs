// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XPBD particle system — simple position-based dynamics for particles.

/// An XPBD particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct XpbdParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
    pub radius: f32,
}

#[allow(dead_code)]
impl XpbdParticle {
    pub fn new(pos: [f32; 3], mass: f32, radius: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        Self {
            pos,
            prev_pos: pos,
            vel: [0.0; 3],
            inv_mass,
            radius,
        }
    }

    pub fn is_static(&self) -> bool {
        self.inv_mass < 1e-9
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2]).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        if self.inv_mass < 1e-9 {
            return 0.0;
        }
        let v2 = self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2];
        0.5 * v2 / self.inv_mass
    }
}

/// XPBD particle system.
#[allow(dead_code)]
pub struct XpbdParticleSystem {
    pub particles: Vec<XpbdParticle>,
    pub gravity: [f32; 3],
    pub time: f32,
    pub steps: u64,
    pub restitution: f32,
    /// Ground plane Y.
    pub ground_y: f32,
}

#[allow(dead_code)]
impl XpbdParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            gravity: [0.0, -9.81, 0.0],
            time: 0.0,
            steps: 0,
            restitution: 0.5,
            ground_y: 0.0,
        }
    }

    pub fn add_particle(&mut self, p: XpbdParticle) -> usize {
        let id = self.particles.len();
        self.particles.push(p);
        id
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
        // Predict.
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
        // Ground collision.
        for p in &mut self.particles {
            if p.is_static() {
                continue;
            }
            let min_y = self.ground_y + p.radius;
            if p.pos[1] < min_y {
                p.pos[1] = min_y;
                if p.vel[1] < 0.0 {
                    p.vel[1] = -p.vel[1] * self.restitution;
                }
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

    pub fn total_kinetic_energy(&self) -> f32 {
        self.particles.iter().map(|p| p.kinetic_energy()).sum()
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for XpbdParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_xpbd_particle_system() -> XpbdParticleSystem {
    XpbdParticleSystem::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_falls() {
        let mut s = new_xpbd_particle_system();
        s.add_particle(XpbdParticle::new([0.0, 10.0, 0.0], 1.0, 0.1));
        s.step(0.5, 5);
        assert!(s.particles[0].pos[1] < 10.0);
    }

    #[test]
    fn static_particle_stays() {
        let mut s = new_xpbd_particle_system();
        s.add_particle(XpbdParticle::new([0.0, 5.0, 0.0], 0.0, 0.1));
        s.step(1.0, 5);
        assert!((s.particles[0].pos[1] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn ground_collision_stops_fall() {
        let mut s = new_xpbd_particle_system();
        s.ground_y = 0.0;
        s.add_particle(XpbdParticle::new([0.0, 0.5, 0.0], 1.0, 0.1));
        for _ in 0..100 {
            s.step(0.02, 2);
        }
        assert!(s.particles[0].pos[1] >= 0.0);
    }

    #[test]
    fn step_count() {
        let mut s = new_xpbd_particle_system();
        s.add_particle(XpbdParticle::new([0.0; 3], 1.0, 0.1));
        s.step(0.1, 2);
        assert_eq!(s.steps, 1);
    }

    #[test]
    fn time_advances() {
        let mut s = new_xpbd_particle_system();
        s.step(0.1, 1);
        assert!((s.time - 0.1).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut s = new_xpbd_particle_system();
        s.add_particle(XpbdParticle::new([0.0, 5.0, 0.0], 1.0, 0.1));
        s.step(0.5, 5);
        assert!(s.total_kinetic_energy() >= 0.0);
    }

    #[test]
    fn particle_count() {
        let mut s = new_xpbd_particle_system();
        s.add_particle(XpbdParticle::new([0.0; 3], 1.0, 0.1));
        s.add_particle(XpbdParticle::new([1.0, 0.0, 0.0], 1.0, 0.1));
        assert_eq!(s.particle_count(), 2);
    }

    #[test]
    fn clear_empties() {
        let mut s = new_xpbd_particle_system();
        s.add_particle(XpbdParticle::new([0.0; 3], 1.0, 0.1));
        s.step(0.1, 1);
        s.clear();
        assert_eq!(s.particle_count(), 0);
    }

    #[test]
    fn static_particle_detection() {
        let p = XpbdParticle::new([0.0; 3], 0.0, 0.1);
        assert!(p.is_static());
    }

    #[test]
    fn restitution_bounces() {
        let mut s = new_xpbd_particle_system();
        s.restitution = 0.8;
        s.ground_y = 0.0;
        s.add_particle(XpbdParticle::new([0.0, 2.0, 0.0], 1.0, 0.1));
        for _ in 0..200 {
            s.step(0.01, 2);
        }
        // Particle should still be above ground.
        assert!(s.particles[0].pos[1] >= -0.05);
    }
}
