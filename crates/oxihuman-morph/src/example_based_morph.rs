// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
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

/// Evaluate the blended result using blend-shape / corrective-shape blending:
///
/// `output[v] = rest_pose[v] + Σ_e _weights[e] * (examples[e].vertices[v] - rest_pose[v])`
///
/// - Indices beyond the shorter of `_weights` / `examples` are silently ignored.
/// - Vertex components are clamped to the length of `rest_pose`.
/// - Mismatched example vertex lengths are handled safely (missing components treated as zero).
pub fn ebm_evaluate(morph: &ExampleBasedMorph, _weights: &[f32]) -> Vec<[f32; 3]> {
    let nv = morph.rest_pose.len();
    let mut out: Vec<[f32; 3]> = morph.rest_pose.clone();

    let n_examples = morph.examples.len().min(_weights.len());
    for (ex, &w) in morph.examples[..n_examples]
        .iter()
        .zip(&_weights[..n_examples])
    {
        if w.abs() < 1e-12 {
            continue;
        }
        for (i, out_v) in out[..nv].iter_mut().enumerate() {
            let ex_pos = ex.vertices.get(i).copied().unwrap_or([0.0f32; 3]);
            let rest = morph.rest_pose[i];
            out_v[0] += w * (ex_pos[0] - rest[0]);
            out_v[1] += w * (ex_pos[1] - rest[1]);
            out_v[2] += w * (ex_pos[2] - rest[2]);
        }
    }

    out
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

    #[test]
    fn test_blend_shape_half_weight() {
        // rest_pose = [[0,0,0]; 3], one example = [[1,1,1]; 3], weight = 0.5
        // expected output = [[0.5, 0.5, 0.5]; 3]
        let rest = vec![[0.0f32; 3]; 3];
        let mut m = new_example_based_morph(rest);
        ebm_add_example(
            &mut m,
            ExamplePose {
                name: "target".into(),
                vertices: vec![[1.0f32; 3]; 3],
                weight: 0.5,
            },
        );
        let out = ebm_evaluate(&m, &[0.5]);
        assert_eq!(out.len(), 3, "output vertex count must match rest pose");
        for (i, v) in out.iter().enumerate() {
            assert!(
                (v[0] - 0.5).abs() < 1e-5,
                "vertex[{i}][0] must be 0.5, got {}",
                v[0]
            );
            assert!(
                (v[1] - 0.5).abs() < 1e-5,
                "vertex[{i}][1] must be 0.5, got {}",
                v[1]
            );
            assert!(
                (v[2] - 0.5).abs() < 1e-5,
                "vertex[{i}][2] must be 0.5, got {}",
                v[2]
            );
        }
    }

    #[test]
    fn test_blend_shape_full_weight_reaches_example() {
        // weight = 1.0 must return exactly the example positions when rest = zero.
        let rest = vec![[0.0f32; 3]; 2];
        let mut m = new_example_based_morph(rest);
        ebm_add_example(
            &mut m,
            ExamplePose {
                name: "full".into(),
                vertices: vec![[3.0, 4.0, 5.0], [6.0, 7.0, 8.0]],
                weight: 1.0,
            },
        );
        let out = ebm_evaluate(&m, &[1.0]);
        assert!(
            (out[0][0] - 3.0).abs() < 1e-5,
            "full-weight must reach example position"
        );
        assert!(
            (out[1][2] - 8.0).abs() < 1e-5,
            "full-weight must reach example position"
        );
    }

    #[test]
    fn test_blend_shape_no_weights_returns_rest() {
        // Passing an empty weight slice must return the rest pose unchanged.
        let rest = vec![[1.0, 2.0, 3.0]; 4];
        let mut m = new_example_based_morph(rest.clone());
        ebm_add_example(
            &mut m,
            ExamplePose {
                name: "ignored".into(),
                vertices: vec![[99.0f32; 3]; 4],
                weight: 1.0,
            },
        );
        let out = ebm_evaluate(&m, &[]);
        for (i, v) in out.iter().enumerate() {
            assert!(
                (v[0] - 1.0).abs() < 1e-5,
                "vertex[{i}] must equal rest when no weights provided"
            );
        }
    }
}
