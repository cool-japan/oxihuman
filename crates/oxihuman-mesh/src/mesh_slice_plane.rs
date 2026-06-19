// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Slice mesh with an arbitrary plane stub.

/// A plane defined by a point and normal.
#[derive(Debug, Clone, Copy)]
pub struct SlicePlane {
    pub origin: [f32; 3],
    pub normal: [f32; 3],
}

impl SlicePlane {
    pub fn new(origin: [f32; 3], normal: [f32; 3]) -> Self {
        let n = normalize3(normal);
        Self { origin, normal: n }
    }

    /// Signed distance from a point to this plane.
    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        let d = [
            point[0] - self.origin[0],
            point[1] - self.origin[1],
            point[2] - self.origin[2],
        ];
        dot3(d, self.normal)
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Result of slicing: two sub-meshes (above and below the plane).
#[derive(Debug, Clone, Default)]
pub struct SliceResult {
    pub above_verts: Vec<[f32; 3]>,
    pub above_tris: Vec<[u32; 3]>,
    pub below_verts: Vec<[f32; 3]>,
    pub below_tris: Vec<[u32; 3]>,
    pub cross_section_verts: Vec<[f32; 3]>,
}

/// Classify vertices of a mesh as above or below a plane.
pub fn classify_vertices(verts: &[[f32; 3]], plane: &SlicePlane) -> Vec<f32> {
    verts.iter().map(|&v| plane.signed_distance(v)).collect()
}

/// Minimum triangle area threshold — triangles below this area (after clipping)
/// are discarded as degenerate.
const MIN_TRIANGLE_AREA: f32 = 1e-10;

/// Compute the area of a triangle defined by three positions.
#[inline]
fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

/// Push a triangle into the accumulating flat-indexed buffers, skipping
/// degenerate triangles whose area is below `MIN_TRIANGLE_AREA`.
#[inline]
fn push_tri(
    verts_out: &mut Vec<[f32; 3]>,
    tris_out: &mut Vec<[u32; 3]>,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) {
    if triangle_area(a, b, c) < MIN_TRIANGLE_AREA {
        return;
    }
    let base = verts_out.len() as u32;
    verts_out.push(a);
    verts_out.push(b);
    verts_out.push(c);
    tris_out.push([base, base + 1, base + 2]);
}

/// Slice a mesh with a plane using exact per-triangle clipping.
///
/// Each triangle is classified by the signed distance of its three vertices:
///
/// * **All 3 on same side** — triangle placed entirely in `above` or `below`.
/// * **1 vertex alone on one side** — the triangle is split: the lone vertex
///   forms one small triangle on its side and the opposite two vertices form a
///   quad (two triangles) on the other side.
/// * **2 vertices alone on one side** — mirror of the above case.
///
/// Vertices that lie exactly on the plane (`|d| < EPS`) are treated as "above"
/// for the purpose of whole-triangle routing.
pub fn slice_mesh_with_plane(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    plane: &SlicePlane,
) -> SliceResult {
    const EPS: f32 = 1e-7;

    let mut above_verts: Vec<[f32; 3]> = Vec::new();
    let mut above_tris: Vec<[u32; 3]> = Vec::new();
    let mut below_verts: Vec<[f32; 3]> = Vec::new();
    let mut below_tris: Vec<[u32; 3]> = Vec::new();
    let mut cross_section_verts: Vec<[f32; 3]> = Vec::new();

    for tri in tris {
        let [ia, ib, ic] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ia >= verts.len() || ib >= verts.len() || ic >= verts.len() {
            continue;
        }
        let pa = verts[ia];
        let pb = verts[ib];
        let pc = verts[ic];

        let da = plane.signed_distance(pa);
        let db = plane.signed_distance(pb);
        let dc = plane.signed_distance(pc);

        // Classify: true = above (including on-plane), false = below
        let sa = da >= -EPS;
        let sb = db >= -EPS;
        let sc = dc >= -EPS;

        let above_count = sa as u8 + sb as u8 + sc as u8;

        match above_count {
            3 => {
                // All vertices above (or on) the plane
                push_tri(&mut above_verts, &mut above_tris, pa, pb, pc);
            }
            0 => {
                // All vertices below the plane
                push_tri(&mut below_verts, &mut below_tris, pa, pb, pc);
            }
            1 => {
                // Exactly one vertex is above; identify it and the two below.
                // The lone above-vertex is (p_lone); the two below are (p1, p2).
                let (p_lone, d_lone, p1, d1, p2, d2) = if sa {
                    (pa, da, pb, db, pc, dc)
                } else if sb {
                    (pb, db, pa, da, pc, dc)
                } else {
                    (pc, dc, pa, da, pb, db)
                };
                let q1 = edge_plane_intersect(p_lone, p1, d_lone, d1);
                let q2 = edge_plane_intersect(p_lone, p2, d_lone, d2);
                cross_section_verts.push(q1);
                cross_section_verts.push(q2);
                // Lone vertex side (above)
                push_tri(&mut above_verts, &mut above_tris, p_lone, q1, q2);
                // Two-vertex side (below): quad [p1, p2, q1] and [p2, q2, q1]
                push_tri(&mut below_verts, &mut below_tris, p1, p2, q1);
                push_tri(&mut below_verts, &mut below_tris, p2, q2, q1);
            }
            2 => {
                // Exactly two vertices are above; identify them and the lone below.
                let (p_lone, d_lone, p1, d1, p2, d2) = if !sa {
                    (pa, da, pb, db, pc, dc)
                } else if !sb {
                    (pb, db, pa, da, pc, dc)
                } else {
                    (pc, dc, pa, da, pb, db)
                };
                let q1 = edge_plane_intersect(p_lone, p1, d_lone, d1);
                let q2 = edge_plane_intersect(p_lone, p2, d_lone, d2);
                cross_section_verts.push(q1);
                cross_section_verts.push(q2);
                // Lone vertex side (below)
                push_tri(&mut below_verts, &mut below_tris, p_lone, q1, q2);
                // Two-vertex side (above): quad [p1, p2, q1] and [p2, q2, q1]
                push_tri(&mut above_verts, &mut above_tris, p1, p2, q1);
                push_tri(&mut above_verts, &mut above_tris, p2, q2, q1);
            }
            _ => unreachable!(),
        }
    }

    SliceResult {
        above_verts,
        above_tris,
        below_verts,
        below_tris,
        cross_section_verts,
    }
}

/// Count triangles on each side of the plane.
pub fn count_triangles_per_side(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    plane: &SlicePlane,
) -> (usize, usize) {
    /* Returns (above_count, below_count) */
    let mut above = 0usize;
    let mut below = 0usize;
    for tri in tris {
        let mut sum = 0.0f32;
        for &idx in tri.iter() {
            let vi = idx as usize;
            if vi < verts.len() {
                sum += plane.signed_distance(verts[vi]);
            }
        }
        if sum >= 0.0 {
            above += 1;
        } else {
            below += 1;
        }
    }
    (above, below)
}

/// Interpolate a point on the edge between two vertices at the plane crossing.
pub fn edge_plane_intersect(a: [f32; 3], b: [f32; 3], da: f32, db: f32) -> [f32; 3] {
    /* Linear interpolation to find where the edge crosses the plane */
    let denom = da - db;
    if denom.abs() < 1e-12 {
        return a;
    }
    let t = da / denom;
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_distance_above() {
        let p = SlicePlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(p.signed_distance([0.0, 1.0, 0.0]) > 0.0 /* above plane */);
    }

    #[test]
    fn test_plane_distance_below() {
        let p = SlicePlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(p.signed_distance([0.0, -1.0, 0.0]) < 0.0 /* below plane */);
    }

    #[test]
    fn test_plane_distance_on() {
        let p = SlicePlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(p.signed_distance([1.0, 0.0, 0.0]).abs() < 1e-6 /* on plane */);
    }

    #[test]
    fn test_classify_vertices_signs() {
        let p = SlicePlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let verts = vec![[0.0f32, 1.0, 0.0], [0.0, -1.0, 0.0]];
        let dists = classify_vertices(&verts, &p);
        assert!(dists[0] > 0.0 /* first above */);
        assert!(dists[1] < 0.0 /* second below */);
    }

    #[test]
    fn test_slice_separates_triangles() {
        let p = SlicePlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let verts = vec![
            [0.0f32, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 2.0, 0.0], /* above */
            [0.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [0.0, -2.0, 0.0], /* below */
        ];
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let result = slice_mesh_with_plane(&verts, &tris, &p);
        assert!(!result.above_tris.is_empty() /* some triangles above */);
        assert!(!result.below_tris.is_empty() /* some triangles below */);
    }

    #[test]
    fn test_count_per_side() {
        let p = SlicePlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let verts = vec![
            [0.0f32, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [0.0, -2.0, 0.0],
        ];
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let (a, b) = count_triangles_per_side(&verts, &tris, &p);
        assert_eq!(a + b, 2 /* total unchanged */);
    }

    #[test]
    fn test_edge_intersect_midpoint() {
        let a = [0.0f32, -1.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        let pt = edge_plane_intersect(a, b, -1.0, 1.0);
        assert!(pt[1].abs() < 1e-5 /* intersection at y=0 */);
    }

    #[test]
    fn test_slice_empty_mesh() {
        let p = SlicePlane::new([0.0; 3], [0.0, 1.0, 0.0]);
        let result = slice_mesh_with_plane(&[], &[], &p);
        assert!(result.above_tris.is_empty() /* nothing above */);
        assert!(result.below_tris.is_empty() /* nothing below */);
    }

    #[test]
    fn test_normalize_zero_vec() {
        let n = normalize3([0.0, 0.0, 0.0]);
        assert_eq!(n, [0.0, 1.0, 0.0] /* fallback normal */);
    }

    /// Build a simple axis-aligned unit cube centred at the origin:
    /// 8 vertices, 12 triangles (2 per face × 6 faces).
    fn unit_cube() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![
            [-0.5f32, -0.5, -0.5], // 0
            [0.5, -0.5, -0.5],     // 1
            [0.5, 0.5, -0.5],      // 2
            [-0.5, 0.5, -0.5],     // 3
            [-0.5, -0.5, 0.5],     // 4
            [0.5, -0.5, 0.5],      // 5
            [0.5, 0.5, 0.5],       // 6
            [-0.5, 0.5, 0.5],      // 7
        ];
        let tris = vec![
            // -Z face
            [0u32, 2, 1],
            [0, 3, 2],
            // +Z face
            [4, 5, 6],
            [4, 6, 7],
            // -X face
            [0, 4, 7],
            [0, 7, 3],
            // +X face
            [1, 2, 6],
            [1, 6, 5],
            // -Y face
            [0, 1, 5],
            [0, 5, 4],
            // +Y face
            [3, 7, 6],
            [3, 6, 2],
        ];
        (verts, tris)
    }

    #[test]
    fn slice_cube_midplane() {
        let (verts, tris) = unit_cube();
        let plane = SlicePlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let result = slice_mesh_with_plane(&verts, &tris, &plane);
        assert!(
            !result.above_tris.is_empty(),
            "cube slice must produce above-triangles"
        );
        assert!(
            !result.below_tris.is_empty(),
            "cube slice must produce below-triangles"
        );
        for &v in &result.cross_section_verts {
            assert!(
                v[1].abs() < 1e-4,
                "cross-section vertex y must be ≈0, got {}",
                v[1]
            );
        }
    }

    #[test]
    fn slice_plane_cuts_straddling_triangle() {
        // A single triangle that straddles y=0:
        //   a below, b and c above
        let verts = vec![
            [0.0f32, -1.0, 0.0], // 0  below
            [1.0, 1.0, 0.0],     // 1  above
            [-1.0, 1.0, 0.0],    // 2  above
        ];
        let tris = vec![[0u32, 1, 2]];
        let plane = SlicePlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let result = slice_mesh_with_plane(&verts, &tris, &plane);
        assert!(
            !result.below_tris.is_empty(),
            "straddling triangle must produce below triangles"
        );
        assert!(
            !result.above_tris.is_empty(),
            "straddling triangle must produce above triangles"
        );
        assert_eq!(
            result.cross_section_verts.len(),
            2,
            "exactly 2 cross-section vertices for a single straddling triangle"
        );
        for &v in &result.cross_section_verts {
            assert!(
                v[1].abs() < 1e-4,
                "cross-section vertex y must be ≈0, got {}",
                v[1]
            );
        }
    }
}
