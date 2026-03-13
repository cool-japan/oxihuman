// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Result of dissolving a vertex.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DissolveVertexResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub dissolved: bool,
}

/// Dissolve a vertex by removing it and re-triangulating the surrounding fan.
#[allow(dead_code)]
pub fn dissolve_vertex(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex: u32,
) -> DissolveVertexResult {
    let mut ring: Vec<u32> = Vec::new();
    let mut keep_faces: Vec<[u32; 3]> = Vec::new();
    for tri in indices.chunks_exact(3) {
        let t = [tri[0], tri[1], tri[2]];
        if t.contains(&vertex) {
            for &v in &t {
                if v != vertex && !ring.contains(&v) {
                    ring.push(v);
                }
            }
        } else {
            keep_faces.push(t);
        }
    }
    if ring.len() < 3 {
        return DissolveVertexResult {
            positions: positions.to_vec(),
            indices: indices.to_vec(),
            dissolved: false,
        };
    }
    let mut new_indices: Vec<u32> = Vec::new();
    for face in &keep_faces {
        new_indices.extend_from_slice(face);
    }
    let pivot = ring[0];
    for i in 1..ring.len() - 1 {
        new_indices.extend_from_slice(&[pivot, ring[i], ring[i + 1]]);
    }
    DissolveVertexResult {
        positions: positions.to_vec(),
        indices: new_indices,
        dissolved: true,
    }
}

/// Check if a vertex can be dissolved (has enough neighbors).
#[allow(dead_code)]
pub fn can_dissolve(indices: &[u32], vertex: u32) -> bool {
    let mut ring: Vec<u32> = Vec::new();
    for tri in indices.chunks_exact(3) {
        let t = [tri[0], tri[1], tri[2]];
        if t.contains(&vertex) {
            for &v in &t {
                if v != vertex && !ring.contains(&v) {
                    ring.push(v);
                }
            }
        }
    }
    ring.len() >= 3
}

/// Count faces adjacent to a vertex.
#[allow(dead_code)]
pub fn adjacent_face_count(indices: &[u32], vertex: u32) -> usize {
    indices
        .chunks_exact(3)
        .filter(|tri| tri.contains(&vertex))
        .count()
}

/// Serialize dissolve result to JSON.
#[allow(dead_code)]
pub fn dissolve_to_json(result: &DissolveVertexResult) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{},\"dissolved\":{}}}",
        result.positions.len(),
        result.indices.len() / 3,
        result.dissolved
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fan_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [-0.5, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3, 0, 3, 4];
        (pos, idx)
    }

    #[test]
    fn test_dissolve_center() {
        let (pos, idx) = fan_mesh();
        let result = dissolve_vertex(&pos, &idx, 0);
        assert!(result.dissolved);
    }

    #[test]
    fn test_dissolve_reduces_faces() {
        let (pos, idx) = fan_mesh();
        let result = dissolve_vertex(&pos, &idx, 0);
        assert!(result.indices.len() / 3 < idx.len() / 3);
    }

    #[test]
    fn test_can_dissolve() {
        let (_, idx) = fan_mesh();
        assert!(can_dissolve(&idx, 0));
    }

    #[test]
    fn test_cannot_dissolve_leaf() {
        let idx = vec![0, 1, 2];
        assert!(!can_dissolve(&idx, 0));
    }

    #[test]
    fn test_adjacent_face_count() {
        let (_, idx) = fan_mesh();
        assert_eq!(adjacent_face_count(&idx, 0), 3);
    }

    #[test]
    fn test_dissolve_preserves_positions() {
        let (pos, idx) = fan_mesh();
        let result = dissolve_vertex(&pos, &idx, 0);
        assert_eq!(result.positions.len(), pos.len());
    }

    #[test]
    fn test_dissolve_to_json() {
        let (pos, idx) = fan_mesh();
        let result = dissolve_vertex(&pos, &idx, 0);
        let json = dissolve_to_json(&result);
        assert!(json.contains("dissolved"));
    }

    #[test]
    fn test_dissolve_non_center() {
        let (pos, idx) = fan_mesh();
        let result = dissolve_vertex(&pos, &idx, 1);
        assert!(!result.dissolved);
    }

    #[test]
    fn test_empty() {
        let result = dissolve_vertex(&[], &[], 0);
        assert!(!result.dissolved);
    }

    #[test]
    fn test_adjacent_face_count_isolated() {
        assert_eq!(adjacent_face_count(&[], 0), 0);
    }
}
