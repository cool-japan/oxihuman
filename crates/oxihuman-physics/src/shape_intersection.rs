#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Exact shape intersection tests (beyond AABB).

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_len_sq(v: [f32; 3]) -> f32 {
    vec3_dot(v, v)
}

fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
pub fn sphere_sphere_intersect(ca: [f32; 3], ra: f32, cb: [f32; 3], rb: f32) -> bool {
    let d_sq = vec3_len_sq(vec3_sub(ca, cb));
    let r_sum = ra + rb;
    d_sq <= r_sum * r_sum
}

#[allow(dead_code)]
pub fn sphere_plane_intersect(c: [f32; 3], r: f32, plane_n: [f32; 3], plane_d: f32) -> bool {
    let dist = vec3_dot(c, plane_n) - plane_d;
    dist.abs() <= r
}

/// Returns the closest approach distance between two line segments.
#[allow(dead_code)]
pub fn capsule_capsule_closest(a0: [f32; 3], a1: [f32; 3], b0: [f32; 3], b1: [f32; 3]) -> f32 {
    segment_segment_dist(a0, a1, b0, b1)
}

/// Returns the minimum distance between two line segments.
#[allow(dead_code)]
pub fn segment_segment_dist(a0: [f32; 3], a1: [f32; 3], b0: [f32; 3], b1: [f32; 3]) -> f32 {
    let da = vec3_sub(a1, a0);
    let db = vec3_sub(b1, b0);
    let r = vec3_sub(a0, b0);

    let a = vec3_dot(da, da);
    let e = vec3_dot(db, db);
    let f = vec3_dot(db, r);

    let (s, t) = if a < 1e-10 && e < 1e-10 {
        (0.0f32, 0.0f32)
    } else if a < 1e-10 {
        (0.0f32, (f / e).clamp(0.0, 1.0))
    } else {
        let c = vec3_dot(da, r);
        if e < 1e-10 {
            ((-c / a).clamp(0.0, 1.0), 0.0f32)
        } else {
            let b_val = vec3_dot(da, db);
            let denom = a * e - b_val * b_val;
            let s = if denom.abs() > 1e-10 {
                ((b_val * f - c * e) / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let t = (b_val * s + f) / e;
            if t < 0.0 {
                ((-c / a).clamp(0.0, 1.0), 0.0f32)
            } else if t > 1.0 {
                (((b_val - c) / a).clamp(0.0, 1.0), 1.0f32)
            } else {
                (s, t)
            }
        }
    };

    let pa = vec3_add(a0, vec3_scale(da, s));
    let pb = vec3_add(b0, vec3_scale(db, t));
    vec3_len_sq(vec3_sub(pa, pb)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spheres_touching() {
        assert!(sphere_sphere_intersect(
            [0.0, 0.0, 0.0],
            1.0,
            [2.0, 0.0, 0.0],
            1.0
        ));
    }

    #[test]
    fn spheres_overlapping() {
        assert!(sphere_sphere_intersect(
            [0.0, 0.0, 0.0],
            1.0,
            [1.0, 0.0, 0.0],
            1.0
        ));
    }

    #[test]
    fn spheres_separated() {
        assert!(!sphere_sphere_intersect(
            [0.0, 0.0, 0.0],
            0.5,
            [3.0, 0.0, 0.0],
            0.5
        ));
    }

    #[test]
    fn sphere_plane_inside() {
        // Sphere at (0,1,0) radius 2, plane y=0 (normal [0,1,0], d=0)
        assert!(sphere_plane_intersect(
            [0.0, 1.0, 0.0],
            2.0,
            [0.0, 1.0, 0.0],
            0.0
        ));
    }

    #[test]
    fn sphere_plane_outside() {
        // Sphere at (0,5,0) radius 1, plane y=0
        assert!(!sphere_plane_intersect(
            [0.0, 5.0, 0.0],
            1.0,
            [0.0, 1.0, 0.0],
            0.0
        ));
    }

    #[test]
    fn segment_segment_same_point() {
        let d = segment_segment_dist(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        );
        assert!((d - 1.0).abs() < 1e-4);
    }

    #[test]
    fn parallel_segments_dist() {
        let d = segment_segment_dist(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        );
        assert!((d - 1.0).abs() < 1e-4);
    }

    #[test]
    fn crossing_segments_near_zero() {
        // Two segments crossing at origin but offset in z
        let d = segment_segment_dist(
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, -1.0, 0.5],
            [0.0, 1.0, 0.5],
        );
        assert!((d - 0.5).abs() < 1e-4);
    }

    #[test]
    fn capsule_capsule_delegates() {
        let d1 =
            capsule_capsule_closest([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]);
        let d2 = segment_segment_dist([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]);
        assert!((d1 - d2).abs() < 1e-6);
    }

    #[test]
    fn sphere_plane_on_surface() {
        // Sphere center exactly at plane distance r from plane
        assert!(sphere_plane_intersect(
            [0.0, 1.0, 0.0],
            1.0,
            [0.0, 1.0, 0.0],
            0.0
        ));
    }
}
