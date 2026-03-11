// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
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

/// Evaluate PSD given current pose (stub: uses nearest pose by Euclidean distance).
pub fn psd_evaluate<'a>(psd: &'a mut PoseSpaceDeform, current_pose: &[f32]) -> &'a [[f32; 3]] {
    for ex in &mut psd.examples {
        let n = ex.pose.len().min(current_pose.len());
        let dist: f32 = (0..n)
            .map(|i| (ex.pose[i] - current_pose[i]).powi(2))
            .sum::<f32>()
            .sqrt();
        ex.weight = if dist < 1e-6 { 1.0 } else { 1.0 / (1.0 + dist) };
    }
    /* Blend all examples by weight (stub: use highest-weight example's deltas) */
    if let Some(best) = psd.examples.iter().max_by(|a, b| {
        a.weight
            .partial_cmp(&b.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        let deltas = best.deltas.clone();
        let n = psd.current_deltas.len().min(deltas.len());
        psd.current_deltas[..n].copy_from_slice(&deltas[..n]);
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
}
