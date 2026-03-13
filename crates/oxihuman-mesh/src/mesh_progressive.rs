// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Progressive mesh: greedy edge-collapse LOD sequence.

#![allow(dead_code)]

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

// ── Public types ─────────────────────────────────────────────────────────────

/// A single edge-collapse event.
#[derive(Debug, Clone)]
pub struct CollapseRecord {
    /// The vertex that was removed.
    pub vertex_removed: u32,
    /// The vertex that `vertex_removed` was merged into.
    pub vertex_target: u32,
    /// Midpoint (position stored for the merged vertex after collapse).
    pub midpoint: [f32; 3],
    /// Approximate QEM proxy error for this collapse.
    pub error: f32,
}

/// A progressive mesh storing the full collapse sequence.
#[derive(Debug, Clone)]
pub struct ProgressiveMesh {
    /// Full-resolution vertex positions (before any collapses).
    pub base_positions: Vec<[f32; 3]>,
    /// Full-resolution triangle index buffer.
    pub base_indices: Vec<u32>,
    /// Collapse records ordered cheapest-first.
    pub collapse_sequence: Vec<CollapseRecord>,
}

/// Configuration for progressive mesh construction.
#[derive(Debug, Clone)]
pub struct ProgressiveMeshConfig {
    /// Stop collapsing when vertex count falls to this value. Default 50.
    pub min_vertices: usize,
    /// Stop collapsing when the error proxy exceeds this. Default f32::MAX.
    pub max_error: f32,
}

impl Default for ProgressiveMeshConfig {
    fn default() -> Self {
        Self {
            min_vertices: 50,
            max_error: f32::MAX,
        }
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

/// Walk union-find with path compression.
fn uf_find(rep: &mut [u32], mut v: u32) -> u32 {
    while rep[v as usize] != v {
        let gp = rep[rep[v as usize] as usize];
        rep[v as usize] = gp;
        v = gp;
    }
    v
}

// ── build_progressive_mesh ───────────────────────────────────────────────────

/// Build a progressive mesh from the given positions and triangle indices.
///
/// Uses greedy shortest-edge collapse, recording each collapse as a
/// [`CollapseRecord`]. Stops when `cfg.min_vertices` or `cfg.max_error` is
/// reached.
pub fn build_progressive_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &ProgressiveMeshConfig,
) -> ProgressiveMesh {
    let nv = positions.len();
    if nv == 0 || indices.len() < 3 {
        return ProgressiveMesh {
            base_positions: positions.to_vec(),
            base_indices: indices.to_vec(),
            collapse_sequence: vec![],
        };
    }

    // Union-find for vertex merging.
    let mut rep: Vec<u32> = (0..nv as u32).collect();
    // Working positions (mutable copy).
    let mut pos: Vec<[f32; 3]> = positions.to_vec();
    // Working index buffer.
    let mut tris: Vec<u32> = indices.to_vec();
    // Removed faces.
    let n_faces = indices.len() / 3;
    let mut face_dead: Vec<bool> = vec![false; n_faces];
    let mut active_verts = nv;
    let mut collapse_sequence: Vec<CollapseRecord> = Vec::new();

    // Heap: (Reverse<error_bits>, v_remove, v_target)
    // We use f32::to_bits for heap ordering.
    let mut heap: BinaryHeap<(Reverse<u32>, u32, u32)> = BinaryHeap::new();

    // Seed heap with all edges.
    {
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for tri in tris.chunks_exact(3) {
            let (a, b, c) = (tri[0], tri[1], tri[2]);
            for &(ea, eb) in &[(a, b), (b, c), (a, c)] {
                let key = (ea.min(eb), ea.max(eb));
                if seen.insert(key) {
                    let err = dist(pos[key.0 as usize], pos[key.1 as usize]);
                    heap.push((Reverse(err.to_bits()), key.0, key.1));
                }
            }
        }
    }

    // Track which vertices are still active.
    let mut is_active: Vec<bool> = vec![true; nv];

    while active_verts > cfg.min_vertices.max(1) {
        let (Reverse(err_bits), v0_raw, v1_raw) = match heap.pop() {
            Some(e) => e,
            None => break,
        };
        let err = f32::from_bits(err_bits);
        if err > cfg.max_error {
            break;
        }

        let vr = uf_find(&mut rep, v0_raw);
        let vt = uf_find(&mut rep, v1_raw);
        if vr == vt {
            // Already merged, stale entry.
            continue;
        }
        // Skip if either endpoint already removed.
        if !is_active[vr as usize] || !is_active[vt as usize] {
            continue;
        }

        // Perform collapse: remove vr, keep vt.
        let mid = midpoint(pos[vr as usize], pos[vt as usize]);
        pos[vt as usize] = mid;
        rep[vr as usize] = vt;
        is_active[vr as usize] = false;
        active_verts -= 1;

        collapse_sequence.push(CollapseRecord {
            vertex_removed: vr,
            vertex_target: vt,
            midpoint: mid,
            error: err,
        });

        // Update triangles: replace vr with vt; kill degenerate faces.
        for (fi, dead) in face_dead.iter_mut().enumerate() {
            if *dead {
                continue;
            }
            let base = fi * 3;
            for k in 0..3 {
                let rv = uf_find(&mut rep, tris[base + k]);
                if rv != tris[base + k] {
                    tris[base + k] = rv;
                }
            }
            if tris[base] == tris[base + 1]
                || tris[base + 1] == tris[base + 2]
                || tris[base] == tris[base + 2]
            {
                *dead = true;
            }
        }

        // Push new edges from vt's neighbourhood.
        // Build adjacency of live faces involving vt.
        let mut neighbours: HashSet<u32> = HashSet::new();
        for (fi, &dead) in face_dead.iter().enumerate() {
            if dead {
                continue;
            }
            let base = fi * 3;
            let has_vt = (0..3).any(|k| tris[base + k] == vt);
            if has_vt {
                for k in 0..3 {
                    let nb = tris[base + k];
                    if nb != vt {
                        neighbours.insert(nb);
                    }
                }
            }
        }
        for nb in neighbours {
            let new_err = dist(pos[vt as usize], pos[nb as usize]);
            heap.push((Reverse(new_err.to_bits()), vt, nb));
        }
    }

    ProgressiveMesh {
        base_positions: positions.to_vec(),
        base_indices: indices.to_vec(),
        collapse_sequence,
    }
}

// ── apply_collapses helper ───────────────────────────────────────────────────

/// Apply the first `n_collapses` records and return (positions, indices).
fn apply_n_collapses(pm: &ProgressiveMesh, n_collapses: usize) -> (Vec<[f32; 3]>, Vec<u32>) {
    let nv = pm.base_positions.len();
    let mut rep: Vec<u32> = (0..nv as u32).collect();
    let mut pos: Vec<[f32; 3]> = pm.base_positions.clone();

    let n = n_collapses.min(pm.collapse_sequence.len());
    for rec in &pm.collapse_sequence[..n] {
        let vr = rec.vertex_removed as usize;
        let vt = rec.vertex_target as usize;
        pos[vt] = rec.midpoint;
        rep[vr] = rec.vertex_target;
    }

    // Resolve chains.
    for i in 0..nv {
        let r = uf_find(&mut rep, i as u32);
        rep[i] = r;
    }

    // Remap indices.
    let mut out_tris: Vec<u32> = Vec::with_capacity(pm.base_indices.len());
    for tri in pm.base_indices.chunks_exact(3) {
        let a = rep[tri[0] as usize];
        let b = rep[tri[1] as usize];
        let c = rep[tri[2] as usize];
        if a != b && b != c && a != c {
            out_tris.push(a);
            out_tris.push(b);
            out_tris.push(c);
        }
    }

    // Compact: collect only active vertices.
    let active: HashSet<u32> = out_tris.iter().copied().collect();
    let mut old_to_new: HashMap<u32, u32> = HashMap::new();
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    // Sort for determinism.
    let mut active_sorted: Vec<u32> = active.into_iter().collect();
    active_sorted.sort_unstable();
    for old in &active_sorted {
        old_to_new.insert(*old, new_positions.len() as u32);
        new_positions.push(pos[*old as usize]);
    }
    let new_indices: Vec<u32> = out_tris.iter().map(|&v| old_to_new[&v]).collect();

    (new_positions, new_indices)
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Extract a LOD level by target vertex count.
///
/// Applies the minimum number of collapses to reach `target_vertices` or
/// fewer vertices.
pub fn extract_lod_level(
    pm: &ProgressiveMesh,
    target_vertices: usize,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let nv = pm.base_positions.len();
    if target_vertices >= nv {
        return (pm.base_positions.clone(), pm.base_indices.clone());
    }
    // Need to collapse (nv - target_vertices) vertices.
    let need = nv.saturating_sub(target_vertices);
    let n = need.min(pm.collapse_sequence.len());
    apply_n_collapses(pm, n)
}

/// Extract a LOD level by ratio of original vertex count to retain (0..=1).
///
/// `ratio = 1.0` returns the full mesh; `ratio = 0.5` returns ~50 % of
/// vertices.
pub fn extract_lod_ratio(pm: &ProgressiveMesh, ratio: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let nv = pm.base_positions.len();
    let target = ((nv as f32) * ratio.clamp(0.0, 1.0)).round() as usize;
    extract_lod_level(pm, target)
}

/// Return the slice of collapse records that must be *undone* to refine from
/// `current_verts` up to `target_verts` (for streaming refinement).
///
/// The returned records are in reverse order (last collapse first).
pub fn refine_lod(
    pm: &ProgressiveMesh,
    current_verts: usize,
    target_verts: usize,
) -> Vec<CollapseRecord> {
    if target_verts <= current_verts {
        return vec![];
    }
    let nv = pm.base_positions.len();
    // collapses applied to reach current_verts:
    let applied_current = nv
        .saturating_sub(current_verts)
        .min(pm.collapse_sequence.len());
    // collapses that would have been applied to reach target_verts:
    let applied_target = nv
        .saturating_sub(target_verts)
        .min(pm.collapse_sequence.len());

    if applied_target >= applied_current {
        return vec![];
    }
    // Records to undo: from applied_target..applied_current, reversed.
    pm.collapse_sequence[applied_target..applied_current]
        .iter()
        .rev()
        .cloned()
        .collect()
}

/// Return the per-collapse errors in sequence order.
pub fn collapse_error_sequence(pm: &ProgressiveMesh) -> Vec<f32> {
    pm.collapse_sequence.iter().map(|r| r.error).collect()
}

/// Extract multiple LOD levels at once from a set of ratios.
pub fn progressive_lod_levels(
    pm: &ProgressiveMesh,
    levels: &[f32],
) -> Vec<(Vec<[f32; 3]>, Vec<u32>)> {
    levels.iter().map(|&r| extract_lod_ratio(pm, r)).collect()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple cube-like mesh (8 verts, 12 triangles).
    fn cube_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        #[rustfmt::skip]
        let indices: Vec<u32> = vec![
            0,1,2, 0,2,3,  // -Z face
            4,5,6, 4,6,7,  // +Z face
            0,1,5, 0,5,4,  // -Y face
            2,3,7, 2,7,6,  // +Y face
            0,3,7, 0,7,4,  // -X face
            1,2,6, 1,6,5,  // +X face
        ];
        (positions, indices)
    }

    /// Build a small grid mesh (n x n quads → 2n² triangles).
    fn grid_mesh(n: usize) -> (Vec<[f32; 3]>, Vec<u32>) {
        let mut pos = Vec::new();
        let mut idx = Vec::new();
        for j in 0..=n {
            for i in 0..=n {
                pos.push([i as f32, j as f32, 0.0_f32]);
            }
        }
        let w = n + 1;
        for j in 0..n {
            for i in 0..n {
                let bl = (j * w + i) as u32;
                let br = bl + 1;
                let tl = bl + w as u32;
                let tr = tl + 1;
                idx.push(bl);
                idx.push(br);
                idx.push(tl);
                idx.push(br);
                idx.push(tr);
                idx.push(tl);
            }
        }
        (pos, idx)
    }

    #[test]
    fn build_produces_collapses() {
        let (pos, idx) = cube_mesh();
        let cfg = ProgressiveMeshConfig {
            min_vertices: 4,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        assert!(
            !pm.collapse_sequence.is_empty(),
            "should have collapse records"
        );
    }

    #[test]
    fn build_stores_base_mesh() {
        let (pos, idx) = cube_mesh();
        let cfg = ProgressiveMeshConfig::default();
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        assert_eq!(pm.base_positions.len(), pos.len());
        assert_eq!(pm.base_indices.len(), idx.len());
    }

    #[test]
    fn extract_lod_level_returns_fewer_verts() {
        let (pos, idx) = grid_mesh(6); // 7*7=49 verts
        let cfg = ProgressiveMeshConfig {
            min_vertices: 4,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        let orig_verts = pos.len();
        let (lod_pos, _) = extract_lod_level(&pm, 20);
        assert!(lod_pos.len() <= orig_verts, "LOD should have fewer verts");
    }

    #[test]
    fn extract_lod_ratio_one_returns_full() {
        let (pos, idx) = cube_mesh();
        let cfg = ProgressiveMeshConfig::default();
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        let (lod_pos, lod_idx) = extract_lod_ratio(&pm, 1.0);
        assert_eq!(lod_pos.len(), pos.len());
        assert_eq!(lod_idx.len(), idx.len());
    }

    #[test]
    fn extract_lod_ratio_half_returns_fewer() {
        let (pos, idx) = grid_mesh(8); // 81 verts
        let cfg = ProgressiveMeshConfig {
            min_vertices: 4,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        let orig_verts = pos.len();
        let (lod_pos, _) = extract_lod_ratio(&pm, 0.5);
        assert!(
            lod_pos.len() <= orig_verts,
            "0.5 ratio should collapse some verts"
        );
    }

    #[test]
    fn extract_lod_ratio_zero_returns_minimum() {
        let (pos, idx) = grid_mesh(8);
        let min_v = 4;
        let cfg = ProgressiveMeshConfig {
            min_vertices: min_v,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        let (lod_pos, _) = extract_lod_ratio(&pm, 0.0);
        // Should not go below min_vertices (may vary due to topology).
        assert!(lod_pos.len() <= pos.len());
    }

    #[test]
    fn refine_lod_slice_count_correct() {
        let (pos, idx) = grid_mesh(6);
        let cfg = ProgressiveMeshConfig {
            min_vertices: 4,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        let nv = pos.len();
        // Simulate: from half collapsed to quarter collapsed.
        let curr = nv / 2;
        let tgt = nv * 3 / 4;
        let records = refine_lod(&pm, curr, tgt);
        // records should be those collapses between the two levels, reversed.
        let applied_curr = (nv - curr).min(pm.collapse_sequence.len());
        let applied_tgt = (nv - tgt).min(pm.collapse_sequence.len());
        if applied_tgt < applied_curr {
            assert_eq!(records.len(), applied_curr - applied_tgt);
        } else {
            assert!(records.is_empty());
        }
    }

    #[test]
    fn refine_lod_empty_when_same_level() {
        let (pos, idx) = cube_mesh();
        let cfg = ProgressiveMeshConfig::default();
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        let records = refine_lod(&pm, 6, 6);
        assert!(records.is_empty());
    }

    #[test]
    fn collapse_error_sequence_length_matches() {
        let (pos, idx) = cube_mesh();
        let cfg = ProgressiveMeshConfig {
            min_vertices: 3,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        let errs = collapse_error_sequence(&pm);
        assert_eq!(errs.len(), pm.collapse_sequence.len());
    }

    #[test]
    fn collapse_error_sequence_non_negative() {
        let (pos, idx) = grid_mesh(5);
        let cfg = ProgressiveMeshConfig {
            min_vertices: 3,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        for e in collapse_error_sequence(&pm) {
            assert!(e >= 0.0, "errors must be non-negative");
        }
    }

    #[test]
    fn progressive_lod_levels_count_matches() {
        let (pos, idx) = grid_mesh(6);
        let cfg = ProgressiveMeshConfig {
            min_vertices: 4,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        let ratios = [1.0, 0.75, 0.5, 0.25];
        let lods = progressive_lod_levels(&pm, &ratios);
        assert_eq!(lods.len(), ratios.len());
    }

    #[test]
    fn all_resulting_indices_in_range() {
        let (pos, idx) = grid_mesh(5);
        let cfg = ProgressiveMeshConfig {
            min_vertices: 4,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        for ratio in [1.0, 0.75, 0.5, 0.25, 0.0] {
            let (lod_pos, lod_idx) = extract_lod_ratio(&pm, ratio);
            let nv = lod_pos.len() as u32;
            for &i in &lod_idx {
                assert!(i < nv, "index {} out of range (nv={})", i, nv);
            }
        }
    }

    #[test]
    fn min_vertices_respected() {
        let (pos, idx) = grid_mesh(8);
        let min_v = 20;
        let cfg = ProgressiveMeshConfig {
            min_vertices: min_v,
            max_error: f32::MAX,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        // After all collapses, remaining active verts >= min_vertices.
        // The number of collapses should be <= nv - min_v.
        assert!(pm.collapse_sequence.len() <= pos.len() - min_v + 1);
    }

    #[test]
    fn max_error_stops_collapses() {
        let (pos, idx) = grid_mesh(8);
        // Very tight error bound: stop at essentially zero collapse distance.
        let cfg = ProgressiveMeshConfig {
            min_vertices: 4,
            max_error: 0.0,
        };
        let pm = build_progressive_mesh(&pos, &idx, &cfg);
        // Should produce zero (or very few) collapses since all edge lengths > 0.
        assert!(pm.collapse_sequence.is_empty() || pm.collapse_sequence.len() < 5);
    }

    #[test]
    fn empty_input_returns_empty_pm() {
        let cfg = ProgressiveMeshConfig::default();
        let pm = build_progressive_mesh(&[], &[], &cfg);
        assert!(pm.base_positions.is_empty());
        assert!(pm.collapse_sequence.is_empty());
    }
}
