// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Clip a mesh against a half-space plane, keeping the positive side and
//! generating a flat cap on the cut face.
//!
//! Unlike `mesh_slice` (which only extracts cross-sections), this module:
//! 1. Retains all geometry on the positive (keep) side of the plane.
//! 2. Interpolates new vertices along cut edges (Sutherland–Hodgman style).
//! 3. Collects the boundary loop produced by the cut, sorts vertices by angle
//!    around the loop centroid, and fan-triangulates a flat cap.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ─────────────────────────────────────────────────────────────────────────────
// Math helpers
// ─────────────────────────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(a: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] * t, a[1] * t, a[2] * t]
}

#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[inline]
fn lerp2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

#[inline]
fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[inline]
fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// ClipPlane
// ─────────────────────────────────────────────────────────────────────────────

/// A half-space plane used for clipping.
///
/// Vertices with `signed_distance >= 0` are on the "keep" side; vertices with
/// a negative signed distance are discarded.
#[derive(Debug, Clone)]
pub struct ClipPlane {
    /// A point that lies on the plane.
    pub origin: [f32; 3],
    /// Unit normal pointing toward the side to *keep*.
    pub normal: [f32; 3],
}

impl ClipPlane {
    /// Construct a new `ClipPlane`.
    pub fn new(origin: [f32; 3], normal: [f32; 3]) -> Self {
        Self { origin, normal }
    }

    /// Signed distance from the plane to `point`.
    ///
    /// `dot(point - origin, normal)` — positive means "above" (keep side).
    #[inline]
    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        dot3(self.normal, sub3(point, self.origin))
    }

    /// Returns `true` if `point` is on the keep side (signed distance ≥ 0).
    #[inline]
    pub fn above(&self, point: [f32; 3]) -> bool {
        self.signed_distance(point) >= 0.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ClipConfig
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration controlling clipping behaviour.
#[derive(Debug, Clone)]
pub struct ClipConfig {
    /// When `true` (default) a flat cap is triangulated over the cut boundary.
    pub fill_cap: bool,
    /// When `true` (default) vertex normals are recomputed after clipping.
    pub smooth_normals: bool,
    /// When `false` (default) the cap normal follows the clip-plane normal.
    /// When `true` the cap normal is flipped inward.
    pub cap_normal_inward: bool,
}

impl Default for ClipConfig {
    fn default() -> Self {
        Self {
            fill_cap: true,
            smooth_normals: true,
            cap_normal_inward: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ClipResult
// ─────────────────────────────────────────────────────────────────────────────

/// The output of a clip operation.
#[derive(Debug, Clone)]
pub struct ClipResult {
    /// The clipped mesh, optionally with the cap included.
    pub mesh: MeshBuffers,
    /// Index into `mesh.positions` where cap vertices begin.
    pub cap_vertex_start: usize,
    /// Index (in units of triangles) into `mesh.indices / 3` where cap faces begin.
    pub cap_face_start: usize,
    /// Number of original faces that were fully on the keep side.
    pub kept_face_count: usize,
    /// Number of original faces that straddled the plane and were cut.
    pub cut_face_count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// A temporary vertex used during clipping before the final buffers are built.
#[derive(Clone, Copy)]
struct TmpVert {
    pos: [f32; 3],
    nrm: [f32; 3],
    uv: [f32; 2],
}

/// Compute the parameter `t` such that `lerp(a, b, t)` lies exactly on the plane.
#[inline]
fn edge_t(d_a: f32, d_b: f32) -> f32 {
    let denom = d_a - d_b;
    if denom.abs() < 1e-12 {
        0.5
    } else {
        d_a / denom
    }
}

/// Append a `TmpVert` to the output buffers and return its index.
#[inline]
fn push_vert(
    v: TmpVert,
    out_pos: &mut Vec<[f32; 3]>,
    out_nrm: &mut Vec<[f32; 3]>,
    out_uv: &mut Vec<[f32; 2]>,
) -> u32 {
    let idx = out_pos.len() as u32;
    out_pos.push(v.pos);
    out_nrm.push(v.nrm);
    out_uv.push(v.uv);
    idx
}

// ─────────────────────────────────────────────────────────────────────────────
// Core clipping function
// ─────────────────────────────────────────────────────────────────────────────

/// Clip `mesh` against `plane`, returning a new mesh that contains only the
/// geometry on the keep side.
///
/// If `config.fill_cap` is `true`, a flat cap is added over the cut boundary.
pub fn clip_mesh(mesh: &MeshBuffers, plane: &ClipPlane, config: &ClipConfig) -> ClipResult {
    let positions = &mesh.positions;
    let normals = &mesh.normals;
    let uvs = &mesh.uvs;
    let indices = &mesh.indices;
    let n_verts = positions.len();

    // Pre-compute signed distances for every vertex.
    let signed_dist: Vec<f32> = positions
        .iter()
        .map(|p| plane.signed_distance(*p))
        .collect();

    // Output accumulators.
    let mut out_pos: Vec<[f32; 3]> = Vec::new();
    let mut out_nrm: Vec<[f32; 3]> = Vec::new();
    let mut out_uv: Vec<[f32; 2]> = Vec::new();
    let mut out_idx: Vec<u32> = Vec::new();

    // Cap boundary accumulator: new cut-edge positions for later cap construction.
    let mut cap_boundary: Vec<[f32; 3]> = Vec::new();

    let mut kept_face_count = 0usize;
    let mut cut_face_count = 0usize;

    // Safe accessors.
    let safe_vert = |i: u32| -> TmpVert {
        let k = i as usize;
        TmpVert {
            pos: if k < n_verts { positions[k] } else { [0.0; 3] },
            nrm: if k < normals.len() {
                normals[k]
            } else {
                [0.0, 1.0, 0.0]
            },
            uv: if k < uvs.len() { uvs[k] } else { [0.0; 2] },
        }
    };
    let safe_dist = |i: u32| -> f32 {
        let k = i as usize;
        if k < signed_dist.len() {
            signed_dist[k]
        } else {
            0.0
        }
    };

    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0], tri[1], tri[2]);
        let d0 = safe_dist(i0);
        let d1 = safe_dist(i1);
        let d2 = safe_dist(i2);

        let above0 = d0 >= 0.0;
        let above1 = d1 >= 0.0;
        let above2 = d2 >= 0.0;
        let above_count = above0 as u8 + above1 as u8 + above2 as u8;

        if above_count == 3 {
            // All above: emit as-is.
            let a = push_vert(safe_vert(i0), &mut out_pos, &mut out_nrm, &mut out_uv);
            let b = push_vert(safe_vert(i1), &mut out_pos, &mut out_nrm, &mut out_uv);
            let c = push_vert(safe_vert(i2), &mut out_pos, &mut out_nrm, &mut out_uv);
            out_idx.extend_from_slice(&[a, b, c]);
            kept_face_count += 1;
        } else if above_count == 0 {
            // All below: discard.
        } else {
            // Mixed triangle: Sutherland–Hodgman clip against the half-space.
            cut_face_count += 1;

            let verts = [
                (safe_vert(i0), d0),
                (safe_vert(i1), d1),
                (safe_vert(i2), d2),
            ];
            let flags = [above0, above1, above2];

            let mut clipped: Vec<TmpVert> = Vec::with_capacity(4);

            for k in 0..3 {
                let (va, da) = verts[k];
                let (vb, db) = verts[(k + 1) % 3];
                let fa = flags[k];
                let fb = flags[(k + 1) % 3];

                if fa {
                    clipped.push(va);
                }
                if fa != fb {
                    // Edge crosses the plane — interpolate a new vertex.
                    let t = edge_t(da, db);
                    let p_cut = lerp3(va.pos, vb.pos, t);
                    let cut_vert = TmpVert {
                        pos: p_cut,
                        nrm: normalize3(lerp3(va.nrm, vb.nrm, t)),
                        uv: lerp2(va.uv, vb.uv, t),
                    };
                    clipped.push(cut_vert);
                    cap_boundary.push(p_cut);
                }
            }

            // Fan-triangulate the clipped polygon (3 or 4 vertices).
            if clipped.len() >= 3 {
                let anchor = push_vert(clipped[0], &mut out_pos, &mut out_nrm, &mut out_uv);
                for vi in 1..clipped.len() - 1 {
                    let bv = push_vert(clipped[vi], &mut out_pos, &mut out_nrm, &mut out_uv);
                    let cv = push_vert(clipped[vi + 1], &mut out_pos, &mut out_nrm, &mut out_uv);
                    out_idx.extend_from_slice(&[anchor, bv, cv]);
                }
            }
        }
    }

    let cap_vertex_start = out_pos.len();
    let cap_face_start = out_idx.len() / 3;

    // Build cap if requested and there is a boundary.
    if config.fill_cap && cap_boundary.len() >= 3 {
        let deduped = deduplicate_points(&cap_boundary, 1e-5);

        if deduped.len() >= 3 {
            let centroid = centroid3(&deduped);
            let cap_n = if config.cap_normal_inward {
                scale3(plane.normal, -1.0)
            } else {
                plane.normal
            };
            let sorted = sort_by_angle_around_centroid(&deduped, centroid, cap_n);

            let center_vert = TmpVert {
                pos: centroid,
                nrm: cap_n,
                uv: [0.5, 0.5],
            };
            let center_idx = push_vert(center_vert, &mut out_pos, &mut out_nrm, &mut out_uv);

            let n_sorted = sorted.len();
            for k in 0..n_sorted {
                let p_a = sorted[k];
                let p_b = sorted[(k + 1) % n_sorted];
                let va = TmpVert {
                    pos: p_a,
                    nrm: cap_n,
                    uv: cap_uv(p_a, centroid, cap_n),
                };
                let vb = TmpVert {
                    pos: p_b,
                    nrm: cap_n,
                    uv: cap_uv(p_b, centroid, cap_n),
                };
                let a = push_vert(va, &mut out_pos, &mut out_nrm, &mut out_uv);
                let b = push_vert(vb, &mut out_pos, &mut out_nrm, &mut out_uv);
                out_idx.extend_from_slice(&[center_idx, a, b]);
            }
        }
    }

    let n = out_pos.len();
    let mut result_mesh = MeshBuffers {
        positions: out_pos,
        normals: out_nrm,
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
        uvs: out_uv,
        indices: out_idx,
        colors: None,
        has_suit: mesh.has_suit,
    };

    if config.smooth_normals {
        compute_normals(&mut result_mesh);
    }

    ClipResult {
        mesh: result_mesh,
        cap_vertex_start,
        cap_face_start,
        kept_face_count,
        cut_face_count,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers for cap construction
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the centroid of a point set.
fn centroid3(pts: &[[f32; 3]]) -> [f32; 3] {
    let n = pts.len() as f32;
    let mut s = [0.0f32; 3];
    for p in pts {
        s = add3(s, *p);
    }
    scale3(s, 1.0 / n)
}

/// Remove near-duplicate points within `eps`.
fn deduplicate_points(pts: &[[f32; 3]], eps: f32) -> Vec<[f32; 3]> {
    let mut out: Vec<[f32; 3]> = Vec::new();
    'outer: for p in pts {
        for q in &out {
            if len3(sub3(*p, *q)) < eps {
                continue 'outer;
            }
        }
        out.push(*p);
    }
    out
}

/// Build two tangent axes perpendicular to `normal` for 2-D angular sorting.
fn plane_axes(normal: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let ref_vec: [f32; 3] = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let ax = normalize3(cross3(ref_vec, normal));
    let ay = normalize3(cross3(normal, ax));
    (ax, ay)
}

/// Sort `pts` by angle around `centroid` within the plane whose normal is `normal`.
fn sort_by_angle_around_centroid(
    pts: &[[f32; 3]],
    centroid: [f32; 3],
    normal: [f32; 3],
) -> Vec<[f32; 3]> {
    let (ax, ay) = plane_axes(normal);
    let mut with_angle: Vec<(f32, [f32; 3])> = pts
        .iter()
        .map(|&p| {
            let d = sub3(p, centroid);
            let u = dot3(d, ax);
            let v = dot3(d, ay);
            let angle = v.atan2(u);
            (angle, p)
        })
        .collect();
    with_angle.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    with_angle.into_iter().map(|(_, p)| p).collect()
}

/// Approximate UV coordinates for a cap vertex by planar projection.
fn cap_uv(p: [f32; 3], centroid: [f32; 3], normal: [f32; 3]) -> [f32; 2] {
    let (ax, ay) = plane_axes(normal);
    let d = sub3(p, centroid);
    let u = dot3(d, ax);
    let v = dot3(d, ay);
    [u * 0.5 + 0.5, v * 0.5 + 0.5]
}

// ─────────────────────────────────────────────────────────────────────────────
// Convenience functions
// ─────────────────────────────────────────────────────────────────────────────

/// Clip `mesh` against all 6 planes of an axis-aligned bounding box.
///
/// Each successive clip uses the result of the previous one.
pub fn clip_to_box(mesh: &MeshBuffers, min: [f32; 3], max: [f32; 3]) -> MeshBuffers {
    let config = ClipConfig::default();
    let planes = [
        ClipPlane::new([min[0], 0.0, 0.0], [1.0, 0.0, 0.0]),
        ClipPlane::new([max[0], 0.0, 0.0], [-1.0, 0.0, 0.0]),
        ClipPlane::new([0.0, min[1], 0.0], [0.0, 1.0, 0.0]),
        ClipPlane::new([0.0, max[1], 0.0], [0.0, -1.0, 0.0]),
        ClipPlane::new([0.0, 0.0, min[2]], [0.0, 0.0, 1.0]),
        ClipPlane::new([0.0, 0.0, max[2]], [0.0, 0.0, -1.0]),
    ];

    let mut current = mesh.clone();
    for plane in &planes {
        let result = clip_mesh(&current, plane, &config);
        current = result.mesh;
        if current.indices.is_empty() {
            break;
        }
    }
    current
}

/// Clip `mesh` keeping only the region where `y >= y_val`.
pub fn clip_above_y(mesh: &MeshBuffers, y_val: f32) -> ClipResult {
    let plane = ClipPlane::new([0.0, y_val, 0.0], [0.0, 1.0, 0.0]);
    clip_mesh(mesh, &plane, &ClipConfig::default())
}

/// Clip `mesh` keeping only the region where `y <= y_val`.
pub fn clip_below_y(mesh: &MeshBuffers, y_val: f32) -> ClipResult {
    let plane = ClipPlane::new([0.0, y_val, 0.0], [0.0, -1.0, 0.0]);
    clip_mesh(mesh, &plane, &ClipConfig::default())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::sphere;

    // ── Helper meshes ────────────────────────────────────────────────────────

    /// Triangulated box (12 triangles).
    fn make_box(min: [f32; 3], max: [f32; 3]) -> MeshBuffers {
        let [x0, y0, z0] = min;
        let [x1, y1, z1] = max;
        let positions = vec![
            [x0, y0, z0],
            [x1, y0, z0],
            [x1, y1, z0],
            [x0, y1, z0],
            [x0, y0, z1],
            [x1, y0, z1],
            [x1, y1, z1],
            [x0, y1, z1],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 2, 6, 7, 2, 7, 3, 0, 3, 7, 0, 7,
            4, 1, 5, 6, 1, 6, 2,
        ];
        let n = positions.len();
        let mut m = MeshBuffers {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            colors: None,
            has_suit: false,
        };
        compute_normals(&mut m);
        m
    }

    /// A single triangle flat on the XZ plane (y = 0).
    fn single_triangle() -> MeshBuffers {
        let n = 3;
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.0, 1.0]],
            normals: vec![[0.0, 1.0, 0.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    // ── ClipPlane tests ──────────────────────────────────────────────────────

    #[test]
    fn clip_plane_signed_distance_origin() {
        let p = ClipPlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((p.signed_distance([0.0, 5.0, 0.0]) - 5.0).abs() < 1e-6);
        assert!((p.signed_distance([0.0, -3.0, 0.0]) + 3.0).abs() < 1e-6);
        assert!((p.signed_distance([0.0, 0.0, 0.0])).abs() < 1e-6);
    }

    #[test]
    fn clip_plane_signed_distance_offset_origin() {
        let p = ClipPlane::new([0.0, 2.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((p.signed_distance([0.0, 2.0, 0.0])).abs() < 1e-6);
        assert!((p.signed_distance([0.0, 5.0, 0.0]) - 3.0).abs() < 1e-6);
        assert!((p.signed_distance([0.0, 0.0, 0.0]) + 2.0).abs() < 1e-6);
    }

    #[test]
    fn clip_plane_above_on_plane() {
        let p = ClipPlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(p.above([0.0, 0.0, 0.0])); // exactly on plane → above
    }

    #[test]
    fn clip_plane_above_positive_side() {
        let p = ClipPlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(p.above([0.0, 1.0, 0.0]));
        assert!(!p.above([0.0, -0.001, 0.0]));
    }

    #[test]
    fn clip_plane_diagonal_normal() {
        let n = normalize3([1.0, 1.0, 0.0]);
        let p = ClipPlane::new([0.0, 0.0, 0.0], n);
        assert!(p.signed_distance([1.0, 1.0, 0.0]) > 0.0);
        assert!(p.signed_distance([-1.0, -1.0, 0.0]) < 0.0);
    }

    // ── All-above / all-below ────────────────────────────────────────────────

    #[test]
    fn all_above_keeps_all_faces() {
        let mesh = make_box([-1.0, 0.0, -1.0], [1.0, 1.0, 1.0]);
        let original_faces = mesh.face_count();
        let result = clip_above_y(&mesh, -5.0);
        assert_eq!(result.cut_face_count, 0);
        assert!(result.mesh.face_count() >= original_faces);
    }

    #[test]
    fn all_below_discards_all_geometry() {
        let mesh = make_box([-1.0, 0.0, -1.0], [1.0, 1.0, 1.0]);
        let result = clip_above_y(&mesh, 5.0);
        assert_eq!(result.mesh.face_count(), 0);
        assert_eq!(result.mesh.positions.len(), 0);
    }

    // ── clip_above_y ─────────────────────────────────────────────────────────

    #[test]
    fn clip_above_y_half_box_all_verts_above() {
        let mesh = make_box([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let result = clip_above_y(&mesh, 0.0);
        assert!(result.mesh.face_count() > 0);
        for p in &result.mesh.positions {
            assert!(p[1] >= -1e-5, "vertex below clip plane: y={}", p[1]);
        }
    }

    #[test]
    fn clip_above_y_cap_vertices_exist() {
        let mesh = make_box([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let result = clip_above_y(&mesh, 0.0);
        assert!(result.cap_vertex_start > 0);
        assert!(result.mesh.positions.len() > result.cap_vertex_start);
    }

    #[test]
    fn clip_above_y_sphere_no_negative_y() {
        let mesh = sphere(1.0, 8, 8);
        let result = clip_above_y(&mesh, 0.0);
        for p in &result.mesh.positions {
            assert!(p[1] >= -1e-4, "vertex below y=0: y={}", p[1]);
        }
    }

    #[test]
    fn clip_above_y_sphere_has_cap_faces() {
        let mesh = sphere(1.0, 8, 8);
        let result = clip_above_y(&mesh, 0.0);
        let total_faces = result.mesh.face_count();
        assert!(
            result.cap_face_start < total_faces,
            "no cap faces generated"
        );
    }

    // ── clip_below_y ─────────────────────────────────────────────────────────

    #[test]
    fn clip_below_y_keeps_lower_half() {
        let mesh = make_box([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let result = clip_below_y(&mesh, 0.0);
        for p in &result.mesh.positions {
            assert!(p[1] <= 1e-5, "vertex above clip plane: y={}", p[1]);
        }
    }

    #[test]
    fn clip_below_y_has_faces() {
        let mesh = make_box([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let result = clip_below_y(&mesh, 0.0);
        assert!(result.mesh.face_count() > 0);
    }

    // ── clip_to_box ──────────────────────────────────────────────────────────

    #[test]
    fn clip_to_box_restricts_positions() {
        let mesh = sphere(2.0, 6, 6);
        let clipped = clip_to_box(&mesh, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        for p in &clipped.positions {
            assert!(p[0] >= -1.01 && p[0] <= 1.01, "x={} out of box", p[0]);
            assert!(p[1] >= -1.01 && p[1] <= 1.01, "y={} out of box", p[1]);
            assert!(p[2] >= -1.01 && p[2] <= 1.01, "z={} out of box", p[2]);
        }
    }

    #[test]
    fn clip_to_box_produces_valid_indices() {
        let mesh = sphere(1.0, 6, 6);
        let clipped = clip_to_box(&mesh, [-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
        let n_verts = clipped.positions.len() as u32;
        for &idx in &clipped.indices {
            assert!(idx < n_verts, "index {} out of range (n={})", idx, n_verts);
        }
    }

    // ── fill_cap = false ─────────────────────────────────────────────────────

    #[test]
    fn fill_cap_false_no_cap_vertices() {
        let mesh = make_box([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let plane = ClipPlane::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let config = ClipConfig {
            fill_cap: false,
            smooth_normals: false,
            cap_normal_inward: false,
        };
        let result = clip_mesh(&mesh, &plane, &config);
        assert_eq!(result.cap_vertex_start, result.mesh.positions.len());
        assert_eq!(result.cap_face_start, result.mesh.face_count());
    }

    // ── single-triangle edge cases ────────────────────────────────────────────

    #[test]
    fn clip_single_triangle_entirely_above() {
        let mesh = single_triangle();
        let result = clip_above_y(&mesh, -1.0);
        assert_eq!(result.kept_face_count, 1);
        assert_eq!(result.cut_face_count, 0);
    }

    #[test]
    fn clip_single_triangle_entirely_below() {
        let mesh = single_triangle();
        let result = clip_above_y(&mesh, 1.0);
        assert_eq!(result.kept_face_count, 0);
        assert_eq!(result.mesh.positions.len(), 0);
    }

    #[test]
    fn cut_face_count_nonzero_for_intersecting_plane() {
        let mesh = make_box([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let result = clip_above_y(&mesh, 0.0);
        assert!(result.cut_face_count > 0, "expected some faces to be cut");
    }

    #[test]
    fn clip_config_default_fill_cap_true() {
        let cfg = ClipConfig::default();
        assert!(cfg.fill_cap);
        assert!(cfg.smooth_normals);
        assert!(!cfg.cap_normal_inward);
    }
}
