// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

pub struct BendParams {
    pub angle_deg: f32,
    pub axis: u8,
    pub limit_min: f32,
    pub limit_max: f32,
}

pub fn new_bend_params(angle_deg: f32, axis: u8) -> BendParams {
    BendParams {
        angle_deg,
        axis,
        limit_min: -1.0,
        limit_max: 1.0,
    }
}

pub fn bend_vertex(p: [f32; 3], params: &BendParams) -> [f32; 3] {
    let angle_rad = params.angle_deg * PI / 180.0;
    match params.axis {
        0 => {
            // Bend along X axis: rotation in the YZ plane, proportional to x position.
            let t = ((p[0] - params.limit_min) / (params.limit_max - params.limit_min + 1e-10))
                .clamp(0.0, 1.0);
            let theta = t * angle_rad;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let y = p[1] * cos_t - p[2] * sin_t;
            let z = p[1] * sin_t + p[2] * cos_t;
            [p[0], y, z]
        }
        1 => {
            // Bend along Y axis: rotation in the XZ plane, proportional to y position.
            let t = ((p[1] - params.limit_min) / (params.limit_max - params.limit_min + 1e-10))
                .clamp(0.0, 1.0);
            let theta = t * angle_rad;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let x = p[0] * cos_t - p[2] * sin_t;
            let z = p[0] * sin_t + p[2] * cos_t;
            [x, p[1], z]
        }
        _ => {
            // Bend along Z axis (default / axis==2): rotation in XY plane,
            // proportional to z position.  This is the original formula.
            let t = ((p[2] - params.limit_min) / (params.limit_max - params.limit_min + 1e-10))
                .clamp(0.0, 1.0);
            let theta = t * angle_rad;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let x = p[0] * cos_t - p[1] * sin_t;
            let y = p[0] * sin_t + p[1] * cos_t;
            [x, y, p[2]]
        }
    }
}

pub fn bend_is_unlimited(params: &BendParams) -> bool {
    params.limit_min <= -1e6 && params.limit_max >= 1e6
}

pub fn bend_angle_at(params: &BendParams, t: f32) -> f32 {
    params.angle_deg * t.clamp(0.0, 1.0)
}

pub fn bend_curvature(params: &BendParams, length: f32) -> f32 {
    if length.abs() < 1e-10 {
        return 0.0;
    }
    params.angle_deg * PI / 180.0 / length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bend_params() {
        /* default limits set correctly */
        let p = new_bend_params(45.0, 2);
        assert!((p.angle_deg - 45.0).abs() < 1e-5);
        assert_eq!(p.axis, 2);
    }

    #[test]
    fn test_bend_vertex_zero_angle() {
        /* zero angle leaves vertex unchanged */
        let params = new_bend_params(0.0, 2);
        let v = [1.0f32, 0.0, 0.5];
        let out = bend_vertex(v, &params);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_bend_is_unlimited_false() {
        /* default limits are not unlimited */
        let params = new_bend_params(10.0, 0);
        assert!(!bend_is_unlimited(&params));
    }

    #[test]
    fn test_bend_is_unlimited_true() {
        /* very wide limits count as unlimited */
        let params = BendParams {
            angle_deg: 10.0,
            axis: 0,
            limit_min: -2e6,
            limit_max: 2e6,
        };
        assert!(bend_is_unlimited(&params));
    }

    #[test]
    fn test_bend_angle_at() {
        /* angle at t=0.5 is half the total */
        let params = new_bend_params(90.0, 2);
        let a = bend_angle_at(&params, 0.5);
        assert!((a - 45.0).abs() < 1e-5);
    }

    #[test]
    fn test_bend_curvature() {
        /* curvature scales with angle and inverse length */
        let params = new_bend_params(180.0, 2);
        let c = bend_curvature(&params, 1.0);
        assert!((c - PI).abs() < 1e-4);
    }

    #[test]
    fn test_bend_axis0_not_identity() {
        // 90° bend around X: at x=limit_max the YZ coords rotate
        let params = BendParams {
            angle_deg: 90.0,
            axis: 0,
            limit_min: 0.0,
            limit_max: 1.0,
        };
        let v = [1.0f32, 1.0, 0.0];
        let out = bend_vertex(v, &params);
        assert!(out != v, "axis=0 bend must differ from identity, got {out:?}");
    }

    #[test]
    fn test_bend_axis1_not_identity() {
        // 90° bend around Y: at y=limit_max the XZ coords rotate
        let params = BendParams {
            angle_deg: 90.0,
            axis: 1,
            limit_min: 0.0,
            limit_max: 1.0,
        };
        let v = [1.0f32, 1.0, 0.0];
        let out = bend_vertex(v, &params);
        assert!(out != v, "axis=1 bend must differ from identity, got {out:?}");
    }

    #[test]
    fn test_bend_axis0_differs_from_axis2() {
        let p0 = BendParams { angle_deg: 45.0, axis: 0, limit_min: 0.0, limit_max: 2.0 };
        let p2 = BendParams { angle_deg: 45.0, axis: 2, limit_min: 0.0, limit_max: 2.0 };
        let v = [1.0f32, 1.0, 1.0];
        assert!(
            bend_vertex(v, &p0) != bend_vertex(v, &p2),
            "axis=0 and axis=2 must produce different results"
        );
    }
}
