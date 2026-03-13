// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Result of a planar cut operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlanarCutResult {
    pub above_positions: Vec<[f32; 3]>,
    pub above_indices: Vec<u32>,
    pub below_positions: Vec<[f32; 3]>,
    pub below_indices: Vec<u32>,
}

/// Classify a point relative to a plane (normal dot product + offset).
#[allow(dead_code)]
pub fn classify_point(point: &[f32; 3], plane_normal: &[f32; 3], plane_d: f32) -> f32 {
    point[0] * plane_normal[0] + point[1] * plane_normal[1] + point[2] * plane_normal[2] + plane_d
}

/// Split mesh by a plane defined by normal and distance.
#[allow(dead_code)]
pub fn planar_cut(
    positions: &[[f32; 3]],
    indices: &[u32],
    plane_normal: [f32; 3],
    plane_d: f32,
) -> PlanarCutResult {
    let mut above_pos = Vec::new();
    let mut above_idx = Vec::new();
    let mut below_pos = Vec::new();
    let mut below_idx = Vec::new();
    let mut above_map = std::collections::HashMap::new();
    let mut below_map = std::collections::HashMap::new();
    for tri in indices.chunks_exact(3) {
        let dists: Vec<f32> = tri.iter().map(|&v| classify_point(&positions[v as usize], &plane_normal, plane_d)).collect();
        let all_above = dists.iter().all(|&d| d >= 0.0);
        let all_below = dists.iter().all(|&d| d < 0.0);
        if all_above {
            let mut mapped = [0u32; 3];
            for (i, &v) in tri.iter().enumerate() {
                let new_v = *above_map.entry(v).or_insert_with(|| {
                    let idx = above_pos.len() as u32;
                    above_pos.push(positions[v as usize]);
                    idx
                });
                mapped[i] = new_v;
            }
            above_idx.extend_from_slice(&mapped);
        } else if all_below {
            let mut mapped = [0u32; 3];
            for (i, &v) in tri.iter().enumerate() {
                let new_v = *below_map.entry(v).or_insert_with(|| {
                    let idx = below_pos.len() as u32;
                    below_pos.push(positions[v as usize]);
                    idx
                });
                mapped[i] = new_v;
            }
            below_idx.extend_from_slice(&mapped);
        }
        // Straddling triangles are discarded for simplicity
    }
    PlanarCutResult {
        above_positions: above_pos, above_indices: above_idx,
        below_positions: below_pos, below_indices: below_idx,
    }
}

/// Count vertices above a plane.
#[allow(dead_code)]
pub fn count_above(positions: &[[f32; 3]], plane_normal: [f32; 3], plane_d: f32) -> usize {
    positions.iter().filter(|p| classify_point(p, &plane_normal, plane_d) >= 0.0).count()
}

/// Count vertices below a plane.
#[allow(dead_code)]
pub fn count_below(positions: &[[f32; 3]], plane_normal: [f32; 3], plane_d: f32) -> usize {
    positions.iter().filter(|p| classify_point(p, &plane_normal, plane_d) < 0.0).count()
}

/// Serialize planar cut result to JSON.
#[allow(dead_code)]
pub fn planar_cut_to_json(result: &PlanarCutResult) -> String {
    format!(
        "{{\"above_verts\":{},\"above_faces\":{},\"below_verts\":{},\"below_faces\":{}}}",
        result.above_positions.len(), result.above_indices.len() / 3,
        result.below_positions.len(), result.below_indices.len() / 3,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.5, 2.0, 0.0],
            [0.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.5, -2.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 3, 4, 5];
        (pos, idx)
    }

    #[test]
    fn test_classify_above() {
        let d = classify_point(&[0.0, 1.0, 0.0], &[0.0, 1.0, 0.0], 0.0);
        assert!(d > 0.0);
    }

    #[test]
    fn test_classify_below() {
        let d = classify_point(&[0.0, -1.0, 0.0], &[0.0, 1.0, 0.0], 0.0);
        assert!(d < 0.0);
    }

    #[test]
    fn test_planar_cut_separates() {
        let (pos, idx) = simple_mesh();
        let result = planar_cut(&pos, &idx, [0.0, 1.0, 0.0], 0.0);
        assert!(!result.above_indices.is_empty());
        assert!(!result.below_indices.is_empty());
    }

    #[test]
    fn test_count_above() {
        let (pos, _) = simple_mesh();
        assert_eq!(count_above(&pos, [0.0, 1.0, 0.0], 0.0), 3);
    }

    #[test]
    fn test_count_below() {
        let (pos, _) = simple_mesh();
        assert_eq!(count_below(&pos, [0.0, 1.0, 0.0], 0.0), 3);
    }

    #[test]
    fn test_all_above() {
        let pos = vec![[0.0, 1.0, 0.0], [1.0, 2.0, 0.0], [0.5, 3.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = planar_cut(&pos, &idx, [0.0, 1.0, 0.0], 0.0);
        assert!(!result.above_indices.is_empty());
        assert!(result.below_indices.is_empty());
    }

    #[test]
    fn test_all_below() {
        let pos = vec![[0.0, -1.0, 0.0], [1.0, -2.0, 0.0], [0.5, -3.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = planar_cut(&pos, &idx, [0.0, 1.0, 0.0], 0.0);
        assert!(result.above_indices.is_empty());
        assert!(!result.below_indices.is_empty());
    }

    #[test]
    fn test_empty_mesh() {
        let result = planar_cut(&[], &[], [0.0, 1.0, 0.0], 0.0);
        assert!(result.above_indices.is_empty());
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = simple_mesh();
        let result = planar_cut(&pos, &idx, [0.0, 1.0, 0.0], 0.0);
        let json = planar_cut_to_json(&result);
        assert!(json.contains("above_verts"));
    }

    #[test]
    fn test_on_plane() {
        let d = classify_point(&[0.0, 0.0, 0.0], &[0.0, 1.0, 0.0], 0.0);
        assert!((d).abs() < 1e-6);
    }
}
