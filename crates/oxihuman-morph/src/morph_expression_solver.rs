// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Solve expression combination from target weights.

#[allow(dead_code)]
pub struct ExpressionSolverV2 {
    pub expression_names: Vec<String>,
    pub combination_matrix: Vec<Vec<f32>>,
}

#[allow(dead_code)]
pub fn new_expression_solver_v2() -> ExpressionSolverV2 {
    ExpressionSolverV2 { expression_names: Vec::new(), combination_matrix: Vec::new() }
}

#[allow(dead_code)]
pub fn esv2_add_expression(s: &mut ExpressionSolverV2, name: &str, morph_weights: Vec<f32>) {
    s.expression_names.push(name.to_string());
    s.combination_matrix.push(morph_weights);
}

#[allow(dead_code)]
pub fn esv2_expression_count(s: &ExpressionSolverV2) -> usize {
    s.expression_names.len()
}

#[allow(dead_code)]
pub fn esv2_solve(s: &ExpressionSolverV2, target_weights: &[f32]) -> Vec<f32> {
    s.combination_matrix.iter().map(|row| {
        row.iter().zip(target_weights.iter()).map(|(a, b)| a * b).sum()
    }).collect()
}

#[allow(dead_code)]
pub fn esv2_get_expression_idx(s: &ExpressionSolverV2, name: &str) -> Option<usize> {
    s.expression_names.iter().position(|n| n == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_expression() {
        let mut s = new_expression_solver_v2();
        esv2_add_expression(&mut s, "smile", vec![1.0, 0.0]);
        assert_eq!(esv2_expression_count(&s), 1);
    }

    #[test]
    fn test_expression_count() {
        let mut s = new_expression_solver_v2();
        esv2_add_expression(&mut s, "a", vec![1.0]);
        esv2_add_expression(&mut s, "b", vec![0.5]);
        assert_eq!(esv2_expression_count(&s), 2);
    }

    #[test]
    fn test_solve_identity() {
        let mut s = new_expression_solver_v2();
        esv2_add_expression(&mut s, "smile", vec![1.0, 0.0]);
        let result = esv2_solve(&s, &[1.0, 0.0]);
        assert!((result[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_solve_zero() {
        let mut s = new_expression_solver_v2();
        esv2_add_expression(&mut s, "smile", vec![1.0, 0.0]);
        let result = esv2_solve(&s, &[0.0, 0.0]);
        assert!(result[0].abs() < 1e-5);
    }

    #[test]
    fn test_get_expression_idx_found() {
        let mut s = new_expression_solver_v2();
        esv2_add_expression(&mut s, "frown", vec![0.0, 1.0]);
        assert_eq!(esv2_get_expression_idx(&s, "frown"), Some(0));
    }

    #[test]
    fn test_get_expression_idx_missing() {
        let s = new_expression_solver_v2();
        assert_eq!(esv2_get_expression_idx(&s, "none"), None);
    }

    #[test]
    fn test_solve_multiple() {
        let mut s = new_expression_solver_v2();
        esv2_add_expression(&mut s, "a", vec![1.0]);
        esv2_add_expression(&mut s, "b", vec![2.0]);
        let result = esv2_solve(&s, &[0.5]);
        assert!((result[0] - 0.5).abs() < 1e-5);
        assert!((result[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_solve() {
        let s = new_expression_solver_v2();
        let result = esv2_solve(&s, &[1.0]);
        assert!(result.is_empty());
    }
}
