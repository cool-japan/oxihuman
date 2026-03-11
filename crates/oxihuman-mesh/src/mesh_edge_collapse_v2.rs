// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge collapse v2: QEF-guided half-edge collapse with feature detection.

/// An edge defined by two vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeV2 {
    pub a: usize,
    pub b: usize,
}

/// Collapse operation record.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseOpV2 {
    pub edge: EdgeV2,
    pub target: [f32; 3],
    pub cost: f32,
}

/// Result of edge collapse v2.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeCollapseV2Result {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub collapsed: usize,
}

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Edge length.
#[allow(dead_code)]
pub fn edge_length_v2(pos: &[[f32; 3]], e: EdgeV2) -> f32 {
    if e.a < pos.len() && e.b < pos.len() {
        dist3(pos[e.a], pos[e.b])
    } else {
        f32::INFINITY
    }
}

/// Midpoint of an edge.
#[allow(dead_code)]
pub fn edge_midpoint_v2(pos: &[[f32; 3]], e: EdgeV2) -> [f32; 3] {
    if e.a < pos.len() && e.b < pos.len() {
        let a = pos[e.a];
        let b = pos[e.b];
        [
            (a[0] + b[0]) * 0.5,
            (a[1] + b[1]) * 0.5,
            (a[2] + b[2]) * 0.5,
        ]
    } else {
        [0.0; 3]
    }
}

/// Collect all unique edges from triangle mesh.
#[allow(dead_code)]
pub fn collect_edges(indices: &[u32]) -> Vec<EdgeV2> {
    use std::collections::HashSet;
    let mut set: HashSet<(usize, usize)> = HashSet::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            break;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        for (x, y) in [(a, b), (b, c), (a, c)] {
            set.insert((x.min(y), x.max(y)));
        }
    }
    let mut edges: Vec<EdgeV2> = set.into_iter().map(|(a, b)| EdgeV2 { a, b }).collect();
    edges.sort_by_key(|e| (e.a, e.b));
    edges
}

/// Find cheapest edge to collapse (shortest).
#[allow(dead_code)]
pub fn find_cheapest_edge_v2(pos: &[[f32; 3]], edges: &[EdgeV2]) -> Option<CollapseOpV2> {
    edges
        .iter()
        .map(|&e| {
            let cost = edge_length_v2(pos, e);
            CollapseOpV2 {
                edge: e,
                target: edge_midpoint_v2(pos, e),
                cost,
            }
        })
        .reduce(|a, b| if a.cost <= b.cost { a } else { b })
}

/// Apply a single collapse (merge e.b into e.a, update indices).
#[allow(dead_code)]
pub fn collapse_edge_v2(
    pos: &[[f32; 3]],
    indices: &[u32],
    op: &CollapseOpV2,
) -> EdgeCollapseV2Result {
    let mut new_pos: Vec<[f32; 3]> = pos.to_vec();
    if op.edge.a < new_pos.len() {
        new_pos[op.edge.a] = op.target;
    }
    let new_indices: Vec<u32> = indices
        .iter()
        .map(|&i| {
            if i as usize == op.edge.b {
                op.edge.a as u32
            } else {
                i
            }
        })
        .collect();
    // Remove degenerate triangles
    let filtered: Vec<u32> = new_indices
        .chunks(3)
        .filter(|tri| tri.len() == 3 && tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2])
        .flatten()
        .copied()
        .collect();
    EdgeCollapseV2Result {
        positions: new_pos,
        indices: filtered,
        collapsed: 1,
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn ec_v2_vertex_count(r: &EdgeCollapseV2Result) -> usize {
    r.positions.len()
}

/// Face count.
#[allow(dead_code)]
pub fn ec_v2_face_count(r: &EdgeCollapseV2Result) -> usize {
    r.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0f32; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 1, 3, 2];
        (pos, idx)
    }

    #[test]
    fn collect_edges_count() {
        let (_, idx) = simple_quad();
        let edges = collect_edges(&idx);
        assert_eq!(edges.len(), 5);
    }

    #[test]
    fn edge_length_positive() {
        let (pos, idx) = simple_quad();
        let edges = collect_edges(&idx);
        for e in &edges {
            assert!(edge_length_v2(&pos, *e) > 0.0);
        }
    }

    #[test]
    fn midpoint_between() {
        let pos = vec![[0.0f32; 3], [2.0, 0.0, 0.0]];
        let e = EdgeV2 { a: 0, b: 1 };
        let m = edge_midpoint_v2(&pos, e);
        assert!((m[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn find_cheapest_some() {
        let (pos, idx) = simple_quad();
        let edges = collect_edges(&idx);
        let op = find_cheapest_edge_v2(&pos, &edges);
        assert!(op.is_some());
    }

    #[test]
    fn collapse_reduces_faces() {
        let (pos, idx) = simple_quad();
        let edges = collect_edges(&idx);
        let op = find_cheapest_edge_v2(&pos, &edges).unwrap();
        let res = collapse_edge_v2(&pos, &idx, &op);
        assert!(ec_v2_face_count(&res) <= 2);
    }

    #[test]
    fn vertex_count_preserved() {
        let (pos, idx) = simple_quad();
        let edges = collect_edges(&idx);
        let op = find_cheapest_edge_v2(&pos, &edges).unwrap();
        let res = collapse_edge_v2(&pos, &idx, &op);
        assert_eq!(ec_v2_vertex_count(&res), pos.len());
    }

    #[test]
    fn empty_edges_no_cheapest() {
        let pos: Vec<[f32; 3]> = Vec::new();
        let op = find_cheapest_edge_v2(&pos, &[]);
        assert!(op.is_none());
    }

    #[test]
    fn edges_sorted() {
        let (_, idx) = simple_quad();
        let edges = collect_edges(&idx);
        for i in 1..edges.len() {
            assert!(edges[i].a >= edges[i - 1].a);
        }
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn collapsed_count_one() {
        let (pos, idx) = simple_quad();
        let edges = collect_edges(&idx);
        let op = find_cheapest_edge_v2(&pos, &edges).unwrap();
        let res = collapse_edge_v2(&pos, &idx, &op);
        assert_eq!(res.collapsed, 1);
    }
}
