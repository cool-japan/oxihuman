// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Slice mesh at multiple Z heights and return 2D contours.

/// A 2D contour from a Z-slice.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SliceContour {
    pub z: f32,
    pub points: Vec<[f32; 2]>,
}

/// Result of stack slicing.
#[allow(dead_code)]
pub struct SliceStackResult {
    pub contours: Vec<SliceContour>,
}

/// Intersect a triangle edge with a Z-plane. Returns None if no intersection.
fn edge_z_intersect(p0: [f32; 3], p1: [f32; 3], z: f32) -> Option<[f32; 2]> {
    let dz = p1[2] - p0[2];
    if dz.abs() < 1e-9 {
        return None;
    }
    let t = (z - p0[2]) / dz;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    let x = p0[0] + t * (p1[0] - p0[0]);
    let y = p0[1] + t * (p1[1] - p0[1]);
    Some([x, y])
}

/// Slice the mesh at a single Z height, returning intersection points.
#[allow(dead_code)]
pub fn slice_at_z(positions: &[[f32; 3]], indices: &[u32], z: f32) -> Vec<[f32; 2]> {
    let mut pts = Vec::new();
    let n_tri = indices.len() / 3;
    for t in 0..n_tri {
        let p0 = positions[indices[t * 3] as usize];
        let p1 = positions[indices[t * 3 + 1] as usize];
        let p2 = positions[indices[t * 3 + 2] as usize];
        for &(a, b) in &[(p0, p1), (p1, p2), (p2, p0)] {
            if let Some(pt) = edge_z_intersect(a, b, z) {
                pts.push(pt);
            }
        }
    }
    pts
}

/// Slice the mesh at multiple Z heights.
#[allow(dead_code)]
pub fn slice_stack(positions: &[[f32; 3]], indices: &[u32], z_heights: &[f32]) -> SliceStackResult {
    let contours = z_heights
        .iter()
        .map(|&z| SliceContour {
            z,
            points: slice_at_z(positions, indices, z),
        })
        .collect();
    SliceStackResult { contours }
}

/// Generate evenly-spaced Z heights between min and max.
#[allow(dead_code)]
pub fn uniform_z_heights(z_min: f32, z_max: f32, count: usize) -> Vec<f32> {
    if count == 0 {
        return vec![];
    }
    if count == 1 {
        return vec![(z_min + z_max) * 0.5];
    }
    let step = (z_max - z_min) / (count - 1) as f32;
    (0..count).map(|i| z_min + i as f32 * step).collect()
}

/// Total number of intersection points across all contours.
#[allow(dead_code)]
pub fn total_contour_points(result: &SliceStackResult) -> usize {
    result.contours.iter().map(|c| c.points.len()).sum()
}

/// Count non-empty contours.
#[allow(dead_code)]
pub fn non_empty_contour_count(result: &SliceStackResult) -> usize {
    result
        .contours
        .iter()
        .filter(|c| !c.points.is_empty())
        .count()
}

/// Bounding box of all intersection points in a contour.
#[allow(dead_code)]
pub fn contour_bounds(contour: &SliceContour) -> ([f32; 2], [f32; 2]) {
    if contour.points.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut mn = contour.points[0];
    let mut mx = contour.points[0];
    for p in &contour.points {
        mn[0] = mn[0].min(p[0]);
        mn[1] = mn[1].min(p[1]);
        mx[0] = mx[0].max(p[0]);
        mx[1] = mx[1].max(p[1]);
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 5, 1, 0, 4, 5, 2, 6, 3, 3, 6, 7, 1, 6, 2, 1, 5,
            6, 0, 3, 7, 0, 7, 4,
        ];
        (positions, indices)
    }

    #[test]
    fn slice_middle_has_points() {
        let (pos, idx) = unit_cube_mesh();
        let pts = slice_at_z(&pos, &idx, 0.5);
        assert!(!pts.is_empty());
    }

    #[test]
    fn slice_above_has_no_points() {
        let (pos, idx) = unit_cube_mesh();
        let pts = slice_at_z(&pos, &idx, 2.0);
        assert!(pts.is_empty());
    }

    #[test]
    fn slice_stack_contour_count() {
        let (pos, idx) = unit_cube_mesh();
        let heights = uniform_z_heights(0.1, 0.9, 5);
        let result = slice_stack(&pos, &idx, &heights);
        assert_eq!(result.contours.len(), 5);
    }

    #[test]
    fn total_contour_points_sum() {
        let (pos, idx) = unit_cube_mesh();
        let heights = uniform_z_heights(0.1, 0.9, 3);
        let result = slice_stack(&pos, &idx, &heights);
        let total = total_contour_points(&result);
        assert!(total > 0);
    }

    #[test]
    fn non_empty_contour_count_cube() {
        let (pos, idx) = unit_cube_mesh();
        let heights = vec![0.5, 1.5];
        let result = slice_stack(&pos, &idx, &heights);
        assert_eq!(non_empty_contour_count(&result), 1);
    }

    #[test]
    fn uniform_z_heights_count() {
        let h = uniform_z_heights(0.0, 1.0, 5);
        assert_eq!(h.len(), 5);
        assert!((h[0] - 0.0).abs() < 1e-5);
        assert!((h[4] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn uniform_z_heights_single() {
        let h = uniform_z_heights(0.0, 1.0, 1);
        assert_eq!(h.len(), 1);
        assert!((h[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn contour_bounds_nonempty() {
        let contour = SliceContour {
            z: 0.5,
            points: vec![[0.0, 0.0], [1.0, 0.5], [0.5, 1.0]],
        };
        let (mn, mx) = contour_bounds(&contour);
        assert!((mn[0] - 0.0).abs() < 1e-5);
        assert!((mx[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn edge_z_intersect_midpoint() {
        let r = edge_z_intersect([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5);
        assert!(r.is_some());
        let pt = r.expect("should succeed");
        assert!((pt[0] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn edge_z_intersect_none_outside() {
        let r = edge_z_intersect([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0);
        assert!(r.is_none());
    }
}
