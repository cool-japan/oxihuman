// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Surface deform binding stub.

/// A surface deform binding entry for one vertex.
#[derive(Debug, Clone)]
pub struct SurfaceDeformBinding {
    pub face_index: usize,
    pub barycentric: [f32; 3],
    pub normal_offset: f32,
}

/// Surface deform deformer state.
#[derive(Debug, Clone)]
pub struct SurfaceDeform {
    pub bindings: Vec<SurfaceDeformBinding>,
    pub strength: f32,
    pub is_bound: bool,
}

impl SurfaceDeform {
    pub fn new(vertex_count: usize) -> Self {
        SurfaceDeform {
            bindings: (0..vertex_count)
                .map(|_| SurfaceDeformBinding {
                    face_index: 0,
                    barycentric: [1.0 / 3.0; 3],
                    normal_offset: 0.0,
                })
                .collect(),
            strength: 1.0,
            is_bound: false,
        }
    }
}

/// Create a new surface deform.
pub fn new_surface_deform(vertex_count: usize) -> SurfaceDeform {
    SurfaceDeform::new(vertex_count)
}

/// Bind the deformer (marks it as ready).
pub fn surface_deform_bind(sd: &mut SurfaceDeform) {
    sd.is_bound = true;
}

/// Unbind the deformer.
pub fn surface_deform_unbind(sd: &mut SurfaceDeform) {
    sd.is_bound = false;
}

/// Set the strength.
pub fn surface_deform_set_strength(sd: &mut SurfaceDeform, strength: f32) {
    sd.strength = strength.clamp(0.0, 1.0);
}

/// Return the vertex count.
pub fn surface_deform_vertex_count(sd: &SurfaceDeform) -> usize {
    sd.bindings.len()
}

/// Return a JSON-like string.
pub fn surface_deform_to_json(sd: &SurfaceDeform) -> String {
    format!(
        r#"{{"vertices":{},"strength":{:.4},"bound":{}}}"#,
        sd.bindings.len(),
        sd.strength,
        sd.is_bound
    )
}

/// Compute barycentric weights sum (should be ~1.0 for valid binding).
pub fn surface_deform_bary_sum(sd: &SurfaceDeform, binding_index: usize) -> f32 {
    if binding_index < sd.bindings.len() {
        let b = &sd.bindings[binding_index];
        b.barycentric[0] + b.barycentric[1] + b.barycentric[2]
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_surface_deform_vertex_count() {
        let sd = new_surface_deform(15);
        assert_eq!(
            surface_deform_vertex_count(&sd),
            15, /* vertex count must match */
        );
    }

    #[test]
    fn test_initial_not_bound() {
        let sd = new_surface_deform(5);
        assert!(!sd.is_bound /* should not be bound initially */,);
    }

    #[test]
    fn test_bind_sets_is_bound() {
        let mut sd = new_surface_deform(5);
        surface_deform_bind(&mut sd);
        assert!(sd.is_bound /* bind should set is_bound */,);
    }

    #[test]
    fn test_unbind_clears_is_bound() {
        let mut sd = new_surface_deform(5);
        surface_deform_bind(&mut sd);
        surface_deform_unbind(&mut sd);
        assert!(!sd.is_bound /* unbind should clear is_bound */,);
    }

    #[test]
    fn test_set_strength_clamps() {
        let mut sd = new_surface_deform(2);
        surface_deform_set_strength(&mut sd, 3.0);
        assert!((sd.strength - 1.0).abs() < 1e-5, /* strength clamped to 1 */);
    }

    #[test]
    fn test_set_strength_negative_clamps() {
        let mut sd = new_surface_deform(2);
        surface_deform_set_strength(&mut sd, -1.0);
        assert!((sd.strength).abs() < 1e-6, /* negative strength clamped to 0 */);
    }

    #[test]
    fn test_bary_sum_near_one() {
        let sd = new_surface_deform(3);
        let s = surface_deform_bary_sum(&sd, 0);
        assert!((s - 1.0).abs() < 1e-5, /* barycentric weights should sum to 1 */);
    }

    #[test]
    fn test_bary_sum_out_of_bounds() {
        let sd = new_surface_deform(2);
        let s = surface_deform_bary_sum(&sd, 99);
        assert!((s).abs() < 1e-6 /* out-of-bounds returns 0 */,);
    }

    #[test]
    fn test_to_json_contains_bound() {
        let sd = new_surface_deform(3);
        let j = surface_deform_to_json(&sd);
        assert!(j.contains("bound") /* JSON must contain bound key */,);
    }

    #[test]
    fn test_default_strength_one() {
        let sd = new_surface_deform(1);
        assert!((sd.strength - 1.0).abs() < 1e-5, /* default strength is 1.0 */);
    }
}
