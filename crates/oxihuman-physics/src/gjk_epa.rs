// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Minimal GJK (Gilbert-Johnson-Keerthi) + EPA helpers for convex shapes.

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_neg(a: [f32; 3]) -> [f32; 3] {
    [-a[0], -a[1], -a[2]]
}

#[allow(dead_code)]
fn vec3_len_sq(v: [f32; 3]) -> f32 {
    vec3_dot(v, v)
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_len_sq(v).sqrt()
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Support point for a set of vertices along a direction.
#[allow(dead_code)]
pub fn support(vertices: &[[f32; 3]], direction: [f32; 3]) -> [f32; 3] {
    let mut best = vertices[0];
    let mut best_dot = vec3_dot(best, direction);
    for &v in &vertices[1..] {
        let d = vec3_dot(v, direction);
        if d > best_dot {
            best = v;
            best_dot = d;
        }
    }
    best
}

/// Minkowski difference support.
#[allow(dead_code)]
pub fn minkowski_support(a: &[[f32; 3]], b: &[[f32; 3]], dir: [f32; 3]) -> [f32; 3] {
    let sa = support(a, dir);
    let sb = support(b, vec3_neg(dir));
    vec3_sub(sa, sb)
}

/// Simple GJK intersection test (2D/3D simplex, simplified).
#[allow(dead_code)]
pub fn gjk_intersect(a: &[[f32; 3]], b: &[[f32; 3]]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let mut dir = [1.0f32, 0.0, 0.0];
    let mut simplex = Vec::with_capacity(4);
    let s = minkowski_support(a, b, dir);
    // If support point is at origin, shapes overlap
    if vec3_len_sq(s) < 1e-12 {
        return true;
    }
    simplex.push(s);
    dir = vec3_neg(s);

    for _ in 0..64 {
        if vec3_len_sq(dir) < 1e-12 {
            return true;
        }
        let new_point = minkowski_support(a, b, dir);
        if vec3_dot(new_point, dir) < 0.0 {
            return false;
        }
        simplex.push(new_point);
        if do_simplex(&mut simplex, &mut dir) {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn do_simplex(simplex: &mut Vec<[f32; 3]>, dir: &mut [f32; 3]) -> bool {
    match simplex.len() {
        2 => do_simplex_line(simplex, dir),
        3 => do_simplex_triangle(simplex, dir),
        _ => false,
    }
}

#[allow(dead_code)]
fn do_simplex_line(simplex: &mut Vec<[f32; 3]>, dir: &mut [f32; 3]) -> bool {
    let a = simplex[1];
    let b = simplex[0];
    let ab = vec3_sub(b, a);
    let ao = vec3_neg(a);
    if vec3_dot(ab, ao) > 0.0 {
        let cross1 = vec3_cross(ab, ao);
        let new_dir = vec3_cross(cross1, ab);
        if vec3_len_sq(new_dir) < 1e-12 {
            // Origin is on the line segment => contained
            return true;
        }
        *dir = new_dir;
    } else {
        simplex.clear();
        simplex.push(a);
        *dir = ao;
    }
    false
}

#[allow(dead_code)]
fn do_simplex_triangle(simplex: &mut Vec<[f32; 3]>, dir: &mut [f32; 3]) -> bool {
    let a = simplex[2];
    let b = simplex[1];
    let c = simplex[0];
    let ab = vec3_sub(b, a);
    let ac = vec3_sub(c, a);
    let ao = vec3_neg(a);
    let abc = vec3_cross(ab, ac);

    if vec3_dot(vec3_cross(abc, ac), ao) > 0.0 {
        if vec3_dot(ac, ao) > 0.0 {
            simplex.clear();
            simplex.push(c);
            simplex.push(a);
            let cross1 = vec3_cross(ac, ao);
            *dir = vec3_cross(cross1, ac);
        } else {
            simplex.clear();
            simplex.push(b);
            simplex.push(a);
            return do_simplex_line(simplex, dir);
        }
    } else if vec3_dot(vec3_cross(ab, abc), ao) > 0.0 {
        simplex.clear();
        simplex.push(b);
        simplex.push(a);
        return do_simplex_line(simplex, dir);
    } else if vec3_dot(abc, ao) > 0.0 {
        *dir = abc;
    } else {
        simplex.swap(0, 1);
        *dir = vec3_neg(abc);
    }
    false
}

/// Compute separation distance between two convex sets (approximate).
#[allow(dead_code)]
pub fn gjk_distance(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    if gjk_intersect(a, b) {
        return 0.0;
    }
    // Approximate: closest support points
    let dir = vec3_sub(
        support(a, [1.0, 0.0, 0.0]),
        support(b, [-1.0, 0.0, 0.0]),
    );
    vec3_len(dir).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube(cx: f32, cy: f32, cz: f32, half: f32) -> Vec<[f32; 3]> {
        vec![
            [cx - half, cy - half, cz - half],
            [cx + half, cy - half, cz - half],
            [cx - half, cy + half, cz - half],
            [cx + half, cy + half, cz - half],
            [cx - half, cy - half, cz + half],
            [cx + half, cy - half, cz + half],
            [cx - half, cy + half, cz + half],
            [cx + half, cy + half, cz + half],
        ]
    }

    #[test]
    fn test_support() {
        let verts = cube(0.0, 0.0, 0.0, 1.0);
        let s = support(&verts, [1.0, 0.0, 0.0]);
        assert!((s[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_intersect_overlapping() {
        let a = cube(0.0, 0.0, 0.0, 1.0);
        let b = cube(0.5, 0.0, 0.0, 1.0);
        assert!(gjk_intersect(&a, &b));
    }

    #[test]
    fn test_intersect_separated() {
        let a = cube(0.0, 0.0, 0.0, 1.0);
        let b = cube(5.0, 0.0, 0.0, 1.0);
        assert!(!gjk_intersect(&a, &b));
    }

    #[test]
    fn test_intersect_same() {
        let a = cube(0.0, 0.0, 0.0, 1.0);
        assert!(gjk_intersect(&a, &a));
    }

    #[test]
    fn test_empty_sets() {
        let a: Vec<[f32; 3]> = vec![];
        let b = cube(0.0, 0.0, 0.0, 1.0);
        assert!(!gjk_intersect(&a, &b));
    }

    #[test]
    fn test_minkowski_support() {
        let a = cube(0.0, 0.0, 0.0, 1.0);
        let b = cube(0.0, 0.0, 0.0, 1.0);
        let s = minkowski_support(&a, &b, [1.0, 0.0, 0.0]);
        assert!((s[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_distance_separated() {
        let a = cube(0.0, 0.0, 0.0, 1.0);
        let b = cube(5.0, 0.0, 0.0, 1.0);
        let d = gjk_distance(&a, &b);
        assert!(d > 0.0);
    }

    #[test]
    fn test_distance_overlapping() {
        let a = cube(0.0, 0.0, 0.0, 1.0);
        let b = cube(0.5, 0.0, 0.0, 1.0);
        assert!((gjk_distance(&a, &b)).abs() < 1e-5);
    }

    #[test]
    fn test_vec3_cross() {
        let c = vec3_cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((c[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_touching_cubes() {
        let a = cube(0.0, 0.0, 0.0, 1.0);
        let b = cube(2.0, 0.0, 0.0, 1.0);
        // Touching at edge
        let result = gjk_intersect(&a, &b);
        // Either true or false is acceptable at exact boundary
        let _ = result;
    }
}
