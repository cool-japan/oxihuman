// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct TaperParams {
    pub factor: f32,
    pub axis: u8,
    pub limit_min: f32,
    pub limit_max: f32,
}

pub fn new_taper_params(factor: f32, axis: u8) -> TaperParams {
    TaperParams {
        factor,
        axis,
        limit_min: -1.0,
        limit_max: 1.0,
    }
}

pub fn taper_vertex(p: [f32; 3], params: &TaperParams) -> [f32; 3] {
    let range = params.limit_max - params.limit_min;
    match params.axis {
        0 => {
            // Taper along X axis: scale Y and Z proportionally to x position.
            let t = if range.abs() < 1e-10 {
                0.5
            } else {
                ((p[0] - params.limit_min) / range).clamp(0.0, 1.0)
            };
            let scale = taper_scale_at(params, t);
            [p[0], p[1] * scale, p[2] * scale]
        }
        1 => {
            // Taper along Y axis: scale X and Z proportionally to y position.
            let t = if range.abs() < 1e-10 {
                0.5
            } else {
                ((p[1] - params.limit_min) / range).clamp(0.0, 1.0)
            };
            let scale = taper_scale_at(params, t);
            [p[0] * scale, p[1], p[2] * scale]
        }
        _ => {
            // Taper along Z axis (default / axis==2): scale X and Y proportionally
            // to z position.  This is the original formula.
            let t = if range.abs() < 1e-10 {
                0.5
            } else {
                ((p[2] - params.limit_min) / range).clamp(0.0, 1.0)
            };
            let scale = taper_scale_at(params, t);
            [p[0] * scale, p[1] * scale, p[2]]
        }
    }
}

pub fn taper_scale_at(params: &TaperParams, t: f32) -> f32 {
    1.0 + params.factor * t
}

pub fn taper_volume_ratio(params: &TaperParams) -> f32 {
    /* integral of (1 + factor*t)^2 from 0 to 1 */
    let f = params.factor;
    1.0 + f + f * f / 3.0
}

pub fn taper_is_uniform(params: &TaperParams) -> bool {
    params.factor.abs() < 1e-10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_taper_params() {
        /* factor and axis stored correctly */
        let p = new_taper_params(0.5, 2);
        assert!((p.factor - 0.5).abs() < 1e-6);
        assert_eq!(p.axis, 2);
    }

    #[test]
    fn test_taper_vertex_at_base() {
        /* at z=limit_min (t=0) scale is 1 */
        let params = new_taper_params(2.0, 2);
        let v = [1.0f32, 1.0, -1.0];
        let out = taper_vertex(v, &params);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_taper_scale_at_zero() {
        /* at t=0 scale is 1 regardless of factor */
        let params = new_taper_params(3.0, 2);
        assert!((taper_scale_at(&params, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_taper_scale_at_one() {
        /* at t=1 scale is 1+factor */
        let params = new_taper_params(0.5, 2);
        assert!((taper_scale_at(&params, 1.0) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_taper_is_uniform_true() {
        /* factor==0 means uniform */
        let params = new_taper_params(0.0, 2);
        assert!(taper_is_uniform(&params));
    }

    #[test]
    fn test_taper_volume_ratio_no_factor() {
        /* with factor=0, ratio is 1 */
        let params = new_taper_params(0.0, 2);
        assert!((taper_volume_ratio(&params) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_taper_axis0_scales_yz() {
        // Taper along X: at x=limit_max (t=1) scale = 1+factor, Y and Z are scaled.
        let params = TaperParams {
            factor: 1.0,
            axis: 0,
            limit_min: 0.0,
            limit_max: 1.0,
        };
        let v = [1.0f32, 1.0, 1.0];
        let out = taper_vertex(v, &params);
        // x unchanged
        assert!((out[0] - 1.0).abs() < 1e-5, "x must be unchanged");
        // y and z scaled by 2.0
        assert!((out[1] - 2.0).abs() < 1e-5, "y should be scaled by factor");
        assert!((out[2] - 2.0).abs() < 1e-5, "z should be scaled by factor");
    }

    #[test]
    fn test_taper_axis1_scales_xz() {
        // Taper along Y: at y=limit_max (t=1) scale = 1+factor, X and Z are scaled.
        let params = TaperParams {
            factor: 1.0,
            axis: 1,
            limit_min: 0.0,
            limit_max: 1.0,
        };
        let v = [1.0f32, 1.0, 1.0];
        let out = taper_vertex(v, &params);
        // y unchanged
        assert!((out[1] - 1.0).abs() < 1e-5, "y must be unchanged");
        // x and z scaled by 2.0
        assert!((out[0] - 2.0).abs() < 1e-5, "x should be scaled by factor");
        assert!((out[2] - 2.0).abs() < 1e-5, "z should be scaled by factor");
    }

    #[test]
    fn test_taper_axis0_differs_from_axis2() {
        let p0 = TaperParams { factor: 0.5, axis: 0, limit_min: 0.0, limit_max: 2.0 };
        let p2 = TaperParams { factor: 0.5, axis: 2, limit_min: 0.0, limit_max: 2.0 };
        let v = [1.0f32, 1.0, 1.0];
        assert!(
            taper_vertex(v, &p0) != taper_vertex(v, &p2),
            "axis=0 and axis=2 must produce different results"
        );
    }
}
