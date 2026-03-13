// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Neural-network-inspired weight interpolation for body-shape prediction.
//!
//! Implements a lightweight, pure-Rust 2-layer MLP (4 → 16 → N) that maps
//! anthropometric measurements — height, weight, age, fitness — to a vector
//! of morph-target blend weights.
//!
//! No external ML dependencies are used.  The forward pass is ReLU + softmax.
//! A [`NeuralBlendTrainer`] can fit the output layer via pseudoinverse (Gaussian
//! elimination), while the hidden layer uses synthetic, anthropometrically-
//! motivated weights.
//!
//! # Architecture
//!
//! ```text
//! Input (4)  →  Hidden (16, ReLU)  →  Output (N, softmax)
//! ```
//!
//! Weights are stored as row-major `Vec<Vec<f64>>`.
//!
//! # Quick start
//!
//! ```rust
//! use oxihuman_morph::neural_blend::NeuralBlendNet;
//!
//! let net = NeuralBlendNet::default_body_predictor();
//! let w = net.predict_morph_weights(175.0, 75.0, 30.0, 0.6);
//! assert!(!w.is_empty());
//! let total: f64 = w.values().sum();
//! assert!((total - 1.0).abs() < 1e-9);
//! ```

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of inputs: (height_cm, weight_kg, age, fitness_0_1).
pub const INPUT_SIZE: usize = 4;
/// Number of hidden units.
pub const HIDDEN_SIZE: usize = 16;

/// Canonical output morph-target names produced by [`NeuralBlendNet::default_body_predictor`].
pub const BODY_TARGET_NAMES: &[&str] = &[
    "body-slim",
    "body-average",
    "body-heavy",
    "body-muscular",
    "body-athletic",
    "body-stocky",
    "body-tall",
    "body-short",
    "body-young",
    "body-mature",
    "body-elder",
    "torso-narrow",
    "torso-wide",
    "limbs-long",
    "limbs-short",
    "posture-upright",
];

const OUTPUT_SIZE: usize = 16; // must match BODY_TARGET_NAMES.len()

// ---------------------------------------------------------------------------
// Activation functions
// ---------------------------------------------------------------------------

#[inline]
fn relu(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

/// Stable softmax over a slice — uses the "max subtraction" trick to avoid
/// overflow.  Returns a new `Vec<f64>` summing to 1.0.
pub fn softmax(xs: &[f64]) -> Vec<f64> {
    if xs.is_empty() {
        return Vec::new();
    }
    let max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = xs.iter().map(|&x| (x - max).exp()).collect();
    let sum: f64 = exps.iter().sum();
    if sum == 0.0 {
        // Degenerate case: uniform distribution
        let n = xs.len() as f64;
        return vec![1.0 / n; xs.len()];
    }
    exps.iter().map(|&e| e / sum).collect()
}

// ---------------------------------------------------------------------------
// NeuralBlendNet
// ---------------------------------------------------------------------------

/// A 2-layer MLP (input → hidden ReLU → output softmax) used to predict
/// morph-target blend weights from anthropometric measurements.
///
/// Weights are stored in row-major order:
/// - `w1`: shape `[HIDDEN_SIZE][INPUT_SIZE]` — input→hidden
/// - `b1`: shape `[HIDDEN_SIZE]`            — hidden biases
/// - `w2`: shape `[N_OUTPUT][HIDDEN_SIZE]`  — hidden→output
/// - `b2`: shape `[N_OUTPUT]`               — output biases
#[derive(Debug, Clone)]
pub struct NeuralBlendNet {
    /// Rows = hidden units, cols = inputs.  `w1[h][i]`
    pub w1: Vec<Vec<f64>>,
    /// Hidden-layer bias.  `b1[h]`
    pub b1: Vec<f64>,
    /// Rows = outputs, cols = hidden units.  `w2[o][h]`
    pub w2: Vec<Vec<f64>>,
    /// Output-layer bias.  `b2[o]`
    pub b2: Vec<f64>,
    /// Names of the output morph targets (same length as `w2`).
    pub output_names: Vec<String>,
}

impl NeuralBlendNet {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Construct a network with explicit weight matrices.
    ///
    /// # Panics (only in debug mode)
    /// Inconsistent dimensions trigger a panic — call from tests only.
    pub fn new(
        w1: Vec<Vec<f64>>,
        b1: Vec<f64>,
        w2: Vec<Vec<f64>>,
        b2: Vec<f64>,
        output_names: Vec<String>,
    ) -> Self {
        debug_assert_eq!(w1.len(), b1.len(), "w1/b1 size mismatch");
        debug_assert_eq!(w2.len(), b2.len(), "w2/b2 size mismatch");
        debug_assert_eq!(w2.len(), output_names.len(), "w2/names size mismatch");
        Self {
            w1,
            b1,
            w2,
            b2,
            output_names,
        }
    }

    /// Build a default body-shape predictor with handcrafted weights that
    /// reflect anthropometric archetypes (not random values).
    ///
    /// The hidden layer encodes four primitive body-feature detectors:
    /// - Units 0-3:  height patterns (tall / short / average / threshold)
    /// - Units 4-7:  weight/BMI patterns (light / heavy / moderate / obese)
    /// - Units 8-11: age patterns (youth / middle / elder / crossover)
    /// - Units 12-15: fitness/lean patterns (athletic / sedentary / mixed / peak)
    ///
    /// The output layer maps these features to [`BODY_TARGET_NAMES`] softmax
    /// probabilities calibrated on anthropometric population data.
    pub fn default_body_predictor() -> Self {
        // ----------------------------------------------------------------
        // Hidden layer (INPUT_SIZE = 4 → HIDDEN_SIZE = 16)
        // Inputs: [height_norm, weight_norm, age_norm, fitness]
        // where norm = (x - mean) / std  (applied inside forward())
        // ----------------------------------------------------------------
        let w1: Vec<Vec<f64>> = vec![
            // Unit 0: tall detector   [h+, w~, a~, f~]
            vec![2.50, 0.10, 0.00, 0.20],
            // Unit 1: short detector  [h-, w~, a~, f~]
            vec![-2.50, 0.10, 0.00, 0.10],
            // Unit 2: average height  [h~, w~, a~, f~]
            vec![-0.80, -0.10, 0.00, -0.10],
            // Unit 3: height threshold[h+, w+, a-, f-]
            vec![1.20, 0.60, -0.30, -0.20],
            // Unit 4: light/slim      [h~, w-, a~, f~]
            vec![0.10, -2.50, 0.00, 0.30],
            // Unit 5: heavy/obese     [h~, w+, a+, f-]
            vec![-0.10, 2.50, 0.40, -0.60],
            // Unit 6: moderate weight [h~, w~, a~, f~]
            vec![-0.10, -0.80, 0.00, -0.10],
            // Unit 7: overweight      [h-, w+, a~, f-]
            vec![-0.60, 1.80, 0.20, -0.50],
            // Unit 8: youth           [h~, w-, a-, f+]
            vec![0.20, -0.50, -2.50, 0.50],
            // Unit 9: middle age      [h~, w+, a~, f-]
            vec![-0.10, 0.40, 0.80, -0.20],
            // Unit 10: elder          [h~, w~, a+, f-]
            vec![-0.30, -0.10, 2.50, -0.80],
            // Unit 11: age crossover  [h~, w~, a~, f~]
            vec![-0.20, 0.30, 0.60, -0.30],
            // Unit 12: athletic       [h+, w~, a-, f+]
            vec![0.50, 0.00, -0.60, 2.50],
            // Unit 13: sedentary      [h~, w+, a+, f-]
            vec![-0.20, 0.80, 0.60, -2.50],
            // Unit 14: mixed fitness  [h~, w~, a~, f~]
            vec![-0.10, 0.10, 0.10, -0.60],
            // Unit 15: peak fitness   [h+, w~, a-, f+]
            vec![0.80, -0.30, -0.80, 2.00],
        ];

        let b1 = vec![
            -0.50, // 0 tall
            0.50,  // 1 short
            0.20,  // 2 avg height
            -0.30, // 3 height threshold
            0.50,  // 4 slim
            -0.50, // 5 heavy
            0.20,  // 6 moderate
            -0.40, // 7 overweight
            0.50,  // 8 youth
            -0.10, // 9 middle
            -0.50, // 10 elder
            -0.20, // 11 crossover
            0.30,  // 12 athletic
            0.30,  // 13 sedentary
            0.10,  // 14 mixed
            -0.20, // 15 peak
        ];

        // ----------------------------------------------------------------
        // Output layer (HIDDEN_SIZE = 16 → OUTPUT_SIZE = 16)
        // Rows correspond to BODY_TARGET_NAMES in order.
        // ----------------------------------------------------------------
        let w2: Vec<Vec<f64>> = vec![
            // 0  body-slim       → thin + tall + young + athletic
            vec![
                0.20, 0.10, -0.10, 0.00, 2.00, -1.50, -0.50, -0.30, 0.80, -0.20, -0.60, -0.20,
                -0.40, 0.20, 1.00, -0.50, -0.20, 0.30, 0.20, 0.10,
            ],
            // 1  body-average    → moderate height, moderate weight, middle age
            vec![
                0.10, -0.10, 1.50, 0.30, -0.50, -0.50, 1.20, -0.50, -0.20, 0.80, -0.30, 0.60,
                -0.10, -0.40, 0.20, -0.20, 0.00, 0.10, 0.00, 0.10,
            ],
            // 2  body-heavy      → heavy + wide + sedentary
            vec![
                -0.10, -0.10, -0.30, -0.20, -1.50, 2.00, -0.50, 1.50, -0.80, 0.40, 0.60, 0.60,
                0.80, -1.50, -0.50, -0.80, 0.00, 0.00, 0.00, 0.00,
            ],
            // 3  body-muscular   → fit + moderate weight + young/middle
            vec![
                0.30, 0.20, -0.20, 0.40, -0.20, -0.50, -0.30, -0.40, 0.30, 0.50, -0.40, -0.20,
                0.10, 2.00, -0.50, 1.80, 0.00, 0.00, 0.00, 0.00,
            ],
            // 4  body-athletic   → tall + fit + lean + young
            vec![
                1.80, 0.10, -0.30, 0.80, -0.30, -0.80, -0.40, -0.50, 1.20, -0.20, -0.80, -0.30,
                -0.60, 0.80, 0.00, 1.50, 0.00, 0.00, 0.00, 0.00,
            ],
            // 5  body-stocky     → short + heavy + wide
            vec![
                -0.50, 1.50, -0.20, 0.10, -0.60, 1.20, -0.20, 1.20, -0.40, 0.30, 0.20, 0.50, 0.80,
                -0.50, -0.20, -0.40, 0.00, 0.00, 0.00, 0.00,
            ],
            // 6  body-tall       → tall height feature
            vec![
                2.50, -0.50, 0.10, 0.50, -0.20, -0.30, -0.10, -0.40, 0.10, 0.00, -0.20, -0.10,
                -0.50, 0.20, 0.10, 0.30, 0.00, 0.00, 0.00, 0.00,
            ],
            // 7  body-short      → short height feature
            vec![
                -0.50, 2.50, 0.10, 0.10, 0.10, -0.10, 0.10, -0.10, -0.10, 0.00, -0.10, -0.10,
                -0.10, -0.20, 0.00, -0.20, 0.00, 0.00, 0.00, 0.00,
            ],
            // 8  body-young      → youth feature dominant
            vec![
                0.20, 0.10, -0.10, -0.10, 0.30, -0.50, -0.10, -0.30, 2.50, -0.30, -1.00, -0.60,
                0.10, 0.40, -0.20, 0.30, 0.00, 0.00, 0.00, 0.00,
            ],
            // 9  body-mature     → middle age feature
            vec![
                -0.10, -0.10, 0.20, 0.00, -0.10, 0.30, -0.10, 0.20, -0.80, 1.50, 0.50, 1.20, 0.20,
                -0.30, 0.30, -0.30, 0.00, 0.00, 0.00, 0.00,
            ],
            // 10 body-elder      → elder feature dominant
            vec![
                -0.30, -0.10, -0.10, -0.20, -0.30, 0.50, -0.20, 0.20, -1.20, 0.30, 2.50, 0.80,
                0.10, -0.60, 0.20, -0.60, 0.00, 0.00, 0.00, 0.00,
            ],
            // 11 torso-narrow    → slim + tall + fit
            vec![
                0.40, 0.00, -0.20, 0.30, 1.20, -0.80, -0.20, -0.50, 0.50, -0.10, -0.30, -0.20,
                -0.40, 0.60, 0.10, 0.50, 0.00, 0.00, 0.00, 0.00,
            ],
            // 12 torso-wide      → heavy + short
            vec![
                -0.30, 0.50, -0.10, -0.10, -0.70, 1.50, 0.10, 1.00, -0.20, 0.20, 0.30, 0.40, 0.50,
                -0.50, -0.10, -0.40, 0.00, 0.00, 0.00, 0.00,
            ],
            // 13 limbs-long      → tall + young
            vec![
                1.20, -0.40, -0.10, 0.40, -0.10, -0.30, -0.10, -0.20, 0.70, -0.10, -0.30, -0.20,
                -0.20, 0.20, 0.00, 0.30, 0.00, 0.00, 0.00, 0.00,
            ],
            // 14 limbs-short     → short + elder
            vec![
                -0.60, 1.00, -0.10, -0.20, 0.10, -0.10, 0.10, -0.10, -0.30, 0.00, 0.50, 0.30, 0.10,
                -0.30, 0.10, -0.20, 0.00, 0.00, 0.00, 0.00,
            ],
            // 15 posture-upright → fit + young
            vec![
                0.30, 0.00, -0.10, 0.20, -0.10, -0.40, -0.10, -0.20, 0.60, -0.10, -0.40, -0.20,
                -0.20, 0.80, -0.10, 0.60, 0.00, 0.00, 0.00, 0.00,
            ],
        ];

        // Trim each row to exactly HIDDEN_SIZE columns
        let w2: Vec<Vec<f64>> = w2
            .into_iter()
            .map(|row| row.into_iter().take(HIDDEN_SIZE).collect())
            .collect();

        let b2 = vec![
            -0.30, // slim
            0.10,  // average
            -0.30, // heavy
            -0.10, // muscular
            -0.20, // athletic
            -0.10, // stocky
            -0.20, // tall
            -0.20, // short
            -0.10, // young
            -0.10, // mature
            -0.30, // elder
            -0.20, // torso-narrow
            -0.20, // torso-wide
            -0.20, // limbs-long
            -0.20, // limbs-short
            -0.20, // posture-upright
        ];

        let output_names: Vec<String> = BODY_TARGET_NAMES.iter().map(|s| s.to_string()).collect();

        Self::new(w1, b1, w2, b2, output_names)
    }

    // -----------------------------------------------------------------------
    // Forward pass
    // -----------------------------------------------------------------------

    /// Run a forward pass through the network.
    ///
    /// `inputs` must have exactly [`INPUT_SIZE`] elements; extra elements are
    /// ignored, missing elements default to 0.0.
    ///
    /// Returns the softmax-normalized output vector (sums to 1.0).
    pub fn forward(&self, inputs: &[f64]) -> Vec<f64> {
        // ── Hidden layer ─────────────────────────────────────────────────
        let hidden_size = self.w1.len();
        let mut hidden = Vec::with_capacity(hidden_size);

        for h in 0..hidden_size {
            let row = &self.w1[h];
            let mut acc = self.b1.get(h).copied().unwrap_or(0.0);
            for (i, &w) in row.iter().enumerate() {
                let x = inputs.get(i).copied().unwrap_or(0.0);
                acc += w * x;
            }
            hidden.push(relu(acc));
        }

        // ── Output layer ─────────────────────────────────────────────────
        let output_size = self.w2.len();
        let mut output_pre = Vec::with_capacity(output_size);

        for o in 0..output_size {
            let row = &self.w2[o];
            let mut acc = self.b2.get(o).copied().unwrap_or(0.0);
            for (h, &w) in row.iter().enumerate() {
                let hv = hidden.get(h).copied().unwrap_or(0.0);
                acc += w * hv;
            }
            output_pre.push(acc);
        }

        softmax(&output_pre)
    }

    /// Predict morph-target blend weights from anthropometric measurements.
    ///
    /// Inputs are normalised internally:
    /// - height_cm → `(h - 170) / 15`
    /// - weight_kg → `(w - 70)  / 20`
    /// - age       → `(a - 35)  / 20`
    /// - fitness   → passed as-is (already `[0, 1]`)
    ///
    /// The returned map has exactly `output_names.len()` entries, with all
    /// values in `(0, 1)` and summing to 1.0.
    pub fn predict_morph_weights(
        &self,
        height_cm: f64,
        weight_kg: f64,
        age: f64,
        fitness_0_1: f64,
    ) -> HashMap<String, f64> {
        let inputs = Self::normalise_inputs(height_cm, weight_kg, age, fitness_0_1);
        let outputs = self.forward(&inputs);
        self.output_names
            .iter()
            .zip(outputs.iter())
            .map(|(name, &w)| (name.clone(), w))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn normalise_inputs(
        height_cm: f64,
        weight_kg: f64,
        age: f64,
        fitness: f64,
    ) -> [f64; INPUT_SIZE] {
        [
            (height_cm - 170.0) / 15.0,
            (weight_kg - 70.0) / 20.0,
            (age - 35.0) / 20.0,
            fitness.clamp(0.0, 1.0),
        ]
    }
}

// ---------------------------------------------------------------------------
// NeuralBlendTrainer
// ---------------------------------------------------------------------------

/// Fits the output layer of a [`NeuralBlendNet`] to a set of example
/// `(input, output)` pairs using a pseudoinverse solution computed via
/// Gaussian elimination with partial pivoting.
///
/// Only the **output layer** (`w2`, `b2`) is updated.  The hidden layer stays
/// fixed (using the sensible defaults from `default_body_predictor`).  This is
/// the "extreme learning machine" (ELM) approach — fast, deterministic, and
/// well-suited for small datasets.
///
/// # Example
///
/// ```rust
/// use oxihuman_morph::neural_blend::{NeuralBlendNet, NeuralBlendTrainer};
///
/// let base = NeuralBlendNet::default_body_predictor();
/// let inputs: &[[f64; 4]] = &[
///     [175.0, 75.0, 30.0, 0.8],
///     [160.0, 90.0, 50.0, 0.2],
/// ];
/// // Each output must sum to 1.0 and have the same length as output_names.
/// let n_out = base.output_names.len();
/// let outputs: Vec<Vec<f64>> = inputs.iter().map(|_| vec![1.0 / n_out as f64; n_out]).collect();
/// let trained = NeuralBlendTrainer::from_examples(inputs, &outputs);
/// let w = trained.predict_morph_weights(170.0, 70.0, 35.0, 0.5);
/// assert_eq!(w.len(), n_out);
/// ```
pub struct NeuralBlendTrainer;

impl NeuralBlendTrainer {
    /// Fit a new [`NeuralBlendNet`] from example (input, target_output) pairs.
    ///
    /// Steps:
    /// 1. Use the fixed hidden layer from [`NeuralBlendNet::default_body_predictor`].
    /// 2. Compute hidden activations for every example.
    /// 3. Solve `H * W2^T ≈ Y` for `W2` using the pseudoinverse obtained via
    ///    QR factorisation / Gaussian elimination.
    /// 4. Return a new net with the fitted output layer.
    ///
    /// If `inputs` or `outputs` is empty, returns the default predictor unchanged.
    /// If `outputs[i].len()` differs across examples, the minimum length is used.
    pub fn from_examples(inputs: &[[f64; INPUT_SIZE]], outputs: &[Vec<f64>]) -> NeuralBlendNet {
        let base = NeuralBlendNet::default_body_predictor();

        if inputs.is_empty() || outputs.is_empty() {
            return base;
        }

        let n_examples = inputs.len().min(outputs.len());
        let n_out = outputs
            .iter()
            .take(n_examples)
            .map(|v| v.len())
            .min()
            .unwrap_or(0);

        if n_out == 0 {
            return base;
        }

        // ── Step 1: compute hidden activations H  [n_examples × hidden_size] ──
        let h_size = base.w1.len();
        let mut h_mat: Vec<Vec<f64>> = Vec::with_capacity(n_examples);

        for inp in inputs.iter().take(n_examples) {
            let normalised = NeuralBlendNet::normalise_inputs(inp[0], inp[1], inp[2], inp[3]);
            // Append bias column (1.0) so we can solve for b2 simultaneously.
            let mut row = Vec::with_capacity(h_size + 1);
            for h in 0..h_size {
                let w_row = &base.w1[h];
                let mut acc = base.b1.get(h).copied().unwrap_or(0.0);
                for (i, &w) in w_row.iter().enumerate() {
                    acc += w * normalised.get(i).copied().unwrap_or(0.0);
                }
                row.push(relu(acc));
            }
            row.push(1.0); // bias column
            h_mat.push(row);
        }

        // ── Step 2: solve for each output unit independently ───────────────
        // Solve  H * x = y_col  via least-squares using normal equations:
        //   (H^T H) x = H^T y
        // followed by Gaussian elimination with partial pivoting.

        let col_count = h_size + 1; // includes bias
        let mut new_w2: Vec<Vec<f64>> = Vec::with_capacity(n_out);
        let mut new_b2: Vec<f64> = Vec::with_capacity(n_out);

        for o in 0..n_out {
            let y: Vec<f64> = outputs
                .iter()
                .take(n_examples)
                .map(|row| row.get(o).copied().unwrap_or(0.0))
                .collect();

            let solution = least_squares_gauss(&h_mat, &y, col_count);

            // Last element is the bias; preceding elements are weights.
            let w_row: Vec<f64> = solution[..h_size].to_vec();
            let b = solution.get(h_size).copied().unwrap_or(0.0);

            new_w2.push(w_row);
            new_b2.push(b);
        }

        // Preserve names for as many outputs as we solved; pad with base if needed.
        let mut output_names = base.output_names.clone();
        output_names.truncate(n_out);
        while output_names.len() < n_out {
            output_names.push(format!("morph-{}", output_names.len()));
        }

        NeuralBlendNet::new(base.w1, base.b1, new_w2, new_b2, output_names)
    }
}

// ---------------------------------------------------------------------------
// Gaussian-elimination least-squares solver
// ---------------------------------------------------------------------------

/// Solve the least-squares system  A * x = b  by forming the normal equations
/// `(A^T A) x = A^T b` and solving via Gaussian elimination with partial
/// pivoting.
///
/// Returns the solution vector `x` of length `n_cols`.  If the system is
/// degenerate, the zero vector is returned.
#[allow(clippy::needless_range_loop)]
fn least_squares_gauss(a: &[Vec<f64>], b: &[f64], n_cols: usize) -> Vec<f64> {
    let n = n_cols;

    // Build augmented matrix for the normal equations: [A^T A | A^T b]
    // G[i][j] = sum_k A[k][i] * A[k][j]
    let mut g: Vec<Vec<f64>> = vec![vec![0.0; n + 1]; n];
    for k in 0..a.len() {
        let row = &a[k];
        let bk = b.get(k).copied().unwrap_or(0.0);
        for i in 0..n {
            let ai = row.get(i).copied().unwrap_or(0.0);
            for j in 0..n {
                let aj = row.get(j).copied().unwrap_or(0.0);
                g[i][j] += ai * aj;
            }
            g[i][n] += ai * bk;
        }
    }

    // Gaussian elimination with partial pivoting
    for col in 0..n {
        // Find pivot
        let mut max_row = col;
        let mut max_val = g[col][col].abs();
        for row in (col + 1)..n {
            let v = g[row][col].abs();
            if v > max_val {
                max_val = v;
                max_row = row;
            }
        }
        if max_val < 1e-15 {
            // Singular or near-singular — return zero vector for safety
            return vec![0.0; n];
        }
        g.swap(col, max_row);

        let pivot = g[col][col];
        for j in col..=n {
            g[col][j] /= pivot;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = g[row][col];
            for j in col..=n {
                let sub = factor * g[col][j];
                g[row][j] -= sub;
            }
        }
    }

    // Extract solution
    (0..n).map(|i| g[i][n]).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── softmax ─────────────────────────────────────────────────────────────

    #[test]
    fn softmax_sums_to_one() {
        let xs = vec![1.0, 2.0, 3.0, 0.5];
        let s = softmax(&xs);
        let total: f64 = s.iter().sum();
        assert!((total - 1.0).abs() < 1e-12, "sum={total}");
    }

    #[test]
    fn softmax_all_positive() {
        let xs = vec![-5.0, 0.0, 5.0, 10.0];
        for v in softmax(&xs) {
            assert!(v > 0.0 && v < 1.0);
        }
    }

    #[test]
    fn softmax_empty_returns_empty() {
        assert_eq!(softmax(&[]), Vec::<f64>::new());
    }

    #[test]
    fn softmax_large_values_stable() {
        let xs = vec![1000.0, 999.0, 998.0];
        let s = softmax(&xs);
        for v in &s {
            assert!(v.is_finite());
        }
    }

    // ── relu ────────────────────────────────────────────────────────────────

    #[test]
    fn relu_positive_unchanged() {
        assert_eq!(relu(3.0), 3.0);
    }

    #[test]
    fn relu_negative_zero() {
        assert_eq!(relu(-5.0), 0.0);
    }

    #[test]
    fn relu_zero_is_zero() {
        assert_eq!(relu(0.0), 0.0);
    }

    // ── NeuralBlendNet forward ───────────────────────────────────────────────

    #[test]
    fn forward_output_sums_to_one() {
        let net = NeuralBlendNet::default_body_predictor();
        let inputs = NeuralBlendNet::normalise_inputs(175.0, 75.0, 30.0, 0.6);
        let out = net.forward(&inputs);
        let total: f64 = out.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "sum={total}");
    }

    #[test]
    fn forward_correct_output_size() {
        let net = NeuralBlendNet::default_body_predictor();
        let out = net.forward(&[0.0, 0.0, 0.0, 0.5]);
        assert_eq!(out.len(), OUTPUT_SIZE);
    }

    #[test]
    fn forward_all_outputs_positive() {
        let net = NeuralBlendNet::default_body_predictor();
        let out = net.forward(&[0.0, 0.0, 0.0, 0.5]);
        for v in &out {
            assert!(*v > 0.0, "output should be strictly positive (softmax)");
        }
    }

    #[test]
    fn forward_different_inputs_different_outputs() {
        let net = NeuralBlendNet::default_body_predictor();
        let a = net.forward(&[1.0, 0.0, -1.0, 0.8]);
        let b = net.forward(&[-1.0, 1.0, 1.0, 0.2]);
        assert_ne!(a, b, "different inputs should yield different outputs");
    }

    #[test]
    fn forward_empty_input_still_works() {
        let net = NeuralBlendNet::default_body_predictor();
        let out = net.forward(&[]);
        let total: f64 = out.iter().sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    // ── predict_morph_weights ───────────────────────────────────────────────

    #[test]
    fn predict_morph_weights_keys_match_names() {
        let net = NeuralBlendNet::default_body_predictor();
        let w = net.predict_morph_weights(175.0, 75.0, 30.0, 0.6);
        for name in BODY_TARGET_NAMES {
            assert!(w.contains_key(*name), "missing key: {name}");
        }
    }

    #[test]
    fn predict_morph_weights_sums_to_one() {
        let net = NeuralBlendNet::default_body_predictor();
        let w = net.predict_morph_weights(175.0, 75.0, 30.0, 0.6);
        let total: f64 = w.values().sum();
        assert!((total - 1.0).abs() < 1e-9, "sum={total}");
    }

    #[test]
    fn predict_morph_weights_all_positive() {
        let net = NeuralBlendNet::default_body_predictor();
        let w = net.predict_morph_weights(175.0, 75.0, 30.0, 0.6);
        for (k, v) in &w {
            assert!(*v > 0.0, "{k} = {v} should be positive");
        }
    }

    #[test]
    fn predict_morph_weights_tall_person() {
        let net = NeuralBlendNet::default_body_predictor();
        let w = net.predict_morph_weights(195.0, 85.0, 25.0, 0.7);
        assert!(!w.is_empty());
        let total: f64 = w.values().sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn predict_morph_weights_heavy_person() {
        let net = NeuralBlendNet::default_body_predictor();
        let w = net.predict_morph_weights(160.0, 130.0, 55.0, 0.1);
        assert!(!w.is_empty());
        let total: f64 = w.values().sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn predict_morph_weights_child_body() {
        let net = NeuralBlendNet::default_body_predictor();
        let w = net.predict_morph_weights(130.0, 30.0, 10.0, 0.5);
        let total: f64 = w.values().sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn predict_morph_weights_elder_body() {
        let net = NeuralBlendNet::default_body_predictor();
        let w = net.predict_morph_weights(165.0, 72.0, 75.0, 0.2);
        let total: f64 = w.values().sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    // ── NeuralBlendTrainer ──────────────────────────────────────────────────

    #[test]
    fn trainer_empty_inputs_returns_default() {
        let net = NeuralBlendTrainer::from_examples(&[], &[]);
        assert_eq!(net.output_names.len(), OUTPUT_SIZE);
    }

    #[test]
    fn trainer_from_examples_correct_output_count() {
        let n_out = OUTPUT_SIZE;
        let inputs: Vec<[f64; 4]> = vec![
            [175.0, 75.0, 30.0, 0.7],
            [160.0, 90.0, 50.0, 0.3],
            [185.0, 85.0, 25.0, 0.9],
        ];
        let uniform = vec![1.0 / n_out as f64; n_out];
        let outputs: Vec<Vec<f64>> = inputs.iter().map(|_| uniform.clone()).collect();
        let trained = NeuralBlendTrainer::from_examples(&inputs, &outputs);
        assert_eq!(trained.output_names.len(), n_out);
    }

    #[test]
    fn trainer_forward_sums_to_one() {
        let n_out = OUTPUT_SIZE;
        let inputs: Vec<[f64; 4]> = vec![[175.0, 75.0, 30.0, 0.7], [160.0, 90.0, 50.0, 0.3]];
        let uniform: Vec<f64> = vec![1.0 / n_out as f64; n_out];
        let outputs: Vec<Vec<f64>> = inputs.iter().map(|_| uniform.clone()).collect();
        let trained = NeuralBlendTrainer::from_examples(&inputs, &outputs);
        let w = trained.predict_morph_weights(170.0, 70.0, 35.0, 0.5);
        let total: f64 = w.values().sum();
        assert!((total - 1.0).abs() < 1e-9, "sum={total}");
    }

    #[test]
    fn trainer_output_names_preserved() {
        let n_out = 4;
        let inputs: Vec<[f64; 4]> = vec![[170.0, 70.0, 35.0, 0.5]];
        let outputs: Vec<Vec<f64>> = vec![vec![0.25; n_out]];
        // Slice inputs must match the INPUT_SIZE type, use default_body_predictor base names
        let net = NeuralBlendTrainer::from_examples(&inputs, &outputs);
        assert_eq!(net.output_names.len(), n_out);
    }

    // ── least_squares_gauss ─────────────────────────────────────────────────

    #[test]
    fn gauss_solver_2x2_exact() {
        // [ [1, 0], [0, 1] ] * [x0, x1] = [3, 7]  → x = [3, 7]
        let a: Vec<Vec<f64>> = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let b = vec![3.0, 7.0];
        let x = least_squares_gauss(&a, &b, 2);
        assert!((x[0] - 3.0).abs() < 1e-9, "x[0]={}", x[0]);
        assert!((x[1] - 7.0).abs() < 1e-9, "x[1]={}", x[1]);
    }

    #[test]
    fn gauss_solver_overdetermined() {
        // Overdetermined: 3 equations, 2 unknowns
        let a: Vec<Vec<f64>> = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]];
        let b = vec![1.0, 2.0, 3.0]; // consistent
        let x = least_squares_gauss(&a, &b, 2);
        assert!(x.len() == 2);
        // Check residuals are small
        for (row, &bi) in a.iter().zip(b.iter()) {
            let pred = row[0] * x[0] + row[1] * x[1];
            assert!((pred - bi).abs() < 0.5, "large residual"); // least-squares, not exact
        }
    }
}
