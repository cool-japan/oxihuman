// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

pub struct TwistParams {
    pub angle_deg: f32,
    pub axis: u8,
    pub limit_min: f32,
    pub limit_max: f32,
}

pub fn new_twist_params(angle_deg: f32, axis: u8) -> TwistParams {
    TwistParams {
        angle_deg,
        axis,
        limit_min: -1.0,
        limit_max: 1.0,
    }
}

pub fn twist_vertex(p: [f32; 3], params: &TwistParams) -> [f32; 3] {
    let range = params.limit_max - params.limit_min;
    match params.axis {
        0 => {
            // Twist around X axis: rotate in YZ plane, angle proportional to x position.
            let t = if range.abs() < 1e-10 {
                0.5
            } else {
                ((p[0] - params.limit_min) / range).clamp(0.0, 1.0)
            };
            let theta = twist_angle_at(params, t) * PI / 180.0;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let y = p[1] * cos_t - p[2] * sin_t;
            let z = p[1] * sin_t + p[2] * cos_t;
            [p[0], y, z]
        }
        1 => {
            // Twist around Y axis: rotate in XZ plane, angle proportional to y position.
            let t = if range.abs() < 1e-10 {
                0.5
            } else {
                ((p[1] - params.limit_min) / range).clamp(0.0, 1.0)
            };
            let theta = twist_angle_at(params, t) * PI / 180.0;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let x = p[0] * cos_t - p[2] * sin_t;
            let z = p[0] * sin_t + p[2] * cos_t;
            [x, p[1], z]
        }
        _ => {
            // Twist around Z axis (default / axis==2): rotate in XY plane,
            // angle proportional to z position.  This is the original formula.
            let t = if range.abs() < 1e-10 {
                0.5
            } else {
                ((p[2] - params.limit_min) / range).clamp(0.0, 1.0)
            };
            let theta = twist_angle_at(params, t) * PI / 180.0;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let x = p[0] * cos_t - p[1] * sin_t;
            let y = p[0] * sin_t + p[1] * cos_t;
            [x, y, p[2]]
        }
    }
}

pub fn twist_angle_at(params: &TwistParams, t: f32) -> f32 {
    params.angle_deg * t.clamp(0.0, 1.0)
}

pub fn twist_is_zero(params: &TwistParams) -> bool {
    params.angle_deg.abs() < 1e-10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_twist_params() {
        /* stores angle and axis */
        let p = new_twist_params(90.0, 2);
        assert!((p.angle_deg - 90.0).abs() < 1e-5);
        assert_eq!(p.axis, 2);
    }

    #[test]
    fn test_twist_vertex_zero() {
        /* zero angle leaves x unchanged */
        let params = new_twist_params(0.0, 2);
        let v = [1.0f32, 0.0, 0.5];
        let out = twist_vertex(v, &params);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_twist_angle_at_half() {
        /* at t=0.5, angle is half total */
        let params = new_twist_params(180.0, 2);
        assert!((twist_angle_at(&params, 0.5) - 90.0).abs() < 1e-5);
    }

    #[test]
    fn test_twist_is_zero_true() {
        /* zero angle is zero */
        let params = new_twist_params(0.0, 2);
        assert!(twist_is_zero(&params));
    }

    #[test]
    fn test_twist_is_zero_false() {
        /* nonzero angle is not zero */
        let params = new_twist_params(1.0, 2);
        assert!(!twist_is_zero(&params));
    }

    #[test]
    fn test_twist_vertex_90deg_at_top() {
        /* at z=limit_max (t=1), 90-degree twist rotates (1,0) to (0,1) */
        let params = TwistParams {
            angle_deg: 90.0,
            axis: 2,
            limit_min: 0.0,
            limit_max: 1.0,
        };
        let v = [1.0f32, 0.0, 1.0];
        let out = twist_vertex(v, &params);
        assert!(out[0].abs() < 1e-5);
        assert!((out[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_twist_axis0_not_identity() {
        // Twist around X axis with 90° at x=limit_max: (0,1,0) → (0, 0, 1)
        let params = TwistParams {
            angle_deg: 90.0,
            axis: 0,
            limit_min: 0.0,
            limit_max: 1.0,
        };
        let v = [1.0f32, 1.0, 0.0];
        let out = twist_vertex(v, &params);
        // z-coordinate should have changed
        assert!(
            out != v,
            "axis=0 twist must differ from identity, got {out:?}"
        );
    }

    #[test]
    fn test_twist_axis1_not_identity() {
        // Twist around Y axis with 90° at y=limit_max: (1,1,0) → (0,1,1)
        let params = TwistParams {
            angle_deg: 90.0,
            axis: 1,
            limit_min: 0.0,
            limit_max: 1.0,
        };
        let v = [1.0f32, 1.0, 0.0];
        let out = twist_vertex(v, &params);
        assert!(
            out != v,
            "axis=1 twist must differ from identity, got {out:?}"
        );
    }

    #[test]
    fn test_twist_axis0_differs_from_axis2() {
        // Same angle/limits, different axis → different result for (1,1,1)
        let base = TwistParams {
            angle_deg: 45.0,
            axis: 2,
            limit_min: 0.0,
            limit_max: 2.0,
        };
        let ax0 = TwistParams { axis: 0, ..TwistParams { angle_deg: 45.0, limit_min: 0.0, limit_max: 2.0, axis: 0 } };
        let v = [1.0f32, 1.0, 1.0];
        let out2 = twist_vertex(v, &base);
        let out0 = twist_vertex(v, &ax0);
        assert!(out2 != out0, "axis=0 and axis=2 must produce different results");
    }
}
