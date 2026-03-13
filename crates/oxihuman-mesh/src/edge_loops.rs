// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Internal 3-vector math helpers
// ---------------------------------------------------------------------------

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = length3(v);
    if len > 1e-12 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ---------------------------------------------------------------------------
// Edge type
// ---------------------------------------------------------------------------

/// An edge in the mesh: unordered pair of vertex indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub a: u32,
    pub b: u32,
}

impl Edge {
    /// Create a new edge, normalising so that `a < b`.
    pub fn new(a: u32, b: u32) -> Self {
        if a <= b {
            Edge { a, b }
        } else {
            Edge { a: b, b: a }
        }
    }

    /// Returns `true` if `v` is one of the two endpoints.
    pub fn contains(&self, v: u32) -> bool {
        self.a == v || self.b == v
    }

    /// Returns the *other* endpoint, or `None` if `v` is not part of this edge.
    pub fn other(&self, v: u32) -> Option<u32> {
        if self.a == v {
            Some(self.b)
        } else if self.b == v {
            Some(self.a)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// extract_edges
// ---------------------------------------------------------------------------

/// Extract all unique edges from a mesh (each edge appears exactly once).
#[allow(dead_code)]
pub fn extract_edges(mesh: &MeshBuffers) -> Vec<Edge> {
    let mut seen: std::collections::HashSet<Edge> = std::collections::HashSet::new();
    for tri in mesh.indices.chunks_exact(3) {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        seen.insert(Edge::new(v0, v1));
        seen.insert(Edge::new(v1, v2));
        seen.insert(Edge::new(v2, v0));
    }
    seen.into_iter().collect()
}

// ---------------------------------------------------------------------------
// edge_adjacency
// ---------------------------------------------------------------------------

/// Build an edge adjacency map: for each vertex, the list of directly-connected vertices.
/// The returned `Vec` is indexed by vertex index; inner vecs are *not* sorted.
#[allow(dead_code)]
pub fn edge_adjacency(mesh: &MeshBuffers) -> Vec<Vec<u32>> {
    let n = mesh.positions.len();
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for tri in mesh.indices.chunks_exact(3) {
        let pairs = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for (a, b) in pairs {
            let a = a as usize;
            let b = b as usize;
            if !adj[a].contains(&(b as u32)) {
                adj[a].push(b as u32);
            }
            if !adj[b].contains(&(a as u32)) {
                adj[b].push(a as u32);
            }
        }
    }
    adj
}

// ---------------------------------------------------------------------------
// extract_edge_loop
// ---------------------------------------------------------------------------

/// Build a map from each edge (as `Edge`) to the list of face indices that contain it.
fn build_edge_to_faces(mesh: &MeshBuffers) -> HashMap<Edge, Vec<usize>> {
    let mut map: HashMap<Edge, Vec<usize>> = HashMap::new();
    for (fi, tri) in mesh.indices.chunks_exact(3).enumerate() {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        for e in [Edge::new(v0, v1), Edge::new(v1, v2), Edge::new(v2, v0)] {
            map.entry(e).or_default().push(fi);
        }
    }
    map
}

/// Extract an edge loop starting from `start_a → start_b`.
///
/// At each step the algorithm looks at faces adjacent to the current vertex `b`,
/// then picks the edge of that face whose direction is most nearly parallel
/// (dot product closest to 1.0) to the current `a → b` direction.
///
/// Returns the sequence of vertex indices.  
/// The loop is closed when the first and last vertex are the same.
#[allow(dead_code)]
pub fn extract_edge_loop(
    mesh: &MeshBuffers,
    start_a: u32,
    start_b: u32,
    max_steps: usize,
) -> Vec<u32> {
    let edge_to_faces = build_edge_to_faces(mesh);
    let mut path: Vec<u32> = vec![start_a, start_b];

    let mut prev = start_a;
    let mut cur = start_b;

    for _ in 0..max_steps {
        let dir = normalize3(sub3(
            mesh.positions[cur as usize],
            mesh.positions[prev as usize],
        ));

        // Collect candidate next vertices from all faces that touch `cur`.
        // A candidate vertex `next` must not equal `prev` (no back-tracking).
        let mut best_v: Option<u32> = None;
        let mut best_dot = -2.0f32;

        // Iterate over all faces adjacent to `cur`
        // by scanning faces that share an edge connected to `cur`.
        for tri in mesh.indices.chunks_exact(3) {
            let verts = [tri[0], tri[1], tri[2]];
            // Does this face contain `cur`?
            if !verts.contains(&cur) {
                continue;
            }
            // Try each vertex in the face except `cur` and `prev`.
            for &v in &verts {
                if v == cur || v == prev {
                    continue;
                }
                // Check this edge actually exists in the mesh
                let edge = Edge::new(cur, v);
                if !edge_to_faces.contains_key(&edge) {
                    continue;
                }
                let next_dir = normalize3(sub3(
                    mesh.positions[v as usize],
                    mesh.positions[cur as usize],
                ));
                let d = dot3(dir, next_dir);
                if d > best_dot {
                    best_dot = d;
                    best_v = Some(v);
                }
            }
        }

        match best_v {
            None => break,
            Some(next) => {
                if next == start_a && path.len() >= 3 {
                    // Closed loop detected
                    path.push(next);
                    break;
                }
                if path.contains(&next) {
                    // We've revisited a vertex that is not the start — stop.
                    break;
                }
                path.push(next);
                prev = cur;
                cur = next;
            }
        }
    }

    path
}

// ---------------------------------------------------------------------------
// boundary_edges
// ---------------------------------------------------------------------------

/// Find all boundary edges: edges belonging to exactly one triangle face.
#[allow(dead_code)]
pub fn boundary_edges(mesh: &MeshBuffers) -> Vec<Edge> {
    let edge_to_faces = build_edge_to_faces(mesh);
    edge_to_faces
        .into_iter()
        .filter(|(_, faces)| faces.len() == 1)
        .map(|(e, _)| e)
        .collect()
}

// ---------------------------------------------------------------------------
// sharp_edges
// ---------------------------------------------------------------------------

/// Compute the face normal for triangle `(v0, v1, v2)` using cross product.
fn face_normal(positions: &[[f32; 3]], v0: u32, v1: u32, v2: u32) -> [f32; 3] {
    let p0 = positions[v0 as usize];
    let p1 = positions[v1 as usize];
    let p2 = positions[v2 as usize];
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    normalize3(cross3(e1, e2))
}

/// Find sharp edges: edges where the dihedral angle exceeds `threshold_rad`.
#[allow(dead_code)]
pub fn sharp_edges(mesh: &MeshBuffers, threshold_rad: f32) -> Vec<Edge> {
    let edge_to_faces = build_edge_to_faces(mesh);
    let faces: Vec<[u32; 3]> = mesh
        .indices
        .chunks_exact(3)
        .map(|t| [t[0], t[1], t[2]])
        .collect();

    let cos_thresh = threshold_rad.cos();

    edge_to_faces
        .into_iter()
        .filter(|(_, face_ids)| {
            if face_ids.len() != 2 {
                return false;
            }
            let f0 = faces[face_ids[0]];
            let f1 = faces[face_ids[1]];
            let n0 = face_normal(&mesh.positions, f0[0], f0[1], f0[2]);
            let n1 = face_normal(&mesh.positions, f1[0], f1[1], f1[2]);
            let d = dot3(n0, n1).clamp(-1.0, 1.0);
            // sharp when the angle between normals > threshold
            d < cos_thresh
        })
        .map(|(e, _)| e)
        .collect()
}

// ---------------------------------------------------------------------------
// uv_seam_edges
// ---------------------------------------------------------------------------

/// Find UV seam edges: edges where the UV coordinates differ by more than `uv_threshold`
/// across their two vertices (i.e., the UV mapping is discontinuous there).
#[allow(dead_code)]
pub fn uv_seam_edges(mesh: &MeshBuffers, uv_threshold: f32) -> Vec<Edge> {
    if mesh.uvs.is_empty() {
        return Vec::new();
    }
    let edge_to_faces = build_edge_to_faces(mesh);
    let faces: Vec<[u32; 3]> = mesh
        .indices
        .chunks_exact(3)
        .map(|t| [t[0], t[1], t[2]])
        .collect();

    // For each shared edge (two faces), compare UVs of the shared vertices
    // as seen from each adjacent face. If any UV differs beyond threshold → seam.
    edge_to_faces
        .into_iter()
        .filter(|(edge, face_ids)| {
            if face_ids.len() != 2 {
                return false;
            }
            // For each shared vertex, get UVs from both faces and compare.
            // A seam occurs when the same geometric position has different UV
            // coordinates in different adjacent faces.

            // Also look at UVs of the shared vertices from the perspective of
            // each neighbouring face to detect welded-position / split-UV seams.
            let f0 = faces[face_ids[0]];
            let f1 = faces[face_ids[1]];

            // Vertices of f0 that match edge.a and edge.b (positionally)
            let pos_a = mesh.positions[edge.a as usize];
            let pos_b = mesh.positions[edge.b as usize];

            let uv_from_face = |face: [u32; 3], target_pos: [f32; 3]| -> Option<[f32; 2]> {
                for &vi in &face {
                    let p = mesh.positions[vi as usize];
                    let dist2 = (p[0] - target_pos[0]).powi(2)
                        + (p[1] - target_pos[1]).powi(2)
                        + (p[2] - target_pos[2]).powi(2);
                    if dist2 < 1e-10 {
                        return Some(mesh.uvs[vi as usize]);
                    }
                }
                None
            };

            let check_seam = |uv1: [f32; 2], uv2: [f32; 2]| -> bool {
                let du = (uv1[0] - uv2[0]).abs();
                let dv = (uv1[1] - uv2[1]).abs();
                du > uv_threshold || dv > uv_threshold
            };

            // Compare UV of vertex `a` across both faces
            if let (Some(u0), Some(u1)) = (uv_from_face(f0, pos_a), uv_from_face(f1, pos_a)) {
                if check_seam(u0, u1) {
                    return true;
                }
            }
            // Compare UV of vertex `b` across both faces
            if let (Some(u0), Some(u1)) = (uv_from_face(f0, pos_b), uv_from_face(f1, pos_b)) {
                if check_seam(u0, u1) {
                    return true;
                }
            }

            // Both vertex positions were found in both faces, and UVs match.
            // No seam detected via positional comparison.
            false
        })
        .map(|(e, _)| e)
        .collect()
}

// ---------------------------------------------------------------------------
// edges_to_loops
// ---------------------------------------------------------------------------

/// Given a set of edges, find all closed loops (cycles) within them.
/// Each returned `Vec<u32>` is a closed sequence of vertex indices
/// where the first and last element are the same vertex.
#[allow(dead_code)]
pub fn edges_to_loops(edges: &[Edge]) -> Vec<Vec<u32>> {
    // Build adjacency restricted to the given edge set.
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for e in edges {
        adj.entry(e.a).or_default().push(e.b);
        adj.entry(e.b).or_default().push(e.a);
    }

    let mut visited_edges: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
    let mut loops: Vec<Vec<u32>> = Vec::new();

    // For each vertex that has exactly 2 neighbours (can be part of a loop),
    // try to trace a cycle.
    let vertices: Vec<u32> = adj.keys().cloned().collect();
    for start in vertices {
        if adj[&start].len() != 2 {
            continue;
        }
        // Try to trace a loop starting from `start`.
        let first_next = adj[&start][0];
        let edge_key = if start < first_next {
            (start, first_next)
        } else {
            (first_next, start)
        };
        if visited_edges.contains(&edge_key) {
            continue;
        }

        let mut path: Vec<u32> = vec![start];
        let mut prev = start;
        let mut cur = first_next;

        loop {
            path.push(cur);
            let ek = if prev < cur { (prev, cur) } else { (cur, prev) };
            visited_edges.insert(ek);

            if cur == start {
                // Closed loop found.
                loops.push(path);
                break;
            }

            // Find next vertex: neighbour of `cur` that is not `prev`.
            let neighbours = match adj.get(&cur) {
                Some(n) => n,
                None => break,
            };
            let next_candidates: Vec<u32> =
                neighbours.iter().cloned().filter(|&v| v != prev).collect();

            if next_candidates.len() != 1 {
                // Branching or dead-end — not a simple loop from this start.
                break;
            }
            prev = cur;
            cur = next_candidates[0];

            // Safety: avoid infinite loops on degenerate input.
            if path.len() > edges.len() + 2 {
                break;
            }
        }
    }
    loops
}

// ---------------------------------------------------------------------------
// edges_to_chains
// ---------------------------------------------------------------------------

/// Given a set of edges, find all open chains (non-cyclic paths).
/// Each returned `Vec<u32>` is a sequence of vertex indices from one endpoint to another.
#[allow(dead_code)]
pub fn edges_to_chains(edges: &[Edge]) -> Vec<Vec<u32>> {
    // Build adjacency.
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for e in edges {
        adj.entry(e.a).or_default().push(e.b);
        adj.entry(e.b).or_default().push(e.a);
    }

    let mut visited_edges: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
    let mut chains: Vec<Vec<u32>> = Vec::new();

    // Endpoints of chains are vertices with degree 1 (or odd degree).
    // Start chains from degree-1 vertices to avoid duplicates.
    let endpoints: Vec<u32> = adj
        .iter()
        .filter(|(_, nbrs)| nbrs.len() == 1)
        .map(|(&v, _)| v)
        .collect();

    for start in endpoints {
        // Check if all edges from this start have been visited.
        let first_next = adj[&start][0];
        let ek = if start < first_next {
            (start, first_next)
        } else {
            (first_next, start)
        };
        if visited_edges.contains(&ek) {
            continue;
        }

        let mut path: Vec<u32> = vec![start];
        let mut prev = start;
        let mut cur = first_next;

        loop {
            path.push(cur);
            let edge_key = if prev < cur { (prev, cur) } else { (cur, prev) };
            visited_edges.insert(edge_key);

            // Find next: neighbour of `cur` that is not `prev` and has an unvisited edge.
            let neighbours = match adj.get(&cur) {
                Some(n) => n.clone(),
                None => break,
            };
            let next_candidates: Vec<u32> = neighbours
                .iter()
                .cloned()
                .filter(|&v| {
                    if v == prev {
                        return false;
                    }
                    let ek2 = if cur < v { (cur, v) } else { (v, cur) };
                    !visited_edges.contains(&ek2)
                })
                .collect();

            if next_candidates.is_empty() {
                break;
            }
            prev = cur;
            cur = next_candidates[0];

            if path.len() > edges.len() + 2 {
                break;
            }
        }

        if path.len() >= 2 {
            chains.push(path);
        }
    }
    chains
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a MeshBuffers directly (no morph dependency needed).
    fn make_mesh(
        positions: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
        uvs: Vec<[f32; 2]>,
        indices: Vec<u32>,
    ) -> MeshBuffers {
        MeshBuffers {
            positions,
            normals,
            tangents: vec![],
            uvs,
            indices,
            colors: None,
            has_suit: false,
        }
    }

    fn single_triangle() -> MeshBuffers {
        make_mesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            vec![[0., 0., 1.]; 3],
            vec![[0., 0.]; 3],
            vec![0, 1, 2],
        )
    }

    fn tetrahedron() -> MeshBuffers {
        make_mesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            vec![[0., 0., 1.]; 4],
            vec![[0., 0.]; 4],
            vec![0, 1, 2, 0, 1, 3, 0, 2, 3, 1, 2, 3],
        )
    }

    /// Flat quad mesh (two coplanar triangles sharing an edge).
    fn flat_quad() -> MeshBuffers {
        make_mesh(
            vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
            vec![[0., 0., 1.]; 4],
            vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]],
            vec![0, 1, 2, 0, 2, 3],
        )
    }

    /// Simple cube mesh (12 triangles, 8 vertices).
    fn cube_mesh() -> MeshBuffers {
        // 8 vertices
        let positions = vec![
            // bottom face
            [0., 0., 0.],
            [1., 0., 0.],
            [1., 1., 0.],
            [0., 1., 0.],
            // top face
            [0., 0., 1.],
            [1., 0., 1.],
            [1., 1., 1.],
            [0., 1., 1.],
        ];
        let normals = vec![[0., 0., 1.]; 8];
        let uvs = vec![[0., 0.]; 8];
        // 6 faces × 2 triangles = 12 triangles
        let indices = vec![
            // bottom (z=0)
            0, 1, 2, 0, 2, 3, // top (z=1)
            4, 6, 5, 4, 7, 6, // front (y=0)
            0, 5, 1, 0, 4, 5, // back (y=1)
            2, 7, 3, 2, 6, 7, // left (x=0)
            0, 3, 7, 0, 7, 4, // right (x=1)
            1, 5, 6, 1, 6, 2,
        ];
        make_mesh(positions, normals, uvs, indices)
    }

    // -----------------------------------------------------------------------
    // Edge::new tests
    // -----------------------------------------------------------------------

    #[test]
    fn edge_new_normalizes_order() {
        let e = Edge::new(5, 3);
        assert_eq!(e.a, 3);
        assert_eq!(e.b, 5);
    }

    #[test]
    fn edge_new_already_ordered() {
        let e = Edge::new(2, 7);
        assert_eq!(e.a, 2);
        assert_eq!(e.b, 7);
    }

    // -----------------------------------------------------------------------
    // Edge::contains
    // -----------------------------------------------------------------------

    #[test]
    fn edge_contains_vertex() {
        let e = Edge::new(3, 7);
        assert!(e.contains(3));
        assert!(e.contains(7));
        assert!(!e.contains(5));
    }

    // -----------------------------------------------------------------------
    // Edge::other
    // -----------------------------------------------------------------------

    #[test]
    fn edge_other_returns_opposite() {
        let e = Edge::new(3, 7);
        assert_eq!(e.other(3), Some(7));
        assert_eq!(e.other(7), Some(3));
        assert_eq!(e.other(99), None);
    }

    // -----------------------------------------------------------------------
    // extract_edges
    // -----------------------------------------------------------------------

    #[test]
    fn extract_edges_triangle_count() {
        let mesh = single_triangle();
        let edges = extract_edges(&mesh);
        assert_eq!(edges.len(), 3, "single triangle has 3 edges");
    }

    #[test]
    fn extract_edges_no_duplicates() {
        let mesh = flat_quad();
        let edges = extract_edges(&mesh);
        // 5 unique edges in a quad split into 2 triangles (4 perimeter + 1 diagonal)
        let mut sorted = edges.clone();
        sorted.sort_by_key(|e| (e.a, e.b));
        let dedup_len = {
            let mut v = sorted.clone();
            v.dedup();
            v.len()
        };
        assert_eq!(sorted.len(), dedup_len, "no duplicate edges");
        assert_eq!(edges.len(), 5, "quad has 5 unique edges");
    }

    // -----------------------------------------------------------------------
    // boundary_edges
    // -----------------------------------------------------------------------

    #[test]
    fn boundary_edges_open_mesh() {
        let mesh = single_triangle();
        let be = boundary_edges(&mesh);
        assert_eq!(be.len(), 3, "single triangle: all 3 edges are boundary");
    }

    #[test]
    fn boundary_edges_closed_mesh() {
        let mesh = tetrahedron();
        let be = boundary_edges(&mesh);
        assert_eq!(be.len(), 0, "closed tetrahedron has no boundary edges");
    }

    // -----------------------------------------------------------------------
    // edge_adjacency
    // -----------------------------------------------------------------------

    #[test]
    fn edge_adjacency_triangle() {
        let mesh = single_triangle();
        let adj = edge_adjacency(&mesh);
        // Each vertex of a triangle connects to exactly 2 others
        assert_eq!(adj.len(), 3);
        for neighbours in &adj {
            assert_eq!(neighbours.len(), 2, "each vertex has 2 neighbours");
        }
    }

    // -----------------------------------------------------------------------
    // edges_to_loops
    // -----------------------------------------------------------------------

    #[test]
    fn edges_to_loops_simple_triangle() {
        // Three edges forming a triangle
        let edges = vec![Edge::new(0, 1), Edge::new(1, 2), Edge::new(2, 0)];
        let loops = edges_to_loops(&edges);
        assert_eq!(loops.len(), 1, "exactly one loop");
        let lp = &loops[0];
        // Closed: first == last
        assert_eq!(lp[0], *lp.last().expect("should succeed"));
    }

    // -----------------------------------------------------------------------
    // edges_to_chains
    // -----------------------------------------------------------------------

    #[test]
    fn edges_to_chains_open_path() {
        // Linear path: 0-1-2-3
        let edges = vec![Edge::new(0, 1), Edge::new(1, 2), Edge::new(2, 3)];
        let chains = edges_to_chains(&edges);
        assert_eq!(chains.len(), 1, "exactly one chain");
        let ch = &chains[0];
        assert_eq!(ch.len(), 4, "chain visits 4 vertices");
    }

    // -----------------------------------------------------------------------
    // sharp_edges
    // -----------------------------------------------------------------------

    #[test]
    fn sharp_edges_flat_mesh_none() {
        let mesh = flat_quad();
        // The two triangles are coplanar (z=0), so the dihedral angle is 0.
        let se = sharp_edges(&mesh, 0.01f32.to_radians()); // threshold: 0.01°
                                                           // Only the shared diagonal edge could be sharp; for coplanar faces it won't be.
        assert_eq!(se.len(), 0, "flat mesh has no sharp edges");
    }

    #[test]
    fn sharp_edges_cube_has_edges() {
        let mesh = cube_mesh();
        // Cube has 90° dihedral angles at all edges between faces.
        // Threshold 45° → all such edges are sharp.
        let threshold = std::f32::consts::FRAC_PI_4; // 45°
        let se = sharp_edges(&mesh, threshold);
        assert!(
            !se.is_empty(),
            "cube must have sharp edges at 90° threshold=45°"
        );
    }

    // -----------------------------------------------------------------------
    // uv_seam_edges
    // -----------------------------------------------------------------------

    #[test]
    fn uv_seam_edges_no_seam_mesh() {
        // A mesh where UVs are consistent across edges (no discontinuities).
        let mesh = flat_quad();
        let seams = uv_seam_edges(&mesh, 0.01);
        // The diagonal shared edge: both adjacent faces use the *same* vertex indices,
        // so UVs are by definition the same → no seam.
        assert_eq!(seams.len(), 0, "flat quad with consistent UVs has no seams");
    }

    // -----------------------------------------------------------------------
    // extract_edge_loop
    // -----------------------------------------------------------------------

    #[test]
    fn extract_edge_loop_min_length() {
        let mesh = flat_quad();
        let loop_path = extract_edge_loop(&mesh, 0, 1, 100);
        // Should return at least 2 vertices (start + at least one step).
        assert!(
            loop_path.len() >= 2,
            "edge loop should have at least 2 vertices"
        );
    }
}
