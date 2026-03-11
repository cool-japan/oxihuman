// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Isotropic remeshing via iterative edge operations:
//! split long edges, collapse short edges, flip edges for valence, tangential smoothing.

use std::collections::{HashMap, HashSet};

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ── math helpers ──────────────────────────────────────────────────────────────

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn vec3_midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    vec3_scale(vec3_add(a, b), 0.5)
}

#[inline]
fn edge_length(positions: &[[f32; 3]], a: usize, b: usize) -> f32 {
    vec3_len(vec3_sub(positions[b], positions[a]))
}

// ── public types ──────────────────────────────────────────────────────────────

/// Parameters controlling isotropic remeshing.
pub struct RemeshParams {
    /// Desired edge length after remeshing.
    pub target_edge_length: f32,
    /// Number of remesh iterations (default 5).
    pub iterations: usize,
    /// Tangential smoothing weight in 0..1 (default 0.5).
    pub smooth_weight: f32,
    /// When true, boundary vertices are not moved during smoothing.
    pub preserve_boundary: bool,
}

impl Default for RemeshParams {
    fn default() -> Self {
        Self {
            target_edge_length: 0.05,
            iterations: 5,
            smooth_weight: 0.5,
            preserve_boundary: true,
        }
    }
}

/// Statistics returned from a remesh operation.
pub struct RemeshResult {
    /// The remeshed mesh.
    pub mesh: MeshBuffers,
    /// Number of vertices added (from edge splits).
    pub vertices_added: usize,
    /// Number of vertices removed (from edge collapses).
    pub vertices_removed: usize,
    /// Total number of edges flipped for valence improvement.
    pub edges_flipped: usize,
    /// Actual number of iterations performed.
    pub iterations_run: usize,
}

// ── public API ────────────────────────────────────────────────────────────────

/// Compute the mean edge length of a triangulated mesh.
pub fn compute_mean_edge_length(mesh: &MeshBuffers) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0usize;
    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a >= mesh.positions.len() || b >= mesh.positions.len() || c >= mesh.positions.len() {
            continue;
        }
        total += edge_length(&mesh.positions, a, b);
        total += edge_length(&mesh.positions, b, c);
        total += edge_length(&mesh.positions, a, c);
        count += 3;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// Return the mean edge length as the target edge length heuristic.
pub fn compute_target_edge_length(mesh: &MeshBuffers) -> f32 {
    compute_mean_edge_length(mesh)
}

/// Split all edges longer than `max_len`, inserting a midpoint vertex.
pub fn split_long_edges(mesh: &MeshBuffers, max_len: f32) -> MeshBuffers {
    let mut positions = mesh.positions.clone();
    let mut uvs = mesh.uvs.clone();
    let mut new_indices: Vec<u32> = Vec::with_capacity(mesh.indices.len() * 2);

    // Map from canonical edge (min, max) to midpoint vertex index
    let mut edge_midpoint: HashMap<(u32, u32), u32> = HashMap::new();

    let get_or_create_midpoint = |positions: &mut Vec<[f32; 3]>,
                                  uvs: &mut Vec<[f32; 2]>,
                                  edge_midpoint: &mut HashMap<(u32, u32), u32>,
                                  a: u32,
                                  b: u32|
     -> u32 {
        let key = (a.min(b), a.max(b));
        if let Some(&mid) = edge_midpoint.get(&key) {
            return mid;
        }
        let pa = positions[a as usize];
        let pb = positions[b as usize];
        let mid_pos = vec3_midpoint(pa, pb);
        let mid_idx = positions.len() as u32;
        positions.push(mid_pos);

        // Interpolate UV if present
        if !uvs.is_empty() && (a as usize) < uvs.len() && (b as usize) < uvs.len() {
            let ua = uvs[a as usize];
            let ub = uvs[b as usize];
            uvs.push([(ua[0] + ub[0]) * 0.5, (ua[1] + ub[1]) * 0.5]);
        }

        edge_midpoint.insert(key, mid_idx);
        mid_idx
    };

    for tri in mesh.indices.chunks_exact(3) {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        if v0 as usize >= mesh.positions.len()
            || v1 as usize >= mesh.positions.len()
            || v2 as usize >= mesh.positions.len()
        {
            continue;
        }

        let l01 = edge_length(&positions, v0 as usize, v1 as usize);
        let l12 = edge_length(&positions, v1 as usize, v2 as usize);
        let l20 = edge_length(&positions, v2 as usize, v0 as usize);

        let split01 = l01 > max_len;
        let split12 = l12 > max_len;
        let split20 = l20 > max_len;

        match (split01, split12, split20) {
            (false, false, false) => {
                // No split needed
                new_indices.extend_from_slice(&[v0, v1, v2]);
            }
            (true, false, false) => {
                let m01 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v0, v1);
                new_indices.extend_from_slice(&[v0, m01, v2]);
                new_indices.extend_from_slice(&[m01, v1, v2]);
            }
            (false, true, false) => {
                let m12 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v1, v2);
                new_indices.extend_from_slice(&[v0, v1, m12]);
                new_indices.extend_from_slice(&[v0, m12, v2]);
            }
            (false, false, true) => {
                let m20 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v2, v0);
                new_indices.extend_from_slice(&[v0, v1, m20]);
                new_indices.extend_from_slice(&[m20, v1, v2]);
            }
            (true, true, false) => {
                let m01 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v0, v1);
                let m12 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v1, v2);
                new_indices.extend_from_slice(&[v0, m01, v2]);
                new_indices.extend_from_slice(&[m01, m12, v2]);
                new_indices.extend_from_slice(&[m01, v1, m12]);
            }
            (true, false, true) => {
                let m01 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v0, v1);
                let m20 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v2, v0);
                new_indices.extend_from_slice(&[v0, m01, m20]);
                new_indices.extend_from_slice(&[m01, v1, v2]);
                new_indices.extend_from_slice(&[m20, m01, v2]);
            }
            (false, true, true) => {
                let m12 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v1, v2);
                let m20 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v2, v0);
                new_indices.extend_from_slice(&[v0, v1, m20]);
                new_indices.extend_from_slice(&[v1, m12, m20]);
                new_indices.extend_from_slice(&[m12, v2, m20]);
            }
            (true, true, true) => {
                // All three edges split — 4-way subdivision
                let m01 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v0, v1);
                let m12 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v1, v2);
                let m20 =
                    get_or_create_midpoint(&mut positions, &mut uvs, &mut edge_midpoint, v2, v0);
                new_indices.extend_from_slice(&[v0, m01, m20]);
                new_indices.extend_from_slice(&[m01, v1, m12]);
                new_indices.extend_from_slice(&[m20, m12, v2]);
                new_indices.extend_from_slice(&[m01, m12, m20]);
            }
        }
    }

    let nv = positions.len();
    // Pad normals and tangents for new vertices
    let normals = vec![[0.0f32, 1.0, 0.0]; nv];
    let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; nv];

    // If uvs grew shorter than positions, pad with zeros
    while uvs.len() < nv {
        uvs.push([0.0, 0.0]);
    }

    let mut out = MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices: new_indices,
        colors: None,
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut out);
    out
}

/// Collapse all edges shorter than `min_len`, merging endpoints to their midpoint.
pub fn collapse_short_edges(mesh: &MeshBuffers, min_len: f32) -> MeshBuffers {
    let nv = mesh.positions.len();
    // Union-find for vertex merging
    let mut rep: Vec<u32> = (0..nv as u32).collect();

    fn find_root(rep: &mut [u32], mut v: u32) -> u32 {
        while rep[v as usize] != v {
            let gp = rep[rep[v as usize] as usize];
            rep[v as usize] = gp;
            v = gp;
        }
        v
    }

    // Collect short edges
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        if a as usize >= nv || b as usize >= nv || c as usize >= nv {
            continue;
        }
        for &(ea, eb) in &[(a, b), (b, c), (a, c)] {
            let key = (ea.min(eb), ea.max(eb));
            if seen.insert(key) {
                let len = edge_length(&mesh.positions, key.0 as usize, key.1 as usize);
                if len < min_len {
                    let ra = find_root(&mut rep, key.0);
                    let rb = find_root(&mut rep, key.1);
                    if ra != rb {
                        // Merge rb -> ra
                        rep[rb as usize] = ra;
                    }
                }
            }
        }
    }

    // Compress roots
    for v in 0..nv as u32 {
        find_root(&mut rep, v);
    }

    // Build new position array (keep only roots)
    let mut root_to_new: HashMap<u32, u32> = HashMap::new();
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();

    for v in 0..nv as u32 {
        let r = find_root(&mut rep, v);
        if let std::collections::hash_map::Entry::Vacant(e) = root_to_new.entry(r) {
            let idx = new_positions.len() as u32;
            e.insert(idx);
            new_positions.push(mesh.positions[r as usize]);
            if !mesh.uvs.is_empty() && (r as usize) < mesh.uvs.len() {
                new_uvs.push(mesh.uvs[r as usize]);
            }
        }
    }

    // Rebuild face list, discarding degenerate faces
    let mut new_indices: Vec<u32> = Vec::with_capacity(mesh.indices.len());
    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        if a as usize >= nv || b as usize >= nv || c as usize >= nv {
            continue;
        }
        let ra = find_root(&mut rep, a);
        let rb = find_root(&mut rep, b);
        let rc = find_root(&mut rep, c);
        if ra == rb || rb == rc || ra == rc {
            continue; // Degenerate after collapse
        }
        let na = root_to_new[&ra];
        let nb = root_to_new[&rb];
        let nc = root_to_new[&rc];
        new_indices.extend_from_slice(&[na, nb, nc]);
    }

    let nn = new_positions.len();
    while new_uvs.len() < nn {
        new_uvs.push([0.0, 0.0]);
    }

    let mut out = MeshBuffers {
        positions: new_positions,
        normals: vec![[0.0f32, 1.0, 0.0]; nn],
        tangents: vec![[1.0f32, 0.0, 0.0, 1.0]; nn],
        uvs: new_uvs,
        indices: new_indices,
        colors: None,
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut out);
    out
}

/// Flip edges that improve vertex valence toward 6 (the ideal for interior vertices).
pub fn flip_edges_for_valence(mesh: &MeshBuffers) -> MeshBuffers {
    let nv = mesh.positions.len();
    if mesh.indices.len() < 6 {
        return mesh.clone();
    }

    // Build face adjacency: for each edge, which faces share it and what is the opposite vertex
    // edge_to_faces: canonical edge -> list of (face_idx, opposite_vertex)
    let mut edge_to_faces: HashMap<(u32, u32), Vec<(usize, u32)>> = HashMap::new();
    let nf = mesh.indices.len() / 3;

    for fi in 0..nf {
        let base = fi * 3;
        let (a, b, c) = (
            mesh.indices[base],
            mesh.indices[base + 1],
            mesh.indices[base + 2],
        );
        if a as usize >= nv || b as usize >= nv || c as usize >= nv {
            continue;
        }
        for &(ea, eb, opp) in &[(a, b, c), (b, c, a), (a, c, b)] {
            let key = (ea.min(eb), ea.max(eb));
            edge_to_faces.entry(key).or_default().push((fi, opp));
        }
    }

    // Compute vertex valence
    let mut valence: Vec<i32> = vec![0; nv];
    for &idx in &mesh.indices {
        if (idx as usize) < nv {
            valence[idx as usize] += 1;
        }
    }

    // Copy face buffer we can modify
    let mut faces: Vec<[u32; 3]> = mesh
        .indices
        .chunks_exact(3)
        .map(|t| [t[0], t[1], t[2]])
        .collect();

    let mut flipped = 0usize;

    for (&(ea, eb), adj) in &edge_to_faces {
        if adj.len() != 2 {
            continue; // Boundary or non-manifold
        }
        let (fi0, opp0) = adj[0];
        let (fi1, opp1) = adj[1];

        let va = ea as usize;
        let vb = eb as usize;
        let vc = opp0 as usize;
        let vd = opp1 as usize;

        if va >= nv || vb >= nv || vc >= nv || vd >= nv {
            continue;
        }
        if vc == vd {
            continue; // Degenerate configuration
        }

        // Current deviation from target valence 6
        let dev_before = (valence[va] - 6).abs()
            + (valence[vb] - 6).abs()
            + (valence[vc] - 6).abs()
            + (valence[vd] - 6).abs();

        // After flip: ea-eb edge replaced by opp0-opp1 edge
        // va and vb each lose 1, vc and vd each gain 1
        let dev_after = (valence[va] - 1 - 6).abs()
            + (valence[vb] - 1 - 6).abs()
            + (valence[vc] + 1 - 6).abs()
            + (valence[vd] + 1 - 6).abs();

        if dev_after < dev_before {
            // Perform flip: replace the two triangles
            faces[fi0] = [opp0, ea, opp1];
            faces[fi1] = [opp0, opp1, eb];

            valence[va] -= 1;
            valence[vb] -= 1;
            valence[vc] += 1;
            valence[vd] += 1;

            flipped += 1;
        }
    }

    let _ = flipped; // used in remesh; ignored here

    let new_indices: Vec<u32> = faces.iter().flat_map(|f| f.iter().copied()).collect();

    let mut out = MeshBuffers {
        positions: mesh.positions.clone(),
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: new_indices,
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut out);
    out
}

/// Move each vertex toward the centroid of its neighbors, projected onto the tangent plane.
pub fn smooth_vertices(mesh: &MeshBuffers, weight: f32) -> MeshBuffers {
    smooth_vertices_inner(mesh, weight, false)
}

/// Internal smooth that can optionally preserve boundary vertices.
fn smooth_vertices_inner(mesh: &MeshBuffers, weight: f32, preserve_boundary: bool) -> MeshBuffers {
    let nv = mesh.positions.len();
    if nv == 0 {
        return mesh.clone();
    }

    // Build adjacency list
    let mut neighbors: Vec<Vec<u32>> = vec![Vec::new(); nv];
    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        if a as usize >= nv || b as usize >= nv || c as usize >= nv {
            continue;
        }
        for &(from, to) in &[(a, b), (a, c), (b, a), (b, c), (c, a), (c, b)] {
            let nbs = &mut neighbors[from as usize];
            if !nbs.contains(&to) {
                nbs.push(to);
            }
        }
    }

    // Find boundary vertices (edges shared by only one triangle)
    let boundary_set: HashSet<u32> = if preserve_boundary {
        let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
        for tri in mesh.indices.chunks_exact(3) {
            let (a, b, c) = (tri[0], tri[1], tri[2]);
            if a as usize >= nv || b as usize >= nv || c as usize >= nv {
                continue;
            }
            for &(ea, eb) in &[(a, b), (b, c), (a, c)] {
                let key = (ea.min(eb), ea.max(eb));
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }
        let mut bset = HashSet::new();
        for ((ea, eb), cnt) in &edge_count {
            if *cnt == 1 {
                bset.insert(*ea);
                bset.insert(*eb);
            }
        }
        bset
    } else {
        HashSet::new()
    };

    let mut new_positions = mesh.positions.clone();

    for v in 0..nv {
        if preserve_boundary && boundary_set.contains(&(v as u32)) {
            continue;
        }
        let nbs = &neighbors[v];
        if nbs.is_empty() {
            continue;
        }

        // Compute centroid of neighbors
        let mut centroid = [0.0f32; 3];
        for &nb in nbs {
            centroid = vec3_add(centroid, mesh.positions[nb as usize]);
        }
        let n = nbs.len() as f32;
        centroid = vec3_scale(centroid, 1.0 / n);

        let pos = mesh.positions[v];
        let delta = vec3_sub(centroid, pos);

        // Project delta onto tangent plane (perpendicular to vertex normal)
        let normal = mesh.normals[v];
        let normal_len_sq = vec3_dot(normal, normal);
        let tangential_delta = if normal_len_sq > 1e-10 {
            let proj = vec3_scale(normal, vec3_dot(delta, normal) / normal_len_sq);
            vec3_sub(delta, proj)
        } else {
            delta
        };

        new_positions[v] = vec3_add(pos, vec3_scale(tangential_delta, weight));
    }

    let mut out = MeshBuffers {
        positions: new_positions,
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

/// Count edges flipped in one pass over the mesh.
fn count_flipped_edges(mesh_before: &MeshBuffers, mesh_after: &MeshBuffers) -> usize {
    // Compare face sets to count differences as a proxy for flips
    let faces_before: HashSet<[u32; 3]> = mesh_before
        .indices
        .chunks_exact(3)
        .map(|t| {
            let mut f = [t[0], t[1], t[2]];
            f.sort();
            f
        })
        .collect();

    let faces_after: HashSet<[u32; 3]> = mesh_after
        .indices
        .chunks_exact(3)
        .map(|t| {
            let mut f = [t[0], t[1], t[2]];
            f.sort();
            f
        })
        .collect();

    // Each flip replaces 2 faces, so count changed faces / 2
    let changed = faces_before.symmetric_difference(&faces_after).count();
    changed / 2
}

/// Perform isotropic remeshing on `mesh` using the given parameters.
pub fn remesh(mesh: &MeshBuffers, params: &RemeshParams) -> RemeshResult {
    let target = params.target_edge_length;
    let max_len = target * 4.0 / 3.0;
    let min_len = target * 4.0 / 5.0;

    let mut current = mesh.clone();
    let mut total_added = 0usize;
    let mut total_removed = 0usize;
    let mut total_flipped = 0usize;

    for _ in 0..params.iterations {
        let before_split = current.positions.len();
        current = split_long_edges(&current, max_len);
        let after_split = current.positions.len();
        total_added += after_split.saturating_sub(before_split);

        let before_collapse = current.positions.len();
        current = collapse_short_edges(&current, min_len);
        let after_collapse = current.positions.len();
        total_removed += before_collapse.saturating_sub(after_collapse);

        let before_flip = current.clone();
        current = flip_edges_for_valence(&current);
        total_flipped += count_flipped_edges(&before_flip, &current);

        current = smooth_vertices_inner(&current, params.smooth_weight, params.preserve_boundary);
    }

    RemeshResult {
        mesh: current,
        vertices_added: total_added,
        vertices_removed: total_removed,
        edges_flipped: total_flipped,
        iterations_run: params.iterations,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Two triangles sharing an edge: (0,1,2) and (1,3,2)
    ///
    /// ```
    ///  0 ── 1
    ///  |  / |
    ///  2 ── 3
    /// ```
    fn two_tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [0.0, 1.0, 0.0], // 2
                [1.0, 1.0, 0.0], // 3
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            indices: vec![0, 1, 2, 1, 3, 2],
            has_suit: false,
        })
    }

    /// Single triangle for simple length tests (edge length = 1.0 for two edges, sqrt(2) for hypotenuse)
    fn one_tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn test_default_params() {
        let p = RemeshParams::default();
        assert!((p.target_edge_length - 0.05).abs() < 1e-6);
        assert_eq!(p.iterations, 5);
        assert!((p.smooth_weight - 0.5).abs() < 1e-6);
        assert!(p.preserve_boundary);
    }

    #[test]
    fn test_compute_mean_edge_length_triangle() {
        let mesh = one_tri_mesh();
        let mean = compute_mean_edge_length(&mesh);
        // Edges: 1.0, 1.0, sqrt(2) ≈ 1.414; mean = (1 + 1 + sqrt(2)) / 3
        let expected = (1.0 + 1.0 + 2.0f32.sqrt()) / 3.0;
        assert!(
            (mean - expected).abs() < 1e-5,
            "mean={} expected={}",
            mean,
            expected
        );
    }

    #[test]
    fn test_split_long_edges_basic() {
        let mesh = two_tri_mesh();
        let verts_before = mesh.positions.len();
        // Max_len = 0.5, so all edges of length ~1.0 and ~1.414 will be split
        let out = split_long_edges(&mesh, 0.5);
        assert!(
            out.positions.len() > verts_before,
            "split should add vertices; before={} after={}",
            verts_before,
            out.positions.len()
        );
        assert_eq!(out.indices.len() % 3, 0, "indices must be multiple of 3");
    }

    #[test]
    fn test_collapse_short_edges_basic() {
        let mesh = two_tri_mesh();
        // Min_len = 2.0 → all edges (len ~1.0) are shorter, should collapse many
        let out = collapse_short_edges(&mesh, 2.0);
        assert!(
            out.positions.len() <= mesh.positions.len(),
            "collapse should reduce or equal vertices"
        );
        assert_eq!(out.indices.len() % 3, 0, "indices must be multiple of 3");
    }

    #[test]
    fn test_remesh_simple_quad() {
        let mesh = two_tri_mesh();
        let params = RemeshParams {
            target_edge_length: 0.5,
            iterations: 2,
            smooth_weight: 0.3,
            preserve_boundary: false,
        };
        let result = remesh(&mesh, &params);
        assert_eq!(result.iterations_run, 2);
        assert_eq!(result.mesh.indices.len() % 3, 0);
        assert!(!result.mesh.positions.is_empty());
    }

    #[test]
    fn test_remesh_iterations() {
        let mesh = two_tri_mesh();
        let params = RemeshParams {
            target_edge_length: 0.4,
            iterations: 3,
            smooth_weight: 0.5,
            preserve_boundary: true,
        };
        let result = remesh(&mesh, &params);
        assert_eq!(result.iterations_run, 3, "should run exactly 3 iterations");
    }

    #[test]
    fn test_smooth_vertices() {
        let mesh = two_tri_mesh();
        // After smoothing, vertex positions should change (interior vertices)
        let smoothed = smooth_vertices(&mesh, 0.5);
        assert_eq!(smoothed.positions.len(), mesh.positions.len());
        assert_eq!(smoothed.indices.len(), mesh.indices.len());
        // Normals should still be finite
        for n in &smoothed.normals {
            assert!(!n[0].is_nan() && !n[1].is_nan() && !n[2].is_nan());
        }
    }

    #[test]
    fn test_compute_target_edge_length() {
        let mesh = one_tri_mesh();
        let target = compute_target_edge_length(&mesh);
        let mean = compute_mean_edge_length(&mesh);
        assert!(
            (target - mean).abs() < 1e-6,
            "target should equal mean edge length"
        );
    }

    #[test]
    fn test_remesh_result_fields() {
        let mesh = two_tri_mesh();
        let params = RemeshParams {
            target_edge_length: 0.3,
            iterations: 2,
            smooth_weight: 0.5,
            preserve_boundary: true,
        };
        let result = remesh(&mesh, &params);
        // All fields should be accessible and non-panicking
        let _added = result.vertices_added;
        let _removed = result.vertices_removed;
        let _flipped = result.edges_flipped;
        assert_eq!(result.iterations_run, 2);
        assert!(!result.mesh.positions.is_empty());
    }

    #[test]
    fn test_flip_edges_for_valence() {
        let mesh = two_tri_mesh();
        let flipped = flip_edges_for_valence(&mesh);
        // The face count must stay the same (flips don't add/remove faces)
        assert_eq!(
            flipped.indices.len(),
            mesh.indices.len(),
            "flip must preserve face count"
        );
        assert_eq!(
            flipped.positions.len(),
            mesh.positions.len(),
            "flip must preserve vertex count"
        );
    }

    #[test]
    fn test_split_preserves_face_count_below_threshold() {
        let mesh = two_tri_mesh();
        // max_len much larger than any edge → no splits
        let out = split_long_edges(&mesh, 100.0);
        assert_eq!(
            out.indices.len(),
            mesh.indices.len(),
            "no splits when threshold is very large"
        );
        assert_eq!(out.positions.len(), mesh.positions.len());
    }

    #[test]
    fn test_collapse_no_op_above_threshold() {
        let mesh = two_tri_mesh();
        // min_len = 0.0 → nothing is shorter than 0, no collapses
        let out = collapse_short_edges(&mesh, 0.0);
        assert_eq!(
            out.positions.len(),
            mesh.positions.len(),
            "no collapses when threshold is zero"
        );
    }
}
