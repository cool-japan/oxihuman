// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! E57 point cloud format export stub.

#[allow(dead_code)]
pub struct E57PointV2 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub intensity: f32,
}

#[allow(dead_code)]
pub struct E57ExportV2 {
    pub points: Vec<E57PointV2>,
    pub guid: String,
}

#[allow(dead_code)]
pub fn new_e57_export_v2(guid: &str) -> E57ExportV2 {
    E57ExportV2 { points: Vec::new(), guid: guid.to_string() }
}

#[allow(dead_code)]
pub fn e57_add_point_v2(exp: &mut E57ExportV2, x: f64, y: f64, z: f64, intensity: f32) {
    exp.points.push(E57PointV2 { x, y, z, intensity });
}

#[allow(dead_code)]
pub fn e57_point_count_v2(exp: &E57ExportV2) -> usize {
    exp.points.len()
}

#[allow(dead_code)]
pub fn e57_bounds_v2(exp: &E57ExportV2) -> Option<([f64; 3], [f64; 3])> {
    if exp.points.is_empty() { return None; }
    let mut mn = [exp.points[0].x, exp.points[0].y, exp.points[0].z];
    let mut mx = mn;
    for p in &exp.points {
        if p.x < mn[0] { mn[0] = p.x; }
        if p.x > mx[0] { mx[0] = p.x; }
        if p.y < mn[1] { mn[1] = p.y; }
        if p.y > mx[1] { mx[1] = p.y; }
        if p.z < mn[2] { mn[2] = p.z; }
        if p.z > mx[2] { mx[2] = p.z; }
    }
    Some((mn, mx))
}

#[allow(dead_code)]
pub fn e57_to_header_string_v2(exp: &E57ExportV2) -> String {
    format!("E57 guid={} points={}", exp.guid, exp.points.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_e57_export_v2("test-guid");
        assert_eq!(exp.guid, "test-guid");
        assert_eq!(e57_point_count_v2(&exp), 0);
    }

    #[test]
    fn test_add_points() {
        let mut exp = new_e57_export_v2("g");
        e57_add_point_v2(&mut exp, 1.0, 2.0, 3.0, 0.5);
        assert_eq!(e57_point_count_v2(&exp), 1);
    }

    #[test]
    fn test_bounds_none_empty() {
        let exp = new_e57_export_v2("g");
        assert!(e57_bounds_v2(&exp).is_none());
    }

    #[test]
    fn test_bounds_two_points() {
        let mut exp = new_e57_export_v2("g");
        e57_add_point_v2(&mut exp, 0.0, 0.0, 0.0, 1.0);
        e57_add_point_v2(&mut exp, 1.0, 2.0, 3.0, 0.5);
        let (mn, mx) = e57_bounds_v2(&exp).unwrap();
        assert!((mn[0]).abs() < 1e-9);
        assert!((mx[2] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_header_string_contains_guid() {
        let exp = new_e57_export_v2("abc-123");
        let s = e57_to_header_string_v2(&exp);
        assert!(s.contains("abc-123"));
    }

    #[test]
    fn test_header_string_contains_count() {
        let mut exp = new_e57_export_v2("g");
        e57_add_point_v2(&mut exp, 0.0, 0.0, 0.0, 1.0);
        let s = e57_to_header_string_v2(&exp);
        assert!(s.contains('1'));
    }

    #[test]
    fn test_add_multiple_points() {
        let mut exp = new_e57_export_v2("g");
        for i in 0..5 {
            e57_add_point_v2(&mut exp, i as f64, 0.0, 0.0, 1.0);
        }
        assert_eq!(e57_point_count_v2(&exp), 5);
    }

    #[test]
    fn test_bounds_min_max() {
        let mut exp = new_e57_export_v2("g");
        e57_add_point_v2(&mut exp, -5.0, -3.0, -1.0, 1.0);
        e57_add_point_v2(&mut exp, 5.0, 3.0, 1.0, 1.0);
        let (mn, mx) = e57_bounds_v2(&exp).unwrap();
        assert!((mn[0] - (-5.0)).abs() < 1e-9);
        assert!((mx[0] - 5.0).abs() < 1e-9);
    }
}
