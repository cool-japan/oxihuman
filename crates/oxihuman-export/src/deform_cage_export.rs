// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Deformation cage export: a coarse proxy cage driving fine mesh deformation.

/// A cage control point.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageControlPoint {
    pub position: [f32; 3],
    pub weight: f32,
}

/// Deformation cage export structure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformCageExport {
    pub points: Vec<CageControlPoint>,
    pub faces: Vec<[u32; 4]>,
}

/// Create a new empty cage.
#[allow(dead_code)]
pub fn new_deform_cage() -> DeformCageExport {
    DeformCageExport {
        points: Vec::new(),
        faces: Vec::new(),
    }
}

/// Add control point.
#[allow(dead_code)]
pub fn add_cage_point(cage: &mut DeformCageExport, position: [f32; 3], weight: f32) {
    cage.points.push(CageControlPoint {
        position,
        weight: weight.max(0.0),
    });
}

/// Add a quad face (indices into points).
#[allow(dead_code)]
pub fn add_cage_face(cage: &mut DeformCageExport, face: [u32; 4]) {
    cage.faces.push(face);
}

/// Control point count.
#[allow(dead_code)]
pub fn cage_point_count(cage: &DeformCageExport) -> usize {
    cage.points.len()
}

/// Face count.
#[allow(dead_code)]
pub fn cage_face_count(cage: &DeformCageExport) -> usize {
    cage.faces.len()
}

/// Total weight sum.
#[allow(dead_code)]
pub fn cage_weight_sum(cage: &DeformCageExport) -> f32 {
    cage.points.iter().map(|p| p.weight).sum()
}

/// Normalize weights so they sum to 1.
#[allow(dead_code)]
pub fn normalize_cage_weights(cage: &mut DeformCageExport) {
    let sum = cage_weight_sum(cage);
    if sum < 1e-12 {
        return;
    }
    for p in &mut cage.points {
        p.weight /= sum;
    }
}

/// Validate: all face indices are valid.
#[allow(dead_code)]
pub fn validate_deform_cage(cage: &DeformCageExport) -> bool {
    let n = cage.points.len() as u32;
    cage.faces.iter().all(|f| f.iter().all(|&i| i < n))
}

/// Centroid of all control points.
#[allow(dead_code)]
pub fn cage_centroid(cage: &DeformCageExport) -> [f32; 3] {
    if cage.points.is_empty() {
        return [0.0; 3];
    }
    let n = cage.points.len() as f32;
    let sum = cage.points.iter().fold([0.0f32; 3], |acc, p| {
        [
            acc[0] + p.position[0],
            acc[1] + p.position[1],
            acc[2] + p.position[2],
        ]
    });
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Export to JSON.
#[allow(dead_code)]
pub fn deform_cage_to_json(cage: &DeformCageExport) -> String {
    format!(
        "{{\"point_count\":{},\"face_count\":{}}}",
        cage_point_count(cage),
        cage_face_count(cage)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_cage() -> DeformCageExport {
        let mut cage = new_deform_cage();
        add_cage_point(&mut cage, [0.0, 0.0, 0.0], 1.0);
        add_cage_point(&mut cage, [1.0, 0.0, 0.0], 1.0);
        add_cage_point(&mut cage, [1.0, 1.0, 0.0], 1.0);
        add_cage_point(&mut cage, [0.0, 1.0, 0.0], 1.0);
        add_cage_face(&mut cage, [0, 1, 2, 3]);
        cage
    }

    #[test]
    fn test_new_cage() {
        let cage = new_deform_cage();
        assert_eq!(cage_point_count(&cage), 0);
    }

    #[test]
    fn test_add_cage_point() {
        let cage = simple_cage();
        assert_eq!(cage_point_count(&cage), 4);
    }

    #[test]
    fn test_cage_face_count() {
        let cage = simple_cage();
        assert_eq!(cage_face_count(&cage), 1);
    }

    #[test]
    fn test_cage_weight_sum() {
        let cage = simple_cage();
        assert!((cage_weight_sum(&cage) - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_cage_weights() {
        let mut cage = simple_cage();
        normalize_cage_weights(&mut cage);
        assert!((cage_weight_sum(&cage) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_deform_cage() {
        let cage = simple_cage();
        assert!(validate_deform_cage(&cage));
    }

    #[test]
    fn test_validate_invalid_face() {
        let mut cage = new_deform_cage();
        add_cage_point(&mut cage, [0.0; 3], 1.0);
        add_cage_face(&mut cage, [0, 99, 0, 0]);
        assert!(!validate_deform_cage(&cage));
    }

    #[test]
    fn test_cage_centroid() {
        let cage = simple_cage();
        let c = cage_centroid(&cage);
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert!((c[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_deform_cage_to_json() {
        let cage = simple_cage();
        let j = deform_cage_to_json(&cage);
        assert!(j.contains("\"point_count\":4"));
    }

    #[test]
    fn test_negative_weight_clamped() {
        let mut cage = new_deform_cage();
        add_cage_point(&mut cage, [0.0; 3], -1.0);
        assert!((cage.points[0].weight).abs() < 1e-9);
    }
}
