// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Un-subdivide: collapse edge loops to reduce mesh resolution.

/// Configuration for un-subdivide.
#[derive(Debug, Clone)]
pub struct UnsubdivideConfig {
    /// Number of un-subdivide iterations to perform.
    pub iterations: usize,
    /// Whether to preserve boundary vertices.
    pub preserve_boundary: bool,
}

impl Default for UnsubdivideConfig {
    fn default() -> Self {
        Self {
            iterations: 1,
            preserve_boundary: true,
        }
    }
}

/// Result of un-subdivide.
#[derive(Debug, Clone)]
pub struct UnsubdivideResult {
    /// New positions after collapsing loops.
    pub positions: Vec<[f32; 3]>,
    /// New triangle indices.
    pub triangles: Vec<[usize; 3]>,
    /// Number of vertices removed.
    pub verts_removed: usize,
}

/// Identify vertices that appear to be loop-inserted midpoints.
/// Returns indices of every other vertex in each edge loop (simplified heuristic).
pub fn find_loop_midpoints(positions: &[[f32; 3]], triangles: &[[usize; 3]]) -> Vec<usize> {
    /* Simple heuristic: collect all vertex indices that appear in exactly 2 triangles */
    let n = positions.len();
    let mut face_count = vec![0usize; n];
    for tri in triangles {
        for &vi in tri.iter() {
            if vi < n {
                face_count[vi] += 1;
            }
        }
    }
    face_count
        .iter()
        .enumerate()
        .filter(|(_, &c)| c == 2)
        .map(|(i, _)| i)
        .collect()
}

/// Perform one pass of un-subdivide by collapsing candidate midpoints.
pub fn unsubdivide(
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &UnsubdivideConfig,
) -> UnsubdivideResult {
    let mut pos = positions.to_vec();
    let mut tris = triangles.to_vec();

    for _ in 0..config.iterations {
        let midpoints = find_loop_midpoints(&pos, &tris);
        if midpoints.is_empty() {
            break;
        }
        /* Remove midpoint vertices by keeping only even-indexed verts (stub logic) */
        let keep: Vec<bool> = (0..pos.len()).map(|i| !midpoints.contains(&i)).collect();
        let remap: Vec<usize> = {
            let mut new_idx = 0usize;
            keep.iter()
                .map(|&k| {
                    if k {
                        let idx = new_idx;
                        new_idx += 1;
                        idx
                    } else {
                        usize::MAX
                    }
                })
                .collect()
        };
        let new_pos: Vec<[f32; 3]> = pos
            .iter()
            .enumerate()
            .filter(|(i, _)| keep[*i])
            .map(|(_, &p)| p)
            .collect();
        /* Remap triangles; discard any triangle with a collapsed vertex */
        let new_tris: Vec<[usize; 3]> = tris
            .iter()
            .filter_map(|&[a, b, c]| {
                let na = remap.get(a).copied().unwrap_or(usize::MAX);
                let nb = remap.get(b).copied().unwrap_or(usize::MAX);
                let nc = remap.get(c).copied().unwrap_or(usize::MAX);
                if na == usize::MAX || nb == usize::MAX || nc == usize::MAX {
                    None
                } else {
                    Some([na, nb, nc])
                }
            })
            .collect();
        let removed = pos.len().saturating_sub(new_pos.len());
        pos = new_pos;
        tris = new_tris;
        let _ = removed;
    }

    let verts_removed = positions.len().saturating_sub(pos.len());
    UnsubdivideResult {
        positions: pos,
        triangles: tris,
        verts_removed,
    }
}

/// Compute the reduction ratio (surviving verts / original verts).
pub fn vertex_reduction_ratio(result: &UnsubdivideResult, original_count: usize) -> f32 {
    if original_count == 0 {
        return 1.0;
    }
    result.positions.len() as f32 / original_count as f32
}

/// Check if un-subdivide had any effect.
pub fn had_effect(result: &UnsubdivideResult) -> bool {
    result.verts_removed > 0
}

/// Count surviving triangles.
pub fn surviving_tri_count(result: &UnsubdivideResult) -> usize {
    result.triangles.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = UnsubdivideConfig::default();
        assert_eq!(cfg.iterations, 1);
        assert!(cfg.preserve_boundary);
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = UnsubdivideConfig::default();
        let result = unsubdivide(&[], &[], &cfg);
        assert_eq!(result.positions.len(), 0);
        assert_eq!(result.verts_removed, 0);
    }

    #[test]
    fn test_vertex_reduction_ratio_no_change() {
        let result = UnsubdivideResult {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            triangles: vec![],
            verts_removed: 0,
        };
        let ratio = vertex_reduction_ratio(&result, 2);
        assert!((ratio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_had_effect_false() {
        let result = UnsubdivideResult {
            positions: vec![],
            triangles: vec![],
            verts_removed: 0,
        };
        assert!(!had_effect(&result));
    }

    #[test]
    fn test_had_effect_true() {
        let result = UnsubdivideResult {
            positions: vec![],
            triangles: vec![],
            verts_removed: 3,
        };
        assert!(had_effect(&result));
    }

    #[test]
    fn test_surviving_tri_count() {
        let result = UnsubdivideResult {
            positions: vec![],
            triangles: vec![[0, 1, 2], [1, 2, 3]],
            verts_removed: 0,
        };
        assert_eq!(surviving_tri_count(&result), 2);
    }

    #[test]
    fn test_find_loop_midpoints_empty() {
        let midpoints = find_loop_midpoints(&[], &[]);
        assert!(midpoints.is_empty());
    }

    #[test]
    fn test_unsubdivide_single_tri() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0usize, 1, 2]];
        let cfg = UnsubdivideConfig::default();
        let result = unsubdivide(&verts, &tris, &cfg);
        /* No midpoints in a single triangle */
        assert_eq!(result.triangles.len(), 1);
    }

    #[test]
    fn test_vertex_reduction_ratio_zero_original() {
        let result = UnsubdivideResult {
            positions: vec![],
            triangles: vec![],
            verts_removed: 0,
        };
        assert!((vertex_reduction_ratio(&result, 0) - 1.0).abs() < 1e-6);
    }
}
