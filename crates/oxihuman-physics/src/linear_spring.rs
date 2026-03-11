// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A linear spring connecting two 3D points with Hooke's law.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearSpring {
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_dot(v, v).sqrt()
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
impl LinearSpring {
    pub fn new(rest_length: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            rest_length,
            stiffness,
            damping,
        }
    }

    pub fn from_positions(a: [f32; 3], b: [f32; 3], stiffness: f32, damping: f32) -> Self {
        Self {
            rest_length: vec3_len(vec3_sub(b, a)),
            stiffness,
            damping,
        }
    }

    /// Compute spring force on body A (toward B when stretched).
    pub fn force_on_a(
        &self,
        pos_a: [f32; 3],
        pos_b: [f32; 3],
        vel_a: [f32; 3],
        vel_b: [f32; 3],
    ) -> [f32; 3] {
        let delta = vec3_sub(pos_b, pos_a);
        let dist = vec3_len(delta);
        if dist < 1e-10 {
            return [0.0; 3];
        }
        let dir = vec3_scale(delta, 1.0 / dist);
        let extension = dist - self.rest_length;
        let rel_vel = vec3_sub(vel_b, vel_a);
        let vel_along = vec3_dot(rel_vel, dir);
        let magnitude = self.stiffness * extension + self.damping * vel_along;
        vec3_scale(dir, magnitude)
    }

    /// Potential energy stored in the spring.
    pub fn potential_energy(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let dist = vec3_len(vec3_sub(pos_b, pos_a));
        let ext = dist - self.rest_length;
        0.5 * self.stiffness * ext * ext
    }

    /// Current extension (positive = stretched, negative = compressed).
    pub fn extension(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        vec3_len(vec3_sub(pos_b, pos_a)) - self.rest_length
    }

    pub fn is_stretched(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> bool {
        self.extension(pos_a, pos_b) > 0.0
    }

    pub fn is_compressed(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> bool {
        self.extension(pos_a, pos_b) < 0.0
    }

    /// Natural frequency in rad/s for a given mass.
    pub fn natural_frequency(&self, mass: f32) -> f32 {
        if mass <= 0.0 {
            return 0.0;
        }
        (self.stiffness / mass).sqrt()
    }

    /// Critical damping coefficient for a given mass.
    pub fn critical_damping(&self, mass: f32) -> f32 {
        2.0 * (self.stiffness * mass).sqrt()
    }

    /// Damping ratio (< 1 underdamped, = 1 critically damped, > 1 overdamped).
    pub fn damping_ratio(&self, mass: f32) -> f32 {
        let cc = self.critical_damping(mass);
        if cc < 1e-10 {
            return 0.0;
        }
        self.damping / cc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = LinearSpring::new(1.0, 10.0, 0.5);
        assert!((s.rest_length - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_from_positions() {
        let s = LinearSpring::from_positions([0.0, 0.0, 0.0], [3.0, 4.0, 0.0], 10.0, 0.0);
        assert!((s.rest_length - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_force_at_rest() {
        let s = LinearSpring::new(1.0, 10.0, 0.0);
        let f = s.force_on_a([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(vec3_len(f) < 1e-6);
    }

    #[test]
    fn test_force_stretched() {
        let s = LinearSpring::new(1.0, 10.0, 0.0);
        let f = s.force_on_a([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!((f[0] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_potential_energy_at_rest() {
        let s = LinearSpring::new(1.0, 10.0, 0.0);
        let e = s.potential_energy([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(e < 1e-10);
    }

    #[test]
    fn test_potential_energy_stretched() {
        let s = LinearSpring::new(1.0, 10.0, 0.0);
        let e = s.potential_energy([0.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
        assert!((e - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_extension() {
        let s = LinearSpring::new(2.0, 10.0, 0.0);
        assert!((s.extension([0.0, 0.0, 0.0], [5.0, 0.0, 0.0]) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_stretched_compressed() {
        let s = LinearSpring::new(3.0, 10.0, 0.0);
        assert!(s.is_stretched([0.0, 0.0, 0.0], [5.0, 0.0, 0.0]));
        assert!(s.is_compressed([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_natural_frequency() {
        let s = LinearSpring::new(1.0, 100.0, 0.0);
        assert!((s.natural_frequency(1.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_damping_ratio() {
        let s = LinearSpring::new(1.0, 100.0, 20.0);
        let dr = s.damping_ratio(1.0);
        assert!((dr - 1.0).abs() < 1e-5);
    }
}
