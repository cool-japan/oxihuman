// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh deformation along a Bezier or cubic spline path.

#[allow(dead_code)]
pub struct CurvePoint {
    pub position: [f32; 3],
    pub tangent: [f32; 3],
    pub up: [f32; 3],
}

#[allow(dead_code)]
pub struct BezierCurve {
    pub control_points: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub struct SplineDeformResult {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub enum CurveAxis {
    X,
    Y,
    Z,
}

/// Linearly interpolate between two 3D points.
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Evaluate a cubic Bezier curve at parameter `t` using de Casteljau's algorithm.
/// Uses the first 4 control points. Clamps `t` to [0, 1].
#[allow(dead_code)]
pub fn evaluate_bezier(curve: &BezierCurve, t: f32) -> [f32; 3] {
    if curve.control_points.len() < 4 {
        return curve.control_points.first().copied().unwrap_or([0.0; 3]);
    }
    let t = t.clamp(0.0, 1.0);
    let p0 = curve.control_points[0];
    let p1 = curve.control_points[1];
    let p2 = curve.control_points[2];
    let p3 = curve.control_points[3];
    // de Casteljau level 1
    let q0 = lerp3(p0, p1, t);
    let q1 = lerp3(p1, p2, t);
    let q2 = lerp3(p2, p3, t);
    // level 2
    let r0 = lerp3(q0, q1, t);
    let r1 = lerp3(q1, q2, t);
    // level 3
    lerp3(r0, r1, t)
}

/// First-derivative (tangent) of the cubic Bezier at `t`.
/// Returns a non-normalized direction vector.
#[allow(dead_code)]
pub fn bezier_tangent(curve: &BezierCurve, t: f32) -> [f32; 3] {
    if curve.control_points.len() < 4 {
        return [0.0, 0.0, 1.0];
    }
    let t = t.clamp(0.0, 1.0);
    let p0 = curve.control_points[0];
    let p1 = curve.control_points[1];
    let p2 = curve.control_points[2];
    let p3 = curve.control_points[3];
    // Derivative: 3 * [(1-t)^2*(p1-p0) + 2(1-t)t*(p2-p1) + t^2*(p3-p2)]
    let u = 1.0 - t;
    let d0 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let d1 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
    let d2 = [p3[0] - p2[0], p3[1] - p2[1], p3[2] - p2[2]];
    let c0 = 3.0 * u * u;
    let c1 = 6.0 * u * t;
    let c2 = 3.0 * t * t;
    [
        c0 * d0[0] + c1 * d1[0] + c2 * d2[0],
        c0 * d0[1] + c1 * d1[1] + c2 * d2[1],
        c0 * d0[2] + c1 * d1[2] + c2 * d2[2],
    ]
}

/// Approximate arc length of the Bezier curve by summing `samples` chord segments.
#[allow(dead_code)]
pub fn bezier_arc_length(curve: &BezierCurve, samples: usize) -> f32 {
    if samples < 2 {
        return 0.0;
    }
    let mut length = 0.0_f32;
    let mut prev = evaluate_bezier(curve, 0.0);
    for i in 1..=samples {
        let t = i as f32 / samples as f32;
        let curr = evaluate_bezier(curve, t);
        let d = [curr[0] - prev[0], curr[1] - prev[1], curr[2] - prev[2]];
        length += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        prev = curr;
    }
    length
}

/// Sample `n` evenly-spaced Frenet frames along the curve.
#[allow(dead_code)]
pub fn sample_curve_points(curve: &BezierCurve, n: usize) -> Vec<CurvePoint> {
    if n == 0 {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(n);
    let world_up = [0.0_f32, 1.0, 0.0];
    for i in 0..n {
        let t = if n == 1 {
            0.0
        } else {
            i as f32 / (n - 1) as f32
        };
        let position = evaluate_bezier(curve, t);
        let tangent = normalize3(bezier_tangent(curve, t));
        let right = normalize3(cross3(tangent, world_up));
        let up = if right[0].abs() + right[1].abs() + right[2].abs() < 1e-6 {
            world_up
        } else {
            normalize3(cross3(right, tangent))
        };
        result.push(CurvePoint {
            position,
            tangent,
            up,
        });
    }
    result
}

/// Deform mesh positions and normals along the Bezier curve.
/// `axis` selects which coordinate drives the `t` parameter along the curve.
/// `strength` blends between original (0) and fully deformed (1).
#[allow(dead_code)]
pub fn deform_mesh_along_curve(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    curve: &BezierCurve,
    axis: CurveAxis,
    strength: f32,
) -> SplineDeformResult {
    if positions.is_empty() {
        return SplineDeformResult {
            positions: Vec::new(),
            normals: Vec::new(),
        };
    }

    // Compute axis range for normalization.
    let axis_vals: Vec<f32> = positions
        .iter()
        .map(|p| match axis {
            CurveAxis::X => p[0],
            CurveAxis::Y => p[1],
            CurveAxis::Z => p[2],
        })
        .collect();
    let min_val = axis_vals.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = axis_vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_val - min_val).max(1e-8);

    let mut out_positions = Vec::with_capacity(positions.len());
    let mut out_normals = Vec::with_capacity(normals.len());
    let world_up = [0.0_f32, 1.0, 0.0];

    for (pos, nor) in positions.iter().zip(normals.iter()) {
        let axis_val = match axis {
            CurveAxis::X => pos[0],
            CurveAxis::Y => pos[1],
            CurveAxis::Z => pos[2],
        };
        let t = ((axis_val - min_val) / range).clamp(0.0, 1.0);
        let curve_pos = evaluate_bezier(curve, t);
        let tangent = normalize3(bezier_tangent(curve, t));
        let right = normalize3(cross3(tangent, world_up));
        let up = if right[0].abs() + right[1].abs() + right[2].abs() < 1e-6 {
            world_up
        } else {
            normalize3(cross3(right, tangent))
        };

        // Lateral offset from the axis-coordinate
        let lateral_u = match axis {
            CurveAxis::X => pos[1],
            CurveAxis::Y => pos[0],
            CurveAxis::Z => pos[0],
        };
        let lateral_v = match axis {
            CurveAxis::X => pos[2],
            CurveAxis::Y => pos[2],
            CurveAxis::Z => pos[1],
        };

        let deformed = [
            curve_pos[0] + right[0] * lateral_u + up[0] * lateral_v,
            curve_pos[1] + right[1] * lateral_u + up[1] * lateral_v,
            curve_pos[2] + right[2] * lateral_u + up[2] * lateral_v,
        ];
        let out_pos = [
            pos[0] + (deformed[0] - pos[0]) * strength,
            pos[1] + (deformed[1] - pos[1]) * strength,
            pos[2] + (deformed[2] - pos[2]) * strength,
        ];

        // Rotate normal to follow the curve frame.
        let deformed_nor = normalize3([
            tangent[0] * nor[2] + right[0] * nor[0] + up[0] * nor[1],
            tangent[1] * nor[2] + right[1] * nor[0] + up[1] * nor[1],
            tangent[2] * nor[2] + right[2] * nor[0] + up[2] * nor[1],
        ]);
        let out_nor = [
            nor[0] + (deformed_nor[0] - nor[0]) * strength,
            nor[1] + (deformed_nor[1] - nor[1]) * strength,
            nor[2] + (deformed_nor[2] - nor[2]) * strength,
        ];
        out_positions.push(out_pos);
        out_normals.push(normalize3(out_nor));
    }

    SplineDeformResult {
        positions: out_positions,
        normals: out_normals,
    }
}

/// Find the parameter `t` in `[0,1]` for which the curve point is closest
/// to the given `point`, projected along `axis`.
#[allow(dead_code)]
pub fn project_to_curve_param(point: [f32; 3], curve: &BezierCurve, axis: CurveAxis) -> f32 {
    let axis_val = match axis {
        CurveAxis::X => point[0],
        CurveAxis::Y => point[1],
        CurveAxis::Z => point[2],
    };
    // Binary search / linear search over 100 samples
    let samples = 100usize;
    let mut best_t = 0.0_f32;
    let mut best_dist = f32::INFINITY;
    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        let cp = evaluate_bezier(curve, t);
        let cv = match axis {
            CurveAxis::X => cp[0],
            CurveAxis::Y => cp[1],
            CurveAxis::Z => cp[2],
        };
        let d = (cv - axis_val).abs();
        if d < best_dist {
            best_dist = d;
            best_t = t;
        }
    }
    best_t
}

/// Normalize a 3D vector to unit length. Returns `[0,0,1]` for near-zero input.
#[allow(dead_code)]
pub fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Compute the cross product of two 3D vectors.
#[allow(dead_code)]
pub fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
fn make_test_curve() -> BezierCurve {
    BezierCurve {
        control_points: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier_at_t0_is_first_control_point() {
        let curve = make_test_curve();
        let p = evaluate_bezier(&curve, 0.0);
        assert!((p[0] - 0.0).abs() < 1e-5 && (p[1]).abs() < 1e-5 && (p[2]).abs() < 1e-5);
    }

    #[test]
    fn bezier_at_t1_is_last_control_point() {
        let curve = make_test_curve();
        let p = evaluate_bezier(&curve, 1.0);
        assert!((p[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn bezier_at_t05_is_midpoint_for_linear_curve() {
        let curve = make_test_curve();
        let p = evaluate_bezier(&curve, 0.5);
        // For evenly-spaced collinear points, midpoint should be at x=1.5
        assert!((p[0] - 1.5).abs() < 1e-4);
    }

    #[test]
    fn bezier_arc_length_positive() {
        let curve = make_test_curve();
        let len = bezier_arc_length(&curve, 100);
        assert!(len > 0.0);
    }

    #[test]
    fn bezier_arc_length_linear_curve_approx_correct() {
        let curve = make_test_curve();
        let len = bezier_arc_length(&curve, 1000);
        // Collinear points 0..3, so length ≈ 3
        assert!((len - 3.0).abs() < 0.1);
    }

    #[test]
    fn sample_curve_points_count_correct() {
        let curve = make_test_curve();
        let pts = sample_curve_points(&curve, 7);
        assert_eq!(pts.len(), 7);
    }

    #[test]
    fn sample_curve_points_empty() {
        let curve = make_test_curve();
        let pts = sample_curve_points(&curve, 0);
        assert!(pts.is_empty());
    }

    #[test]
    fn deform_mesh_changes_positions_with_nonzero_strength() {
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let normals = vec![[0.0f32, 0.0, 1.0]; 3];
        let curve = BezierCurve {
            control_points: vec![
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, -1.0, 0.0],
                [3.0, 0.0, 0.0],
            ],
        };
        let result = deform_mesh_along_curve(&positions, &normals, &curve, CurveAxis::X, 1.0);
        // The middle point should move (curve is not straight)
        assert_ne!(result.positions[1][1], positions[1][1]);
    }

    #[test]
    fn deform_empty_mesh_returns_empty() {
        let curve = make_test_curve();
        let result = deform_mesh_along_curve(&[], &[], &curve, CurveAxis::X, 1.0);
        assert!(result.positions.is_empty());
        assert!(result.normals.is_empty());
    }

    #[test]
    fn deform_strength_zero_no_change() {
        let positions = vec![[0.0f32, 0.5, 0.0], [1.5, 0.5, 0.0], [3.0, 0.5, 0.0]];
        let normals = vec![[0.0f32, 0.0, 1.0]; 3];
        let curve = BezierCurve {
            control_points: vec![
                [0.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, -2.0, 0.0],
                [3.0, 0.0, 0.0],
            ],
        };
        let result = deform_mesh_along_curve(&positions, &normals, &curve, CurveAxis::X, 0.0);
        for (orig, out) in positions.iter().zip(result.positions.iter()) {
            assert!((orig[0] - out[0]).abs() < 1e-5);
            assert!((orig[1] - out[1]).abs() < 1e-5);
            assert!((orig[2] - out[2]).abs() < 1e-5);
        }
    }

    #[test]
    fn project_to_curve_param_t0_near_start() {
        let curve = make_test_curve();
        let t = project_to_curve_param([0.0, 100.0, 100.0], &curve, CurveAxis::X);
        assert!(t < 0.1, "expected t near 0, got {t}");
    }

    #[test]
    fn project_to_curve_param_t1_near_end() {
        let curve = make_test_curve();
        let t = project_to_curve_param([3.0, 0.0, 0.0], &curve, CurveAxis::X);
        assert!(t > 0.9, "expected t near 1, got {t}");
    }

    #[test]
    fn normalize3_unit_output() {
        let v = normalize3([3.0, 4.0, 0.0]);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cross3_standard_basis() {
        let x = [1.0_f32, 0.0, 0.0];
        let y = [0.0_f32, 1.0, 0.0];
        let z = cross3(x, y);
        assert!((z[0]).abs() < 1e-6);
        assert!((z[1]).abs() < 1e-6);
        assert!((z[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bezier_tangent_linear_curve_points_forward() {
        let curve = make_test_curve();
        let tan = bezier_tangent(&curve, 0.5);
        // For collinear points along X, tangent should point in +X
        assert!(tan[0] > 0.0);
    }
}
