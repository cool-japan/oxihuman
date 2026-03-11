// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Disconnected mesh island detection using Union-Find (disjoint set union).
//!
//! Each "island" is a set of vertices that are mutually reachable via edges.
//! Faces are used to derive edges; any face with all vertices in the same
//! component belongs to that island.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IslandConfig {
    /// Merge islands whose bounding-box centroids are closer than this.
    pub merge_distance: f32,
    /// Whether to sort the output islands by decreasing vertex count.
    pub sort_descending: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshIsland {
    /// Vertex indices belonging to this island.
    pub vertices: Vec<usize>,
    /// Face indices (into the original face list) belonging to this island.
    pub faces: Vec<usize>,
    /// Axis-aligned bounding box: \[min_x, min_y, min_z, max_x, max_y, max_z\].
    pub aabb: [f32; 6],
    /// Centroid of all vertex positions.
    pub centroid: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IslandResult {
    pub islands: Vec<MeshIsland>,
}

// ── Union-Find ───────────────────────────────────────────────────────────────

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Return the default island detection configuration.
#[allow(dead_code)]
pub fn default_island_config() -> IslandConfig {
    IslandConfig {
        merge_distance: 0.0,
        sort_descending: true,
    }
}

/// Detect disconnected islands in a triangle mesh.
///
/// * `positions` – flat \[x, y, z, x, y, z, …\] vertex positions (len = 3 × V).
/// * `indices`   – flat triangle indices \[a, b, c, a, b, c, …\] (len = 3 × F).
#[allow(dead_code)]
pub fn detect_islands(positions: &[f32], indices: &[usize], config: &IslandConfig) -> IslandResult {
    let vertex_count = positions.len() / 3;
    if vertex_count == 0 {
        return IslandResult { islands: vec![] };
    }

    let mut uf = UnionFind::new(vertex_count);
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        uf.union(a, b);
        uf.union(b, c);
    }

    // Group vertices by root.
    let mut root_map: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for v in 0..vertex_count {
        let root = uf.find(v);
        root_map.entry(root).or_default().push(v);
    }

    // Build island objects.
    let mut islands: Vec<MeshIsland> = root_map
        .values()
        .map(|verts| {
            let aabb = compute_vertex_aabb(positions, verts);
            let centroid = compute_centroid(positions, verts);
            MeshIsland {
                vertices: verts.clone(),
                faces: vec![],
                aabb,
                centroid,
            }
        })
        .collect();

    // Assign faces.
    for f in 0..face_count {
        let a = indices[f * 3];
        let root = uf.find(a);
        // Find the island whose representative root matches.
        if let Some(isl) = islands
            .iter_mut()
            .find(|i| !i.vertices.is_empty() && uf.find(i.vertices[0]) == root)
        {
            isl.faces.push(f);
        }
    }

    if config.sort_descending {
        islands.sort_by(|a, b| b.vertices.len().cmp(&a.vertices.len()));
    }

    IslandResult { islands }
}

/// Return the number of islands in a result.
#[allow(dead_code)]
pub fn island_count(result: &IslandResult) -> usize {
    result.islands.len()
}

/// Return a reference to the largest island (most vertices), or `None`.
#[allow(dead_code)]
pub fn largest_island(result: &IslandResult) -> Option<&MeshIsland> {
    result.islands.iter().max_by_key(|i| i.vertices.len())
}

/// Return a reference to the smallest island (fewest vertices), or `None`.
#[allow(dead_code)]
pub fn smallest_island(result: &IslandResult) -> Option<&MeshIsland> {
    result.islands.iter().min_by_key(|i| i.vertices.len())
}

/// Return the vertex count of a specific island.
#[allow(dead_code)]
pub fn island_vertex_count(island: &MeshIsland) -> usize {
    island.vertices.len()
}

/// Return the face count of a specific island.
#[allow(dead_code)]
pub fn island_face_count(island: &MeshIsland) -> usize {
    island.faces.len()
}

/// Return the AABB of an island as `[min_x, min_y, min_z, max_x, max_y, max_z]`.
#[allow(dead_code)]
pub fn island_bounding_box(island: &MeshIsland) -> [f32; 6] {
    island.aabb
}

/// Merge islands whose centroids are within `distance` of each other.
///
/// This does *not* re-run Union-Find; it simply concatenates island data.
#[allow(dead_code)]
pub fn merge_nearby_islands(result: &mut IslandResult, distance: f32) {
    let n = result.islands.len();
    if n < 2 {
        return;
    }

    let mut merged = vec![false; n];
    let mut out: Vec<MeshIsland> = Vec::new();

    for i in 0..n {
        if merged[i] {
            continue;
        }
        let mut combined_verts = result.islands[i].vertices.clone();
        let mut combined_faces = result.islands[i].faces.clone();
        for (m, isl) in merged[(i + 1)..]
            .iter_mut()
            .zip(result.islands[(i + 1)..].iter())
        {
            if *m {
                continue;
            }
            let d = centroid_distance(result.islands[i].centroid, isl.centroid);
            if d <= distance {
                combined_verts.extend_from_slice(&isl.vertices);
                combined_faces.extend_from_slice(&isl.faces);
                *m = true;
            }
        }
        // Recompute aabb / centroid from combined vertex lists without positions
        // (approximation using sub-island aabbs).
        let aabb = merge_aabbs(
            result.islands[i].aabb,
            result.islands[i..n]
                .iter()
                .enumerate()
                .filter(|(k, _)| merged[i + k] || *k == 0)
                .map(|(_, isl)| isl.aabb)
                .fold(result.islands[i].aabb, merge_aabbs_pair),
        );
        let centroid = [
            (aabb[0] + aabb[3]) * 0.5,
            (aabb[1] + aabb[4]) * 0.5,
            (aabb[2] + aabb[5]) * 0.5,
        ];
        out.push(MeshIsland {
            vertices: combined_verts,
            faces: combined_faces,
            aabb,
            centroid,
        });
    }
    result.islands = out;
}

/// Return the centroid of an island.
#[allow(dead_code)]
pub fn island_centroid(island: &MeshIsland) -> [f32; 3] {
    island.centroid
}

/// Sort islands by vertex count.  `descending = true` → largest first.
#[allow(dead_code)]
pub fn sort_islands_by_size(result: &mut IslandResult, descending: bool) {
    if descending {
        result.islands.sort_by(|a, b| b.vertices.len().cmp(&a.vertices.len()));
    } else {
        result.islands.sort_by(|a, b| a.vertices.len().cmp(&b.vertices.len()));
    }
}

/// Serialize island statistics to a minimal JSON string.
#[allow(dead_code)]
pub fn islands_to_json(result: &IslandResult) -> String {
    let mut s = String::from("[");
    for (idx, isl) in result.islands.iter().enumerate() {
        if idx > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"index\":{},\"vertices\":{},\"faces\":{},\
             \"centroid\":[{:.4},{:.4},{:.4}]}}",
            idx,
            isl.vertices.len(),
            isl.faces.len(),
            isl.centroid[0],
            isl.centroid[1],
            isl.centroid[2],
        ));
    }
    s.push(']');
    s
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn compute_vertex_aabb(positions: &[f32], verts: &[usize]) -> [f32; 6] {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for &v in verts {
        for k in 0..3 {
            let val = positions[v * 3 + k];
            if val < min[k] {
                min[k] = val;
            }
            if val > max[k] {
                max[k] = val;
            }
        }
    }
    [min[0], min[1], min[2], max[0], max[1], max[2]]
}

fn compute_centroid(positions: &[f32], verts: &[usize]) -> [f32; 3] {
    if verts.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for &v in verts {
        for k in 0..3 {
            sum[k] += positions[v * 3 + k];
        }
    }
    let n = verts.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

fn centroid_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn merge_aabbs_pair(a: [f32; 6], b: [f32; 6]) -> [f32; 6] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].min(b[2]),
        a[3].max(b[3]),
        a[4].max(b[4]),
        a[5].max(b[5]),
    ]
}

fn merge_aabbs(a: [f32; 6], b: [f32; 6]) -> [f32; 6] {
    merge_aabbs_pair(a, b)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_triangle_positions() -> Vec<f32> {
        // Triangle 0: (0,0,0), (1,0,0), (0,1,0)
        // Triangle 1 (separate island): (5,5,5), (6,5,5), (5,6,5)
        vec![
            0.0, 0.0, 0.0, // v0
            1.0, 0.0, 0.0, // v1
            0.0, 1.0, 0.0, // v2
            5.0, 5.0, 5.0, // v3
            6.0, 5.0, 5.0, // v4
            5.0, 6.0, 5.0, // v5
        ]
    }

    fn two_triangle_indices() -> Vec<usize> {
        vec![0, 1, 2, 3, 4, 5]
    }

    #[test]
    fn test_default_island_config() {
        let cfg = default_island_config();
        assert_eq!(cfg.merge_distance, 0.0);
        assert!(cfg.sort_descending);
    }

    #[test]
    fn test_detect_two_islands() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        assert_eq!(island_count(&result), 2);
    }

    #[test]
    fn test_detect_single_island() {
        let pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let idx = vec![0, 1, 2];
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        assert_eq!(island_count(&result), 1);
    }

    #[test]
    fn test_detect_empty_mesh() {
        let cfg = default_island_config();
        let result = detect_islands(&[], &[], &cfg);
        assert_eq!(island_count(&result), 0);
    }

    #[test]
    fn test_largest_island() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        let largest = largest_island(&result).unwrap();
        assert_eq!(largest.vertices.len(), 3);
    }

    #[test]
    fn test_smallest_island() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        let smallest = smallest_island(&result).unwrap();
        assert_eq!(smallest.vertices.len(), 3);
    }

    #[test]
    fn test_island_vertex_count() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        let total: usize = result.islands.iter().map(island_vertex_count).sum();
        assert_eq!(total, 6);
    }

    #[test]
    fn test_island_face_count() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        let total: usize = result.islands.iter().map(island_face_count).sum();
        assert_eq!(total, 2);
    }

    #[test]
    fn test_island_bounding_box() {
        let pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let idx = vec![0, 1, 2];
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        let bb = island_bounding_box(&result.islands[0]);
        assert!(bb[0] <= bb[3]);
        assert!(bb[1] <= bb[4]);
    }

    #[test]
    fn test_island_centroid() {
        let pos = vec![0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 2.0, 0.0];
        let idx = vec![0, 1, 2];
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        let c = island_centroid(&result.islands[0]);
        assert!((c[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_sort_islands_by_size_ascending() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let mut cfg = default_island_config();
        cfg.sort_descending = false;
        let mut result = detect_islands(&pos, &idx, &cfg);
        sort_islands_by_size(&mut result, false);
        for w in result.islands.windows(2) {
            assert!(w[0].vertices.len() <= w[1].vertices.len());
        }
    }

    #[test]
    fn test_sort_islands_by_size_descending() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let mut result = detect_islands(&pos, &idx, &cfg);
        sort_islands_by_size(&mut result, true);
        for w in result.islands.windows(2) {
            assert!(w[0].vertices.len() >= w[1].vertices.len());
        }
    }

    #[test]
    fn test_islands_to_json_non_empty() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        let json = islands_to_json(&result);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_islands_to_json_empty() {
        let result = IslandResult { islands: vec![] };
        let json = islands_to_json(&result);
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_merge_nearby_islands_no_merge() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let mut result = detect_islands(&pos, &idx, &cfg);
        // Islands are ~8 units apart; don't merge with distance 1.0.
        merge_nearby_islands(&mut result, 1.0);
        assert_eq!(island_count(&result), 2);
    }

    #[test]
    fn test_merge_nearby_islands_merge_all() {
        let pos = two_triangle_positions();
        let idx = two_triangle_indices();
        let cfg = default_island_config();
        let mut result = detect_islands(&pos, &idx, &cfg);
        merge_nearby_islands(&mut result, 100.0);
        assert_eq!(island_count(&result), 1);
    }

    #[test]
    fn test_connected_quad_mesh_one_island() {
        // Two triangles sharing edge v1-v2 → one island.
        let pos = vec![
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0,
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        let cfg = default_island_config();
        let result = detect_islands(&pos, &idx, &cfg);
        assert_eq!(island_count(&result), 1);
        assert_eq!(result.islands[0].vertices.len(), 4);
    }
}
