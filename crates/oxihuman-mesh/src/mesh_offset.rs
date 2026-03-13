// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh offset: push vertices along their normals by a given distance.
//!
//! Provides uniform offset, per-vertex variable offset, shrink-wrap onto a
//! target mesh, and convenience grow/shrink helpers, plus a double-sided
//! shell builder.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
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

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Parameters for a uniform normal-direction offset.
pub struct OffsetParams {
    /// Uniform offset distance (positive = outward along normal).
    pub distance: f32,
    /// Recompute normals after the offset has been applied.
    pub recompute_normals: bool,
    /// Basic self-intersection avoidance: clamp per-vertex offset so it does
    /// not exceed half the shortest incident edge length.
    pub clamp_self_intersect: bool,
}

/// Result returned by offset operations.
pub struct OffsetResult {
    /// The offset mesh.
    pub mesh: MeshBuffers,
    /// Smallest per-vertex displacement magnitude actually applied.
    pub min_offset: f32,
    /// Largest per-vertex displacement magnitude actually applied.
    pub max_offset: f32,
    /// Mean per-vertex displacement magnitude.
    pub mean_offset: f32,
}

// ---------------------------------------------------------------------------
// Internal: build per-vertex shortest-edge map for clamping
// ---------------------------------------------------------------------------

fn shortest_incident_edge(mesh: &MeshBuffers) -> Vec<f32> {
    let n = mesh.positions.len();
    let mut min_len = vec![f32::MAX; n];

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }
        let e01 = len3(sub3(mesh.positions[i1], mesh.positions[i0]));
        let e12 = len3(sub3(mesh.positions[i2], mesh.positions[i1]));
        let e20 = len3(sub3(mesh.positions[i0], mesh.positions[i2]));

        min_len[i0] = min_len[i0].min(e01).min(e20);
        min_len[i1] = min_len[i1].min(e01).min(e12);
        min_len[i2] = min_len[i2].min(e12).min(e20);
    }

    // Vertices with no incident edges get a generous default
    for v in &mut min_len {
        if *v == f32::MAX {
            *v = 1.0;
        }
    }
    min_len
}

// ---------------------------------------------------------------------------
// Core: offset_mesh (uniform)
// ---------------------------------------------------------------------------

/// Move each vertex by `normal * distance`.
///
/// If `params.clamp_self_intersect` is true the displacement for vertex *i* is
/// clamped to `min(distance, shortest_incident_edge[i] / 2)` in magnitude so
/// that obvious self-intersections are suppressed.
pub fn offset_mesh(mesh: &MeshBuffers, params: &OffsetParams) -> OffsetResult {
    let n = mesh.positions.len();
    let min_edges = if params.clamp_self_intersect {
        shortest_incident_edge(mesh)
    } else {
        vec![f32::MAX; n]
    };

    let mut positions = mesh.positions.clone();
    let mut actual: Vec<f32> = Vec::with_capacity(n);

    for i in 0..n {
        let norm = normalize3(mesh.normals[i]);
        let max_d = min_edges[i] * 0.5;
        let d = if params.clamp_self_intersect {
            params.distance.clamp(-max_d, max_d)
        } else {
            params.distance
        };
        positions[i] = add3(positions[i], scale3(norm, d));
        actual.push(d.abs());
    }

    let min_offset = actual.iter().cloned().fold(f32::MAX, f32::min);
    let max_offset = actual.iter().cloned().fold(f32::MIN, f32::max);
    let mean_offset = if n == 0 {
        0.0
    } else {
        actual.iter().sum::<f32>() / n as f32
    };

    let mut out = MeshBuffers {
        positions,
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: mesh.indices.clone(),
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    };

    if params.recompute_normals {
        compute_normals(&mut out);
    }

    OffsetResult {
        mesh: out,
        min_offset,
        max_offset,
        mean_offset,
    }
}

// ---------------------------------------------------------------------------
// Core: offset_mesh_variable (per-vertex distances)
// ---------------------------------------------------------------------------

/// Move vertex *i* by `normal[i] * distances[i]`.
///
/// If `distances` is shorter than the vertex count the remainder defaults to
/// `0.0`.  Normals are recomputed when `recompute_normals` is `true`.
pub fn offset_mesh_variable(
    mesh: &MeshBuffers,
    distances: &[f32],
    recompute_normals: bool,
) -> OffsetResult {
    let n = mesh.positions.len();
    let mut positions = mesh.positions.clone();
    let mut actual: Vec<f32> = Vec::with_capacity(n);

    for (i, pos) in positions.iter_mut().enumerate() {
        let d = distances.get(i).copied().unwrap_or(0.0);
        let norm = normalize3(mesh.normals[i]);
        *pos = add3(*pos, scale3(norm, d));
        actual.push(d.abs());
    }

    let min_offset = actual.iter().cloned().fold(f32::MAX, f32::min);
    let max_offset = actual.iter().cloned().fold(f32::MIN, f32::max);
    let mean_offset = if n == 0 {
        0.0
    } else {
        actual.iter().sum::<f32>() / n as f32
    };

    let mut out = MeshBuffers {
        positions,
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: mesh.indices.clone(),
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    };

    if recompute_normals {
        compute_normals(&mut out);
    }

    OffsetResult {
        mesh: out,
        min_offset,
        max_offset,
        mean_offset,
    }
}

// ---------------------------------------------------------------------------
// Helper: closest point on a triangle (barycentric-clamped projection)
// ---------------------------------------------------------------------------

/// Return the closest point on triangle (a, b, c) to query point p.
pub fn closest_point_on_triangle(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ap = sub3(p, a);

    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = sub3(p, b);
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return add3(a, scale3(ab, v));
    }

    let cp = sub3(p, c);
    let d5 = dot3(ab, cp);
    let d6 = dot3(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return add3(a, scale3(ac, w));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return add3(b, scale3(sub3(c, b), w));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    add3(a, add3(scale3(ab, v), scale3(ac, w)))
}

// ---------------------------------------------------------------------------
// shrink_wrap
// ---------------------------------------------------------------------------

/// Move each source vertex toward the nearest point on `target` surface by
/// `factor` (0 = no move, 1 = full snap to target surface).
///
/// Nearest-point search is brute-force over all target triangles.
/// Normals are recomputed after the wrap.
pub fn shrink_wrap(mesh: &MeshBuffers, target: &MeshBuffers, factor: f32) -> MeshBuffers {
    let factor = factor.clamp(0.0, 1.0);
    let tpos = &target.positions;
    let tidx = &target.indices;
    let n_tris = tidx.len() / 3;

    let mut positions = mesh.positions.clone();

    for p in positions.iter_mut() {
        let mut best_dist2 = f32::MAX;
        let mut best_pt = *p;

        for ti in 0..n_tris {
            let i0 = tidx[ti * 3] as usize;
            let i1 = tidx[ti * 3 + 1] as usize;
            let i2 = tidx[ti * 3 + 2] as usize;
            if i0 >= tpos.len() || i1 >= tpos.len() || i2 >= tpos.len() {
                continue;
            }
            let cp = closest_point_on_triangle(*p, tpos[i0], tpos[i1], tpos[i2]);
            let diff = sub3(cp, *p);
            let d2 = dot3(diff, diff);
            if d2 < best_dist2 {
                best_dist2 = d2;
                best_pt = cp;
            }
        }

        // Interpolate: p + factor * (best_pt - p)
        let delta = sub3(best_pt, *p);
        *p = add3(*p, scale3(delta, factor));
    }

    let mut out = MeshBuffers {
        positions,
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: mesh.indices.clone(),
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut out);
    out
}

// ---------------------------------------------------------------------------
// Convenience wrappers
// ---------------------------------------------------------------------------

/// Offset all vertices outward (positive distance along normals).
pub fn grow_mesh(mesh: &MeshBuffers, distance: f32) -> MeshBuffers {
    let params = OffsetParams {
        distance: distance.abs(),
        recompute_normals: true,
        clamp_self_intersect: false,
    };
    offset_mesh(mesh, &params).mesh
}

/// Offset all vertices inward (negative distance, i.e. against normals).
pub fn shrink_mesh(mesh: &MeshBuffers, distance: f32) -> MeshBuffers {
    let params = OffsetParams {
        distance: -distance.abs(),
        recompute_normals: true,
        clamp_self_intersect: false,
    };
    offset_mesh(mesh, &params).mesh
}

// ---------------------------------------------------------------------------
// shell_offset: create a double-sided shell
// ---------------------------------------------------------------------------

/// Create a double-sided shell by combining the original mesh (outer) with an
/// inward-offset copy (inner, with flipped winding/normals).
///
/// If the mesh has open boundary edges the function stitches quad strips
/// connecting the outer and inner boundary loops to close the shell.
#[allow(clippy::too_many_arguments)]
pub fn shell_offset(mesh: &MeshBuffers, thickness: f32) -> MeshBuffers {
    // ---- outer layer (original) ----
    let outer_pos = mesh.positions.clone();
    let outer_norm = mesh.normals.clone();
    let outer_uv = mesh.uvs.clone();
    let outer_idx = mesh.indices.clone();
    let n_outer = outer_pos.len();

    // ---- inner layer (inward offset + flipped winding) ----
    let t = thickness.abs();
    let mut inner_pos: Vec<[f32; 3]> = outer_pos
        .iter()
        .zip(outer_norm.iter())
        .map(|(p, n)| {
            let nrm = normalize3(*n);
            sub3(*p, scale3(nrm, t))
        })
        .collect();
    let inner_uv = outer_uv.clone();

    // Flip winding for the inner shell (reverse each triangle)
    let inner_idx: Vec<u32> = outer_idx
        .chunks_exact(3)
        .flat_map(|tri| {
            [
                tri[0] + n_outer as u32,
                tri[2] + n_outer as u32,
                tri[1] + n_outer as u32,
            ]
        })
        .collect();

    // Merge positions + normals + uvs
    let mut all_pos = outer_pos.clone();
    all_pos.append(&mut inner_pos);

    let inner_norm: Vec<[f32; 3]> = outer_norm.iter().map(|n| scale3(*n, -1.0)).collect();
    let mut all_norm = outer_norm.clone();
    all_norm.extend_from_slice(&inner_norm);

    let mut all_uv = outer_uv.clone();
    all_uv.extend_from_slice(&inner_uv);

    let mut all_idx = outer_idx.clone();
    all_idx.extend_from_slice(&inner_idx);

    // ---- stitch boundary edges if mesh has open boundaries ----
    let boundary = find_open_boundary_edges(mesh);
    if !boundary.is_empty() {
        // Each boundary edge (v0, v1) generates a quad (two triangles) connecting
        // outer edge to inner edge.
        for (v0, v1) in boundary {
            let o0 = v0 as u32;
            let o1 = v1 as u32;
            let i0 = v0 as u32 + n_outer as u32;
            let i1 = v1 as u32 + n_outer as u32;
            // Quad: o0, o1, i1, i0
            all_idx.extend_from_slice(&[o0, o1, i1]);
            all_idx.extend_from_slice(&[o0, i1, i0]);
        }
    }

    let mut out = MeshBuffers {
        positions: all_pos,
        normals: all_norm,
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; all_pos_len_placeholder()],
        uvs: all_uv,
        indices: all_idx,
        colors: None,
        has_suit: mesh.has_suit,
    };

    // Fix tangents length
    let nv = out.positions.len();
    out.tangents = vec![[1.0, 0.0, 0.0, 1.0]; nv];

    compute_normals(&mut out);
    out
}

// We need a dummy to avoid a "use before init" compile error above.
// Real length is set right after construction.
#[inline(always)]
fn all_pos_len_placeholder() -> usize {
    0
}

// ---------------------------------------------------------------------------
// Internal: find open boundary edges (edges belonging to only one triangle)
// ---------------------------------------------------------------------------

fn find_open_boundary_edges(mesh: &MeshBuffers) -> Vec<(usize, usize)> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let edges = [(a, b), (b, c), (c, a)];
        for (u, v) in edges {
            let key = if u < v { (u, v) } else { (v, u) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    edge_count
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|(k, _)| k)
        .collect()
}

// ---------------------------------------------------------------------------
// New API: MeshOffsetConfig / MeshOffsetResult and related helpers
// ---------------------------------------------------------------------------

/// Configuration for the new-style mesh offset API.
pub struct MeshOffsetConfig {
    /// Uniform offset distance (positive = outward along normals).
    pub distance: f32,
    /// Normal smoothing iterations before offsetting (0 = no smoothing).
    pub smooth_iterations: u32,
    /// Clamp per-vertex offset to half the shortest incident edge.
    pub clamp_to_edge: bool,
    /// Quality threshold below which the result is flagged as low quality.
    pub quality_threshold: f32,
}

/// Result returned by the new-style offset API.
pub struct MeshOffsetResult {
    /// The offset vertex positions (same count as input).
    pub positions: Vec<[f32; 3]>,
    /// The (possibly smoothed) offset normals used.
    pub normals: Vec<[f32; 3]>,
    /// Number of vertices in the result.
    pub vertex_count: usize,
    /// Bounding box: [min_x, min_y, min_z, max_x, max_y, max_z].
    pub aabb: [f32; 6],
    /// Overall quality score in [0, 1].
    pub quality_score: f32,
    /// True if any face was detected as flipped (self-intersection indicator).
    pub has_self_intersections: bool,
}

/// Return a `MeshOffsetConfig` with sensible defaults.
pub fn default_offset_config() -> MeshOffsetConfig {
    MeshOffsetConfig {
        distance: 0.01,
        smooth_iterations: 1,
        clamp_to_edge: true,
        quality_threshold: 0.5,
    }
}

/// Compute smoothed per-vertex normals suitable for offsetting.
///
/// Each normal is averaged over its neighbouring face normals weighted by
/// face area, then renormalised.  This reduces creasing artifacts.
pub fn compute_offset_normals(
    positions: &[[f32; 3]],
    indices: &[u32],
    iterations: u32,
) -> Vec<[f32; 3]> {
    let n = positions.len();
    if n == 0 {
        return vec![];
    }

    // Compute face-area-weighted normals from scratch
    let mut acc: Vec<[f32; 3]> = vec![[0.0; 3]; n];

    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }
        let ab = sub3(positions[i1], positions[i0]);
        let ac = sub3(positions[i2], positions[i0]);
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ]; // area-weighted normal
        for &vi in &[i0, i1, i2] {
            acc[vi] = add3(acc[vi], cross);
        }
    }

    let mut normals: Vec<[f32; 3]> = acc.iter().map(|&n| normalize3(n)).collect();

    // Iterative Laplacian smoothing of normals
    if iterations > 0 {
        // Build adjacency: vertex -> set of adjacent vertex normals
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
        for tri in indices.chunks_exact(3) {
            let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if i0 >= n || i1 >= n || i2 >= n {
                continue;
            }
            adj[i0].push(i1);
            adj[i0].push(i2);
            adj[i1].push(i0);
            adj[i1].push(i2);
            adj[i2].push(i0);
            adj[i2].push(i1);
        }
        for _ in 0..iterations {
            let prev = normals.clone();
            for i in 0..n {
                if adj[i].is_empty() {
                    continue;
                }
                let mut sum = prev[i];
                for &j in &adj[i] {
                    sum = add3(sum, prev[j]);
                }
                normals[i] = normalize3(sum);
            }
        }
    }

    normals
}

/// Smooth existing normals over the mesh adjacency for `iterations` passes.
pub fn smooth_offset_normals(
    normals: &[[f32; 3]],
    indices: &[u32],
    iterations: u32,
) -> Vec<[f32; 3]> {
    let n = normals.len();
    if n == 0 || iterations == 0 {
        return normals.to_vec();
    }

    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }
        adj[i0].push(i1);
        adj[i0].push(i2);
        adj[i1].push(i0);
        adj[i1].push(i2);
        adj[i2].push(i0);
        adj[i2].push(i1);
    }

    let mut out = normals.to_vec();
    for _ in 0..iterations {
        let prev = out.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut sum = prev[i];
            for &j in &adj[i] {
                sum = add3(sum, prev[j]);
            }
            out[i] = normalize3(sum);
        }
    }
    out
}

/// Detect self-intersecting faces after offsetting by checking for sign-flipped
/// face normals compared to the original.
///
/// Returns the indices of faces whose winding reversed.
pub fn check_self_intersection(
    original_pos: &[[f32; 3]],
    offset_pos: &[[f32; 3]],
    indices: &[u32],
) -> Vec<usize> {
    let n_orig = original_pos.len();
    let n_off = offset_pos.len();
    let mut flipped = vec![];

    for (fi, tri) in indices.chunks_exact(3).enumerate() {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n_orig || i1 >= n_orig || i2 >= n_orig {
            continue;
        }
        if i0 >= n_off || i1 >= n_off || i2 >= n_off {
            continue;
        }

        let orig_n = face_normal_raw(original_pos[i0], original_pos[i1], original_pos[i2]);
        let off_n = face_normal_raw(offset_pos[i0], offset_pos[i1], offset_pos[i2]);
        if dot3(orig_n, off_n) < 0.0 {
            flipped.push(fi);
        }
    }
    flipped
}

/// Compute the number of vertices in a potential offset result (always same as input).
pub fn offset_vertex_count(positions: &[[f32; 3]]) -> usize {
    positions.len()
}

/// Clamp an offset distance to `[-max_dist, max_dist]`.
pub fn clamp_offset_distance(distance: f32, max_dist: f32) -> f32 {
    distance.clamp(-max_dist.abs(), max_dist.abs())
}

/// Compute the axis-aligned bounding box of offset positions.
///
/// Returns `[min_x, min_y, min_z, max_x, max_y, max_z]`.
pub fn offset_bounding_box(positions: &[[f32; 3]]) -> [f32; 6] {
    if positions.is_empty() {
        return [0.0; 6];
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for &p in positions.iter().skip(1) {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    [mn[0], mn[1], mn[2], mx[0], mx[1], mx[2]]
}

/// Apply a signed offset: positive moves outward, negative moves inward.
///
/// Normals must be pre-normalised.
pub fn signed_offset(positions: &[[f32; 3]], normals: &[[f32; 3]], distance: f32) -> Vec<[f32; 3]> {
    positions
        .iter()
        .zip(normals.iter())
        .map(|(&p, &n)| {
            let nrm = normalize3(n);
            add3(p, scale3(nrm, distance))
        })
        .collect()
}

/// Compute a quality score for the offset result in [0, 1].
///
/// Score is reduced by the fraction of self-intersecting faces and by large
/// deviations from the requested distance.
pub fn offset_quality_score(
    original_pos: &[[f32; 3]],
    offset_pos: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
    requested_distance: f32,
) -> f32 {
    let n = original_pos.len().min(offset_pos.len()).min(normals.len());
    if n == 0 {
        return 1.0;
    }

    // Intersection penalty
    let flipped = check_self_intersection(original_pos, offset_pos, indices);
    let total_faces = (indices.len() / 3).max(1);
    let intersection_penalty = flipped.len() as f32 / total_faces as f32;

    // Distance accuracy penalty
    let mut dist_err_sum = 0.0f32;
    for i in 0..n {
        let nrm = normalize3(normals[i]);
        let actual = dot3(sub3(offset_pos[i], original_pos[i]), nrm);
        dist_err_sum += (actual - requested_distance).abs();
    }
    let mean_err = dist_err_sum / n as f32;
    let dist_penalty = (mean_err / (requested_distance.abs() + 1e-6)).min(1.0);

    (1.0 - intersection_penalty * 0.5 - dist_penalty * 0.5).clamp(0.0, 1.0)
}

// Helper: raw (un-normalised) face normal
#[inline]
fn face_normal_raw(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn flat_quad() -> MeshBuffers {
        // Two triangles forming a z=0 quad, normals pointing +Z
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    fn single_tri() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    fn unit_sphere_approx() -> MeshBuffers {
        // Icosahedron-like: 6 vertices on axes, normals = positions
        let positions = vec![
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
        let normals = positions.clone();
        MeshBuffers::from_morph(MB {
            positions,
            normals,
            uvs: vec![[0.0, 0.0]; 6],
            indices: vec![0, 2, 4, 0, 4, 3, 0, 3, 5, 0, 5, 2],
            has_suit: false,
        })
    }

    // -----------------------------------------------------------------------
    // offset_mesh tests
    // -----------------------------------------------------------------------

    #[test]
    fn offset_mesh_moves_vertices_along_normals() {
        let mesh = flat_quad();
        let params = OffsetParams {
            distance: 1.0,
            recompute_normals: false,
            clamp_self_intersect: false,
        };
        let result = offset_mesh(&mesh, &params);
        for p in &result.mesh.positions {
            assert!((p[2] - 1.0).abs() < 1e-5, "z should be 1.0, got {}", p[2]);
        }
    }

    #[test]
    fn offset_mesh_negative_distance_goes_inward() {
        let mesh = flat_quad();
        let params = OffsetParams {
            distance: -0.5,
            recompute_normals: false,
            clamp_self_intersect: false,
        };
        let result = offset_mesh(&mesh, &params);
        for p in &result.mesh.positions {
            assert!((p[2] + 0.5).abs() < 1e-5, "z should be -0.5, got {}", p[2]);
        }
    }

    #[test]
    fn offset_mesh_preserves_vertex_count() {
        let mesh = flat_quad();
        let params = OffsetParams {
            distance: 0.3,
            recompute_normals: true,
            clamp_self_intersect: false,
        };
        let result = offset_mesh(&mesh, &params);
        assert_eq!(result.mesh.positions.len(), mesh.positions.len());
        assert_eq!(result.mesh.indices.len(), mesh.indices.len());
    }

    #[test]
    fn offset_mesh_statistics_correct() {
        let mesh = flat_quad();
        let params = OffsetParams {
            distance: 2.0,
            recompute_normals: false,
            clamp_self_intersect: false,
        };
        let result = offset_mesh(&mesh, &params);
        assert!((result.min_offset - 2.0).abs() < 1e-5);
        assert!((result.max_offset - 2.0).abs() < 1e-5);
        assert!((result.mean_offset - 2.0).abs() < 1e-5);
    }

    #[test]
    fn offset_mesh_zero_distance_is_identity() {
        let mesh = flat_quad();
        let params = OffsetParams {
            distance: 0.0,
            recompute_normals: false,
            clamp_self_intersect: false,
        };
        let result = offset_mesh(&mesh, &params);
        for (a, b) in mesh.positions.iter().zip(result.mesh.positions.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
            assert!((a[1] - b[1]).abs() < 1e-6);
            assert!((a[2] - b[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn offset_mesh_clamp_does_not_exceed_half_edge() {
        let mesh = flat_quad();
        // Edge length in flat_quad is 1.0; half = 0.5
        let params = OffsetParams {
            distance: 100.0,
            recompute_normals: false,
            clamp_self_intersect: true,
        };
        let result = offset_mesh(&mesh, &params);
        // All z values should be <= 0.5
        for p in &result.mesh.positions {
            assert!(p[2] <= 0.5 + 1e-5, "z={} exceeds half-edge limit", p[2]);
        }
        assert!(result.max_offset <= 0.5 + 1e-5);
    }

    // -----------------------------------------------------------------------
    // offset_mesh_variable tests
    // -----------------------------------------------------------------------

    #[test]
    fn variable_offset_per_vertex_distances() {
        let mesh = flat_quad();
        let distances = vec![0.1, 0.2, 0.3, 0.4];
        let result = offset_mesh_variable(&mesh, &distances, false);
        let expected_z = [0.1f32, 0.2, 0.3, 0.4];
        for (i, p) in result.mesh.positions.iter().enumerate() {
            assert!(
                (p[2] - expected_z[i]).abs() < 1e-5,
                "vertex {} z expected {} got {}",
                i,
                expected_z[i],
                p[2]
            );
        }
    }

    #[test]
    fn variable_offset_short_distances_fallback_to_zero() {
        let mesh = flat_quad();
        // Only 2 distances for 4 vertices
        let distances = vec![1.0, 1.0];
        let result = offset_mesh_variable(&mesh, &distances, false);
        // Vertices 2 and 3 should stay at z=0
        assert!((result.mesh.positions[2][2]).abs() < 1e-5);
        assert!((result.mesh.positions[3][2]).abs() < 1e-5);
    }

    #[test]
    fn variable_offset_statistics_reflect_per_vertex() {
        let mesh = flat_quad();
        let distances = vec![0.0, 0.0, 0.0, 4.0];
        let result = offset_mesh_variable(&mesh, &distances, false);
        assert!((result.min_offset - 0.0).abs() < 1e-5);
        assert!((result.max_offset - 4.0).abs() < 1e-5);
        assert!((result.mean_offset - 1.0).abs() < 1e-5); // (0+0+0+4)/4
    }

    // -----------------------------------------------------------------------
    // closest_point_on_triangle tests
    // -----------------------------------------------------------------------

    #[test]
    fn closest_point_on_triangle_inside() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [2.0, 0.0, 0.0];
        let c = [0.0, 2.0, 0.0];
        let p = [0.5, 0.5, 0.0];
        let cp = closest_point_on_triangle(p, a, b, c);
        assert!((cp[0] - 0.5).abs() < 1e-5);
        assert!((cp[1] - 0.5).abs() < 1e-5);
        assert!(cp[2].abs() < 1e-5);
    }

    #[test]
    fn closest_point_on_triangle_above_projects_down() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [2.0, 0.0, 0.0];
        let c = [0.0, 2.0, 0.0];
        let p = [0.5, 0.5, 5.0]; // directly above interior
        let cp = closest_point_on_triangle(p, a, b, c);
        assert!((cp[0] - 0.5).abs() < 1e-5);
        assert!((cp[1] - 0.5).abs() < 1e-5);
        assert!(cp[2].abs() < 1e-5);
    }

    #[test]
    fn closest_point_on_triangle_near_vertex() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        // Point near corner a but outside triangle
        let p = [-1.0, -1.0, 0.0];
        let cp = closest_point_on_triangle(p, a, b, c);
        // Closest should be vertex a
        assert!((cp[0] - a[0]).abs() < 1e-5);
        assert!((cp[1] - a[1]).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // shrink_wrap tests
    // -----------------------------------------------------------------------

    #[test]
    fn shrink_wrap_factor_zero_is_identity() {
        let mesh = flat_quad();
        let target = single_tri();
        let out = shrink_wrap(&mesh, &target, 0.0);
        for (a, b) in mesh.positions.iter().zip(out.positions.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-5);
            assert!((a[1] - b[1]).abs() < 1e-5);
        }
    }

    #[test]
    fn shrink_wrap_factor_one_snaps_to_target() {
        // Source: single vertex at (0.5, 0.5, 5.0)
        // Target: z=0 triangle containing (0.5, 0.5, 0) in its interior
        let source = MeshBuffers::from_morph(MB {
            positions: vec![[0.5, 0.5, 5.0]],
            normals: vec![[0.0, 0.0, -1.0]],
            uvs: vec![[0.0, 0.0]],
            indices: vec![],
            has_suit: false,
        });
        let target = MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        });
        let out = shrink_wrap(&source, &target, 1.0);
        // z should be snapped to ~0.0
        assert!(
            out.positions[0][2].abs() < 1e-4,
            "z={}",
            out.positions[0][2]
        );
    }

    // -----------------------------------------------------------------------
    // grow_mesh / shrink_mesh tests
    // -----------------------------------------------------------------------

    #[test]
    fn grow_mesh_increases_z_for_upward_normals() {
        let mesh = flat_quad();
        let out = grow_mesh(&mesh, 0.5);
        for p in &out.positions {
            assert!(p[2] > 0.4, "z={}", p[2]);
        }
    }

    #[test]
    fn shrink_mesh_decreases_z_for_upward_normals() {
        let mesh = flat_quad();
        let out = shrink_mesh(&mesh, 0.5);
        for p in &out.positions {
            assert!(p[2] < -0.4, "z={}", p[2]);
        }
    }

    // -----------------------------------------------------------------------
    // shell_offset tests
    // -----------------------------------------------------------------------

    #[test]
    fn shell_offset_doubles_vertex_count() {
        let mesh = single_tri();
        let out = shell_offset(&mesh, 0.1);
        // At minimum outer + inner = 6 vertices (might be more due to stitching)
        assert!(
            out.positions.len() >= 6,
            "got {} verts",
            out.positions.len()
        );
    }

    #[test]
    fn shell_offset_normals_recomputed() {
        let mesh = flat_quad();
        let out = shell_offset(&mesh, 0.1);
        // Normals should be non-zero
        for n in &out.normals {
            let l = len3(*n);
            assert!(l > 0.5, "degenerate normal: {:?}", n);
        }
    }

    // -----------------------------------------------------------------------
    // sphere grow sanity
    // -----------------------------------------------------------------------

    #[test]
    fn sphere_grow_radius_increases() {
        let mesh = unit_sphere_approx();
        let out = grow_mesh(&mesh, 0.5);
        for (orig, grown) in mesh.positions.iter().zip(out.positions.iter()) {
            let r_orig = len3(*orig);
            let r_grown = len3(*grown);
            assert!(
                r_grown > r_orig - 1e-4,
                "radius did not increase: {} -> {}",
                r_orig,
                r_grown
            );
        }
    }

    #[test]
    fn shell_offset_index_count_increases() {
        let mesh = flat_quad();
        let out = shell_offset(&mesh, 0.05);
        // Inner + outer + stitching should give more indices
        assert!(out.indices.len() >= mesh.indices.len() * 2);
    }

    // -----------------------------------------------------------------------
    // New API tests: MeshOffsetConfig, compute_offset_normals, etc.
    // -----------------------------------------------------------------------

    fn quad_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn quad_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    fn quad_normals_up() -> Vec<[f32; 3]> {
        vec![[0.0f32, 0.0, 1.0]; 4]
    }

    #[test]
    fn default_offset_config_distance_positive() {
        let cfg = default_offset_config();
        assert!(cfg.distance > 0.0);
    }

    #[test]
    fn default_offset_config_smooth_iterations_sensible() {
        let cfg = default_offset_config();
        assert!(cfg.smooth_iterations <= 10);
    }

    #[test]
    fn compute_offset_normals_returns_unit_vectors() {
        let pos = quad_positions();
        let idx = quad_indices();
        let normals = compute_offset_normals(&pos, &idx, 0);
        for n in &normals {
            let l = len3(*n);
            assert!(
                (l - 1.0).abs() < 0.01 || l < 1e-6,
                "normal not unit: l={}",
                l
            );
        }
    }

    #[test]
    fn compute_offset_normals_returns_same_count() {
        let pos = quad_positions();
        let idx = quad_indices();
        let normals = compute_offset_normals(&pos, &idx, 1);
        assert_eq!(normals.len(), pos.len());
    }

    #[test]
    fn smooth_offset_normals_returns_same_count() {
        let nrm = quad_normals_up();
        let idx = quad_indices();
        let out = smooth_offset_normals(&nrm, &idx, 2);
        assert_eq!(out.len(), nrm.len());
    }

    #[test]
    fn smooth_offset_normals_zero_iterations_is_identity() {
        let nrm = quad_normals_up();
        let idx = quad_indices();
        let out = smooth_offset_normals(&nrm, &idx, 0);
        for (a, b) in nrm.iter().zip(out.iter()) {
            assert!((a[2] - b[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn check_self_intersection_no_flips_on_small_offset() {
        let pos = quad_positions();
        let idx = quad_indices();
        let nrm = quad_normals_up();
        let offset_pos = signed_offset(&pos, &nrm, 0.01);
        let flipped = check_self_intersection(&pos, &offset_pos, &idx);
        assert!(flipped.is_empty(), "expected no flips, got {:?}", flipped);
    }

    #[test]
    fn offset_vertex_count_matches_positions() {
        let pos = quad_positions();
        assert_eq!(offset_vertex_count(&pos), 4);
    }

    #[test]
    fn offset_vertex_count_empty() {
        assert_eq!(offset_vertex_count(&[]), 0);
    }

    #[test]
    fn clamp_offset_distance_within_range() {
        assert!((clamp_offset_distance(5.0, 2.0) - 2.0).abs() < 1e-6);
        assert!((clamp_offset_distance(-5.0, 2.0) + 2.0).abs() < 1e-6);
        assert!((clamp_offset_distance(1.0, 2.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn offset_bounding_box_correct() {
        let pos = quad_positions();
        let aabb = offset_bounding_box(&pos);
        assert!((aabb[0] - 0.0).abs() < 1e-6); // min_x
        assert!((aabb[3] - 1.0).abs() < 1e-6); // max_x
    }

    #[test]
    fn offset_bounding_box_empty_positions() {
        let aabb = offset_bounding_box(&[]);
        assert_eq!(aabb, [0.0; 6]);
    }

    #[test]
    fn signed_offset_positive_moves_along_normal() {
        let pos = quad_positions();
        let nrm = quad_normals_up();
        let out = signed_offset(&pos, &nrm, 1.0);
        for p in &out {
            assert!((p[2] - 1.0).abs() < 1e-5, "z={}", p[2]);
        }
    }

    #[test]
    fn signed_offset_negative_moves_against_normal() {
        let pos = quad_positions();
        let nrm = quad_normals_up();
        let out = signed_offset(&pos, &nrm, -0.5);
        for p in &out {
            assert!((p[2] + 0.5).abs() < 1e-5, "z={}", p[2]);
        }
    }

    #[test]
    fn offset_quality_score_near_one_for_clean_offset() {
        let pos = quad_positions();
        let nrm = quad_normals_up();
        let idx = quad_indices();
        let offset_pos = signed_offset(&pos, &nrm, 0.1);
        let score = offset_quality_score(&pos, &offset_pos, &nrm, &idx, 0.1);
        assert!(score > 0.8, "quality score too low: {}", score);
    }

    #[test]
    fn offset_quality_score_empty_is_one() {
        let score = offset_quality_score(&[], &[], &[], &[], 0.1);
        assert!((score - 1.0).abs() < 1e-6);
    }
}
