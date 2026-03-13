// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mean curvature flow smoothing.

/// Configuration for mean curvature flow.
#[derive(Clone, Debug)]
pub struct McfConfig {
    pub dt: f32,
    pub iterations: usize,
    pub preserve_volume: bool,
}

impl Default for McfConfig {
    fn default() -> Self {
        Self {
            dt: 0.01,
            iterations: 5,
            preserve_volume: false,
        }
    }
}

/// Result of a mean curvature flow run.
#[derive(Clone, Debug, Default)]
pub struct McfResult {
    pub positions: Vec<[f32; 3]>,
    pub iterations_done: usize,
    pub avg_displacement: f32,
}

fn cotangent(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    /* cot(angle at vertex a in triangle a-b-c) */
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

fn vec_len(v: [f32; 3]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Run one step of mean curvature flow.
pub fn mcf_step(positions: &[[f32; 3]], indices: &[u32], dt: f32) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut laplacian = vec![[0.0_f32; 3]; n];
    let mut weight_sum = vec![0.0_f32; n];

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];
        // Cotangent weights
        let w_c = cotangent(c, a, b).max(0.0); // opposite to edge a-b
        let w_b = cotangent(b, a, c).max(0.0); // opposite to edge a-c
        let w_a = cotangent(a, b, c).max(0.0); // opposite to edge b-c

        for k in 0..3 {
            laplacian[ia][k] += w_c * (b[k] - a[k]) + w_b * (c[k] - a[k]);
            laplacian[ib][k] += w_c * (a[k] - b[k]) + w_a * (c[k] - b[k]);
            laplacian[ic][k] += w_b * (a[k] - c[k]) + w_a * (b[k] - c[k]);
        }
        weight_sum[ia] += w_c + w_b;
        weight_sum[ib] += w_c + w_a;
        weight_sum[ic] += w_b + w_a;
    }

    let mut result = positions.to_vec();
    for i in 0..n {
        let w = weight_sum[i];
        if w > 1e-10 {
            for k in 0..3 {
                result[i][k] += dt * laplacian[i][k] / w;
            }
        }
    }
    result
}

/// Run mean curvature flow for `iterations` steps.
pub fn mean_curvature_flow(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &McfConfig,
) -> McfResult {
    let mut pos = positions.to_vec();
    for iter in 0..config.iterations {
        let next = mcf_step(&pos, indices, config.dt);
        pos = next;
        if iter == config.iterations.saturating_sub(1) {}
    }
    let avg_displacement = if positions.is_empty() {
        0.0
    } else {
        let sum: f32 = positions
            .iter()
            .zip(pos.iter())
            .map(|(a, b)| vec_len([b[0] - a[0], b[1] - a[1], b[2] - a[2]]))
            .sum();
        sum / positions.len() as f32
    };
    McfResult {
        positions: pos,
        iterations_done: config.iterations,
        avg_displacement,
    }
}

/// Return the number of vertices in an McfResult.
pub fn mcf_vertex_count(r: &McfResult) -> usize {
    r.positions.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
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
    fn mcf_config_default_reasonable() {
        let c = McfConfig::default();
        assert!(c.dt > 0.0);
        assert!(c.iterations > 0);
    }

    #[test]
    fn mcf_step_preserves_vertex_count() {
        let (pos, idx) = square_mesh();
        let next = mcf_step(&pos, &idx, 0.01);
        assert_eq!(next.len(), pos.len());
    }

    #[test]
    fn mcf_step_positions_finite() {
        let (pos, idx) = square_mesh();
        let next = mcf_step(&pos, &idx, 0.01);
        for p in &next {
            assert!(p.iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn mean_curvature_flow_result_count() {
        let (pos, idx) = square_mesh();
        let cfg = McfConfig {
            iterations: 3,
            ..Default::default()
        };
        let r = mean_curvature_flow(&pos, &idx, &cfg);
        assert_eq!(r.positions.len(), pos.len());
        assert_eq!(r.iterations_done, 3);
    }

    #[test]
    fn mean_curvature_flow_avg_displacement_nonneg() {
        let (pos, idx) = square_mesh();
        let cfg = McfConfig::default();
        let r = mean_curvature_flow(&pos, &idx, &cfg);
        assert!(r.avg_displacement >= 0.0);
    }

    #[test]
    fn mcf_vertex_count_matches() {
        let (pos, idx) = square_mesh();
        let cfg = McfConfig::default();
        let r = mean_curvature_flow(&pos, &idx, &cfg);
        assert_eq!(mcf_vertex_count(&r), pos.len());
    }

    #[test]
    fn mcf_zero_iterations() {
        let (pos, idx) = square_mesh();
        let cfg = McfConfig {
            iterations: 0,
            ..Default::default()
        };
        let r = mean_curvature_flow(&pos, &idx, &cfg);
        assert_eq!(r.positions.len(), pos.len());
    }

    #[test]
    fn cotangent_right_angle() {
        /* cot(90°) should be ~0 */
        let cot = cotangent([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(cot.abs() < 1e-5, "cot={cot}");
    }
}
