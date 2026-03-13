// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Crease / hard-edge marking for subdivision surfaces.
//!
//! Edges can be marked as "creased" so that a Catmull-Clark subdivider does not
//! smooth them.  A weight of `0.0` means fully smooth; `1.0` means infinitely
//! sharp (a true corner / hard edge).

#![allow(dead_code)]

use std::collections::HashMap;

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Math helpers (kept private)
// ---------------------------------------------------------------------------

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A directed edge represented as canonical `(min, max)` vertex pair.
pub type EdgeKey = (u32, u32);

/// Crease weight: `0.0` = smooth (no crease), `1.0` = fully sharp.
pub struct CreaseEdge {
    pub v0: u32,
    pub v1: u32,
    /// Weight in `[0, 1]`.
    pub weight: f32,
}

/// Stores per-edge crease weights, keyed by canonical `(min, max)` edge.
pub struct CreaseMap {
    edges: HashMap<EdgeKey, f32>,
}

/// Configuration used by the auto-crease helpers.
pub struct CreaseConfig {
    /// Dihedral angle threshold (radians) for `auto_crease_by_angle`.
    /// Default: `π / 4` (45°).
    pub angle_threshold_rad: f32,
    /// Crease weight assigned to boundary edges. Default: `1.0`.
    pub boundary_crease: f32,
    /// Default crease weight for all other edges. Default: `0.0`.
    pub default_crease: f32,
}

impl Default for CreaseConfig {
    fn default() -> Self {
        Self {
            angle_threshold_rad: std::f32::consts::FRAC_PI_4,
            boundary_crease: 1.0,
            default_crease: 0.0,
        }
    }
}

/// Per-edge crease data ready for the subdivision algorithm.
pub struct CreaseSubdivData {
    /// For each vertex index, `true` means the vertex sits on at least one
    /// fully-sharp edge.
    pub sharp_vertex_flags: Vec<bool>,
    /// `(v0, v1, weight)` triples; `v0 < v1`.
    pub crease_weights: Vec<(u32, u32, f32)>,
}

/// Summary statistics about a `CreaseMap`.
pub struct CreaseStats {
    pub total_creases: usize,
    pub fully_sharp: usize,
    pub partially_sharp: usize,
    pub avg_weight: f32,
}

// ---------------------------------------------------------------------------
// EdgeKey helper
// ---------------------------------------------------------------------------

/// Returns the canonical `(min, max)` edge key for vertex pair `(v0, v1)`.
fn edge_key(v0: u32, v1: u32) -> EdgeKey {
    if v0 <= v1 {
        (v0, v1)
    } else {
        (v1, v0)
    }
}

// ---------------------------------------------------------------------------
// CreaseMap impl
// ---------------------------------------------------------------------------

impl CreaseMap {
    /// Create an empty crease map.
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    /// Set the crease weight for the edge `(v0, v1)`.
    ///
    /// The weight is clamped to `[0, 1]`.  The pair is stored in canonical
    /// `(min, max)` order so `set(a, b, w)` and `set(b, a, w)` refer to the
    /// same edge.
    pub fn set(&mut self, v0: u32, v1: u32, weight: f32) {
        let key = edge_key(v0, v1);
        self.edges.insert(key, weight.clamp(0.0, 1.0));
    }

    /// Return the crease weight for `(v0, v1)`, or `0.0` if not present.
    pub fn get(&self, v0: u32, v1: u32) -> f32 {
        *self.edges.get(&edge_key(v0, v1)).unwrap_or(&0.0)
    }

    /// Remove the crease entry for `(v0, v1)` if it exists.
    pub fn remove(&mut self, v0: u32, v1: u32) {
        self.edges.remove(&edge_key(v0, v1));
    }

    /// Number of crease entries.
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// `true` when no crease entries are present.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Collect all entries into a `Vec<CreaseEdge>`.
    pub fn all_creases(&self) -> Vec<CreaseEdge> {
        self.edges
            .iter()
            .map(|(&(v0, v1), &weight)| CreaseEdge { v0, v1, weight })
            .collect()
    }

    /// Edges with `weight >= threshold`.
    pub fn sharp_edges(&self, threshold: f32) -> Vec<CreaseEdge> {
        self.edges
            .iter()
            .filter(|(_, &w)| w >= threshold)
            .map(|(&(v0, v1), &weight)| CreaseEdge { v0, v1, weight })
            .collect()
    }

    /// Edges with `weight < threshold`.
    pub fn smooth_edges(&self, threshold: f32) -> Vec<CreaseEdge> {
        self.edges
            .iter()
            .filter(|(_, &w)| w < threshold)
            .map(|(&(v0, v1), &weight)| CreaseEdge { v0, v1, weight })
            .collect()
    }
}

impl Default for CreaseMap {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Standalone functions
// ---------------------------------------------------------------------------

/// Build a `CreaseMap` by computing the dihedral angle between adjacent face
/// normals.  Any edge whose dihedral angle exceeds `config.angle_threshold_rad`
/// is assigned a crease weight of `1.0`.
pub fn auto_crease_by_angle(mesh: &MeshBuffers, config: &CreaseConfig) -> CreaseMap {
    let pos = &mesh.positions;
    let idx = &mesh.indices;
    let n_faces = idx.len() / 3;

    // Compute face normals.
    let face_normal = |fi: usize| -> [f32; 3] {
        let a = pos[idx[fi * 3] as usize];
        let b = pos[idx[fi * 3 + 1] as usize];
        let c = pos[idx[fi * 3 + 2] as usize];
        normalize3(cross3(sub3(b, a), sub3(c, a)))
    };

    // Build edge → list of face indices.
    let mut edge_faces: HashMap<EdgeKey, Vec<usize>> = HashMap::new();
    for fi in 0..n_faces {
        for e in 0..3 {
            let va = idx[fi * 3 + e];
            let vb = idx[fi * 3 + (e + 1) % 3];
            edge_faces.entry(edge_key(va, vb)).or_default().push(fi);
        }
    }

    let mut crease_map = CreaseMap::new();
    for (&(v0, v1), faces) in &edge_faces {
        if faces.len() == 2 {
            let n0 = face_normal(faces[0]);
            let n1 = face_normal(faces[1]);
            // Clamp dot to [-1, 1] to avoid NaN from acos.
            let d = dot3(n0, n1).clamp(-1.0, 1.0);
            let angle = d.acos(); // dihedral angle between normals
            if angle > config.angle_threshold_rad {
                crease_map.set(v0, v1, 1.0);
            }
        }
    }
    crease_map
}

/// Mark all boundary edges (edges with only one adjacent face) with
/// `config.boundary_crease` weight.
pub fn mark_boundary_edges(mesh: &MeshBuffers, config: &CreaseConfig) -> CreaseMap {
    let idx = &mesh.indices;
    let n_faces = idx.len() / 3;

    let mut edge_count: HashMap<EdgeKey, u32> = HashMap::new();
    for fi in 0..n_faces {
        for e in 0..3 {
            let va = idx[fi * 3 + e];
            let vb = idx[fi * 3 + (e + 1) % 3];
            *edge_count.entry(edge_key(va, vb)).or_insert(0) += 1;
        }
    }

    let mut crease_map = CreaseMap::new();
    for (&(v0, v1), &count) in &edge_count {
        if count == 1 {
            crease_map.set(v0, v1, config.boundary_crease);
        }
    }
    crease_map
}

/// Merge two crease maps, taking the **maximum** weight when an edge appears in
/// both.
pub fn merge_crease_maps(a: &CreaseMap, b: &CreaseMap) -> CreaseMap {
    let mut result = CreaseMap::new();
    for (&key, &w) in &a.edges {
        result.edges.insert(key, w);
    }
    for (&key, &w) in &b.edges {
        result
            .edges
            .entry(key)
            .and_modify(|existing| *existing = existing.max(w))
            .or_insert(w);
    }
    result
}

/// Build the `CreaseSubdivData` consumed by a subdivision algorithm.
///
/// `n_verts` is the total number of vertices in the mesh.
pub fn apply_crease_to_subdivision_config(
    crease_map: &CreaseMap,
    n_verts: usize,
) -> CreaseSubdivData {
    let mut sharp_vertex_flags = vec![false; n_verts];
    let mut crease_weights: Vec<(u32, u32, f32)> = Vec::with_capacity(crease_map.len());

    for (&(v0, v1), &w) in &crease_map.edges {
        crease_weights.push((v0, v1, w));
        if w >= 1.0 {
            if (v0 as usize) < n_verts {
                sharp_vertex_flags[v0 as usize] = true;
            }
            if (v1 as usize) < n_verts {
                sharp_vertex_flags[v1 as usize] = true;
            }
        }
    }

    CreaseSubdivData {
        sharp_vertex_flags,
        crease_weights,
    }
}

/// Compute summary statistics for a `CreaseMap`.
pub fn crease_stats(crease_map: &CreaseMap) -> CreaseStats {
    let total = crease_map.len();
    if total == 0 {
        return CreaseStats {
            total_creases: 0,
            fully_sharp: 0,
            partially_sharp: 0,
            avg_weight: 0.0,
        };
    }

    let mut fully_sharp = 0usize;
    let mut partially_sharp = 0usize;
    let mut weight_sum = 0.0f32;

    for &w in crease_map.edges.values() {
        weight_sum += w;
        if (w - 1.0).abs() < 1e-6 {
            fully_sharp += 1;
        } else {
            partially_sharp += 1;
        }
    }

    CreaseStats {
        total_creases: total,
        fully_sharp,
        partially_sharp,
        avg_weight: weight_sum / total as f32,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // -----------------------------------------------------------------------
    // Mesh helpers
    // -----------------------------------------------------------------------

    /// Single triangle: positions at (0,0,0), (1,0,0), (0,1,0).
    fn single_triangle_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    /// Two triangles sharing edge (1,2) that form a 90° dihedral angle.
    ///
    /// Face 0: (0,0,0), (1,0,0), (0,1,0)  — normal ≈ (0,0,1)
    /// Face 1: (0,0,0), (0,1,0), (0,0,1)  — normal ≈ (-1,0,0)  (≈ 90° apart)
    fn two_face_mesh_90deg() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [0.0, 1.0, 0.0], // 2
                [0.0, 0.0, 1.0], // 3
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            // Face0 uses verts 0,1,2; Face1 uses verts 0,2,3.
            // They share edge (0,2).
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    /// Two co-planar triangles sharing edge (0,2).
    ///
    /// Face 0: (0,0,0), (1,0,0), (0,1,0)  — normal (0,0,1)
    /// Face 1: (0,0,0), (0,1,0), (-1,0,0) — normal (0,0,1)  (0° apart)
    fn two_face_coplanar_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],  // 0
                [1.0, 0.0, 0.0],  // 1
                [0.0, 1.0, 0.0],  // 2
                [-1.0, 0.0, 0.0], // 3
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    // -----------------------------------------------------------------------
    // CreaseMap: set / get
    // -----------------------------------------------------------------------

    #[test]
    fn crease_map_set_and_get() {
        let mut cm = CreaseMap::new();
        cm.set(3, 7, 0.8);
        assert!((cm.get(3, 7) - 0.8).abs() < 1e-6);
        // Reverse order should hit same canonical key.
        assert!((cm.get(7, 3) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn crease_map_get_missing_returns_zero() {
        let cm = CreaseMap::new();
        assert_eq!(cm.get(0, 1), 0.0);
    }

    #[test]
    fn crease_map_weight_clamped_to_one() {
        let mut cm = CreaseMap::new();
        cm.set(0, 1, 2.5);
        assert!((cm.get(0, 1) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn crease_map_weight_clamped_to_zero() {
        let mut cm = CreaseMap::new();
        cm.set(0, 1, -0.5);
        assert_eq!(cm.get(0, 1), 0.0);
    }

    // -----------------------------------------------------------------------
    // CreaseMap: remove
    // -----------------------------------------------------------------------

    #[test]
    fn crease_map_remove() {
        let mut cm = CreaseMap::new();
        cm.set(1, 2, 1.0);
        assert_eq!(cm.len(), 1);
        cm.remove(1, 2);
        assert!(cm.is_empty());
    }

    #[test]
    fn crease_map_remove_nonexistent_is_noop() {
        let mut cm = CreaseMap::new();
        cm.set(0, 1, 0.5);
        cm.remove(9, 10); // does not exist
        assert_eq!(cm.len(), 1);
    }

    // -----------------------------------------------------------------------
    // CreaseMap: len / is_empty
    // -----------------------------------------------------------------------

    #[test]
    fn crease_map_len_and_is_empty() {
        let mut cm = CreaseMap::new();
        assert!(cm.is_empty());
        cm.set(0, 1, 0.5);
        cm.set(2, 3, 1.0);
        assert_eq!(cm.len(), 2);
        assert!(!cm.is_empty());
    }

    // -----------------------------------------------------------------------
    // CreaseMap: sharp_edges / smooth_edges
    // -----------------------------------------------------------------------

    #[test]
    fn sharp_edges_filter() {
        let mut cm = CreaseMap::new();
        cm.set(0, 1, 1.0);
        cm.set(1, 2, 0.5);
        cm.set(2, 3, 0.2);
        let sharp = cm.sharp_edges(0.8);
        assert_eq!(sharp.len(), 1);
        assert_eq!(sharp[0].v0, 0);
        assert_eq!(sharp[0].v1, 1);
    }

    #[test]
    fn smooth_edges_filter() {
        let mut cm = CreaseMap::new();
        cm.set(0, 1, 1.0);
        cm.set(1, 2, 0.3);
        let smooth = cm.smooth_edges(0.8);
        assert_eq!(smooth.len(), 1);
        assert!((smooth[0].weight - 0.3).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // auto_crease_by_angle
    // -----------------------------------------------------------------------

    #[test]
    fn auto_crease_90deg_above_threshold() {
        let mesh = two_face_mesh_90deg();
        let config = CreaseConfig {
            angle_threshold_rad: std::f32::consts::FRAC_PI_4, // 45°
            ..Default::default()
        };
        let cm = auto_crease_by_angle(&mesh, &config);
        // The shared edge (0,2) should be marked because dihedral ≈ 90° > 45°.
        assert!(
            cm.get(0, 2) >= 1.0 - 1e-4,
            "shared edge must be creased; weight = {}",
            cm.get(0, 2)
        );
    }

    #[test]
    fn auto_crease_coplanar_below_threshold() {
        let mesh = two_face_coplanar_mesh();
        let config = CreaseConfig {
            angle_threshold_rad: std::f32::consts::FRAC_PI_4, // 45°
            ..Default::default()
        };
        let cm = auto_crease_by_angle(&mesh, &config);
        // Co-planar faces share edge (0,2) with 0° angle → NOT creased.
        assert!(
            cm.get(0, 2) < 1e-4,
            "co-planar edge must not be creased; weight = {}",
            cm.get(0, 2)
        );
    }

    #[test]
    fn auto_crease_single_triangle_no_creases() {
        let mesh = single_triangle_mesh();
        let config = CreaseConfig::default();
        let cm = auto_crease_by_angle(&mesh, &config);
        // A single triangle has no interior edges → nothing creased.
        assert!(cm.is_empty());
    }

    // -----------------------------------------------------------------------
    // mark_boundary_edges
    // -----------------------------------------------------------------------

    #[test]
    fn mark_boundary_all_three_edges_of_single_triangle() {
        let mesh = single_triangle_mesh();
        let config = CreaseConfig::default();
        let cm = mark_boundary_edges(&mesh, &config);
        // All three edges of a lone triangle are boundary edges.
        assert_eq!(cm.len(), 3);
        assert!((cm.get(0, 1) - 1.0).abs() < 1e-6);
        assert!((cm.get(1, 2) - 1.0).abs() < 1e-6);
        assert!((cm.get(0, 2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn mark_boundary_shared_edge_not_boundary() {
        let mesh = two_face_mesh_90deg();
        let config = CreaseConfig::default();
        let cm = mark_boundary_edges(&mesh, &config);
        // Shared edge (0,2) must NOT appear.
        assert_eq!(cm.get(0, 2), 0.0);
        // But all four outer edges should be boundary.
        assert_eq!(cm.len(), 4);
    }

    // -----------------------------------------------------------------------
    // merge_crease_maps
    // -----------------------------------------------------------------------

    #[test]
    fn merge_takes_max_weight_for_shared_edge() {
        let mut a = CreaseMap::new();
        a.set(0, 1, 0.4);
        let mut b = CreaseMap::new();
        b.set(0, 1, 0.9);
        let merged = merge_crease_maps(&a, &b);
        assert!((merged.get(0, 1) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn merge_includes_edges_unique_to_each_map() {
        let mut a = CreaseMap::new();
        a.set(0, 1, 0.5);
        let mut b = CreaseMap::new();
        b.set(2, 3, 0.7);
        let merged = merge_crease_maps(&a, &b);
        assert_eq!(merged.len(), 2);
        assert!((merged.get(0, 1) - 0.5).abs() < 1e-6);
        assert!((merged.get(2, 3) - 0.7).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // apply_crease_to_subdivision_config
    // -----------------------------------------------------------------------

    #[test]
    fn apply_crease_sharp_vertex_flags() {
        let mut cm = CreaseMap::new();
        cm.set(1, 3, 1.0);
        let data = apply_crease_to_subdivision_config(&cm, 6);
        assert!(data.sharp_vertex_flags[1]);
        assert!(data.sharp_vertex_flags[3]);
        assert!(!data.sharp_vertex_flags[0]);
        assert!(!data.sharp_vertex_flags[2]);
    }

    #[test]
    fn apply_crease_partial_weight_not_in_sharp_flags() {
        let mut cm = CreaseMap::new();
        cm.set(0, 2, 0.5); // partial crease
        let data = apply_crease_to_subdivision_config(&cm, 4);
        // 0.5 < 1.0 so neither vertex gets the sharp flag.
        assert!(!data.sharp_vertex_flags[0]);
        assert!(!data.sharp_vertex_flags[2]);
    }

    #[test]
    fn apply_crease_weights_vec_populated() {
        let mut cm = CreaseMap::new();
        cm.set(0, 1, 0.6);
        cm.set(2, 3, 1.0);
        let data = apply_crease_to_subdivision_config(&cm, 5);
        assert_eq!(data.crease_weights.len(), 2);
    }

    // -----------------------------------------------------------------------
    // crease_stats
    // -----------------------------------------------------------------------

    #[test]
    fn crease_stats_empty_map() {
        let cm = CreaseMap::new();
        let s = crease_stats(&cm);
        assert_eq!(s.total_creases, 0);
        assert_eq!(s.fully_sharp, 0);
        assert_eq!(s.partially_sharp, 0);
        assert_eq!(s.avg_weight, 0.0);
    }

    #[test]
    fn crease_stats_mixed() {
        let mut cm = CreaseMap::new();
        cm.set(0, 1, 1.0);
        cm.set(1, 2, 1.0);
        cm.set(2, 3, 0.5);
        let s = crease_stats(&cm);
        assert_eq!(s.total_creases, 3);
        assert_eq!(s.fully_sharp, 2);
        assert_eq!(s.partially_sharp, 1);
        // avg = (1.0 + 1.0 + 0.5) / 3 ≈ 0.833
        assert!((s.avg_weight - (2.5 / 3.0)).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // all_creases round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn all_creases_round_trip() {
        let mut cm = CreaseMap::new();
        cm.set(10, 20, 0.3);
        cm.set(5, 15, 0.7);
        let all = cm.all_creases();
        assert_eq!(all.len(), 2);
        // All weights must be in [0,1].
        for ce in &all {
            assert!(ce.weight >= 0.0 && ce.weight <= 1.0);
            assert!(ce.v0 <= ce.v1, "canonical order violated");
        }
    }
}
