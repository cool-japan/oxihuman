#![allow(dead_code)]
//! Vertex ring (1-ring neighborhood).

/// A vertex ring (1-ring neighborhood of a vertex).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexRing {
    pub center: u32,
    pub neighbors: Vec<u32>,
}

/// Compute the 1-ring neighborhood of a vertex.
#[allow(dead_code)]
pub fn vertex_one_ring(indices: &[[u32; 3]], vertex: u32) -> Vec<u32> {
    use std::collections::BTreeSet;
    let mut neighbors = BTreeSet::new();
    for tri in indices {
        for i in 0..3 {
            if tri[i] == vertex {
                neighbors.insert(tri[(i + 1) % 3]);
                neighbors.insert(tri[(i + 2) % 3]);
            }
        }
    }
    neighbors.into_iter().collect()
}

/// Return the number of neighbors in a ring.
#[allow(dead_code)]
pub fn ring_size(ring: &[u32]) -> usize {
    ring.len()
}

/// Compute centroid of ring neighbor positions.
#[allow(dead_code)]
pub fn ring_centroid(positions: &[[f32; 3]], ring: &[u32]) -> [f32; 3] {
    if ring.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for &v in ring {
        let p = positions[v as usize];
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    let n = ring.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Estimate ring normal as average of incident face normals.
#[allow(dead_code)]
pub fn ring_normal(positions: &[[f32; 3]], indices: &[[u32; 3]], vertex: u32) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    for tri in indices {
        if tri.contains(&vertex) {
            let a = positions[tri[0] as usize];
            let b = positions[tri[1] as usize];
            let c = positions[tri[2] as usize];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            sum[0] += ab[1] * ac[2] - ab[2] * ac[1];
            sum[1] += ab[2] * ac[0] - ab[0] * ac[2];
            sum[2] += ab[0] * ac[1] - ab[1] * ac[0];
        }
    }
    let len = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt();
    if len < 1e-12 {
        return [0.0; 3];
    }
    [sum[0] / len, sum[1] / len, sum[2] / len]
}

/// Check if a vertex is on the boundary (has an edge with only one incident face).
#[allow(dead_code)]
pub fn ring_is_boundary(indices: &[[u32; 3]], vertex: u32) -> bool {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices {
        if tri.contains(&vertex) {
            for i in 0..3 {
                let a = tri[i];
                let b = tri[(i + 1) % 3];
                let key = if a < b { (a, b) } else { (b, a) };
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }
    }
    edge_count.values().any(|&c| c == 1)
}

/// Compute the total area of faces incident to a vertex.
#[allow(dead_code)]
pub fn ring_area(positions: &[[f32; 3]], indices: &[[u32; 3]], vertex: u32) -> f32 {
    let mut total = 0.0f32;
    for tri in indices {
        if tri.contains(&vertex) {
            let a = positions[tri[0] as usize];
            let b = positions[tri[1] as usize];
            let c = positions[tri[2] as usize];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            total += 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        }
    }
    total
}

/// Return edges incident to a vertex as pairs.
#[allow(dead_code)]
pub fn ring_edges(indices: &[[u32; 3]], vertex: u32) -> Vec<(u32, u32)> {
    use std::collections::BTreeSet;
    let mut edges = BTreeSet::new();
    for tri in indices {
        if tri.contains(&vertex) {
            for i in 0..3 {
                let a = tri[i];
                let b = tri[(i + 1) % 3];
                if a == vertex || b == vertex {
                    let key = if a < b { (a, b) } else { (b, a) };
                    edges.insert(key);
                }
            }
        }
    }
    edges.into_iter().collect()
}

/// Convert a VertexRing to a Vec of neighbor indices.
#[allow(dead_code)]
pub fn ring_to_vec(ring: &VertexRing) -> Vec<u32> {
    ring.neighbors.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        ];
        let i = vec![[0, 1, 2], [0, 2, 3]];
        (p, i)
    }

    #[test]
    fn test_vertex_one_ring() {
        let (_, i) = quad_mesh();
        let ring = vertex_one_ring(&i, 0);
        assert_eq!(ring.len(), 3);
    }

    #[test]
    fn test_ring_size() {
        assert_eq!(ring_size(&[1, 2, 3]), 3);
    }

    #[test]
    fn test_ring_centroid() {
        let p = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let c = ring_centroid(&p, &[0, 1, 2]);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ring_centroid_empty() {
        let c = ring_centroid(&[], &[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_ring_normal() {
        let (p, i) = quad_mesh();
        let n = ring_normal(&p, &i, 0);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_ring_is_boundary() {
        let (_, i) = quad_mesh();
        assert!(ring_is_boundary(&i, 1));
    }

    #[test]
    fn test_ring_area() {
        let (p, i) = quad_mesh();
        let a = ring_area(&p, &i, 0);
        assert!(a > 0.0);
    }

    #[test]
    fn test_ring_edges() {
        let (_, i) = quad_mesh();
        let edges = ring_edges(&i, 0);
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_ring_to_vec() {
        let ring = VertexRing { center: 0, neighbors: vec![1, 2, 3] };
        let v = ring_to_vec(&ring);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_vertex_one_ring_isolated() {
        let i: Vec<[u32; 3]> = vec![[1, 2, 3]];
        let ring = vertex_one_ring(&i, 0);
        assert!(ring.is_empty());
    }
}
