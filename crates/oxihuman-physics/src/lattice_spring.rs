// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A 2D lattice of particles connected by springs.
#[allow(dead_code)]
pub struct LatticeParticle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub pinned: bool,
}

#[allow(dead_code)]
pub struct LatticeSpring {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
}

#[allow(dead_code)]
pub struct LatticeSim {
    pub particles: Vec<LatticeParticle>,
    pub springs: Vec<LatticeSpring>,
}

#[allow(dead_code)]
impl LatticeSim {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            springs: Vec::new(),
        }
    }
    pub fn add_particle(&mut self, x: f32, y: f32, pinned: bool) -> usize {
        let idx = self.particles.len();
        self.particles.push(LatticeParticle {
            pos: [x, y],
            vel: [0.0; 2],
            pinned,
        });
        idx
    }
    pub fn add_spring(&mut self, a: usize, b: usize, stiffness: f32) {
        let dx = self.particles[b].pos[0] - self.particles[a].pos[0];
        let dy = self.particles[b].pos[1] - self.particles[a].pos[1];
        let rest_len = (dx * dx + dy * dy).sqrt();
        self.springs.push(LatticeSpring {
            a,
            b,
            rest_len,
            stiffness,
        });
    }
    pub fn step(&mut self, dt: f32, damping: f32) {
        let mut forces = vec![[0.0f32; 2]; self.particles.len()];
        for s in &self.springs {
            let pa = self.particles[s.a].pos;
            let pb = self.particles[s.b].pos;
            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let len = (dx * dx + dy * dy).sqrt().max(1e-8);
            let f = s.stiffness * (len - s.rest_len) / len;
            forces[s.a][0] += f * dx;
            forces[s.a][1] += f * dy;
            forces[s.b][0] -= f * dx;
            forces[s.b][1] -= f * dy;
        }
        for (i, p) in self.particles.iter_mut().enumerate() {
            if p.pinned {
                continue;
            }
            p.vel[0] = (p.vel[0] + forces[i][0] * dt) * (1.0 - damping * dt);
            p.vel[1] = (p.vel[1] + forces[i][1] * dt) * (1.0 - damping * dt);
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
    pub fn total_kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .map(|p| 0.5 * (p.vel[0].powi(2) + p.vel[1].powi(2)))
            .sum()
    }
    pub fn center_of_mass(&self) -> [f32; 2] {
        if self.particles.is_empty() {
            return [0.0; 2];
        }
        let n = self.particles.len() as f32;
        let sx: f32 = self.particles.iter().map(|p| p.pos[0]).sum();
        let sy: f32 = self.particles.iter().map(|p| p.pos[1]).sum();
        [sx / n, sy / n]
    }
    pub fn aabb(&self) -> ([f32; 2], [f32; 2]) {
        if self.particles.is_empty() {
            return ([0.0; 2], [0.0; 2]);
        }
        let mut mn = [f32::INFINITY; 2];
        let mut mx = [f32::NEG_INFINITY; 2];
        for p in &self.particles {
            for i in 0..2 {
                mn[i] = mn[i].min(p.pos[i]);
                mx[i] = mx[i].max(p.pos[i]);
            }
        }
        (mn, mx)
    }
}

impl Default for LatticeSim {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_lattice_sim() -> LatticeSim {
    LatticeSim::new()
}
#[allow(dead_code)]
pub fn ls_add_particle(s: &mut LatticeSim, x: f32, y: f32, pinned: bool) -> usize {
    s.add_particle(x, y, pinned)
}
#[allow(dead_code)]
pub fn ls_add_spring(s: &mut LatticeSim, a: usize, b: usize, stiffness: f32) {
    s.add_spring(a, b, stiffness);
}
#[allow(dead_code)]
pub fn ls_step(s: &mut LatticeSim, dt: f32, damping: f32) {
    s.step(dt, damping);
}
#[allow(dead_code)]
pub fn ls_particle_count(s: &LatticeSim) -> usize {
    s.particle_count()
}
#[allow(dead_code)]
pub fn ls_spring_count(s: &LatticeSim) -> usize {
    s.spring_count()
}
#[allow(dead_code)]
pub fn ls_kinetic_energy(s: &LatticeSim) -> f32 {
    s.total_kinetic_energy()
}
#[allow(dead_code)]
pub fn ls_center_of_mass(s: &LatticeSim) -> [f32; 2] {
    s.center_of_mass()
}
#[allow(dead_code)]
pub fn ls_aabb(s: &LatticeSim) -> ([f32; 2], [f32; 2]) {
    s.aabb()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add_particle() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, 0.0, 0.0, false);
        ls_add_particle(&mut s, 1.0, 0.0, false);
        assert_eq!(ls_particle_count(&s), 2);
    }
    #[test]
    fn test_add_spring() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, 0.0, 0.0, false);
        ls_add_particle(&mut s, 1.0, 0.0, false);
        ls_add_spring(&mut s, 0, 1, 10.0);
        assert_eq!(ls_spring_count(&s), 1);
    }
    #[test]
    fn test_step_doesnt_panic() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, 0.0, 0.0, true);
        ls_add_particle(&mut s, 0.5, 0.0, false);
        ls_add_spring(&mut s, 0, 1, 5.0);
        ls_step(&mut s, 0.016, 0.1);
    }
    #[test]
    fn test_pinned_particle_doesnt_move() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, 0.0, 0.0, true);
        ls_add_particle(&mut s, 2.0, 0.0, false);
        ls_add_spring(&mut s, 0, 1, 100.0);
        for _ in 0..10 {
            ls_step(&mut s, 0.01, 0.0);
        }
        assert_eq!(s.particles[0].pos, [0.0, 0.0]);
    }
    #[test]
    fn test_kinetic_energy_zero_initially() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, 0.0, 0.0, false);
        assert_eq!(ls_kinetic_energy(&s), 0.0);
    }
    #[test]
    fn test_center_of_mass() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, 0.0, 0.0, false);
        ls_add_particle(&mut s, 2.0, 0.0, false);
        let com = ls_center_of_mass(&s);
        assert!((com[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn test_aabb() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, -1.0, -2.0, false);
        ls_add_particle(&mut s, 3.0, 4.0, false);
        let (mn, mx) = ls_aabb(&s);
        assert_eq!(mn[0], -1.0);
        assert_eq!(mx[1], 4.0);
    }
    #[test]
    fn test_spring_rest_length() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, 0.0, 0.0, false);
        ls_add_particle(&mut s, 3.0, 4.0, false);
        ls_add_spring(&mut s, 0, 1, 1.0);
        assert!((s.springs[0].rest_len - 5.0).abs() < 1e-5);
    }
    #[test]
    fn test_empty_lattice() {
        let s = new_lattice_sim();
        assert_eq!(ls_particle_count(&s), 0);
        assert_eq!(ls_spring_count(&s), 0);
        assert_eq!(ls_kinetic_energy(&s), 0.0);
    }
    #[test]
    fn test_step_increases_kinetic_energy() {
        let mut s = new_lattice_sim();
        ls_add_particle(&mut s, 0.0, 0.0, true);
        // Place at 0.5 but the spring is added between these two, giving rest_len = 0.5
        ls_add_particle(&mut s, 0.5, 0.0, false);
        ls_add_spring(&mut s, 0, 1, 50.0);
        // Manually displace particle 1 away from rest to create spring force
        s.particles[1].pos = [2.0, 0.0];
        ls_step(&mut s, 0.01, 0.0);
        assert!(ls_kinetic_energy(&s) > 0.0);
    }
}
