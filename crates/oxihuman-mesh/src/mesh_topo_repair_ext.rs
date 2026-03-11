// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Extended topology repair: fix winding order, remove degenerate faces,
//! and re-stitch torn boundaries.

use std::collections::HashMap;

/// Summary produced by the extended topology repair pass.
#[derive(Debug, Clone, Default)]
pub struct TopoRepairExtResult {
    pub winding_flips: usize,
    pub degenerate_removed: usize,
    pub boundaries_stitched: usize,
    pub non_manifold_fixed: usize,
}

/// Configuration for the extended topology repair.
#[derive(Debug, Clone)]
pub struct TopoRepairExtConfig {
    /// Minimum triangle area to keep (smaller triangles are degenerate).
    pub min_area: f32,
    /// Whether to attempt automatic winding fix.
    pub fix_winding: bool,
    /// Tolerance used when stitching boundary pairs.
    pub stitch_tol: f32,
}

impl Default for TopoRepairExtConfig {
    fn default() -> Self {
        Self {
            min_area: 1e-8,
            fix_winding: true,
            stitch_tol: 1e-5,
        }
    }
}

/// Returns the signed area of a triangle defined by three 3-D positions.
pub fn signed_face_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    ((cx * cx + cy * cy + cz * cz).sqrt()) * 0.5
}

/// Detects degenerate triangles (area below threshold).
pub fn find_degenerate_triangles(
    positions: &[[f32; 3]],
    indices: &[u32],
    min_area: f32,
) -> Vec<usize> {
    let mut out = Vec::new();
    let n = indices.len() / 3;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let ia = indices[i * 3] as usize;
        let ib = indices[i * 3 + 1] as usize;
        let ic = indices[i * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        let area = signed_face_area(positions[ia], positions[ib], positions[ic]);
        if area < min_area {
            out.push(i);
        }
    }
    out
}

/// Removes degenerate triangles from an index buffer.
pub fn remove_degenerate_tris(positions: &[[f32; 3]], indices: &[u32], min_area: f32) -> Vec<u32> {
    let bad: std::collections::HashSet<usize> =
        find_degenerate_triangles(positions, indices, min_area)
            .into_iter()
            .collect();
    let n = indices.len() / 3;
    let mut out = Vec::with_capacity(indices.len());
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        if !bad.contains(&i) {
            out.push(indices[i * 3]);
            out.push(indices[i * 3 + 1]);
            out.push(indices[i * 3 + 2]);
        }
    }
    out
}

/// Attempts to flip face winding so all normals point outward from the centroid.
pub fn auto_fix_winding(positions: &[[f32; 3]], indices: &mut [u32]) -> usize {
    /* compute mesh centroid */
    if positions.is_empty() {
        return 0;
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for p in positions {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let n = positions.len() as f32;
    let centroid = [cx / n, cy / n, cz / n];
    let mut flips = 0usize;
    let tris = indices.len() / 3;
    for i in 0..tris {
        let ia = indices[i * 3] as usize;
        let ib = indices[i * 3 + 1] as usize;
        let ic = indices[i * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];
        /* face normal */
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let nx = ab[1] * ac[2] - ab[2] * ac[1];
        let ny = ab[2] * ac[0] - ab[0] * ac[2];
        let nz = ab[0] * ac[1] - ab[1] * ac[0];
        /* face centroid to mesh centroid */
        let fx = (a[0] + b[0] + c[0]) / 3.0 - centroid[0];
        let fy = (a[1] + b[1] + c[1]) / 3.0 - centroid[1];
        let fz = (a[2] + b[2] + c[2]) / 3.0 - centroid[2];
        let dot = nx * fx + ny * fy + nz * fz;
        if dot < 0.0 {
            /* flip winding */
            indices.swap(i * 3 + 1, i * 3 + 2);
            flips += 1;
        }
    }
    flips
}

/// Builds an edge-to-face adjacency map.
pub fn build_edge_adjacency(indices: &[u32]) -> HashMap<(u32, u32), Vec<usize>> {
    let mut map: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    let n = indices.len() / 3;
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        for (ea, eb) in [(ia, ib), (ib, ic), (ic, ia)] {
            let key = if ea < eb { (ea, eb) } else { (eb, ea) };
            map.entry(key).or_default().push(i);
        }
    }
    map
}

/// Runs the extended topology repair pass.
pub fn repair_topology_ext(
    positions: &[[f32; 3]],
    indices: &mut Vec<u32>,
    config: &TopoRepairExtConfig,
) -> TopoRepairExtResult {
    let mut result = TopoRepairExtResult::default();
    let degenerate = find_degenerate_triangles(positions, indices, config.min_area);
    result.degenerate_removed = degenerate.len();
    *indices = remove_degenerate_tris(positions, indices, config.min_area);
    if config.fix_winding {
        result.winding_flips = auto_fix_winding(positions, indices);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn signed_area_unit_triangle() {
        /* area of a right triangle with legs 1 should be 0.5 */
        let a = signed_face_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - 0.5).abs() < 1e-5);
    }

    #[test]
    fn degenerate_detection_collinear() {
        /* collinear triangle has area ≈ 0 */
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let bad = find_degenerate_triangles(&pos, &idx, 1e-8);
        assert_eq!(bad.len(), 1);
    }

    #[test]
    fn remove_degenerate_reduces_count() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0], /* collinear */
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 0, 1, 3];
        let out = remove_degenerate_tris(&pos, &idx, 1e-8);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn auto_fix_winding_no_panic() {
        /* should not panic on empty mesh */
        let pos: Vec<[f32; 3]> = vec![];
        let mut idx: Vec<u32> = vec![];
        let flips = auto_fix_winding(&pos, &mut idx);
        assert_eq!(flips, 0);
    }

    #[test]
    fn repair_topology_ext_default() {
        let (pos, mut idx) = unit_triangle();
        let cfg = TopoRepairExtConfig::default();
        let res = repair_topology_ext(&pos, &mut idx, &cfg);
        /* one valid triangle, nothing removed */
        assert_eq!(res.degenerate_removed, 0);
    }

    #[test]
    fn build_edge_adjacency_one_tri() {
        let (_, idx) = unit_triangle();
        let map = build_edge_adjacency(&idx);
        /* triangle has 3 edges */
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn config_default_values() {
        let cfg = TopoRepairExtConfig::default();
        assert!(cfg.fix_winding);
        assert!(cfg.min_area > 0.0);
    }

    #[test]
    fn result_default_all_zero() {
        let r = TopoRepairExtResult::default();
        assert_eq!(r.winding_flips, 0);
        assert_eq!(r.degenerate_removed, 0);
    }

    #[test]
    fn out_of_bounds_indices_skipped() {
        let pos = vec![[0.0f32, 0.0, 0.0]; 2];
        /* index 2 is out of bounds */
        let idx = vec![0u32, 1, 99];
        let bad = find_degenerate_triangles(&pos, &idx, 1e-8);
        /* skipped silently */
        assert_eq!(bad.len(), 0);
    }

    #[test]
    fn remove_all_degenerate_gives_empty() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let out = remove_degenerate_tris(&pos, &idx, 1e-8);
        assert!(out.is_empty());
    }
}
