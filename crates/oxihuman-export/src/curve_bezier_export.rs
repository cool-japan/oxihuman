// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export cubic Bezier curve data.

use std::f32::consts::PI;

/// A cubic Bezier curve control point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BezierControlPoint {
    pub position: [f32; 3],
    pub handle_in: [f32; 3],
    pub handle_out: [f32; 3],
}

/// A cubic Bezier curve export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BezierCurveExport {
    pub control_points: Vec<BezierControlPoint>,
    pub closed: bool,
}

/// Create a new Bezier curve export.
#[allow(dead_code)]
pub fn new_bezier_curve_export(closed: bool) -> BezierCurveExport {
    BezierCurveExport {
        control_points: Vec::new(),
        closed,
    }
}

/// Add a control point.
#[allow(dead_code)]
pub fn add_bezier_cp(curve: &mut BezierCurveExport, cp: BezierControlPoint) {
    curve.control_points.push(cp);
}

/// Count control points.
#[allow(dead_code)]
pub fn bezier_cp_count(curve: &BezierCurveExport) -> usize {
    curve.control_points.len()
}

/// Evaluate the cubic Bezier at parameter t ∈ `[0,1]` between points i and i+1.
#[allow(dead_code)]
pub fn eval_bezier_segment(
    p0: [f32; 3],
    h_out: [f32; 3],
    h_in: [f32; 3],
    p1: [f32; 3],
    t: f32,
) -> [f32; 3] {
    let t2 = t * t;
    let t3 = t2 * t;
    let u = 1.0 - t;
    let u2 = u * u;
    let u3 = u2 * u;
    [
        u3 * p0[0] + 3.0 * u2 * t * h_out[0] + 3.0 * u * t2 * h_in[0] + t3 * p1[0],
        u3 * p0[1] + 3.0 * u2 * t * h_out[1] + 3.0 * u * t2 * h_in[1] + t3 * p1[1],
        u3 * p0[2] + 3.0 * u2 * t * h_out[2] + 3.0 * u * t2 * h_in[2] + t3 * p1[2],
    ]
}

/// Sample the curve at a given parameter (0.0 = start, 1.0 = end of entire curve).
#[allow(dead_code)]
pub fn sample_bezier_curve(curve: &BezierCurveExport, t: f32) -> [f32; 3] {
    let n = curve.control_points.len();
    if n == 0 {
        return [0.0; 3];
    }
    if n == 1 {
        return curve.control_points[0].position;
    }
    let seg_count = if curve.closed { n } else { n - 1 };
    if seg_count == 0 {
        return curve.control_points[0].position;
    }
    let t_clamped = t.clamp(0.0, 1.0) * seg_count as f32;
    let seg = (t_clamped as usize).min(seg_count - 1);
    let local_t = t_clamped - seg as f32;
    let i0 = seg;
    let i1 = (seg + 1) % n;
    let cp0 = &curve.control_points[i0];
    let cp1 = &curve.control_points[i1];
    eval_bezier_segment(
        cp0.position,
        cp0.handle_out,
        cp1.handle_in,
        cp1.position,
        local_t,
    )
}

/// Approximate arc length by sampling.
#[allow(dead_code)]
pub fn bezier_arc_length_approx(curve: &BezierCurveExport, samples: usize) -> f32 {
    if samples < 2 {
        return 0.0;
    }
    let mut prev = sample_bezier_curve(curve, 0.0);
    let mut total = 0.0f32;
    for i in 1..samples {
        let t = i as f32 / (samples - 1) as f32;
        let cur = sample_bezier_curve(curve, t);
        let dx = cur[0] - prev[0];
        let dy = cur[1] - prev[1];
        let dz = cur[2] - prev[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
        prev = cur;
    }
    total
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn bezier_curve_to_json(curve: &BezierCurveExport) -> String {
    format!(
        "{{\"control_points\":{},\"closed\":{}}}",
        curve.control_points.len(),
        curve.closed
    )
}

/// Convert degrees to radians (utility).
#[allow(dead_code)]
pub fn deg_to_rad_bc(deg: f32) -> f32 {
    deg * PI / 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_curve() -> BezierCurveExport {
        let mut c = new_bezier_curve_export(false);
        add_bezier_cp(
            &mut c,
            BezierControlPoint {
                position: [0.0, 0.0, 0.0],
                handle_in: [0.0, 0.0, 0.0],
                handle_out: [0.333, 0.0, 0.0],
            },
        );
        add_bezier_cp(
            &mut c,
            BezierControlPoint {
                position: [1.0, 0.0, 0.0],
                handle_in: [0.666, 0.0, 0.0],
                handle_out: [1.0, 0.0, 0.0],
            },
        );
        c
    }

    #[test]
    fn test_cp_count() {
        let c = straight_curve();
        assert_eq!(bezier_cp_count(&c), 2);
    }

    #[test]
    fn test_sample_at_zero() {
        let c = straight_curve();
        let p = sample_bezier_curve(&c, 0.0);
        assert!(p[0].abs() < 1e-5);
    }

    #[test]
    fn test_sample_at_one() {
        let c = straight_curve();
        let p = sample_bezier_curve(&c, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_arc_length_positive() {
        let c = straight_curve();
        let l = bezier_arc_length_approx(&c, 20);
        assert!(l > 0.5);
    }

    #[test]
    fn test_arc_length_approx_straight() {
        let c = straight_curve();
        let l = bezier_arc_length_approx(&c, 100);
        assert!((l - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_eval_segment_at_zero() {
        let p = eval_bezier_segment([0.0; 3], [0.0; 3], [0.0; 3], [1.0, 0.0, 0.0], 0.0);
        assert!(p[0].abs() < 1e-5);
    }

    #[test]
    fn test_bezier_curve_to_json() {
        let c = straight_curve();
        let j = bezier_curve_to_json(&c);
        assert!(j.contains("control_points"));
    }

    #[test]
    fn test_empty_curve() {
        let c = new_bezier_curve_export(false);
        let p = sample_bezier_curve(&c, 0.5);
        assert_eq!(p, [0.0; 3]);
    }

    #[test]
    fn test_deg_to_rad() {
        let r = deg_to_rad_bc(180.0);
        assert!((r - std::f32::consts::PI).abs() < 1e-5);
    }

    #[test]
    fn test_sample_clamps_above_one() {
        let c = straight_curve();
        let p0 = sample_bezier_curve(&c, 1.0);
        let p1 = sample_bezier_curve(&c, 2.0);
        assert!((p0[0] - p1[0]).abs() < 1e-4);
    }
}
