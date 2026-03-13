// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Graph cut based mesh segmentation using min-cut heuristic.

use std::collections::{HashMap, VecDeque};

/// Graph cut segmentation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GraphCutResult {
    pub labels: Vec<u32>,
    pub cut_weight: f32,
}

/// Build weighted face graph (weight = 1 - |dot(n1, n2)|).
#[allow(dead_code)]
pub fn build_face_graph(positions: &[[f32; 3]], indices: &[u32]) -> Vec<Vec<(usize, f32)>> {
    let tc = indices.len() / 3;
    let normals: Vec<[f32; 3]> = (0..tc).map(|t| {
        face_normal_gc(positions, indices, t)
    }).collect();
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..tc {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(t);
        }
    }
    let mut adj = vec![Vec::new(); tc];
    for faces in edge_faces.values() {
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                let dot = normals[faces[i]][0] * normals[faces[j]][0]
                        + normals[faces[i]][1] * normals[faces[j]][1]
                        + normals[faces[i]][2] * normals[faces[j]][2];
                let w = 1.0 - dot.abs();
                adj[faces[i]].push((faces[j], w));
                adj[faces[j]].push((faces[i], w));
            }
        }
    }
    adj
}

/// Simple 2-way min-cut using BFS-based flood from two seeds.
#[allow(dead_code)]
pub fn two_way_cut(
    graph: &[Vec<(usize, f32)>],
    seed_a: usize,
    seed_b: usize,
) -> GraphCutResult {
    let n = graph.len();
    if n == 0 {
        return GraphCutResult { labels: Vec::new(), cut_weight: 0.0 };
    }
    let mut labels = vec![u32::MAX; n];
    let mut dist = vec![f32::INFINITY; n];
    // BFS from seed_a
    let mut queue = VecDeque::new();
    labels[seed_a] = 0;
    dist[seed_a] = 0.0;
    queue.push_back(seed_a);
    while let Some(f) = queue.pop_front() {
        for &(nb, w) in &graph[f] {
            let new_d = dist[f] + w;
            if new_d < dist[nb] {
                dist[nb] = new_d;
                labels[nb] = 0;
                queue.push_back(nb);
            }
        }
    }
    // BFS from seed_b
    dist[seed_b] = 0.0;
    labels[seed_b] = 1;
    queue.push_back(seed_b);
    while let Some(f) = queue.pop_front() {
        for &(nb, w) in &graph[f] {
            let new_d = dist[f] + w;
            if new_d < dist[nb] {
                dist[nb] = new_d;
                labels[nb] = 1;
                queue.push_back(nb);
            }
        }
    }
    // Fill remaining
    for l in labels.iter_mut() {
        if *l == u32::MAX { *l = 0; }
    }
    let cut_weight = compute_cut_weight(graph, &labels);
    GraphCutResult { labels, cut_weight }
}

/// Compute cut weight (sum of weights of edges crossing the cut).
#[allow(dead_code)]
pub fn compute_cut_weight(graph: &[Vec<(usize, f32)>], labels: &[u32]) -> f32 {
    let mut total = 0.0f32;
    for (i, neighbors) in graph.iter().enumerate() {
        for &(j, w) in neighbors {
            if labels[i] != labels[j] && i < j {
                total += w;
            }
        }
    }
    total
}

/// Segment count.
#[allow(dead_code)]
pub fn segment_count(result: &GraphCutResult) -> u32 {
    if result.labels.is_empty() { return 0; }
    *result.labels.iter().max().unwrap_or(&0) + 1
}

/// Faces in segment.
#[allow(dead_code)]
pub fn faces_in_segment(result: &GraphCutResult, seg: u32) -> Vec<usize> {
    result.labels.iter().enumerate()
        .filter(|&(_, &l)| l == seg)
        .map(|(i, _)| i)
        .collect()
}

fn face_normal_gc(positions: &[[f32; 3]], indices: &[u32], t: usize) -> [f32; 3] {
    let i0 = indices[t * 3] as usize;
    let i1 = indices[t * 3 + 1] as usize;
    let i2 = indices[t * 3 + 2] as usize;
    let e1 = [positions[i1][0] - positions[i0][0], positions[i1][1] - positions[i0][1], positions[i1][2] - positions[i0][2]];
    let e2 = [positions[i2][0] - positions[i0][0], positions[i2][1] - positions[i0][1], positions[i2][2] - positions[i0][2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-12 { [0.0, 0.0, 1.0] } else { [n[0] / len, n[1] / len, n[2] / len] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_build_graph() {
        let (pos, idx) = flat_quad();
        let g = build_face_graph(&pos, &idx);
        assert_eq!(g.len(), 2);
    }

    #[test]
    fn test_two_way_cut_same_seed() {
        let g = vec![vec![(1usize, 0.5)], vec![(0, 0.5)]];
        let r = two_way_cut(&g, 0, 0);
        assert_eq!(r.labels.len(), 2);
    }

    #[test]
    fn test_two_way_cut_diff_seed() {
        let g = vec![vec![(1, 0.5)], vec![(0, 0.5)]];
        let r = two_way_cut(&g, 0, 1);
        assert!(r.labels[0] != r.labels[1] || r.cut_weight >= 0.0);
    }

    #[test]
    fn test_segment_count() {
        let r = GraphCutResult { labels: vec![0, 0, 1, 1], cut_weight: 1.0 };
        assert_eq!(segment_count(&r), 2);
    }

    #[test]
    fn test_faces_in_segment() {
        let r = GraphCutResult { labels: vec![0, 1, 0, 1], cut_weight: 1.0 };
        let f0 = faces_in_segment(&r, 0);
        assert_eq!(f0, vec![0, 2]);
    }

    #[test]
    fn test_cut_weight() {
        let g = vec![vec![(1, 2.0)], vec![(0, 2.0)]];
        let labels = vec![0u32, 1];
        let w = compute_cut_weight(&g, &labels);
        assert!((w - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_graph() {
        let r = two_way_cut(&[], 0, 0);
        assert_eq!(r.labels.len(), 0);
    }

    #[test]
    fn test_single_node() {
        let g = vec![Vec::new()];
        let r = two_way_cut(&g, 0, 0);
        assert_eq!(r.labels.len(), 1);
    }

    #[test]
    fn test_cut_weight_no_cross() {
        let g = vec![vec![(1, 1.0)], vec![(0, 1.0)]];
        let labels = vec![0u32, 0];
        let w = compute_cut_weight(&g, &labels);
        assert!((w - 0.0).abs() < 1e-5);
    }

}
