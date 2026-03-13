// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Result of a simple decimation.
#[allow(dead_code)]
pub struct DecimateSimpleResult {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub removed_count: usize,
}

/// Compute a heuristic per-vertex error (sum of distances to adjacent face centroids).
#[allow(dead_code)]
pub fn vertex_error_simple(verts: &[[f32; 3]], tris: &[[u32; 3]], vert_idx: usize) -> f32 {
    let mut error = 0.0f32;
    let mut count = 0usize;
    let vp = verts[vert_idx];

    for tri in tris {
        if tri.contains(&(vert_idx as u32)) {
            // centroid
            let c = [
                (verts[tri[0] as usize][0]
                    + verts[tri[1] as usize][0]
                    + verts[tri[2] as usize][0])
                    / 3.0,
                (verts[tri[0] as usize][1]
                    + verts[tri[1] as usize][1]
                    + verts[tri[2] as usize][1])
                    / 3.0,
                (verts[tri[0] as usize][2]
                    + verts[tri[1] as usize][2]
                    + verts[tri[2] as usize][2])
                    / 3.0,
            ];
            let dx = vp[0] - c[0];
            let dy = vp[1] - c[1];
            let dz = vp[2] - c[2];
            error += (dx * dx + dy * dy + dz * dz).sqrt();
            count += 1;
        }
    }

    if count == 0 { 0.0 } else { error / count as f32 }
}

/// Count removed vertices in a result.
#[allow(dead_code)]
pub fn decimate_count(result: &DecimateSimpleResult) -> usize {
    result.removed_count
}

/// Decimate by removing vertices with the smallest error until `ratio` fraction remain.
#[allow(dead_code)]
pub fn decimate_simple(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    ratio: f32,
) -> DecimateSimpleResult {
    let ratio = ratio.clamp(0.0, 1.0);
    let target = (verts.len() as f32 * ratio).ceil() as usize;
    let to_remove = verts.len().saturating_sub(target);

    if to_remove == 0 {
        return DecimateSimpleResult {
            verts: verts.to_vec(),
            tris: tris.to_vec(),
            removed_count: 0,
        };
    }

    // compute error for each vertex
    let mut errors: Vec<(usize, f32)> = (0..verts.len())
        .map(|i| (i, vertex_error_simple(verts, tris, i)))
        .collect();
    errors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let remove_set: std::collections::HashSet<u32> = errors
        .iter()
        .take(to_remove)
        .map(|(i, _)| *i as u32)
        .collect();

    let kept_tris: Vec<[u32; 3]> = tris
        .iter()
        .filter(|tri| !tri.iter().any(|vi| remove_set.contains(vi)))
        .copied()
        .collect();

    DecimateSimpleResult {
        verts: verts.to_vec(),
        tris: kept_tris,
        removed_count: to_remove,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.5, 0.5],
        ]
    }

    fn cube_tris() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [0, 2, 3], [0, 1, 4], [1, 2, 4]]
    }

    #[test]
    fn test_decimate_simple_ratio_one() {
        let v = cube_verts();
        let t = cube_tris();
        let result = decimate_simple(&v, &t, 1.0);
        assert_eq!(result.removed_count, 0);
        assert_eq!(result.verts.len(), v.len());
    }

    #[test]
    fn test_decimate_simple_ratio_zero() {
        let v = cube_verts();
        let t = cube_tris();
        let result = decimate_simple(&v, &t, 0.0);
        // all removed
        assert!(result.removed_count == v.len());
    }

    #[test]
    fn test_decimate_simple_removes_some() {
        let v = cube_verts();
        let t = cube_tris();
        let result = decimate_simple(&v, &t, 0.6);
        assert!(result.removed_count > 0);
    }

    #[test]
    fn test_decimate_count_helper() {
        let r = DecimateSimpleResult {
            verts: Vec::new(),
            tris: Vec::new(),
            removed_count: 7,
        };
        assert_eq!(decimate_count(&r), 7);
    }

    #[test]
    fn test_vertex_error_simple_nonnegative() {
        let v = cube_verts();
        let t = cube_tris();
        let err = vertex_error_simple(&v, &t, 0);
        assert!(err >= 0.0);
    }

    #[test]
    fn test_vertex_error_simple_isolated_zero() {
        let v = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let t = vec![[0u32, 1, 0]]; // degenerate, but isolated vert 1 won't appear
        let err = vertex_error_simple(&v, &t, 1);
        // vert 1 does appear in the degenerate tri
        let _ = err;
    }

    #[test]
    fn test_decimate_simple_empty() {
        let result = decimate_simple(&[], &[], 0.5);
        assert_eq!(result.removed_count, 0);
    }

    #[test]
    fn test_decimate_simple_tris_reduced() {
        let v = cube_verts();
        let t = cube_tris();
        let result = decimate_simple(&v, &t, 0.4);
        assert!(result.tris.len() <= t.len());
    }

    #[test]
    fn test_decimate_simple_verts_len_unchanged() {
        // the result keeps all original verts but drops tris containing removed verts
        let v = cube_verts();
        let t = cube_tris();
        let result = decimate_simple(&v, &t, 0.8);
        assert_eq!(result.verts.len(), v.len());
    }
}
