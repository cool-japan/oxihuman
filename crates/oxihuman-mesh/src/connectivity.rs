// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Public data types
// ---------------------------------------------------------------------------

/// Statistics about mesh connectivity.
#[derive(Debug, Clone)]
pub struct ConnectivityStats {
    pub vertex_count: usize,
    pub face_count: usize,
    pub edge_count: usize,
    pub boundary_edge_count: usize,
    /// Edges shared by exactly 2 faces.
    pub manifold_edge_count: usize,
    /// Edges shared by 3 or more faces.
    pub non_manifold_edge_count: usize,
    pub connected_component_count: usize,
    /// V - E + F
    pub euler_characteristic: i64,
    /// No boundary edges.
    pub is_closed: bool,
    /// No non-manifold edges.
    pub is_manifold: bool,
}

// ---------------------------------------------------------------------------
// Internal union-find (disjoint set union)
// ---------------------------------------------------------------------------

struct Dsu {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Dsu {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // path halving
            x = self.parent[x];
        }
        x
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

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build a map from canonical edge (min, max) -> face-reference count.
fn build_edge_map(mesh: &MeshBuffers) -> HashMap<(u32, u32), u32> {
    let mut map: HashMap<(u32, u32), u32> = HashMap::new();
    let indices = &mesh.indices;
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        for &(p, q) in &[(a, b), (b, c), (c, a)] {
            let key = (p.min(q), p.max(q));
            *map.entry(key).or_insert(0) += 1;
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute connectivity statistics for a mesh.
#[allow(dead_code)]
pub fn connectivity_stats(mesh: &MeshBuffers) -> ConnectivityStats {
    let vertex_count = mesh.vertex_count();
    let face_count = mesh.face_count();

    let edge_map = build_edge_map(mesh);
    let edge_count = edge_map.len();

    let mut boundary_edge_count = 0usize;
    let mut manifold_edge_count = 0usize;
    let mut non_manifold_edge_count = 0usize;

    for &count in edge_map.values() {
        match count {
            1 => boundary_edge_count += 1,
            2 => manifold_edge_count += 1,
            _ => non_manifold_edge_count += 1,
        }
    }

    let connected_component_count = connected_component_count(mesh);
    let euler_characteristic = vertex_count as i64 - edge_count as i64 + face_count as i64;
    let is_closed = boundary_edge_count == 0;
    let is_manifold = non_manifold_edge_count == 0;

    ConnectivityStats {
        vertex_count,
        face_count,
        edge_count,
        boundary_edge_count,
        manifold_edge_count,
        non_manifold_edge_count,
        connected_component_count,
        euler_characteristic,
        is_closed,
        is_manifold,
    }
}

/// Find all connected components using union-find with path compression.
///
/// Returns a Vec where `component_id[vertex]` is the component index (0-based,
/// densely packed).
#[allow(dead_code)]
pub fn find_connected_components(mesh: &MeshBuffers) -> Vec<usize> {
    let n = mesh.vertex_count();
    if n == 0 {
        return Vec::new();
    }

    let mut dsu = Dsu::new(n);
    let indices = &mesh.indices;
    let face_count = indices.len() / 3;

    for f in 0..face_count {
        let a = indices[f * 3] as usize;
        let b = indices[f * 3 + 1] as usize;
        let c = indices[f * 3 + 2] as usize;
        dsu.union(a, b);
        dsu.union(b, c);
    }

    // Compress root ids to a dense range.
    let mut root_to_id: HashMap<usize, usize> = HashMap::new();
    let mut next_id = 0usize;
    let mut component_id = vec![0usize; n];

    for (v, slot) in component_id.iter_mut().enumerate() {
        let root = dsu.find(v);
        let id = root_to_id.entry(root).or_insert_with(|| {
            let id = next_id;
            next_id += 1;
            id
        });
        *slot = *id;
    }

    component_id
}

/// Count the number of connected components.
#[allow(dead_code)]
pub fn connected_component_count(mesh: &MeshBuffers) -> usize {
    let ids = find_connected_components(mesh);
    if ids.is_empty() {
        return 0;
    }
    let max_id = ids.iter().copied().max().unwrap_or(0);
    max_id + 1
}

/// Find boundary edges (edges belonging to exactly 1 face).
///
/// Returns `Vec<(vertex_a, vertex_b)>` in canonical (min, max) order.
#[allow(dead_code)]
pub fn find_boundary_edges(mesh: &MeshBuffers) -> Vec<(u32, u32)> {
    let edge_map = build_edge_map(mesh);
    edge_map
        .into_iter()
        .filter(|&(_, count)| count == 1)
        .map(|(edge, _)| edge)
        .collect()
}

/// Find boundary loops: sequences of connected boundary edges forming closed loops.
///
/// Returns `Vec<Vec<u32>>` where each inner Vec is a sequence of vertex indices
/// that forms a closed loop (last vertex connects back to first).
#[allow(dead_code)]
pub fn find_boundary_loops(mesh: &MeshBuffers) -> Vec<Vec<u32>> {
    let boundary = find_boundary_edges(mesh);
    if boundary.is_empty() {
        return Vec::new();
    }

    // Build adjacency: vertex -> list of boundary neighbours.
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for (a, b) in &boundary {
        adj.entry(*a).or_default().push(*b);
        adj.entry(*b).or_default().push(*a);
    }

    let mut visited_edges: HashMap<(u32, u32), bool> = HashMap::new();
    for &(a, b) in &boundary {
        visited_edges.insert((a, b), false);
        visited_edges.insert((b, a), false);
    }

    let mut loops: Vec<Vec<u32>> = Vec::new();

    // Walk from each unvisited boundary vertex.
    let mut visited_verts: HashMap<u32, bool> = boundary
        .iter()
        .flat_map(|&(a, b)| [(a, false), (b, false)])
        .collect();

    for start in boundary
        .iter()
        .flat_map(|&(a, b)| [a, b])
        .collect::<Vec<_>>()
    {
        if visited_verts.get(&start).copied().unwrap_or(true) {
            continue;
        }

        let mut chain: Vec<u32> = vec![start];
        let mut current = start;
        *visited_verts.entry(start).or_insert(true) = true;

        while let Some(neighbours) = adj.get(&current).cloned() {
            let mut next_opt = None;
            for &nb in &neighbours {
                let edge = (current, nb);
                if !visited_edges.get(&edge).copied().unwrap_or(true) {
                    next_opt = Some(nb);
                    break;
                }
            }

            match next_opt {
                None => break,
                Some(next) => {
                    visited_edges.insert((current, next), true);
                    visited_edges.insert((next, current), true);
                    if next == start {
                        // Closed loop
                        loops.push(chain.clone());
                        break;
                    }
                    if visited_verts.get(&next).copied().unwrap_or(false) {
                        // Hit an already-visited vertex — partial chain, not a
                        // closed loop in this walk; store as open chain.
                        chain.push(next);
                        loops.push(chain.clone());
                        break;
                    }
                    *visited_verts.entry(next).or_insert(true) = true;
                    chain.push(next);
                    current = next;
                }
            }
        }
    }

    loops
}

/// Compute vertex valence: number of edges incident to each vertex.
///
/// Returns a `Vec<usize>` of length `vertex_count`.
#[allow(dead_code)]
pub fn vertex_valence(mesh: &MeshBuffers) -> Vec<usize> {
    let n = mesh.vertex_count();
    let mut valence = vec![0usize; n];
    let edge_map = build_edge_map(mesh);
    for &(a, b) in edge_map.keys() {
        valence[a as usize] += 1;
        valence[b as usize] += 1;
    }
    valence
}

/// Find non-manifold edges (shared by 3 or more faces).
///
/// Returns `Vec<(vertex_a, vertex_b)>` in canonical (min, max) order.
#[allow(dead_code)]
pub fn find_non_manifold_edges(mesh: &MeshBuffers) -> Vec<(u32, u32)> {
    let edge_map = build_edge_map(mesh);
    edge_map
        .into_iter()
        .filter(|&(_, count)| count >= 3)
        .map(|(edge, _)| edge)
        .collect()
}

/// Split a mesh into its connected components.
///
/// Returns one `MeshBuffers` per component, each with remapped indices.
#[allow(dead_code)]
pub fn split_components(mesh: &MeshBuffers) -> Vec<MeshBuffers> {
    let n = mesh.vertex_count();
    if n == 0 {
        return Vec::new();
    }

    let component_id = find_connected_components(mesh);
    let num_components = if component_id.is_empty() {
        0
    } else {
        component_id.iter().copied().max().unwrap_or(0) + 1
    };

    // Map: component -> (old_vertex -> new_vertex)
    let mut comp_vertex_map: Vec<HashMap<u32, u32>> = vec![HashMap::new(); num_components];
    let mut comp_positions: Vec<Vec<[f32; 3]>> = vec![Vec::new(); num_components];
    let mut comp_normals: Vec<Vec<[f32; 3]>> = vec![Vec::new(); num_components];
    let mut comp_tangents: Vec<Vec<[f32; 4]>> = vec![Vec::new(); num_components];
    let mut comp_uvs: Vec<Vec<[f32; 2]>> = vec![Vec::new(); num_components];
    let mut comp_colors: Vec<Option<Vec<[f32; 4]>>> = vec![None; num_components];

    // Pre-allocate color buffers if source has colors.
    if mesh.colors.is_some() {
        for c in comp_colors.iter_mut() {
            *c = Some(Vec::new());
        }
    }

    // Assign vertices to components.
    for (v, &comp) in component_id.iter().enumerate() {
        let new_idx = comp_positions[comp].len() as u32;
        comp_vertex_map[comp].insert(v as u32, new_idx);
        comp_positions[comp].push(mesh.positions[v]);
        comp_normals[comp].push(mesh.normals[v]);
        comp_tangents[comp].push(mesh.tangents[v]);
        comp_uvs[comp].push(mesh.uvs[v]);
        if let (Some(src_colors), Some(dst_colors)) =
            (mesh.colors.as_ref(), comp_colors[comp].as_mut())
        {
            dst_colors.push(src_colors[v]);
        }
    }

    // Build index buffers per component.
    let mut comp_indices: Vec<Vec<u32>> = vec![Vec::new(); num_components];
    let face_count = mesh.indices.len() / 3;
    for f in 0..face_count {
        let a = mesh.indices[f * 3];
        let b = mesh.indices[f * 3 + 1];
        let c = mesh.indices[f * 3 + 2];
        // All three vertices of a face belong to the same component.
        let comp = component_id[a as usize];
        let map = &comp_vertex_map[comp];
        comp_indices[comp].push(map[&a]);
        comp_indices[comp].push(map[&b]);
        comp_indices[comp].push(map[&c]);
    }

    // Assemble MeshBuffers.
    (0..num_components)
        .map(|comp| MeshBuffers {
            positions: comp_positions[comp].clone(),
            normals: comp_normals[comp].clone(),
            tangents: comp_tangents[comp].clone(),
            uvs: comp_uvs[comp].clone(),
            indices: comp_indices[comp].clone(),
            colors: comp_colors[comp].clone(),
            has_suit: mesh.has_suit,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            has_suit: false,
        })
    }

    /// Single triangle: 3 verts, 1 face, 3 edges (all boundary).
    fn single_triangle() -> MeshBuffers {
        make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    /// Tetrahedron: 4 verts, 4 faces, 6 edges (all manifold, closed).
    fn tetrahedron() -> MeshBuffers {
        make_mesh(
            vec![
                [1.0, 1.0, 1.0],
                [-1.0, -1.0, 1.0],
                [-1.0, 1.0, -1.0],
                [1.0, -1.0, -1.0],
            ],
            vec![0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 3, 2],
        )
    }

    /// Two separate triangles (no shared vertices).
    fn two_separate_triangles() -> MeshBuffers {
        make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 3, 4, 5],
        )
    }

    /// Two triangles sharing an edge -> connected quad (1 component, 1 manifold edge).
    fn connected_quad() -> MeshBuffers {
        make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 1, 3, 2],
        )
    }

    // ------------------------------------------------------------------
    // Tests
    // ------------------------------------------------------------------

    #[test]
    fn single_triangle_stats() {
        let mesh = single_triangle();
        let s = connectivity_stats(&mesh);
        assert_eq!(s.vertex_count, 3);
        assert_eq!(s.face_count, 1);
        assert_eq!(s.edge_count, 3);
        assert_eq!(s.boundary_edge_count, 3);
        assert_eq!(s.manifold_edge_count, 0);
        assert_eq!(s.non_manifold_edge_count, 0);
        assert_eq!(s.connected_component_count, 1);
        assert!(!s.is_closed);
        assert!(s.is_manifold);
    }

    #[test]
    fn two_separate_triangles_two_components() {
        let mesh = two_separate_triangles();
        assert_eq!(connected_component_count(&mesh), 2);
    }

    #[test]
    fn connected_quad_one_component() {
        let mesh = connected_quad();
        assert_eq!(connected_component_count(&mesh), 1);
    }

    #[test]
    fn boundary_edges_of_triangle() {
        let mesh = single_triangle();
        let boundary = find_boundary_edges(&mesh);
        assert_eq!(boundary.len(), 3);
        // All three edges of the triangle must be present.
        let mut sorted = boundary.clone();
        sorted.sort();
        assert!(sorted.contains(&(0, 1)));
        assert!(sorted.contains(&(0, 2)));
        assert!(sorted.contains(&(1, 2)));
    }

    #[test]
    fn closed_mesh_no_boundary() {
        let mesh = tetrahedron();
        let boundary = find_boundary_edges(&mesh);
        assert!(
            boundary.is_empty(),
            "tetrahedron should have no boundary edges"
        );
        let s = connectivity_stats(&mesh);
        assert!(s.is_closed);
    }

    #[test]
    fn vertex_valence_triangle() {
        let mesh = single_triangle();
        let val = vertex_valence(&mesh);
        // In a single triangle every vertex touches 2 edges.
        assert_eq!(val, vec![2, 2, 2]);
    }

    #[test]
    fn non_manifold_edge_detection() {
        // Build a mesh where edge (0,1) is shared by 3 faces.
        let mesh = make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            // Three triangles sharing edge (0,1)
            vec![0, 1, 2, 0, 1, 3, 0, 1, 4],
        );
        let nm = find_non_manifold_edges(&mesh);
        assert!(!nm.is_empty(), "should detect non-manifold edge (0,1)");
        assert!(nm.contains(&(0, 1)));
    }

    #[test]
    fn split_components_count_matches() {
        let mesh = two_separate_triangles();
        let parts = split_components(&mesh);
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn split_components_each_connected() {
        let mesh = two_separate_triangles();
        let parts = split_components(&mesh);
        for part in &parts {
            assert_eq!(
                connected_component_count(part),
                1,
                "each split component should itself be connected"
            );
        }
    }

    #[test]
    fn euler_characteristic_sphere_is_2() {
        // Tetrahedron is topologically a sphere: V - E + F = 4 - 6 + 4 = 2.
        let mesh = tetrahedron();
        let s = connectivity_stats(&mesh);
        assert_eq!(
            s.euler_characteristic, 2,
            "tetrahedron Euler characteristic should be 2, got {}",
            s.euler_characteristic
        );
    }

    #[test]
    fn boundary_loop_forms_closed_chain() {
        // A single triangle has one boundary loop of 3 vertices.
        let mesh = single_triangle();
        let loops = find_boundary_loops(&mesh);
        assert_eq!(loops.len(), 1, "expected exactly one boundary loop");
        assert_eq!(loops[0].len(), 3, "boundary loop should have 3 vertices");
    }

    #[test]
    fn connected_component_id_consistent() {
        let mesh = two_separate_triangles();
        let ids = find_connected_components(&mesh);
        // Vertices 0..=2 share a component, 3..=5 share another.
        assert_eq!(ids[0], ids[1]);
        assert_eq!(ids[1], ids[2]);
        assert_eq!(ids[3], ids[4]);
        assert_eq!(ids[4], ids[5]);
        assert_ne!(ids[0], ids[3]);
    }

    #[test]
    fn is_manifold_true_for_clean_mesh() {
        let mesh = tetrahedron();
        let s = connectivity_stats(&mesh);
        assert!(s.is_manifold, "tetrahedron should be manifold");
        assert_eq!(s.non_manifold_edge_count, 0);
    }
}
