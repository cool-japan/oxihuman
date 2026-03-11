// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cage (volumetric) morph target deformer stub.

/// A cage vertex used for volumetric morph deformation.
#[derive(Debug, Clone, Copy)]
pub struct CageVertex {
    pub position: [f32; 3],
    pub delta: [f32; 3],
}

/// A cage morph target.
#[derive(Debug, Clone)]
pub struct CageMorph {
    pub cage_vertices: Vec<CageVertex>,
    pub weight: f32,
}

impl CageMorph {
    pub fn new(vertex_count: usize) -> Self {
        CageMorph {
            cage_vertices: (0..vertex_count)
                .map(|_| CageVertex {
                    position: [0.0; 3],
                    delta: [0.0; 3],
                })
                .collect(),
            weight: 0.0,
        }
    }
}

/// Create a new cage morph with given vertex count.
pub fn new_cage_morph(vertex_count: usize) -> CageMorph {
    CageMorph::new(vertex_count)
}

/// Set the cage vertex position and delta.
pub fn cage_set_vertex(morph: &mut CageMorph, index: usize, position: [f32; 3], delta: [f32; 3]) {
    if index < morph.cage_vertices.len() {
        morph.cage_vertices[index] = CageVertex { position, delta };
    }
}

/// Set the blend weight.
pub fn cage_set_weight(morph: &mut CageMorph, weight: f32) {
    morph.weight = weight.clamp(0.0, 1.0);
}

/// Return a JSON-like string.
pub fn cage_to_json(morph: &CageMorph) -> String {
    format!(
        r#"{{"cage_vertices":{},"weight":{:.4}}}"#,
        morph.cage_vertices.len(),
        morph.weight
    )
}

/// Return the number of cage vertices.
pub fn cage_vertex_count(morph: &CageMorph) -> usize {
    morph.cage_vertices.len()
}

/// Return total delta magnitude (sum of delta lengths).
pub fn cage_total_delta_magnitude(morph: &CageMorph) -> f32 {
    morph
        .cage_vertices
        .iter()
        .map(|v| {
            (v.delta[0] * v.delta[0] + v.delta[1] * v.delta[1] + v.delta[2] * v.delta[2]).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cage_vertex_count() {
        let c = new_cage_morph(12);
        assert_eq!(cage_vertex_count(&c), 12 /* vertex count must match */,);
    }

    #[test]
    fn test_initial_weight_zero() {
        let c = new_cage_morph(5);
        assert!((c.weight).abs() < 1e-6, /* initial weight should be 0 */);
    }

    #[test]
    fn test_set_vertex_updates() {
        let mut c = new_cage_morph(4);
        cage_set_vertex(&mut c, 0, [1.0, 2.0, 3.0], [0.1, 0.2, 0.3]);
        let v = c.cage_vertices[0];
        assert!((v.position[0] - 1.0).abs() < 1e-5, /* position x must match */);
    }

    #[test]
    fn test_set_weight_clamps_to_one() {
        let mut c = new_cage_morph(2);
        cage_set_weight(&mut c, 5.0);
        assert!((c.weight - 1.0).abs() < 1e-5 /* weight clamped to 1 */,);
    }

    #[test]
    fn test_set_weight_negative_clamps() {
        let mut c = new_cage_morph(2);
        cage_set_weight(&mut c, -2.0);
        assert!((c.weight).abs() < 1e-6, /* negative weight clamped to 0 */);
    }

    #[test]
    fn test_to_json_contains_weight() {
        let c = new_cage_morph(3);
        let j = cage_to_json(&c);
        assert!(j.contains("weight") /* JSON must contain weight */,);
    }

    #[test]
    fn test_total_delta_zero_initially() {
        let c = new_cage_morph(6);
        assert!((cage_total_delta_magnitude(&c)).abs() < 1e-6, /* total delta is 0 initially */);
    }

    #[test]
    fn test_total_delta_after_set() {
        let mut c = new_cage_morph(2);
        cage_set_vertex(&mut c, 0, [0.0; 3], [1.0, 0.0, 0.0]);
        assert!(cage_total_delta_magnitude(&c) > 0.0, /* delta should be positive */);
    }

    #[test]
    fn test_set_out_of_bounds_ignored() {
        let mut c = new_cage_morph(2);
        cage_set_vertex(&mut c, 999, [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(
            cage_vertex_count(&c),
            2, /* vertex count should be unchanged */
        );
    }

    #[test]
    fn test_json_vertex_count() {
        let c = new_cage_morph(7);
        let j = cage_to_json(&c);
        assert!(j.contains("7") /* JSON should contain vertex count */,);
    }
}
