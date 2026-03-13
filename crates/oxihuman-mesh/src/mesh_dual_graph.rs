// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Dual graph of a triangle mesh.
//!
//! In the dual graph every triangle face becomes a node and every shared edge
//! between two triangles becomes an undirected arc.  Boundary edges have no
//! arc (they border only one face).

#![allow(dead_code)]

/// Configuration for the dual graph builder.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DualGraphConfig {
    /// If `true`, store the arc weight as the Euclidean distance between the
    /// centroids of the two adjacent faces.  Otherwise weight = 1.
    pub weighted_arcs: bool,
}

/// Returns a sensible default [`DualGraphConfig`].
#[allow(dead_code)]
pub fn default_dual_graph_config() -> DualGraphConfig {
    DualGraphConfig { weighted_arcs: true }
}

/// A node in the dual graph (represents one mesh triangle).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DualNode {
    /// Triangle index in the source mesh.
    pub face: usize,
    /// Centroid of the triangle.
    pub centroid: [f32; 3],
    /// Arc indices incident to this node.
    pub arcs: Vec<usize>,
}

/// An undirected arc in the dual graph (represents a shared mesh edge).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DualArc {
    /// First node (lower face index).
    pub node_a: usize,
    /// Second node (higher face index).
    pub node_b: usize,
    /// Arc weight (centroid-to-centroid distance or 1.0).
    pub weight: f32,
}

/// The dual graph of a triangle mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DualGraph {
    /// All dual nodes (one per triangle face).
    pub nodes: Vec<DualNode>,
    /// All dual arcs (one per shared interior edge).
    pub arcs: Vec<DualArc>,
    /// Configuration.
    pub config: DualGraphConfig,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

fn tri_centroid(verts: &[[f32; 3]], indices: &[u32], tri: usize) -> [f32; 3] {
    let b = tri * 3;
    let v0 = verts[indices[b] as usize];
    let v1 = verts[indices[b+1] as usize];
    let v2 = verts[indices[b+2] as usize];
    [
        (v0[0]+v1[0]+v2[0])/3.0,
        (v0[1]+v1[1]+v2[1])/3.0,
        (v0[2]+v1[2]+v2[2])/3.0,
    ]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0]-a[0]; let dy = b[1]-a[1]; let dz = b[2]-a[2];
    (dx*dx + dy*dy + dz*dz).sqrt()
}

/// Build the dual graph from a triangle mesh.
#[allow(dead_code)]
pub fn build_dual_graph(
    verts: &[[f32; 3]],
    indices: &[u32],
    config: DualGraphConfig,
) -> DualGraph {
    let face_count = indices.len() / 3;
    let mut nodes: Vec<DualNode> = (0..face_count)
        .map(|f| DualNode {
            face: f,
            centroid: tri_centroid(verts, indices, f),
            arcs: vec![],
        })
        .collect();

    // Map each directed edge (v_lo, v_hi) to the first face that owns it.
    use std::collections::HashMap;
    let mut edge_map: HashMap<(usize, usize), usize> = HashMap::new();
    let mut arcs: Vec<DualArc> = Vec::new();

    for f in 0..face_count {
        let b = f * 3;
        for k in 0..3 {
            let a = indices[b + k] as usize;
            let c = indices[b + (k+1) % 3] as usize;
            let key = if a < c { (a, c) } else { (c, a) };
            if let Some(&other) = edge_map.get(&key) {
                // Shared edge between faces `other` and `f`.
                let w = if config.weighted_arcs {
                    dist3(nodes[other].centroid, nodes[f].centroid)
                } else {
                    1.0
                };
                let arc_idx = arcs.len();
                let (na, nb) = if other < f { (other, f) } else { (f, other) };
                arcs.push(DualArc { node_a: na, node_b: nb, weight: w });
                nodes[na].arcs.push(arc_idx);
                nodes[nb].arcs.push(arc_idx);
            } else {
                edge_map.insert(key, f);
            }
        }
    }

    DualGraph { nodes, arcs, config }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Number of nodes in the dual graph (= number of faces in the mesh).
#[allow(dead_code)]
pub fn dual_node_count(g: &DualGraph) -> usize {
    g.nodes.len()
}

/// Number of arcs in the dual graph (= number of shared interior edges).
#[allow(dead_code)]
pub fn dual_arc_count(g: &DualGraph) -> usize {
    g.arcs.len()
}

/// Indices of neighbouring nodes of node `n`.
#[allow(dead_code)]
pub fn dual_neighbors(g: &DualGraph, n: usize) -> Vec<usize> {
    g.nodes[n].arcs.iter().map(|&ai| {
        let arc = &g.arcs[ai];
        if arc.node_a == n { arc.node_b } else { arc.node_a }
    }).collect()
}

/// Centroid of the triangle corresponding to node `n`.
#[allow(dead_code)]
pub fn dual_node_centroid(g: &DualGraph, n: usize) -> [f32; 3] {
    g.nodes[n].centroid
}

/// Serialize the dual graph to a simple JSON string.
#[allow(dead_code)]
pub fn dual_graph_to_json(g: &DualGraph) -> String {
    format!(
        "{{\"node_count\":{},\"arc_count\":{}}}",
        g.nodes.len(),
        g.arcs.len()
    )
}

/// Returns `true` if the dual graph is connected (BFS from node 0).
#[allow(dead_code)]
pub fn dual_graph_is_connected(g: &DualGraph) -> bool {
    if g.nodes.is_empty() { return true; }
    let n = g.nodes.len();
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(0);
    visited[0] = true;
    while let Some(cur) = queue.pop_front() {
        for nb in dual_neighbors(g, cur) {
            if !visited[nb] {
                visited[nb] = true;
                queue.push_back(nb);
            }
        }
    }
    visited.iter().all(|&v| v)
}

/// Find a path of node indices from `start` to `goal` (BFS, unweighted).
/// Returns `None` if no path exists.
#[allow(dead_code)]
pub fn dual_path_between(g: &DualGraph, start: usize, goal: usize) -> Option<Vec<usize>> {
    if start >= g.nodes.len() || goal >= g.nodes.len() { return None; }
    if start == goal { return Some(vec![start]); }
    let n = g.nodes.len();
    let mut prev = vec![usize::MAX; n];
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    visited[start] = true;
    while let Some(cur) = queue.pop_front() {
        for nb in dual_neighbors(g, cur) {
            if !visited[nb] {
                visited[nb] = true;
                prev[nb] = cur;
                if nb == goal {
                    // Reconstruct path.
                    let mut path = vec![goal];
                    let mut c = goal;
                    while c != start {
                        c = prev[c];
                        path.push(c);
                    }
                    path.reverse();
                    return Some(path);
                }
                queue.push_back(nb);
            }
        }
    }
    None
}

/// Clear all arcs and neighbour lists (leaves nodes in place).
#[allow(dead_code)]
pub fn dual_graph_clear(g: &mut DualGraph) {
    g.arcs.clear();
    for node in &mut g.nodes {
        node.arcs.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        (verts, indices)
    }

    fn single_tri_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let verts = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let indices = vec![0u32,1,2];
        (verts, indices)
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&[], &[], cfg);
        assert_eq!(dual_node_count(&g), 0);
        assert_eq!(dual_arc_count(&g), 0);
        assert!(dual_graph_is_connected(&g));
    }

    #[test]
    fn test_single_tri_no_arcs() {
        let (verts, indices) = single_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        assert_eq!(dual_node_count(&g), 1);
        assert_eq!(dual_arc_count(&g), 0);
        assert!(dual_graph_is_connected(&g));
    }

    #[test]
    fn test_two_tris_one_arc() {
        let (verts, indices) = two_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        assert_eq!(dual_node_count(&g), 2);
        assert_eq!(dual_arc_count(&g), 1);
    }

    #[test]
    fn test_neighbors_two_tris() {
        let (verts, indices) = two_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        let n0 = dual_neighbors(&g, 0);
        let n1 = dual_neighbors(&g, 1);
        assert_eq!(n0, vec![1]);
        assert_eq!(n1, vec![0]);
    }

    #[test]
    fn test_centroid_single_tri() {
        let (verts, indices) = single_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        let c = dual_node_centroid(&g, 0);
        assert!((c[0] - 1.0/3.0).abs() < 1e-5);
        assert!((c[1] - 1.0/3.0).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let (verts, indices) = two_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        let json = dual_graph_to_json(&g);
        assert!(json.contains("node_count"));
        assert!(json.contains("arc_count"));
    }

    #[test]
    fn test_connected_two_tris() {
        let (verts, indices) = two_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        assert!(dual_graph_is_connected(&g));
    }

    #[test]
    fn test_path_between() {
        let (verts, indices) = two_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        let path = dual_path_between(&g, 0, 1);
        assert!(path.is_some());
        let p = path.expect("should succeed");
        assert_eq!(p.first(), Some(&0));
        assert_eq!(p.last(), Some(&1));
    }

    #[test]
    fn test_path_to_self() {
        let (verts, indices) = two_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        let path = dual_path_between(&g, 0, 0);
        assert_eq!(path, Some(vec![0]));
    }

    #[test]
    fn test_clear_removes_arcs() {
        let (verts, indices) = two_tri_mesh();
        let cfg = default_dual_graph_config();
        let mut g = build_dual_graph(&verts, &indices, cfg);
        dual_graph_clear(&mut g);
        assert_eq!(dual_arc_count(&g), 0);
        assert!(dual_neighbors(&g, 0).is_empty());
    }

    #[test]
    fn test_arc_weight_positive() {
        let (verts, indices) = two_tri_mesh();
        let cfg = default_dual_graph_config();
        let g = build_dual_graph(&verts, &indices, cfg);
        assert!(g.arcs[0].weight > 0.0);
    }

    #[test]
    fn test_unweighted_arc() {
        let (verts, indices) = two_tri_mesh();
        let cfg = DualGraphConfig { weighted_arcs: false };
        let g = build_dual_graph(&verts, &indices, cfg);
        assert_eq!(g.arcs[0].weight, 1.0);
    }
}
