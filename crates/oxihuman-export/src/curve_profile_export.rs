// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Curve profile export for bevel/extrude profiles.

/// A 2D profile point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ProfilePoint {
    pub x: f32,
    pub y: f32,
}

/// Curve profile export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveProfileExport {
    pub name: String,
    pub points: Vec<ProfilePoint>,
}

#[allow(dead_code)]
pub fn new_curve_profile(name: &str) -> CurveProfileExport {
    CurveProfileExport {
        name: name.to_string(),
        points: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn cp_add_point(p: &mut CurveProfileExport, x: f32, y: f32) {
    p.points.push(ProfilePoint { x, y });
}

#[allow(dead_code)]
pub fn cp_point_count(p: &CurveProfileExport) -> usize {
    p.points.len()
}

#[allow(dead_code)]
pub fn cp_point_at(p: &CurveProfileExport, idx: usize) -> Option<ProfilePoint> {
    p.points.get(idx).copied()
}

#[allow(dead_code)]
pub fn cp_arc_length(p: &CurveProfileExport) -> f32 {
    p.points
        .windows(2)
        .map(|w| {
            let dx = w[1].x - w[0].x;
            let dy = w[1].y - w[0].y;
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

#[allow(dead_code)]
pub fn cp_bounding_box(p: &CurveProfileExport) -> (f32, f32, f32, f32) {
    if p.points.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for pt in &p.points {
        min_x = min_x.min(pt.x);
        min_y = min_y.min(pt.y);
        max_x = max_x.max(pt.x);
        max_y = max_y.max(pt.y);
    }
    (min_x, min_y, max_x, max_y)
}

#[allow(dead_code)]
pub fn cp_clear(p: &mut CurveProfileExport) {
    p.points.clear();
}

#[allow(dead_code)]
pub fn curve_profile_to_json(p: &CurveProfileExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"points\":{},\"arc_length\":{:.6}}}",
        p.name,
        p.points.len(),
        cp_arc_length(p)
    )
}

#[allow(dead_code)]
pub fn cp_reverse(p: &mut CurveProfileExport) {
    p.points.reverse();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(cp_point_count(&new_curve_profile("p")), 0);
    }

    #[test]
    fn test_add_point() {
        let mut p = new_curve_profile("p");
        cp_add_point(&mut p, 0.0, 0.0);
        assert_eq!(cp_point_count(&p), 1);
    }

    #[test]
    fn test_point_at() {
        let mut p = new_curve_profile("p");
        cp_add_point(&mut p, 1.0, 2.0);
        let pt = cp_point_at(&p, 0).unwrap();
        assert!((pt.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_point_at_oob() {
        assert!(cp_point_at(&new_curve_profile("p"), 0).is_none());
    }

    #[test]
    fn test_arc_length() {
        let mut p = new_curve_profile("p");
        cp_add_point(&mut p, 0.0, 0.0);
        cp_add_point(&mut p, 3.0, 4.0);
        assert!((cp_arc_length(&p) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_bounding_box() {
        let mut p = new_curve_profile("p");
        cp_add_point(&mut p, -1.0, 2.0);
        cp_add_point(&mut p, 3.0, -4.0);
        let (x0, y0, x1, y1) = cp_bounding_box(&p);
        assert!((x0 - (-1.0)).abs() < 1e-6);
        assert!((y1 - 2.0).abs() < 1e-6);
        let _ = (x1, y0);
    }

    #[test]
    fn test_clear() {
        let mut p = new_curve_profile("p");
        cp_add_point(&mut p, 0.0, 0.0);
        cp_clear(&mut p);
        assert_eq!(cp_point_count(&p), 0);
    }

    #[test]
    fn test_to_json() {
        let p = new_curve_profile("bevel");
        assert!(curve_profile_to_json(&p).contains("\"name\":\"bevel\""));
    }

    #[test]
    fn test_reverse() {
        let mut p = new_curve_profile("p");
        cp_add_point(&mut p, 0.0, 0.0);
        cp_add_point(&mut p, 1.0, 1.0);
        cp_reverse(&mut p);
        assert!((p.points[0].x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_arc_length_empty() {
        let p = new_curve_profile("p");
        assert!((cp_arc_length(&p)).abs() < 1e-6);
    }
}
