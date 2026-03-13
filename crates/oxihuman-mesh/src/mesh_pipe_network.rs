// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pipe network mesh: multiple tube segments joined at junction nodes.

use std::f32::consts::PI;

/// A pipe junction node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PipeNode {
    pub position: [f32; 3],
    pub id: u32,
}

/// A pipe edge connecting two nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PipeEdge {
    pub from: u32,
    pub to: u32,
    pub radius: f32,
}

/// The resulting mesh of a pipe network.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PipeNetworkMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub pipe_count: usize,
}

/// Generate a tube mesh for a single pipe segment.
#[allow(dead_code)]
pub fn tube_segment(
    from: [f32; 3],
    to: [f32; 3],
    radius: f32,
    segments: usize,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let n = segments.max(3);
    let fwd = normalize3([to[0] - from[0], to[1] - from[1], to[2] - from[2]]);
    let (right, up) = frame_from_forward(fwd);

    let mut pos = Vec::with_capacity(2 * n);
    for &base in &[from, to] {
        for i in 0..n {
            let angle = 2.0 * PI * i as f32 / n as f32;
            let (s, c) = angle.sin_cos();
            pos.push([
                base[0] + (right[0] * c + up[0] * s) * radius,
                base[1] + (right[1] * c + up[1] * s) * radius,
                base[2] + (right[2] * c + up[2] * s) * radius,
            ]);
        }
    }

    let mut idx = Vec::new();
    for i in 0..n {
        let a = i as u32;
        let b = ((i + 1) % n) as u32;
        let c = (n + i) as u32;
        let d = (n + (i + 1) % n) as u32;
        idx.extend_from_slice(&[a, b, c, b, d, c]);
    }
    (pos, idx)
}

/// Build a pipe network mesh from nodes and edges.
#[allow(dead_code)]
pub fn build_pipe_network(
    nodes: &[PipeNode],
    edges: &[PipeEdge],
    segments: usize,
) -> PipeNetworkMesh {
    let mut all_pos: Vec<[f32; 3]> = Vec::new();
    let mut all_idx: Vec<u32> = Vec::new();

    for edge in edges {
        let from_node = nodes.iter().find(|n| n.id == edge.from);
        let to_node = nodes.iter().find(|n| n.id == edge.to);
        if let (Some(f), Some(t)) = (from_node, to_node) {
            let offset = all_pos.len() as u32;
            let (pos, idx) = tube_segment(f.position, t.position, edge.radius, segments);
            all_pos.extend(pos);
            all_idx.extend(idx.iter().map(|&i| i + offset));
        }
    }

    let pipe_count = edges.len();
    PipeNetworkMesh {
        positions: all_pos,
        indices: all_idx,
        pipe_count,
    }
}

/// Total triangle count.
#[allow(dead_code)]
pub fn pipe_triangle_count(m: &PipeNetworkMesh) -> usize {
    m.indices.len() / 3
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn frame_from_forward(fwd: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up_hint = if fwd[1].abs() < 0.9 {
        [0.0_f32, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let right = normalize3(cross3(fwd, up_hint));
    let up = normalize3(cross3(right, fwd));
    (right, up)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_nodes() -> Vec<PipeNode> {
        vec![
            PipeNode {
                position: [0.0, 0.0, 0.0],
                id: 0,
            },
            PipeNode {
                position: [0.0, 0.0, 2.0],
                id: 1,
            },
        ]
    }

    fn one_edge() -> Vec<PipeEdge> {
        vec![PipeEdge {
            from: 0,
            to: 1,
            radius: 0.1,
        }]
    }

    #[test]
    fn single_pipe_vertex_count() {
        let (pos, _) = tube_segment([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.1, 6);
        assert_eq!(pos.len(), 12);
    }

    #[test]
    fn single_pipe_triangle_count() {
        let (_, idx) = tube_segment([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.1, 6);
        assert_eq!(idx.len() / 3, 12);
    }

    #[test]
    fn network_one_pipe() {
        let m = build_pipe_network(&two_nodes(), &one_edge(), 6);
        assert_eq!(m.pipe_count, 1);
    }

    #[test]
    fn network_vertex_count() {
        let m = build_pipe_network(&two_nodes(), &one_edge(), 6);
        assert_eq!(m.positions.len(), 12);
    }

    #[test]
    fn network_indices_multiple_of_three() {
        let m = build_pipe_network(&two_nodes(), &one_edge(), 6);
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn missing_node_skipped() {
        let nodes = two_nodes();
        let edges = vec![PipeEdge {
            from: 0,
            to: 99,
            radius: 0.1,
        }];
        let m = build_pipe_network(&nodes, &edges, 6);
        assert!(m.positions.is_empty());
    }

    #[test]
    fn pipe_triangle_count_helper() {
        let m = build_pipe_network(&two_nodes(), &one_edge(), 6);
        assert_eq!(pipe_triangle_count(&m), m.indices.len() / 3);
    }

    #[test]
    fn min_3_segments_enforced() {
        let (pos, _) = tube_segment([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.1, 0);
        assert_eq!(pos.len(), 6);
    }

    #[test]
    fn two_edges_double_vertices() {
        let nodes = vec![
            PipeNode {
                position: [0.0, 0.0, 0.0],
                id: 0,
            },
            PipeNode {
                position: [1.0, 0.0, 0.0],
                id: 1,
            },
            PipeNode {
                position: [2.0, 0.0, 0.0],
                id: 2,
            },
        ];
        let edges = vec![
            PipeEdge {
                from: 0,
                to: 1,
                radius: 0.1,
            },
            PipeEdge {
                from: 1,
                to: 2,
                radius: 0.1,
            },
        ];
        let m = build_pipe_network(&nodes, &edges, 6);
        assert_eq!(m.positions.len(), 24);
    }

    #[test]
    fn empty_network() {
        let m = build_pipe_network(&[], &[], 6);
        assert!(m.positions.is_empty());
        assert_eq!(m.pipe_count, 0);
    }
}
