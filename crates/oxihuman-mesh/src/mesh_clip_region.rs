// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Clip a mesh to an axis-aligned region (AABB).

/// Axis-aligned clip region.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ClipRegion {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Result of clipping a mesh to a region.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClipRegionResult {
    /// Positions inside the region.
    pub inside_positions: Vec<[f32; 3]>,
    /// Triangles where ALL three vertices are inside the region.
    pub inside_indices: Vec<u32>,
    /// Number of rejected triangles.
    pub rejected_count: usize,
}

/// Check if a position is inside a clip region.
#[allow(dead_code)]
pub fn position_in_region(p: [f32; 3], region: ClipRegion) -> bool {
    (0..3).all(|k| p[k] >= region.min[k] && p[k] <= region.max[k])
}

/// Clip a mesh to an AABB: only keep triangles fully inside.
#[allow(dead_code)]
pub fn clip_to_region(
    positions: &[[f32; 3]],
    indices: &[u32],
    region: ClipRegion,
) -> ClipRegionResult {
    let in_flags: Vec<bool> = positions
        .iter()
        .map(|&p| position_in_region(p, region))
        .collect();
    let tri_count = indices.len() / 3;
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_indices: Vec<u32> = Vec::new();
    let mut remap: Vec<Option<u32>> = vec![None; positions.len()];
    let mut rejected = 0usize;

    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        if i0 < in_flags.len()
            && i1 < in_flags.len()
            && i2 < in_flags.len()
            && in_flags[i0]
            && in_flags[i1]
            && in_flags[i2]
        {
            for &vi in &[i0, i1, i2] {
                if remap[vi].is_none() {
                    remap[vi] = Some(new_positions.len() as u32);
                    new_positions.push(positions[vi]);
                }
                new_indices.push(remap[vi].unwrap_or(0));
            }
        } else {
            rejected += 1;
        }
    }
    ClipRegionResult {
        inside_positions: new_positions,
        inside_indices: new_indices,
        rejected_count: rejected,
    }
}

/// Count accepted triangles.
#[allow(dead_code)]
pub fn accepted_triangle_count(result: &ClipRegionResult) -> usize {
    result.inside_indices.len() / 3
}

/// Validate indices are in bounds.
#[allow(dead_code)]
pub fn clip_indices_valid(result: &ClipRegionResult) -> bool {
    let n = result.inside_positions.len() as u32;
    result.inside_indices.iter().all(|&i| i < n)
}

/// Serialize result to JSON.
#[allow(dead_code)]
pub fn clip_region_to_json(result: &ClipRegionResult) -> String {
    format!(
        "{{\"accepted_triangles\":{},\"rejected\":{}}}",
        accepted_triangle_count(result),
        result.rejected_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_region() -> ClipRegion {
        ClipRegion {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        }
    }

    fn tri_inside() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.1, 0.1, 0.1], [0.5, 0.1, 0.1], [0.3, 0.5, 0.1]];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    fn tri_outside() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn test_position_in_region_inside() {
        assert!(position_in_region([0.5, 0.5, 0.5], unit_region()));
    }

    #[test]
    fn test_position_in_region_outside() {
        assert!(!position_in_region([2.0, 0.5, 0.5], unit_region()));
    }

    #[test]
    fn test_clip_accepts_inside_tri() {
        let (pos, idx) = tri_inside();
        let r = clip_to_region(&pos, &idx, unit_region());
        assert_eq!(accepted_triangle_count(&r), 1);
    }

    #[test]
    fn test_clip_rejects_outside_tri() {
        let (pos, idx) = tri_outside();
        let r = clip_to_region(&pos, &idx, unit_region());
        assert_eq!(accepted_triangle_count(&r), 0);
        assert_eq!(r.rejected_count, 1);
    }

    #[test]
    fn test_clip_indices_valid() {
        let (pos, idx) = tri_inside();
        let r = clip_to_region(&pos, &idx, unit_region());
        assert!(clip_indices_valid(&r));
    }

    #[test]
    fn test_clip_empty_mesh() {
        let r = clip_to_region(&[], &[], unit_region());
        assert_eq!(accepted_triangle_count(&r), 0);
    }

    #[test]
    fn test_clip_region_to_json() {
        let (pos, idx) = tri_inside();
        let r = clip_to_region(&pos, &idx, unit_region());
        let j = clip_region_to_json(&r);
        assert!(j.contains("accepted_triangles"));
    }

    #[test]
    fn test_partial_clip() {
        let (mut pos, mut idx) = tri_inside();
        let (pos2, idx2) = tri_outside();
        let offset = pos.len() as u32;
        pos.extend_from_slice(&pos2);
        idx.extend(idx2.iter().map(|&i| i + offset));
        let r = clip_to_region(&pos, &idx, unit_region());
        assert_eq!(accepted_triangle_count(&r), 1);
        assert_eq!(r.rejected_count, 1);
    }

    #[test]
    fn test_clip_region_on_boundary() {
        let region = ClipRegion {
            min: [0.0; 3],
            max: [0.0; 3],
        };
        let pos = vec![[0.0, 0.0, 0.0]; 3];
        let idx = vec![0u32, 1, 2];
        let r = clip_to_region(&pos, &idx, region);
        assert_eq!(accepted_triangle_count(&r), 1);
    }
}
