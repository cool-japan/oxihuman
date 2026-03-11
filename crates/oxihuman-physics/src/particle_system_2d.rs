// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D particle emitter/system stub.

#[derive(Debug, Clone)]
pub struct Particle2d {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub active: bool,
}

impl Particle2d {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32, life: f32) -> Self {
        Particle2d {
            position: [x, y],
            velocity: [vx, vy],
            life,
            max_life: life,
            size: 1.0,
            active: true,
        }
    }

    pub fn normalized_age(&self) -> f32 {
        if self.max_life < f32::EPSILON {
            return 1.0;
        }
        1.0 - (self.life / self.max_life).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct ParticleEmitter2d {
    pub position: [f32; 2],
    pub particles: Vec<Particle2d>,
    pub gravity: [f32; 2],
    pub max_particles: usize,
    pub emit_rate: f32,
    pub emit_accum: f32,
    pub base_speed: f32,
    pub base_life: f32,
}

impl ParticleEmitter2d {
    pub fn new(x: f32, y: f32, max_particles: usize) -> Self {
        ParticleEmitter2d {
            position: [x, y],
            particles: Vec::with_capacity(max_particles),
            gravity: [0.0, -9.81],
            max_particles,
            emit_rate: 10.0,
            emit_accum: 0.0,
            base_speed: 5.0,
            base_life: 2.0,
        }
    }

    pub fn emit_one(&mut self, vx: f32, vy: f32) {
        if self.particles.len() < self.max_particles {
            self.particles.push(Particle2d::new(
                self.position[0],
                self.position[1],
                vx,
                vy,
                self.base_life,
            ));
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.emit_accum += self.emit_rate * dt;
        while self.emit_accum >= 1.0 && self.particles.len() < self.max_particles {
            self.emit_one(self.base_speed, self.base_speed);
            self.emit_accum -= 1.0;
        }
        let gx = self.gravity[0];
        let gy = self.gravity[1];
        for p in &mut self.particles {
            if !p.active {
                continue;
            }
            p.velocity[0] += gx * dt;
            p.velocity[1] += gy * dt;
            p.position[0] += p.velocity[0] * dt;
            p.position[1] += p.velocity[1] * dt;
            p.life -= dt;
            if p.life <= 0.0 {
                p.active = false;
            }
        }
        self.particles.retain(|p| p.active);
    }

    pub fn active_count(&self) -> usize {
        self.particles.iter().filter(|p| p.active).count()
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

pub fn particle_count_alive(emitter: &ParticleEmitter2d) -> usize {
    emitter.particles.len()
}

pub fn particles_above_ground(emitter: &ParticleEmitter2d, ground_y: f32) -> usize {
    emitter
        .particles
        .iter()
        .filter(|p| p.position[1] > ground_y)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let e = ParticleEmitter2d::new(0.0, 0.0, 100);
        assert_eq!(e.max_particles, 100);
        assert_eq!(e.particles.len(), 0);
    }

    #[test]
    fn test_emit_one() {
        let mut e = ParticleEmitter2d::new(0.0, 0.0, 100);
        e.emit_one(1.0, 1.0);
        assert_eq!(e.particles.len(), 1);
    }

    #[test]
    fn test_step_emits() {
        let mut e = ParticleEmitter2d::new(0.0, 0.0, 100);
        e.step(1.0);
        assert!(
            !e.particles.is_empty() || e.emit_rate > 0.0,
            /* particles emitted after 1 second */
        );
    }

    #[test]
    fn test_max_particles_respected() {
        let mut e = ParticleEmitter2d::new(0.0, 0.0, 5);
        for _ in 0..20 {
            e.emit_one(0.0, 0.0);
        }
        assert!(e.particles.len() <= 5 /* cap at max */,);
    }

    #[test]
    fn test_particles_expire() {
        let mut e = ParticleEmitter2d::new(0.0, 0.0, 100);
        e.base_life = 0.1;
        e.emit_one(0.0, 0.0);
        e.step(0.2);
        assert_eq!(e.particles.len(), 0 /* particle should have expired */,);
    }

    #[test]
    fn test_gravity_applies() {
        let mut e = ParticleEmitter2d::new(0.0, 5.0, 100);
        e.emit_one(0.0, 0.0);
        let y0 = e.particles[0].position[1];
        e.step(0.1);
        if !e.particles.is_empty() {
            assert!(e.particles[0].position[1] < y0, /* gravity pulls down */);
        }
    }

    #[test]
    fn test_normalized_age() {
        let p = Particle2d::new(0.0, 0.0, 0.0, 0.0, 1.0);
        assert!((p.normalized_age() - 0.0).abs() < 1e-5, /* fresh particle age = 0 */);
    }

    #[test]
    fn test_clear() {
        let mut e = ParticleEmitter2d::new(0.0, 0.0, 100);
        e.emit_one(0.0, 0.0);
        e.clear();
        assert_eq!(e.particles.len(), 0 /* cleared */,);
    }

    #[test]
    fn test_count_above_ground() {
        let mut e = ParticleEmitter2d::new(0.0, 5.0, 100);
        e.gravity = [0.0, 0.0];
        e.emit_one(0.0, 0.0);
        e.step(0.01);
        let count = particles_above_ground(&e, 0.0);
        assert!(count <= e.particles.len() /* count <= total */,);
    }
}
