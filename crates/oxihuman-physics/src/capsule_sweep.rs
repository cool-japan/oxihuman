// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Result of a capsule sweep test.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapsuleSweepResult {
    pub hit: bool,
    pub t: f32,
    pub normal: [f32; 3],
    pub point: [f32; 3],
}

#[allow(dead_code)]
impl CapsuleSweepResult {
    pub fn miss() -> Self {
        Self {
            hit: false,
            t: f32::MAX,
            normal: [0.0, 0.0, 0.0],
            point: [0.0, 0.0, 0.0],
        }
    }

    pub fn new_hit(t: f32, normal: [f32; 3], point: [f32; 3]) -> Self {
        Self {
            hit: true,
            t,
            normal,
            point,
        }
    }
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
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_dot(v, v).sqrt()
}

#[allow(dead_code)]
fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = vec3_len(v);
    if l < 1e-10 {
        [0.0, 0.0, 0.0]
    } else {
        vec3_scale(v, 1.0 / l)
    }
}

/// Sweep a sphere along a direction and test against a plane at y=plane_y.
#[allow(dead_code)]
pub fn sphere_sweep_plane(
    center: [f32; 3],
    radius: f32,
    direction: [f32; 3],
    plane_y: f32,
) -> CapsuleSweepResult {
    let dist = center[1] - radius - plane_y;
    if dist <= 0.0 {
        return CapsuleSweepResult::new_hit(0.0, [0.0, 1.0, 0.0], [center[0], plane_y, center[2]]);
    }
    if direction[1] >= 0.0 {
        return CapsuleSweepResult::miss();
    }
    let t = dist / (-direction[1]);
    if (0.0..=1.0).contains(&t) {
        let hit_point = vec3_add(center, vec3_scale(direction, t));
        CapsuleSweepResult::new_hit(t, [0.0, 1.0, 0.0], [hit_point[0], plane_y, hit_point[2]])
    } else {
        CapsuleSweepResult::miss()
    }
}

/// Sweep a sphere along a direction and test against another static sphere.
#[allow(dead_code)]
pub fn sphere_sweep_sphere(
    center_a: [f32; 3],
    radius_a: f32,
    direction: [f32; 3],
    center_b: [f32; 3],
    radius_b: f32,
) -> CapsuleSweepResult {
    let combined = radius_a + radius_b;
    let d = vec3_sub(center_a, center_b);
    let a_coeff = vec3_dot(direction, direction);
    let b_coeff = 2.0 * vec3_dot(d, direction);
    let c_coeff = vec3_dot(d, d) - combined * combined;
    let disc = b_coeff * b_coeff - 4.0 * a_coeff * c_coeff;
    if disc < 0.0 {
        return CapsuleSweepResult::miss();
    }
    let sqrt_disc = disc.sqrt();
    let t = (-b_coeff - sqrt_disc) / (2.0 * a_coeff);
    if (0.0..=1.0).contains(&t) {
        let hit_center = vec3_add(center_a, vec3_scale(direction, t));
        let normal = vec3_normalize(vec3_sub(hit_center, center_b));
        let point = vec3_add(center_b, vec3_scale(normal, radius_b));
        CapsuleSweepResult::new_hit(t, normal, point)
    } else {
        CapsuleSweepResult::miss()
    }
}

/// Compute the closest distance between two line segments (for capsule-capsule).
#[allow(dead_code)]
pub fn segment_distance_sq(
    a0: [f32; 3], a1: [f32; 3],
    b0: [f32; 3], b1: [f32; 3],
) -> f32 {
    let d1 = vec3_sub(a1, a0);
    let d2 = vec3_sub(b1, b0);
    let r = vec3_sub(a0, b0);
    let a = vec3_dot(d1, d1);
    let e = vec3_dot(d2, d2);
    let f = vec3_dot(d2, r);
    if a < 1e-10 && e < 1e-10 {
        return vec3_dot(r, r);
    }
    let b = vec3_dot(d1, d2);
    let c = vec3_dot(d1, r);
    let denom = a * e - b * b;
    let s = if denom > 1e-10 {
        ((b * f - c * e) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let t = ((b * s + f) / e).clamp(0.0, 1.0);
    let closest_a = vec3_add(a0, vec3_scale(d1, s));
    let closest_b = vec3_add(b0, vec3_scale(d2, t));
    let diff = vec3_sub(closest_a, closest_b);
    vec3_dot(diff, diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miss() {
        let r = CapsuleSweepResult::miss();
        assert!(!r.hit);
    }

    #[test]
    fn test_sphere_sweep_plane_hit() {
        let r = sphere_sweep_plane([0.0, 2.0, 0.0], 0.5, [0.0, -3.0, 0.0], 0.0);
        assert!(r.hit);
        assert!(r.t > 0.0 && r.t <= 1.0);
    }

    #[test]
    fn test_sphere_sweep_plane_miss() {
        let r = sphere_sweep_plane([0.0, 2.0, 0.0], 0.5, [0.0, 1.0, 0.0], 0.0);
        assert!(!r.hit);
    }

    #[test]
    fn test_sphere_sweep_plane_already_penetrating() {
        let r = sphere_sweep_plane([0.0, 0.3, 0.0], 0.5, [0.0, -1.0, 0.0], 0.0);
        assert!(r.hit);
        assert!((r.t).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sphere_sweep_sphere_hit() {
        let r = sphere_sweep_sphere(
            [-5.0, 0.0, 0.0], 1.0,
            [10.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], 1.0,
        );
        assert!(r.hit);
    }

    #[test]
    fn test_sphere_sweep_sphere_miss() {
        let r = sphere_sweep_sphere(
            [-5.0, 0.0, 0.0], 0.5,
            [0.0, 10.0, 0.0],
            [5.0, 0.0, 0.0], 0.5,
        );
        assert!(!r.hit);
    }

    #[test]
    fn test_segment_distance_same_point() {
        let d = segment_distance_sq([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_segment_distance_parallel() {
        let d = segment_distance_sq(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
        );
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_new_hit() {
        let r = CapsuleSweepResult::new_hit(0.5, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(r.hit);
        assert!((r.t - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vec3_normalize() {
        let n = vec3_normalize([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-6);
    }
}
