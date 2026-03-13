// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Result of collapsing faces.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseFaceResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub collapsed_count: usize,
}

/// Collapse a face to its centroid, removing the face and merging vertices.
#[allow(dead_code)]
pub fn collapse_face(
    positions: &[[f32; 3]],
    indices: &[u32],
    face_index: usize,
) -> CollapseFaceResult {
    let mut new_positions = positions.to_vec();
    let mut new_indices: Vec<u32> = Vec::new();
    let tri_start = face_index * 3;
    if tri_start + 2 >= indices.len() {
        return CollapseFaceResult {
            positions: new_positions,
            indices: indices.to_vec(),
            collapsed_count: 0,
        };
    }
    let a = indices[tri_start] as usize;
    let b = indices[tri_start + 1] as usize;
    let c = indices[tri_start + 2] as usize;
    let centroid = [
        (new_positions[a][0] + new_positions[b][0] + new_positions[c][0]) / 3.0,
        (new_positions[a][1] + new_positions[b][1] + new_positions[c][1]) / 3.0,
        (new_positions[a][2] + new_positions[b][2] + new_positions[c][2]) / 3.0,
    ];
    new_positions[a] = centroid;
    for (i, tri) in indices.chunks_exact(3).enumerate() {
        if i == face_index {
            continue;
        }
        let mut t = [tri[0], tri[1], tri[2]];
        for v in &mut t {
            if *v == b as u32 || *v == c as u32 {
                *v = a as u32;
            }
        }
        if t[0] != t[1] && t[1] != t[2] && t[0] != t[2] {
            new_indices.extend_from_slice(&t);
        }
    }
    CollapseFaceResult { positions: new_positions, indices: new_indices, collapsed_count: 1 }
}

/// Compute face centroid.
#[allow(dead_code)]
pub fn face_centroid(positions: &[[f32; 3]], indices: &[u32], face_index: usize) -> [f32; 3] {
    let s = face_index * 3;
    let a = indices[s] as usize;
    let b = indices[s + 1] as usize;
    let c = indices[s + 2] as usize;
    [
        (positions[a][0] + positions[b][0] + positions[c][0]) / 3.0,
        (positions[a][1] + positions[b][1] + positions[c][1]) / 3.0,
        (positions[a][2] + positions[b][2] + positions[c][2]) / 3.0,
    ]
}

/// Check if a face is degenerate (zero area).
#[allow(dead_code)]
pub fn is_degenerate_face(positions: &[[f32; 3]], indices: &[u32], face_index: usize) -> bool {
    let s = face_index * 3;
    let a = indices[s] as usize;
    let b = indices[s + 1] as usize;
    let c = indices[s + 2] as usize;
    let ab = [positions[b][0] - positions[a][0], positions[b][1] - positions[a][1], positions[b][2] - positions[a][2]];
    let ac = [positions[c][0] - positions[a][0], positions[c][1] - positions[a][1], positions[c][2] - positions[a][2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let area_sq = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
    area_sq < 1e-12
}

/// Count total faces.
#[allow(dead_code)]
pub fn face_count(indices: &[u32]) -> usize {
    indices.len() / 3
}

/// Serialize collapse result to JSON.
#[allow(dead_code)]
pub fn collapse_to_json(result: &CollapseFaceResult) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{},\"collapsed\":{}}}",
        result.positions.len(),
        result.indices.len() / 3,
        result.collapsed_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0], [1.5, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        (pos, idx)
    }

    #[test]
    fn test_collapse_removes_face() {
        let (pos, idx) = two_tris();
        let result = collapse_face(&pos, &idx, 0);
        assert_eq!(result.collapsed_count, 1);
    }

    #[test]
    fn test_face_count() {
        let (_, idx) = two_tris();
        assert_eq!(face_count(&idx), 2);
    }

    #[test]
    fn test_face_centroid() {
        let (pos, idx) = two_tris();
        let c = face_centroid(&pos, &idx, 0);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_is_not_degenerate() {
        let (pos, idx) = two_tris();
        assert!(!is_degenerate_face(&pos, &idx, 0));
    }

    #[test]
    fn test_is_degenerate() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0, 1, 2];
        assert!(is_degenerate_face(&pos, &idx, 0));
    }

    #[test]
    fn test_out_of_bounds() {
        let (pos, idx) = two_tris();
        let result = collapse_face(&pos, &idx, 99);
        assert_eq!(result.collapsed_count, 0);
    }

    #[test]
    fn test_collapse_to_json() {
        let (pos, idx) = two_tris();
        let result = collapse_face(&pos, &idx, 0);
        let json = collapse_to_json(&result);
        assert!(json.contains("collapsed"));
    }

    #[test]
    fn test_single_face_collapse() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = collapse_face(&pos, &idx, 0);
        assert!(result.indices.is_empty());
    }

    #[test]
    fn test_collapse_preserves_vertex_count() {
        let (pos, idx) = two_tris();
        let result = collapse_face(&pos, &idx, 0);
        assert_eq!(result.positions.len(), pos.len());
    }

    #[test]
    fn test_centroid_values() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let idx = vec![0, 1, 2];
        let c = face_centroid(&pos, &idx, 0);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
    }
}
