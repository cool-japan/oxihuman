// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sphere rigid body with dynamics.

use std::f32::consts::PI;

/// A sphere rigid body.
#[derive(Debug, Clone)]
pub struct SphereBody {
    pub radius: f32,
    pub mass: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub restitution: f32,
    pub enabled: bool,
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
impl SphereBody {
    pub fn new(radius: f32, mass: f32) -> Self {
        SphereBody {
            radius: radius.max(1e-6),
            mass: mass.max(1e-9),
            position: [0.0; 3],
            velocity: [0.0; 3],
            restitution: 0.5,
            enabled: true,
        }
    }

    pub fn step(&mut self, dt: f32, gravity: [f32; 3]) {
        if !self.enabled {
            return;
        }
        self.velocity[0] += gravity[0] * dt;
        self.velocity[1] += gravity[1] * dt;
        self.velocity[2] += gravity[2] * dt;
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;
    }

    pub fn apply_impulse(&mut self, j: [f32; 3]) {
        let inv_m = 1.0 / self.mass;
        self.velocity[0] += j[0] * inv_m;
        self.velocity[1] += j[1] * inv_m;
        self.velocity[2] += j[2] * inv_m;
    }

    pub fn volume(&self) -> f32 {
        (4.0 / 3.0) * PI * self.radius.powi(3)
    }

    pub fn surface_area(&self) -> f32 {
        4.0 * PI * self.radius.powi(2)
    }

    pub fn moment_of_inertia(&self) -> f32 {
        (2.0 / 5.0) * self.mass * self.radius.powi(2)
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * dot3(self.velocity, self.velocity)
    }

    pub fn speed(&self) -> f32 {
        len3(self.velocity)
    }

    pub fn intersects(&self, other: &SphereBody) -> bool {
        let d = len3(sub3(self.position, other.position));
        d < self.radius + other.radius
    }

    pub fn distance_to(&self, other: &SphereBody) -> f32 {
        let d = len3(sub3(self.position, other.position));
        (d - self.radius - other.radius).max(0.0)
    }

    pub fn stop(&mut self) {
        self.velocity = [0.0; 3];
    }
}

pub fn new_sphere_body(radius: f32, mass: f32) -> SphereBody {
    SphereBody::new(radius, mass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_formula() {
        let s = new_sphere_body(1.0, 1.0);
        assert!((s.volume() - (4.0 / 3.0) * PI).abs() < 1e-5);
    }

    #[test]
    fn surface_area() {
        let s = new_sphere_body(1.0, 1.0);
        assert!((s.surface_area() - 4.0 * PI).abs() < 1e-5);
    }

    #[test]
    fn moment_of_inertia() {
        let s = new_sphere_body(1.0, 5.0);
        assert!((s.moment_of_inertia() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn step_gravity() {
        let mut s = new_sphere_body(0.1, 1.0);
        s.step(1.0, [0.0, -9.8, 0.0]);
        assert!(s.position[1] < 0.0);
    }

    #[test]
    fn apply_impulse() {
        let mut s = new_sphere_body(0.1, 2.0);
        s.apply_impulse([4.0, 0.0, 0.0]);
        assert!((s.velocity[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn intersection() {
        let s1 = new_sphere_body(1.0, 1.0);
        let mut s2 = new_sphere_body(1.0, 1.0);
        s2.position = [1.5, 0.0, 0.0];
        assert!(s1.intersects(&s2));
    }

    #[test]
    fn no_intersection() {
        let s1 = new_sphere_body(0.5, 1.0);
        let mut s2 = new_sphere_body(0.5, 1.0);
        s2.position = [5.0, 0.0, 0.0];
        assert!(!s1.intersects(&s2));
    }

    #[test]
    fn kinetic_energy() {
        let mut s = new_sphere_body(1.0, 2.0);
        s.velocity = [3.0, 4.0, 0.0];
        assert!((s.kinetic_energy() - 25.0).abs() < 1e-5);
    }

    #[test]
    fn disabled_no_step() {
        let mut s = new_sphere_body(1.0, 1.0);
        s.enabled = false;
        s.step(1.0, [0.0, -9.8, 0.0]);
        assert_eq!(s.position[1], 0.0);
    }
}
