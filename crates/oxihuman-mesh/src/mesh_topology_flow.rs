// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Topology flow analysis: edge loops, edge rings, poles detection.

use std::collections::HashMap;

#[allow(dead_code)]
pub struct HalfEdge {
    pub vertex: u32,
    pub next: u32,
    pub twin: Option<u32>,
    pub face: u32,
}

#[allow(dead_code)]
pub struct TopologyMesh {
    pub half_edges: Vec<HalfEdge>,
    pub vertex_count: usize,
    pub face_count: usize,
}

#[allow(dead_code)]
pub struct TopoEdgeLoop {
    pub vertices: Vec<u32>,
    pub closed: bool,
}

#[allow(dead_code)]
pub struct Pole {
    pub vertex: u32,
    pub valence: u32,
}

#[allow(dead_code)]
pub enum FlowDir {
    Forward,
    Backward,
    Both,
}

/// Build a half-edge topology from positions and triangle indices.
#[allow(dead_code)]
pub fn build_topology(positions: &[[f32; 3]], indices: &[u32]) -> TopologyMesh {
    let vertex_count = positions.len();
    let face_count = indices.len() / 3;

    let mut half_edges: Vec<HalfEdge> = Vec::with_capacity(face_count * 3);

    // Build half-edges
    for face in 0..face_count {
        let base = face * 3;
        for k in 0..3 {
            let he_idx = (base + k) as u32;
            let v = indices[base + k];
            let next = (base + (k + 1) % 3) as u32;
            half_edges.push(HalfEdge {
                vertex: v,
                next,
                twin: None,
                face: face as u32,
            });
            let _ = he_idx;
        }
    }

    // Build twin map: edge (v0, v1) -> half-edge index
    // A half-edge he at index i points from he[prev(i)].vertex to he[i].vertex
    let total_he = half_edges.len();
    let mut edge_map: HashMap<(u32, u32), u32> = HashMap::with_capacity(total_he);

    for face in 0..face_count {
        let base = face * 3;
        for k in 0..3 {
            let he_idx = (base + k) as u32;
            let from = indices[base + (k + 2) % 3];
            let to = indices[base + k];
            edge_map.insert((from, to), he_idx);
        }
    }

    for face in 0..face_count {
        let base = face * 3;
        for k in 0..3 {
            let he_idx = base + k;
            let from = indices[base + (k + 2) % 3];
            let to = indices[base + k];
            if let Some(&twin_idx) = edge_map.get(&(to, from)) {
                half_edges[he_idx].twin = Some(twin_idx);
            }
        }
    }

    TopologyMesh {
        half_edges,
        vertex_count,
        face_count,
    }
}

/// Return the valence (number of adjacent vertices) of a vertex.
#[allow(dead_code)]
pub fn vertex_valence(topo: &TopologyMesh, vertex_id: u32) -> u32 {
    let mut count = 0u32;
    for (i, he) in topo.half_edges.iter().enumerate() {
        if he.vertex == vertex_id {
            // This HE ends at vertex_id; the source is the prev HE's vertex
            let _ = i;
            count += 1;
        }
    }
    count
}

/// Find all poles (vertices with valence != 4, for quad meshes).
#[allow(dead_code)]
pub fn find_poles(topo: &TopologyMesh) -> Vec<Pole> {
    let mut valences = vec![0u32; topo.vertex_count];
    for he in &topo.half_edges {
        if (he.vertex as usize) < topo.vertex_count {
            valences[he.vertex as usize] += 1;
        }
    }
    valences
        .into_iter()
        .enumerate()
        .filter(|&(_, v)| v != 4)
        .map(|(i, v)| Pole {
            vertex: i as u32,
            valence: v,
        })
        .collect()
}

/// Walk an edge loop starting from a given half-edge.
#[allow(dead_code)]
pub fn find_edge_loop(topo: &TopologyMesh, start_he: u32, dir: FlowDir) -> TopoEdgeLoop {
    let max_steps = topo.half_edges.len() + 1;
    let mut vertices = Vec::new();
    let mut current = start_he as usize;

    if current >= topo.half_edges.len() {
        return TopoEdgeLoop {
            vertices,
            closed: false,
        };
    }

    let _ = dir;

    let start_vertex = topo.half_edges[current].vertex;
    vertices.push(start_vertex);

    for _ in 0..max_steps {
        let next = topo.half_edges[current].next as usize;
        if next >= topo.half_edges.len() {
            break;
        }
        current = next;
        let v = topo.half_edges[current].vertex;
        if v == start_vertex && !vertices.is_empty() && vertices.len() > 1 {
            return TopoEdgeLoop {
                vertices,
                closed: true,
            };
        }
        if vertices.contains(&v) {
            break;
        }
        vertices.push(v);
    }

    TopoEdgeLoop {
        vertices,
        closed: false,
    }
}

/// Walk an edge ring starting from a given half-edge (parallel edges).
#[allow(dead_code)]
pub fn find_edge_ring(topo: &TopologyMesh, start_he: u32) -> TopoEdgeLoop {
    let max_steps = topo.half_edges.len() + 1;
    let mut vertices = Vec::new();
    let mut current = start_he as usize;

    if current >= topo.half_edges.len() {
        return TopoEdgeLoop {
            vertices,
            closed: false,
        };
    }

    let start_v = topo.half_edges[current].vertex;
    vertices.push(start_v);

    for _ in 0..max_steps {
        // Jump to twin then advance two steps (for quads)
        let twin = match topo.half_edges[current].twin {
            Some(t) => t as usize,
            None => break,
        };
        if twin >= topo.half_edges.len() {
            break;
        }
        // advance next then next again in the twin face
        let n1 = topo.half_edges[twin].next as usize;
        if n1 >= topo.half_edges.len() {
            break;
        }
        let n2 = topo.half_edges[n1].next as usize;
        if n2 >= topo.half_edges.len() {
            break;
        }
        current = n2;
        let v = topo.half_edges[current].vertex;
        if v == start_v {
            return TopoEdgeLoop {
                vertices,
                closed: true,
            };
        }
        if vertices.contains(&v) {
            break;
        }
        vertices.push(v);
    }

    TopoEdgeLoop {
        vertices,
        closed: false,
    }
}

/// Check if the mesh is manifold (every edge has 1 or 2 half-edges).
#[allow(dead_code)]
pub fn is_manifold(topo: &TopologyMesh) -> bool {
    // Count half-edges per undirected edge
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for he in &topo.half_edges {
        let face_base = (he.face as usize) * 3;
        // We track by (min, max) of the directed edge vertices
        let v_from = {
            // Find the previous HE vertex in this face
            let base = (he.face as usize) * 3;
            let mut prev_v = 0u32;
            for k in 0..3 {
                if (base + k) as u32 == {
                    // find index of this he
                    let mut idx = 0u32;
                    for (i, h) in topo.half_edges.iter().enumerate() {
                        if std::ptr::eq(h, he) {
                            idx = i as u32;
                            break;
                        }
                    }
                    idx
                } {
                    prev_v = topo.half_edges[base + (k + 2) % 3].vertex;
                    break;
                }
            }
            let _ = face_base;
            prev_v
        };
        let v_to = he.vertex;
        let key = if v_from < v_to {
            (v_from, v_to)
        } else {
            (v_to, v_from)
        };
        *edge_count.entry(key).or_insert(0) += 1;
    }

    edge_count.values().all(|&c| c == 1 || c == 2)
}

/// Find boundary vertices (adjacent to boundary edges).
#[allow(dead_code)]
pub fn boundary_vertices(topo: &TopologyMesh) -> Vec<u32> {
    let mut bverts = std::collections::HashSet::new();
    for (i, he) in topo.half_edges.iter().enumerate() {
        if he.twin.is_none() {
            // This is a boundary edge; vertices are he.vertex and prev vertex
            bverts.insert(he.vertex);
            let face_base = (he.face as usize) * 3;
            // find previous he in this face
            for k in 0..3 {
                if face_base + k == i {
                    let prev_v = topo.half_edges[face_base + (k + 2) % 3].vertex;
                    bverts.insert(prev_v);
                }
            }
        }
    }
    let mut result: Vec<u32> = bverts.into_iter().collect();
    result.sort_unstable();
    result
}

/// Return the number of vertices of a given face.
#[allow(dead_code)]
pub fn face_vertex_count(topo: &TopologyMesh, face_id: u32) -> u32 {
    // Each face stores exactly 3 half-edges (triangles)
    let base = (face_id as usize) * 3;
    if base + 2 >= topo.half_edges.len() {
        return 0;
    }
    3
}

/// Return face indices adjacent to a given face.
#[allow(dead_code)]
pub fn adjacent_faces(topo: &TopologyMesh, face_id: u32) -> Vec<u32> {
    let base = (face_id as usize) * 3;
    if base + 2 >= topo.half_edges.len() {
        return vec![];
    }
    let mut adj = Vec::new();
    for k in 0..3 {
        if let Some(twin) = topo.half_edges[base + k].twin {
            if (twin as usize) < topo.half_edges.len() {
                let tf = topo.half_edges[twin as usize].face;
                if tf != face_id {
                    adj.push(tf);
                }
            }
        }
    }
    adj.sort_unstable();
    adj.dedup();
    adj
}

/// Return vertex indices adjacent to a given vertex.
#[allow(dead_code)]
pub fn adjacent_vertices(topo: &TopologyMesh, vertex_id: u32) -> Vec<u32> {
    let mut adj = std::collections::HashSet::new();
    // An HE whose vertex == vertex_id contributes a directed edge from prev to vertex_id
    // We want all vertices directly connected
    for (i, he) in topo.half_edges.iter().enumerate() {
        if he.vertex == vertex_id {
            let face_base = (he.face as usize) * 3;
            // find k
            for k in 0..3 {
                if face_base + k == i {
                    let prev_v = topo.half_edges[face_base + (k + 2) % 3].vertex;
                    adj.insert(prev_v);
                    // next vertex
                    let next_v = topo.half_edges[face_base + (k + 1) % 3].vertex;
                    adj.insert(next_v);
                }
            }
        }
    }
    adj.remove(&vertex_id);
    let mut result: Vec<u32> = adj.into_iter().collect();
    result.sort_unstable();
    result
}

/// Return (vertex_count, edge_count, face_count).
#[allow(dead_code)]
pub fn topology_stats(topo: &TopologyMesh) -> (usize, usize, usize) {
    let mut edge_set: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
    for (i, he) in topo.half_edges.iter().enumerate() {
        let face_base = (he.face as usize) * 3;
        let mut v_from = 0u32;
        for k in 0..3 {
            if face_base + k == i {
                v_from = topo.half_edges[face_base + (k + 2) % 3].vertex;
                break;
            }
        }
        let v_to = he.vertex;
        let key = if v_from < v_to {
            (v_from, v_to)
        } else {
            (v_to, v_from)
        };
        edge_set.insert(key);
    }
    (topo.vertex_count, edge_set.len(), topo.face_count)
}

/// Check if the mesh is closed (no boundary edges).
#[allow(dead_code)]
pub fn is_closed_mesh(topo: &TopologyMesh) -> bool {
    topo.half_edges.iter().all(|he| he.twin.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_mesh() -> TopologyMesh {
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        build_topology(&positions, &indices)
    }

    fn two_triangles() -> TopologyMesh {
        // Two triangles sharing edge (1,2)
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 1, 3, 2];
        build_topology(&positions, &indices)
    }

    #[test]
    fn build_triangle_topology() {
        let topo = triangle_mesh();
        assert_eq!(topo.half_edges.len(), 3);
        assert_eq!(topo.face_count, 1);
        assert_eq!(topo.vertex_count, 3);
    }

    #[test]
    fn triangle_face_vertex_count() {
        let topo = triangle_mesh();
        assert_eq!(face_vertex_count(&topo, 0), 3);
    }

    #[test]
    fn triangle_no_twins_boundary() {
        let topo = triangle_mesh();
        assert!(topo.half_edges.iter().all(|he| he.twin.is_none()));
    }

    #[test]
    fn two_triangles_shared_edge_has_twins() {
        let topo = two_triangles();
        let twin_count = topo
            .half_edges
            .iter()
            .filter(|he| he.twin.is_some())
            .count();
        assert_eq!(twin_count, 2);
    }

    #[test]
    fn vertex_valence_triangle() {
        let topo = triangle_mesh();
        // Each vertex appears in 1 triangle, valence = 1 edge from it
        assert_eq!(vertex_valence(&topo, 0), 1);
    }

    #[test]
    fn find_poles_triangle() {
        let topo = triangle_mesh();
        let poles = find_poles(&topo);
        // All vertices have valence != 4
        assert_eq!(poles.len(), 3);
    }

    #[test]
    fn boundary_vertices_triangle() {
        let topo = triangle_mesh();
        let bv = boundary_vertices(&topo);
        assert_eq!(bv.len(), 3);
    }

    #[test]
    fn boundary_vertices_two_triangles() {
        let topo = two_triangles();
        let bv = boundary_vertices(&topo);
        // All 4 vertices are on boundary
        assert_eq!(bv.len(), 4);
    }

    #[test]
    fn adjacent_faces_two_triangles() {
        let topo = two_triangles();
        let adj0 = adjacent_faces(&topo, 0);
        assert!(adj0.contains(&1));
        let adj1 = adjacent_faces(&topo, 1);
        assert!(adj1.contains(&0));
    }

    #[test]
    fn topology_stats_triangle() {
        let topo = triangle_mesh();
        let (v, e, f) = topology_stats(&topo);
        assert_eq!(v, 3);
        assert_eq!(e, 3);
        assert_eq!(f, 1);
    }

    #[test]
    fn topology_stats_two_triangles() {
        let topo = two_triangles();
        let (v, e, f) = topology_stats(&topo);
        assert_eq!(v, 4);
        assert_eq!(e, 5);
        assert_eq!(f, 2);
    }

    #[test]
    fn is_closed_single_triangle() {
        let topo = triangle_mesh();
        assert!(!is_closed_mesh(&topo));
    }

    #[test]
    fn edge_loop_start_returns_vertices() {
        let topo = triangle_mesh();
        let loop_ = find_edge_loop(&topo, 0, FlowDir::Forward);
        assert!(!loop_.vertices.is_empty());
    }

    #[test]
    fn adjacent_vertices_triangle() {
        let topo = triangle_mesh();
        let adj = adjacent_vertices(&topo, 0);
        // vertex 0 is adjacent to 1 and 2
        assert_eq!(adj.len(), 2);
        assert!(adj.contains(&1) || adj.contains(&2));
    }
}
