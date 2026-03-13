// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh warp: apply a displacement field to mesh vertices.

#[allow(dead_code)]
pub struct WarpField {
    pub displacements: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn new_warp_field(n: usize) -> WarpField {
    WarpField { displacements: vec![[0.0; 3]; n] }
}

#[allow(dead_code)]
pub fn wf_set(field: &mut WarpField, i: usize, d: [f32; 3]) {
    if i < field.displacements.len() {
        field.displacements[i] = d;
    }
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn wf_apply(positions: &mut [[f32; 3]], field: &WarpField) {
    let n = positions.len().min(field.displacements.len());
    for i in 0..n {
        positions[i][0] += field.displacements[i][0];
        positions[i][1] += field.displacements[i][1];
        positions[i][2] += field.displacements[i][2];
    }
}

#[allow(dead_code)]
pub fn wf_magnitude_max(field: &WarpField) -> f32 {
    field.displacements.iter().map(|d| {
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    }).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn wf_magnitude_avg(field: &WarpField) -> f32 {
    let n = field.displacements.len();
    if n == 0 { return 0.0; }
    let sum: f32 = field.displacements.iter().map(|d| {
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    }).sum();
    sum / n as f32
}

#[allow(dead_code)]
pub fn wf_scale(field: &mut WarpField, s: f32) {
    for d in &mut field.displacements {
        d[0] *= s; d[1] *= s; d[2] *= s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_field_no_movement() {
        let field = new_warp_field(3);
        let mut positions = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let before = positions.clone();
        wf_apply(&mut positions, &field);
        assert_eq!(positions, before);
    }

    #[test]
    fn test_set_and_apply() {
        let mut field = new_warp_field(2);
        wf_set(&mut field, 0, [1.0, 0.0, 0.0]);
        let mut positions = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        wf_apply(&mut positions, &field);
        assert!((positions[0][0] - 1.0).abs() < 1e-5);
        assert!((positions[1][0]).abs() < 1e-5);
    }

    #[test]
    fn test_magnitude_max() {
        let mut field = new_warp_field(2);
        wf_set(&mut field, 0, [3.0, 4.0, 0.0]);
        wf_set(&mut field, 1, [1.0, 0.0, 0.0]);
        assert!((wf_magnitude_max(&field) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_magnitude_avg() {
        let mut field = new_warp_field(2);
        wf_set(&mut field, 0, [3.0, 4.0, 0.0]);
        wf_set(&mut field, 1, [3.0, 4.0, 0.0]);
        assert!((wf_magnitude_avg(&field) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_scale() {
        let mut field = new_warp_field(1);
        wf_set(&mut field, 0, [2.0, 0.0, 0.0]);
        wf_scale(&mut field, 3.0);
        assert!((field.displacements[0][0] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_field_magnitude() {
        let field = new_warp_field(0);
        assert_eq!(wf_magnitude_max(&field), 0.0);
        assert_eq!(wf_magnitude_avg(&field), 0.0);
    }

    #[test]
    fn test_set_out_of_bounds_ignored() {
        let mut field = new_warp_field(2);
        wf_set(&mut field, 99, [1.0, 1.0, 1.0]);
        assert_eq!(wf_magnitude_max(&field), 0.0);
    }

    #[test]
    fn test_scale_zero() {
        let mut field = new_warp_field(2);
        wf_set(&mut field, 0, [5.0, 5.0, 5.0]);
        wf_scale(&mut field, 0.0);
        assert_eq!(wf_magnitude_max(&field), 0.0);
    }
}
