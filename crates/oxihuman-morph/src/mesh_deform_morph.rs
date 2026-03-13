// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh deform cage morph target stub.

/// A mesh deform binding for one driven vertex.
#[derive(Debug, Clone)]
pub struct MeshDeformBinding {
    pub cage_vertex_indices: [usize; 4],
    pub weights: [f32; 4],
}

impl Default for MeshDeformBinding {
    fn default() -> Self {
        MeshDeformBinding {
            cage_vertex_indices: [0, 1, 2, 3],
            weights: [0.25; 4],
        }
    }
}

/// Mesh deform morph state.
#[derive(Debug, Clone)]
pub struct MeshDeformMorph {
    pub bindings: Vec<MeshDeformBinding>,
    pub blend_weight: f32,
    pub bound: bool,
}

impl MeshDeformMorph {
    pub fn new(vertex_count: usize) -> Self {
        MeshDeformMorph {
            bindings: (0..vertex_count)
                .map(|_| MeshDeformBinding::default())
                .collect(),
            blend_weight: 0.0,
            bound: false,
        }
    }
}

/// Create a new mesh deform morph.
pub fn new_mesh_deform_morph(vertex_count: usize) -> MeshDeformMorph {
    MeshDeformMorph::new(vertex_count)
}

/// Set the blend weight.
pub fn mdm_set_weight(mdm: &mut MeshDeformMorph, weight: f32) {
    mdm.blend_weight = weight.clamp(0.0, 1.0);
}

/// Bind the deformer.
pub fn mdm_bind(mdm: &mut MeshDeformMorph) {
    mdm.bound = true;
}

/// Unbind the deformer.
pub fn mdm_unbind(mdm: &mut MeshDeformMorph) {
    mdm.bound = false;
}

/// Return vertex count.
pub fn mdm_vertex_count(mdm: &MeshDeformMorph) -> usize {
    mdm.bindings.len()
}

/// Return a JSON-like string.
pub fn mdm_to_json(mdm: &MeshDeformMorph) -> String {
    format!(
        r#"{{"vertices":{},"weight":{:.4},"bound":{}}}"#,
        mdm.bindings.len(),
        mdm.blend_weight,
        mdm.bound
    )
}

/// Check that all binding weights sum to approximately 1.0.
pub fn mdm_validate_weights(mdm: &MeshDeformMorph) -> bool {
    mdm.bindings
        .iter()
        .all(|b| (b.weights.iter().sum::<f32>() - 1.0).abs() < 1e-4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mdm_vertex_count() {
        let m = new_mesh_deform_morph(8);
        assert_eq!(mdm_vertex_count(&m), 8 /* vertex count must match */,);
    }

    #[test]
    fn test_initial_weight_zero() {
        let m = new_mesh_deform_morph(3);
        assert!((m.blend_weight).abs() < 1e-6, /* initial weight should be 0 */);
    }

    #[test]
    fn test_set_weight_clamps() {
        let mut m = new_mesh_deform_morph(2);
        mdm_set_weight(&mut m, 5.0);
        assert!((m.blend_weight - 1.0).abs() < 1e-5, /* weight clamped to 1 */);
    }

    #[test]
    fn test_bind_sets_bound() {
        let mut m = new_mesh_deform_morph(2);
        mdm_bind(&mut m);
        assert!(m.bound /* bind should set bound flag */,);
    }

    #[test]
    fn test_unbind_clears_bound() {
        let mut m = new_mesh_deform_morph(2);
        mdm_bind(&mut m);
        mdm_unbind(&mut m);
        assert!(!m.bound /* unbind should clear bound flag */,);
    }

    #[test]
    fn test_initial_weights_valid() {
        let m = new_mesh_deform_morph(4);
        assert!(mdm_validate_weights(&m), /* default weights should sum to 1 */);
    }

    #[test]
    fn test_to_json_contains_weight() {
        let m = new_mesh_deform_morph(3);
        let j = mdm_to_json(&m);
        assert!(j.contains("weight") /* JSON must contain weight */,);
    }

    #[test]
    fn test_to_json_contains_bound() {
        let m = new_mesh_deform_morph(3);
        let j = mdm_to_json(&m);
        assert!(j.contains("bound") /* JSON must contain bound */,);
    }

    #[test]
    fn test_set_weight_negative_clamps() {
        let mut m = new_mesh_deform_morph(2);
        mdm_set_weight(&mut m, -1.0);
        assert!((m.blend_weight).abs() < 1e-6, /* negative weight clamped to 0 */);
    }

    #[test]
    fn test_initial_not_bound() {
        let m = new_mesh_deform_morph(5);
        assert!(!m.bound /* should not be bound initially */,);
    }
}
