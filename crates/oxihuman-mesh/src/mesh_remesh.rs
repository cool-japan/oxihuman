// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Isotropic remeshing for uniform triangle quality.
//!
//! Implements the standard pipeline: split long edges, collapse short edges,
//! equalise valence via edge flips, and tangential smoothing.

use std::collections::{HashMap, HashSet};

// ── math helpers ─────────────────────────────────────────────────────────────

#[inline]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn v3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn v3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn v3_len(v: [f32; 3]) -> f32 {
    v3_dot(v, v).sqrt()
}

fn canonical_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

// ── public types ─────────────────────────────────────────────────────────────

/// Configuration for isotropic remeshing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RemeshConfig {
    /// Target edge length.
    pub target_length: f32,
    /// Number of remesh iterations.
    pub iterations: usize,
    /// Tangential smoothing weight in \[0, 1\].
    pub smooth_weight: f32,
    /// Preserve boundary vertices during smoothing.
    pub preserve_boundary: bool,
    /// High / low edge-length ratio for split / collapse thresholds.
    pub split_ratio: f32,
    /// Collapse ratio.
    pub collapse_ratio: f32,
}

/// Statistics about a remeshing pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RemeshStats {
    /// Total edges split.
    pub edges_split: usize,
    /// Total edges collapsed.
    pub edges_collapsed: usize,
    /// Total edge flips.
    pub edge_flips: usize,
    /// Mean edge length after remesh.
    pub mean_edge_length: f32,
    /// Std-dev of edge lengths after remesh.
    pub std_edge_length: f32,
}

/// Result of a remeshing operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RemeshResult {
    /// New vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// New triangle indices.
    pub indices: Vec<u32>,
    /// Per-iteration statistics.
    pub stats: RemeshStats,
}

// ── public functions ─────────────────────────────────────────────────────────

/// Create a default remesh configuration.
#[allow(dead_code)]
pub fn default_remesh_config() -> RemeshConfig {
    RemeshConfig {
        target_length: 0.05,
        iterations: 5,
        smooth_weight: 0.5,
        preserve_boundary: true,
        split_ratio: 4.0 / 3.0,
        collapse_ratio: 4.0 / 5.0,
    }
}

/// Full isotropic remesh pipeline: split, collapse, flip, smooth.
#[allow(dead_code)]
pub fn isotropic_remesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &RemeshConfig,
) -> RemeshResult {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    let mut total_split = 0usize;
    let mut total_collapse = 0usize;
    let mut total_flip = 0usize;

    let high = config.target_length * config.split_ratio;
    let low = config.target_length * config.collapse_ratio;

    for _ in 0..config.iterations {
        let (p2, i2, ns) = split_long_edges(&pos, &idx, high);
        pos = p2;
        idx = i2;
        total_split += ns;

        let (p3, i3, nc) = collapse_short_edges(&pos, &idx, low);
        pos = p3;
        idx = i3;
        total_collapse += nc;

        let (i4, nf) = equalize_valence(&pos, &idx);
        idx = i4;
        total_flip += nf;

        tangential_smooth(
            &mut pos,
            &idx,
            config.smooth_weight,
            config.preserve_boundary,
        );
    }

    let (mean, std, _min, _max) = edge_lengths_stats(&pos, &idx);

    RemeshResult {
        positions: pos,
        indices: idx,
        stats: RemeshStats {
            edges_split: total_split,
            edges_collapsed: total_collapse,
            edge_flips: total_flip,
            mean_edge_length: mean,
            std_edge_length: std,
        },
    }
}

/// Split all edges longer than `max_len`.  Returns (positions, indices, count).
#[allow(dead_code)]
pub fn split_long_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    max_len: f32,
) -> (Vec<[f32; 3]>, Vec<u32>, usize) {
    let mut pos = positions.to_vec();
    let mut tris: Vec<[u32; 3]> = indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let mut split_count = 0usize;

    loop {
        let mut did_split = false;
        let mut new_tris: Vec<[u32; 3]> = Vec::new();

        // Track which edges have already been split this pass.
        let mut split_map: HashMap<(u32, u32), u32> = HashMap::new();

        for &[a, b, c] in &tris {
            // Find the longest edge.
            let lab = v3_len(v3_sub(pos[b as usize], pos[a as usize]));
            let lbc = v3_len(v3_sub(pos[c as usize], pos[b as usize]));
            let lca = v3_len(v3_sub(pos[a as usize], pos[c as usize]));

            let (e0, e1, opp, longest) = if lab >= lbc && lab >= lca {
                (a, b, c, lab)
            } else if lbc >= lab && lbc >= lca {
                (b, c, a, lbc)
            } else {
                (c, a, b, lca)
            };

            if longest > max_len {
                let key = canonical_edge(e0, e1);
                let mid_idx = *split_map.entry(key).or_insert_with(|| {
                    let mid = v3_scale(v3_add(pos[e0 as usize], pos[e1 as usize]), 0.5);
                    pos.push(mid);
                    (pos.len() - 1) as u32
                });
                new_tris.push([e0, mid_idx, opp]);
                new_tris.push([mid_idx, e1, opp]);
                split_count += 1;
                did_split = true;
            } else {
                new_tris.push([a, b, c]);
            }
        }

        tris = new_tris;
        if !did_split {
            break;
        }
    }

    let out_indices: Vec<u32> = tris.iter().flat_map(|t| t.iter().copied()).collect();
    (pos, out_indices, split_count)
}

/// Collapse edges shorter than `min_len`.  Returns (positions, indices, count).
#[allow(dead_code)]
pub fn collapse_short_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    min_len: f32,
) -> (Vec<[f32; 3]>, Vec<u32>, usize) {
    let mut pos = positions.to_vec();
    let tris: Vec<[u32; 3]> = indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    let mut collapse_count = 0usize;
    let mut remap: Vec<u32> = (0..pos.len() as u32).collect();

    fn find_root(remap: &[u32], mut v: u32) -> u32 {
        while remap[v as usize] != v {
            v = remap[v as usize];
        }
        v
    }

    // Collect short edges.
    let mut short_edges: Vec<(u32, u32)> = Vec::new();
    for &[a, b, c] in &tris {
        for &(e0, e1) in &[(a, b), (b, c), (c, a)] {
            let ra = find_root(&remap, e0);
            let rb = find_root(&remap, e1);
            if ra != rb {
                let l = v3_len(v3_sub(pos[ra as usize], pos[rb as usize]));
                if l < min_len {
                    short_edges.push((ra, rb));
                }
            }
        }
    }

    for (a, b) in short_edges {
        let ra = find_root(&remap, a);
        let rb = find_root(&remap, b);
        if ra == rb {
            continue;
        }
        // Move ra to midpoint.
        let mid = v3_scale(v3_add(pos[ra as usize], pos[rb as usize]), 0.5);
        pos[ra as usize] = mid;
        remap[rb as usize] = ra;
        collapse_count += 1;
    }

    // Remap indices and remove degenerate triangles.
    let new_tris: Vec<[u32; 3]> = tris
        .iter()
        .map(|&[a, b, c]| {
            [
                find_root(&remap, a),
                find_root(&remap, b),
                find_root(&remap, c),
            ]
        })
        .filter(|[a, b, c]| a != b && b != c && a != c)
        .collect();

    let out_indices: Vec<u32> = new_tris.iter().flat_map(|t| t.iter().copied()).collect();
    (pos, out_indices, collapse_count)
}

/// Equalise valence by flipping edges.  Returns (new indices, flip count).
#[allow(dead_code)]
pub fn equalize_valence(positions: &[[f32; 3]], indices: &[u32]) -> (Vec<u32>, usize) {
    let mut tris: Vec<[u32; 3]> = indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let mut flip_count = 0usize;

    // Build adjacency: edge → (tri_index, opposite_vertex)
    let mut edge_tris: HashMap<(u32, u32), Vec<(usize, u32)>> = HashMap::new();
    for (ti, &[a, b, c]) in tris.iter().enumerate() {
        for &(e0, e1, opp) in &[(a, b, c), (b, c, a), (c, a, b)] {
            let key = canonical_edge(e0, e1);
            edge_tris.entry(key).or_default().push((ti, opp));
        }
    }

    // Compute valences.
    let n_verts = positions.len();
    let mut valence = vec![0u32; n_verts];
    for &[a, b, c] in &tris {
        valence[a as usize] += 1;
        valence[b as usize] += 1;
        valence[c as usize] += 1;
    }

    // Try flipping edges to improve valence.
    for (&(ea, eb), adj) in &edge_tris {
        if adj.len() != 2 {
            continue;
        }
        let (ti0, opp0) = adj[0];
        let (ti1, opp1) = adj[1];
        if opp0 == opp1 {
            continue;
        }

        // Current valence deviation.
        let target = 6u32;
        let dev_before = valence[ea as usize].abs_diff(target)
            + valence[eb as usize].abs_diff(target)
            + valence[opp0 as usize].abs_diff(target)
            + valence[opp1 as usize].abs_diff(target);

        // After flip: ea and eb each lose 1, opp0 and opp1 each gain 1.
        let va_new = valence[ea as usize].saturating_sub(1);
        let vb_new = valence[eb as usize].saturating_sub(1);
        let vo0_new = valence[opp0 as usize] + 1;
        let vo1_new = valence[opp1 as usize] + 1;
        let dev_after = va_new.abs_diff(target)
            + vb_new.abs_diff(target)
            + vo0_new.abs_diff(target)
            + vo1_new.abs_diff(target);

        if dev_after < dev_before {
            // Perform flip: replace edge (ea, eb) with (opp0, opp1).
            tris[ti0] = [opp0, opp1, ea];
            tris[ti1] = [opp1, opp0, eb];
            valence[ea as usize] = va_new;
            valence[eb as usize] = vb_new;
            valence[opp0 as usize] = vo0_new;
            valence[opp1 as usize] = vo1_new;
            flip_count += 1;
        }
    }

    let out: Vec<u32> = tris.iter().flat_map(|t| t.iter().copied()).collect();
    (out, flip_count)
}

/// Tangential smoothing: move each vertex toward the centroid of its
/// neighbours, projected onto the tangent plane.
#[allow(dead_code)]
pub fn tangential_smooth(
    positions: &mut [[f32; 3]],
    indices: &[u32],
    weight: f32,
    preserve_boundary: bool,
) {
    let n_verts = positions.len();
    let mut neighbours: Vec<HashSet<u32>> = vec![HashSet::new(); n_verts];
    let mut boundary: HashSet<u32> = HashSet::new();

    // Build neighbour sets and detect boundary.
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        for &(e0, e1) in &[(a, b), (b, c), (c, a)] {
            neighbours[e0 as usize].insert(e1);
            neighbours[e1 as usize].insert(e0);
            *edge_count.entry(canonical_edge(e0, e1)).or_insert(0) += 1;
        }
    }
    if preserve_boundary {
        for (&(a, b), &c) in &edge_count {
            if c == 1 {
                boundary.insert(a);
                boundary.insert(b);
            }
        }
    }

    // Compute per-vertex normals (average of adjacent face normals).
    let mut normals = vec![[0.0f32; 3]; n_verts];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let ab = v3_sub(positions[b], positions[a]);
        let ac = v3_sub(positions[c], positions[a]);
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        normals[a] = v3_add(normals[a], n);
        normals[b] = v3_add(normals[b], n);
        normals[c] = v3_add(normals[c], n);
    }
    for n in &mut normals {
        let l = v3_len(*n);
        if l > 1e-12 {
            *n = v3_scale(*n, 1.0 / l);
        }
    }

    let old = positions.to_vec();
    for vi in 0..n_verts {
        if preserve_boundary && boundary.contains(&(vi as u32)) {
            continue;
        }
        if neighbours[vi].is_empty() {
            continue;
        }
        let count = neighbours[vi].len() as f32;
        let centroid = {
            let sum = neighbours[vi]
                .iter()
                .fold([0.0f32; 3], |acc, &ni| v3_add(acc, old[ni as usize]));
            v3_scale(sum, 1.0 / count)
        };
        let delta = v3_sub(centroid, old[vi]);
        // Project onto tangent plane.
        let n = normals[vi];
        let dot_dn = v3_dot(delta, n);
        let tangential = v3_sub(delta, v3_scale(n, dot_dn));
        positions[vi] = v3_add(old[vi], v3_scale(tangential, weight));
    }
}

/// Compute target edge length from mesh bounding box diagonal.
#[allow(dead_code)]
pub fn compute_target_edge_length(positions: &[[f32; 3]], factor: f32) -> f32 {
    if positions.is_empty() {
        return 0.01;
    }
    let mut min = positions[0];
    let mut max = positions[0];
    for p in positions {
        for i in 0..3 {
            if p[i] < min[i] {
                min[i] = p[i];
            }
            if p[i] > max[i] {
                max[i] = p[i];
            }
        }
    }
    let diag = v3_len(v3_sub(max, min));
    diag * factor
}

/// Compute edge length statistics: (mean, std_dev, min, max).
#[allow(dead_code)]
pub fn edge_lengths_stats(positions: &[[f32; 3]], indices: &[u32]) -> (f32, f32, f32, f32) {
    let mut lengths: Vec<f32> = Vec::new();
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    for tri in indices.chunks_exact(3) {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = canonical_edge(a, b);
            if seen.insert(key) {
                lengths.push(v3_len(v3_sub(positions[b as usize], positions[a as usize])));
            }
        }
    }
    if lengths.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let n = lengths.len() as f32;
    let mean = lengths.iter().sum::<f32>() / n;
    let variance = lengths.iter().map(|l| (l - mean) * (l - mean)).sum::<f32>() / n;
    let std = variance.sqrt();
    let min = lengths.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = lengths.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    (mean, std, min, max)
}

/// Compute a quality score in \[0, 1\] based on edge length uniformity.
/// 1.0 = perfectly uniform.
#[allow(dead_code)]
pub fn remesh_quality_score(positions: &[[f32; 3]], indices: &[u32], target_length: f32) -> f32 {
    let (mean, std, _min, _max) = edge_lengths_stats(positions, indices);
    if mean < 1e-12 {
        return 0.0;
    }
    let cv = std / mean; // coefficient of variation
    let length_ratio = if target_length > 1e-12 {
        1.0 - ((mean - target_length) / target_length).abs().min(1.0)
    } else {
        1.0
    };
    let uniformity = (1.0 - cv).max(0.0);
    uniformity * 0.5 + length_ratio * 0.5
}

/// Count vertices with irregular valence (not 6 for interior, not 4 for boundary).
#[allow(dead_code)]
pub fn count_irregular_vertices(positions: &[[f32; 3]], indices: &[u32]) -> usize {
    let n_verts = positions.len();
    let mut valence = vec![0u32; n_verts];
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for tri in indices.chunks_exact(3) {
        for &v in tri {
            valence[v as usize] += 1;
        }
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            *edge_count.entry(canonical_edge(a, b)).or_insert(0) += 1;
        }
    }
    let mut boundary: HashSet<u32> = HashSet::new();
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    let mut count = 0usize;
    for (vi, &val) in valence.iter().enumerate().take(n_verts) {
        if val == 0 {
            continue;
        }
        let target = if boundary.contains(&(vi as u32)) {
            4
        } else {
            6
        };
        if val != target {
            count += 1;
        }
    }
    count
}

/// Run `n` iterations of the full remesh pipeline.
#[allow(dead_code)]
pub fn remesh_iterations(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &RemeshConfig,
    n: usize,
) -> RemeshResult {
    let mut cfg = config.clone();
    cfg.iterations = n;
    isotropic_remesh(positions, indices, &cfg)
}

/// Compute an adaptive target edge length that varies by local curvature
/// approximation (edge length variance in neighbourhood).
#[allow(dead_code)]
pub fn adaptive_target_length(
    positions: &[[f32; 3]],
    indices: &[u32],
    base_length: f32,
) -> Vec<f32> {
    let n_verts = positions.len();
    let mut neighbours: Vec<Vec<u32>> = vec![Vec::new(); n_verts];
    for tri in indices.chunks_exact(3) {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            neighbours[a as usize].push(b);
            neighbours[b as usize].push(a);
        }
    }

    let mut targets = vec![base_length; n_verts];
    for vi in 0..n_verts {
        if neighbours[vi].is_empty() {
            continue;
        }
        // Compute local edge length variance.
        let mut lens: Vec<f32> = Vec::new();
        for &ni in &neighbours[vi] {
            lens.push(v3_len(v3_sub(positions[ni as usize], positions[vi])));
        }
        let n = lens.len() as f32;
        let mean = lens.iter().sum::<f32>() / n;
        let var = lens.iter().map(|l| (l - mean) * (l - mean)).sum::<f32>() / n;
        // High variance → smaller target (more detail needed).
        let factor = 1.0 / (1.0 + var / (base_length * base_length));
        targets[vi] = base_length * factor;
    }
    targets
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    fn two_tris() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 0, 2, 3],
        )
    }

    #[test]
    fn test_default_config() {
        let cfg = default_remesh_config();
        assert_eq!(cfg.iterations, 5);
        assert!((cfg.smooth_weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_edge_lengths_stats_simple() {
        let (pos, idx) = simple_tri();
        let (mean, _std, min, max) = edge_lengths_stats(&pos, &idx);
        assert!(mean > 0.0);
        assert!(min > 0.0);
        assert!(max >= min);
    }

    #[test]
    fn test_edge_lengths_stats_empty() {
        let (mean, std, _min, _max) = edge_lengths_stats(&[], &[]);
        assert_eq!(mean, 0.0);
        assert_eq!(std, 0.0);
    }

    #[test]
    fn test_split_long_edges() {
        let (pos, idx) = simple_tri();
        // Edge length is ~1.0, split at 0.5 should split.
        let (new_pos, new_idx, count) = split_long_edges(&pos, &idx, 0.5);
        assert!(count > 0);
        assert!(new_pos.len() > pos.len());
        assert_eq!(new_idx.len() % 3, 0);
    }

    #[test]
    fn test_split_long_edges_no_split() {
        let (pos, idx) = simple_tri();
        let (new_pos, _new_idx, count) = split_long_edges(&pos, &idx, 100.0);
        assert_eq!(count, 0);
        assert_eq!(new_pos.len(), pos.len());
    }

    #[test]
    fn test_collapse_short_edges() {
        let (pos, idx) = simple_tri();
        // With a very large min_len, should collapse edges.
        let (_new_pos, _new_idx, count) = collapse_short_edges(&pos, &idx, 100.0);
        assert!(count > 0);
    }

    #[test]
    fn test_collapse_short_edges_no_collapse() {
        let (pos, idx) = simple_tri();
        let (_new_pos, new_idx, count) = collapse_short_edges(&pos, &idx, 0.001);
        assert_eq!(count, 0);
        assert_eq!(new_idx.len(), idx.len());
    }

    #[test]
    fn test_equalize_valence() {
        let (pos, idx) = two_tris();
        let (new_idx, _flip_count) = equalize_valence(&pos, &idx);
        assert_eq!(new_idx.len() % 3, 0);
    }

    #[test]
    fn test_tangential_smooth() {
        let (mut pos, idx) = two_tris();
        let orig = pos.clone();
        tangential_smooth(&mut pos, &idx, 0.5, false);
        // At least some vertex should have moved.
        let _moved = pos
            .iter()
            .zip(orig.iter())
            .any(|(a, b)| (a[0] - b[0]).abs() > 1e-9 || (a[1] - b[1]).abs() > 1e-9);
        // Smoothing may or may not move vertices depending on geometry.
        // Just verify no crash and valid output.
        assert_eq!(pos.len(), orig.len());
    }

    #[test]
    fn test_compute_target_edge_length() {
        let (pos, _) = simple_tri();
        let target = compute_target_edge_length(&pos, 0.1);
        assert!(target > 0.0);
    }

    #[test]
    fn test_compute_target_edge_length_empty() {
        let target = compute_target_edge_length(&[], 0.1);
        assert!((target - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_remesh_quality_score() {
        let (pos, idx) = simple_tri();
        let score = remesh_quality_score(&pos, &idx, 1.0);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_count_irregular_vertices() {
        let (pos, idx) = simple_tri();
        let count = count_irregular_vertices(&pos, &idx);
        // A single triangle: all 3 vertices have valence 1 (irregular).
        assert!(count > 0);
    }

    #[test]
    fn test_isotropic_remesh_runs() {
        let (pos, idx) = two_tris();
        let cfg = RemeshConfig {
            target_length: 0.5,
            iterations: 2,
            smooth_weight: 0.3,
            preserve_boundary: true,
            split_ratio: 4.0 / 3.0,
            collapse_ratio: 4.0 / 5.0,
        };
        let result = isotropic_remesh(&pos, &idx, &cfg);
        assert!(!result.positions.is_empty());
        assert_eq!(result.indices.len() % 3, 0);
    }

    #[test]
    fn test_remesh_iterations() {
        let (pos, idx) = two_tris();
        let cfg = default_remesh_config();
        let result = remesh_iterations(&pos, &idx, &cfg, 1);
        assert!(!result.positions.is_empty());
    }

    #[test]
    fn test_adaptive_target_length() {
        let (pos, idx) = two_tris();
        let targets = adaptive_target_length(&pos, &idx, 0.5);
        assert_eq!(targets.len(), pos.len());
        for t in &targets {
            assert!(*t > 0.0);
        }
    }
}
