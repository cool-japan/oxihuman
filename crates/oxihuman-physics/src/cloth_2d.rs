// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D cloth simulation using edge springs.

#[derive(Debug, Clone)]
pub struct ClothParticle2d {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub mass: f32,
    pub pinned: bool,
}

impl ClothParticle2d {
    pub fn new(x: f32, y: f32, mass: f32) -> Self {
        ClothParticle2d {
            position: [x, y],
            velocity: [0.0; 2],
            mass,
            pinned: false,
        }
    }

    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ClothSpring2d {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl ClothSpring2d {
    pub fn new(a: usize, b: usize, rest_length: f32, stiffness: f32) -> Self {
        ClothSpring2d {
            a,
            b,
            rest_length,
            stiffness,
            damping: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cloth2d {
    pub particles: Vec<ClothParticle2d>,
    pub springs: Vec<ClothSpring2d>,
    pub gravity: [f32; 2],
}

impl Cloth2d {
    pub fn new(gravity: [f32; 2]) -> Self {
        Cloth2d {
            particles: Vec::new(),
            springs: Vec::new(),
            gravity,
        }
    }

    pub fn add_particle(&mut self, p: ClothParticle2d) -> usize {
        self.particles.push(p);
        self.particles.len() - 1
    }

    pub fn add_spring(&mut self, s: ClothSpring2d) {
        self.springs.push(s);
    }

    pub fn step(&mut self, dt: f32) {
        let n = self.particles.len();
        let mut forces = vec![[0.0f32; 2]; n];
        for (i, p) in self.particles.iter().enumerate() {
            if !p.pinned {
                forces[i][0] += p.mass * self.gravity[0];
                forces[i][1] += p.mass * self.gravity[1];
            }
        }
        let springs = self.springs.clone();
        for s in &springs {
            apply_spring_2d(&self.particles, &mut forces, s);
        }
        for (p, f) in self.particles.iter_mut().zip(forces.iter()) {
            if p.pinned {
                continue;
            }
            p.velocity[0] += f[0] / p.mass * dt;
            p.velocity[1] += f[1] / p.mass * dt;
            p.position[0] += p.velocity[0] * dt;
            p.position[1] += p.velocity[1] * dt;
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }
}

pub fn apply_spring_2d(particles: &[ClothParticle2d], forces: &mut [[f32; 2]], s: &ClothSpring2d) {
    let pa = &particles[s.a];
    let pb = &particles[s.b];
    let dx = pb.position[0] - pa.position[0];
    let dy = pb.position[1] - pa.position[1];
    let dist = (dx * dx + dy * dy).sqrt().max(1e-8);
    let extension = dist - s.rest_length;
    let mag = s.stiffness * extension;
    let nx = dx / dist;
    let ny = dy / dist;
    forces[s.a][0] += mag * nx;
    forces[s.a][1] += mag * ny;
    forces[s.b][0] -= mag * nx;
    forces[s.b][1] -= mag * ny;
}

pub fn cloth_total_kinetic_energy(cloth: &Cloth2d) -> f32 {
    cloth
        .particles
        .iter()
        .map(|p| 0.5 * p.mass * (p.velocity[0].powi(2) + p.velocity[1].powi(2)))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let cloth = Cloth2d::new([0.0, -9.81]);
        assert_eq!(cloth.particle_count(), 0);
    }

    #[test]
    fn test_add_particle() {
        let mut cloth = Cloth2d::new([0.0, -9.81]);
        cloth.add_particle(ClothParticle2d::new(0.0, 0.0, 1.0));
        assert_eq!(cloth.particle_count(), 1);
    }

    #[test]
    fn test_add_spring() {
        let mut cloth = Cloth2d::new([0.0, -9.81]);
        cloth.add_particle(ClothParticle2d::new(0.0, 0.0, 1.0));
        cloth.add_particle(ClothParticle2d::new(1.0, 0.0, 1.0));
        cloth.add_spring(ClothSpring2d::new(0, 1, 1.0, 100.0));
        assert_eq!(cloth.spring_count(), 1);
    }

    #[test]
    fn test_pinned_particle_doesnt_move() {
        let mut cloth = Cloth2d::new([0.0, -9.81]);
        cloth.add_particle(ClothParticle2d::new(0.0, 0.0, 1.0).pinned());
        cloth.step(0.01);
        assert!((cloth.particles[0].position[1] - 0.0).abs() < 1e-6, /* pinned stays */);
    }

    #[test]
    fn test_gravity_moves_free_particle() {
        let mut cloth = Cloth2d::new([0.0, -9.81]);
        cloth.add_particle(ClothParticle2d::new(0.0, 0.0, 1.0));
        cloth.step(0.1);
        assert!(cloth.particles[0].position[1] < 0.0, /* gravity pulls down */);
    }

    #[test]
    fn test_spring_force_prevents_drift() {
        let mut cloth = Cloth2d::new([0.0, 0.0]);
        let idx0 = cloth.add_particle(ClothParticle2d::new(0.0, 0.0, 1.0).pinned());
        let idx1 = cloth.add_particle(ClothParticle2d::new(2.0, 0.0, 1.0));
        cloth.add_spring(ClothSpring2d::new(idx0, idx1, 1.0, 1000.0));
        cloth.step(0.001);
        let x1 = cloth.particles[1].position[0];
        assert!(x1 < 2.0, /* spring pulls particle back toward rest length */);
    }

    #[test]
    fn test_kinetic_energy_at_rest() {
        let mut cloth = Cloth2d::new([0.0, 0.0]);
        cloth.add_particle(ClothParticle2d::new(0.0, 0.0, 1.0));
        let ke = cloth_total_kinetic_energy(&cloth);
        assert!((ke - 0.0).abs() < 1e-6 /* no velocity = zero KE */,);
    }

    #[test]
    fn test_multi_step() {
        let mut cloth = Cloth2d::new([0.0, -9.81]);
        cloth.add_particle(ClothParticle2d::new(0.0, 0.0, 1.0));
        for _ in 0..100 {
            cloth.step(0.01);
        }
        assert!(cloth.particles[0].position[1].is_finite(), /* position finite */);
    }

    #[test]
    fn test_spring_rest_length() {
        let s = ClothSpring2d::new(0, 1, 2.5, 100.0);
        assert!((s.rest_length - 2.5).abs() < 1e-5);
    }
}
