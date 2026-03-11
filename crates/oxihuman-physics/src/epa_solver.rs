#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Expanding Polytope Algorithm stub (penetration depth after GJK).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EpaResult {
    pub depth: f32,
    pub normal: [f32; 3],
    pub point: [f32; 3],
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v).max(1e-10);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Simple EPA stub: returns the closest face to origin from the polytope.
#[allow(dead_code)]
pub fn epa_stub(
    _shape_a: &[[f32; 3]],
    _shape_b: &[[f32; 3]],
    simplex: &[[f32; 3]],
) -> EpaResult {
    if simplex.len() < 3 {
        return epa_no_collision();
    }
    let (_, depth, normal) = epa_closest_face(simplex);
    EpaResult {
        depth,
        normal,
        point: [0.0; 3],
    }
}

/// Returns (face_index, depth, normal) for the face closest to origin.
#[allow(dead_code)]
pub fn epa_closest_face(polytope: &[[f32; 3]]) -> (usize, f32, [f32; 3]) {
    let n = polytope.len();
    if n < 3 {
        return (0, 0.0, [0.0, 1.0, 0.0]);
    }
    let mut best_idx = 0;
    let mut best_dist = f32::INFINITY;
    let mut best_normal = [0.0f32, 1.0, 0.0];

    // Treat each triple of consecutive vertices as a triangle face.
    let faces = n / 3;
    for i in 0..faces {
        let a = polytope[i * 3];
        let b = polytope[i * 3 + 1];
        let c = polytope[i * 3 + 2];
        let ab = sub3(b, a);
        let ac = sub3(c, a);
        let n_raw = cross3(ab, ac);
        let normal = normalize3(n_raw);
        let dist = dot3(normal, a).abs();
        if dist < best_dist {
            best_dist = dist;
            best_normal = normal;
            best_idx = i;
        }
    }
    (best_idx, best_dist, best_normal)
}

#[allow(dead_code)]
pub fn epa_no_collision() -> EpaResult {
    EpaResult {
        depth: 0.0,
        normal: [0.0, 1.0, 0.0],
        point: [0.0; 3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_collision_depth_zero() {
        let r = epa_no_collision();
        assert!(r.depth.abs() < 1e-6);
    }

    #[test]
    fn test_no_collision_normal() {
        let r = epa_no_collision();
        assert!((r.normal[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_epa_stub_empty_simplex() {
        let r = epa_stub(&[], &[], &[]);
        assert!(r.depth.abs() < 1e-6);
    }

    #[test]
    fn test_epa_stub_small_simplex() {
        let simplex = [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = epa_stub(&[], &[], &simplex);
        assert!(r.depth.abs() < 1e-6);
    }

    #[test]
    fn test_epa_closest_face_basic() {
        let polytope = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let (idx, depth, _normal) = epa_closest_face(&polytope);
        assert_eq!(idx, 0);
        assert!(depth >= 0.0);
    }

    #[test]
    fn test_epa_closest_face_depth_nonneg() {
        let polytope = [
            [1.0f32, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let (_, depth, _) = epa_closest_face(&polytope);
        assert!(depth >= 0.0);
    }

    #[test]
    fn test_epa_stub_triangle_simplex() {
        let simplex = [
            [1.0f32, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let r = epa_stub(&[], &[], &simplex);
        assert!(r.depth >= 0.0);
    }

    #[test]
    fn test_epa_result_clone() {
        let r = epa_no_collision();
        let r2 = r.clone();
        assert!((r2.depth - r.depth).abs() < 1e-6);
    }

    #[test]
    fn test_epa_closest_face_less_than_three() {
        let polytope = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let (_, depth, _) = epa_closest_face(&polytope);
        assert_eq!(depth, 0.0);
    }
}
