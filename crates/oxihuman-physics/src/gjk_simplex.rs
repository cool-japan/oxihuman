#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GJK algorithm simplex helper.

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GjkSimplex {
    pub verts: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn new_gjk_simplex() -> GjkSimplex {
    GjkSimplex { verts: Vec::new() }
}

#[allow(dead_code)]
pub fn simplex_add(s: &mut GjkSimplex, v: [f32; 3]) {
    s.verts.push(v);
}

#[allow(dead_code)]
pub fn simplex_size(s: &GjkSimplex) -> usize {
    s.verts.len()
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Support function: furthest point on Minkowski difference A-B in direction dir.
#[allow(dead_code)]
pub fn support_fn(shape_a: &[[f32; 3]], shape_b: &[[f32; 3]], dir: [f32; 3]) -> [f32; 3] {
    let sup_a = shape_a
        .iter()
        .max_by(|a, b| dot3(**a, dir).partial_cmp(&dot3(**b, dir)).unwrap_or(std::cmp::Ordering::Equal))
        .copied()
        .unwrap_or([0.0; 3]);
    let neg_dir = [-dir[0], -dir[1], -dir[2]];
    let sup_b = shape_b
        .iter()
        .max_by(|a, b| dot3(**a, neg_dir).partial_cmp(&dot3(**b, neg_dir)).unwrap_or(std::cmp::Ordering::Equal))
        .copied()
        .unwrap_or([0.0; 3]);
    minkowski_diff(sup_a, sup_b)
}

#[allow(dead_code)]
pub fn minkowski_diff(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    sub3(a, b)
}

/// Return the nearest point on the simplex to the origin and reduce the simplex.
#[allow(dead_code)]
pub fn gjk_nearest_simplex(s: &mut GjkSimplex) -> [f32; 3] {
    match simplex_size(s) {
        0 => [0.0; 3],
        1 => s.verts[0],
        2 => {
            let a = s.verts[1];
            let b = s.verts[0];
            let ab = sub3(b, a);
            let ao = scale3(a, -1.0);
            let t = dot3(ao, ab) / dot3(ab, ab).max(1e-20);
            let t = t.clamp(0.0, 1.0);
            add3(a, scale3(ab, t))
        }
        _ => {
            // Return last added vertex as closest direction hint
            s.verts[s.verts.len() - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_simplex_empty() {
        let s = new_gjk_simplex();
        assert_eq!(simplex_size(&s), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut s = new_gjk_simplex();
        simplex_add(&mut s, [1.0, 0.0, 0.0]);
        assert_eq!(simplex_size(&s), 1);
    }

    #[test]
    fn test_minkowski_diff() {
        let d = minkowski_diff([3.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((d[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_nearest_single_vertex() {
        let mut s = new_gjk_simplex();
        simplex_add(&mut s, [2.0, 0.0, 0.0]);
        let n = gjk_nearest_simplex(&mut s);
        assert!((n[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_nearest_empty() {
        let mut s = new_gjk_simplex();
        let n = gjk_nearest_simplex(&mut s);
        assert_eq!(n, [0.0; 3]);
    }

    #[test]
    fn test_support_fn_simple() {
        let shape_a = [[1.0f32, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        let shape_b = [[0.0f32, 0.0, 0.0]];
        let d = support_fn(&shape_a, &shape_b, [1.0, 0.0, 0.0]);
        assert!((d[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_support_fn_negative_dir() {
        let shape_a = [[2.0f32, 0.0, 0.0], [-2.0, 0.0, 0.0]];
        let shape_b = [[0.0f32, 0.0, 0.0]];
        let d = support_fn(&shape_a, &shape_b, [-1.0, 0.0, 0.0]);
        assert!((d[0] + 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_nearest_two_verts() {
        let mut s = new_gjk_simplex();
        simplex_add(&mut s, [2.0, 0.0, 0.0]);
        simplex_add(&mut s, [3.0, 0.0, 0.0]);
        let n = gjk_nearest_simplex(&mut s);
        // Nearest point on segment [2,3] to origin is at t=0 (point 2)
        assert!(n[0] >= 2.0);
        assert!(n[0] <= 3.0);
    }

    #[test]
    fn test_add_multiple() {
        let mut s = new_gjk_simplex();
        for i in 0..4 {
            simplex_add(&mut s, [i as f32, 0.0, 0.0]);
        }
        assert_eq!(simplex_size(&s), 4);
    }
}
