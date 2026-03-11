// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sparse (delta-only) blend shape stub.

/// A sparse delta entry: only stores non-zero vertex deltas.
#[derive(Debug, Clone)]
pub struct SparseDelta {
    pub vertex_index: u32,
    pub delta: [f32; 3],
}

/// Sparse blend shape storing only affected vertices.
#[derive(Debug, Clone)]
pub struct SparseBlendShape {
    pub name: String,
    pub deltas: Vec<SparseDelta>,
    pub total_vertex_count: usize,
    pub weight: f32,
    pub enabled: bool,
}

impl SparseBlendShape {
    pub fn new(name: impl Into<String>, total_vertex_count: usize) -> Self {
        SparseBlendShape {
            name: name.into(),
            deltas: Vec::new(),
            total_vertex_count,
            weight: 0.0,
            enabled: true,
        }
    }
}

/// Create a new sparse blend shape.
pub fn new_sparse_blend_shape(
    name: impl Into<String>,
    total_vertex_count: usize,
) -> SparseBlendShape {
    SparseBlendShape::new(name, total_vertex_count)
}

/// Add a delta entry.
pub fn sbs_add_delta(shape: &mut SparseBlendShape, delta: SparseDelta) {
    shape.deltas.push(delta);
}

/// Apply the sparse blend shape to a position buffer (stub: no-op).
pub fn sbs_apply(shape: &SparseBlendShape, positions: &mut [[f32; 3]]) {
    /* Stub: applies weighted deltas; currently no-op */
    for d in &shape.deltas {
        let idx = d.vertex_index as usize;
        if idx < positions.len() {
            positions[idx][0] += d.delta[0] * shape.weight;
            positions[idx][1] += d.delta[1] * shape.weight;
            positions[idx][2] += d.delta[2] * shape.weight;
        }
    }
}

/// Return delta count.
pub fn sbs_delta_count(shape: &SparseBlendShape) -> usize {
    shape.deltas.len()
}

/// Set shape weight.
pub fn sbs_set_weight(shape: &mut SparseBlendShape, weight: f32) {
    shape.weight = weight.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn sbs_set_enabled(shape: &mut SparseBlendShape, enabled: bool) {
    shape.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn sbs_to_json(shape: &SparseBlendShape) -> String {
    format!(
        r#"{{"name":"{}","delta_count":{},"weight":{},"enabled":{}}}"#,
        shape.name,
        shape.deltas.len(),
        shape.weight,
        shape.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_no_deltas() {
        let s = new_sparse_blend_shape("brow_up", 500);
        assert_eq!(sbs_delta_count(&s), 0 /* no deltas initially */,);
    }

    #[test]
    fn test_add_delta() {
        let mut s = new_sparse_blend_shape("brow_up", 500);
        sbs_add_delta(
            &mut s,
            SparseDelta {
                vertex_index: 10,
                delta: [0.0, 0.1, 0.0],
            },
        );
        assert_eq!(sbs_delta_count(&s), 1 /* one delta after add */,);
    }

    #[test]
    fn test_apply_with_weight() {
        let mut s = new_sparse_blend_shape("lift", 4);
        sbs_add_delta(
            &mut s,
            SparseDelta {
                vertex_index: 0,
                delta: [0.0, 1.0, 0.0],
            },
        );
        sbs_set_weight(&mut s, 0.5);
        let mut positions = vec![[0.0; 3]; 4];
        sbs_apply(&s, &mut positions);
        assert!((positions[0][1] - 0.5).abs() < 1e-5, /* Y must be displaced by weight */);
    }

    #[test]
    fn test_weight_clamped() {
        let mut s = new_sparse_blend_shape("x", 2);
        sbs_set_weight(&mut s, 2.0);
        assert!((s.weight - 1.0).abs() < 1e-6, /* weight must be clamped to 1.0 */);
    }

    #[test]
    fn test_weight_clamped_negative() {
        let mut s = new_sparse_blend_shape("x", 2);
        sbs_set_weight(&mut s, -0.5);
        assert!((s.weight).abs() < 1e-6, /* weight must be clamped to 0.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_sparse_blend_shape("x", 2);
        sbs_set_enabled(&mut s, false);
        assert!(!s.enabled /* enabled must be false */,);
    }

    #[test]
    fn test_to_json_contains_name() {
        let s = new_sparse_blend_shape("smile", 10);
        let j = sbs_to_json(&s);
        assert!(j.contains("smile") /* json must contain shape name */,);
    }

    #[test]
    fn test_apply_out_of_bounds_ignored() {
        let mut s = new_sparse_blend_shape("x", 2);
        sbs_add_delta(
            &mut s,
            SparseDelta {
                vertex_index: 999,
                delta: [1.0, 1.0, 1.0],
            },
        );
        sbs_set_weight(&mut s, 1.0);
        let mut positions = vec![[0.0; 3]; 2];
        sbs_apply(&s, &mut positions);
        assert!((positions[0][0]).abs() < 1e-6, /* out-of-bounds delta must be ignored */);
    }

    #[test]
    fn test_total_vertex_count() {
        let s = new_sparse_blend_shape("x", 128);
        assert_eq!(
            s.total_vertex_count,
            128, /* total vertex count must match */
        );
    }

    #[test]
    fn test_enabled_default() {
        let s = new_sparse_blend_shape("x", 2);
        assert!(s.enabled /* must be enabled by default */,);
    }
}
