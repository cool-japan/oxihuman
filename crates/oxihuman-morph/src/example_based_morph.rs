// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Example-based deformation stub.

/// A single example pose containing vertex positions.
#[derive(Debug, Clone)]
pub struct ExamplePose {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub weight: f32,
}

/// Example-based morph system.
#[derive(Debug, Clone)]
pub struct ExampleBasedMorph {
    pub rest_pose: Vec<[f32; 3]>,
    pub examples: Vec<ExamplePose>,
    pub enabled: bool,
}

impl ExampleBasedMorph {
    pub fn new(rest_pose: Vec<[f32; 3]>) -> Self {
        ExampleBasedMorph {
            rest_pose,
            examples: Vec::new(),
            enabled: true,
        }
    }
}

/// Create a new example-based morph from a rest pose.
pub fn new_example_based_morph(rest_pose: Vec<[f32; 3]>) -> ExampleBasedMorph {
    ExampleBasedMorph::new(rest_pose)
}

/// Add an example pose.
pub fn ebm_add_example(morph: &mut ExampleBasedMorph, example: ExamplePose) {
    morph.examples.push(example);
}

/// Evaluate the blended result (stub: returns rest pose).
pub fn ebm_evaluate(morph: &ExampleBasedMorph, _weights: &[f32]) -> Vec<[f32; 3]> {
    /* Stub: returns rest pose unchanged */
    morph.rest_pose.clone()
}

/// Return the number of examples.
pub fn ebm_example_count(morph: &ExampleBasedMorph) -> usize {
    morph.examples.len()
}

/// Return the vertex count.
pub fn ebm_vertex_count(morph: &ExampleBasedMorph) -> usize {
    morph.rest_pose.len()
}

/// Enable or disable the morph system.
pub fn ebm_set_enabled(morph: &mut ExampleBasedMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn ebm_to_json(morph: &ExampleBasedMorph) -> String {
    format!(
        r#"{{"vertex_count":{},"example_count":{},"enabled":{}}}"#,
        morph.rest_pose.len(),
        morph.examples.len(),
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vertex_count() {
        let m = new_example_based_morph(vec![[0.0; 3]; 10]);
        assert_eq!(
            ebm_vertex_count(&m),
            10, /* vertex count must match rest pose */
        );
    }

    #[test]
    fn test_no_examples_initially() {
        let m = new_example_based_morph(vec![[0.0; 3]; 5]);
        assert_eq!(ebm_example_count(&m), 0 /* no examples initially */,);
    }

    #[test]
    fn test_add_example() {
        let mut m = new_example_based_morph(vec![[0.0; 3]; 4]);
        ebm_add_example(
            &mut m,
            ExamplePose {
                name: "smile".into(),
                vertices: vec![[0.1, 0.0, 0.0]; 4],
                weight: 1.0,
            },
        );
        assert_eq!(ebm_example_count(&m), 1 /* one example after add */,);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_example_based_morph(vec![[0.0; 3]; 6]);
        let out = ebm_evaluate(&m, &[]);
        assert_eq!(
            out.len(),
            6, /* output length must match vertex count */
        );
    }

    #[test]
    fn test_evaluate_returns_rest() {
        let rest = vec![[1.0, 2.0, 3.0]; 3];
        let m = new_example_based_morph(rest.clone());
        let out = ebm_evaluate(&m, &[]);
        assert!((out[0][0] - rest[0][0]).abs() < 1e-6, /* must return rest pose */);
    }

    #[test]
    fn test_set_enabled() {
        let mut m = new_example_based_morph(vec![]);
        ebm_set_enabled(&mut m, false);
        assert!(!m.enabled /* enabled must be false */,);
    }

    #[test]
    fn test_to_json() {
        let m = new_example_based_morph(vec![[0.0; 3]; 3]);
        let j = ebm_to_json(&m);
        assert!(j.contains("\"vertex_count\""), /* json must contain vertex_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_example_based_morph(vec![]);
        assert!(m.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_multiple_examples() {
        let mut m = new_example_based_morph(vec![[0.0; 3]; 2]);
        for i in 0..4 {
            ebm_add_example(
                &mut m,
                ExamplePose {
                    name: format!("pose_{i}"),
                    vertices: vec![[0.0; 3]; 2],
                    weight: 0.5,
                },
            );
        }
        assert_eq!(
            ebm_example_count(&m),
            4, /* four examples must be stored */
        );
    }

    #[test]
    fn test_json_contains_example_count() {
        let m = new_example_based_morph(vec![]);
        let j = ebm_to_json(&m);
        assert!(j.contains("\"example_count\""), /* json must contain example_count */);
    }
}
