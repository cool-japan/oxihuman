// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

use oxihuman_core::parser::target::TargetFile;

/// For a single vertex: which targets affect it and their delta magnitudes.
#[derive(Debug, Clone)]
pub struct VertexInfluence {
    pub vertex_id: u32,
    /// (target_name, delta_magnitude) sorted by magnitude descending.
    pub influences: Vec<(String, f32)>,
}

impl VertexInfluence {
    /// Total influence magnitude across all targets.
    pub fn total_magnitude(&self) -> f32 {
        self.influences.iter().map(|(_, m)| m).sum()
    }

    /// Name of the strongest influencing target, or None if no influences.
    pub fn dominant_target(&self) -> Option<&str> {
        self.influences.first().map(|(name, _)| name.as_str())
    }
}

/// Map from vertex_id to VertexInfluence.
/// Built from a collection of TargetFile objects.
#[derive(Debug)]
pub struct InfluenceMap {
    pub vertex_count: usize,
    influences: HashMap<u32, VertexInfluence>,
}

impl InfluenceMap {
    /// Build from a list of (name, target) pairs.
    pub fn build(targets: &[(&str, &TargetFile)]) -> Self {
        let mut map: HashMap<u32, Vec<(String, f32)>> = HashMap::new();

        for (name, target) in targets {
            for delta in &target.deltas {
                let mag = (delta.dx * delta.dx + delta.dy * delta.dy + delta.dz * delta.dz).sqrt();
                map.entry(delta.vid)
                    .or_default()
                    .push((name.to_string(), mag));
            }
        }

        let mut influences: HashMap<u32, VertexInfluence> = HashMap::new();
        for (vid, mut infl_list) in map {
            infl_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            influences.insert(
                vid,
                VertexInfluence {
                    vertex_id: vid,
                    influences: infl_list,
                },
            );
        }

        let vertex_count = influences.len();
        Self {
            vertex_count,
            influences,
        }
    }

    /// Get influence info for a specific vertex. Returns None if vertex is unaffected.
    pub fn get(&self, vertex_id: u32) -> Option<&VertexInfluence> {
        self.influences.get(&vertex_id)
    }

    /// Number of vertices that are affected by at least one target.
    pub fn affected_vertex_count(&self) -> usize {
        self.influences.len()
    }

    /// Iterate over all VertexInfluence entries.
    pub fn iter(&self) -> impl Iterator<Item = &VertexInfluence> {
        self.influences.values()
    }

    /// Find the top-N most-influenced vertices (by total magnitude).
    pub fn top_vertices(&self, n: usize) -> Vec<&VertexInfluence> {
        let mut all: Vec<&VertexInfluence> = self.influences.values().collect();
        all.sort_by(|a, b| {
            b.total_magnitude()
                .partial_cmp(&a.total_magnitude())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all.truncate(n);
        all
    }

    /// Find all vertices affected by a specific target.
    pub fn vertices_for_target(&self, target_name: &str) -> Vec<u32> {
        let mut result: Vec<u32> = self
            .influences
            .values()
            .filter(|vi| vi.influences.iter().any(|(name, _)| name == target_name))
            .map(|vi| vi.vertex_id)
            .collect();
        result.sort_unstable();
        result
    }

    /// Find all targets that affect a specific vertex.
    pub fn targets_for_vertex(&self, vertex_id: u32) -> Vec<(&str, f32)> {
        self.influences
            .get(&vertex_id)
            .map(|vi| {
                vi.influences
                    .iter()
                    .map(|(name, mag)| (name.as_str(), *mag))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Compute per-target statistics: (target_name, vertex_count, total_delta_magnitude).
    pub fn target_stats(&self) -> Vec<(String, usize, f32)> {
        let mut stats: HashMap<String, (usize, f32)> = HashMap::new();
        for vi in self.influences.values() {
            for (name, mag) in &vi.influences {
                let entry = stats.entry(name.clone()).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += mag;
            }
        }
        let mut result: Vec<(String, usize, f32)> = stats
            .into_iter()
            .map(|(name, (count, total))| (name, count, total))
            .collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Find "isolated" vertices: affected by only 1 target.
    pub fn isolated_vertices(&self) -> Vec<u32> {
        let mut result: Vec<u32> = self
            .influences
            .values()
            .filter(|vi| vi.influences.len() == 1)
            .map(|vi| vi.vertex_id)
            .collect();
        result.sort_unstable();
        result
    }

    /// Find "shared" vertices: affected by N or more targets.
    pub fn shared_vertices(&self, min_targets: usize) -> Vec<u32> {
        let mut result: Vec<u32> = self
            .influences
            .values()
            .filter(|vi| vi.influences.len() >= min_targets)
            .map(|vi| vi.vertex_id)
            .collect();
        result.sort_unstable();
        result
    }
}

/// Summary statistics for an InfluenceMap.
#[derive(Debug, Clone)]
pub struct InfluenceMapStats {
    /// Total number of affected vertices.
    pub affected_vertices: usize,
    /// Total number of distinct targets referenced.
    pub target_count: usize,
    /// Average number of targets per affected vertex.
    pub avg_targets_per_vertex: f32,
    /// Maximum number of targets on a single vertex.
    pub max_targets_per_vertex: usize,
    /// Total sum of all delta magnitudes.
    pub total_magnitude: f32,
}

/// Convenience constructor: build an InfluenceMap from owned (name, TargetFile) pairs.
pub fn build_influence_map(targets: &[(&str, &TargetFile)]) -> InfluenceMap {
    InfluenceMap::build(targets)
}

/// Return the top-`n` (target_name, magnitude) pairs for a given vertex,
/// sorted descending by magnitude. Returns empty vec if vertex not found.
pub fn top_influences_for_vertex(
    map: &InfluenceMap,
    vertex_id: u32,
    n: usize,
) -> Vec<(String, f32)> {
    map.get(vertex_id)
        .map(|vi| vi.influences.iter().take(n).cloned().collect())
        .unwrap_or_default()
}

/// Fraction of `vertex_ids` that are covered (affected) by `target_name`.
/// Returns 0.0 if `vertex_ids` is empty.
pub fn target_vertex_coverage(map: &InfluenceMap, target_name: &str, vertex_ids: &[u32]) -> f32 {
    if vertex_ids.is_empty() {
        return 0.0;
    }
    let covered = vertex_ids
        .iter()
        .filter(|&&vid| {
            map.get(vid)
                .map(|vi| vi.influences.iter().any(|(n, _)| n == target_name))
                .unwrap_or(false)
        })
        .count();
    covered as f32 / vertex_ids.len() as f32
}

/// Jaccard overlap between two targets: |vertices(A) ∩ vertices(B)| / |vertices(A) ∪ vertices(B)|.
/// Returns 0.0 if both targets affect zero vertices.
pub fn vertex_target_overlap(map: &InfluenceMap, target_a: &str, target_b: &str) -> f32 {
    let set_a: std::collections::HashSet<u32> =
        map.vertices_for_target(target_a).into_iter().collect();
    let set_b: std::collections::HashSet<u32> =
        map.vertices_for_target(target_b).into_iter().collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Compute aggregate statistics for an InfluenceMap.
pub fn influence_map_stats(map: &InfluenceMap) -> InfluenceMapStats {
    let affected_vertices = map.affected_vertex_count();

    let mut total_targets_sum: usize = 0;
    let mut max_targets_per_vertex: usize = 0;

    for vi in map.iter() {
        let cnt = vi.influences.len();
        total_targets_sum += cnt;
        if cnt > max_targets_per_vertex {
            max_targets_per_vertex = cnt;
        }
    }

    let stats = map.target_stats();
    let target_count = stats.len();
    let total_magnitude: f32 = stats.iter().map(|(_, _, m)| m).sum();

    let avg_targets_per_vertex = if affected_vertices == 0 {
        0.0
    } else {
        total_targets_sum as f32 / affected_vertices as f32
    };

    InfluenceMapStats {
        affected_vertices,
        target_count,
        avg_targets_per_vertex,
        max_targets_per_vertex,
        total_magnitude,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_core::parser::target::{Delta, TargetFile};

    fn make_target(name: &str, deltas: Vec<Delta>) -> TargetFile {
        TargetFile {
            name: name.to_string(),
            deltas,
        }
    }

    fn delta(vid: u32, dx: f32, dy: f32, dz: f32) -> Delta {
        Delta { vid, dx, dy, dz }
    }

    // ── InfluenceMap::build ────────────────────────────────────────────────

    #[test]
    fn build_empty_no_vertices() {
        let map = InfluenceMap::build(&[]);
        assert_eq!(map.vertex_count, 0);
        assert_eq!(map.affected_vertex_count(), 0);
    }

    #[test]
    fn build_single_target_single_vertex() {
        let t = make_target("height", vec![delta(10, 1.0, 0.0, 0.0)]);
        let map = InfluenceMap::build(&[("height", &t)]);
        assert_eq!(map.vertex_count, 1);
        let vi = map.get(10).unwrap();
        assert_eq!(vi.vertex_id, 10);
        assert_eq!(vi.influences.len(), 1);
        assert_eq!(vi.influences[0].0, "height");
        assert!((vi.influences[0].1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn build_multiple_targets_same_vertex() {
        let t1 = make_target("m1", vec![delta(42, 1.0, 0.0, 0.0)]);
        let t2 = make_target("m2", vec![delta(42, 0.0, 1.0, 0.0)]);
        let t3 = make_target("m3", vec![delta(42, 0.0, 0.0, 1.0)]);
        let map = InfluenceMap::build(&[("m1", &t1), ("m2", &t2), ("m3", &t3)]);
        assert_eq!(map.vertex_count, 1);
        let vi = map.get(42).unwrap();
        assert_eq!(vi.influences.len(), 3);
    }

    // ── VertexInfluence helpers ────────────────────────────────────────────

    #[test]
    fn vertex_influence_total_magnitude() {
        let t1 = make_target("a", vec![delta(5, 3.0, 4.0, 0.0)]); // mag = 5.0
        let t2 = make_target("b", vec![delta(5, 0.0, 0.0, 2.0)]); // mag = 2.0
        let map = InfluenceMap::build(&[("a", &t1), ("b", &t2)]);
        let vi = map.get(5).unwrap();
        assert!((vi.total_magnitude() - 7.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_influence_dominant_target() {
        let t1 = make_target("small", vec![delta(7, 0.0, 0.0, 1.0)]); // mag = 1.0
        let t2 = make_target("large", vec![delta(7, 3.0, 4.0, 0.0)]); // mag = 5.0
        let map = InfluenceMap::build(&[("small", &t1), ("large", &t2)]);
        let vi = map.get(7).unwrap();
        assert_eq!(vi.dominant_target(), Some("large"));
    }

    #[test]
    fn dominant_target_none_for_empty() {
        let vi = VertexInfluence {
            vertex_id: 0,
            influences: vec![],
        };
        assert_eq!(vi.dominant_target(), None);
        assert!((vi.total_magnitude() - 0.0).abs() < 1e-9);
    }

    // ── affected_vertex_count & top_vertices ──────────────────────────────

    #[test]
    fn affected_vertex_count_correct() {
        let t = make_target(
            "t",
            vec![
                delta(1, 0.1, 0.0, 0.0),
                delta(2, 0.2, 0.0, 0.0),
                delta(3, 0.3, 0.0, 0.0),
            ],
        );
        let map = InfluenceMap::build(&[("t", &t)]);
        assert_eq!(map.affected_vertex_count(), 3);
    }

    #[test]
    fn top_vertices_sorted_desc() {
        let t = make_target(
            "t",
            vec![
                delta(1, 1.0, 0.0, 0.0), // mag 1
                delta(2, 3.0, 4.0, 0.0), // mag 5
                delta(3, 0.0, 2.0, 0.0), // mag 2
            ],
        );
        let map = InfluenceMap::build(&[("t", &t)]);
        let top = map.top_vertices(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].vertex_id, 2); // highest total mag = 5
        assert_eq!(top[1].vertex_id, 3); // second = 2
    }

    #[test]
    fn top_vertices_clamps_to_available() {
        let t = make_target("t", vec![delta(0, 1.0, 0.0, 0.0)]);
        let map = InfluenceMap::build(&[("t", &t)]);
        let top = map.top_vertices(100);
        assert_eq!(top.len(), 1);
    }

    // ── vertices_for_target & targets_for_vertex ──────────────────────────

    #[test]
    fn vertices_for_target_correct() {
        let t1 = make_target(
            "alpha",
            vec![delta(10, 1.0, 0.0, 0.0), delta(20, 1.0, 0.0, 0.0)],
        );
        let t2 = make_target(
            "beta",
            vec![delta(20, 0.5, 0.0, 0.0), delta(30, 0.5, 0.0, 0.0)],
        );
        let map = InfluenceMap::build(&[("alpha", &t1), ("beta", &t2)]);
        let verts_alpha = map.vertices_for_target("alpha");
        assert_eq!(verts_alpha, vec![10, 20]);
        let verts_beta = map.vertices_for_target("beta");
        assert_eq!(verts_beta, vec![20, 30]);
    }

    #[test]
    fn vertices_for_unknown_target_empty() {
        let t = make_target("real", vec![delta(1, 1.0, 0.0, 0.0)]);
        let map = InfluenceMap::build(&[("real", &t)]);
        assert!(map.vertices_for_target("ghost").is_empty());
    }

    #[test]
    fn targets_for_vertex_returns_all() {
        let t1 = make_target("x", vec![delta(99, 1.0, 0.0, 0.0)]);
        let t2 = make_target("y", vec![delta(99, 0.0, 2.0, 0.0)]);
        let t3 = make_target("z", vec![delta(99, 0.0, 0.0, 3.0)]);
        let map = InfluenceMap::build(&[("x", &t1), ("y", &t2), ("z", &t3)]);
        let targets = map.targets_for_vertex(99);
        assert_eq!(targets.len(), 3);
        let names: Vec<&str> = targets.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"x"));
        assert!(names.contains(&"y"));
        assert!(names.contains(&"z"));
    }

    #[test]
    fn targets_for_unknown_vertex_empty() {
        let map = InfluenceMap::build(&[]);
        assert!(map.targets_for_vertex(999).is_empty());
    }

    // ── target_stats ──────────────────────────────────────────────────────

    #[test]
    fn target_stats_vertex_count_correct() {
        let t1 = make_target("aa", vec![delta(1, 1.0, 0.0, 0.0), delta(2, 2.0, 0.0, 0.0)]);
        let t2 = make_target(
            "bb",
            vec![
                delta(2, 0.5, 0.0, 0.0),
                delta(3, 0.5, 0.0, 0.0),
                delta(4, 0.5, 0.0, 0.0),
            ],
        );
        let map = InfluenceMap::build(&[("aa", &t1), ("bb", &t2)]);
        let stats = map.target_stats();
        let aa = stats.iter().find(|(n, _, _)| n == "aa").unwrap();
        let bb = stats.iter().find(|(n, _, _)| n == "bb").unwrap();
        assert_eq!(aa.1, 2); // aa affects 2 vertices
        assert_eq!(bb.1, 3); // bb affects 3 vertices
        assert!((aa.2 - 3.0).abs() < 1e-5); // 1.0 + 2.0
        assert!((bb.2 - 1.5).abs() < 1e-5); // 0.5 + 0.5 + 0.5
    }

    // ── isolated_vertices & shared_vertices ───────────────────────────────

    #[test]
    fn isolated_vertices_single_target() {
        let t1 = make_target("only", vec![delta(5, 1.0, 0.0, 0.0)]);
        let t2 = make_target(
            "shared",
            vec![delta(5, 0.5, 0.0, 0.0), delta(6, 0.5, 0.0, 0.0)],
        );
        let map = InfluenceMap::build(&[("only", &t1), ("shared", &t2)]);
        let isolated = map.isolated_vertices();
        // vertex 5 has 2 targets, vertex 6 has 1 target
        assert_eq!(isolated, vec![6]);
    }

    #[test]
    fn shared_vertices_min_two() {
        let t1 = make_target("p", vec![delta(1, 1.0, 0.0, 0.0), delta(2, 1.0, 0.0, 0.0)]);
        let t2 = make_target("q", vec![delta(2, 1.0, 0.0, 0.0), delta(3, 1.0, 0.0, 0.0)]);
        let t3 = make_target("r", vec![delta(2, 1.0, 0.0, 0.0)]);
        let map = InfluenceMap::build(&[("p", &t1), ("q", &t2), ("r", &t3)]);
        let shared2 = map.shared_vertices(2);
        // vertex 2 has 3 targets, vertex 3 has 1, vertex 1 has 1
        assert_eq!(shared2, vec![2]);
        let shared3 = map.shared_vertices(3);
        assert_eq!(shared3, vec![2]);
        let shared4 = map.shared_vertices(4);
        assert!(shared4.is_empty());
    }

    // ── influences sorted by magnitude ────────────────────────────────────

    #[test]
    fn influences_sorted_by_magnitude() {
        let t_big = make_target("big", vec![delta(0, 3.0, 4.0, 0.0)]); // mag = 5.0
        let t_mid = make_target("mid", vec![delta(0, 0.0, 2.0, 0.0)]); // mag = 2.0
        let t_small = make_target("small", vec![delta(0, 1.0, 0.0, 0.0)]); // mag = 1.0
        let map = InfluenceMap::build(&[("small", &t_small), ("big", &t_big), ("mid", &t_mid)]);
        let vi = map.get(0).unwrap();
        assert_eq!(vi.influences.len(), 3);
        assert_eq!(vi.influences[0].0, "big");
        assert!((vi.influences[0].1 - 5.0).abs() < 1e-5);
        assert_eq!(vi.influences[1].0, "mid");
        assert!((vi.influences[1].1 - 2.0).abs() < 1e-5);
        assert_eq!(vi.influences[2].0, "small");
        assert!((vi.influences[2].1 - 1.0).abs() < 1e-5);
    }

    // ── Free functions ────────────────────────────────────────────────────

    #[test]
    fn build_influence_map_fn_equivalent() {
        let t = make_target("t", vec![delta(1, 1.0, 0.0, 0.0)]);
        let map = build_influence_map(&[("t", &t)]);
        assert_eq!(map.vertex_count, 1);
        assert!(map.get(1).is_some());
    }

    #[test]
    fn top_influences_for_vertex_returns_n() {
        let t1 = make_target("big", vec![delta(0, 3.0, 4.0, 0.0)]); // mag 5
        let t2 = make_target("mid", vec![delta(0, 0.0, 2.0, 0.0)]); // mag 2
        let t3 = make_target("small", vec![delta(0, 1.0, 0.0, 0.0)]); // mag 1
        let map = InfluenceMap::build(&[("big", &t1), ("mid", &t2), ("small", &t3)]);
        let top2 = top_influences_for_vertex(&map, 0, 2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].0, "big");
        assert_eq!(top2[1].0, "mid");
    }

    #[test]
    fn top_influences_for_unknown_vertex_empty() {
        let map = InfluenceMap::build(&[]);
        assert!(top_influences_for_vertex(&map, 999, 5).is_empty());
    }

    #[test]
    fn target_vertex_coverage_fraction() {
        let t1 = make_target(
            "cover",
            vec![delta(1, 1.0, 0.0, 0.0), delta(2, 1.0, 0.0, 0.0)],
        );
        let map = InfluenceMap::build(&[("cover", &t1)]);
        // vertices [1, 2, 3]: 3 is not covered
        let cov = target_vertex_coverage(&map, "cover", &[1, 2, 3]);
        assert!((cov - 2.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn target_vertex_coverage_empty_returns_zero() {
        let map = InfluenceMap::build(&[]);
        assert!((target_vertex_coverage(&map, "any", &[])).abs() < 1e-9);
    }

    #[test]
    fn vertex_target_overlap_identical_sets() {
        let t = make_target("t", vec![delta(1, 1.0, 0.0, 0.0), delta(2, 1.0, 0.0, 0.0)]);
        let map = InfluenceMap::build(&[("t", &t)]);
        // Same target compared against itself -> Jaccard = 1.0
        let overlap = vertex_target_overlap(&map, "t", "t");
        assert!((overlap - 1.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_target_overlap_disjoint_sets() {
        let t1 = make_target("a", vec![delta(1, 1.0, 0.0, 0.0)]);
        let t2 = make_target("b", vec![delta(2, 1.0, 0.0, 0.0)]);
        let map = InfluenceMap::build(&[("a", &t1), ("b", &t2)]);
        let overlap = vertex_target_overlap(&map, "a", "b");
        assert!((overlap - 0.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_target_overlap_partial() {
        // a: {1, 2}, b: {2, 3}  => intersection={2}, union={1,2,3} => Jaccard=1/3
        let t1 = make_target("a", vec![delta(1, 1.0, 0.0, 0.0), delta(2, 1.0, 0.0, 0.0)]);
        let t2 = make_target("b", vec![delta(2, 1.0, 0.0, 0.0), delta(3, 1.0, 0.0, 0.0)]);
        let map = InfluenceMap::build(&[("a", &t1), ("b", &t2)]);
        let overlap = vertex_target_overlap(&map, "a", "b");
        assert!((overlap - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn influence_map_stats_basic() {
        let t1 = make_target("x", vec![delta(1, 1.0, 0.0, 0.0), delta(2, 1.0, 0.0, 0.0)]);
        let t2 = make_target("y", vec![delta(2, 0.0, 1.0, 0.0), delta(3, 0.0, 0.0, 1.0)]);
        let map = InfluenceMap::build(&[("x", &t1), ("y", &t2)]);
        let stats = influence_map_stats(&map);
        assert_eq!(stats.affected_vertices, 3); // vertices 1, 2, 3
        assert_eq!(stats.target_count, 2); // targets x, y
                                           // total magnitude: x->v1=1, x->v2=1, y->v2=1, y->v3=1 => 4.0
        assert!((stats.total_magnitude - 4.0).abs() < 1e-5);
        // max_targets_per_vertex: vertex 2 has 2 targets
        assert_eq!(stats.max_targets_per_vertex, 2);
    }

    #[test]
    fn influence_map_stats_empty() {
        let map = InfluenceMap::build(&[]);
        let stats = influence_map_stats(&map);
        assert_eq!(stats.affected_vertices, 0);
        assert_eq!(stats.target_count, 0);
        assert!((stats.total_magnitude).abs() < 1e-9);
        assert!((stats.avg_targets_per_vertex).abs() < 1e-9);
    }

    #[test]
    fn iter_visits_all_vertices() {
        let t = make_target(
            "t",
            vec![
                delta(10, 1.0, 0.0, 0.0),
                delta(20, 2.0, 0.0, 0.0),
                delta(30, 3.0, 0.0, 0.0),
            ],
        );
        let map = InfluenceMap::build(&[("t", &t)]);
        let mut vids: Vec<u32> = map.iter().map(|vi| vi.vertex_id).collect();
        vids.sort_unstable();
        assert_eq!(vids, vec![10, 20, 30]);
    }
}
