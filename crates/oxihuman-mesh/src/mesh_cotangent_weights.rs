// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cotangent weight computation for Laplacian operators.

/// A single cotangent weight entry for the sparse Laplacian.
#[derive(Clone, Debug)]
pub struct CotWeight {
    pub row: usize,
    pub col: usize,
    pub weight: f32,
}

/// Result of cotangent weight computation.
#[derive(Clone, Debug, Default)]
pub struct CotWeightResult {
    pub weights: Vec<CotWeight>,
    pub vertex_areas: Vec<f32>,
}

/// Compute cot(angle at vertex `a` in triangle `a-b-c`).
pub fn cot_angle_at(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let dot: f32 = ab.iter().zip(ac.iter()).map(|(u, v)| u * v).sum();
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let sin_len: f32 = cross.iter().map(|v| v * v).sum::<f32>().sqrt();
    if sin_len < 1e-10 {
        0.0
    } else {
        dot / sin_len
    }
}

fn triangle_area_cot(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    0.5 * cross.iter().map(|v| v * v).sum::<f32>().sqrt()
}

/// Build cotangent Laplacian weights for a triangle mesh.
///
/// For each edge (i,j) shared by triangles, the weight is
/// w_ij = 0.5 * (cot α + cot β)
/// where α and β are the angles opposite to edge (i,j).
pub fn build_cotangent_weights(positions: &[[f32; 3]], indices: &[u32]) -> CotWeightResult {
    let n = positions.len();
    let tri_count = indices.len() / 3;
    let mut weight_map: std::collections::HashMap<(usize, usize), f32> =
        std::collections::HashMap::new();
    let mut vertex_areas = vec![0.0_f32; n];

    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];

        let cot_a = cot_angle_at(a, b, c).max(0.0);
        let cot_b = cot_angle_at(b, a, c).max(0.0);
        let cot_c = cot_angle_at(c, a, b).max(0.0);

        // Edge (ib, ic) uses cot_a
        let ea = (ib.min(ic), ib.max(ic));
        *weight_map.entry(ea).or_insert(0.0) += 0.5 * cot_a;

        // Edge (ia, ic) uses cot_b
        let eb = (ia.min(ic), ia.max(ic));
        *weight_map.entry(eb).or_insert(0.0) += 0.5 * cot_b;

        // Edge (ia, ib) uses cot_c
        let ec = (ia.min(ib), ia.max(ib));
        *weight_map.entry(ec).or_insert(0.0) += 0.5 * cot_c;

        // Mixed-area accumulation (Voronoi area approximation)
        let area = triangle_area_cot(a, b, c);
        vertex_areas[ia] += area / 3.0;
        vertex_areas[ib] += area / 3.0;
        vertex_areas[ic] += area / 3.0;
    }

    let weights: Vec<CotWeight> = weight_map
        .into_iter()
        .flat_map(|((i, j), w)| {
            vec![
                CotWeight {
                    row: i,
                    col: j,
                    weight: w,
                },
                CotWeight {
                    row: j,
                    col: i,
                    weight: w,
                },
            ]
        })
        .collect();

    CotWeightResult {
        weights,
        vertex_areas,
    }
}

/// Return the number of weight entries.
pub fn cot_weight_count(r: &CotWeightResult) -> usize {
    r.weights.len()
}

/// Return the total weight sum.
pub fn cot_total_weight(r: &CotWeightResult) -> f32 {
    r.weights.iter().map(|w| w.weight).sum()
}

/// Return the average mixed area.
pub fn cot_avg_vertex_area(r: &CotWeightResult) -> f32 {
    if r.vertex_areas.is_empty() {
        return 0.0;
    }
    r.vertex_areas.iter().sum::<f32>() / r.vertex_areas.len() as f32
}

/// Check that all weights are non-negative.
pub fn cot_weights_nonneg(r: &CotWeightResult) -> bool {
    r.weights.iter().all(|w| w.weight >= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equilateral_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        let s = 3.0_f32.sqrt() * 0.5;
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, s, 0.0]];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    fn two_tris() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn cot_angle_right_angle() {
        let cot = cot_angle_at([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(cot.abs() < 1e-5, "cot right angle should be ~0, got {cot}");
    }

    #[test]
    fn build_cot_weights_triangle_count() {
        let (pos, idx) = equilateral_tri();
        let r = build_cotangent_weights(&pos, &idx);
        /* Each edge symmetric → 2×3 = 6 entries for a single triangle */
        assert_eq!(cot_weight_count(&r), 6);
    }

    #[test]
    fn cot_weights_nonneg_check() {
        let (pos, idx) = two_tris();
        let r = build_cotangent_weights(&pos, &idx);
        assert!(cot_weights_nonneg(&r));
    }

    #[test]
    fn vertex_areas_positive() {
        let (pos, idx) = equilateral_tri();
        let r = build_cotangent_weights(&pos, &idx);
        for &a in &r.vertex_areas {
            assert!(a >= 0.0);
        }
    }

    #[test]
    fn cot_total_weight_positive() {
        let (pos, idx) = equilateral_tri();
        let r = build_cotangent_weights(&pos, &idx);
        assert!(cot_total_weight(&r) > 0.0);
    }

    #[test]
    fn cot_avg_vertex_area_positive() {
        let (pos, idx) = equilateral_tri();
        let r = build_cotangent_weights(&pos, &idx);
        assert!(cot_avg_vertex_area(&r) > 0.0);
    }

    #[test]
    fn empty_mesh_empty_result() {
        let r = build_cotangent_weights(&[], &[]);
        assert_eq!(cot_weight_count(&r), 0);
    }

    #[test]
    fn cot_weight_count_two_tris() {
        let (pos, idx) = two_tris();
        let r = build_cotangent_weights(&pos, &idx);
        /* Two tris share 1 edge, so 5 unique edges → 10 directed entries */
        assert_eq!(cot_weight_count(&r), 10);
    }
}
