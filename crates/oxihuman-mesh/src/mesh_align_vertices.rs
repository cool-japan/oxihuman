// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Align selected vertices to a world axis, average, or custom plane.

/// The axis to align to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignAxisAV {
    X,
    Y,
    Z,
}

impl AlignAxisAV {
    /// Returns the axis index (0 = X, 1 = Y, 2 = Z).
    pub fn index(self) -> usize {
        match self {
            AlignAxisAV::X => 0,
            AlignAxisAV::Y => 1,
            AlignAxisAV::Z => 2,
        }
    }
}

/// Result of the align-vertices operation.
#[derive(Debug, Clone, Default)]
pub struct AlignVerticesResult {
    pub vertices_aligned: usize,
    pub target_value: f32,
    pub axis: usize,
}

/// Computes the average coordinate along the given axis for the selected vertices.
pub fn average_along_axis(positions: &[[f32; 3]], vertex_ids: &[u32], axis: AlignAxisAV) -> f32 {
    if vertex_ids.is_empty() {
        return 0.0;
    }
    let ai = axis.index();
    let mut sum = 0.0f32;
    let mut count = 0usize;
    for &vi in vertex_ids {
        let vi = vi as usize;
        if vi < positions.len() {
            sum += positions[vi][ai];
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        sum / count as f32
    }
}

/// Aligns selected vertices to a fixed coordinate value along an axis.
pub fn align_to_value(
    positions: &mut [[f32; 3]],
    vertex_ids: &[u32],
    axis: AlignAxisAV,
    value: f32,
) -> usize {
    let ai = axis.index();
    let mut count = 0usize;
    for &vi in vertex_ids {
        let vi = vi as usize;
        if vi < positions.len() {
            positions[vi][ai] = value;
            count += 1;
        }
    }
    count
}

/// Aligns selected vertices to their average position along the given axis.
pub fn align_to_average(
    positions: &mut [[f32; 3]],
    vertex_ids: &[u32],
    axis: AlignAxisAV,
) -> AlignVerticesResult {
    let avg = average_along_axis(positions, vertex_ids, axis);
    let aligned = align_to_value(positions, vertex_ids, axis, avg);
    AlignVerticesResult {
        vertices_aligned: aligned,
        target_value: avg,
        axis: axis.index(),
    }
}

/// Aligns all selected vertices to the minimum coordinate along the axis.
pub fn align_to_min(
    positions: &mut [[f32; 3]],
    vertex_ids: &[u32],
    axis: AlignAxisAV,
) -> AlignVerticesResult {
    let ai = axis.index();
    let min_val = vertex_ids
        .iter()
        .filter_map(|&vi| {
            let vi = vi as usize;
            if vi < positions.len() {
                Some(positions[vi][ai])
            } else {
                None
            }
        })
        .fold(f32::INFINITY, f32::min);
    if min_val.is_infinite() {
        return AlignVerticesResult::default();
    }
    let aligned = align_to_value(positions, vertex_ids, axis, min_val);
    AlignVerticesResult {
        vertices_aligned: aligned,
        target_value: min_val,
        axis: ai,
    }
}

/// Aligns all selected vertices to the maximum coordinate along the axis.
pub fn align_to_max(
    positions: &mut [[f32; 3]],
    vertex_ids: &[u32],
    axis: AlignAxisAV,
) -> AlignVerticesResult {
    let ai = axis.index();
    let max_val = vertex_ids
        .iter()
        .filter_map(|&vi| {
            let vi = vi as usize;
            if vi < positions.len() {
                Some(positions[vi][ai])
            } else {
                None
            }
        })
        .fold(f32::NEG_INFINITY, f32::max);
    if max_val.is_infinite() {
        return AlignVerticesResult::default();
    }
    let aligned = align_to_value(positions, vertex_ids, axis, max_val);
    AlignVerticesResult {
        vertices_aligned: aligned,
        target_value: max_val,
        axis: ai,
    }
}

/// Returns the bounding range [min, max] of selected vertices along an axis.
pub fn vertex_range(
    positions: &[[f32; 3]],
    vertex_ids: &[u32],
    axis: AlignAxisAV,
) -> Option<(f32, f32)> {
    let ai = axis.index();
    let vals: Vec<f32> = vertex_ids
        .iter()
        .filter_map(|&vi| {
            let vi = vi as usize;
            if vi < positions.len() {
                Some(positions[vi][ai])
            } else {
                None
            }
        })
        .collect();
    if vals.is_empty() {
        return None;
    }
    let min = vals.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_verts() -> Vec<[f32; 3]> {
        vec![[0.0, 1.0, 0.0], [2.0, 3.0, 0.0], [4.0, 5.0, 0.0]]
    }

    #[test]
    fn average_along_x() {
        let pos = three_verts();
        let avg = average_along_axis(&pos, &[0, 1, 2], AlignAxisAV::X);
        assert!((avg - 2.0).abs() < 1e-5);
    }

    #[test]
    fn align_to_value_sets_all() {
        let mut pos = three_verts();
        let count = align_to_value(&mut pos, &[0, 1, 2], AlignAxisAV::Y, 0.0);
        assert_eq!(count, 3);
        for p in &pos {
            assert!((p[1]).abs() < 1e-5);
        }
    }

    #[test]
    fn align_to_average_result() {
        let mut pos = three_verts();
        let res = align_to_average(&mut pos, &[0, 1, 2], AlignAxisAV::X);
        assert_eq!(res.vertices_aligned, 3);
        assert!((res.target_value - 2.0).abs() < 1e-5);
    }

    #[test]
    fn align_to_min_result() {
        let mut pos = three_verts();
        let res = align_to_min(&mut pos, &[0, 1, 2], AlignAxisAV::X);
        assert!((res.target_value - 0.0).abs() < 1e-5);
    }

    #[test]
    fn align_to_max_result() {
        let mut pos = three_verts();
        let res = align_to_max(&mut pos, &[0, 1, 2], AlignAxisAV::X);
        assert!((res.target_value - 4.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_range_correct() {
        let pos = three_verts();
        let (min, max) = vertex_range(&pos, &[0, 1, 2], AlignAxisAV::X).expect("should succeed");
        assert!((min - 0.0).abs() < 1e-5);
        assert!((max - 4.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_range_empty() {
        let pos = three_verts();
        let r = vertex_range(&pos, &[], AlignAxisAV::X);
        assert!(r.is_none());
    }

    #[test]
    fn axis_index_correct() {
        assert_eq!(AlignAxisAV::X.index(), 0);
        assert_eq!(AlignAxisAV::Y.index(), 1);
        assert_eq!(AlignAxisAV::Z.index(), 2);
    }

    #[test]
    fn align_out_of_bounds_skipped() {
        let mut pos = three_verts();
        let count = align_to_value(&mut pos, &[99], AlignAxisAV::X, 0.0);
        assert_eq!(count, 0);
    }
}
