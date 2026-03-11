#![allow(dead_code)]

//! Edge utility functions.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeUtil {
    pub a: u32,
    pub b: u32,
}

#[allow(dead_code)]
pub fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

#[allow(dead_code)]
pub fn edge_midpoint(positions: &[[f32; 3]], a: u32, b: u32) -> [f32; 3] {
    let pa = positions[a as usize];
    let pb = positions[b as usize];
    [(pa[0]+pb[0])*0.5, (pa[1]+pb[1])*0.5, (pa[2]+pb[2])*0.5]
}

#[allow(dead_code)]
pub fn edge_vector(positions: &[[f32; 3]], a: u32, b: u32) -> [f32; 3] {
    let pa = positions[a as usize];
    let pb = positions[b as usize];
    [pb[0]-pa[0], pb[1]-pa[1], pb[2]-pa[2]]
}

#[allow(dead_code)]
pub fn edge_is_degenerate(positions: &[[f32; 3]], a: u32, b: u32, eps: f32) -> bool {
    let v = edge_vector(positions, a, b);
    v[0]*v[0] + v[1]*v[1] + v[2]*v[2] < eps * eps
}

#[allow(dead_code)]
pub fn edges_share_vertex(e1: (u32, u32), e2: (u32, u32)) -> bool {
    e1.0 == e2.0 || e1.0 == e2.1 || e1.1 == e2.0 || e1.1 == e2.1
}

#[allow(dead_code)]
pub fn edge_face_count(indices: &[u32], a: u32, b: u32) -> usize {
    let key = edge_key(a, b);
    let mut count = 0;
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
            for &(ea, eb) in &edges {
                if edge_key(ea, eb) == key {
                    count += 1;
                }
            }
        }
    }
    count
}

#[allow(dead_code)]
pub fn edge_opposite_vertex(indices: &[u32], a: u32, b: u32) -> Option<u32> {
    let key = edge_key(a, b);
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            let has_edge = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])]
                .iter().any(|&(ea, eb)| edge_key(ea, eb) == key);
            if has_edge {
                for &v in tri {
                    if v != a && v != b {
                        return Some(v);
                    }
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn edge_to_json(positions: &[[f32; 3]], a: u32, b: u32) -> String {
    let mid = edge_midpoint(positions, a, b);
    let v = edge_vector(positions, a, b);
    let len = (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();
    format!("{{\"a\":{},\"b\":{},\"length\":{:.6},\"midpoint\":[{:.4},{:.4},{:.4}]}}", a, b, len, mid[0], mid[1], mid[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[2.0,0.0,0.0],[0.0,2.0,0.0]]
    }

    #[test]
    fn test_edge_key_order() {
        assert_eq!(edge_key(5, 3), (3, 5));
        assert_eq!(edge_key(3, 5), (3, 5));
    }

    #[test]
    fn test_edge_midpoint() {
        let m = edge_midpoint(&pos(), 0, 1);
        assert!((m[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_edge_vector() {
        let v = edge_vector(&pos(), 0, 1);
        assert!((v[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_edge_is_degenerate() {
        let p = vec![[0.0,0.0,0.0],[0.0,0.0,0.0]];
        assert!(edge_is_degenerate(&p, 0, 1, 0.001));
    }

    #[test]
    fn test_edge_not_degenerate() {
        assert!(!edge_is_degenerate(&pos(), 0, 1, 0.001));
    }

    #[test]
    fn test_edges_share_vertex() {
        assert!(edges_share_vertex((0, 1), (1, 2)));
        assert!(!edges_share_vertex((0, 1), (2, 3)));
    }

    #[test]
    fn test_edge_face_count() {
        let idx = vec![0, 1, 2];
        assert_eq!(edge_face_count(&idx, 0, 1), 1);
    }

    #[test]
    fn test_edge_opposite_vertex() {
        let idx = vec![0, 1, 2];
        assert_eq!(edge_opposite_vertex(&idx, 0, 1), Some(2));
    }

    #[test]
    fn test_edge_to_json() {
        let j = edge_to_json(&pos(), 0, 1);
        assert!(j.contains("\"a\":0"));
    }

    #[test]
    fn test_edge_opposite_none() {
        let idx = vec![0, 1, 2];
        assert_eq!(edge_opposite_vertex(&idx, 3, 4), None);
    }
}
