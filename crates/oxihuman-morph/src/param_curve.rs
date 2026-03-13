// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A single point on a parameter curve.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CurvePoint {
    pub t: f32,
    pub value: f32,
}

/// A piecewise-linear parameter curve defined by sorted points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParamCurve {
    pub points: Vec<CurvePoint>,
}

/// Create a new empty parameter curve.
#[allow(dead_code)]
pub fn new_param_curve() -> ParamCurve {
    ParamCurve { points: Vec::new() }
}

/// Add a point, keeping the list sorted by t.
#[allow(dead_code)]
pub fn add_curve_point(curve: &mut ParamCurve, t: f32, value: f32) {
    let pt = CurvePoint { t, value };
    let pos = curve.points.partition_point(|p| p.t < t);
    curve.points.insert(pos, pt);
}

/// Evaluate the curve at parameter t using linear interpolation.
#[allow(dead_code)]
pub fn evaluate_curve(curve: &ParamCurve, t: f32) -> f32 {
    if curve.points.is_empty() {
        return 0.0;
    }
    if curve.points.len() == 1 || t <= curve.points[0].t {
        return curve.points[0].value;
    }
    let last = curve.points.len() - 1;
    if t >= curve.points[last].t {
        return curve.points[last].value;
    }
    for i in 0..last {
        let a = &curve.points[i];
        let b = &curve.points[i + 1];
        if (a.t..=b.t).contains(&t) {
            let frac = if (b.t - a.t).abs() < 1e-12 {
                0.0
            } else {
                (t - a.t) / (b.t - a.t)
            };
            return a.value + (b.value - a.value) * frac;
        }
    }
    curve.points[last].value
}

/// Return the number of control points.
#[allow(dead_code)]
pub fn curve_point_count(curve: &ParamCurve) -> usize {
    curve.points.len()
}

/// Return the minimum t value, or 0 if empty.
#[allow(dead_code)]
pub fn curve_min(curve: &ParamCurve) -> f32 {
    curve.points.first().map_or(0.0, |p| p.t)
}

/// Return the maximum t value, or 0 if empty.
#[allow(dead_code)]
pub fn curve_max(curve: &ParamCurve) -> f32 {
    curve.points.last().map_or(0.0, |p| p.t)
}

/// Serialize the curve to a JSON string.
#[allow(dead_code)]
pub fn curve_to_json(curve: &ParamCurve) -> String {
    let pts: Vec<String> = curve
        .points
        .iter()
        .map(|p| format!("{{\"t\":{:.4},\"v\":{:.4}}}", p.t, p.value))
        .collect();
    format!("{{\"points\":[{}]}}", pts.join(","))
}

/// Remove all points from the curve.
#[allow(dead_code)]
pub fn curve_clear(curve: &mut ParamCurve) {
    curve.points.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_curve_empty() {
        let c = new_param_curve();
        assert_eq!(curve_point_count(&c), 0);
    }

    #[test]
    fn add_point() {
        let mut c = new_param_curve();
        add_curve_point(&mut c, 0.0, 1.0);
        assert_eq!(curve_point_count(&c), 1);
    }

    #[test]
    fn evaluate_single_point() {
        let mut c = new_param_curve();
        add_curve_point(&mut c, 0.5, 0.8);
        assert!((evaluate_curve(&c, 0.5) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn evaluate_interpolation() {
        let mut c = new_param_curve();
        add_curve_point(&mut c, 0.0, 0.0);
        add_curve_point(&mut c, 1.0, 1.0);
        assert!((evaluate_curve(&c, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn evaluate_before_first() {
        let mut c = new_param_curve();
        add_curve_point(&mut c, 1.0, 0.5);
        assert!((evaluate_curve(&c, 0.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn evaluate_after_last() {
        let mut c = new_param_curve();
        add_curve_point(&mut c, 0.0, 0.2);
        add_curve_point(&mut c, 1.0, 0.8);
        assert!((evaluate_curve(&c, 2.0) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn min_max() {
        let mut c = new_param_curve();
        add_curve_point(&mut c, 0.5, 0.0);
        add_curve_point(&mut c, 2.0, 1.0);
        assert!((curve_min(&c) - 0.5).abs() < 1e-6);
        assert!((curve_max(&c) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn clear_curve() {
        let mut c = new_param_curve();
        add_curve_point(&mut c, 0.0, 0.0);
        curve_clear(&mut c);
        assert_eq!(curve_point_count(&c), 0);
    }

    #[test]
    fn to_json() {
        let mut c = new_param_curve();
        add_curve_point(&mut c, 0.0, 1.0);
        let j = curve_to_json(&c);
        assert!(j.contains("\"points\""));
    }

    #[test]
    fn evaluate_empty() {
        let c = new_param_curve();
        assert!(evaluate_curve(&c, 0.5).abs() < 1e-6);
    }
}
