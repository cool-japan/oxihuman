// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pose space deformation (PSD) stub.

/// A single PSD example pose.
#[derive(Debug, Clone)]
pub struct PsdExample {
    pub pose: Vec<f32>,
    pub deltas: Vec<[f32; 3]>,
    pub weight: f32,
}

/// Pose space deformer.
#[derive(Debug, Clone)]
pub struct PoseSpaceDeform {
    pub examples: Vec<PsdExample>,
    pub current_deltas: Vec<[f32; 3]>,
}

impl PoseSpaceDeform {
    pub fn new(vertex_count: usize) -> Self {
        PoseSpaceDeform {
            examples: Vec::new(),
            current_deltas: vec![[0.0; 3]; vertex_count],
        }
    }
}

/// Create a new PSD deformer.
pub fn new_psd(vertex_count: usize) -> PoseSpaceDeform {
    PoseSpaceDeform::new(vertex_count)
}

/// Add a PSD example.
pub fn psd_add_example(psd: &mut PoseSpaceDeform, pose: Vec<f32>, deltas: Vec<[f32; 3]>) {
    psd.examples.push(PsdExample {
        pose,
        deltas,
        weight: 0.0,
    });
}

/// Return example count.
pub fn psd_example_count(psd: &PoseSpaceDeform) -> usize {
    psd.examples.len()
}

/// Evaluate PSD given current pose via normalized weighted blending of all example deltas.
///
/// Weight per example: `w_e = 1 / (1 + dist(pose_e, current_pose))`.
/// Exact match (dist < 1e-6): returns that example's deltas directly (no blending).
///
/// Blend formula: `Σ w_e * deltas_e[v] / Σ w_e`
pub fn psd_evaluate<'a>(psd: &'a mut PoseSpaceDeform, current_pose: &[f32]) -> &'a [[f32; 3]] {
    // Compute per-example weights and check for an exact match.
    for ex in &mut psd.examples {
        let n = ex.pose.len().min(current_pose.len());
        let dist: f32 = (0..n)
            .map(|i| (ex.pose[i] - current_pose[i]).powi(2))
            .sum::<f32>()
            .sqrt();
        ex.weight = if dist < 1e-6 {
            f32::INFINITY
        } else {
            1.0 / (1.0 + dist)
        };
    }

    // Short-circuit: if any example is an exact match, use its deltas directly.
    let exact_idx = psd.examples.iter().position(|ex| ex.weight.is_infinite());

    if let Some(idx) = exact_idx {
        let deltas = psd.examples[idx].deltas.clone();
        let nv = psd.current_deltas.len().min(deltas.len());
        for d in &mut psd.current_deltas {
            *d = [0.0; 3];
        }
        psd.current_deltas[..nv].copy_from_slice(&deltas[..nv]);
        return &psd.current_deltas;
    }

    // Normalized weighted blend: Σ w_e * delta_e[v] / Σ w_e
    let weight_sum: f32 = psd.examples.iter().map(|ex| ex.weight).sum();

    // Zero out current_deltas before accumulation.
    for d in &mut psd.current_deltas {
        *d = [0.0; 3];
    }

    if weight_sum < 1e-12 {
        return &psd.current_deltas;
    }

    let nv = psd.current_deltas.len();
    for ex in psd.examples.iter() {
        let w = ex.weight / weight_sum;
        let copy_len = nv.min(ex.deltas.len());
        for v in 0..copy_len {
            psd.current_deltas[v][0] += w * ex.deltas[v][0];
            psd.current_deltas[v][1] += w * ex.deltas[v][1];
            psd.current_deltas[v][2] += w * ex.deltas[v][2];
        }
    }

    &psd.current_deltas
}

/// Reset all current deltas to zero.
pub fn psd_reset(psd: &mut PoseSpaceDeform) {
    for d in &mut psd.current_deltas {
        *d = [0.0; 3];
    }
}

/// Return a JSON-like string.
pub fn psd_to_json(psd: &PoseSpaceDeform) -> String {
    format!(
        r#"{{"examples":{},"vertices":{}}}"#,
        psd.examples.len(),
        psd.current_deltas.len()
    )
}

/// Return vertex count.
pub fn psd_vertex_count(psd: &PoseSpaceDeform) -> usize {
    psd.current_deltas.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_psd_vertex_count() {
        let p = new_psd(12);
        assert_eq!(psd_vertex_count(&p), 12 /* vertex count must match */,);
    }

    #[test]
    fn test_initial_no_examples() {
        let p = new_psd(5);
        assert_eq!(
            psd_example_count(&p),
            0, /* should start with no examples */
        );
    }

    #[test]
    fn test_add_example_increases_count() {
        let mut p = new_psd(5);
        psd_add_example(&mut p, vec![0.0; 4], vec![[0.0; 3]; 5]);
        assert_eq!(psd_example_count(&p), 1 /* count should increase */,);
    }

    #[test]
    fn test_evaluate_exact_pose_sets_deltas() {
        let mut p = new_psd(3);
        psd_add_example(&mut p, vec![1.0, 0.0], vec![[0.5, 0.0, 0.0]; 3]);
        psd_evaluate(&mut p, &[1.0, 0.0]);
        assert!(p.current_deltas[0][0] > 0.0, /* deltas should be set for exact pose */);
    }

    #[test]
    fn test_reset_zeroes_deltas() {
        let mut p = new_psd(3);
        psd_add_example(&mut p, vec![0.0; 2], vec![[1.0; 3]; 3]);
        psd_evaluate(&mut p, &[0.0; 2]);
        psd_reset(&mut p);
        for d in &p.current_deltas {
            assert!((d[0]).abs() < 1e-6 /* reset should zero deltas */,);
        }
    }

    #[test]
    fn test_to_json_contains_examples() {
        let p = new_psd(4);
        let j = psd_to_json(&p);
        assert!(j.contains("examples") /* JSON must contain examples */,);
    }

    #[test]
    fn test_to_json_contains_vertices() {
        let p = new_psd(7);
        let j = psd_to_json(&p);
        assert!(j.contains("7") /* JSON should contain vertex count */,);
    }

    #[test]
    fn test_initial_deltas_zero() {
        let p = new_psd(6);
        for d in &p.current_deltas {
            assert!((d[0]).abs() < 1e-6 /* initial deltas should be 0 */,);
        }
    }

    #[test]
    fn test_multiple_examples() {
        let mut p = new_psd(2);
        psd_add_example(&mut p, vec![0.0], vec![[0.0; 3]; 2]);
        psd_add_example(&mut p, vec![1.0], vec![[1.0; 3]; 2]);
        assert_eq!(
            psd_example_count(&p),
            2, /* two examples should be stored */
        );
    }

    #[test]
    fn test_example_weights_initially_zero() {
        let mut p = new_psd(2);
        psd_add_example(&mut p, vec![0.0], vec![[0.0; 3]; 2]);
        assert!((p.examples[0].weight).abs() < 1e-6, /* initial example weight is 0 */);
    }

    #[test]
    fn test_evaluate_no_examples_keeps_zero() {
        let mut p = new_psd(3);
        psd_evaluate(&mut p, &[0.5]);
        for d in &p.current_deltas {
            assert!((d[0]).abs() < 1e-6 /* no examples means zero deltas */,);
        }
    }

    #[test]
    fn test_blend_two_equidistant_examples_is_average() {
        // Two examples equidistant from current_pose = [0.5].
        // Example A: pose=[0.0], deltas=[[2.0, 0.0, 0.0]; 2]
        // Example B: pose=[1.0], deltas=[[0.0, 0.0, 0.0]; 2]
        // dist_A = 0.5, dist_B = 0.5 → w_A = 1/1.5, w_B = 1/1.5 (equal).
        // Normalized: w_A = w_B = 0.5 → output = average = [[1.0, 0.0, 0.0]; 2]
        let mut p = new_psd(2);
        psd_add_example(&mut p, vec![0.0], vec![[2.0f32, 0.0, 0.0]; 2]);
        psd_add_example(&mut p, vec![1.0], vec![[0.0f32, 0.0, 0.0]; 2]);
        psd_evaluate(&mut p, &[0.5]);
        // Because weights are equal the result must be the average: [1.0, 0.0, 0.0].
        let expected_x = 1.0f32;
        for (i, d) in p.current_deltas.iter().enumerate() {
            assert!(
                (d[0] - expected_x).abs() < 1e-4,
                "vertex[{i}][0]: expected {expected_x}, got {}",
                d[0]
            );
            assert!(
                (d[1]).abs() < 1e-6,
                "vertex[{i}][1]: expected 0, got {}",
                d[1]
            );
        }
    }

    #[test]
    fn test_exact_pose_match_returns_that_example_directly() {
        // Exact match: current_pose == example A's pose → A's deltas returned verbatim.
        let mut p = new_psd(2);
        psd_add_example(&mut p, vec![1.0, 0.0], vec![[5.0f32, 6.0, 7.0]; 2]);
        psd_add_example(&mut p, vec![0.0, 1.0], vec![[0.0f32; 3]; 2]);
        psd_evaluate(&mut p, &[1.0, 0.0]);
        assert!(
            (p.current_deltas[0][0] - 5.0).abs() < 1e-5,
            "exact match must copy A deltas"
        );
        assert!(
            (p.current_deltas[0][1] - 6.0).abs() < 1e-5,
            "exact match must copy A deltas"
        );
    }

    #[test]
    fn test_weighted_blend_asymmetric() {
        // Pose A at distance 1.0 from query → w_A = 1/2.0 = 0.5
        // Pose B at distance 4.0 from query → w_B = 1/5.0 = 0.2
        // w_sum = 0.7, normalized: w_A = 0.5/0.7 ≈ 0.714, w_B = 0.2/0.7 ≈ 0.286
        // A delta = [1.0, 0, 0], B delta = [0.0, 0, 0]
        // Expected output[0] ≈ [0.714, 0, 0]
        let mut p = new_psd(1);
        psd_add_example(&mut p, vec![0.0f32], vec![[1.0f32, 0.0, 0.0]]);
        psd_add_example(&mut p, vec![0.0f32], vec![[0.0f32, 0.0, 0.0]]);
        // Override weights manually for deterministic distance:
        // distance from current_pose=[1.0] to A=[0.0] = 1.0 → w_A = 0.5
        // distance from current_pose=[1.0] to B=[5.0] = 4.0 → w_B = 0.2
        // Re-add with controlled poses.
        let mut p2 = new_psd(1);
        psd_add_example(&mut p2, vec![0.0f32], vec![[1.0f32, 0.0, 0.0]]);
        psd_add_example(&mut p2, vec![5.0f32], vec![[0.0f32, 0.0, 0.0]]);
        psd_evaluate(&mut p2, &[1.0]);
        let expected = 0.5f32 / (0.5 + 0.2); // ≈ 0.7143
        assert!(
            (p2.current_deltas[0][0] - expected).abs() < 1e-3,
            "asymmetric blend: expected ≈{expected:.4}, got {}",
            p2.current_deltas[0][0]
        );
    }
}
