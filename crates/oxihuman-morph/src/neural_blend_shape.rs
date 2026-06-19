// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Neural network driven blend shape stub.

/// Activation function types for neural blend shape inference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NbsActivation {
    Relu,
    Tanh,
    Sigmoid,
}

/// A neural-network-driven blend shape evaluator stub.
#[derive(Debug, Clone)]
pub struct NeuralBlendShape {
    pub input_dim: usize,
    pub output_dim: usize,
    pub activation: NbsActivation,
    pub weights: Vec<f32>,
    pub bias: Vec<f32>,
    pub enabled: bool,
}

impl NeuralBlendShape {
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        NeuralBlendShape {
            input_dim,
            output_dim,
            activation: NbsActivation::Relu,
            weights: vec![0.0; input_dim * output_dim],
            bias: vec![0.0; output_dim],
            enabled: true,
        }
    }
}

/// Create a new neural blend shape evaluator.
pub fn new_neural_blend_shape(input_dim: usize, output_dim: usize) -> NeuralBlendShape {
    NeuralBlendShape::new(input_dim, output_dim)
}

/// Run a forward pass through the single-layer MLP.
///
/// Computes `out[j] = activation(Σ_i input[i] * weights[i * output_dim + j] + bias[j])`.
/// If the evaluator is disabled or the input is empty, returns a zero vector.
/// If `input.len() != nbs.input_dim`, the input is zero-padded or clamped silently.
pub fn nbs_forward(nbs: &NeuralBlendShape, input: &[f32]) -> Vec<f32> {
    if !nbs.enabled || nbs.output_dim == 0 {
        return vec![0.0; nbs.output_dim];
    }

    // Build a working input slice of exactly input_dim length (zero-padded / clamped).
    let effective_len = input.len().min(nbs.input_dim);
    // Accumulate output
    let mut out = nbs.bias.clone();
    // out[j] = bias[j] + Σ_i input[i] * weights[i * output_dim + j]
    for (i, &x) in input[..effective_len].iter().enumerate() {
        if x == 0.0 {
            continue;
        }
        let row_offset = i * nbs.output_dim;
        for (j, o) in out.iter_mut().enumerate() {
            *o += x * nbs.weights[row_offset + j];
        }
    }

    // Apply activation in-place.
    match nbs.activation {
        NbsActivation::Relu => {
            for v in out.iter_mut() {
                *v = v.max(0.0);
            }
        }
        NbsActivation::Tanh => {
            for v in out.iter_mut() {
                *v = v.tanh();
            }
        }
        NbsActivation::Sigmoid => {
            for v in out.iter_mut() {
                *v = 1.0 / (1.0 + (-*v).exp());
            }
        }
    }

    out
}

/// Set the activation function.
pub fn nbs_set_activation(nbs: &mut NeuralBlendShape, activation: NbsActivation) {
    nbs.activation = activation;
}

/// Enable or disable the evaluator.
pub fn nbs_set_enabled(nbs: &mut NeuralBlendShape, enabled: bool) {
    nbs.enabled = enabled;
}

/// Load weights from a flat slice (stub: copies up to available length).
pub fn nbs_load_weights(nbs: &mut NeuralBlendShape, weights: &[f32]) {
    let n = weights.len().min(nbs.weights.len());
    nbs.weights[..n].copy_from_slice(&weights[..n]);
}

/// Serialize to JSON-like string.
pub fn nbs_to_json(nbs: &NeuralBlendShape) -> String {
    format!(
        r#"{{"input_dim":{},"output_dim":{},"enabled":{}}}"#,
        nbs.input_dim, nbs.output_dim, nbs.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dims() {
        let nbs = new_neural_blend_shape(8, 4);
        assert_eq!(nbs.input_dim, 8 /* input dim must match */,);
        assert_eq!(nbs.output_dim, 4 /* output dim must match */,);
    }

    #[test]
    fn test_default_enabled() {
        let nbs = new_neural_blend_shape(4, 2);
        assert!(nbs.enabled /* should be enabled by default */,);
    }

    #[test]
    fn test_forward_output_length() {
        let nbs = new_neural_blend_shape(4, 6);
        let out = nbs_forward(&nbs, &[0.0; 4]);
        assert_eq!(
            out.len(),
            6, /* forward output length must match output_dim */
        );
    }

    #[test]
    fn test_forward_disabled_still_runs() {
        let mut nbs = new_neural_blend_shape(4, 3);
        nbs_set_enabled(&mut nbs, false);
        let out = nbs_forward(&nbs, &[1.0; 4]);
        assert_eq!(
            out.len(),
            3, /* output length unchanged when disabled */
        );
    }

    #[test]
    fn test_set_activation() {
        let mut nbs = new_neural_blend_shape(2, 2);
        nbs_set_activation(&mut nbs, NbsActivation::Tanh);
        assert_eq!(
            nbs.activation,
            NbsActivation::Tanh, /* activation must be set */
        );
    }

    #[test]
    fn test_load_weights() {
        let mut nbs = new_neural_blend_shape(2, 2);
        nbs_load_weights(&mut nbs, &[1.0, 2.0, 3.0, 4.0]);
        assert!((nbs.weights[0] - 1.0).abs() < 1e-6, /* first weight must match */);
    }

    #[test]
    fn test_load_weights_partial() {
        let mut nbs = new_neural_blend_shape(4, 4);
        nbs_load_weights(&mut nbs, &[5.0, 6.0]);
        assert!((nbs.weights[0] - 5.0).abs() < 1e-6, /* partial load should succeed */);
    }

    #[test]
    fn test_to_json_contains_dims() {
        let nbs = new_neural_blend_shape(3, 5);
        let j = nbs_to_json(&nbs);
        assert!(j.contains("\"input_dim\""), /* json must contain input_dim */);
        assert!(j.contains("\"output_dim\""), /* json must contain output_dim */);
    }

    #[test]
    fn test_weight_count() {
        let nbs = new_neural_blend_shape(3, 4);
        assert_eq!(
            nbs.weights.len(),
            12, /* weight vector length must be input*output */
        );
    }

    #[test]
    fn test_bias_count() {
        let nbs = new_neural_blend_shape(3, 4);
        assert_eq!(
            nbs.bias.len(),
            4, /* bias length must equal output_dim */
        );
    }

    #[test]
    fn nbs_forward_identity_weights_zero_bias() {
        // Build a 3-in, 3-out NBS with identity weight matrix (w[i*od+j] = 1 if i==j else 0),
        // zero bias, ReLU activation. Output[j] should equal input[j] for all j < min(id,od).
        let id = 3;
        let od = 3;
        let mut nbs = new_neural_blend_shape(id, od);
        nbs_set_activation(&mut nbs, NbsActivation::Relu);
        // Identity weight matrix
        let mut weights = vec![0.0_f32; id * od];
        for k in 0..id.min(od) {
            weights[k * od + k] = 1.0;
        }
        nbs_load_weights(&mut nbs, &weights);
        // bias already zero from new_neural_blend_shape

        let input = [1.0_f32, 2.0, 3.0];
        let out = nbs_forward(&nbs, &input);

        for (j, (&o, &i)) in out.iter().zip(input.iter()).enumerate() {
            assert!((o - i).abs() < 1e-6, "output[{}] = {} expected {}", j, o, i);
        }
    }

    #[test]
    fn nbs_forward_weights_used_not_zero() {
        // With non-zero weights and non-zero input the output must differ from the old stub's
        // all-zero result. Use a 2-in, 2-out NBS with weights [1,0, 0,1] and input [0.5, -0.5].
        // After ReLU: out = [0.5, 0.0]  (not [0,0]).
        let mut nbs = new_neural_blend_shape(2, 2);
        nbs_set_activation(&mut nbs, NbsActivation::Relu);
        nbs_load_weights(&mut nbs, &[1.0, 0.0, 0.0, 1.0]);
        let out = nbs_forward(&nbs, &[0.5, -0.5]);
        // At least one output component must be non-zero.
        assert!(
            out.iter().any(|&v| v.abs() > 1e-6),
            "expected non-zero output with non-zero weights, got {:?}",
            out
        );
    }
}
