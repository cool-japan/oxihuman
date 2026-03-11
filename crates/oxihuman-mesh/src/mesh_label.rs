// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// BodyRegion enum
// ---------------------------------------------------------------------------

/// Semantic body region labels for mesh vertex/face segmentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BodyRegion {
    Unknown = 0,
    Head = 1,
    Neck = 2,
    Torso = 3,
    UpperArmLeft = 4,
    UpperArmRight = 5,
    LowerArmLeft = 6,
    LowerArmRight = 7,
    HandLeft = 8,
    HandRight = 9,
    UpperLegLeft = 10,
    UpperLegRight = 11,
    LowerLegLeft = 12,
    LowerLegRight = 13,
    FootLeft = 14,
    FootRight = 15,
}

impl BodyRegion {
    /// Return all variants in order.
    pub fn all() -> Vec<BodyRegion> {
        vec![
            BodyRegion::Unknown,
            BodyRegion::Head,
            BodyRegion::Neck,
            BodyRegion::Torso,
            BodyRegion::UpperArmLeft,
            BodyRegion::UpperArmRight,
            BodyRegion::LowerArmLeft,
            BodyRegion::LowerArmRight,
            BodyRegion::HandLeft,
            BodyRegion::HandRight,
            BodyRegion::UpperLegLeft,
            BodyRegion::UpperLegRight,
            BodyRegion::LowerLegLeft,
            BodyRegion::LowerLegRight,
            BodyRegion::FootLeft,
            BodyRegion::FootRight,
        ]
    }

    /// Human-readable name of the region.
    pub fn name(&self) -> &'static str {
        match self {
            BodyRegion::Unknown => "Unknown",
            BodyRegion::Head => "Head",
            BodyRegion::Neck => "Neck",
            BodyRegion::Torso => "Torso",
            BodyRegion::UpperArmLeft => "UpperArmLeft",
            BodyRegion::UpperArmRight => "UpperArmRight",
            BodyRegion::LowerArmLeft => "LowerArmLeft",
            BodyRegion::LowerArmRight => "LowerArmRight",
            BodyRegion::HandLeft => "HandLeft",
            BodyRegion::HandRight => "HandRight",
            BodyRegion::UpperLegLeft => "UpperLegLeft",
            BodyRegion::UpperLegRight => "UpperLegRight",
            BodyRegion::LowerLegLeft => "LowerLegLeft",
            BodyRegion::LowerLegRight => "LowerLegRight",
            BodyRegion::FootLeft => "FootLeft",
            BodyRegion::FootRight => "FootRight",
        }
    }

    /// Distinct RGB color for visualization.
    pub fn color_rgb(&self) -> [u8; 3] {
        match self {
            BodyRegion::Unknown => [128, 128, 128],
            BodyRegion::Head => [255, 60, 60],
            BodyRegion::Neck => [255, 140, 20],
            BodyRegion::Torso => [255, 220, 50],
            BodyRegion::UpperArmLeft => [80, 200, 80],
            BodyRegion::UpperArmRight => [20, 140, 20],
            BodyRegion::LowerArmLeft => [60, 220, 220],
            BodyRegion::LowerArmRight => [20, 140, 180],
            BodyRegion::HandLeft => [80, 80, 255],
            BodyRegion::HandRight => [20, 20, 180],
            BodyRegion::UpperLegLeft => [180, 60, 220],
            BodyRegion::UpperLegRight => [120, 20, 160],
            BodyRegion::LowerLegLeft => [255, 140, 200],
            BodyRegion::LowerLegRight => [200, 80, 140],
            BodyRegion::FootLeft => [220, 220, 100],
            BodyRegion::FootRight => [160, 160, 20],
        }
    }

    /// Returns `true` if this is a left-side region.
    pub fn is_left(&self) -> bool {
        matches!(
            self,
            BodyRegion::UpperArmLeft
                | BodyRegion::LowerArmLeft
                | BodyRegion::HandLeft
                | BodyRegion::UpperLegLeft
                | BodyRegion::LowerLegLeft
                | BodyRegion::FootLeft
        )
    }

    /// Returns `true` if this is a right-side region.
    pub fn is_right(&self) -> bool {
        matches!(
            self,
            BodyRegion::UpperArmRight
                | BodyRegion::LowerArmRight
                | BodyRegion::HandRight
                | BodyRegion::UpperLegRight
                | BodyRegion::LowerLegRight
                | BodyRegion::FootRight
        )
    }

    /// Flip left ↔ right. Central/unknown regions return themselves.
    pub fn mirror(&self) -> BodyRegion {
        match self {
            BodyRegion::UpperArmLeft => BodyRegion::UpperArmRight,
            BodyRegion::UpperArmRight => BodyRegion::UpperArmLeft,
            BodyRegion::LowerArmLeft => BodyRegion::LowerArmRight,
            BodyRegion::LowerArmRight => BodyRegion::LowerArmLeft,
            BodyRegion::HandLeft => BodyRegion::HandRight,
            BodyRegion::HandRight => BodyRegion::HandLeft,
            BodyRegion::UpperLegLeft => BodyRegion::UpperLegRight,
            BodyRegion::UpperLegRight => BodyRegion::UpperLegLeft,
            BodyRegion::LowerLegLeft => BodyRegion::LowerLegRight,
            BodyRegion::LowerLegRight => BodyRegion::LowerLegLeft,
            BodyRegion::FootLeft => BodyRegion::FootRight,
            BodyRegion::FootRight => BodyRegion::FootLeft,
            other => *other,
        }
    }

    /// Convert a raw `u8` to a `BodyRegion` (unknown for out-of-range values).
    pub fn from_u8(v: u8) -> BodyRegion {
        match v {
            1 => BodyRegion::Head,
            2 => BodyRegion::Neck,
            3 => BodyRegion::Torso,
            4 => BodyRegion::UpperArmLeft,
            5 => BodyRegion::UpperArmRight,
            6 => BodyRegion::LowerArmLeft,
            7 => BodyRegion::LowerArmRight,
            8 => BodyRegion::HandLeft,
            9 => BodyRegion::HandRight,
            10 => BodyRegion::UpperLegLeft,
            11 => BodyRegion::UpperLegRight,
            12 => BodyRegion::LowerLegLeft,
            13 => BodyRegion::LowerLegRight,
            14 => BodyRegion::FootLeft,
            15 => BodyRegion::FootRight,
            _ => BodyRegion::Unknown,
        }
    }
}

// ---------------------------------------------------------------------------
// MeshLabels
// ---------------------------------------------------------------------------

/// Per-vertex region label storage.
pub struct MeshLabels {
    pub vertex_labels: Vec<BodyRegion>,
    pub vertex_count: usize,
}

impl MeshLabels {
    /// Create a new label set with every vertex set to `Unknown`.
    pub fn new(vertex_count: usize) -> Self {
        MeshLabels {
            vertex_labels: vec![BodyRegion::Unknown; vertex_count],
            vertex_count,
        }
    }

    /// Set the label of vertex `vi`.
    pub fn set(&mut self, vi: usize, label: BodyRegion) {
        if vi < self.vertex_count {
            self.vertex_labels[vi] = label;
        }
    }

    /// Get the label of vertex `vi`. Returns `Unknown` for out-of-range indices.
    pub fn get(&self, vi: usize) -> BodyRegion {
        self.vertex_labels
            .get(vi)
            .copied()
            .unwrap_or(BodyRegion::Unknown)
    }

    /// Count vertices labeled with the given region.
    pub fn count_region(&self, region: BodyRegion) -> usize {
        self.vertex_labels.iter().filter(|&&r| r == region).count()
    }

    /// Fraction of vertices that have a non-Unknown label (0.0 – 1.0).
    pub fn labeled_fraction(&self) -> f32 {
        if self.vertex_count == 0 {
            return 0.0;
        }
        let labeled = self
            .vertex_labels
            .iter()
            .filter(|&&r| r != BodyRegion::Unknown)
            .count();
        labeled as f32 / self.vertex_count as f32
    }

    /// Collect all vertex indices whose label matches `region`.
    pub fn vertices_in(&self, region: BodyRegion) -> Vec<usize> {
        self.vertex_labels
            .iter()
            .enumerate()
            .filter_map(|(i, &r)| if r == region { Some(i) } else { None })
            .collect()
    }

    /// Compute (min, max) AABB corners for the given region.
    ///
    /// Returns `None` if no vertex belongs to the region.
    pub fn region_bbox(
        &self,
        region: BodyRegion,
        positions: &[[f32; 3]],
    ) -> Option<([f32; 3], [f32; 3])> {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        let mut found = false;
        for (i, &r) in self.vertex_labels.iter().enumerate() {
            if r == region {
                if let Some(p) = positions.get(i) {
                    for k in 0..3 {
                        if p[k] < min[k] {
                            min[k] = p[k];
                        }
                        if p[k] > max[k] {
                            max[k] = p[k];
                        }
                    }
                    found = true;
                }
            }
        }
        if found {
            Some((min, max))
        } else {
            None
        }
    }

    /// Compute the centroid position for the given region.
    ///
    /// Returns `None` if no vertex belongs to the region.
    pub fn region_centroid(&self, region: BodyRegion, positions: &[[f32; 3]]) -> Option<[f32; 3]> {
        let mut sum = [0.0f32; 3];
        let mut count = 0usize;
        for (i, &r) in self.vertex_labels.iter().enumerate() {
            if r == region {
                if let Some(p) = positions.get(i) {
                    sum[0] += p[0];
                    sum[1] += p[1];
                    sum[2] += p[2];
                    count += 1;
                }
            }
        }
        if count == 0 {
            None
        } else {
            let n = count as f32;
            Some([sum[0] / n, sum[1] / n, sum[2] / n])
        }
    }

    /// Re-label each vertex as the majority label among itself and its
    /// triangle neighbors. Runs `iterations` passes.
    pub fn smooth(&mut self, indices: &[u32], iterations: usize) {
        // Build adjacency: vertex → set of neighboring vertices
        let n = self.vertex_count;
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let tri_count = indices.len() / 3;
        for t in 0..tri_count {
            let a = indices[t * 3] as usize;
            let b = indices[t * 3 + 1] as usize;
            let c = indices[t * 3 + 2] as usize;
            if a < n && b < n {
                adj[a].push(b);
                adj[b].push(a);
            }
            if b < n && c < n {
                adj[b].push(c);
                adj[c].push(b);
            }
            if a < n && c < n {
                adj[a].push(c);
                adj[c].push(a);
            }
        }

        for _ in 0..iterations {
            let old = self.vertex_labels.clone();
            for vi in 0..n {
                let mut freq: HashMap<u8, usize> = HashMap::new();
                // include self
                *freq.entry(old[vi] as u8).or_insert(0) += 1;
                for &nb in &adj[vi] {
                    *freq.entry(old[nb] as u8).or_insert(0) += 1;
                }
                // pick the most frequent label
                if let Some((&best_byte, _)) = freq.iter().max_by_key(|(_, &v)| v) {
                    self.vertex_labels[vi] = BodyRegion::from_u8(best_byte);
                }
            }
        }
    }

    /// Serialize labels to one byte per vertex (the `repr(u8)` discriminant).
    pub fn to_bytes(&self) -> Vec<u8> {
        self.vertex_labels.iter().map(|&r| r as u8).collect()
    }

    /// Deserialize from a byte slice created by [`Self::to_bytes`].
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let vertex_labels: Vec<BodyRegion> =
            bytes.iter().map(|&b| BodyRegion::from_u8(b)).collect();
        let vertex_count = vertex_labels.len();
        MeshLabels {
            vertex_labels,
            vertex_count,
        }
    }

    /// Build a per-vertex color buffer using each region's `color_rgb`.
    pub fn to_vertex_colors(&self) -> Vec<[u8; 3]> {
        self.vertex_labels.iter().map(|r| r.color_rgb()).collect()
    }
}

// ---------------------------------------------------------------------------
// label_by_height
// ---------------------------------------------------------------------------

/// Assign body region labels based on the normalized Y-height of each vertex.
///
/// The bounding box Y range is divided into bands corresponding to the major
/// body regions (same heuristic as body_landmark but assigns full regions).
pub fn label_by_height(mesh: &MeshBuffers) -> MeshLabels {
    let positions = &mesh.positions;
    let n = positions.len();
    let mut labels = MeshLabels::new(n);

    if n == 0 {
        return labels;
    }

    let y_min = positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    let y_max = positions
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let y_range = y_max - y_min;

    if y_range < f32::EPSILON {
        // Flat mesh – label everything as torso
        for vi in 0..n {
            labels.set(vi, BodyRegion::Torso);
        }
        return labels;
    }

    // Y-fraction thresholds (from bottom=0 to top=1):
    //   0.00 – 0.06  feet
    //   0.06 – 0.18  lower legs
    //   0.18 – 0.35  upper legs
    //   0.35 – 0.65  torso
    //   0.65 – 0.72  neck
    //   0.72 – 1.00  head
    //   arm heuristic: X-extent beyond a threshold while at torso height
    let x_min = positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let x_max = positions
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let x_mid = (x_min + x_max) * 0.5;
    let x_half = (x_max - x_min) * 0.5;

    for (vi, &p) in positions.iter().enumerate() {
        let yf = (p[1] - y_min) / y_range;
        let xf = (p[0] - x_mid) / (x_half + f32::EPSILON); // -1..1, left is positive in typical coords

        let label = if yf < 0.06 {
            // feet – left/right by X
            if xf >= 0.0 {
                BodyRegion::FootLeft
            } else {
                BodyRegion::FootRight
            }
        } else if yf < 0.18 {
            if xf >= 0.0 {
                BodyRegion::LowerLegLeft
            } else {
                BodyRegion::LowerLegRight
            }
        } else if yf < 0.35 {
            if xf >= 0.0 {
                BodyRegion::UpperLegLeft
            } else {
                BodyRegion::UpperLegRight
            }
        } else if yf < 0.65 {
            // torso band – check for arms extending outward
            let abs_xf = xf.abs();
            if abs_xf > 0.70 {
                // arm region
                if yf > 0.55 {
                    // upper arm (shoulder area)
                    if xf >= 0.0 {
                        BodyRegion::UpperArmLeft
                    } else {
                        BodyRegion::UpperArmRight
                    }
                } else if yf > 0.44 {
                    if xf >= 0.0 {
                        BodyRegion::LowerArmLeft
                    } else {
                        BodyRegion::LowerArmRight
                    }
                } else if xf >= 0.0 {
                    BodyRegion::HandLeft
                } else {
                    BodyRegion::HandRight
                }
            } else {
                BodyRegion::Torso
            }
        } else if yf < 0.72 {
            BodyRegion::Neck
        } else {
            BodyRegion::Head
        };

        labels.set(vi, label);
    }

    labels
}

// ---------------------------------------------------------------------------
// flood_fill_label
// ---------------------------------------------------------------------------

/// Flood-fill region growing from `seed_vertex`, assigning `label` to all
/// reachable vertices whose normals differ by at most `max_angle_deg` from
/// their neighbors' normals.
pub fn flood_fill_label(
    mesh: &MeshBuffers,
    seed_vertex: usize,
    label: BodyRegion,
    max_angle_deg: f32,
    existing: &mut MeshLabels,
) {
    let n = mesh.positions.len();
    if seed_vertex >= n {
        return;
    }

    // Build adjacency
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let tri_count = mesh.indices.len() / 3;
    for t in 0..tri_count {
        let a = mesh.indices[t * 3] as usize;
        let b = mesh.indices[t * 3 + 1] as usize;
        let c = mesh.indices[t * 3 + 2] as usize;
        if a < n && b < n {
            adj[a].push(b);
            adj[b].push(a);
        }
        if b < n && c < n {
            adj[b].push(c);
            adj[c].push(b);
        }
        if a < n && c < n {
            adj[a].push(c);
            adj[c].push(a);
        }
    }

    let cos_thresh = max_angle_deg.to_radians().cos();
    let normals = &mesh.normals;

    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(seed_vertex);
    visited[seed_vertex] = true;

    while let Some(vi) = queue.pop_front() {
        existing.set(vi, label);
        let ni = if vi < normals.len() {
            normals[vi]
        } else {
            [0.0, 1.0, 0.0]
        };

        for &nb in &adj[vi] {
            if visited[nb] {
                continue;
            }
            let nj = if nb < normals.len() {
                normals[nb]
            } else {
                [0.0, 1.0, 0.0]
            };
            // dot product of unit normals
            let dot = ni[0] * nj[0] + ni[1] * nj[1] + ni[2] * nj[2];
            if dot >= cos_thresh {
                visited[nb] = true;
                queue.push_back(nb);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// propagate_labels
// ---------------------------------------------------------------------------

/// Fill Unknown-labeled vertices by propagating labels from labeled neighbors.
/// Runs until no more Unknown vertices can be filled.
pub fn propagate_labels(mesh: &MeshBuffers, labels: &mut MeshLabels) {
    let n = mesh.positions.len();
    if n == 0 {
        return;
    }

    // Build adjacency
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let tri_count = mesh.indices.len() / 3;
    for t in 0..tri_count {
        let a = mesh.indices[t * 3] as usize;
        let b = mesh.indices[t * 3 + 1] as usize;
        let c = mesh.indices[t * 3 + 2] as usize;
        if a < n && b < n {
            adj[a].push(b);
            adj[b].push(a);
        }
        if b < n && c < n {
            adj[b].push(c);
            adj[c].push(b);
        }
        if a < n && c < n {
            adj[a].push(c);
            adj[c].push(a);
        }
    }

    loop {
        let mut changed = false;
        for (vi, neighbors) in adj.iter().enumerate() {
            if labels.get(vi) != BodyRegion::Unknown {
                continue;
            }
            // Find any labeled neighbor
            for &nb in neighbors {
                let lbl = labels.get(nb);
                if lbl != BodyRegion::Unknown {
                    labels.set(vi, lbl);
                    changed = true;
                    break;
                }
            }
        }
        if !changed {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// body_seed_vertices
// ---------------------------------------------------------------------------

/// Heuristic: find a representative seed vertex for each body region.
///
/// Uses height-based bounding-box fractions, similar to `label_by_height`.
pub fn body_seed_vertices(mesh: &MeshBuffers) -> Vec<(BodyRegion, usize)> {
    let positions = &mesh.positions;
    let n = positions.len();
    if n == 0 {
        return Vec::new();
    }

    let y_min = positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    let y_max = positions
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let y_range = (y_max - y_min).max(f32::EPSILON);

    let x_min = positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let x_max = positions
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let x_mid = (x_min + x_max) * 0.5;

    // Target normalized (yf, xf) for each region
    let targets: &[(BodyRegion, f32, f32)] = &[
        (BodyRegion::Head, 0.90, 0.0),
        (BodyRegion::Neck, 0.68, 0.0),
        (BodyRegion::Torso, 0.50, 0.0),
        (BodyRegion::UpperArmLeft, 0.60, 0.80),
        (BodyRegion::UpperArmRight, 0.60, -0.80),
        (BodyRegion::LowerArmLeft, 0.49, 0.85),
        (BodyRegion::LowerArmRight, 0.49, -0.85),
        (BodyRegion::HandLeft, 0.38, 0.90),
        (BodyRegion::HandRight, 0.38, -0.90),
        (BodyRegion::UpperLegLeft, 0.27, 0.25),
        (BodyRegion::UpperLegRight, 0.27, -0.25),
        (BodyRegion::LowerLegLeft, 0.12, 0.25),
        (BodyRegion::LowerLegRight, 0.12, -0.25),
        (BodyRegion::FootLeft, 0.03, 0.25),
        (BodyRegion::FootRight, 0.03, -0.25),
    ];

    let x_half = ((x_max - x_min) * 0.5).max(f32::EPSILON);

    let mut results = Vec::new();
    for &(region, target_yf, target_xf) in targets {
        // Find vertex closest to the target (yf, xf) pair
        let mut best_vi = 0usize;
        let mut best_dist = f32::INFINITY;
        for (vi, p) in positions.iter().enumerate() {
            let yf = (p[1] - y_min) / y_range;
            let xf = (p[0] - x_mid) / x_half;
            let d = (yf - target_yf).powi(2) + (xf - target_xf).powi(2);
            if d < best_dist {
                best_dist = d;
                best_vi = vi;
            }
        }
        results.push((region, best_vi));
    }
    results
}

// ---------------------------------------------------------------------------
// region_boundary_edges
// ---------------------------------------------------------------------------

/// Return all edges (pairs of vertex indices) that separate two different regions.
pub fn region_boundary_edges(labels: &MeshLabels, indices: &[u32]) -> Vec<(u32, u32)> {
    let mut boundary = Vec::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        let la = labels.get(a as usize);
        let lb = labels.get(b as usize);
        let lc = labels.get(c as usize);
        if la != lb {
            boundary.push((a.min(b), a.max(b)));
        }
        if lb != lc {
            boundary.push((b.min(c), b.max(c)));
        }
        if la != lc {
            boundary.push((a.min(c), a.max(c)));
        }
    }
    // Deduplicate
    boundary.sort_unstable();
    boundary.dedup();
    boundary
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Build a simple 4-vertex quad (2 triangles) spread over Y = 0..1.
    fn make_test_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [-0.5, 0.0, 0.0],
                [0.5, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [-0.5, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    /// Build a tall body-like mesh with many vertices spread over Y = 0..2.
    fn make_body_mesh() -> MeshBuffers {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        // 11 height levels × 5 X positions
        for yi in 0..=10 {
            let y = yi as f32 * 0.2; // 0.0 .. 2.0
            for xi in 0..5 {
                let x = (xi as f32 - 2.0) * 0.3;
                positions.push([x, y, 0.0]);
                normals.push([0.0, 0.0, 1.0]);
                uvs.push([0.0, 0.0]);
            }
        }
        // Triangle strip-like indices
        let mut indices = Vec::new();
        for row in 0..10usize {
            for col in 0..4usize {
                let a = (row * 5 + col) as u32;
                let b = a + 1;
                let c = a + 5;
                let d = b + 5;
                indices.extend_from_slice(&[a, b, c, b, d, c]);
            }
        }
        MeshBuffers::from_morph(MB {
            positions,
            normals,
            uvs,
            indices,
            has_suit: false,
        })
    }

    // ------------------------------------------------------------------
    // BodyRegion tests
    // ------------------------------------------------------------------

    #[test]
    fn test_all_returns_16_variants() {
        assert_eq!(BodyRegion::all().len(), 16);
    }

    #[test]
    fn test_from_u8_roundtrip() {
        for r in BodyRegion::all() {
            let byte = r as u8;
            assert_eq!(BodyRegion::from_u8(byte), r, "roundtrip failed for {:?}", r);
        }
    }

    #[test]
    fn test_from_u8_out_of_range_is_unknown() {
        assert_eq!(BodyRegion::from_u8(200), BodyRegion::Unknown);
        assert_eq!(BodyRegion::from_u8(16), BodyRegion::Unknown);
    }

    #[test]
    fn test_mirror_left_right() {
        assert_eq!(BodyRegion::HandLeft.mirror(), BodyRegion::HandRight);
        assert_eq!(BodyRegion::HandRight.mirror(), BodyRegion::HandLeft);
        assert_eq!(BodyRegion::FootLeft.mirror(), BodyRegion::FootRight);
        assert_eq!(BodyRegion::Torso.mirror(), BodyRegion::Torso);
        assert_eq!(BodyRegion::Head.mirror(), BodyRegion::Head);
        assert_eq!(BodyRegion::Unknown.mirror(), BodyRegion::Unknown);
    }

    #[test]
    fn test_is_left_right() {
        assert!(BodyRegion::UpperArmLeft.is_left());
        assert!(!BodyRegion::UpperArmLeft.is_right());
        assert!(BodyRegion::FootRight.is_right());
        assert!(!BodyRegion::FootRight.is_left());
        assert!(!BodyRegion::Head.is_left());
        assert!(!BodyRegion::Head.is_right());
    }

    #[test]
    fn test_color_rgb_unique() {
        let colors: Vec<_> = BodyRegion::all().iter().map(|r| r.color_rgb()).collect();
        // Unknown is grey; the rest should all be distinct
        let non_unknown: Vec<_> = BodyRegion::all()
            .iter()
            .filter(|&&r| r != BodyRegion::Unknown)
            .map(|r| r.color_rgb())
            .collect();
        let mut sorted = non_unknown.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            non_unknown.len(),
            "duplicate region colors: {:?}",
            colors
        );
    }

    #[test]
    fn test_name_non_empty() {
        for r in BodyRegion::all() {
            assert!(!r.name().is_empty());
        }
    }

    // ------------------------------------------------------------------
    // MeshLabels tests
    // ------------------------------------------------------------------

    #[test]
    fn test_mesh_labels_new_all_unknown() {
        let labels = MeshLabels::new(10);
        assert_eq!(labels.vertex_count, 10);
        assert!(labels
            .vertex_labels
            .iter()
            .all(|&r| r == BodyRegion::Unknown));
    }

    #[test]
    fn test_set_get_label() {
        let mut labels = MeshLabels::new(5);
        labels.set(2, BodyRegion::Head);
        assert_eq!(labels.get(2), BodyRegion::Head);
        assert_eq!(labels.get(0), BodyRegion::Unknown);
        // out-of-range
        assert_eq!(labels.get(99), BodyRegion::Unknown);
    }

    #[test]
    fn test_count_region() {
        let mut labels = MeshLabels::new(6);
        labels.set(0, BodyRegion::Torso);
        labels.set(1, BodyRegion::Torso);
        labels.set(2, BodyRegion::Head);
        assert_eq!(labels.count_region(BodyRegion::Torso), 2);
        assert_eq!(labels.count_region(BodyRegion::Head), 1);
        assert_eq!(labels.count_region(BodyRegion::Neck), 0);
    }

    #[test]
    fn test_labeled_fraction() {
        let mut labels = MeshLabels::new(4);
        assert!((labels.labeled_fraction() - 0.0).abs() < 1e-6);
        labels.set(0, BodyRegion::Head);
        labels.set(1, BodyRegion::Torso);
        assert!((labels.labeled_fraction() - 0.5).abs() < 1e-6);
        // empty
        let empty = MeshLabels::new(0);
        assert!((empty.labeled_fraction() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_vertices_in() {
        let mut labels = MeshLabels::new(5);
        labels.set(1, BodyRegion::Neck);
        labels.set(3, BodyRegion::Neck);
        let verts = labels.vertices_in(BodyRegion::Neck);
        assert_eq!(verts, vec![1, 3]);
        assert!(labels.vertices_in(BodyRegion::Head).is_empty());
    }

    #[test]
    fn test_to_bytes_from_bytes_roundtrip() {
        let mut labels = MeshLabels::new(5);
        labels.set(0, BodyRegion::Head);
        labels.set(2, BodyRegion::Torso);
        labels.set(4, BodyRegion::FootRight);
        let bytes = labels.to_bytes();
        assert_eq!(bytes.len(), 5);
        let restored = MeshLabels::from_bytes(&bytes);
        assert_eq!(restored.vertex_count, 5);
        for i in 0..5 {
            assert_eq!(restored.get(i), labels.get(i));
        }
    }

    #[test]
    fn test_to_vertex_colors_length() {
        let mut labels = MeshLabels::new(3);
        labels.set(0, BodyRegion::Head);
        labels.set(1, BodyRegion::Torso);
        let colors = labels.to_vertex_colors();
        assert_eq!(colors.len(), 3);
        assert_eq!(colors[0], BodyRegion::Head.color_rgb());
        assert_eq!(colors[2], BodyRegion::Unknown.color_rgb());
    }

    #[test]
    fn test_region_bbox_none_when_empty() {
        let labels = MeshLabels::new(4);
        let positions = vec![[0.0f32; 3]; 4];
        assert!(labels.region_bbox(BodyRegion::Head, &positions).is_none());
    }

    #[test]
    fn test_region_bbox_and_centroid() {
        let mut labels = MeshLabels::new(3);
        labels.set(0, BodyRegion::Head);
        labels.set(1, BodyRegion::Head);
        labels.set(2, BodyRegion::Torso);
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 4.0, 0.0], [1.0, 1.0, 0.0]];
        let (mn, mx) = labels.region_bbox(BodyRegion::Head, &positions).unwrap();
        assert!((mn[0] - 0.0).abs() < 1e-6);
        assert!((mx[1] - 4.0).abs() < 1e-6);
        let c = labels
            .region_centroid(BodyRegion::Head, &positions)
            .unwrap();
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 2.0).abs() < 1e-6);
        assert!(labels
            .region_centroid(BodyRegion::Neck, &positions)
            .is_none());
    }

    #[test]
    fn test_smooth_labels() {
        // 4 vertices in a quad; 3 labeled Torso, 1 labeled Head.
        // After one smooth pass the Head should flip to Torso (majority).
        let mut labels = MeshLabels::new(4);
        labels.set(0, BodyRegion::Torso);
        labels.set(1, BodyRegion::Torso);
        labels.set(2, BodyRegion::Torso);
        labels.set(3, BodyRegion::Head);
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        labels.smooth(&indices, 3);
        // After smoothing the isolated Head vertex should become Torso
        assert_eq!(labels.get(3), BodyRegion::Torso);
    }

    // ------------------------------------------------------------------
    // label_by_height tests
    // ------------------------------------------------------------------

    #[test]
    fn test_label_by_height_basic() {
        let mesh = make_body_mesh();
        let labels = label_by_height(&mesh);
        assert_eq!(labels.vertex_count, mesh.positions.len());
        // Should be fully labeled (no Unknown for a tall mesh)
        assert!(labels.labeled_fraction() > 0.8);
    }

    #[test]
    fn test_label_by_height_top_is_head() {
        let mesh = make_body_mesh();
        let labels = label_by_height(&mesh);
        let n = mesh.positions.len();
        // topmost vertices (last 5) should all be Head
        for vi in (n - 5)..n {
            assert_eq!(
                labels.get(vi),
                BodyRegion::Head,
                "vertex {} should be Head",
                vi
            );
        }
    }

    #[test]
    fn test_label_by_height_bottom_is_foot() {
        let mesh = make_body_mesh();
        let labels = label_by_height(&mesh);
        // bottommost 5 vertices should be feet
        for vi in 0..5 {
            let lbl = labels.get(vi);
            assert!(
                lbl == BodyRegion::FootLeft || lbl == BodyRegion::FootRight,
                "vertex {} label {:?} should be a foot",
                vi,
                lbl
            );
        }
    }

    #[test]
    fn test_label_by_height_flat_mesh_all_torso() {
        let flat = MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.5, 0.0], [1.0, 0.5, 0.0], [0.5, 0.5, 1.0]],
            normals: vec![[0.0, 1.0, 0.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        });
        let labels = label_by_height(&flat);
        for vi in 0..3 {
            assert_eq!(labels.get(vi), BodyRegion::Torso);
        }
    }

    // ------------------------------------------------------------------
    // flood_fill_label tests
    // ------------------------------------------------------------------

    #[test]
    fn test_flood_fill_labels_entire_flat_mesh() {
        let mesh = make_test_mesh();
        let mut labels = MeshLabels::new(4);
        // All normals are the same (Z-up), so 180-degree threshold fills all
        flood_fill_label(&mesh, 0, BodyRegion::Torso, 180.0, &mut labels);
        for vi in 0..4 {
            assert_eq!(labels.get(vi), BodyRegion::Torso);
        }
    }

    #[test]
    fn test_flood_fill_out_of_range_seed() {
        let mesh = make_test_mesh();
        let mut labels = MeshLabels::new(4);
        // Should not panic
        flood_fill_label(&mesh, 999, BodyRegion::Head, 90.0, &mut labels);
        // Nothing should be labeled
        assert_eq!(labels.labeled_fraction(), 0.0);
    }

    // ------------------------------------------------------------------
    // propagate_labels test
    // ------------------------------------------------------------------

    #[test]
    fn test_propagate_labels_fills_unknown() {
        let mesh = make_test_mesh();
        let mut labels = MeshLabels::new(4);
        // Only label vertex 0
        labels.set(0, BodyRegion::Torso);
        propagate_labels(&mesh, &mut labels);
        // All connected vertices should be filled
        assert!(labels.labeled_fraction() > 0.5);
    }

    // ------------------------------------------------------------------
    // body_seed_vertices test
    // ------------------------------------------------------------------

    #[test]
    fn test_body_seed_vertices_returns_15() {
        let mesh = make_body_mesh();
        let seeds = body_seed_vertices(&mesh);
        assert_eq!(
            seeds.len(),
            15,
            "expected 15 seeds (no Unknown), got {}",
            seeds.len()
        );
    }

    #[test]
    fn test_body_seed_vertices_valid_indices() {
        let mesh = make_body_mesh();
        let seeds = body_seed_vertices(&mesh);
        let n = mesh.positions.len();
        for (region, vi) in &seeds {
            assert!(
                *vi < n,
                "seed vertex {} for {:?} is out of range",
                vi,
                region
            );
        }
    }

    #[test]
    fn test_body_seed_empty_mesh() {
        let empty = MeshBuffers::from_morph(MB {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        });
        assert!(body_seed_vertices(&empty).is_empty());
    }

    // ------------------------------------------------------------------
    // region_boundary_edges tests
    // ------------------------------------------------------------------

    #[test]
    fn test_boundary_edges_same_label_no_boundary() {
        let mut labels = MeshLabels::new(4);
        for vi in 0..4 {
            labels.set(vi, BodyRegion::Torso);
        }
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let edges = region_boundary_edges(&labels, &indices);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_boundary_edges_two_regions() {
        let mut labels = MeshLabels::new(4);
        labels.set(0, BodyRegion::Head);
        labels.set(1, BodyRegion::Head);
        labels.set(2, BodyRegion::Torso);
        labels.set(3, BodyRegion::Torso);
        // Triangle 0-1-2 spans Head/Torso → 2 boundary edges
        // Triangle 0-2-3 spans Head/Torso → 2 boundary edges (some shared)
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let edges = region_boundary_edges(&labels, &indices);
        assert!(!edges.is_empty());
        // All returned edges should connect vertices with different labels
        for (a, b) in &edges {
            assert_ne!(labels.get(*a as usize), labels.get(*b as usize));
        }
    }

    #[test]
    fn test_boundary_edges_deduplicated() {
        // Two triangles sharing an edge between two regions
        let mut labels = MeshLabels::new(4);
        labels.set(0, BodyRegion::Head);
        labels.set(1, BodyRegion::Head);
        labels.set(2, BodyRegion::Torso);
        labels.set(3, BodyRegion::Torso);
        let indices: Vec<u32> = vec![0, 1, 2, 1, 3, 2];
        let edges = region_boundary_edges(&labels, &indices);
        // Ensure no duplicate edges
        let mut sorted = edges.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(edges.len(), sorted.len(), "boundary edges have duplicates");
    }

    // ------------------------------------------------------------------
    // File I/O test (writes to /tmp/)
    // ------------------------------------------------------------------

    #[test]
    fn test_write_label_bytes_to_file() {
        let mut labels = MeshLabels::new(8);
        for vi in 0..8 {
            labels.set(vi, BodyRegion::from_u8((vi % 16) as u8));
        }
        let bytes = labels.to_bytes();
        std::fs::write("/tmp/oxihuman_mesh_label_test.bin", &bytes).expect("write failed");
        let read_back = std::fs::read("/tmp/oxihuman_mesh_label_test.bin").expect("read failed");
        let restored = MeshLabels::from_bytes(&read_back);
        for vi in 0..8 {
            assert_eq!(restored.get(vi), labels.get(vi));
        }
    }
}
