// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Result of bridging two edge loops together.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BridgeEdgeResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub bridge_face_count: usize,
}

/// Bridge two edge sequences by creating quad faces between them.
#[allow(dead_code)]
pub fn bridge_edges(
    positions: &[[f32; 3]],
    loop_a: &[u32],
    loop_b: &[u32],
) -> BridgeEdgeResult {
    let new_positions = positions.to_vec();
    let mut new_indices = Vec::new();
    let count = loop_a.len().min(loop_b.len());
    let mut bridge_face_count = 0;
    for i in 0..count {
        let next = (i + 1) % count;
        let a0 = loop_a[i];
        let a1 = loop_a[next];
        let b0 = loop_b[i];
        let b1 = loop_b[next];
        new_indices.extend_from_slice(&[a0, b0, b1]);
        new_indices.extend_from_slice(&[a0, b1, a1]);
        bridge_face_count += 2;
    }
    BridgeEdgeResult { new_positions, new_indices, bridge_face_count }
}

/// Compute the centroid of a set of vertex indices.
#[allow(dead_code)]
pub fn edge_loop_centroid(positions: &[[f32; 3]], loop_indices: &[u32]) -> [f32; 3] {
    let mut c = [0.0f32; 3];
    if loop_indices.is_empty() {
        return c;
    }
    for &idx in loop_indices {
        let p = &positions[idx as usize];
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let n = loop_indices.len() as f32;
    c[0] /= n;
    c[1] /= n;
    c[2] /= n;
    c
}

/// Distance between two loop centroids.
#[allow(dead_code)]
pub fn loop_distance(positions: &[[f32; 3]], loop_a: &[u32], loop_b: &[u32]) -> f32 {
    let ca = edge_loop_centroid(positions, loop_a);
    let cb = edge_loop_centroid(positions, loop_b);
    let dx = ca[0] - cb[0];
    let dy = ca[1] - cb[1];
    let dz = ca[2] - cb[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Validate that loop indices are within bounds.
#[allow(dead_code)]
pub fn validate_loop(positions: &[[f32; 3]], loop_indices: &[u32]) -> bool {
    let n = positions.len() as u32;
    loop_indices.iter().all(|&i| i < n)
}

/// Serialize bridge result to JSON string.
#[allow(dead_code)]
pub fn bridge_to_json(result: &BridgeEdgeResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"index_count\":{},\"bridge_faces\":{}}}",
        result.new_positions.len(),
        result.new_indices.len(),
        result.bridge_face_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0], [1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
            [0.0, 1.0, 1.0], [1.0, 1.0, 1.0],
        ]
    }

    #[test]
    fn test_bridge_basic() {
        let pos = square_positions();
        let la = vec![0, 1, 3, 2];
        let lb = vec![4, 5, 7, 6];
        let result = bridge_edges(&pos, &la, &lb);
        assert_eq!(result.bridge_face_count, 8);
        assert!(!result.new_indices.is_empty());
    }

    #[test]
    fn test_bridge_preserves_positions() {
        let pos = square_positions();
        let result = bridge_edges(&pos, &[0, 1], &[4, 5]);
        assert_eq!(result.new_positions.len(), pos.len());
    }

    #[test]
    fn test_centroid() {
        let pos = square_positions();
        let c = edge_loop_centroid(&pos, &[0, 1]);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_loop_distance() {
        let pos = square_positions();
        let d = loop_distance(&pos, &[0, 1], &[4, 5]);
        assert!(d > 0.0);
    }

    #[test]
    fn test_validate_loop_ok() {
        let pos = square_positions();
        assert!(validate_loop(&pos, &[0, 1, 2, 3]));
    }

    #[test]
    fn test_validate_loop_fail() {
        let pos = square_positions();
        assert!(!validate_loop(&pos, &[0, 1, 99]));
    }

    #[test]
    fn test_empty_loops() {
        let pos = square_positions();
        let result = bridge_edges(&pos, &[], &[]);
        assert_eq!(result.bridge_face_count, 0);
    }

    #[test]
    fn test_bridge_to_json() {
        let pos = square_positions();
        let result = bridge_edges(&pos, &[0, 1], &[4, 5]);
        let json = bridge_to_json(&result);
        assert!(json.contains("bridge_faces"));
    }

    #[test]
    fn test_centroid_empty() {
        let pos = square_positions();
        let c = edge_loop_centroid(&pos, &[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_mismatched_lengths() {
        let pos = square_positions();
        let result = bridge_edges(&pos, &[0, 1, 2], &[4, 5]);
        assert_eq!(result.bridge_face_count, 4);
    }
}
