// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A NURBS curve for export.
#[allow(dead_code)]
pub struct NurbsCurveExport {
    pub name: String,
    pub degree: usize,
    pub control_points: Vec<[f32; 4]>, // x, y, z, w
    pub knots: Vec<f32>,
}

/// Create a new NURBS curve export.
#[allow(dead_code)]
pub fn new_nurbs_curve_export(name: &str, degree: usize) -> NurbsCurveExport {
    NurbsCurveExport {
        name: name.to_string(),
        degree,
        control_points: Vec::new(),
        knots: Vec::new(),
    }
}

/// Add a control point (homogeneous).
#[allow(dead_code)]
pub fn add_nurbs_control_point(curve: &mut NurbsCurveExport, p: [f32; 3], weight: f32) {
    curve.control_points.push([p[0], p[1], p[2], weight]);
}

/// Generate uniform knot vector.
#[allow(dead_code)]
pub fn generate_uniform_knots(curve: &mut NurbsCurveExport) {
    let n = curve.control_points.len();
    if n == 0 {
        return;
    }
    let d = curve.degree;
    let nk = n + d + 1;
    curve.knots = (0..nk).map(|i| i as f32 / (nk - 1) as f32).collect();
}

/// Count control points.
#[allow(dead_code)]
pub fn control_point_count_nurbs(curve: &NurbsCurveExport) -> usize {
    curve.control_points.len()
}

/// Validate NURBS curve (enough knots).
#[allow(dead_code)]
pub fn validate_nurbs(curve: &NurbsCurveExport) -> bool {
    let n = curve.control_points.len();
    if n == 0 {
        return false;
    }
    curve.knots.len() == n + curve.degree + 1
}

/// Compute centroid of control points (ignoring weight).
#[allow(dead_code)]
pub fn nurbs_centroid(curve: &NurbsCurveExport) -> [f32; 3] {
    if curve.control_points.is_empty() {
        return [0.0; 3];
    }
    let n = curve.control_points.len() as f32;
    let mut s = [0.0_f32; 3];
    for p in &curve.control_points {
        s[0] += p[0];
        s[1] += p[1];
        s[2] += p[2];
    }
    [s[0] / n, s[1] / n, s[2] / n]
}

/// Average weight of control points.
#[allow(dead_code)]
pub fn avg_weight(curve: &NurbsCurveExport) -> f32 {
    if curve.control_points.is_empty() {
        return 0.0;
    }
    curve.control_points.iter().map(|p| p[3]).sum::<f32>() / curve.control_points.len() as f32
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn nurbs_to_json(curve: &NurbsCurveExport) -> String {
    format!(
        r#"{{"name":"{}","degree":{},"control_points":{},"knots":{}}}"#,
        curve.name,
        curve.degree,
        curve.control_points.len(),
        curve.knots.len()
    )
}

/// Build a simple line NURBS (degree 1).
#[allow(dead_code)]
pub fn linear_nurbs_curve(name: &str, points: &[[f32; 3]]) -> NurbsCurveExport {
    let mut c = new_nurbs_curve_export(name, 1);
    for &p in points {
        add_nurbs_control_point(&mut c, p, 1.0);
    }
    generate_uniform_knots(&mut c);
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_curve() {
        let c = new_nurbs_curve_export("test", 3);
        assert_eq!(c.degree, 3);
    }

    #[test]
    fn add_control_points() {
        let mut c = new_nurbs_curve_export("c", 2);
        add_nurbs_control_point(&mut c, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(control_point_count_nurbs(&c), 1);
    }

    #[test]
    fn uniform_knots_count() {
        let mut c = new_nurbs_curve_export("c", 2);
        for _ in 0..4 {
            add_nurbs_control_point(&mut c, [0.0, 0.0, 0.0], 1.0);
        }
        generate_uniform_knots(&mut c);
        assert_eq!(c.knots.len(), 4 + 2 + 1);
    }

    #[test]
    fn validate_with_knots() {
        let c = linear_nurbs_curve("line", &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        assert!(validate_nurbs(&c));
    }

    #[test]
    fn centroid_line() {
        let c = linear_nurbs_curve("l", &[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let ctr = nurbs_centroid(&c);
        assert!((ctr[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn avg_weight_one() {
        let c = linear_nurbs_curve("l", &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert!((avg_weight(&c) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_has_name() {
        let c = new_nurbs_curve_export("myline", 1);
        let j = nurbs_to_json(&c);
        assert!(j.contains("\"name\":\"myline\""));
    }

    #[test]
    fn validate_fails_no_knots() {
        let mut c = new_nurbs_curve_export("c", 2);
        add_nurbs_control_point(&mut c, [0.0, 0.0, 0.0], 1.0);
        assert!(!validate_nurbs(&c));
    }

    #[test]
    fn linear_curve_built() {
        let c = linear_nurbs_curve("line", &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        assert_eq!(control_point_count_nurbs(&c), 3);
    }

    #[test]
    fn empty_centroid() {
        let c = new_nurbs_curve_export("x", 1);
        let ctr = nurbs_centroid(&c);
        assert_eq!(ctr, [0.0; 3]);
    }
}
