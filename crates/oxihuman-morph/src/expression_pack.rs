#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Packaged expression set (e.g. FACS-like).

/// A single named expression definition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionDef {
    pub name: String,
    pub morph_indices: Vec<usize>,
    pub morph_weights: Vec<f32>,
}

/// A collection of named expression definitions.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ExpressionPack {
    pub expressions: Vec<ExpressionDef>,
}

/// Create an empty `ExpressionPack`.
#[allow(dead_code)]
pub fn new_expression_pack() -> ExpressionPack {
    ExpressionPack::default()
}

/// Add a named expression to the pack.
///
/// `indices` and `weights` must have the same length; mismatched lengths are
/// silently truncated to the shorter.
#[allow(dead_code)]
pub fn add_expression(
    pack: &mut ExpressionPack,
    name: &str,
    indices: Vec<usize>,
    weights: Vec<f32>,
) {
    let n = indices.len().min(weights.len());
    pack.expressions.push(ExpressionDef {
        name: name.to_string(),
        morph_indices: indices[..n].to_vec(),
        morph_weights: weights[..n].to_vec(),
    });
}

/// Apply a named expression to `out_weights`, scaled by `strength`.
///
/// Returns `true` if the expression was found, `false` otherwise.
#[allow(dead_code)]
pub fn apply_expression(
    pack: &ExpressionPack,
    name: &str,
    out_weights: &mut [f32],
    strength: f32,
) -> bool {
    let strength = strength.clamp(0.0, 1.0);
    if let Some(expr) = pack.expressions.iter().find(|e| e.name == name) {
        for (&idx, &w) in expr.morph_indices.iter().zip(expr.morph_weights.iter()) {
            if idx < out_weights.len() {
                out_weights[idx] += w * strength;
            }
        }
        return true;
    }
    false
}

/// Return the number of expressions in the pack.
#[allow(dead_code)]
pub fn expression_count(pack: &ExpressionPack) -> usize {
    pack.expressions.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pack_is_empty() {
        let p = new_expression_pack();
        assert_eq!(expression_count(&p), 0);
    }

    #[test]
    fn add_expression_increments_count() {
        let mut p = new_expression_pack();
        add_expression(&mut p, "smile", vec![0, 1], vec![0.8, 0.4]);
        assert_eq!(expression_count(&p), 1);
    }

    #[test]
    fn apply_existing_expression_returns_true() {
        let mut p = new_expression_pack();
        add_expression(&mut p, "smile", vec![0], vec![1.0]);
        let mut w = vec![0.0_f32; 4];
        assert!(apply_expression(&p, "smile", &mut w, 1.0));
    }

    #[test]
    fn apply_missing_expression_returns_false() {
        let p = new_expression_pack();
        let mut w = vec![0.0_f32; 4];
        assert!(!apply_expression(&p, "frown", &mut w, 1.0));
    }

    #[test]
    fn apply_expression_sets_correct_weights() {
        let mut p = new_expression_pack();
        add_expression(&mut p, "brow_raise", vec![2], vec![0.9]);
        let mut w = vec![0.0_f32; 4];
        apply_expression(&p, "brow_raise", &mut w, 1.0);
        assert!((w[2] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn apply_expression_scaled_by_strength() {
        let mut p = new_expression_pack();
        add_expression(&mut p, "e", vec![0], vec![1.0]);
        let mut w = vec![0.0_f32; 2];
        apply_expression(&p, "e", &mut w, 0.5);
        assert!((w[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn add_expression_mismatched_len_truncates() {
        let mut p = new_expression_pack();
        add_expression(&mut p, "e", vec![0, 1, 2], vec![1.0, 1.0]);
        assert_eq!(p.expressions[0].morph_indices.len(), 2);
    }

    #[test]
    fn apply_expression_out_of_range_index_skipped() {
        let mut p = new_expression_pack();
        add_expression(&mut p, "e", vec![99], vec![1.0]);
        let mut w = vec![0.0_f32; 4];
        apply_expression(&p, "e", &mut w, 1.0); // must not panic
    }

    #[test]
    fn expression_count_multiple() {
        let mut p = new_expression_pack();
        for i in 0..5u32 {
            add_expression(&mut p, &i.to_string(), vec![], vec![]);
        }
        assert_eq!(expression_count(&p), 5);
    }

    #[test]
    fn apply_expression_accumulates() {
        let mut p = new_expression_pack();
        add_expression(&mut p, "e", vec![0], vec![0.4]);
        let mut w = vec![0.2_f32; 2];
        apply_expression(&p, "e", &mut w, 1.0);
        assert!((w[0] - 0.6).abs() < 1e-6);
    }
}
