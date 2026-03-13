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

/// Evaluate the rig for a pose (stub: zeroed output).
pub fn ddr_evaluate(rig: &DataDrivenRig, _pose: &[f32]) -> Vec<f32> {
    /* Stub: returns zeroed shape output */
    vec![0.0; rig.shape_dim]
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
}
