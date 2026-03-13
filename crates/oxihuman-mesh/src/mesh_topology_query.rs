// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Topology query: genus, Euler number, connected components stub.

use std::collections::{HashMap, HashSet, VecDeque};

/// Topological properties of a mesh.
#[derive(Debug, Clone, Default)]
pub struct TopologyInfo {
    pub vertex_count: usize,
    pub edge_count: usize,
    pub face_count: usize,
    pub euler_characteristic: i32,
    pub genus: i32,
    pub connected_components: usize,
    pub boundary_edges: usize,
    pub is_manifold: bool,
}

/// Build the edge set of a triangle mesh.
pub fn build_edge_set(tris: &[[u32; 3]]) -> HashSet<(u32, u32)> {
    let mut edges = HashSet::new();
    for tri in tris {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let e = if a < b { (a, b) } else { (b, a) };
            edges.insert(e);
        }
    }
    edges
}

/// Count edges that appear in exactly one triangle (boundary edges).
pub fn count_boundary_edges(tris: &[[u32; 3]]) -> usize {
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for tri in tris {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    edge_count.values().filter(|&&c| c == 1).count()
}

/// Compute Euler characteristic: V - E + F.
pub fn euler_characteristic(v: usize, e: usize, f: usize) -> i32 {
    v as i32 - e as i32 + f as i32
}

/// Estimate genus from Euler characteristic (for closed orientable surfaces).
pub fn genus_from_euler(euler: i32, components: usize) -> i32 {
    /* χ = 2*(1 - g) for closed surface; g = 1 - χ/2 */
    (components as i32 - euler / 2).max(0)
}

/// Count connected components of the mesh (vertex adjacency BFS).
pub fn connected_components(verts_count: usize, tris: &[[u32; 3]]) -> usize {
    if verts_count == 0 {
        return 0;
    }
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); verts_count];
    for tri in tris {
        for i in 0..3 {
            let a = tri[i] as usize;
            let b = tri[(i + 1) % 3] as usize;
            if a < verts_count && b < verts_count {
                adj[a].push(b as u32);
                adj[b].push(a as u32);
            }
        }
    }
    let mut visited = vec![false; verts_count];
    let mut components = 0usize;
    for start in 0..verts_count {
        if visited[start] {
            continue;
        }
        components += 1;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited[start] = true;
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                let nb = nb as usize;
                if !visited[nb] {
                    visited[nb] = true;
                    queue.push_back(nb);
                }
            }
        }
    }
    components
}

/// Full topology query for a mesh.
pub fn query_topology(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> TopologyInfo {
    let v = verts.len();
    let f = tris.len();
    let edges = build_edge_set(tris);
    let e = edges.len();
    let euler = euler_characteristic(v, e, f);
    let comps = connected_components(v, tris);
    let g = genus_from_euler(euler, comps);
    let boundary = count_boundary_edges(tris);
    let is_manifold = boundary == 0; /* stub: closed manifold if no boundary edges */

    TopologyInfo {
        vertex_count: v,
        edge_count: e,
        face_count: f,
        euler_characteristic: euler,
        genus: g,
        connected_components: comps,
        boundary_edges: boundary,
        is_manifold,
    }
}

/// Check if a mesh is topologically equivalent to a sphere (genus 0, Euler = 2).
pub fn is_topological_sphere(info: &TopologyInfo) -> bool {
    info.genus == 0 && info.euler_characteristic == 2 && info.is_manifold
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tetrahedron() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let v = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let t = vec![[0u32, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]];
        (v, t)
    }

    #[test]
    fn test_edge_set_triangle_count() {
        let tris = vec![[0u32, 1, 2]];
        let edges = build_edge_set(&tris);
        assert_eq!(edges.len(), 3 /* triangle has 3 edges */);
    }

    #[test]
    fn test_euler_characteristic_tetrahedron() {
        /* Tetrahedron: V=4, E=6, F=4 → χ = 2 */
        let e = euler_characteristic(4, 6, 4);
        assert_eq!(e, 2 /* Euler = 2 for sphere topology */);
    }

    #[test]
    fn test_genus_sphere() {
        let g = genus_from_euler(2, 1);
        assert_eq!(g, 0 /* sphere genus = 0 */);
    }

    #[test]
    fn test_genus_torus() {
        /* Torus: χ = 0 → g = 1 */
        let g = genus_from_euler(0, 1);
        assert_eq!(g, 1 /* torus genus = 1 */);
    }

    #[test]
    fn test_connected_components_single() {
        let (_, tris) = tetrahedron();
        let comps = connected_components(4, &tris);
        assert_eq!(comps, 1 /* tetrahedron is connected */);
    }

    #[test]
    fn test_connected_components_two_separate() {
        /* Two triangles sharing no vertices */
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let comps = connected_components(6, &tris);
        assert_eq!(comps, 2 /* two components */);
    }

    #[test]
    fn test_boundary_edges_open_mesh() {
        /* Single triangle: all 3 edges are boundary edges */
        let tris = vec![[0u32, 1, 2]];
        let b = count_boundary_edges(&tris);
        assert_eq!(b, 3 /* open triangle has 3 boundary edges */);
    }

    #[test]
    fn test_query_topology_tetrahedron() {
        let (verts, tris) = tetrahedron();
        let info = query_topology(&verts, &tris);
        assert_eq!(info.vertex_count, 4 /* 4 vertices */);
        assert_eq!(info.face_count, 4 /* 4 faces */);
        assert_eq!(info.edge_count, 6 /* 6 edges */);
    }

    #[test]
    fn test_query_topology_empty() {
        let info = query_topology(&[], &[]);
        assert_eq!(info.connected_components, 0 /* empty mesh */);
    }
}
