// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ML-based expression retargeting adapter stub.

/// Mapping from a source expression weight to a target expression weight.
#[derive(Debug, Clone)]
pub struct RetargetMapping {
    pub source_idx: usize,
    pub target_idx: usize,
    pub gain: f32,
    pub offset: f32,
}

/// ML expression retargeting adapter.
#[derive(Debug, Clone)]
pub struct ExpressionRetargetMl {
    pub mappings: Vec<RetargetMapping>,
    pub source_dim: usize,
    pub target_dim: usize,
    pub enabled: bool,
}

impl ExpressionRetargetMl {
    pub fn new(source_dim: usize, target_dim: usize) -> Self {
        ExpressionRetargetMl {
            mappings: Vec::new(),
            source_dim,
            target_dim,
            enabled: true,
        }
    }
}

/// Create a new ML expression retargeting adapter.
pub fn new_expression_retarget_ml(source_dim: usize, target_dim: usize) -> ExpressionRetargetMl {
    ExpressionRetargetMl::new(source_dim, target_dim)
}

/// Add a mapping rule.
pub fn erml_add_mapping(adapter: &mut ExpressionRetargetMl, mapping: RetargetMapping) {
    adapter.mappings.push(mapping);
}

/// Retarget source weights to target weights (stub: zeroed output).
pub fn erml_retarget(adapter: &ExpressionRetargetMl, source: &[f32]) -> Vec<f32> {
    /* Stub: applies linear mappings from source to target */
    let mut out = vec![0.0f32; adapter.target_dim];
    for m in &adapter.mappings {
        if m.source_idx < source.len() && m.target_idx < out.len() {
            out[m.target_idx] += source[m.source_idx] * m.gain + m.offset;
        }
    }
    out
}

/// Return mapping count.
pub fn erml_mapping_count(adapter: &ExpressionRetargetMl) -> usize {
    adapter.mappings.len()
}

/// Enable or disable.
pub fn erml_set_enabled(adapter: &mut ExpressionRetargetMl, enabled: bool) {
    adapter.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn erml_to_json(adapter: &ExpressionRetargetMl) -> String {
    format!(
        r#"{{"source_dim":{},"target_dim":{},"mappings":{},"enabled":{}}}"#,
        adapter.source_dim,
        adapter.target_dim,
        adapter.mappings.len(),
        adapter.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dims() {
        let a = new_expression_retarget_ml(10, 15);
        assert_eq!(a.source_dim, 10 /* source_dim must match */,);
        assert_eq!(a.target_dim, 15 /* target_dim must match */,);
    }

    #[test]
    fn test_no_mappings_initially() {
        let a = new_expression_retarget_ml(5, 5);
        assert_eq!(erml_mapping_count(&a), 0 /* no mappings initially */,);
    }

    #[test]
    fn test_add_mapping() {
        let mut a = new_expression_retarget_ml(5, 5);
        erml_add_mapping(
            &mut a,
            RetargetMapping {
                source_idx: 0,
                target_idx: 0,
                gain: 1.0,
                offset: 0.0,
            },
        );
        assert_eq!(erml_mapping_count(&a), 1 /* one mapping after add */,);
    }

    #[test]
    fn test_retarget_output_length() {
        let a = new_expression_retarget_ml(4, 6);
        let out = erml_retarget(&a, &[0.5; 4]);
        assert_eq!(out.len(), 6 /* output length must match target_dim */,);
    }

    #[test]
    fn test_retarget_with_gain() {
        let mut a = new_expression_retarget_ml(2, 2);
        erml_add_mapping(
            &mut a,
            RetargetMapping {
                source_idx: 0,
                target_idx: 0,
                gain: 2.0,
                offset: 0.0,
            },
        );
        let out = erml_retarget(&a, &[0.5, 0.0]);
        assert!((out[0] - 1.0).abs() < 1e-5, /* gain of 2 on 0.5 must yield 1.0 */);
    }

    #[test]
    fn test_retarget_with_offset() {
        let mut a = new_expression_retarget_ml(2, 2);
        erml_add_mapping(
            &mut a,
            RetargetMapping {
                source_idx: 0,
                target_idx: 0,
                gain: 0.0,
                offset: 0.3,
            },
        );
        let out = erml_retarget(&a, &[0.0, 0.0]);
        assert!((out[0] - 0.3).abs() < 1e-5, /* offset alone must appear in output */);
    }

    #[test]
    fn test_set_enabled() {
        let mut a = new_expression_retarget_ml(2, 2);
        erml_set_enabled(&mut a, false);
        assert!(!a.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_dims() {
        let a = new_expression_retarget_ml(3, 4);
        let j = erml_to_json(&a);
        assert!(j.contains("\"source_dim\""), /* json must contain source_dim */);
    }

    #[test]
    fn test_enabled_default() {
        let a = new_expression_retarget_ml(1, 1);
        assert!(a.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_retarget_zeroed_without_mappings() {
        let a = new_expression_retarget_ml(3, 3);
        let out = erml_retarget(&a, &[1.0, 1.0, 1.0]);
        assert!(out.iter().all(|&v| v.abs() < 1e-6), /* no mappings means zero output */);
    }
}
