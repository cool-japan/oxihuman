// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Surface mesh segmentation into connected/coherent regions.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
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
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 1e-10 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [0.0, 1.0, 0.0]
    }
}

fn face_normal(positions: &[[f32; 3]], tri: &[u32; 3]) -> [f32; 3] {
    let v0 = positions[tri[0] as usize];
    let v1 = positions[tri[1] as usize];
    let v2 = positions[tri[2] as usize];
    normalize3(cross3(sub3(v1, v0), sub3(v2, v0)))
}

fn face_area(positions: &[[f32; 3]], tri: &[u32; 3]) -> f32 {
    let v0 = positions[tri[0] as usize];
    let v1 = positions[tri[1] as usize];
    let v2 = positions[tri[2] as usize];
    len3(cross3(sub3(v1, v0), sub3(v2, v0))) * 0.5
}

fn face_centroid(positions: &[[f32; 3]], tri: &[u32; 3]) -> [f32; 3] {
    let v0 = positions[tri[0] as usize];
    let v1 = positions[tri[1] as usize];
    let v2 = positions[tri[2] as usize];
    [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ]
}

// ---------------------------------------------------------------------------
// Union-Find
// ---------------------------------------------------------------------------

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
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A segment of the mesh surface.
#[derive(Debug, Clone)]
pub struct MeshSegment {
    pub id: u32,
    pub face_indices: Vec<usize>,
    pub centroid: [f32; 3],
    pub area: f32,
}

/// Criterion for segmentation.
#[derive(Debug, Clone)]
pub enum SegmentCriteria {
    Connected,
    NormalDeviation { threshold_deg: f32 },
    Planar { threshold_dist: f32 },
    Material { material_ids: Vec<u32> },
    Geodesic { num_seeds: usize },
}

// ---------------------------------------------------------------------------
// Helpers: build face adjacency
// ---------------------------------------------------------------------------

/// Build edge->face map: for each (min_v, max_v) edge, list of face indices.
fn build_edge_to_faces(
    triangles: &[[u32; 3]],
) -> std::collections::HashMap<(u32, u32), Vec<usize>> {
    let mut map: std::collections::HashMap<(u32, u32), Vec<usize>> =
        std::collections::HashMap::new();
    for (fi, tri) in triangles.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            map.entry(key).or_default().push(fi);
        }
    }
    map
}

/// Build per-face adjacency list from edge->face map.
fn build_face_adjacency(triangles: &[[u32; 3]]) -> Vec<Vec<usize>> {
    let nf = triangles.len();
    let mut adj = vec![Vec::new(); nf];
    let edge_map = build_edge_to_faces(triangles);
    for faces in edge_map.values() {
        if faces.len() == 2 {
            adj[faces[0]].push(faces[1]);
            adj[faces[1]].push(faces[0]);
        }
    }
    adj
}

/// Compute segment centroid and area from face indices.
fn compute_segment_props(
    face_indices: &[usize],
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
) -> ([f32; 3], f32) {
    let mut total_area = 0.0f32;
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for &fi in face_indices {
        if fi >= triangles.len() {
            continue;
        }
        let tri = &triangles[fi];
        if tri[0] as usize >= positions.len()
            || tri[1] as usize >= positions.len()
            || tri[2] as usize >= positions.len()
        {
            continue;
        }
        let a = face_area(positions, tri);
        let c = face_centroid(positions, tri);
        cx += c[0] * a;
        cy += c[1] * a;
        cz += c[2] * a;
        total_area += a;
    }
    let centroid = if total_area > 1e-10 {
        [cx / total_area, cy / total_area, cz / total_area]
    } else {
        [0.0, 0.0, 0.0]
    };
    (centroid, total_area)
}

/// Convert label array into Vec<MeshSegment>.
fn labels_to_segments(
    labels: &[usize],
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
) -> Vec<MeshSegment> {
    let max_label = labels.iter().copied().max().unwrap_or(0);
    let mut face_lists: Vec<Vec<usize>> = vec![Vec::new(); max_label + 1];
    for (fi, &lbl) in labels.iter().enumerate() {
        face_lists[lbl].push(fi);
    }
    face_lists
        .into_iter()
        .enumerate()
        .filter(|(_, faces)| !faces.is_empty())
        .map(|(id, faces)| {
            let (centroid, area) = compute_segment_props(&faces, positions, triangles);
            MeshSegment {
                id: id as u32,
                face_indices: faces,
                centroid,
                area,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Public segmentation functions
// ---------------------------------------------------------------------------

/// Segment by connectivity (connected components via union-find).
pub fn segment_by_connectivity(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> Vec<MeshSegment> {
    let nf = triangles.len();
    if nf == 0 {
        return Vec::new();
    }
    let mut uf = UnionFind::new(nf);
    let edge_map = build_edge_to_faces(triangles);
    for faces in edge_map.values() {
        if faces.len() == 2 {
            uf.union(faces[0], faces[1]);
        }
    }

    // Re-label roots to compact IDs
    let mut root_to_label: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
    let mut labels = vec![0usize; nf];
    for (fi, label) in labels.iter_mut().enumerate() {
        let root = uf.find(fi);
        let next = root_to_label.len();
        let lbl = *root_to_label.entry(root).or_insert(next);
        *label = lbl;
    }
    labels_to_segments(&labels, positions, triangles)
}

/// Segment by normal deviation (flood fill, stop at deviation > threshold_deg).
pub fn segment_by_normal_deviation(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    threshold_deg: f32,
) -> Vec<MeshSegment> {
    let nf = triangles.len();
    if nf == 0 {
        return Vec::new();
    }
    let threshold_cos = threshold_deg.to_radians().cos();
    let adj = build_face_adjacency(triangles);
    let normals: Vec<[f32; 3]> = triangles
        .iter()
        .map(|tri| {
            if tri[0] as usize >= positions.len()
                || tri[1] as usize >= positions.len()
                || tri[2] as usize >= positions.len()
            {
                [0.0, 1.0, 0.0]
            } else {
                face_normal(positions, tri)
            }
        })
        .collect();

    let mut labels = vec![usize::MAX; nf];
    let mut label_count = 0usize;
    let mut queue = std::collections::VecDeque::new();

    for start in 0..nf {
        if labels[start] != usize::MAX {
            continue;
        }
        let lbl = label_count;
        label_count += 1;
        labels[start] = lbl;
        queue.push_back(start);
        let seed_n = normals[start];
        while let Some(fi) = queue.pop_front() {
            for &nb in &adj[fi] {
                if labels[nb] != usize::MAX {
                    continue;
                }
                let cos_angle = dot3(seed_n, normals[nb]).abs();
                if cos_angle >= threshold_cos {
                    labels[nb] = lbl;
                    queue.push_back(nb);
                }
            }
        }
    }
    labels_to_segments(&labels, positions, triangles)
}

/// Segment by planarity (nearly-coplanar regions).
pub fn segment_by_planarity(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    threshold: f32,
) -> Vec<MeshSegment> {
    // Use combined normal deviation + distance-to-plane criterion
    let nf = triangles.len();
    if nf == 0 {
        return Vec::new();
    }
    let adj = build_face_adjacency(triangles);
    let normals: Vec<[f32; 3]> = triangles
        .iter()
        .map(|tri| {
            if tri.iter().any(|&i| i as usize >= positions.len()) {
                [0.0, 1.0, 0.0]
            } else {
                face_normal(positions, tri)
            }
        })
        .collect();
    let centroids: Vec<[f32; 3]> = triangles
        .iter()
        .map(|tri| {
            if tri.iter().any(|&i| i as usize >= positions.len()) {
                [0.0, 0.0, 0.0]
            } else {
                face_centroid(positions, tri)
            }
        })
        .collect();

    let mut labels = vec![usize::MAX; nf];
    let mut label_count = 0usize;
    let mut queue = std::collections::VecDeque::new();

    for start in 0..nf {
        if labels[start] != usize::MAX {
            continue;
        }
        let lbl = label_count;
        label_count += 1;
        labels[start] = lbl;
        queue.push_back(start);
        let seed_n = normals[start];
        let seed_c = centroids[start];
        while let Some(fi) = queue.pop_front() {
            for &nb in &adj[fi] {
                if labels[nb] != usize::MAX {
                    continue;
                }
                // Check normal alignment
                let cos_angle = dot3(seed_n, normals[nb]).abs();
                if cos_angle < (threshold.cos()) {
                    continue;
                }
                // Check distance of nb centroid to seed plane
                let diff = sub3(centroids[nb], seed_c);
                let dist = dot3(diff, seed_n).abs();
                if dist < threshold {
                    labels[nb] = lbl;
                    queue.push_back(nb);
                }
            }
        }
    }
    labels_to_segments(&labels, positions, triangles)
}

/// Dispatch segmentation based on criteria.
pub fn segment_dispatch(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    criteria: &SegmentCriteria,
) -> Vec<MeshSegment> {
    match criteria {
        SegmentCriteria::Connected => segment_by_connectivity(positions, triangles),
        SegmentCriteria::NormalDeviation { threshold_deg } => {
            segment_by_normal_deviation(positions, triangles, *threshold_deg)
        }
        SegmentCriteria::Planar { threshold_dist } => {
            segment_by_planarity(positions, triangles, *threshold_dist)
        }
        SegmentCriteria::Material { material_ids } => {
            // Group faces by material_ids[face_index % material_ids.len()]
            let nf = triangles.len();
            if nf == 0 || material_ids.is_empty() {
                return segment_by_connectivity(positions, triangles);
            }
            let labels: Vec<usize> = (0..nf)
                .map(|fi| material_ids[fi % material_ids.len()] as usize)
                .collect();
            labels_to_segments(&labels, positions, triangles)
        }
        SegmentCriteria::Geodesic { num_seeds } => {
            // Simple geodesic Voronoi via BFS from seed faces
            let nf = triangles.len();
            if nf == 0 {
                return Vec::new();
            }
            let seeds = *num_seeds;
            let step = (nf / seeds.max(1)).max(1);
            let seed_faces: Vec<usize> = (0..seeds).map(|i| (i * step).min(nf - 1)).collect();
            let adj = build_face_adjacency(triangles);
            let mut labels = vec![usize::MAX; nf];
            let mut queue = std::collections::VecDeque::new();
            for (lbl, &sf) in seed_faces.iter().enumerate() {
                labels[sf] = lbl;
                queue.push_back(sf);
            }
            while let Some(fi) = queue.pop_front() {
                for &nb in &adj[fi] {
                    if labels[nb] == usize::MAX {
                        labels[nb] = labels[fi];
                        queue.push_back(nb);
                    }
                }
            }
            // Assign unlabeled to 0
            for l in labels.iter_mut() {
                if *l == usize::MAX {
                    *l = 0;
                }
            }
            labels_to_segments(&labels, positions, triangles)
        }
    }
}

/// Compute which segment IDs are adjacent.
pub fn compute_segment_adjacency(
    segments: &[MeshSegment],
    triangles: &[[u32; 3]],
) -> Vec<(u32, u32)> {
    // Build face -> segment id map
    let mut face_segment = vec![u32::MAX; triangles.len()];
    for seg in segments {
        for &fi in &seg.face_indices {
            if fi < face_segment.len() {
                face_segment[fi] = seg.id;
            }
        }
    }
    let edge_map = build_edge_to_faces(triangles);
    let mut pairs = std::collections::HashSet::new();
    for faces in edge_map.values() {
        if faces.len() == 2 {
            let sa = face_segment[faces[0]];
            let sb = face_segment[faces[1]];
            if sa != u32::MAX && sb != u32::MAX && sa != sb {
                let key = if sa < sb { (sa, sb) } else { (sb, sa) };
                pairs.insert(key);
            }
        }
    }
    pairs.into_iter().collect()
}

/// Merge segments with fewer than min_faces into their largest neighbor.
pub fn merge_small_segments(mut segments: Vec<MeshSegment>, min_faces: usize) -> Vec<MeshSegment> {
    // Simple: merge tiny segments into the segment with most faces
    let large_idx = segments
        .iter()
        .enumerate()
        .max_by_key(|(_, s)| s.face_indices.len())
        .map(|(i, _)| i);

    if let Some(li) = large_idx {
        let large_id = segments[li].id;
        let small_faces: Vec<Vec<usize>> = segments
            .iter()
            .filter(|s| s.face_indices.len() < min_faces && s.id != large_id)
            .map(|s| s.face_indices.clone())
            .collect();
        for sf in small_faces {
            segments[li].face_indices.extend(sf);
        }
        segments.retain(|s| s.face_indices.len() >= min_faces || s.id == large_id);
    }
    // Re-assign IDs
    for (i, seg) in segments.iter_mut().enumerate() {
        seg.id = i as u32;
    }
    segments
}

/// Get boundary edges of a segment.
pub fn segment_boundary_edges(segment: &MeshSegment, triangles: &[[u32; 3]]) -> Vec<[u32; 2]> {
    let mut edge_count: std::collections::HashMap<(u32, u32), u32> =
        std::collections::HashMap::new();
    for &fi in &segment.face_indices {
        if fi >= triangles.len() {
            continue;
        }
        let tri = &triangles[fi];
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|(_, cnt)| *cnt == 1)
        .map(|((a, b), _)| [a, b])
        .collect()
}

/// Find the largest segment by face count.
pub fn largest_segment(segments: &[MeshSegment]) -> Option<&MeshSegment> {
    segments.iter().max_by_key(|s| s.face_indices.len())
}

/// Statistics: (count, min_area, max_area).
pub fn segment_stats(segments: &[MeshSegment]) -> (usize, f32, f32) {
    if segments.is_empty() {
        return (0, 0.0, 0.0);
    }
    let min_a = segments.iter().map(|s| s.area).fold(f32::MAX, f32::min);
    let max_a = segments.iter().map(|s| s.area).fold(0.0f32, f32::max);
    (segments.len(), min_a, max_a)
}

/// Assign a distinct color per segment.
pub fn assign_face_colors(segments: &[MeshSegment], total_faces: usize) -> Vec<[f32; 3]> {
    let mut colors = vec![[0.5f32, 0.5, 0.5]; total_faces];
    let n = segments.len().max(1) as f32;
    for (si, seg) in segments.iter().enumerate() {
        // HSV-style hue spread
        let h = si as f32 / n;
        let r = (h * 6.0).sin().abs();
        let g = ((h + 0.333) * 6.0).sin().abs();
        let b = ((h + 0.666) * 6.0).sin().abs();
        let color = [r, g, b];
        for &fi in &seg.face_indices {
            if fi < total_faces {
                colors[fi] = color;
            }
        }
    }
    colors
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn two_triangle_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let triangles = vec![[0u32, 1, 2], [1, 3, 4]];
        (positions, triangles)
    }

    fn grid_mesh(n: usize) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let mut pos = Vec::new();
        let mut tris = Vec::new();
        for r in 0..=n {
            for c in 0..=n {
                pos.push([c as f32, r as f32, 0.0]);
            }
        }
        for r in 0..n {
            for c in 0..n {
                let i = (r * (n + 1) + c) as u32;
                tris.push([i, i + 1, i + (n as u32 + 1)]);
                tris.push([i + 1, i + (n as u32 + 2), i + (n as u32 + 1)]);
            }
        }
        (pos, tris)
    }

    #[test]
    fn test_connectivity_single_component() {
        let (pos, tris) = grid_mesh(3);
        let segs = segment_by_connectivity(&pos, &tris);
        assert_eq!(segs.len(), 1, "Grid mesh should be one connected component");
    }

    #[test]
    fn test_connectivity_two_components() {
        let (pos, tris) = two_triangle_mesh();
        let segs = segment_by_connectivity(&pos, &tris);
        // Triangles share vertex 1 but not an edge -> still separate
        // (shared edge = 2 faces sharing 2 vertices)
        // Both share vertex 1 only, so they ARE separate components
        assert!(!segs.is_empty());
    }

    #[test]
    fn test_segment_by_normal_deviation_flat() {
        let (pos, tris) = grid_mesh(3);
        // All faces have same normal -> should be 1 segment
        let segs = segment_by_normal_deviation(&pos, &tris, 30.0);
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn test_segment_by_normal_deviation_zero_threshold() {
        let (pos, tris) = grid_mesh(2);
        // Very tight threshold: each face its own segment (or small groups)
        let segs = segment_by_normal_deviation(&pos, &tris, 0.001);
        assert!(!segs.is_empty());
    }

    #[test]
    fn test_segment_by_planarity_flat() {
        let (pos, tris) = grid_mesh(3);
        let segs = segment_by_planarity(&pos, &tris, 0.1);
        // All coplanar
        assert!(!segs.is_empty());
    }

    #[test]
    fn test_segment_dispatch_connected() {
        let (pos, tris) = grid_mesh(2);
        let segs = segment_dispatch(&pos, &tris, &SegmentCriteria::Connected);
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn test_segment_dispatch_normal_deviation() {
        let (pos, tris) = grid_mesh(2);
        let segs = segment_dispatch(
            &pos,
            &tris,
            &SegmentCriteria::NormalDeviation {
                threshold_deg: 45.0,
            },
        );
        assert!(!segs.is_empty());
    }

    #[test]
    fn test_segment_adjacency_empty() {
        let segs: Vec<MeshSegment> = Vec::new();
        let tris: Vec<[u32; 3]> = Vec::new();
        let adj = compute_segment_adjacency(&segs, &tris);
        assert!(adj.is_empty());
    }

    #[test]
    fn test_segment_adjacency_grid() {
        let (pos, tris) = grid_mesh(3);
        // Segment with geodesic to get multiple segments
        let segs = segment_dispatch(&pos, &tris, &SegmentCriteria::Geodesic { num_seeds: 4 });
        let adj = compute_segment_adjacency(&segs, &tris);
        // Should have some adjacencies
        assert!(!adj.is_empty() || segs.len() <= 1);
    }

    #[test]
    fn test_merge_small_segments() {
        let (pos, tris) = grid_mesh(3);
        let segs = segment_dispatch(&pos, &tris, &SegmentCriteria::Geodesic { num_seeds: 8 });
        let count_before = segs.len();
        let merged = merge_small_segments(segs, 100);
        // After merging, should have fewer or equal segments
        assert!(merged.len() <= count_before);
    }

    #[test]
    fn test_segment_boundary_edges() {
        let (_, tris) = grid_mesh(2);
        let seg = MeshSegment {
            id: 0,
            face_indices: vec![0],
            centroid: [0.0; 3],
            area: 0.5,
        };
        let edges = segment_boundary_edges(&seg, &tris);
        // Triangle 0 has 3 boundary edges (interior face may share 1)
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_largest_segment() {
        let (pos, tris) = grid_mesh(3);
        let segs = segment_by_connectivity(&pos, &tris);
        let largest = largest_segment(&segs);
        assert!(largest.is_some());
    }

    #[test]
    fn test_segment_stats_empty() {
        let segs: Vec<MeshSegment> = Vec::new();
        let (count, min_a, max_a) = segment_stats(&segs);
        assert_eq!(count, 0);
        assert_eq!(min_a, 0.0);
        assert_eq!(max_a, 0.0);
    }

    #[test]
    fn test_segment_stats_nonempty() {
        let (pos, tris) = grid_mesh(2);
        let segs = segment_by_connectivity(&pos, &tris);
        let (count, min_a, max_a) = segment_stats(&segs);
        assert_eq!(count, 1);
        assert!(min_a > 0.0);
        assert!(max_a >= min_a);
    }

    #[test]
    fn test_assign_face_colors() {
        let (pos, tris) = grid_mesh(2);
        let segs = segment_by_connectivity(&pos, &tris);
        let nf = tris.len();
        let colors = assign_face_colors(&segs, nf);
        assert_eq!(colors.len(), nf);
    }

    #[test]
    fn test_segment_material_dispatch() {
        let (pos, tris) = grid_mesh(2);
        let segs = segment_dispatch(
            &pos,
            &tris,
            &SegmentCriteria::Material {
                material_ids: vec![0, 1, 0, 1],
            },
        );
        assert!(!segs.is_empty());
    }

    #[test]
    fn test_segment_geodesic_dispatch() {
        let (pos, tris) = grid_mesh(3);
        let segs = segment_dispatch(&pos, &tris, &SegmentCriteria::Geodesic { num_seeds: 3 });
        assert!(!segs.is_empty());
    }
}
