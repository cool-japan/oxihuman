// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh hole detection and hole-filling (triangulated polygon patches).
//!
//! Provides algorithms to find open boundary loops in a mesh and fill them
//! with new triangulated geometry using fan, ear-clip, or minimum-area strategies.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
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
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

#[inline]
fn len2(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

// ---------------------------------------------------------------------------
// Public data types
// ---------------------------------------------------------------------------

/// A hole: an ordered boundary loop of vertex indices.
#[derive(Debug, Clone)]
pub struct MeshHole {
    /// Ordered boundary vertices forming a closed loop.
    pub loop_vertices: Vec<u32>,
    /// Approximate planar area of the hole.
    pub area_estimate: f32,
}

impl MeshHole {
    /// Number of vertices in the boundary loop.
    pub fn vertex_count(&self) -> usize {
        self.loop_vertices.len()
    }

    /// Compute the perimeter length of the boundary loop.
    pub fn perimeter(&self, positions: &[[f32; 3]]) -> f32 {
        let n = self.loop_vertices.len();
        if n < 2 {
            return 0.0;
        }
        let mut total = 0.0f32;
        for i in 0..n {
            let a = self.loop_vertices[i] as usize;
            let b = self.loop_vertices[(i + 1) % n] as usize;
            if a < positions.len() && b < positions.len() {
                let d = sub3(positions[b], positions[a]);
                total += len3(d);
            }
        }
        total
    }

    /// Compute the centroid of the boundary loop vertices.
    pub fn centroid(&self, positions: &[[f32; 3]]) -> [f32; 3] {
        let n = self.loop_vertices.len();
        if n == 0 {
            return [0.0, 0.0, 0.0];
        }
        let mut sum = [0.0f32; 3];
        let mut count = 0usize;
        for &vi in &self.loop_vertices {
            let idx = vi as usize;
            if idx < positions.len() {
                sum = add3(sum, positions[idx]);
                count += 1;
            }
        }
        if count == 0 {
            return [0.0, 0.0, 0.0];
        }
        let inv = 1.0 / count as f32;
        scale3(sum, inv)
    }

    /// Estimate the normal of the hole polygon using Newell's method.
    pub fn normal_estimate(&self, positions: &[[f32; 3]]) -> [f32; 3] {
        let n = self.loop_vertices.len();
        if n < 3 {
            return [0.0, 1.0, 0.0];
        }
        let mut normal = [0.0f32; 3];
        for i in 0..n {
            let a = self.loop_vertices[i] as usize;
            let b = self.loop_vertices[(i + 1) % n] as usize;
            if a >= positions.len() || b >= positions.len() {
                continue;
            }
            let pa = positions[a];
            let pb = positions[b];
            normal[0] += (pa[1] - pb[1]) * (pa[2] + pb[2]);
            normal[1] += (pa[2] - pb[2]) * (pa[0] + pb[0]);
            normal[2] += (pa[0] - pb[0]) * (pa[1] + pb[1]);
        }
        normalize3(normal)
    }
}

/// Result of hole filling.
#[derive(Debug, Clone)]
pub struct PatchResult {
    /// The resulting mesh with holes filled.
    pub mesh: MeshBuffers,
    /// Number of holes that were filled.
    pub holes_filled: usize,
    /// Number of new vertices added.
    pub new_vertices: usize,
    /// Number of new triangles added.
    pub new_triangles: usize,
}

/// Hole filling strategy.
#[derive(Debug, Clone, PartialEq)]
pub enum PatchStrategy {
    /// Triangle fan from centroid (fast, works for convex holes).
    Fan,
    /// Ear-clipping triangulation (works for any simple polygon).
    EarClip,
    /// Minimum total triangle area (greedy).
    MinArea,
}

// ---------------------------------------------------------------------------
// Internal: build boundary edge map
// ---------------------------------------------------------------------------

fn build_boundary_edge_map(mesh: &MeshBuffers) -> HashMap<(u32, u32), u32> {
    let mut map: HashMap<(u32, u32), u32> = HashMap::new();
    let indices = &mesh.indices;
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        for &(p, q) in &[(a, b), (b, c), (c, a)] {
            let key = (p.min(q), p.max(q));
            *map.entry(key).or_insert(0) += 1;
        }
    }
    map
}

/// Get directed boundary edges: edges used exactly once in the mesh,
/// keeping the direction as they appear in the winding order.
fn directed_boundary_edges(mesh: &MeshBuffers) -> Vec<(u32, u32)> {
    let edge_count = build_boundary_edge_map(mesh);
    let indices = &mesh.indices;
    let face_count = indices.len() / 3;
    let mut directed = Vec::new();
    for f in 0..face_count {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        for &(p, q) in &[(a, b), (b, c), (c, a)] {
            let key = (p.min(q), p.max(q));
            if edge_count.get(&key).copied().unwrap_or(0) == 1 {
                directed.push((p, q));
            }
        }
    }
    directed
}

/// Walk directed boundary edges into ordered loops.
fn walk_boundary_loops(directed_edges: &[(u32, u32)]) -> Vec<Vec<u32>> {
    if directed_edges.is_empty() {
        return Vec::new();
    }

    // Build a map from start vertex -> end vertex for directed edges.
    let mut next_map: HashMap<u32, u32> = HashMap::new();
    for &(a, b) in directed_edges {
        next_map.insert(a, b);
    }

    let mut visited: HashMap<u32, bool> = next_map.keys().map(|&k| (k, false)).collect();
    let mut loops = Vec::new();

    for &start in next_map.keys() {
        if visited.get(&start).copied().unwrap_or(true) {
            continue;
        }

        let mut loop_verts = vec![start];
        *visited.entry(start).or_insert(true) = true;
        let mut current = start;

        loop {
            match next_map.get(&current) {
                None => break,
                Some(&next) => {
                    if next == start {
                        // Closed loop complete
                        loops.push(loop_verts);
                        break;
                    }
                    if visited.get(&next).copied().unwrap_or(false) {
                        // Hit visited vertex — close off
                        loop_verts.push(next);
                        loops.push(loop_verts);
                        break;
                    }
                    *visited.entry(next).or_insert(true) = true;
                    loop_verts.push(next);
                    current = next;
                }
            }
        }
    }

    loops
}

// ---------------------------------------------------------------------------
// Planar area estimate for a polygon loop
// ---------------------------------------------------------------------------

fn estimate_polygon_area(loop_verts: &[u32], positions: &[[f32; 3]]) -> f32 {
    let n = loop_verts.len();
    if n < 3 {
        return 0.0;
    }
    // Compute centroid
    let mut c = [0.0f32; 3];
    let mut count = 0usize;
    for &vi in loop_verts {
        let idx = vi as usize;
        if idx < positions.len() {
            c = add3(c, positions[idx]);
            count += 1;
        }
    }
    if count == 0 {
        return 0.0;
    }
    let inv = 1.0 / count as f32;
    c = scale3(c, inv);

    // Sum triangle areas from centroid
    let mut area = 0.0f32;
    for i in 0..n {
        let a = loop_verts[i] as usize;
        let b = loop_verts[(i + 1) % n] as usize;
        if a >= positions.len() || b >= positions.len() {
            continue;
        }
        let ab = sub3(positions[b], positions[a]);
        let ac = sub3(c, positions[a]);
        let cr = cross3(ab, ac);
        area += len3(cr) * 0.5;
    }
    area
}

// ---------------------------------------------------------------------------
// Public API: find holes
// ---------------------------------------------------------------------------

/// Find all holes (open boundary loops) in a mesh.
pub fn find_holes(mesh: &MeshBuffers) -> Vec<MeshHole> {
    let directed = directed_boundary_edges(mesh);
    let loops = walk_boundary_loops(&directed);

    loops
        .into_iter()
        .filter(|lp| lp.len() >= 3)
        .map(|loop_vertices| {
            let area_estimate = estimate_polygon_area(&loop_vertices, &mesh.positions);
            MeshHole {
                loop_vertices,
                area_estimate,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Public API: is_watertight / hole_count
// ---------------------------------------------------------------------------

/// Check mesh for any open boundary edges; returns true if there are none.
pub fn is_watertight(mesh: &MeshBuffers) -> bool {
    let edge_map = build_boundary_edge_map(mesh);
    edge_map.values().all(|&count| count == 2)
}

/// Count the number of distinct hole loops in the mesh.
pub fn hole_count(mesh: &MeshBuffers) -> usize {
    find_holes(mesh).len()
}

// ---------------------------------------------------------------------------
// Public API: fan_patch
// ---------------------------------------------------------------------------

/// Fan patch: add centroid vertex, fan triangles from it to boundary.
///
/// Returns `(centroid_position, triangles)` where triangles are indices into
/// a vertex array formed by appending the centroid after `positions`.
pub fn fan_patch(positions: &[[f32; 3]], loop_verts: &[u32]) -> ([f32; 3], Vec<u32>) {
    let n = loop_verts.len();
    if n < 3 {
        return ([0.0, 0.0, 0.0], Vec::new());
    }

    // Compute centroid
    let mut c = [0.0f32; 3];
    let mut count = 0usize;
    for &vi in loop_verts {
        let idx = vi as usize;
        if idx < positions.len() {
            c = add3(c, positions[idx]);
            count += 1;
        }
    }
    if count > 0 {
        let inv = 1.0 / count as f32;
        c = scale3(c, inv);
    }

    // Centroid index = positions.len() (will be appended to the combined array)
    let centroid_idx = positions.len() as u32;
    let mut triangles = Vec::with_capacity(n * 3);

    for i in 0..n {
        let a = loop_verts[i];
        let b = loop_verts[(i + 1) % n];
        // Fan from centroid: centroid, a, b  (CCW when boundary is CCW)
        triangles.push(centroid_idx);
        triangles.push(a);
        triangles.push(b);
    }

    (c, triangles)
}

// ---------------------------------------------------------------------------
// Public API: project_polygon_2d
// ---------------------------------------------------------------------------

/// Project 3D polygon vertices onto a best-fit 2D plane defined by `normal`.
///
/// Builds a local (u, v) coordinate frame perpendicular to `normal` and
/// returns the 2-D projections in that frame.
pub fn project_polygon_2d(pts: &[[f32; 3]], normal: &[f32; 3]) -> Vec<[f32; 2]> {
    if pts.is_empty() {
        return Vec::new();
    }

    let n = normalize3(*normal);

    // Build tangent u: pick an arbitrary vector not parallel to n
    let arbitrary = if n[0].abs() < 0.9 {
        [1.0f32, 0.0, 0.0]
    } else {
        [0.0f32, 1.0, 0.0]
    };
    let u = normalize3(cross3(n, arbitrary));
    let v = cross3(n, u);

    // Project centroid to origin
    let centroid = {
        let mut s = [0.0f32; 3];
        for p in pts {
            s = add3(s, *p);
        }
        scale3(s, 1.0 / pts.len() as f32)
    };

    pts.iter()
        .map(|&p| {
            let d = sub3(p, centroid);
            [dot3(d, u), dot3(d, v)]
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Public API: polygon_signed_area_2d
// ---------------------------------------------------------------------------

/// Compute the signed area of a 2D polygon using the shoelace formula.
/// Positive area means CCW winding.
pub fn polygon_signed_area_2d(pts: &[[f32; 2]]) -> f32 {
    let n = pts.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts[i][0] * pts[j][1];
        area -= pts[j][0] * pts[i][1];
    }
    area * 0.5
}

// ---------------------------------------------------------------------------
// Public API: is_ear
// ---------------------------------------------------------------------------

/// Check if vertex `i` in `loop_verts` forms an "ear".
///
/// An ear is a triangle (prev, i, next) that is:
/// - locally convex (correct winding), and
/// - contains no other loop vertices inside it.
pub fn is_ear(i: usize, loop_verts: &[u32], positions: &[[f32; 3]]) -> bool {
    let n = loop_verts.len();
    if n < 3 {
        return false;
    }

    let prev = loop_verts[(i + n - 1) % n] as usize;
    let curr = loop_verts[i] as usize;
    let next = loop_verts[(i + 1) % n] as usize;

    if prev >= positions.len() || curr >= positions.len() || next >= positions.len() {
        return false;
    }

    // Compute normal to determine overall polygon orientation
    let hole_normal = {
        let mut normal = [0.0f32; 3];
        for j in 0..n {
            let a = loop_verts[j] as usize;
            let b = loop_verts[(j + 1) % n] as usize;
            if a >= positions.len() || b >= positions.len() {
                continue;
            }
            let pa = positions[a];
            let pb = positions[b];
            normal[0] += (pa[1] - pb[1]) * (pa[2] + pb[2]);
            normal[1] += (pa[2] - pb[2]) * (pa[0] + pb[0]);
            normal[2] += (pa[0] - pb[0]) * (pa[1] + pb[1]);
        }
        normalize3(normal)
    };

    let pa = positions[prev];
    let pb = positions[curr];
    let pc = positions[next];

    // Check convexity: cross(pb-pa, pc-pa) should have same direction as hole_normal
    let ab = sub3(pb, pa);
    let ac = sub3(pc, pa);
    let cr = cross3(ab, ac);
    let convexity = dot3(cr, hole_normal);
    if convexity <= 0.0 {
        return false; // reflex vertex — not an ear
    }

    // Check that no other loop vertex is inside triangle (pa, pb, pc)
    for (j, &lv) in loop_verts.iter().enumerate() {
        if j == (i + n - 1) % n || j == i || j == (i + 1) % n {
            continue;
        }
        let vj = lv as usize;
        if vj >= positions.len() {
            continue;
        }
        let p = positions[vj];
        if point_in_triangle_3d(p, pa, pb, pc, hole_normal) {
            return false;
        }
    }

    true
}

/// Test if point `p` is inside triangle (a, b, c) using barycentric coords projected to 2D.
fn point_in_triangle_3d(
    p: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    normal: [f32; 3],
) -> bool {
    // Use sign of cross products in the plane defined by `normal`
    let ab = sub3(b, a);
    let bc = sub3(c, b);
    let ca = sub3(a, c);

    let ap = sub3(p, a);
    let bp = sub3(p, b);
    let cp = sub3(p, c);

    let d1 = dot3(cross3(ab, ap), normal);
    let d2 = dot3(cross3(bc, bp), normal);
    let d3 = dot3(cross3(ca, cp), normal);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

// ---------------------------------------------------------------------------
// Public API: ear_clip
// ---------------------------------------------------------------------------

/// Ear-clipping triangulation of a boundary loop.
///
/// Returns list of triangle index triples (global vertex indices from `loop_verts`).
pub fn ear_clip(loop_verts: &[u32], positions: &[[f32; 3]]) -> Vec<[u32; 3]> {
    let n = loop_verts.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return vec![[loop_verts[0], loop_verts[1], loop_verts[2]]];
    }

    let mut remaining: Vec<u32> = loop_verts.to_vec();
    let mut triangles = Vec::with_capacity(n - 2);

    let max_iter = n * n + n;
    let mut iter_count = 0usize;

    while remaining.len() > 3 {
        iter_count += 1;
        if iter_count > max_iter {
            break; // degenerate polygon; avoid infinite loop
        }

        let len = remaining.len();
        let mut ear_found = false;

        for i in 0..len {
            if is_ear(i, &remaining, positions) {
                let prev = remaining[(i + len - 1) % len];
                let curr = remaining[i];
                let next = remaining[(i + 1) % len];
                triangles.push([prev, curr, next]);
                remaining.remove(i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            // Fallback: just clip the first vertex (handles degenerate cases)
            let len = remaining.len();
            let prev = remaining[len - 1];
            let curr = remaining[0];
            let next = remaining[1];
            triangles.push([prev, curr, next]);
            remaining.remove(0);
        }
    }

    // Last triangle
    if remaining.len() == 3 {
        triangles.push([remaining[0], remaining[1], remaining[2]]);
    }

    triangles
}

// ---------------------------------------------------------------------------
// MinArea greedy triangulation
// ---------------------------------------------------------------------------

/// Greedy minimum-area triangulation of a polygon loop.
fn min_area_triangulate(loop_verts: &[u32], positions: &[[f32; 3]]) -> Vec<[u32; 3]> {
    let n = loop_verts.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return vec![[loop_verts[0], loop_verts[1], loop_verts[2]]];
    }

    let mut remaining: Vec<u32> = loop_verts.to_vec();
    let mut triangles = Vec::with_capacity(n - 2);

    while remaining.len() > 3 {
        let len = remaining.len();
        let mut best_idx = 0;
        let mut best_area = f32::MAX;

        for i in 0..len {
            let prev = remaining[(i + len - 1) % len] as usize;
            let curr = remaining[i] as usize;
            let next = remaining[(i + 1) % len] as usize;

            if prev >= positions.len() || curr >= positions.len() || next >= positions.len() {
                continue;
            }

            let ab = sub3(positions[curr], positions[prev]);
            let ac = sub3(positions[next], positions[prev]);
            let cr = cross3(ab, ac);
            let area = len3(cr) * 0.5;

            if area < best_area {
                best_area = area;
                best_idx = i;
            }
        }

        let len = remaining.len();
        let prev = remaining[(best_idx + len - 1) % len];
        let curr = remaining[best_idx];
        let next = remaining[(best_idx + 1) % len];
        triangles.push([prev, curr, next]);
        remaining.remove(best_idx);
    }

    if remaining.len() == 3 {
        triangles.push([remaining[0], remaining[1], remaining[2]]);
    }

    triangles
}

// ---------------------------------------------------------------------------
// Public API: fill_hole
// ---------------------------------------------------------------------------

/// Fill a specific hole using the given strategy.
///
/// Returns `(new_positions, new_triangle_indices)` where triangle indices are
/// into the combined vertex array (original positions + new_positions appended).
pub fn fill_hole(
    mesh: &MeshBuffers,
    hole: &MeshHole,
    strategy: PatchStrategy,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let loop_verts = &hole.loop_vertices;
    let positions = &mesh.positions;

    if loop_verts.len() < 3 {
        return (Vec::new(), Vec::new());
    }

    match strategy {
        PatchStrategy::Fan => {
            let (centroid, tris) = fan_patch(positions, loop_verts);
            (vec![centroid], tris)
        }
        PatchStrategy::EarClip => {
            let tri_verts = ear_clip(loop_verts, positions);
            let mut flat = Vec::with_capacity(tri_verts.len() * 3);
            for t in tri_verts {
                flat.push(t[0]);
                flat.push(t[1]);
                flat.push(t[2]);
            }
            (Vec::new(), flat)
        }
        PatchStrategy::MinArea => {
            let tri_verts = min_area_triangulate(loop_verts, positions);
            let mut flat = Vec::with_capacity(tri_verts.len() * 3);
            for t in tri_verts {
                flat.push(t[0]);
                flat.push(t[1]);
                flat.push(t[2]);
            }
            (Vec::new(), flat)
        }
    }
}

// ---------------------------------------------------------------------------
// Public API: fill_holes
// ---------------------------------------------------------------------------

/// Fill all holes in a mesh using the given strategy.
pub fn fill_holes(mesh: &MeshBuffers, strategy: PatchStrategy) -> PatchResult {
    let holes = find_holes(mesh);
    let holes_filled = holes.len();

    if holes_filled == 0 {
        return PatchResult {
            mesh: mesh.clone(),
            holes_filled: 0,
            new_vertices: 0,
            new_triangles: 0,
        };
    }

    // Build combined mesh
    let mut new_positions = mesh.positions.clone();
    let mut new_normals = mesh.normals.clone();
    let mut new_tangents = mesh.tangents.clone();
    let mut new_uvs = mesh.uvs.clone();
    let mut new_indices = mesh.indices.clone();

    let mut total_new_verts = 0usize;
    let mut total_new_tris = 0usize;

    for hole in &holes {
        // Temporarily build a helper mesh with current positions for fill_hole calls
        let helper_mesh = MeshBuffers {
            positions: new_positions.clone(),
            normals: new_normals.clone(),
            tangents: new_tangents.clone(),
            uvs: new_uvs.clone(),
            indices: new_indices.clone(),
            colors: None,
            has_suit: mesh.has_suit,
        };

        let (patch_positions, patch_indices) = fill_hole(&helper_mesh, hole, strategy.clone());

        let patch_tri_count = patch_indices.len() / 3;

        // Append new vertices
        for &pos in &patch_positions {
            new_positions.push(pos);
            // Estimate normal from hole
            let normal = hole.normal_estimate(&helper_mesh.positions);
            new_normals.push(normal);
            new_tangents.push([1.0, 0.0, 0.0, 1.0]);
            new_uvs.push([0.0, 0.0]);
        }

        // Append new triangle indices
        new_indices.extend_from_slice(&patch_indices);

        total_new_verts += patch_positions.len();
        total_new_tris += patch_tri_count;
    }

    let result_mesh = MeshBuffers {
        positions: new_positions,
        normals: new_normals,
        tangents: new_tangents,
        uvs: new_uvs,
        indices: new_indices,
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    };

    PatchResult {
        mesh: result_mesh,
        holes_filled,
        new_vertices: total_new_verts,
        new_triangles: total_new_tris,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            has_suit: false,
        })
    }

    /// Build a flat disc mesh with a hole: outer ring of 8 verts,
    /// each consecutive pair connected to center (but center removed),
    /// leaving a boundary loop.
    fn open_disc_mesh() -> MeshBuffers {
        // 8-sided ring mesh: verts 0-7 on a unit circle, no center vertex.
        // Each triangle: (i, (i+1)%8, (i+2)%8) creates a ring with one open boundary.
        // Simpler: two triangles forming a quad strip around a square, with top open.
        //
        // Actually: make a simple mesh with an obvious hole.
        // Mesh: a "C" shape — two triangles with a shared edge, leaving 4 boundary verts.
        //
        // Vertices:
        //   0: (0,0,0)  1: (1,0,0)  2: (2,0,0)
        //   3: (0,1,0)  4: (1,1,0)  5: (2,1,0)
        //   6: (0,2,0)  7: (1,2,0)  8: (2,2,0)
        //
        // Top face: 6-7-8 (closed region). Bottom face: 0-1-2 (closed region).
        // Side faces: left (0-3-6, 0-6-3), right (2-5-8, ...)
        // but leave out the top-open boundary.
        //
        // Simpler: a flat plane with the center quad removed.
        // Let's use a donut-like: outer 4 verts, inner 4 verts (hole).
        //   Outer: 0=(0,0,0), 1=(2,0,0), 2=(2,2,0), 3=(0,2,0)
        //   Inner: 4=(0.5,0.5,0), 5=(1.5,0.5,0), 6=(1.5,1.5,0), 7=(0.5,1.5,0)
        // Triangulate the ring between outer and inner.
        let positions = vec![
            [0.0, 0.0, 0.0], // 0
            [2.0, 0.0, 0.0], // 1
            [2.0, 2.0, 0.0], // 2
            [0.0, 2.0, 0.0], // 3
            [0.5, 0.5, 0.0], // 4
            [1.5, 0.5, 0.0], // 5
            [1.5, 1.5, 0.0], // 6
            [0.5, 1.5, 0.0], // 7
        ];
        // Ring triangles (outer -> inner)
        let indices = vec![
            0, 1, 5, 0, 5, 4, // bottom side
            1, 2, 6, 1, 6, 5, // right side
            2, 3, 7, 2, 7, 6, // top side
            3, 0, 4, 3, 4, 7, // left side
        ];
        make_mesh(positions, indices)
    }

    /// A simple closed tetrahedron (no holes).
    fn closed_tetrahedron() -> MeshBuffers {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.333, 1.0],
        ];
        let indices = vec![
            0, 1, 2, // base
            0, 1, 3, // front
            1, 2, 3, // right
            2, 0, 3, // left
        ];
        make_mesh(positions, indices)
    }

    /// A mesh with two holes: bottom and top openings of a cylinder-like shape.
    fn cylinder_open_both_ends() -> MeshBuffers {
        // 4-sided prism, no caps
        let positions = vec![
            [0.0, 0.0, 0.0], // 0: bottom ring
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4: top ring
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ];
        // Side faces only (CCW from outside)
        let indices = vec![
            0, 1, 5, 0, 5, 4, // front
            1, 2, 6, 1, 6, 5, // right
            2, 3, 7, 2, 7, 6, // back
            3, 0, 4, 3, 4, 7, // left
        ];
        make_mesh(positions, indices)
    }

    // -----------------------------------------------------------------------
    // Test 1: closed mesh is watertight
    // -----------------------------------------------------------------------
    #[test]
    fn test_closed_mesh_is_watertight() {
        let mesh = closed_tetrahedron();
        assert!(is_watertight(&mesh), "tetrahedron should be watertight");
    }

    // -----------------------------------------------------------------------
    // Test 2: open mesh is not watertight
    // -----------------------------------------------------------------------
    #[test]
    fn test_open_mesh_not_watertight() {
        let mesh = open_disc_mesh();
        assert!(!is_watertight(&mesh), "ring mesh should not be watertight");
    }

    // -----------------------------------------------------------------------
    // Test 3: find_holes returns correct count for donut mesh
    // -----------------------------------------------------------------------
    #[test]
    fn test_find_holes_donut_count() {
        let mesh = open_disc_mesh();
        let holes = find_holes(&mesh);
        // The ring mesh has two boundary loops: outer (0,1,2,3) and inner (4,5,6,7)
        assert!(
            !holes.is_empty(),
            "donut mesh should have at least one hole"
        );
    }

    // -----------------------------------------------------------------------
    // Test 4: find_holes returns 0 for closed mesh
    // -----------------------------------------------------------------------
    #[test]
    fn test_find_holes_closed_mesh_zero() {
        let mesh = closed_tetrahedron();
        let holes = find_holes(&mesh);
        assert_eq!(holes.len(), 0, "closed mesh should have no holes");
    }

    // -----------------------------------------------------------------------
    // Test 5: hole_count matches find_holes
    // -----------------------------------------------------------------------
    #[test]
    fn test_hole_count_matches_find_holes() {
        let mesh = open_disc_mesh();
        assert_eq!(hole_count(&mesh), find_holes(&mesh).len());
    }

    // -----------------------------------------------------------------------
    // Test 6: MeshHole vertex_count
    // -----------------------------------------------------------------------
    #[test]
    fn test_mesh_hole_vertex_count() {
        let hole = MeshHole {
            loop_vertices: vec![0, 1, 2, 3],
            area_estimate: 1.0,
        };
        assert_eq!(hole.vertex_count(), 4);
    }

    // -----------------------------------------------------------------------
    // Test 7: MeshHole perimeter
    // -----------------------------------------------------------------------
    #[test]
    fn test_mesh_hole_perimeter() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let hole = MeshHole {
            loop_vertices: vec![0, 1, 2, 3],
            area_estimate: 1.0,
        };
        let p = hole.perimeter(&positions);
        assert!(
            (p - 4.0).abs() < 1e-5,
            "square perimeter should be 4.0, got {p}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 8: MeshHole centroid
    // -----------------------------------------------------------------------
    #[test]
    fn test_mesh_hole_centroid() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ];
        let hole = MeshHole {
            loop_vertices: vec![0, 1, 2, 3],
            area_estimate: 4.0,
        };
        let c = hole.centroid(&positions);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // Test 9: fan_patch generates correct triangle count
    // -----------------------------------------------------------------------
    #[test]
    fn test_fan_patch_triangle_count() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let loop_verts = vec![0u32, 1, 2, 3];
        let (centroid, tris) = fan_patch(&positions, &loop_verts);
        // 4 boundary verts => 4 triangles => 12 indices
        assert_eq!(
            tris.len(),
            12,
            "fan_patch should produce 4 triangles for 4-vert loop"
        );
        // Centroid should be at (0.5, 0.5, 0.0)
        assert!((centroid[0] - 0.5).abs() < 1e-5);
        assert!((centroid[1] - 0.5).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // Test 10: ear_clip produces correct triangle count
    // -----------------------------------------------------------------------
    #[test]
    fn test_ear_clip_triangle_count() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [3.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ];
        let loop_verts = vec![0u32, 1, 2, 3];
        let tris = ear_clip(&loop_verts, &positions);
        // A quadrilateral should produce 2 triangles
        assert_eq!(tris.len(), 2, "ear_clip should produce 2 tris for quad");
    }

    // -----------------------------------------------------------------------
    // Test 11: polygon_signed_area_2d square
    // -----------------------------------------------------------------------
    #[test]
    fn test_polygon_signed_area_2d_square() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let area = polygon_signed_area_2d(&pts);
        assert!(
            (area - 0.5).abs() < 1e-5 || (area - 1.0).abs() < 1e-5,
            "CCW unit square signed area should be 0.5 (shoelace) or 1.0, got {area}"
        );
        // The shoelace formula for unit square CCW gives +0.5 * 2 = 1.0
        // Let me check: (0,0)->(1,0)->(1,1)->(0,1)
        // = (0*0 - 1*0) + (1*1 - 1*0) + (1*1 - 0*1) + (0*0 - 0*1)
        // Shoelace: sum(x_i * y_{i+1} - x_{i+1} * y_i) / 2
        // = (0*0-1*0) + (1*1-1*0) + (1*1-0*1) + (0*0-0*1) = 0 + 1 + 1 + 0 = 2 => area = 1.0
        assert!(
            (area.abs() - 1.0).abs() < 1e-5,
            "unit square area magnitude should be 1.0, got {area}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 12: fill_holes Fan strategy increases triangle count
    // -----------------------------------------------------------------------
    #[test]
    fn test_fill_holes_fan_increases_triangles() {
        let mesh = cylinder_open_both_ends();
        let orig_tri_count = mesh.face_count();
        let result = fill_holes(&mesh, PatchStrategy::Fan);
        assert!(
            result.holes_filled > 0,
            "cylinder should have fillable holes"
        );
        assert!(
            result.mesh.face_count() > orig_tri_count,
            "filled mesh should have more triangles"
        );
        assert!(result.new_triangles > 0);
    }

    // -----------------------------------------------------------------------
    // Test 13: fill_holes EarClip strategy
    // -----------------------------------------------------------------------
    #[test]
    fn test_fill_holes_earclip() {
        let mesh = cylinder_open_both_ends();
        let result = fill_holes(&mesh, PatchStrategy::EarClip);
        assert!(result.holes_filled > 0, "EarClip should fill holes");
        assert!(result.new_triangles > 0, "EarClip should add triangles");
    }

    // -----------------------------------------------------------------------
    // Test 14: fill_holes MinArea strategy
    // -----------------------------------------------------------------------
    #[test]
    fn test_fill_holes_min_area() {
        let mesh = cylinder_open_both_ends();
        let result = fill_holes(&mesh, PatchStrategy::MinArea);
        assert!(result.holes_filled > 0, "MinArea should fill holes");
        assert!(result.new_triangles > 0, "MinArea should add triangles");
    }

    // -----------------------------------------------------------------------
    // Test 15: project_polygon_2d preserves relative distances
    // -----------------------------------------------------------------------
    #[test]
    fn test_project_polygon_2d_unit_square() {
        let pts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normal = [0.0f32, 0.0, 1.0];
        let proj = project_polygon_2d(&pts, &normal);
        assert_eq!(proj.len(), 4);
        // Check that projected distances match 3D distances
        let d3d = len3(sub3(pts[1], pts[0]));
        let d2d = len2([proj[1][0] - proj[0][0], proj[1][1] - proj[0][1]]);
        assert!(
            (d3d - d2d).abs() < 1e-5,
            "projected distance should match 3D: {d3d} vs {d2d}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 16: is_ear detects ear on convex quad
    // -----------------------------------------------------------------------
    #[test]
    fn test_is_ear_convex_quad() {
        // A flat CCW square in XY plane
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let loop_verts = vec![0u32, 1, 2, 3];
        // All vertices of a convex polygon should be ears
        let ears: Vec<bool> = (0..4).map(|i| is_ear(i, &loop_verts, &positions)).collect();
        assert!(
            ears.iter().any(|&e| e),
            "convex quad should have at least one ear, got: {ears:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 17: normal_estimate for flat horizontal loop points up
    // -----------------------------------------------------------------------
    #[test]
    fn test_normal_estimate_flat_loop() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let hole = MeshHole {
            loop_vertices: vec![0, 1, 2, 3],
            area_estimate: 1.0,
        };
        let n = hole.normal_estimate(&positions);
        // Should point in +Z or -Z direction
        assert!(
            n[2].abs() > 0.9,
            "flat loop normal should point in Z, got {n:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 18: write results to /tmp/ for inspection
    // -----------------------------------------------------------------------
    #[test]
    fn test_write_results_to_tmp() {
        let mesh = cylinder_open_both_ends();
        let result = fill_holes(&mesh, PatchStrategy::Fan);

        let report = format!(
            "holes_filled: {}\nnew_vertices: {}\nnew_triangles: {}\ntotal_verts: {}\ntotal_tris: {}\n",
            result.holes_filled,
            result.new_vertices,
            result.new_triangles,
            result.mesh.vertex_count(),
            result.mesh.face_count(),
        );

        let tmp = std::env::temp_dir().join("mesh_patch_report.txt");
        std::fs::write(&tmp, &report).ok();

        // Verify the file was written
        let content = std::fs::read_to_string(&tmp).expect("should succeed");
        assert!(content.contains("holes_filled"));
    }
}
