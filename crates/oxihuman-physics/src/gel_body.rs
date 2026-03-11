// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gel body: a visco-elastic deformable body modeled as a mass-spring lattice.

/// A single gel particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GelParticle {
    pub pos: [f32; 3],
    pub rest_pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
}

#[allow(dead_code)]
impl GelParticle {
    pub fn new(pos: [f32; 3], inv_mass: f32) -> Self {
        Self {
            pos,
            rest_pos: pos,
            vel: [0.0; 3],
            inv_mass,
        }
    }

    pub fn displacement(&self) -> f32 {
        let dx = self.pos[0] - self.rest_pos[0];
        let dy = self.pos[1] - self.rest_pos[1];
        let dz = self.pos[2] - self.rest_pos[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn is_fixed(&self) -> bool {
        self.inv_mass == 0.0
    }
}

/// A spring connecting two gel particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GelSpring {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// Gel material properties.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GelMaterial {
    pub young_modulus: f32,
    pub poisson_ratio: f32,
    pub viscosity: f32,
    pub density: f32,
}

impl Default for GelMaterial {
    fn default() -> Self {
        Self {
            young_modulus: 1000.0,
            poisson_ratio: 0.45,
            viscosity: 10.0,
            density: 1050.0,
        }
    }
}

/// The gel body simulation.
#[allow(dead_code)]
pub struct GelBody {
    pub particles: Vec<GelParticle>,
    pub springs: Vec<GelSpring>,
    pub material: GelMaterial,
}

#[allow(dead_code)]
impl GelBody {
    pub fn new(material: GelMaterial) -> Self {
        Self {
            particles: Vec::new(),
            springs: Vec::new(),
            material,
        }
    }

    pub fn add_particle(&mut self, pos: [f32; 3], inv_mass: f32) -> usize {
        let idx = self.particles.len();
        self.particles.push(GelParticle::new(pos, inv_mass));
        idx
    }

    pub fn add_spring(&mut self, a: usize, b: usize, stiffness: f32, damping: f32) {
        let pa = self.particles[a].pos;
        let pb = self.particles[b].pos;
        let dx = pb[0] - pa[0];
        let dy = pb[1] - pa[1];
        let dz = pb[2] - pa[2];
        let rest_len = (dx * dx + dy * dy + dz * dz).sqrt();
        self.springs.push(GelSpring {
            a,
            b,
            rest_len,
            stiffness,
            damping,
        });
    }

    /// Integrate one step under gravity.
    pub fn step(&mut self, dt: f32, gravity: f32) {
        let mut forces = vec![[0.0_f32; 3]; self.particles.len()];
        for s in &self.springs {
            let pa = self.particles[s.a].pos;
            let pb = self.particles[s.b].pos;
            let va = self.particles[s.a].vel;
            let vb = self.particles[s.b].vel;
            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let dz = pb[2] - pa[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist < 1e-9 {
                continue;
            }
            let nx = dx / dist;
            let ny = dy / dist;
            let nz = dz / dist;
            let spring_f = s.stiffness * (dist - s.rest_len);
            let rel_vel = (vb[0] - va[0]) * nx + (vb[1] - va[1]) * ny + (vb[2] - va[2]) * nz;
            let f = spring_f + s.damping * rel_vel;
            forces[s.a][0] += f * nx;
            forces[s.a][1] += f * ny;
            forces[s.a][2] += f * nz;
            forces[s.b][0] -= f * nx;
            forces[s.b][1] -= f * ny;
            forces[s.b][2] -= f * nz;
        }
        for (i, p) in self.particles.iter_mut().enumerate() {
            if p.is_fixed() {
                continue;
            }
            p.vel[0] += forces[i][0] * p.inv_mass * dt;
            p.vel[1] += (forces[i][1] * p.inv_mass - gravity) * dt;
            p.vel[2] += forces[i][2] * p.inv_mass * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
            p.pos[2] += p.vel[2] * dt;
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }

    pub fn max_displacement(&self) -> f32 {
        self.particles
            .iter()
            .map(|p| p.displacement())
            .fold(0.0_f32, f32::max)
    }

    pub fn elastic_energy(&self) -> f32 {
        self.springs
            .iter()
            .map(|s| {
                let pa = self.particles[s.a].pos;
                let pb = self.particles[s.b].pos;
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                let dz = pb[2] - pa[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                0.5 * s.stiffness * (dist - s.rest_len).powi(2)
            })
            .sum()
    }

    /// Bulk modulus from Young's modulus and Poisson ratio.
    pub fn bulk_modulus(&self) -> f32 {
        self.material.young_modulus / (3.0 * (1.0 - 2.0 * self.material.poisson_ratio))
    }

    /// Shear modulus.
    pub fn shear_modulus(&self) -> f32 {
        self.material.young_modulus / (2.0 * (1.0 + self.material.poisson_ratio))
    }
}

pub fn new_gel_body(mat: GelMaterial) -> GelBody {
    GelBody::new(mat)
}

pub fn gb_add_particle(body: &mut GelBody, pos: [f32; 3], inv_mass: f32) -> usize {
    body.add_particle(pos, inv_mass)
}

pub fn gb_add_spring(body: &mut GelBody, a: usize, b: usize, k: f32, c: f32) {
    body.add_spring(a, b, k, c);
}

pub fn gb_step(body: &mut GelBody, dt: f32, gravity: f32) {
    body.step(dt, gravity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_body_empty() {
        let b = new_gel_body(GelMaterial::default());
        assert_eq!(b.particle_count(), 0);
    }

    #[test]
    fn add_particle() {
        let mut b = new_gel_body(GelMaterial::default());
        gb_add_particle(&mut b, [0.0; 3], 1.0);
        assert_eq!(b.particle_count(), 1);
    }

    #[test]
    fn add_spring() {
        let mut b = new_gel_body(GelMaterial::default());
        gb_add_particle(&mut b, [0.0; 3], 1.0);
        gb_add_particle(&mut b, [1.0, 0.0, 0.0], 1.0);
        gb_add_spring(&mut b, 0, 1, 100.0, 1.0);
        assert_eq!(b.spring_count(), 1);
    }

    #[test]
    fn spring_rest_length() {
        let mut b = new_gel_body(GelMaterial::default());
        gb_add_particle(&mut b, [0.0; 3], 1.0);
        gb_add_particle(&mut b, [2.0, 0.0, 0.0], 1.0);
        gb_add_spring(&mut b, 0, 1, 10.0, 0.0);
        assert!((b.springs[0].rest_len - 2.0).abs() < 1e-5);
    }

    #[test]
    fn fixed_particle_no_move() {
        let mut b = new_gel_body(GelMaterial::default());
        gb_add_particle(&mut b, [0.0; 3], 0.0);
        gb_add_particle(&mut b, [1.0, 0.0, 0.0], 1.0);
        gb_add_spring(&mut b, 0, 1, 100.0, 1.0);
        let old = b.particles[0].pos;
        gb_step(&mut b, 0.01, 9.81);
        assert_eq!(b.particles[0].pos, old);
    }

    #[test]
    fn gravity_moves_free_particle_down() {
        let mut b = new_gel_body(GelMaterial::default());
        gb_add_particle(&mut b, [0.0; 3], 0.0);
        gb_add_particle(&mut b, [1.0, 0.0, 0.0], 1.0);
        gb_add_spring(&mut b, 0, 1, 0.0, 0.0);
        let old_y = b.particles[1].pos[1];
        gb_step(&mut b, 0.01, 9.81);
        assert!(b.particles[1].pos[1] < old_y);
    }

    #[test]
    fn elastic_energy_at_rest_zero() {
        let mut b = new_gel_body(GelMaterial::default());
        gb_add_particle(&mut b, [0.0; 3], 1.0);
        gb_add_particle(&mut b, [1.0, 0.0, 0.0], 1.0);
        gb_add_spring(&mut b, 0, 1, 100.0, 0.0);
        assert!(b.elastic_energy() < 1e-8);
    }

    #[test]
    fn bulk_modulus_positive() {
        let b = new_gel_body(GelMaterial::default());
        assert!(b.bulk_modulus() > 0.0);
    }

    #[test]
    fn shear_modulus_positive() {
        let b = new_gel_body(GelMaterial::default());
        assert!(b.shear_modulus() > 0.0);
    }

    #[test]
    fn max_displacement_starts_zero() {
        let mut b = new_gel_body(GelMaterial::default());
        gb_add_particle(&mut b, [0.0; 3], 1.0);
        assert!(b.max_displacement() < 1e-8);
    }

    #[test]
    fn no_nan_after_steps() {
        let mut b = new_gel_body(GelMaterial::default());
        let a = gb_add_particle(&mut b, [0.0; 3], 0.0);
        let c = gb_add_particle(&mut b, [1.0, 0.0, 0.0], 1.0);
        gb_add_spring(&mut b, a, c, 50.0, 2.0);
        for _ in 0..20 {
            gb_step(&mut b, 0.005, 0.0);
        }
        for p in &b.particles {
            assert!(!p.pos[0].is_nan());
        }
    }
}
