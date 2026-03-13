// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sew / merge mesh patches along shared boundary edges.

/// Config for patch sewing.
#[derive(Clone, Debug)]
pub struct PatchSewConfig {
    pub weld_threshold: f32,
}

impl Default for PatchSewConfig {
    fn default() -> Self {
        Self {
            weld_threshold: 1e-4,
        }
    }
}

/// Result of sewing two patches.
#[derive(Clone, Debug, Default)]
pub struct PatchSewResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub seam_vert_pairs: usize,
}

fn dist3_ps(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    d.iter().map(|v| v * v).sum::<f32>().sqrt()
}

/// Merge two patches into one and weld boundary vertices within `weld_threshold`.
pub fn sew_patches(
    pos_a: &[[f32; 3]],
    idx_a: &[u32],
    pos_b: &[[f32; 3]],
    idx_b: &[u32],
    config: &PatchSewConfig,
) -> PatchSewResult {
    // Concatenate positions
    let mut positions: Vec<[f32; 3]> = pos_a.to_vec();
    let offset = pos_a.len() as u32;
    positions.extend_from_slice(pos_b);

    // Offset patch-B indices
    let mut indices: Vec<u32> = idx_a.to_vec();
    for &i in idx_b {
        indices.push(i + offset);
    }

    // Build a remap table: for each vertex in B, find a matching vertex in A
    let mut remap: Vec<u32> = (0u32..positions.len() as u32).collect();
    let mut seam_vert_pairs = 0;

    for (bi, &pb) in pos_b.iter().enumerate() {
        let bi_global = bi as u32 + offset;
        for (ai, &pa) in pos_a.iter().enumerate() {
            if dist3_ps(pa, pb) <= config.weld_threshold {
                remap[bi_global as usize] = ai as u32;
                seam_vert_pairs += 1;
                break;
            }
        }
    }

    // Apply remap to indices
    for i in indices.iter_mut() {
        *i = remap[*i as usize];
    }

    // Remove degenerate triangles
    let indices: Vec<u32> = indices
        .chunks(3)
        .filter(|t| t.len() == 3 && t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
        .flatten()
        .copied()
        .collect();

    PatchSewResult {
        positions,
        indices,
        seam_vert_pairs,
    }
}

/// Return vertex count.
pub fn patch_sew_vertex_count(r: &PatchSewResult) -> usize {
    r.positions.len()
}

/// Return triangle count.
pub fn patch_sew_triangle_count(r: &PatchSewResult) -> usize {
    r.indices.len() / 3
}

/// Return number of welded seam vertex pairs.
pub fn patch_sew_seam_pairs(r: &PatchSewResult) -> usize {
    r.seam_vert_pairs
}

/// Check that all indices are valid for the merged position buffer.
pub fn patch_sew_indices_valid(r: &PatchSewResult) -> bool {
    let n = r.positions.len() as u32;
    r.indices.iter().all(|&i| i < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patch_a() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    fn patch_b_adjacent() -> (Vec<[f32; 3]>, Vec<u32>) {
        /* Shares edge (0,0,0)-(1,0,0) with patch A */
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, -1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn sew_combines_vertices() {
        let (pa, ia) = patch_a();
        let (pb, ib) = patch_b_adjacent();
        let cfg = PatchSewConfig::default();
        let r = sew_patches(&pa, &ia, &pb, &ib, &cfg);
        assert!(patch_sew_vertex_count(&r) > 0);
    }

    #[test]
    fn sew_welds_shared_edge() {
        let (pa, ia) = patch_a();
        let (pb, ib) = patch_b_adjacent();
        let cfg = PatchSewConfig::default();
        let r = sew_patches(&pa, &ia, &pb, &ib, &cfg);
        assert!(patch_sew_seam_pairs(&r) >= 2); // 2 shared verts
    }

    #[test]
    fn sew_indices_valid() {
        let (pa, ia) = patch_a();
        let (pb, ib) = patch_b_adjacent();
        let cfg = PatchSewConfig::default();
        let r = sew_patches(&pa, &ia, &pb, &ib, &cfg);
        assert!(patch_sew_indices_valid(&r));
    }

    #[test]
    fn sew_triangle_count_consistent() {
        let (pa, ia) = patch_a();
        let (pb, ib) = patch_b_adjacent();
        let cfg = PatchSewConfig::default();
        let r = sew_patches(&pa, &ia, &pb, &ib, &cfg);
        assert_eq!(patch_sew_triangle_count(&r) * 3, r.indices.len());
    }

    #[test]
    fn sew_no_weld_keeps_all_vertices() {
        let (pa, ia) = patch_a();
        let pb = vec![[10.0, 0.0, 0.0], [11.0, 0.0, 0.0], [10.5, 1.0, 0.0]];
        let ib = vec![0, 1, 2];
        let cfg = PatchSewConfig {
            weld_threshold: 0.0,
        };
        let r = sew_patches(&pa, &ia, &pb, &ib, &cfg);
        assert_eq!(patch_sew_vertex_count(&r), pa.len() + pb.len());
    }

    #[test]
    fn sew_positions_all_finite() {
        let (pa, ia) = patch_a();
        let (pb, ib) = patch_b_adjacent();
        let cfg = PatchSewConfig::default();
        let r = sew_patches(&pa, &ia, &pb, &ib, &cfg);
        for p in &r.positions {
            assert!(p.iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn sew_empty_patches() {
        let r = sew_patches(&[], &[], &[], &[], &PatchSewConfig::default());
        assert_eq!(patch_sew_vertex_count(&r), 0);
        assert_eq!(patch_sew_triangle_count(&r), 0);
    }

    #[test]
    fn config_default_positive_threshold() {
        let c = PatchSewConfig::default();
        assert!(c.weld_threshold > 0.0);
    }
}
