// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simplicial chain complex for mesh topology: vertices, edges, faces, boundary operator.

/// A simplicial chain complex built from a triangle mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainComplex {
    pub num_vertices: usize,
    pub edges: Vec<(u32, u32)>,
    pub triangles: Vec<(u32, u32, u32)>,
}

/// Build a chain complex from mesh indices.
#[allow(dead_code)]
pub fn build_chain_complex(num_verts: usize, indices: &[u32]) -> ChainComplex {
    use std::collections::HashSet;
    let tc = indices.len() / 3;
    let mut edge_set: HashSet<(u32, u32)> = HashSet::new();
    let mut tris = Vec::with_capacity(tc);
    for t in 0..tc {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        tris.push((a, b, c));
        for &(x, y) in &[(a, b), (b, c), (c, a)] {
            let e = if x < y { (x, y) } else { (y, x) };
            edge_set.insert(e);
        }
    }
    let mut edges: Vec<(u32, u32)> = edge_set.into_iter().collect();
    edges.sort();
    ChainComplex { num_vertices: num_verts, edges, triangles: tris }
}

/// Number of 0-simplices (vertices).
#[allow(dead_code)]
pub fn chain_vertex_count(cc: &ChainComplex) -> usize {
    cc.num_vertices
}

/// Number of 1-simplices (edges).
#[allow(dead_code)]
pub fn chain_edge_count(cc: &ChainComplex) -> usize {
    cc.edges.len()
}

/// Number of 2-simplices (triangles).
#[allow(dead_code)]
pub fn chain_triangle_count(cc: &ChainComplex) -> usize {
    cc.triangles.len()
}

/// Euler characteristic: V - E + F.
#[allow(dead_code)]
pub fn chain_euler_characteristic(cc: &ChainComplex) -> i64 {
    cc.num_vertices as i64 - cc.edges.len() as i64 + cc.triangles.len() as i64
}

/// Boundary of a triangle: returns its 3 edges as sorted pairs.
#[allow(dead_code)]
pub fn triangle_boundary(tri: (u32, u32, u32)) -> [(u32, u32); 3] {
    let (a, b, c) = tri;
    let mut edges = [
        if a < b { (a, b) } else { (b, a) },
        if b < c { (b, c) } else { (c, b) },
        if c < a { (c, a) } else { (a, c) },
    ];
    edges.sort();
    edges
}

/// Boundary of an edge: returns its 2 vertices.
#[allow(dead_code)]
pub fn edge_boundary(e: (u32, u32)) -> [u32; 2] {
    [e.0, e.1]
}

/// Check if the boundary of a boundary is empty (should always be true for valid complex).
#[allow(dead_code)]
pub fn boundary_squared_is_zero(cc: &ChainComplex) -> bool {
    // boundary(boundary(triangle)) should be the empty set in Z_2 homology
    for &tri in &cc.triangles {
        let edges = triangle_boundary(tri);
        let mut vertex_count = std::collections::HashMap::new();
        for e in &edges {
            let verts = edge_boundary(*e);
            for &v in &verts {
                *vertex_count.entry(v).or_insert(0u32) += 1;
            }
        }
        // In Z_2, each vertex should appear exactly 2 times (even), so boundary is 0.
        for &count in vertex_count.values() {
            if count % 2 != 0 {
                return false;
            }
        }
    }
    true
}

/// Betti number b0 estimate: connected components via union-find.
#[allow(dead_code)]
pub fn betti_0_estimate(cc: &ChainComplex) -> usize {
    let n = cc.num_vertices;
    if n == 0 { return 0; }
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }
    for &(a, b) in &cc.edges {
        let ra = find(&mut parent, a as usize);
        let rb = find(&mut parent, b as usize);
        if ra != rb {
            parent[ra] = rb;
        }
    }
    let mut roots = std::collections::HashSet::new();
    for i in 0..n {
        roots.insert(find(&mut parent, i));
    }
    roots.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tri() -> ChainComplex {
        build_chain_complex(3, &[0, 1, 2])
    }

    fn tetra() -> ChainComplex {
        build_chain_complex(4, &[0, 1, 2, 0, 3, 1, 0, 2, 3, 1, 3, 2])
    }

    #[test]
    fn test_vertex_count() {
        assert_eq!(chain_vertex_count(&simple_tri()), 3);
    }

    #[test]
    fn test_edge_count_triangle() {
        assert_eq!(chain_edge_count(&simple_tri()), 3);
    }

    #[test]
    fn test_triangle_count() {
        assert_eq!(chain_triangle_count(&simple_tri()), 1);
    }

    #[test]
    fn test_euler_triangle() {
        assert_eq!(chain_euler_characteristic(&simple_tri()), 1);
    }

    #[test]
    fn test_euler_tetrahedron() {
        let cc = tetra();
        // V=4, E=6, F=4 => chi=2
        assert_eq!(chain_euler_characteristic(&cc), 2);
    }

    #[test]
    fn test_boundary_squared_zero() {
        assert!(boundary_squared_is_zero(&simple_tri()));
        assert!(boundary_squared_is_zero(&tetra()));
    }

    #[test]
    fn test_triangle_boundary() {
        let edges = triangle_boundary((0, 1, 2));
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_edge_boundary() {
        let verts = edge_boundary((3, 7));
        assert_eq!(verts, [3, 7]);
    }

    #[test]
    fn test_betti_0_single_component() {
        assert_eq!(betti_0_estimate(&simple_tri()), 1);
    }

    #[test]
    fn test_betti_0_two_components() {
        // Two disconnected triangles
        let cc = build_chain_complex(6, &[0, 1, 2, 3, 4, 5]);
        assert_eq!(betti_0_estimate(&cc), 2);
    }

}
