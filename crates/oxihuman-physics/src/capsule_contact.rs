// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Contact information from capsule-capsule collision detection.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CapsuleContactResult {
    pub point_a: [f32; 3],
    pub point_b: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub is_colliding: bool,
}

/// A capsule defined by two endpoints and a radius.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PhysCapsule {
    pub p0: [f32; 3],
    pub p1: [f32; 3],
    pub radius: f32,
}

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
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
fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len < f32::EPSILON {
        return [0.0, 1.0, 0.0];
    }
    vec3_scale(v, 1.0 / len)
}

#[allow(dead_code)]
fn closest_point_on_segment(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    let ab = vec3_sub(b, a);
    let ap = vec3_sub(p, a);
    let t = vec3_dot(ap, ab) / vec3_dot(ab, ab).max(f32::EPSILON);
    let t = t.clamp(0.0, 1.0);
    vec3_add(a, vec3_scale(ab, t))
}

#[allow(dead_code)]
impl PhysCapsule {
    pub fn new(p0: [f32; 3], p1: [f32; 3], radius: f32) -> Self {
        Self {
            p0,
            p1,
            radius: radius.max(0.0),
        }
    }

    pub fn height(&self) -> f32 {
        vec3_len(vec3_sub(self.p1, self.p0))
    }

    pub fn center(&self) -> [f32; 3] {
        vec3_scale(vec3_add(self.p0, self.p1), 0.5)
    }

    pub fn volume(&self) -> f32 {
        let h = self.height();
        let r = self.radius;
        PI * r * r * h + (4.0 / 3.0) * PI * r * r * r
    }

    pub fn surface_area(&self) -> f32 {
        let h = self.height();
        let r = self.radius;
        2.0 * PI * r * h + 4.0 * PI * r * r
    }

    pub fn test_capsule(&self, other: &PhysCapsule) -> CapsuleContactResult {
        let d1 = vec3_sub(self.p1, self.p0);
        let d2 = vec3_sub(other.p1, other.p0);
        let r = vec3_sub(self.p0, other.p0);

        let a = vec3_dot(d1, d1);
        let e = vec3_dot(d2, d2);
        let f = vec3_dot(d2, r);

        let (s, t);
        if a <= f32::EPSILON && e <= f32::EPSILON {
            s = 0.0;
            t = 0.0;
        } else if a <= f32::EPSILON {
            s = 0.0;
            t = (f / e).clamp(0.0, 1.0);
        } else {
            let c = vec3_dot(d1, r);
            if e <= f32::EPSILON {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else {
                let b = vec3_dot(d1, d2);
                let denom = a * e - b * b;
                s = if denom.abs() > f32::EPSILON {
                    ((b * f - c * e) / denom).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                t = ((b * s + f) / e).clamp(0.0, 1.0);
            }
        }

        let closest_a = vec3_add(self.p0, vec3_scale(d1, s));
        let closest_b = vec3_add(other.p0, vec3_scale(d2, t));
        let diff = vec3_sub(closest_a, closest_b);
        let dist = vec3_len(diff);
        let sum_radii = self.radius + other.radius;
        let depth = sum_radii - dist;
        let normal = if dist > f32::EPSILON {
            vec3_normalize(diff)
        } else {
            [0.0, 1.0, 0.0]
        };

        CapsuleContactResult {
            point_a: closest_a,
            point_b: closest_b,
            normal,
            depth,
            is_colliding: depth > 0.0,
        }
    }

    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        let closest = closest_point_on_segment(p, self.p0, self.p1);
        let d = vec3_sub(p, closest);
        vec3_dot(d, d) <= self.radius * self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!((c.radius - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_height() {
        let c = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5);
        assert!((c.height() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_center() {
        let c = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 4.0, 0.0], 1.0);
        let ctr = c.center();
        assert!((ctr[1] - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_volume_positive() {
        let c = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!(c.volume() > 0.0);
    }

    #[test]
    fn test_collision_overlap() {
        let a = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        let b = PhysCapsule::new([0.5, 0.0, 0.0], [0.5, 1.0, 0.0], 0.5);
        let result = a.test_capsule(&b);
        assert!(result.is_colliding);
        assert!(result.depth > 0.0);
    }

    #[test]
    fn test_no_collision() {
        let a = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1);
        let b = PhysCapsule::new([5.0, 0.0, 0.0], [5.0, 1.0, 0.0], 0.1);
        let result = a.test_capsule(&b);
        assert!(!result.is_colliding);
    }

    #[test]
    fn test_contains_point() {
        let c = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 1.0);
        assert!(c.contains_point([0.0, 1.0, 0.0]));
        assert!(!c.contains_point([5.0, 0.0, 0.0]));
    }

    #[test]
    fn test_surface_area() {
        let c = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!(c.surface_area() > 0.0);
    }

    #[test]
    fn test_touching_capsules() {
        let a = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        let b = PhysCapsule::new([1.0, 0.0, 0.0], [1.0, 1.0, 0.0], 0.5);
        let result = a.test_capsule(&b);
        assert!(result.depth.abs() < 0.01);
    }

    #[test]
    fn test_degenerate_capsule() {
        let a = PhysCapsule::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        let b = PhysCapsule::new([0.5, 0.0, 0.0], [0.5, 0.0, 0.0], 1.0);
        let result = a.test_capsule(&b);
        assert!(result.is_colliding);
    }
}
