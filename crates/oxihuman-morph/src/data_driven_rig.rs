// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Data-driven rigging system stub.

/// A sample of captured rig state used for data-driven training.
#[derive(Debug, Clone)]
pub struct RigSample {
    pub pose_params: Vec<f32>,
    pub shape_output: Vec<f32>,
}

/// Data-driven rig that maps pose parameters to shape deltas.
#[derive(Debug, Clone)]
pub struct DataDrivenRig {
    pub samples: Vec<RigSample>,
    pub param_dim: usize,
    pub shape_dim: usize,
    pub enabled: bool,
}

impl DataDrivenRig {
    pub fn new(param_dim: usize, shape_dim: usize) -> Self {
        DataDrivenRig {
            samples: Vec::new(),
            param_dim,
            shape_dim,
            enabled: true,
        }
    }
}

/// Create a new data-driven rig.
pub fn new_data_driven_rig(param_dim: usize, shape_dim: usize) -> DataDrivenRig {
    DataDrivenRig::new(param_dim, shape_dim)
}

/// Add a training sample to the rig.
pub fn ddr_add_sample(rig: &mut DataDrivenRig, sample: RigSample) {
    rig.samples.push(sample);
}

/// Compute the Euclidean distance between two pose parameter slices.
///
/// Handles mismatched lengths by treating missing elements as 0.
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().max(b.len());
    let mut sum_sq = 0.0_f32;
    for i in 0..len {
        let ai = if i < a.len() { a[i] } else { 0.0 };
        let bi = if i < b.len() { b[i] } else { 0.0 };
        let diff = ai - bi;
        sum_sq += diff * diff;
    }
    sum_sq.sqrt()
}

/// Evaluate the rig for a pose via inverse-distance-weighted (IDW) regression over samples.
///
/// For each sample `s`:
/// - `dist = euclidean_distance(pose, &s.pose_params)`
/// - If `dist < 1e-9` (exact match), immediately return a clone of that sample's output.
/// - Otherwise weight = `1.0 / dist`.
///
/// Final output = `Σ(w * s.shape_output) / Σw`, clamped to `rig.shape_dim`.
/// If there are no samples, returns a zero vector.
pub fn ddr_evaluate(rig: &DataDrivenRig, pose: &[f32]) -> Vec<f32> {
    if rig.samples.is_empty() {
        return vec![0.0; rig.shape_dim];
    }

    let mut weight_sum = 0.0_f32;
    let mut accum = vec![0.0_f32; rig.shape_dim];

    for sample in &rig.samples {
        let dist = euclidean_distance(pose, &sample.pose_params);
        if dist < 1e-9 {
            // Exact match — return immediately, clamped to shape_dim.
            let mut out = sample.shape_output.clone();
            out.truncate(rig.shape_dim);
            out.resize(rig.shape_dim, 0.0);
            return out;
        }
        let w = 1.0 / dist;
        weight_sum += w;
        let contribution_len = sample.shape_output.len().min(rig.shape_dim);
        for (a, &s) in accum[..contribution_len]
            .iter_mut()
            .zip(&sample.shape_output[..contribution_len])
        {
            *a += w * s;
        }
    }

    if weight_sum == 0.0 {
        return vec![0.0; rig.shape_dim];
    }

    for v in accum.iter_mut() {
        *v /= weight_sum;
    }
    accum
}

/// Return sample count.
pub fn ddr_sample_count(rig: &DataDrivenRig) -> usize {
    rig.samples.len()
}

/// Enable or disable the rig.
pub fn ddr_set_enabled(rig: &mut DataDrivenRig, enabled: bool) {
    rig.enabled = enabled;
}

/// Clear all training samples.
pub fn ddr_clear_samples(rig: &mut DataDrivenRig) {
    rig.samples.clear();
}

/// Serialize to JSON-like string.
pub fn ddr_to_json(rig: &DataDrivenRig) -> String {
    format!(
        r#"{{"param_dim":{},"shape_dim":{},"samples":{},"enabled":{}}}"#,
        rig.param_dim,
        rig.shape_dim,
        rig.samples.len(),
        rig.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dims() {
        let rig = new_data_driven_rig(10, 20);
        assert_eq!(rig.param_dim, 10 /* param_dim must match */,);
        assert_eq!(rig.shape_dim, 20 /* shape_dim must match */,);
    }

    #[test]
    fn test_no_samples_initially() {
        let rig = new_data_driven_rig(5, 5);
        assert_eq!(
            ddr_sample_count(&rig),
            0, /* must have no samples initially */
        );
    }

    #[test]
    fn test_add_sample() {
        let mut rig = new_data_driven_rig(3, 3);
        ddr_add_sample(
            &mut rig,
            RigSample {
                pose_params: vec![0.1, 0.2, 0.3],
                shape_output: vec![0.0, 0.0, 0.0],
            },
        );
        assert_eq!(ddr_sample_count(&rig), 1 /* one sample after add */,);
    }

    #[test]
    fn test_evaluate_length() {
        let rig = new_data_driven_rig(4, 6);
        let out = ddr_evaluate(&rig, &[0.0; 4]);
        assert_eq!(out.len(), 6 /* output length must match shape_dim */,);
    }

    #[test]
    fn test_evaluate_zeroed() {
        let rig = new_data_driven_rig(2, 4);
        let out = ddr_evaluate(&rig, &[1.0, 0.5]);
        assert!(out.iter().all(|&v| v.abs() < 1e-6), /* stub must return zeros */);
    }

    #[test]
    fn test_set_enabled() {
        let mut rig = new_data_driven_rig(2, 2);
        ddr_set_enabled(&mut rig, false);
        assert!(!rig.enabled /* enabled must be false */,);
    }

    #[test]
    fn test_clear_samples() {
        let mut rig = new_data_driven_rig(2, 2);
        ddr_add_sample(
            &mut rig,
            RigSample {
                pose_params: vec![0.0; 2],
                shape_output: vec![0.0; 2],
            },
        );
        ddr_clear_samples(&mut rig);
        assert_eq!(ddr_sample_count(&rig), 0 /* samples must be cleared */,);
    }

    #[test]
    fn test_to_json() {
        let rig = new_data_driven_rig(4, 8);
        let j = ddr_to_json(&rig);
        assert!(j.contains("\"param_dim\""), /* json must contain param_dim */);
    }

    #[test]
    fn test_enabled_by_default() {
        let rig = new_data_driven_rig(1, 1);
        assert!(rig.enabled /* enabled by default */,);
    }

    #[test]
    fn test_many_samples() {
        let mut rig = new_data_driven_rig(2, 2);
        for _ in 0..10 {
            ddr_add_sample(
                &mut rig,
                RigSample {
                    pose_params: vec![0.0; 2],
                    shape_output: vec![0.0; 2],
                },
            );
        }
        assert_eq!(
            ddr_sample_count(&rig),
            10, /* ten samples must be stored */
        );
    }

    #[test]
    fn ddr_exact_pose_match_returns_sample() {
        // Add a single sample at pose [1.0, 2.0] with shape_output [0.7, 0.3].
        // Evaluating at the exact same pose must return [0.7, 0.3].
        let mut rig = new_data_driven_rig(2, 2);
        ddr_add_sample(
            &mut rig,
            RigSample {
                pose_params: vec![1.0, 2.0],
                shape_output: vec![0.7, 0.3],
            },
        );
        let out = ddr_evaluate(&rig, &[1.0, 2.0]);
        assert!((out[0] - 0.7).abs() < 1e-5, "expected 0.7 got {}", out[0]);
        assert!((out[1] - 0.3).abs() < 1e-5, "expected 0.3 got {}", out[1]);
    }

    #[test]
    fn ddr_two_samples_midpoint() {
        // Two samples: A at pose [0.0] shape [1.0], B at pose [2.0] shape [3.0].
        // Query at pose [1.0]: dist_A = 1, dist_B = 1 → equal weights → blend = (1+3)/2 = 2.0.
        let mut rig = new_data_driven_rig(1, 1);
        ddr_add_sample(
            &mut rig,
            RigSample {
                pose_params: vec![0.0],
                shape_output: vec![1.0],
            },
        );
        ddr_add_sample(
            &mut rig,
            RigSample {
                pose_params: vec![2.0],
                shape_output: vec![3.0],
            },
        );
        let out = ddr_evaluate(&rig, &[1.0]);
        assert!(
            (out[0] - 2.0).abs() < 1e-5,
            "expected 2.0 at midpoint, got {}",
            out[0]
        );
    }
}
