// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lattice-based morph target deformer stub.

/// Lattice dimensions.
#[derive(Debug, Clone, Copy)]
pub struct LatticeDims {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl LatticeDims {
    pub fn total_points(&self) -> usize {
        self.x * self.y * self.z
    }
}

/// A lattice-based morph target.
#[derive(Debug, Clone)]
pub struct LatticeMorph {
    pub dims: LatticeDims,
    /// Lattice control point displacements [x, y, z].
    pub displacements: Vec<[f32; 3]>,
    pub weight: f32,
}

impl LatticeMorph {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        let dims = LatticeDims { x, y, z };
        let n = dims.total_points();
        LatticeMorph {
            dims,
            displacements: vec![[0.0; 3]; n],
            weight: 0.0,
        }
    }
}

/// Create a new lattice morph with given dimensions.
pub fn new_lattice_morph(x: usize, y: usize, z: usize) -> LatticeMorph {
    LatticeMorph::new(x, y, z)
}

/// Set a lattice control point displacement.
pub fn lattice_set_point(morph: &mut LatticeMorph, index: usize, disp: [f32; 3]) {
    if index < morph.displacements.len() {
        morph.displacements[index] = disp;
    }
}

/// Get a lattice control point displacement.
pub fn lattice_get_point(morph: &LatticeMorph, index: usize) -> [f32; 3] {
    if index < morph.displacements.len() {
        morph.displacements[index]
    } else {
        [0.0; 3]
    }
}

/// Set the blend weight.
pub fn lattice_set_weight(morph: &mut LatticeMorph, weight: f32) {
    morph.weight = weight.clamp(0.0, 1.0);
}

/// Return the total number of lattice control points.
pub fn lattice_point_count(morph: &LatticeMorph) -> usize {
    morph.dims.total_points()
}

/// Return a JSON-like string.
pub fn lattice_to_json(morph: &LatticeMorph) -> String {
    format!(
        r#"{{"dims":[{},{},{}],"weight":{:.4},"points":{}}}"#,
        morph.dims.x,
        morph.dims.y,
        morph.dims.z,
        morph.weight,
        morph.displacements.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lattice_correct_point_count() {
        let m = new_lattice_morph(2, 3, 4);
        assert_eq!(lattice_point_count(&m), 24 /* 2x3x4 = 24 points */,);
    }

    #[test]
    fn test_initial_weight_zero() {
        let m = new_lattice_morph(2, 2, 2);
        assert!((m.weight).abs() < 1e-6, /* initial weight should be 0 */);
    }

    #[test]
    fn test_set_point_updates_displacement() {
        let mut m = new_lattice_morph(2, 2, 2);
        lattice_set_point(&mut m, 0, [1.0, 2.0, 3.0]);
        let p = lattice_get_point(&m, 0);
        assert!((p[0] - 1.0).abs() < 1e-5 /* x component must match */,);
    }

    #[test]
    fn test_get_out_of_bounds_returns_zero() {
        let m = new_lattice_morph(2, 2, 2);
        let p = lattice_get_point(&m, 999);
        assert!((p[0]).abs() < 1e-6, /* out-of-bounds returns zero vector */);
    }

    #[test]
    fn test_set_weight_clamps() {
        let mut m = new_lattice_morph(2, 2, 2);
        lattice_set_weight(&mut m, 2.5);
        assert!((m.weight - 1.0).abs() < 1e-5, /* weight clamped to 1.0 */);
    }

    #[test]
    fn test_set_weight_negative_clamps() {
        let mut m = new_lattice_morph(2, 2, 2);
        lattice_set_weight(&mut m, -1.0);
        assert!((m.weight).abs() < 1e-6, /* negative weight clamped to 0 */);
    }

    #[test]
    fn test_to_json_contains_dims() {
        let m = new_lattice_morph(3, 4, 5);
        let j = lattice_to_json(&m);
        assert!(j.contains("dims") /* JSON should contain dims key */,);
    }

    #[test]
    fn test_dims_total_points() {
        let d = LatticeDims { x: 2, y: 3, z: 5 };
        assert_eq!(d.total_points(), 30 /* 2*3*5 = 30 */,);
    }

    #[test]
    fn test_set_multiple_points() {
        let mut m = new_lattice_morph(2, 2, 2);
        lattice_set_point(&mut m, 1, [0.0, 1.0, 0.0]);
        lattice_set_point(&mut m, 2, [0.0, 0.0, 1.0]);
        assert!((lattice_get_point(&m, 1)[1] - 1.0).abs() < 1e-5, /* y of point 1 */);
        assert!((lattice_get_point(&m, 2)[2] - 1.0).abs() < 1e-5, /* z of point 2 */);
    }

    #[test]
    fn test_displacements_initialized_zero() {
        let m = new_lattice_morph(3, 3, 3);
        for d in &m.displacements {
            assert!((d[0]).abs() < 1e-6, /* all displacements start at zero */);
        }
    }
}
