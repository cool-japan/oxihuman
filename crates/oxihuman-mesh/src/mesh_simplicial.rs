// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simplicial complex operations (chain complex, boundary operators).

use std::collections::HashMap;

/// A 0-simplex (vertex).
#[allow(dead_code)]
pub struct Simplex0 {
    pub vertex: usize,
}

/// A 1-simplex (edge).
#[allow(dead_code)]
pub struct Simplex1 {
    pub v0: usize,
    pub v1: usize,
}

/// A 2-simplex (triangle).
#[allow(dead_code)]
pub struct Simplex2 {
    pub v0: usize,
    pub v1: usize,
    pub v2: usize,
}

/// A simplicial complex containing vertices, edges, and triangles.
#[allow(dead_code)]
pub struct SimplicialComplex {
    pub vertices: Vec<Simplex0>,
    pub edges: Vec<Simplex1>,
    pub triangles: Vec<Simplex2>,
    pub vertex_count: usize,
}

/// Create a new empty `SimplicialComplex` with `vertex_count` vertices.
#[allow(dead_code)]
pub fn new_simplicial_complex(vertex_count: usize) -> SimplicialComplex {
    let vertices = (0..vertex_count).map(|i| Simplex0 { vertex: i }).collect();
    SimplicialComplex {
        vertices,
        edges: Vec::new(),
        triangles: Vec::new(),
        vertex_count,
    }
}

/// Canonical edge key: always (min, max).
fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Add an edge to the complex if not already present.
#[allow(dead_code)]
pub fn add_simplex1(complex: &mut SimplicialComplex, v0: usize, v1: usize) {
    let key = edge_key(v0, v1);
    let already = complex.edges.iter().any(|e| edge_key(e.v0, e.v1) == key);
    if !already {
        complex.edges.push(Simplex1 { v0, v1 });
    }
}

/// Add a triangle to the complex, also auto-adding its three edges.
#[allow(dead_code)]
pub fn add_simplex2(complex: &mut SimplicialComplex, v0: usize, v1: usize, v2: usize) {
    add_simplex1(complex, v0, v1);
    add_simplex1(complex, v1, v2);
    add_simplex1(complex, v0, v2);
    // Check for duplicate triangle
    let already = complex.triangles.iter().any(|t| {
        let mut tv = [t.v0, t.v1, t.v2];
        let mut nv = [v0, v1, v2];
        tv.sort();
        nv.sort();
        tv == nv
    });
    if !already {
        complex.triangles.push(Simplex2 { v0, v1, v2 });
    }
}

/// Return the boundary of an edge: [Simplex0{v0}, Simplex0{v1}].
#[allow(dead_code)]
pub fn boundary_of_edge(e: &Simplex1) -> [Simplex0; 2] {
    [Simplex0 { vertex: e.v0 }, Simplex0 { vertex: e.v1 }]
}

/// Return the three boundary edges of a triangle, with canonical vertex ordering.
#[allow(dead_code)]
pub fn boundary_of_triangle(t: &Simplex2) -> [Simplex1; 3] {
    let (a, b, c) = (t.v0, t.v1, t.v2);
    [
        Simplex1 {
            v0: a.min(b),
            v1: a.max(b),
        },
        Simplex1 {
            v0: b.min(c),
            v1: b.max(c),
        },
        Simplex1 {
            v0: a.min(c),
            v1: a.max(c),
        },
    ]
}

/// Euler characteristic: V - E + F.
#[allow(dead_code)]
pub fn euler_characteristic(complex: &SimplicialComplex) -> i32 {
    complex.vertex_count as i32 - complex.edges.len() as i32 + complex.triangles.len() as i32
}

/// Count connected components (Betti-0) via Union-Find.
#[allow(dead_code)]
pub fn betti_0(complex: &SimplicialComplex) -> usize {
    let n = complex.vertex_count;
    if n == 0 {
        return 0;
    }
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[ra] = rb;
        }
    }

    for e in &complex.edges {
        if e.v0 < n && e.v1 < n {
            union(&mut parent, e.v0, e.v1);
        }
    }

    let mut roots = std::collections::HashSet::new();
    for i in 0..n {
        roots.insert(find(&mut parent, i));
    }
    roots.len()
}

/// Build a `SimplicialComplex` from a flat triangle index buffer.
#[allow(dead_code)]
pub fn from_mesh_indices(vertex_count: usize, indices: &[u32]) -> SimplicialComplex {
    let mut complex = new_simplicial_complex(vertex_count);
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let v0 = indices[f * 3] as usize;
        let v1 = indices[f * 3 + 1] as usize;
        let v2 = indices[f * 3 + 2] as usize;
        add_simplex2(&mut complex, v0, v1, v2);
    }
    complex
}

/// Return the number of edges in the complex.
#[allow(dead_code)]
pub fn simplex1_count(complex: &SimplicialComplex) -> usize {
    complex.edges.len()
}

/// Return the number of triangles in the complex.
#[allow(dead_code)]
pub fn simplex2_count(complex: &SimplicialComplex) -> usize {
    complex.triangles.len()
}

/// Return true if every edge is shared by at most 2 triangles.
#[allow(dead_code)]
pub fn is_manifold_simplex(complex: &SimplicialComplex) -> bool {
    let mut edge_face_count: HashMap<(usize, usize), usize> = HashMap::new();
    for t in &complex.triangles {
        let edges = [
            edge_key(t.v0, t.v1),
            edge_key(t.v1, t.v2),
            edge_key(t.v0, t.v2),
        ];
        for ek in edges {
            *edge_face_count.entry(ek).or_insert(0) += 1;
        }
    }
    edge_face_count.values().all(|&c| c <= 2)
}

/// Return references to edges that are shared by exactly one triangle (boundary).
#[allow(dead_code)]
pub fn boundary_edges_simplex(complex: &SimplicialComplex) -> Vec<&Simplex1> {
    let mut edge_face_count: HashMap<(usize, usize), usize> = HashMap::new();
    for t in &complex.triangles {
        let edges = [
            edge_key(t.v0, t.v1),
            edge_key(t.v1, t.v2),
            edge_key(t.v0, t.v2),
        ];
        for ek in edges {
            *edge_face_count.entry(ek).or_insert(0) += 1;
        }
    }
    complex
        .edges
        .iter()
        .filter(|e| {
            let k = edge_key(e.v0, e.v1);
            edge_face_count.get(&k).copied().unwrap_or(0) == 1
        })
        .collect()
}

/// Build the dual graph: for each triangle, list the indices of adjacent triangles.
#[allow(dead_code)]
pub fn dual_graph_simplex(complex: &SimplicialComplex) -> Vec<Vec<usize>> {
    let nf = complex.triangles.len();
    // Map from canonical edge → list of triangle indices
    let mut edge_to_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, t) in complex.triangles.iter().enumerate() {
        let edges = [
            edge_key(t.v0, t.v1),
            edge_key(t.v1, t.v2),
            edge_key(t.v0, t.v2),
        ];
        for ek in edges {
            edge_to_faces.entry(ek).or_default().push(fi);
        }
    }
    let mut adj = vec![Vec::new(); nf];
    for faces in edge_to_faces.values() {
        if faces.len() == 2 {
            adj[faces[0]].push(faces[1]);
            adj[faces[1]].push(faces[0]);
        }
    }
    adj
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_simplicial_complex() {
        let c = new_simplicial_complex(5);
        assert_eq!(c.vertex_count, 5);
        assert_eq!(c.vertices.len(), 5);
        assert!(c.edges.is_empty());
        assert!(c.triangles.is_empty());
    }

    #[test]
    fn test_add_simplex1_no_duplicate() {
        let mut c = new_simplicial_complex(4);
        add_simplex1(&mut c, 0, 1);
        add_simplex1(&mut c, 0, 1);
        add_simplex1(&mut c, 1, 0); // same edge reversed
        assert_eq!(c.edges.len(), 1);
    }

    #[test]
    fn test_add_simplex2_adds_edges() {
        let mut c = new_simplicial_complex(4);
        add_simplex2(&mut c, 0, 1, 2);
        assert_eq!(c.triangles.len(), 1);
        assert_eq!(c.edges.len(), 3);
    }

    #[test]
    fn test_add_simplex2_no_duplicate_triangle() {
        let mut c = new_simplicial_complex(4);
        add_simplex2(&mut c, 0, 1, 2);
        add_simplex2(&mut c, 1, 2, 0); // same triangle
        assert_eq!(c.triangles.len(), 1);
    }

    #[test]
    fn test_boundary_of_edge() {
        let e = Simplex1 { v0: 3, v1: 7 };
        let b = boundary_of_edge(&e);
        assert_eq!(b[0].vertex, 3);
        assert_eq!(b[1].vertex, 7);
    }

    #[test]
    fn test_boundary_of_triangle_three_edges() {
        let t = Simplex2 {
            v0: 0,
            v1: 1,
            v2: 2,
        };
        let edges = boundary_of_triangle(&t);
        assert_eq!(edges.len(), 3);
        // All edges should be canonical (v0 < v1)
        for e in &edges {
            assert!(e.v0 <= e.v1);
        }
    }

    #[test]
    fn test_euler_characteristic_single_triangle() {
        // V=3, E=3, F=1  =>  3-3+1 = 1
        let mut c = new_simplicial_complex(3);
        add_simplex2(&mut c, 0, 1, 2);
        assert_eq!(euler_characteristic(&c), 1);
    }

    #[test]
    fn test_euler_characteristic_tetrahedron() {
        // Tetrahedron: V=4, E=6, F=4  => 4-6+4=2
        let mut c = new_simplicial_complex(4);
        add_simplex2(&mut c, 0, 1, 2);
        add_simplex2(&mut c, 0, 1, 3);
        add_simplex2(&mut c, 0, 2, 3);
        add_simplex2(&mut c, 1, 2, 3);
        assert_eq!(euler_characteristic(&c), 2);
    }

    #[test]
    fn test_betti_0_connected() {
        let mut c = new_simplicial_complex(4);
        add_simplex1(&mut c, 0, 1);
        add_simplex1(&mut c, 1, 2);
        add_simplex1(&mut c, 2, 3);
        assert_eq!(betti_0(&c), 1);
    }

    #[test]
    fn test_betti_0_two_components() {
        let mut c = new_simplicial_complex(4);
        add_simplex1(&mut c, 0, 1);
        add_simplex1(&mut c, 2, 3);
        assert_eq!(betti_0(&c), 2);
    }

    #[test]
    fn test_from_mesh_indices() {
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let c = from_mesh_indices(4, &indices);
        assert_eq!(simplex2_count(&c), 2);
        assert_eq!(simplex1_count(&c), 5); // shared edge counted once
    }

    #[test]
    fn test_is_manifold_simplex_true() {
        let mut c = new_simplicial_complex(4);
        add_simplex2(&mut c, 0, 1, 2);
        add_simplex2(&mut c, 0, 2, 3);
        assert!(is_manifold_simplex(&c));
    }

    #[test]
    fn test_is_manifold_simplex_false() {
        // Three triangles sharing one edge → non-manifold
        let mut c = new_simplicial_complex(5);
        add_simplex2(&mut c, 0, 1, 2);
        add_simplex2(&mut c, 0, 1, 3);
        add_simplex2(&mut c, 0, 1, 4);
        assert!(!is_manifold_simplex(&c));
    }

    #[test]
    fn test_boundary_edges_simplex() {
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let c = from_mesh_indices(4, &indices);
        let boundary = boundary_edges_simplex(&c);
        // A flat strip of two triangles has 4 boundary edges
        assert_eq!(boundary.len(), 4);
    }

    #[test]
    fn test_dual_graph_simplex_adjacency() {
        let mut c = new_simplicial_complex(4);
        add_simplex2(&mut c, 0, 1, 2);
        add_simplex2(&mut c, 0, 2, 3);
        let adj = dual_graph_simplex(&c);
        assert_eq!(adj.len(), 2);
        assert!(adj[0].contains(&1));
        assert!(adj[1].contains(&0));
    }

    #[test]
    fn test_simplex1_count_and_simplex2_count() {
        let mut c = new_simplicial_complex(4);
        add_simplex2(&mut c, 0, 1, 2);
        assert_eq!(simplex1_count(&c), 3);
        assert_eq!(simplex2_count(&c), 1);
    }
}
