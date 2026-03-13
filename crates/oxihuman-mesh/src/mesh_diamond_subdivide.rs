// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Result of diamond (sqrt3) subdivision.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiamondSubdivideResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Perform diamond subdivision: insert centroid per face, connect to original vertices.
#[allow(dead_code)]
pub fn diamond_subdivide(positions: &[[f32; 3]], indices: &[u32]) -> DiamondSubdivideResult {
    let mut new_pos = positions.to_vec();
    let mut new_idx = Vec::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let centroid = [
            (positions[a][0] + positions[b][0] + positions[c][0]) / 3.0,
            (positions[a][1] + positions[b][1] + positions[c][1]) / 3.0,
            (positions[a][2] + positions[b][2] + positions[c][2]) / 3.0,
        ];
        let ci = new_pos.len() as u32;
        new_pos.push(centroid);
        new_idx.extend_from_slice(&[tri[0], tri[1], ci]);
        new_idx.extend_from_slice(&[tri[1], tri[2], ci]);
        new_idx.extend_from_slice(&[tri[2], tri[0], ci]);
    }
    DiamondSubdivideResult { positions: new_pos, indices: new_idx }
}

/// Count resulting faces from diamond subdivision.
#[allow(dead_code)]
pub fn diamond_face_count(original_face_count: usize) -> usize {
    original_face_count * 3
}

/// Count resulting vertices from diamond subdivision.
#[allow(dead_code)]
pub fn diamond_vertex_count(original_vertex_count: usize, original_face_count: usize) -> usize {
    original_vertex_count + original_face_count
}

/// Perform multiple iterations of diamond subdivision.
#[allow(dead_code)]
pub fn diamond_subdivide_n(positions: &[[f32; 3]], indices: &[u32], iterations: usize) -> DiamondSubdivideResult {
    let mut result = DiamondSubdivideResult { positions: positions.to_vec(), indices: indices.to_vec() };
    for _ in 0..iterations {
        result = diamond_subdivide(&result.positions, &result.indices);
    }
    result
}

/// Serialize result to JSON.
#[allow(dead_code)]
pub fn diamond_to_json(result: &DiamondSubdivideResult) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{}}}",
        result.positions.len(),
        result.indices.len() / 3
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]], vec![0,1,2])
    }

    #[test]
    fn test_single_tri() {
        let (p, i) = tri();
        let r = diamond_subdivide(&p, &i);
        assert_eq!(r.positions.len(), 4);
        assert_eq!(r.indices.len(), 9);
    }

    #[test]
    fn test_face_count_formula() {
        assert_eq!(diamond_face_count(1), 3);
        assert_eq!(diamond_face_count(4), 12);
    }

    #[test]
    fn test_vertex_count_formula() {
        assert_eq!(diamond_vertex_count(3, 1), 4);
    }

    #[test]
    fn test_two_iterations() {
        let (p, i) = tri();
        let r = diamond_subdivide_n(&p, &i, 2);
        assert_eq!(r.indices.len() / 3, 9);
    }

    #[test]
    fn test_centroid_position() {
        let (p, i) = tri();
        let r = diamond_subdivide(&p, &i);
        let c = &r.positions[3];
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_preserves_original() {
        let (p, i) = tri();
        let r = diamond_subdivide(&p, &i);
        assert_eq!(r.positions[0], p[0]);
        assert_eq!(r.positions[1], p[1]);
    }

    #[test]
    fn test_empty() {
        let r = diamond_subdivide(&[], &[]);
        assert!(r.positions.is_empty());
        assert!(r.indices.is_empty());
    }

    #[test]
    fn test_to_json() {
        let (p, i) = tri();
        let r = diamond_subdivide(&p, &i);
        let json = diamond_to_json(&r);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_zero_iterations() {
        let (p, i) = tri();
        let r = diamond_subdivide_n(&p, &i, 0);
        assert_eq!(r.positions.len(), 3);
    }

    #[test]
    fn test_quad_mesh() {
        let pos = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0]];
        let idx = vec![0,1,2, 0,2,3];
        let r = diamond_subdivide(&pos, &idx);
        assert_eq!(r.indices.len() / 3, 6);
    }
}
