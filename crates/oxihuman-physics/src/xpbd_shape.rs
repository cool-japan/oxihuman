// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XPBD shape-matching constraint for rigid-like deformable bodies.

/// A shape-matching particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeParticle {
    /// Current position.
    pub pos: [f32; 3],
    /// Rest (goal) position in local frame.
    pub rest_pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
}

#[allow(dead_code)]
impl ShapeParticle {
    pub fn new(pos: [f32; 3], mass: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        Self {
            pos,
            rest_pos: pos,
            vel: [0.0; 3],
            inv_mass,
        }
    }
}

/// XPBD shape-matching body.
#[allow(dead_code)]
pub struct XpbdShape {
    pub particles: Vec<ShapeParticle>,
    pub gravity: [f32; 3],
    /// Shape-matching stiffness in [0, 1].
    pub stiffness: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl XpbdShape {
    pub fn new(stiffness: f32) -> Self {
        Self {
            particles: Vec::new(),
            gravity: [0.0, -9.81, 0.0],
            stiffness: stiffness.clamp(0.0, 1.0),
            time: 0.0,
            steps: 0,
        }
    }

    pub fn add_particle(&mut self, p: ShapeParticle) -> usize {
        let id = self.particles.len();
        self.particles.push(p);
        id
    }

    /// Compute the center of mass.
    pub fn center_of_mass(&self) -> [f32; 3] {
        if self.particles.is_empty() {
            return [0.0; 3];
        }
        let n = self.particles.len() as f32;
        let mut c = [0.0f32; 3];
        for p in &self.particles {
            c[0] += p.pos[0];
            c[1] += p.pos[1];
            c[2] += p.pos[2];
        }
        [c[0] / n, c[1] / n, c[2] / n]
    }

    /// Compute the rest center of mass.
    pub fn rest_center(&self) -> [f32; 3] {
        if self.particles.is_empty() {
            return [0.0; 3];
        }
        let n = self.particles.len() as f32;
        let mut c = [0.0f32; 3];
        for p in &self.particles {
            c[0] += p.rest_pos[0];
            c[1] += p.rest_pos[1];
            c[2] += p.rest_pos[2];
        }
        [c[0] / n, c[1] / n, c[2] / n]
    }

    pub fn step(&mut self, dt: f32) {
        // Integrate gravity.
        for p in &mut self.particles {
            if p.inv_mass < 1e-9 {
                continue;
            }
            p.vel[0] += self.gravity[0] * dt;
            p.vel[1] += self.gravity[1] * dt;
            p.vel[2] += self.gravity[2] * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
            p.pos[2] += p.vel[2] * dt;
        }
        // Shape matching: pull particles towards their goal positions.
        let cm_cur = self.center_of_mass();
        let cm_rest = self.rest_center();
        // Translate: goal_i = cm_cur + (rest_pos_i - cm_rest).
        for p in &mut self.particles {
            if p.inv_mass < 1e-9 {
                continue;
            }
            let goal = [
                cm_cur[0] + (p.rest_pos[0] - cm_rest[0]),
                cm_cur[1] + (p.rest_pos[1] - cm_rest[1]),
                cm_cur[2] + (p.rest_pos[2] - cm_rest[2]),
            ];
            p.pos[0] += self.stiffness * (goal[0] - p.pos[0]);
            p.pos[1] += self.stiffness * (goal[1] - p.pos[1]);
            p.pos[2] += self.stiffness * (goal[2] - p.pos[2]);
            // Update velocity from position change (approximate).
            let inv_dt = 1.0 / dt.max(1e-9);
            p.vel[0] = (p.pos[0] - (p.pos[0] - p.vel[0] * dt)) * inv_dt;
            p.vel[1] = (p.pos[1] - (p.pos[1] - p.vel[1] * dt)) * inv_dt;
            p.vel[2] = (p.pos[2] - (p.pos[2] - p.vel[2] * dt)) * inv_dt;
        }
        self.time += dt;
        self.steps += 1;
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn total_kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .filter(|p| p.inv_mass > 1e-9)
            .map(|p| {
                let v2 = p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1] + p.vel[2] * p.vel[2];
                0.5 * v2 / p.inv_mass
            })
            .sum()
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for XpbdShape {
    fn default() -> Self {
        Self::new(0.5)
    }
}

pub fn new_xpbd_shape(stiffness: f32) -> XpbdShape {
    XpbdShape::new(stiffness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_falls() {
        let mut s = new_xpbd_shape(0.0);
        s.add_particle(ShapeParticle::new([0.0, 5.0, 0.0], 1.0));
        s.step(0.5);
        assert!(s.particles[0].pos[1] < 5.0);
    }

    #[test]
    fn high_stiffness_maintains_shape() {
        let mut s = new_xpbd_shape(1.0);
        s.gravity = [0.0; 3];
        s.add_particle(ShapeParticle::new([0.0, 0.0, 0.0], 1.0));
        s.add_particle(ShapeParticle::new([1.0, 0.0, 0.0], 1.0));
        // Perturb.
        s.particles[0].pos[1] = 2.0;
        s.step(0.1);
        // With high stiffness shape matching should pull back.
        assert!(s.particles[0].pos[1].abs() < 2.0);
    }

    #[test]
    fn center_of_mass_correct() {
        let mut s = new_xpbd_shape(0.5);
        s.add_particle(ShapeParticle::new([0.0, 0.0, 0.0], 1.0));
        s.add_particle(ShapeParticle::new([2.0, 0.0, 0.0], 1.0));
        let cm = s.center_of_mass();
        assert!((cm[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn step_count() {
        let mut s = new_xpbd_shape(0.5);
        s.add_particle(ShapeParticle::new([0.0; 3], 1.0));
        s.step(0.1);
        s.step(0.1);
        assert_eq!(s.steps, 2);
    }

    #[test]
    fn time_advances() {
        let mut s = new_xpbd_shape(0.5);
        s.step(0.2);
        assert!((s.time - 0.2).abs() < 1e-5);
    }

    #[test]
    fn particle_count() {
        let mut s = new_xpbd_shape(0.5);
        s.add_particle(ShapeParticle::new([0.0; 3], 1.0));
        s.add_particle(ShapeParticle::new([1.0, 0.0, 0.0], 1.0));
        assert_eq!(s.particle_count(), 2);
    }

    #[test]
    fn clear_empties() {
        let mut s = new_xpbd_shape(0.5);
        s.add_particle(ShapeParticle::new([0.0; 3], 1.0));
        s.step(0.1);
        s.clear();
        assert_eq!(s.particle_count(), 0);
    }

    #[test]
    fn stiffness_clamped() {
        let s = new_xpbd_shape(2.0);
        assert!(s.stiffness <= 1.0);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut s = new_xpbd_shape(0.5);
        s.add_particle(ShapeParticle::new([0.0, 5.0, 0.0], 1.0));
        s.step(0.3);
        assert!(s.total_kinetic_energy() >= 0.0);
    }

    #[test]
    fn rest_center_matches_initial() {
        let mut s = new_xpbd_shape(0.5);
        s.add_particle(ShapeParticle::new([1.0, 0.0, 0.0], 1.0));
        s.add_particle(ShapeParticle::new([3.0, 0.0, 0.0], 1.0));
        let rc = s.rest_center();
        assert!((rc[0] - 2.0).abs() < 1e-5);
    }
}
