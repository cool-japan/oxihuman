// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ruled surface between two curves.

/// Result of a ruled surface construction.
#[derive(Debug, Clone)]
pub struct RuledSurface {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub u_count: usize,
    pub v_count: usize,
}

/// Interpolate linearly between two 3-D points.
pub fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Build a ruled surface between curve `a` and curve `b`.
/// Both curves must have the same number of sample points.
/// `v_divs` is the number of divisions along the ruling direction.
pub fn build_ruled_surface(
    curve_a: &[[f32; 3]],
    curve_b: &[[f32; 3]],
    v_divs: usize,
) -> RuledSurface {
    let u = curve_a.len();
    if u < 2 || curve_b.len() != u || v_divs < 1 {
        return RuledSurface {
            verts: vec![],
            tris: vec![],
            u_count: 0,
            v_count: 0,
        };
    }
    let v = v_divs + 1;
    let mut verts = Vec::with_capacity(u * v);
    for i in 0..u {
        for j in 0..v {
            let t = j as f32 / v_divs as f32;
            verts.push(lerp3(curve_a[i], curve_b[i], t));
        }
    }
    let mut tris = Vec::new();
    for i in 0..(u - 1) {
        for j in 0..(v - 1) {
            let a = (i * v + j) as u32;
            let b = (i * v + j + 1) as u32;
            let c = ((i + 1) * v + j) as u32;
            let d = ((i + 1) * v + j + 1) as u32;
            tris.push([a, c, b]);
            tris.push([b, c, d]);
        }
    }
    RuledSurface {
        verts,
        tris,
        u_count: u,
        v_count: v,
    }
}

/// Return the vertex count of a ruled surface.
pub fn ruled_vertex_count(surf: &RuledSurface) -> usize {
    surf.verts.len()
}

/// Return the triangle count of a ruled surface.
pub fn ruled_tri_count(surf: &RuledSurface) -> usize {
    surf.tris.len()
}

/// Validate that a ruled surface has no degenerate triangles (all indices in range).
pub fn validate_ruled_surface(surf: &RuledSurface) -> bool {
    let n = surf.verts.len() as u32;
    surf.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_a(n: usize) -> Vec<[f32; 3]> {
        (0..n).map(|i| [i as f32, 0.0, 0.0]).collect()
    }
    fn line_b(n: usize) -> Vec<[f32; 3]> {
        (0..n).map(|i| [i as f32, 0.0, 1.0]).collect()
    }

    #[test]
    fn test_ruled_vertex_count() {
        /* two lines of 5 points, 3 v-divs → 5*4 = 20 verts */
        let s = build_ruled_surface(&line_a(5), &line_b(5), 3);
        assert_eq!(ruled_vertex_count(&s), 20);
    }

    #[test]
    fn test_ruled_tri_count() {
        /* (5-1)*(3) quads, 2 tris each → 24 tris */
        let s = build_ruled_surface(&line_a(5), &line_b(5), 3);
        assert_eq!(ruled_tri_count(&s), 24);
    }

    #[test]
    fn test_ruled_empty_on_mismatch() {
        let s = build_ruled_surface(&line_a(3), &line_b(5), 2);
        assert_eq!(ruled_vertex_count(&s), 0);
    }

    #[test]
    fn test_ruled_empty_on_zero_divs() {
        let s = build_ruled_surface(&line_a(4), &line_b(4), 0);
        assert_eq!(ruled_vertex_count(&s), 0);
    }

    #[test]
    fn test_lerp3_midpoint() {
        let a = [0.0, 0.0, 0.0];
        let b = [2.0, 4.0, 6.0];
        let m = lerp3(a, b, 0.5);
        assert!((m[0] - 1.0).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
        assert!((m[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_validate_ruled_surface() {
        let s = build_ruled_surface(&line_a(4), &line_b(4), 2);
        assert!(validate_ruled_surface(&s));
    }

    #[test]
    fn test_ruled_u_v_counts() {
        let s = build_ruled_surface(&line_a(3), &line_b(3), 4);
        assert_eq!(s.u_count, 3);
        assert_eq!(s.v_count, 5);
    }

    #[test]
    fn test_lerp3_endpoints() {
        let a = [1.0, 2.0, 3.0];
        let b = [7.0, 8.0, 9.0];
        let at_zero = lerp3(a, b, 0.0);
        let at_one = lerp3(a, b, 1.0);
        assert_eq!(at_zero, a);
        assert_eq!(at_one, b);
    }

    #[test]
    fn test_ruled_single_v_div() {
        /* v_divs=1 gives 2 rows: 4*2=8 verts, (3)*(1)*2=6 tris */
        let s = build_ruled_surface(&line_a(4), &line_b(4), 1);
        assert_eq!(ruled_vertex_count(&s), 8);
        assert_eq!(ruled_tri_count(&s), 6);
    }
}
