// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Velocity-Verlet integrator for particle systems.

/// A single particle with position, velocity, and acceleration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VvParticle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub acc: [f32; 3],
    pub mass: f32,
    pub inv_mass: f32,
}

#[allow(dead_code)]
impl VvParticle {
    pub fn new(pos: [f32; 3], mass: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        Self {
            pos,
            vel: [0.0; 3],
            acc: [0.0; 3],
            mass,
            inv_mass,
        }
    }

    pub fn with_vel(mut self, vel: [f32; 3]) -> Self {
        self.vel = vel;
        self
    }

    pub fn kinetic_energy(&self) -> f32 {
        let v2 = self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2];
        0.5 * self.mass * v2
    }

    pub fn speed(&self) -> f32 {
        (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1] + self.vel[2] * self.vel[2]).sqrt()
    }
}

/// Velocity-Verlet integrator state.
#[allow(dead_code)]
pub struct VelocityVerlet {
    pub particles: Vec<VvParticle>,
    pub gravity: [f32; 3],
    pub damping: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl VelocityVerlet {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            gravity: [0.0, -9.81, 0.0],
            damping: 0.999,
            time: 0.0,
            steps: 0,
        }
    }

    pub fn add_particle(&mut self, p: VvParticle) -> usize {
        let id = self.particles.len();
        self.particles.push(p);
        id
    }

    pub fn set_gravity(&mut self, g: [f32; 3]) {
        self.gravity = g;
    }

    pub fn set_damping(&mut self, d: f32) {
        self.damping = d.clamp(0.0, 1.0);
    }

    pub fn apply_force(&mut self, id: usize, force: [f32; 3]) {
        if let Some(p) = self.particles.get_mut(id) {
            p.acc[0] += force[0] * p.inv_mass;
            p.acc[1] += force[1] * p.inv_mass;
            p.acc[2] += force[2] * p.inv_mass;
        }
    }

    /// Integrate one step using velocity-Verlet.
    pub fn step(&mut self, dt: f32) {
        for p in &mut self.particles {
            // Include gravity in acceleration.
            let ax = p.acc[0] + self.gravity[0];
            let ay = p.acc[1] + self.gravity[1];
            let az = p.acc[2] + self.gravity[2];

            // Update position: x += v*dt + 0.5*a*dt^2
            p.pos[0] += p.vel[0] * dt + 0.5 * ax * dt * dt;
            p.pos[1] += p.vel[1] * dt + 0.5 * ay * dt * dt;
            p.pos[2] += p.vel[2] * dt + 0.5 * az * dt * dt;

            // Update velocity: v += a*dt, apply damping.
            p.vel[0] = (p.vel[0] + ax * dt) * self.damping;
            p.vel[1] = (p.vel[1] + ay * dt) * self.damping;
            p.vel[2] = (p.vel[2] + az * dt) * self.damping;

            // Reset per-step forces (gravity is applied externally).
            p.acc = [0.0; 3];
        }
        self.time += dt;
        self.steps += 1;
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

impl Default for VelocityVerlet {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_velocity_verlet() -> VelocityVerlet {
    VelocityVerlet::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_falls_under_gravity() {
        let mut vv = new_velocity_verlet();
        vv.add_particle(VvParticle::new([0.0, 10.0, 0.0], 1.0));
        vv.step(1.0);
        assert!(vv.particles[0].pos[1] < 10.0);
    }

    #[test]
    fn kinetic_energy_increases() {
        let mut vv = new_velocity_verlet();
        vv.add_particle(VvParticle::new([0.0, 100.0, 0.0], 1.0));
        let e0 = vv.total_kinetic_energy();
        vv.step(0.1);
        let e1 = vv.total_kinetic_energy();
        assert!(e1 > e0);
    }

    #[test]
    fn no_gravity_no_motion() {
        let mut vv = new_velocity_verlet();
        vv.set_gravity([0.0, 0.0, 0.0]);
        vv.set_damping(1.0);
        vv.add_particle(VvParticle::new([1.0, 2.0, 3.0], 1.0));
        vv.step(1.0);
        let p = &vv.particles[0];
        assert!((p.pos[0] - 1.0).abs() < 1e-5);
        assert!((p.pos[1] - 2.0).abs() < 1e-5);
        assert!((p.pos[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn applied_force_moves_particle() {
        let mut vv = new_velocity_verlet();
        vv.set_gravity([0.0, 0.0, 0.0]);
        vv.set_damping(1.0);
        let id = vv.add_particle(VvParticle::new([0.0; 3], 1.0));
        vv.apply_force(id, [10.0, 0.0, 0.0]);
        vv.step(1.0);
        assert!(vv.particles[0].pos[0] > 0.0);
    }

    #[test]
    fn damping_reduces_speed() {
        let mut vv = new_velocity_verlet();
        vv.set_gravity([0.0, 0.0, 0.0]);
        vv.set_damping(0.5);
        vv.add_particle(VvParticle::new([0.0; 3], 1.0).with_vel([10.0, 0.0, 0.0]));
        vv.step(0.01);
        assert!(vv.particles[0].speed() < 10.0);
    }

    #[test]
    fn step_count_increments() {
        let mut vv = new_velocity_verlet();
        vv.step(0.016);
        vv.step(0.016);
        assert_eq!(vv.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut vv = new_velocity_verlet();
        vv.step(0.5);
        assert!((vv.time - 0.5).abs() < 1e-6);
    }

    #[test]
    fn clear_resets() {
        let mut vv = new_velocity_verlet();
        vv.add_particle(VvParticle::new([0.0; 3], 1.0));
        vv.step(1.0);
        vv.clear();
        assert_eq!(vv.particle_count(), 0);
        assert_eq!(vv.steps, 0);
    }

    #[test]
    fn infinite_mass_particle_static() {
        let mut vv = new_velocity_verlet();
        vv.set_gravity([0.0, 0.0, 0.0]);
        vv.add_particle(VvParticle::new([5.0, 5.0, 5.0], 0.0)); // inv_mass=0
        vv.apply_force(0, [1000.0, 0.0, 0.0]);
        vv.step(1.0);
        // static particle should not move from force (acc stays 0), but gravity is zero
        // pos only changes by vel*dt + 0.5*acc*dt^2; vel=0, acc=0 → no change
        assert!((vv.particles[0].pos[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn particle_count() {
        let mut vv = new_velocity_verlet();
        vv.add_particle(VvParticle::new([0.0; 3], 1.0));
        vv.add_particle(VvParticle::new([1.0; 3], 2.0));
        assert_eq!(vv.particle_count(), 2);
    }
}
