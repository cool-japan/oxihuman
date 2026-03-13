// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ShearParams {
    pub factor: f32,
    pub axis: u8,
    pub shear_direction: u8,
}

pub fn new_shear_params(factor: f32, axis: u8, shear_dir: u8) -> ShearParams {
    ShearParams {
        factor,
        axis,
        shear_direction: shear_dir,
    }
}

pub fn shear_vertex(p: [f32; 3], params: &ShearParams) -> [f32; 3] {
    match (params.axis, params.shear_direction) {
        (2, 0) => [p[0] + params.factor * p[2], p[1], p[2]],
        (2, 1) => [p[0], p[1] + params.factor * p[2], p[2]],
        (1, 0) => [p[0] + params.factor * p[1], p[1], p[2]],
        (0, 1) => [p[0], p[1] + params.factor * p[0], p[2]],
        _ => [p[0] + params.factor * p[2], p[1], p[2]],
    }
}

pub fn shear_matrix_2x2(params: &ShearParams) -> [[f32; 2]; 2] {
    if params.shear_direction == 0 {
        [[1.0, params.factor], [0.0, 1.0]]
    } else {
        [[1.0, 0.0], [params.factor, 1.0]]
    }
}

pub fn shear_is_identity(params: &ShearParams) -> bool {
    params.factor.abs() < 1e-10
}

pub fn shear_area_ratio(params: &ShearParams) -> f32 {
    /* det of shear matrix is always 1 */
    let m = shear_matrix_2x2(params);
    m[0][0] * m[1][1] - m[0][1] * m[1][0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_shear_params() {
        /* factor and axes stored correctly */
        let p = new_shear_params(0.5, 2, 0);
        assert!((p.factor - 0.5).abs() < 1e-6);
        assert_eq!(p.axis, 2);
    }

    #[test]
    fn test_shear_vertex_x_along_z() {
        /* shear x by z */
        let params = new_shear_params(1.0, 2, 0);
        let v = [0.0f32, 0.0, 2.0];
        let out = shear_vertex(v, &params);
        assert!((out[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_shear_is_identity_true() {
        /* factor=0 is identity */
        let params = new_shear_params(0.0, 2, 0);
        assert!(shear_is_identity(&params));
    }

    #[test]
    fn test_shear_area_ratio_one() {
        /* shear preserves area (det=1) */
        let params = new_shear_params(3.7, 2, 0);
        assert!((shear_area_ratio(&params) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_shear_matrix_2x2_dir0() {
        /* direction 0: top-right off-diagonal */
        let params = new_shear_params(2.0, 2, 0);
        let m = shear_matrix_2x2(&params);
        assert!((m[0][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_shear_matrix_2x2_dir1() {
        /* direction 1: bottom-left off-diagonal */
        let params = new_shear_params(2.0, 2, 1);
        let m = shear_matrix_2x2(&params);
        assert!((m[1][0] - 2.0).abs() < 1e-6);
    }
}
