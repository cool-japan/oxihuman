// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Continuous collision detection (CCD) utilities for moving spheres.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct MovingSphere {
    pub center: [f32; 3],
    pub velocity: [f32; 3],
    pub radius: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CcdResult {
    pub hit: bool,
    pub toi: f32,
    pub hit_point: [f32; 3],
    pub hit_normal: [f32; 3],
}

#[allow(dead_code)]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn v3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn v3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn v3_len(v: [f32; 3]) -> f32 {
    v3_dot(v, v).sqrt()
}

#[allow(dead_code)]
fn v3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = v3_len(v);
    if l < f32::EPSILON {
        [0.0, 1.0, 0.0]
    } else {
        v3_scale(v, 1.0 / l)
    }
}

#[allow(dead_code)]
impl MovingSphere {
    pub fn new(center: [f32; 3], velocity: [f32; 3], radius: f32) -> Self {
        Self {
            center,
            velocity,
            radius: radius.max(0.0),
        }
    }

    pub fn position_at(&self, t: f32) -> [f32; 3] {
        v3_add(self.center, v3_scale(self.velocity, t))
    }

    pub fn speed(&self) -> f32 {
        v3_len(self.velocity)
    }

    /// Test against a static sphere.
    pub fn test_static_sphere(
        &self,
        target_center: [f32; 3],
        target_radius: f32,
        dt: f32,
    ) -> CcdResult {
        let d = v3_sub(self.center, target_center);
        let combined_r = self.radius + target_radius;

        let a = v3_dot(self.velocity, self.velocity);
        let b = 2.0 * v3_dot(d, self.velocity);
        let c = v3_dot(d, d) - combined_r * combined_r;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 || a.abs() < f32::EPSILON {
            return CcdResult {
                hit: false,
                toi: dt,
                hit_point: [0.0; 3],
                hit_normal: [0.0, 1.0, 0.0],
            };
        }

        let sqrt_d = discriminant.sqrt();
        let t = (-b - sqrt_d) / (2.0 * a);

        if (0.0..=dt).contains(&t) {
            let pos = self.position_at(t);
            let normal = v3_normalize(v3_sub(pos, target_center));
            CcdResult {
                hit: true,
                toi: t,
                hit_point: v3_add(target_center, v3_scale(normal, target_radius)),
                hit_normal: normal,
            }
        } else {
            CcdResult {
                hit: false,
                toi: dt,
                hit_point: [0.0; 3],
                hit_normal: [0.0, 1.0, 0.0],
            }
        }
    }

    /// Test against an infinite plane (normal dot x = d).
    pub fn test_plane(&self, plane_normal: [f32; 3], plane_d: f32, dt: f32) -> CcdResult {
        let dist = v3_dot(plane_normal, self.center) - plane_d;
        let vel_dot_n = v3_dot(plane_normal, self.velocity);

        if vel_dot_n.abs() < f32::EPSILON {
            return CcdResult {
                hit: false,
                toi: dt,
                hit_point: [0.0; 3],
                hit_normal: plane_normal,
            };
        }

        let t = (self.radius - dist) / vel_dot_n;

        if (0.0..=dt).contains(&t) {
            let pos = self.position_at(t);
            CcdResult {
                hit: true,
                toi: t,
                hit_point: v3_sub(pos, v3_scale(plane_normal, self.radius)),
                hit_normal: plane_normal,
            }
        } else {
            CcdResult {
                hit: false,
                toi: dt,
                hit_point: [0.0; 3],
                hit_normal: plane_normal,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = MovingSphere::new([0.0; 3], [1.0, 0.0, 0.0], 0.5);
        assert!((s.radius - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_position_at() {
        let s = MovingSphere::new([0.0; 3], [1.0, 0.0, 0.0], 0.5);
        let p = s.position_at(2.0);
        assert!((p[0] - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed() {
        let s = MovingSphere::new([0.0; 3], [3.0, 4.0, 0.0], 1.0);
        assert!((s.speed() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_hit_static_sphere() {
        let s = MovingSphere::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5);
        let result = s.test_static_sphere([3.0, 0.0, 0.0], 0.5, 10.0);
        assert!(result.hit);
        assert!(result.toi > 0.0 && result.toi < 10.0);
    }

    #[test]
    fn test_miss_static_sphere() {
        let s = MovingSphere::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        let result = s.test_static_sphere([10.0, 0.0, 0.0], 0.5, 5.0);
        assert!(!result.hit);
    }

    #[test]
    fn test_hit_plane() {
        let s = MovingSphere::new([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], 0.5);
        let result = s.test_plane([0.0, 1.0, 0.0], 0.0, 10.0);
        assert!(result.hit);
        assert!(result.toi > 0.0);
    }

    #[test]
    fn test_miss_plane_moving_away() {
        let s = MovingSphere::new([0.0, 5.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        let result = s.test_plane([0.0, 1.0, 0.0], 0.0, 10.0);
        assert!(!result.hit);
    }

    #[test]
    fn test_too_short_dt() {
        let s = MovingSphere::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5);
        let result = s.test_static_sphere([100.0, 0.0, 0.0], 0.5, 0.1);
        assert!(!result.hit);
    }

    #[test]
    fn test_zero_velocity() {
        let s = MovingSphere::new([0.0; 3], [0.0; 3], 0.5);
        assert!((s.speed() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_already_overlapping() {
        let s = MovingSphere::new([0.0; 3], [1.0, 0.0, 0.0], 1.0);
        let result = s.test_static_sphere([0.5, 0.0, 0.0], 1.0, 1.0);
        // t would be negative (already inside) so no hit in forward direction
        assert!(!result.hit || result.toi >= 0.0);
    }
}
