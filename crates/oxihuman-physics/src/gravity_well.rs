// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A gravitational well that attracts bodies toward its center.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct GravityWell {
    center: [f32; 3],
    mass: f32,
    gravitational_constant: f32,
    min_distance: f32,
    max_distance: f32,
    active: bool,
}

#[allow(dead_code)]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn v3_len_sq(v: [f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

#[allow(dead_code)]
fn v3_len(v: [f32; 3]) -> f32 {
    v3_len_sq(v).sqrt()
}

#[allow(dead_code)]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
impl GravityWell {
    pub fn new(center: [f32; 3], mass: f32) -> Self {
        Self {
            center,
            mass: mass.max(0.0),
            gravitational_constant: 6.674e-11,
            min_distance: 0.1,
            max_distance: f32::MAX,
            active: true,
        }
    }

    pub fn with_constant(mut self, g: f32) -> Self {
        self.gravitational_constant = g.max(0.0);
        self
    }

    pub fn with_min_distance(mut self, d: f32) -> Self {
        self.min_distance = d.max(f32::EPSILON);
        self
    }

    pub fn with_max_distance(mut self, d: f32) -> Self {
        self.max_distance = d.max(0.0);
        self
    }

    pub fn force_on(&self, point: [f32; 3], point_mass: f32) -> [f32; 3] {
        if !self.active {
            return [0.0; 3];
        }
        let dir = v3_sub(self.center, point);
        let dist = v3_len(dir).max(self.min_distance);
        if dist > self.max_distance {
            return [0.0; 3];
        }
        let magnitude = self.gravitational_constant * self.mass * point_mass / (dist * dist);
        let norm = v3_scale(dir, 1.0 / dist);
        v3_scale(norm, magnitude)
    }

    pub fn potential_at(&self, point: [f32; 3]) -> f32 {
        let dist = v3_len(v3_sub(point, self.center)).max(self.min_distance);
        -self.gravitational_constant * self.mass / dist
    }

    pub fn escape_velocity(&self, distance: f32) -> f32 {
        let d = distance.max(self.min_distance);
        (2.0 * self.gravitational_constant * self.mass / d).sqrt()
    }

    pub fn orbital_velocity(&self, distance: f32) -> f32 {
        let d = distance.max(self.min_distance);
        (self.gravitational_constant * self.mass / d).sqrt()
    }

    pub fn orbital_period(&self, distance: f32) -> f32 {
        let d = distance.max(self.min_distance);
        2.0 * PI * (d * d * d / (self.gravitational_constant * self.mass)).sqrt()
    }

    pub fn center(&self) -> [f32; 3] {
        self.center
    }

    pub fn set_center(&mut self, c: [f32; 3]) {
        self.center = c;
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let gw = GravityWell::new([0.0; 3], 1e10);
        assert!(gw.is_active());
    }

    #[test]
    fn test_force_direction() {
        let gw = GravityWell::new([0.0; 3], 1e10).with_constant(1.0);
        let f = gw.force_on([10.0, 0.0, 0.0], 1.0);
        assert!(f[0] < 0.0); // attracted towards center
    }

    #[test]
    fn test_force_magnitude_decreases_with_distance() {
        let gw = GravityWell::new([0.0; 3], 1e10).with_constant(1.0);
        let f_near = gw.force_on([1.0, 0.0, 0.0], 1.0);
        let f_far = gw.force_on([2.0, 0.0, 0.0], 1.0);
        assert!(f_near[0].abs() > f_far[0].abs());
    }

    #[test]
    fn test_inactive() {
        let mut gw = GravityWell::new([0.0; 3], 1e10);
        gw.set_active(false);
        let f = gw.force_on([1.0, 0.0, 0.0], 1.0);
        assert!((f[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_distance() {
        let gw = GravityWell::new([0.0; 3], 1e10)
            .with_constant(1.0)
            .with_max_distance(5.0);
        let f = gw.force_on([10.0, 0.0, 0.0], 1.0);
        assert!((f[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_potential_negative() {
        let gw = GravityWell::new([0.0; 3], 1e10).with_constant(1.0);
        assert!(gw.potential_at([1.0, 0.0, 0.0]) < 0.0);
    }

    #[test]
    fn test_escape_velocity_positive() {
        let gw = GravityWell::new([0.0; 3], 1e10).with_constant(1.0);
        assert!(gw.escape_velocity(1.0) > 0.0);
    }

    #[test]
    fn test_orbital_velocity() {
        let gw = GravityWell::new([0.0; 3], 1e10).with_constant(1.0);
        let v_orb = gw.orbital_velocity(1.0);
        let v_esc = gw.escape_velocity(1.0);
        assert!(v_esc > v_orb);
    }

    #[test]
    fn test_orbital_period() {
        let gw = GravityWell::new([0.0; 3], 1e10).with_constant(1.0);
        assert!(gw.orbital_period(1.0) > 0.0);
    }

    #[test]
    fn test_set_center() {
        let mut gw = GravityWell::new([0.0; 3], 1.0);
        gw.set_center([5.0, 5.0, 5.0]);
        assert_eq!(gw.center(), [5.0, 5.0, 5.0]);
    }
}
