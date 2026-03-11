// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
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

/// Slice a mesh with a plane (stub: partitions triangles by centroid side).
pub fn slice_mesh_with_plane(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    plane: &SlicePlane,
) -> SliceResult {
    /* Stub: classify each triangle by its centroid signed distance */
    let mut above_verts = Vec::new();
    let mut above_tris = Vec::new();
    let mut below_verts = Vec::new();
    let mut below_tris = Vec::new();
    let mut cross_verts = Vec::new();

    for tri in tris {
        let mut centroid = [0.0f32; 3];
        for idx in tri.iter() {
            let vi = *idx as usize;
            if vi < verts.len() {
                for k in 0..3 {
                    centroid[k] += verts[vi][k];
                }
            }
        }
        centroid.iter_mut().for_each(|x| *x /= 3.0);
        let dist = plane.signed_distance(centroid);

        let offset_a = above_verts.len() as u32;
        let offset_b = below_verts.len() as u32;

        if dist >= 0.0 {
            for &idx in tri.iter() {
                let vi = idx as usize;
                if vi < verts.len() {
                    above_verts.push(verts[vi]);
                }
            }
            above_tris.push([offset_a, offset_a + 1, offset_a + 2]);
        } else {
            for &idx in tri.iter() {
                let vi = idx as usize;
                if vi < verts.len() {
                    below_verts.push(verts[vi]);
                }
            }
            below_tris.push([offset_b, offset_b + 1, offset_b + 2]);
        }

        /* Record cross-section vertex at centroid when dist is near zero */
        if dist.abs() < 0.05 {
            cross_verts.push(centroid);
        }
    }

    SliceResult {
        above_verts,
        above_tris,
        below_verts,
        below_tris,
        cross_section_verts: cross_verts,
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
}
