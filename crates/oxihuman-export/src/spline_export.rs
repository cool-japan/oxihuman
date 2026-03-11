// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SplineType {
    Bezier,
    NURBS,
    Poly,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SplineControlPoint {
    pub position: [f32; 3],
    pub handle_left: [f32; 3],
    pub handle_right: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SplineExport {
    pub name: String,
    pub spline_type: SplineType,
    pub points: Vec<SplineControlPoint>,
    pub closed: bool,
}

#[allow(dead_code)]
pub fn new_spline_export(name: &str, spline_type: SplineType) -> SplineExport {
    SplineExport {
        name: name.to_string(),
        spline_type,
        points: Vec::new(),
        closed: false,
    }
}

#[allow(dead_code)]
pub fn se_add_point(exp: &mut SplineExport, point: SplineControlPoint) {
    exp.points.push(point);
}

#[allow(dead_code)]
pub fn se_point_count(exp: &SplineExport) -> usize {
    exp.points.len()
}

#[allow(dead_code)]
pub fn se_set_closed(exp: &mut SplineExport, closed: bool) {
    exp.closed = closed;
}

#[allow(dead_code)]
pub fn se_to_json(exp: &SplineExport) -> String {
    let type_name = se_spline_type_name(exp);
    format!(
        r#"{{"name":"{}","type":"{}","point_count":{},"closed":{}}}"#,
        exp.name,
        type_name,
        exp.points.len(),
        exp.closed
    )
}

#[allow(dead_code)]
pub fn se_validate(exp: &SplineExport) -> bool {
    !exp.name.is_empty() && !exp.points.is_empty()
}

#[allow(dead_code)]
pub fn se_spline_type_name(exp: &SplineExport) -> &'static str {
    match exp.spline_type {
        SplineType::Bezier => "bezier",
        SplineType::NURBS => "nurbs",
        SplineType::Poly => "poly",
    }
}

#[allow(dead_code)]
pub fn se_arc_length_approx(exp: &SplineExport) -> f32 {
    let pts = &exp.points;
    if pts.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..pts.len() - 1 {
        let a = pts[i].position;
        let b = pts[i + 1].position;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_point(x: f32) -> SplineControlPoint {
        SplineControlPoint {
            position: [x, 0.0, 0.0],
            handle_left: [x - 0.5, 0.0, 0.0],
            handle_right: [x + 0.5, 0.0, 0.0],
        }
    }

    #[test]
    fn test_new_spline() {
        let s = new_spline_export("curve", SplineType::Bezier);
        assert_eq!(s.name, "curve");
        assert_eq!(s.spline_type, SplineType::Bezier);
    }

    #[test]
    fn test_add_point() {
        let mut s = new_spline_export("curve", SplineType::Bezier);
        se_add_point(&mut s, make_point(0.0));
        se_add_point(&mut s, make_point(1.0));
        assert_eq!(se_point_count(&s), 2);
    }

    #[test]
    fn test_set_closed() {
        let mut s = new_spline_export("curve", SplineType::Poly);
        se_set_closed(&mut s, true);
        assert!(s.closed);
    }

    #[test]
    fn test_type_name_bezier() {
        let s = new_spline_export("c", SplineType::Bezier);
        assert_eq!(se_spline_type_name(&s), "bezier");
    }

    #[test]
    fn test_type_name_nurbs() {
        let s = new_spline_export("c", SplineType::NURBS);
        assert_eq!(se_spline_type_name(&s), "nurbs");
    }

    #[test]
    fn test_validate_empty_points() {
        let s = new_spline_export("curve", SplineType::Bezier);
        assert!(!se_validate(&s));
    }

    #[test]
    fn test_arc_length() {
        let mut s = new_spline_export("curve", SplineType::Poly);
        se_add_point(&mut s, make_point(0.0));
        se_add_point(&mut s, make_point(2.0));
        let len = se_arc_length_approx(&s);
        assert!((len - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let mut s = new_spline_export("curve", SplineType::Bezier);
        se_add_point(&mut s, make_point(0.0));
        let j = se_to_json(&s);
        assert!(j.contains("point_count"));
        assert!(j.contains("bezier"));
    }
}
