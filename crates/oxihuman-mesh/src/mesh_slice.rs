// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh slicing with a plane — cross-section extraction, mesh splitting,
//! body-measurement circumference, and width-profile utilities.
//!
//! Algorithm overview for `slice_mesh`:
//! 1. For each triangle, classify all 3 verts by signed distance to the plane.
//! 2. If any triangle straddles the plane (verts on both sides), compute
//!    edge-plane intersections to get a crossing segment.
//! 3. Collect all crossing segments, then chain them into ordered closed loops.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;

type InterpVertFn = dyn Fn(
    [f32; 3],
    [f32; 3],
    [f32; 3],
    [f32; 3],
    [f32; 2],
    [f32; 2],
    f32,
    f32,
) -> ([f32; 3], [f32; 3], [f32; 2]);

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

#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// SlicePlane
// ─────────────────────────────────────────────────────────────────────────────

/// A plane defined by a point (origin) and a unit normal.
#[derive(Debug, Clone)]
pub struct SlicePlane {
    /// A point on the plane.
    pub origin: [f32; 3],
    /// Unit normal (must be normalised by the caller for correct results).
    pub normal: [f32; 3],
}

impl SlicePlane {
    /// Construct a new `SlicePlane`.
    pub fn new(origin: [f32; 3], normal: [f32; 3]) -> Self {
        Self { origin, normal }
    }

    /// Signed distance from the plane to `point`.
    /// Positive = on the normal side ("above"), negative = opposite side ("below").
    #[inline]
    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        dot3(self.normal, sub3(point, self.origin))
    }

    /// z = 0 plane (XY plane), normal pointing +Z.
    pub fn xy() -> Self {
        Self::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0])
    }

    /// y = 0 plane (XZ plane), normal pointing +Y.
    pub fn xz() -> Self {
        Self::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    }

    /// x = 0 plane (YZ plane), normal pointing +X.
    pub fn yz() -> Self {
        Self::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0])
    }

    /// Horizontal plane at a given Y height, normal pointing +Y.
    pub fn at_height(y: f32) -> Self {
        Self::new([0.0, y, 0.0], [0.0, 1.0, 0.0])
    }

    /// Compute two tangent vectors orthogonal to `self.normal`, forming a
    /// right-handed local coordinate frame (tangent, bitangent, normal).
    pub fn tangent_frame(&self) -> ([f32; 3], [f32; 3]) {
        let n = self.normal;
        // Pick an up vector that is not (nearly) parallel to n
        let up = if n[0].abs() < 0.9 {
            [1.0f32, 0.0, 0.0]
        } else {
            [0.0f32, 1.0, 0.0]
        };
        let tangent = normalize3(cross3(up, n));
        let bitangent = cross3(n, tangent);
        (tangent, bitangent)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CrossSection
// ─────────────────────────────────────────────────────────────────────────────

/// A 2-D cross-section polygon produced by intersecting a mesh with a plane.
#[derive(Debug, Clone)]
pub struct CrossSection {
    /// Ordered 3-D points on the plane.
    pub points: Vec<[f32; 3]>,
    /// `true` if the polygon forms a closed loop (first point == last point
    /// conceptually, i.e. edge from last back to first exists).
    pub closed: bool,
}

impl CrossSection {
    /// Number of distinct vertices in the section.
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// Perimeter — sum of edge lengths.
    /// If `closed`, includes the wrap-around edge from last to first.
    pub fn perimeter(&self) -> f32 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0f32;
        let n = self.points.len();
        for i in 0..n - 1 {
            total += len3(sub3(self.points[i + 1], self.points[i]));
        }
        if self.closed {
            total += len3(sub3(self.points[0], self.points[n - 1]));
        }
        total
    }

    /// Project the 3-D points onto the plane's local 2-D frame.
    pub fn to_2d(&self, plane: &SlicePlane) -> Vec<[f32; 2]> {
        let (tan, bitan) = plane.tangent_frame();
        self.points
            .iter()
            .map(|&p| {
                let rel = sub3(p, plane.origin);
                [dot3(rel, tan), dot3(rel, bitan)]
            })
            .collect()
    }

    /// Approximate enclosed area using the shoelace formula on the 2-D projection.
    /// Returns `0.0` for open sections or fewer than 3 points.
    pub fn area(&self, plane: &SlicePlane) -> f32 {
        if !self.closed || self.points.len() < 3 {
            return 0.0;
        }
        let pts2d = self.to_2d(plane);
        let n = pts2d.len();
        let mut sum = 0.0f32;
        for i in 0..n {
            let j = (i + 1) % n;
            sum += pts2d[i][0] * pts2d[j][1];
            sum -= pts2d[j][0] * pts2d[i][1];
        }
        sum.abs() * 0.5
    }

    /// Centroid of the section points (arithmetic mean).
    pub fn centroid(&self) -> [f32; 3] {
        if self.points.is_empty() {
            return [0.0, 0.0, 0.0];
        }
        let n = self.points.len() as f32;
        let mut acc = [0.0f32; 3];
        for p in &self.points {
            acc[0] += p[0];
            acc[1] += p[1];
            acc[2] += p[2];
        }
        [acc[0] / n, acc[1] / n, acc[2] / n]
    }

    /// Axis-aligned bounding box of the section in 3-D space.
    /// Returns `([f32::MAX; 3], [f32::MIN; 3])` if there are no points.
    pub fn bbox_3d(&self) -> ([f32; 3], [f32; 3]) {
        if self.points.is_empty() {
            return ([f32::MAX; 3], [f32::MIN_POSITIVE; 3]);
        }
        let mut mn = self.points[0];
        let mut mx = self.points[0];
        for p in &self.points {
            for i in 0..3 {
                if p[i] < mn[i] {
                    mn[i] = p[i];
                }
                if p[i] > mx[i] {
                    mx[i] = p[i];
                }
            }
        }
        (mn, mx)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SliceResult
// ─────────────────────────────────────────────────────────────────────────────

/// The full result of slicing a mesh with a plane.
#[derive(Debug, Clone)]
pub struct SliceResult {
    /// All cross-section loops found.
    pub sections: Vec<CrossSection>,
    /// Number of triangles entirely above (or on) the plane.
    pub triangle_count_above: usize,
    /// Number of triangles entirely below the plane.
    pub triangle_count_below: usize,
    /// Number of raw intersection edges before loop-chaining.
    pub intersection_edge_count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Free functions — intersection helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Ray-plane intersection.
///
/// Returns `None` if the ray is parallel to (or lies within) the plane.
/// Otherwise returns the 3-D intersection point.
pub fn ray_plane_intersect(
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    plane: &SlicePlane,
) -> Option<[f32; 3]> {
    let denom = dot3(plane.normal, ray_dir);
    if denom.abs() < 1e-12 {
        return None; // parallel
    }
    let t = dot3(plane.normal, sub3(plane.origin, ray_origin)) / denom;
    Some(add3(ray_origin, scale3(ray_dir, t)))
}

/// Edge-plane intersection.
///
/// Given two vertices presumed to be on **opposite** sides of the plane,
/// returns the interpolated intersection point.
pub fn edge_plane_intersect(v0: [f32; 3], v1: [f32; 3], plane: &SlicePlane) -> [f32; 3] {
    let d0 = plane.signed_distance(v0);
    let d1 = plane.signed_distance(v1);
    let denom = d0 - d1;
    if denom.abs() < 1e-30 {
        // Degenerate — return midpoint
        return lerp3(v0, v1, 0.5);
    }
    let t = d0 / denom;
    lerp3(v0, v1, t)
}

// ─────────────────────────────────────────────────────────────────────────────
// slice_mesh
// ─────────────────────────────────────────────────────────────────────────────

/// Slice a mesh with a plane and return cross-section loop(s).
///
/// For each triangle that straddles the plane exactly two edge-plane intersection
/// points are computed, giving one "crossing segment".  All segments are then
/// chained (by matching endpoints) into ordered closed loops.
pub fn slice_mesh(mesh: &MeshBuffers, plane: &SlicePlane) -> SliceResult {
    let positions = &mesh.positions;
    let indices = &mesh.indices;
    let n_tris = indices.len() / 3;

    // epsilon for "on the plane"
    const EPS: f32 = 1e-6;

    let mut segments: Vec<([f32; 3], [f32; 3])> = Vec::new();
    let mut count_above = 0usize;
    let mut count_below = 0usize;

    for tri in 0..n_tris {
        let i0 = indices[tri * 3] as usize;
        let i1 = indices[tri * 3 + 1] as usize;
        let i2 = indices[tri * 3 + 2] as usize;

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];

        let d0 = plane.signed_distance(p0);
        let d1 = plane.signed_distance(p1);
        let d2 = plane.signed_distance(p2);

        let above0 = d0 >= -EPS;
        let above1 = d1 >= -EPS;
        let above2 = d2 >= -EPS;

        if above0 && above1 && above2 {
            count_above += 1;
            continue;
        }
        if !above0 && !above1 && !above2 {
            count_below += 1;
            continue;
        }

        // Triangle straddles — collect intersection points for each crossing edge
        let verts = [p0, p1, p2];
        let dists = [d0, d1, d2];
        let mut pts: Vec<[f32; 3]> = Vec::with_capacity(2);

        for e in 0..3 {
            let a = e;
            let b = (e + 1) % 3;
            let da = dists[a];
            let db = dists[b];
            // Crosses if one is above EPS and the other below -EPS
            if (da > EPS && db < -EPS) || (da < -EPS && db > EPS) {
                pts.push(edge_plane_intersect(verts[a], verts[b], plane));
            } else if da.abs() <= EPS {
                // vertex exactly on plane
                pts.push(verts[a]);
            }
        }

        // Deduplicate nearly-identical points
        pts.dedup_by(|a, b| len3(sub3(*a, *b)) < EPS);

        if pts.len() >= 2 {
            segments.push((pts[0], pts[1]));
        }
    }

    let intersection_edge_count = segments.len();

    // Chain segments into ordered loops
    let sections = chain_segments(segments);

    SliceResult {
        sections,
        triangle_count_above: count_above,
        triangle_count_below: count_below,
        intersection_edge_count,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Segment chaining
// ─────────────────────────────────────────────────────────────────────────────

fn pts_near(a: [f32; 3], b: [f32; 3]) -> bool {
    len3(sub3(a, b)) < 1e-4
}

/// Given a list of unordered line segments, chain them into ordered loops.
fn chain_segments(mut segments: Vec<([f32; 3], [f32; 3])>) -> Vec<CrossSection> {
    let mut result: Vec<CrossSection> = Vec::new();

    while !segments.is_empty() {
        // Start a new chain with the first remaining segment
        let first = segments.remove(0);
        let mut chain: Vec<[f32; 3]> = vec![first.0, first.1];

        // Try to extend the chain by finding a segment whose start/end matches
        // the current chain tip
        loop {
            let tip = chain[chain.len() - 1];
            let head = chain[0];

            // Check if chain closes
            if chain.len() > 2 && pts_near(tip, head) {
                chain.pop(); // remove duplicated closing point
                break;
            }

            let mut found = false;
            for i in 0..segments.len() {
                let (s, e) = segments[i];
                if pts_near(tip, s) {
                    chain.push(e);
                    segments.remove(i);
                    found = true;
                    break;
                } else if pts_near(tip, e) {
                    chain.push(s);
                    segments.remove(i);
                    found = true;
                    break;
                }
            }
            if !found {
                break;
            }
        }

        // Determine if the chain is closed
        let closed = chain.len() >= 3 && pts_near(chain[chain.len() - 1], chain[0]);

        // If closed, remove the redundant final point that equals first
        let mut points = chain;
        if closed && points.len() >= 2 && pts_near(points[points.len() - 1], points[0]) {
            points.pop();
        }

        result.push(CrossSection { points, closed });
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// split_mesh
// ─────────────────────────────────────────────────────────────────────────────

/// Split a mesh into two halves at `plane`.
///
/// Returns `(above_mesh, below_mesh)`.  Triangles entirely on one side are kept
/// whole; triangles that straddle the plane are clipped and the resulting
/// sub-triangles are added to the appropriate half.
pub fn split_mesh(mesh: &MeshBuffers, plane: &SlicePlane) -> (MeshBuffers, MeshBuffers) {
    const EPS: f32 = 1e-6;

    let mut above_pos: Vec<[f32; 3]> = Vec::new();
    let mut above_nrm: Vec<[f32; 3]> = Vec::new();
    let mut above_uv: Vec<[f32; 2]> = Vec::new();
    let mut above_idx: Vec<u32> = Vec::new();

    let mut below_pos: Vec<[f32; 3]> = Vec::new();
    let mut below_nrm: Vec<[f32; 3]> = Vec::new();
    let mut below_uv: Vec<[f32; 2]> = Vec::new();
    let mut below_idx: Vec<u32> = Vec::new();

    let positions = &mesh.positions;
    let normals = &mesh.normals;
    let uvs = &mesh.uvs;
    let indices = &mesh.indices;
    let n_tris = indices.len() / 3;

    // Helper: push a triangle to a buffer set
    let push_tri = |pos: &mut Vec<[f32; 3]>,
                    nrm: &mut Vec<[f32; 3]>,
                    uv: &mut Vec<[f32; 2]>,
                    idx: &mut Vec<u32>,
                    verts: &[[f32; 3]; 3],
                    vnorms: &[[f32; 3]; 3],
                    vuvs: &[[f32; 2]; 3]| {
        let base = pos.len() as u32;
        for k in 0..3 {
            pos.push(verts[k]);
            nrm.push(vnorms[k]);
            uv.push(vuvs[k]);
        }
        idx.push(base);
        idx.push(base + 1);
        idx.push(base + 2);
    };

    // Interpolate between two vertices by signed distance
    let interp_v = |p0: [f32; 3],
                    p1: [f32; 3],
                    n0: [f32; 3],
                    n1: [f32; 3],
                    u0: [f32; 2],
                    u1: [f32; 2],
                    d0: f32,
                    d1: f32|
     -> ([f32; 3], [f32; 3], [f32; 2]) {
        let denom = d0 - d1;
        let t = if denom.abs() < 1e-30 { 0.5 } else { d0 / denom };
        let pos = lerp3(p0, p1, t);
        let nrm = normalize3(lerp3(n0, n1, t));
        let uv = [u0[0] + (u1[0] - u0[0]) * t, u0[1] + (u1[1] - u0[1]) * t];
        (pos, nrm, uv)
    };

    for tri in 0..n_tris {
        let i = [
            indices[tri * 3] as usize,
            indices[tri * 3 + 1] as usize,
            indices[tri * 3 + 2] as usize,
        ];
        let p = [positions[i[0]], positions[i[1]], positions[i[2]]];
        let n = [
            if i[0] < normals.len() {
                normals[i[0]]
            } else {
                [0.0, 1.0, 0.0]
            },
            if i[1] < normals.len() {
                normals[i[1]]
            } else {
                [0.0, 1.0, 0.0]
            },
            if i[2] < normals.len() {
                normals[i[2]]
            } else {
                [0.0, 1.0, 0.0]
            },
        ];
        let u = [
            if i[0] < uvs.len() {
                uvs[i[0]]
            } else {
                [0.0, 0.0]
            },
            if i[1] < uvs.len() {
                uvs[i[1]]
            } else {
                [0.0, 0.0]
            },
            if i[2] < uvs.len() {
                uvs[i[2]]
            } else {
                [0.0, 0.0]
            },
        ];
        let d = [
            plane.signed_distance(p[0]),
            plane.signed_distance(p[1]),
            plane.signed_distance(p[2]),
        ];

        let above_mask: [bool; 3] = [d[0] >= -EPS, d[1] >= -EPS, d[2] >= -EPS];
        let n_above = above_mask.iter().filter(|&&b| b).count();

        if n_above == 3 {
            push_tri(
                &mut above_pos,
                &mut above_nrm,
                &mut above_uv,
                &mut above_idx,
                &p,
                &n,
                &u,
            );
        } else if n_above == 0 {
            push_tri(
                &mut below_pos,
                &mut below_nrm,
                &mut below_uv,
                &mut below_idx,
                &p,
                &n,
                &u,
            );
        } else {
            // Straddles — clip using Sutherland-Hodgman style
            // Collect above and below vertex lists, inserting intersection points
            clip_triangle(
                &p,
                &n,
                &u,
                &d,
                &above_mask,
                &interp_v,
                &mut above_pos,
                &mut above_nrm,
                &mut above_uv,
                &mut above_idx,
                &mut below_pos,
                &mut below_nrm,
                &mut below_uv,
                &mut below_idx,
                push_tri,
            );
        }
    }

    let make_mesh =
        |pos: Vec<[f32; 3]>, nrm: Vec<[f32; 3]>, uv: Vec<[f32; 2]>, idx: Vec<u32>| -> MeshBuffers {
            let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; pos.len()];
            MeshBuffers {
                positions: pos,
                normals: nrm,
                tangents,
                uvs: uv,
                indices: idx,
                colors: None,
                has_suit: mesh.has_suit,
            }
        };

    (
        make_mesh(above_pos, above_nrm, above_uv, above_idx),
        make_mesh(below_pos, below_nrm, below_uv, below_idx),
    )
}

// Clip a straddling triangle, appending sub-triangles to above / below buffers.
#[allow(clippy::too_many_arguments)]
fn clip_triangle(
    p: &[[f32; 3]; 3],
    n: &[[f32; 3]; 3],
    u: &[[f32; 2]; 3],
    d: &[f32; 3],
    above_mask: &[bool; 3],
    interp_v: &InterpVertFn,
    above_pos: &mut Vec<[f32; 3]>,
    above_nrm: &mut Vec<[f32; 3]>,
    above_uv: &mut Vec<[f32; 2]>,
    above_idx: &mut Vec<u32>,
    below_pos: &mut Vec<[f32; 3]>,
    below_nrm: &mut Vec<[f32; 3]>,
    below_uv: &mut Vec<[f32; 2]>,
    below_idx: &mut Vec<u32>,
    push_tri: impl Fn(
        &mut Vec<[f32; 3]>,
        &mut Vec<[f32; 3]>,
        &mut Vec<[f32; 2]>,
        &mut Vec<u32>,
        &[[f32; 3]; 3],
        &[[f32; 3]; 3],
        &[[f32; 2]; 3],
    ),
) {
    let n_above = above_mask.iter().filter(|&&b| b).count();

    // Reorder so that the "lone" vertex is first
    let (order_a, order_b) = if n_above == 1 {
        // 1 above, 2 below: lone is the above vertex
        let lone = above_mask.iter().position(|&b| b).unwrap_or(0);
        let others: Vec<usize> = (0..3).filter(|&i| i != lone).collect();
        ([lone, others[0], others[1]], true)
    } else {
        // 2 above, 1 below: lone is the below vertex
        let lone = above_mask.iter().position(|&b| !b).unwrap_or(0);
        let others: Vec<usize> = (0..3).filter(|&i| i != lone).collect();
        ([lone, others[0], others[1]], false)
    };

    let v0 = p[order_a[0]];
    let v1 = p[order_a[1]];
    let v2 = p[order_a[2]];
    let n0 = n[order_a[0]];
    let n1 = n[order_a[1]];
    let n2 = n[order_a[2]];
    let u0 = u[order_a[0]];
    let u1 = u[order_a[1]];
    let u2 = u[order_a[2]];
    let d0 = d[order_a[0]];
    let d1 = d[order_a[1]];
    let d2 = d[order_a[2]];

    // Intersections on edge (v0-v1) and (v0-v2)
    let (ip01, in01, iu01) = interp_v(v0, v1, n0, n1, u0, u1, d0, d1);
    let (ip02, in02, iu02) = interp_v(v0, v2, n0, n2, u0, u2, d0, d2);

    if order_b {
        // v0 is above: one triangle above, two below
        let tri_a = [v0, ip01, ip02];
        let nrm_a = [n0, in01, in02];
        let uv_a = [u0, iu01, iu02];
        push_tri(
            above_pos, above_nrm, above_uv, above_idx, &tri_a, &nrm_a, &uv_a,
        );
        let tri_b1 = [ip01, v1, v2];
        let nrm_b1 = [in01, n1, n2];
        let uv_b1 = [iu01, u1, u2];
        push_tri(
            below_pos, below_nrm, below_uv, below_idx, &tri_b1, &nrm_b1, &uv_b1,
        );
        let tri_b2 = [ip01, v2, ip02];
        let nrm_b2 = [in01, n2, in02];
        let uv_b2 = [iu01, u2, iu02];
        push_tri(
            below_pos, below_nrm, below_uv, below_idx, &tri_b2, &nrm_b2, &uv_b2,
        );
    } else {
        // v0 is below: one triangle below, two above
        let tri_b = [v0, ip01, ip02];
        let nrm_b = [n0, in01, in02];
        let uv_b = [u0, iu01, iu02];
        push_tri(
            below_pos, below_nrm, below_uv, below_idx, &tri_b, &nrm_b, &uv_b,
        );
        let tri_a1 = [ip01, v1, v2];
        let nrm_a1 = [in01, n1, n2];
        let uv_a1 = [iu01, u1, u2];
        push_tri(
            above_pos, above_nrm, above_uv, above_idx, &tri_a1, &nrm_a1, &uv_a1,
        );
        let tri_a2 = [ip01, v2, ip02];
        let nrm_a2 = [in01, n2, in02];
        let uv_a2 = [iu01, u2, iu02];
        push_tri(
            above_pos, above_nrm, above_uv, above_idx, &tri_a2, &nrm_a2, &uv_a2,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Convenience functions
// ─────────────────────────────────────────────────────────────────────────────

/// Extract a horizontal cross-section at a given Y height.
///
/// Returns the first (and usually only) `CrossSection` found, or an empty
/// section if no intersection exists.
pub fn horizontal_slice(mesh: &MeshBuffers, y: f32) -> CrossSection {
    let plane = SlicePlane::at_height(y);
    let result = slice_mesh(mesh, &plane);
    result.sections.into_iter().next().unwrap_or(CrossSection {
        points: vec![],
        closed: false,
    })
}

/// Measure the perimeter of the cross-section at a given Y height.
/// Useful for body measurements (e.g., waist, chest circumference).
pub fn circumference_at_height(mesh: &MeshBuffers, y: f32) -> f32 {
    let section = horizontal_slice(mesh, y);
    section.perimeter()
}

/// Compute the bounding-box width (X-axis extent) of the mesh cross-section
/// at each of the given Y heights.
pub fn width_profile(mesh: &MeshBuffers, y_values: &[f32]) -> Vec<f32> {
    y_values
        .iter()
        .map(|&y| {
            let section = horizontal_slice(mesh, y);
            if section.points.is_empty() {
                return 0.0;
            }
            let (mn, mx) = section.bbox_3d();
            mx[0] - mn[0]
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal builder helper for tests
// ─────────────────────────────────────────────────────────────────────────────

fn make_mesh_buffers(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> MeshBuffers {
    let n = positions.len();
    MeshBuffers {
        normals: vec![[0.0, 1.0, 0.0]; n],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
        uvs: vec![[0.0, 0.0]; n],
        indices,
        positions,
        colors: None,
        has_suit: false,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Build a simple unit cube mesh (12 triangles)
// ─────────────────────────────────────────────────────────────────────────────

fn unit_cube_mesh() -> MeshBuffers {
    // 8 corners of a unit cube centred at (0,0,0) spanning ±0.5
    let positions: Vec<[f32; 3]> = vec![
        [-0.5, -0.5, -0.5], // 0
        [0.5, -0.5, -0.5],  // 1
        [0.5, 0.5, -0.5],   // 2
        [-0.5, 0.5, -0.5],  // 3
        [-0.5, -0.5, 0.5],  // 4
        [0.5, -0.5, 0.5],   // 5
        [0.5, 0.5, 0.5],    // 6
        [-0.5, 0.5, 0.5],   // 7
    ];
    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        // bottom (-Y)
        0,1,2,  0,2,3,
        // top (+Y)
        4,6,5,  4,7,6,
        // front (-Z)
        0,4,5,  0,5,1,
        // back (+Z)
        2,6,7,  2,7,3,
        // left (-X)
        0,3,7,  0,7,4,
        // right (+X)
        1,5,6,  1,6,2,
    ];
    make_mesh_buffers(positions, indices)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn write_report(name: &str, text: &str) {
        let path = format!("/tmp/{name}");
        let mut f = std::fs::File::create(&path).expect("open /tmp file");
        writeln!(f, "{text}").unwrap();
    }

    // A single triangle that lies exactly in y=0 (XZ plane)
    fn flat_triangle_mesh() -> MeshBuffers {
        make_mesh_buffers(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            vec![0, 1, 2],
        )
    }

    // Two triangles: one above y=0, one below y=0
    fn two_tri_straddle_mesh() -> MeshBuffers {
        //      y=+1 ---- triangle A (indices 0,1,2)
        //   ---y=0---
        //      y=-1 ---- triangle B (indices 3,4,5)
        make_mesh_buffers(
            vec![
                [-1.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [0.0, -1.0, 1.0],
            ],
            vec![0, 1, 2, 3, 4, 5],
        )
    }

    // A single triangle with verts spanning y=-1 to y=+1
    fn straddling_triangle_mesh() -> MeshBuffers {
        make_mesh_buffers(
            vec![
                [0.0, -1.0, 0.0], // below
                [1.0, 1.0, 0.0],  // above
                [-1.0, 1.0, 0.0], // above
            ],
            vec![0, 1, 2],
        )
    }

    // ── SlicePlane tests ──────────────────────────────────────────────────────

    #[test]
    fn slice_plane_signed_distance_above() {
        let plane = SlicePlane::at_height(0.0);
        let d = plane.signed_distance([0.0, 2.0, 0.0]);
        assert!(d > 0.0, "point above plane should have positive distance");
        write_report("test_signed_distance_above.txt", &format!("d={d}"));
    }

    #[test]
    fn slice_plane_signed_distance_below() {
        let plane = SlicePlane::at_height(0.0);
        let d = plane.signed_distance([0.0, -3.0, 0.0]);
        assert!(d < 0.0, "point below plane should have negative distance");
    }

    #[test]
    fn slice_plane_signed_distance_on_plane() {
        let plane = SlicePlane::at_height(1.0);
        let d = plane.signed_distance([5.0, 1.0, 7.0]);
        assert!(d.abs() < 1e-5, "point on plane should have ~0 distance");
    }

    #[test]
    fn slice_plane_presets() {
        let xy = SlicePlane::xy();
        assert!((xy.normal[2] - 1.0).abs() < 1e-6);
        let xz = SlicePlane::xz();
        assert!((xz.normal[1] - 1.0).abs() < 1e-6);
        let yz = SlicePlane::yz();
        assert!((yz.normal[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn slice_plane_at_height() {
        let plane = SlicePlane::at_height(3.5);
        assert!((plane.origin[1] - 3.5).abs() < 1e-6);
    }

    // ── edge_plane_intersect tests ────────────────────────────────────────────

    #[test]
    fn edge_plane_intersect_midpoint() {
        let plane = SlicePlane::at_height(0.0);
        let v0 = [0.0, -1.0, 0.0];
        let v1 = [0.0, 1.0, 0.0];
        let pt = edge_plane_intersect(v0, v1, &plane);
        assert!(
            (pt[1]).abs() < 1e-5,
            "intersection should be at y=0, got {}",
            pt[1]
        );
        write_report("test_edge_intersect_midpoint.txt", &format!("{pt:?}"));
    }

    #[test]
    fn edge_plane_intersect_three_quarter() {
        let plane = SlicePlane::at_height(0.5);
        let v0 = [0.0, 0.0, 0.0]; // d = -0.5
        let v1 = [0.0, 2.0, 0.0]; // d = +1.5
        let pt = edge_plane_intersect(v0, v1, &plane);
        assert!((pt[1] - 0.5).abs() < 1e-5, "y should be 0.5, got {}", pt[1]);
    }

    // ── ray_plane_intersect tests ─────────────────────────────────────────────

    #[test]
    fn ray_plane_intersect_hits() {
        let plane = SlicePlane::at_height(2.0);
        let origin = [0.0, 0.0, 0.0];
        let dir = [0.0, 1.0, 0.0];
        let hit = ray_plane_intersect(origin, dir, &plane);
        assert!(hit.is_some(), "ray should intersect horizontal plane");
        let pt = hit.unwrap();
        assert!((pt[1] - 2.0).abs() < 1e-5, "hit y should be 2.0");
        write_report("test_ray_plane_hit.txt", &format!("{pt:?}"));
    }

    #[test]
    fn ray_plane_intersect_parallel_misses() {
        let plane = SlicePlane::at_height(1.0);
        let origin = [0.0, 0.0, 0.0];
        let dir = [1.0, 0.0, 0.0]; // parallel to XZ plane
        let hit = ray_plane_intersect(origin, dir, &plane);
        assert!(hit.is_none(), "parallel ray should return None");
    }

    // ── slice_mesh tests ──────────────────────────────────────────────────────

    #[test]
    fn slice_mesh_cube_at_midpoint() {
        let cube = unit_cube_mesh();
        let plane = SlicePlane::at_height(0.0);
        let result = slice_mesh(&cube, &plane);

        write_report(
            "test_slice_cube.txt",
            &format!(
                "sections={} above={} below={} edges={}",
                result.sections.len(),
                result.triangle_count_above,
                result.triangle_count_below,
                result.intersection_edge_count
            ),
        );

        assert!(
            result.intersection_edge_count > 0,
            "cube sliced at y=0 should produce intersections"
        );
        assert!(
            !result.sections.is_empty(),
            "should produce at least one section"
        );
    }

    #[test]
    fn slice_mesh_no_intersection() {
        // Slice the cube well above it — should have 0 intersections
        let cube = unit_cube_mesh();
        let plane = SlicePlane::at_height(10.0);
        let result = slice_mesh(&cube, &plane);
        assert_eq!(result.intersection_edge_count, 0);
        assert_eq!(
            result.triangle_count_above, 0,
            "all tris should be below y=10"
        );
        assert_eq!(result.triangle_count_below, cube.face_count());
    }

    #[test]
    fn slice_mesh_straddling_triangle() {
        let mesh = straddling_triangle_mesh();
        let plane = SlicePlane::at_height(0.0);
        let result = slice_mesh(&mesh, &plane);
        assert_eq!(
            result.intersection_edge_count, 1,
            "one crossing segment expected"
        );
    }

    // ── CrossSection tests ────────────────────────────────────────────────────

    #[test]
    fn cross_section_perimeter_square() {
        // A unit square section in the XZ plane at y=0
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let section = CrossSection {
            points: pts,
            closed: true,
        };
        let perim = section.perimeter();
        assert!(
            (perim - 4.0).abs() < 1e-5,
            "unit square perimeter should be 4, got {perim}"
        );
        write_report("test_perimeter_square.txt", &format!("perim={perim}"));
    }

    #[test]
    fn cross_section_area_unit_square() {
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let section = CrossSection {
            points: pts,
            closed: true,
        };
        let plane = SlicePlane::at_height(0.0);
        let area = section.area(&plane);
        assert!(
            (area - 1.0).abs() < 0.01,
            "unit square area should be ~1.0, got {area}"
        );
        write_report("test_area_square.txt", &format!("area={area}"));
    }

    #[test]
    fn cross_section_centroid() {
        let pts = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.0, 2.0],
            [0.0, 0.0, 2.0],
        ];
        let section = CrossSection {
            points: pts,
            closed: true,
        };
        let c = section.centroid();
        assert!((c[0] - 1.0).abs() < 1e-5, "centroid X should be 1.0");
        assert!((c[2] - 1.0).abs() < 1e-5, "centroid Z should be 1.0");
    }

    #[test]
    fn cross_section_bbox_3d() {
        let pts = vec![[-1.0, 0.0, 2.0], [3.0, 0.0, -1.0]];
        let section = CrossSection {
            points: pts,
            closed: false,
        };
        let (mn, mx) = section.bbox_3d();
        assert!((mn[0] - -1.0).abs() < 1e-5);
        assert!((mx[0] - 3.0).abs() < 1e-5);
        assert!((mn[2] - -1.0).abs() < 1e-5);
        assert!((mx[2] - 2.0).abs() < 1e-5);
    }

    // ── split_mesh tests ──────────────────────────────────────────────────────

    #[test]
    fn split_mesh_cube_at_zero() {
        let cube = unit_cube_mesh();
        let plane = SlicePlane::at_height(0.0);
        let (above, below) = split_mesh(&cube, &plane);

        write_report(
            "test_split_cube.txt",
            &format!(
                "above_verts={} above_faces={} below_verts={} below_faces={}",
                above.vertex_count(),
                above.face_count(),
                below.vertex_count(),
                below.face_count()
            ),
        );

        assert!(above.face_count() > 0, "above mesh should have triangles");
        assert!(below.face_count() > 0, "below mesh should have triangles");

        // All above vertices should be at y >= -epsilon
        for p in &above.positions {
            assert!(p[1] >= -1e-4, "above mesh vertex y={} is below plane", p[1]);
        }
        // All below vertices should be at y <= +epsilon
        for p in &below.positions {
            assert!(p[1] <= 1e-4, "below mesh vertex y={} is above plane", p[1]);
        }
    }

    // ── horizontal_slice / circumference tests ────────────────────────────────

    #[test]
    fn circumference_at_equator_is_positive() {
        let cube = unit_cube_mesh();
        let c = circumference_at_height(&cube, 0.0);
        assert!(
            c > 0.0,
            "circumference at cube equator should be positive, got {c}"
        );
        write_report("test_circumference.txt", &format!("circumference={c}"));
    }

    #[test]
    fn circumference_outside_mesh_is_zero() {
        let cube = unit_cube_mesh();
        let c = circumference_at_height(&cube, 100.0);
        assert_eq!(c, 0.0, "circumference above cube should be 0");
    }

    // ── width_profile tests ───────────────────────────────────────────────────

    #[test]
    fn width_profile_length_matches_input() {
        let cube = unit_cube_mesh();
        let ys = vec![-0.4, -0.2, 0.0, 0.2, 0.4];
        let widths = width_profile(&cube, &ys);
        assert_eq!(
            widths.len(),
            ys.len(),
            "width_profile length must match y_values length"
        );
        write_report("test_width_profile.txt", &format!("{widths:?}"));
    }

    #[test]
    fn width_profile_nonzero_at_valid_heights() {
        let cube = unit_cube_mesh();
        let ys = vec![0.0f32];
        let widths = width_profile(&cube, &ys);
        assert!(
            widths[0] > 0.0,
            "width at cube equator should be > 0, got {}",
            widths[0]
        );
    }

    // ── to_2d tests ───────────────────────────────────────────────────────────

    #[test]
    fn to_2d_preserves_count() {
        let pts = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [-1.0, 0.0, 0.0]];
        let section = CrossSection {
            points: pts.clone(),
            closed: true,
        };
        let plane = SlicePlane::at_height(0.0);
        let pts2d = section.to_2d(&plane);
        assert_eq!(pts2d.len(), pts.len());
    }

    // ── two_tri_straddle_mesh: above/below counts ─────────────────────────────

    #[test]
    fn two_separate_tris_count() {
        let mesh = two_tri_straddle_mesh();
        let plane = SlicePlane::at_height(0.0);
        let result = slice_mesh(&mesh, &plane);
        assert_eq!(result.triangle_count_above, 1, "exactly one tri above y=0");
        assert_eq!(result.triangle_count_below, 1, "exactly one tri below y=0");
        assert_eq!(
            result.intersection_edge_count, 0,
            "no straddling tris → no segments"
        );
        write_report("test_two_tri.txt", &format!("{:?}", result.sections.len()));
    }
}
