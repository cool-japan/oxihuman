// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A radial force field emanating from a point in space.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ForceFieldRadial {
    center: [f32; 3],
    strength: f32,
    falloff: FalloffType,
    max_radius: f32,
    active: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FalloffType {
    Constant,
    Linear,
    InverseSquare,
}

#[allow(dead_code)]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn v3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
impl ForceFieldRadial {
    pub fn new(center: [f32; 3], strength: f32) -> Self {
        Self {
            center,
            strength,
            falloff: FalloffType::InverseSquare,
            max_radius: f32::MAX,
            active: true,
        }
    }

    pub fn with_falloff(mut self, falloff: FalloffType) -> Self {
        self.falloff = falloff;
        self
    }

    pub fn with_max_radius(mut self, r: f32) -> Self {
        self.max_radius = r.max(0.0);
        self
    }

    pub fn force_at(&self, point: [f32; 3]) -> [f32; 3] {
        if !self.active {
            return [0.0; 3];
        }
        let dir = v3_sub(point, self.center);
        let dist = v3_len(dir);
        if dist < f32::EPSILON || dist > self.max_radius {
            return [0.0; 3];
        }
        let normalized = v3_scale(dir, 1.0 / dist);
        let magnitude = match self.falloff {
            FalloffType::Constant => self.strength,
            FalloffType::Linear => self.strength * (1.0 - dist / self.max_radius).max(0.0),
            FalloffType::InverseSquare => self.strength / (dist * dist),
        };
        v3_scale(normalized, magnitude)
    }

    pub fn magnitude_at_distance(&self, dist: f32) -> f32 {
        if dist < f32::EPSILON || dist > self.max_radius {
            return 0.0;
        }
        match self.falloff {
            FalloffType::Constant => self.strength.abs(),
            FalloffType::Linear => (self.strength * (1.0 - dist / self.max_radius).max(0.0)).abs(),
            FalloffType::InverseSquare => (self.strength / (dist * dist)).abs(),
        }
    }

    pub fn center(&self) -> [f32; 3] {
        self.center
    }

    pub fn set_center(&mut self, center: [f32; 3]) {
        self.center = center;
    }

    pub fn strength(&self) -> f32 {
        self.strength
    }

    pub fn set_strength(&mut self, s: f32) {
        self.strength = s;
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn max_radius(&self) -> f32 {
        self.max_radius
    }

    /// Volume of the spherical influence region.
    pub fn influence_volume(&self) -> f32 {
        if self.max_radius >= f32::MAX * 0.5 {
            return f32::INFINITY;
        }
        (4.0 / 3.0) * PI * self.max_radius * self.max_radius * self.max_radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let f = ForceFieldRadial::new([0.0; 3], 10.0);
        assert!(f.is_active());
        assert!((f.strength() - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_force_away_from_center() {
        let f = ForceFieldRadial::new([0.0; 3], 100.0).with_falloff(FalloffType::Constant);
        let force = f.force_at([1.0, 0.0, 0.0]);
        assert!(force[0] > 0.0);
    }

    #[test]
    fn test_force_at_center() {
        let f = ForceFieldRadial::new([0.0; 3], 100.0);
        let force = f.force_at([0.0; 3]);
        assert!((force[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inverse_square_falloff() {
        let f = ForceFieldRadial::new([0.0; 3], 100.0).with_falloff(FalloffType::InverseSquare);
        let m1 = f.magnitude_at_distance(1.0);
        let m2 = f.magnitude_at_distance(2.0);
        assert!((m1 / m2 - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_linear_falloff() {
        let f = ForceFieldRadial::new([0.0; 3], 10.0)
            .with_falloff(FalloffType::Linear)
            .with_max_radius(10.0);
        let m = f.magnitude_at_distance(5.0);
        assert!((m - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_max_radius() {
        let f = ForceFieldRadial::new([0.0; 3], 10.0).with_max_radius(5.0);
        let force = f.force_at([10.0, 0.0, 0.0]);
        assert!((force[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inactive() {
        let mut f = ForceFieldRadial::new([0.0; 3], 10.0);
        f.set_active(false);
        let force = f.force_at([1.0, 0.0, 0.0]);
        assert!((force[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_negative_strength() {
        let f = ForceFieldRadial::new([0.0; 3], -10.0).with_falloff(FalloffType::Constant);
        let force = f.force_at([1.0, 0.0, 0.0]);
        assert!(force[0] < 0.0);
    }

    #[test]
    fn test_set_center() {
        let mut f = ForceFieldRadial::new([0.0; 3], 10.0);
        f.set_center([1.0, 2.0, 3.0]);
        assert_eq!(f.center(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_influence_volume() {
        let f = ForceFieldRadial::new([0.0; 3], 10.0).with_max_radius(1.0);
        let expected = (4.0 / 3.0) * PI;
        assert!((f.influence_volume() - expected).abs() < 0.01);
    }
}
