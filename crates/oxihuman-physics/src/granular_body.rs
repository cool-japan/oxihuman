// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Granular body: a collection of grains with frictional contact and gravity.

use std::f32::consts::PI;

/// A single granular particle (grain).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Grain {
    pub id: u32,
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub radius: f32,
    pub mass: f32,
    pub restitution: f32,
}

#[allow(dead_code)]
impl Grain {
    pub fn new(id: u32, pos: [f32; 2], radius: f32, mass: f32) -> Self {
        Self {
            id,
            pos,
            vel: [0.0; 2],
            radius,
            mass,
            restitution: 0.3,
        }
    }

    pub fn area(&self) -> f32 {
        PI * self.radius * self.radius
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * (self.vel[0] * self.vel[0] + self.vel[1] * self.vel[1])
    }
}

/// A granular body simulation.
#[allow(dead_code)]
pub struct GranularBody {
    pub grains: Vec<Grain>,
    pub friction: f32,
    pub floor_y: f32,
}

#[allow(dead_code)]
impl GranularBody {
    pub fn new(friction: f32, floor_y: f32) -> Self {
        Self {
            grains: Vec::new(),
            friction,
            floor_y,
        }
    }

    pub fn add_grain(&mut self, pos: [f32; 2], radius: f32, mass: f32) -> u32 {
        let id = self.grains.len() as u32;
        self.grains.push(Grain::new(id, pos, radius, mass));
        id
    }

    /// Integrate under gravity for one step.
    pub fn integrate(&mut self, dt: f32, gravity: f32) {
        for g in &mut self.grains {
            g.vel[1] -= gravity * dt;
            g.pos[0] += g.vel[0] * dt;
            g.pos[1] += g.vel[1] * dt;
        }
    }

    /// Resolve floor collisions.
    pub fn resolve_floor(&mut self) {
        for g in &mut self.grains {
            let floor = self.floor_y + g.radius;
            if g.pos[1] < floor {
                g.pos[1] = floor;
                if g.vel[1] < 0.0 {
                    g.vel[1] = -g.vel[1] * g.restitution;
                    g.vel[0] *= 1.0 - self.friction;
                }
            }
        }
    }

    /// Resolve grain-grain collisions (elastic with restitution).
    pub fn resolve_grain_contacts(&mut self) {
        let n = self.grains.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.grains[j].pos[0] - self.grains[i].pos[0];
                let dy = self.grains[j].pos[1] - self.grains[i].pos[1];
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = self.grains[i].radius + self.grains[j].radius;
                if dist >= min_dist || dist < 1e-9 {
                    continue;
                }
                let nx = dx / dist;
                let ny = dy / dist;
                // Push apart — fully resolve penetration
                let overlap = min_dist - dist;
                let mi = self.grains[i].mass;
                let mj = self.grains[j].mass;
                let total = mi + mj;
                self.grains[i].pos[0] -= nx * overlap * mj / total;
                self.grains[i].pos[1] -= ny * overlap * mj / total;
                self.grains[j].pos[0] += nx * overlap * mi / total;
                self.grains[j].pos[1] += ny * overlap * mi / total;
                // Velocity exchange
                let vi_n = self.grains[i].vel[0] * nx + self.grains[i].vel[1] * ny;
                let vj_n = self.grains[j].vel[0] * nx + self.grains[j].vel[1] * ny;
                let e = (self.grains[i].restitution + self.grains[j].restitution) * 0.5;
                let vi_new = (mi * vi_n + mj * vj_n + mj * e * (vj_n - vi_n)) / total;
                let vj_new = (mi * vi_n + mj * vj_n + mi * e * (vi_n - vj_n)) / total;
                self.grains[i].vel[0] += (vi_new - vi_n) * nx;
                self.grains[i].vel[1] += (vi_new - vi_n) * ny;
                self.grains[j].vel[0] += (vj_new - vj_n) * nx;
                self.grains[j].vel[1] += (vj_new - vj_n) * ny;
            }
        }
    }

    pub fn step(&mut self, dt: f32, gravity: f32) {
        self.integrate(dt, gravity);
        self.resolve_floor();
        self.resolve_grain_contacts();
    }

    pub fn grain_count(&self) -> usize {
        self.grains.len()
    }

    pub fn total_kinetic_energy(&self) -> f32 {
        self.grains.iter().map(|g| g.kinetic_energy()).sum()
    }

    pub fn center_of_mass(&self) -> [f32; 2] {
        if self.grains.is_empty() {
            return [0.0; 2];
        }
        let total_mass: f32 = self.grains.iter().map(|g| g.mass).sum();
        let cx: f32 = self.grains.iter().map(|g| g.pos[0] * g.mass).sum::<f32>() / total_mass;
        let cy: f32 = self.grains.iter().map(|g| g.pos[1] * g.mass).sum::<f32>() / total_mass;
        [cx, cy]
    }
}

pub fn new_granular_body(friction: f32, floor_y: f32) -> GranularBody {
    GranularBody::new(friction, floor_y)
}

pub fn grb_add(body: &mut GranularBody, pos: [f32; 2], radius: f32, mass: f32) -> u32 {
    body.add_grain(pos, radius, mass)
}

pub fn grb_step(body: &mut GranularBody, dt: f32, gravity: f32) {
    body.step(dt, gravity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_body() {
        let b = new_granular_body(0.1, 0.0);
        assert_eq!(b.grain_count(), 0);
    }

    #[test]
    fn add_grain() {
        let mut b = new_granular_body(0.1, 0.0);
        grb_add(&mut b, [0.0, 1.0], 0.1, 1.0);
        assert_eq!(b.grain_count(), 1);
    }

    #[test]
    fn grain_area_formula() {
        let g = Grain::new(0, [0.0; 2], 1.0, 1.0);
        assert!((g.area() - PI).abs() < 1e-5);
    }

    #[test]
    fn gravity_drops_grain() {
        let mut b = new_granular_body(0.0, -100.0);
        grb_add(&mut b, [0.0, 1.0], 0.1, 1.0);
        let old_y = b.grains[0].pos[1];
        grb_step(&mut b, 0.1, 9.81);
        assert!(b.grains[0].pos[1] < old_y);
    }

    #[test]
    fn floor_collision_bounces() {
        let mut b = new_granular_body(0.0, 0.0);
        grb_add(&mut b, [0.0, 0.05], 0.1, 1.0);
        b.grains[0].vel[1] = -5.0;
        b.resolve_floor();
        assert!(b.grains[0].vel[1] >= 0.0);
    }

    #[test]
    fn grain_contact_separation() {
        let mut b = new_granular_body(0.0, -100.0);
        grb_add(&mut b, [0.0, 0.0], 0.5, 1.0);
        grb_add(&mut b, [0.5, 0.0], 0.5, 1.0); // overlapping
        b.resolve_grain_contacts();
        let dx = b.grains[1].pos[0] - b.grains[0].pos[0];
        assert!(dx >= 1.0 - 1e-4);
    }

    #[test]
    fn total_kinetic_energy_positive() {
        let mut b = new_granular_body(0.0, -100.0);
        grb_add(&mut b, [0.0, 5.0], 0.1, 1.0);
        grb_step(&mut b, 0.1, 9.81);
        assert!(b.total_kinetic_energy() > 0.0);
    }

    #[test]
    fn center_of_mass_single_grain() {
        let mut b = new_granular_body(0.0, -100.0);
        grb_add(&mut b, [3.0, 4.0], 0.1, 2.0);
        let cm = b.center_of_mass();
        assert!((cm[0] - 3.0).abs() < 1e-5);
        assert!((cm[1] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn no_nan_after_many_steps() {
        let mut b = new_granular_body(0.1, 0.0);
        for i in 0..5 {
            grb_add(&mut b, [i as f32 * 0.3, 2.0], 0.1, 1.0);
        }
        for _ in 0..30 {
            grb_step(&mut b, 0.01, 9.81);
        }
        for g in &b.grains {
            assert!(!g.pos[0].is_nan());
        }
    }

    #[test]
    fn kinetic_energy_formula() {
        let mut g = Grain::new(0, [0.0; 2], 0.1, 2.0);
        g.vel = [3.0, 4.0];
        assert!((g.kinetic_energy() - 25.0).abs() < 1e-4);
    }
}
