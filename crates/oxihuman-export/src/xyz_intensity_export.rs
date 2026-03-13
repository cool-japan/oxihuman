// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XYZ with intensity export.

#[allow(dead_code)]
pub struct XyzIntPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: f32,
}

#[allow(dead_code)]
pub struct XyzIntExport {
    pub points: Vec<XyzIntPoint>,
}

#[allow(dead_code)]
pub fn new_xyz_int_export() -> XyzIntExport {
    XyzIntExport { points: Vec::new() }
}

#[allow(dead_code)]
pub fn xyzint_add_point(exp: &mut XyzIntExport, x: f32, y: f32, z: f32, intensity: f32) {
    exp.points.push(XyzIntPoint { x, y, z, intensity });
}

#[allow(dead_code)]
pub fn xyzint_point_count(exp: &XyzIntExport) -> usize {
    exp.points.len()
}

#[allow(dead_code)]
pub fn xyzint_max_intensity(exp: &XyzIntExport) -> f32 {
    exp.points.iter().map(|p| p.intensity).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn xyzint_to_string(exp: &XyzIntExport) -> String {
    let mut out = String::new();
    for p in &exp.points {
        out.push_str(&format!("{} {} {} {}\n", p.x, p.y, p.z, p.intensity));
    }
    out
}

#[allow(dead_code)]
pub fn xyzint_normalize_intensity(exp: &mut XyzIntExport) {
    let max = xyzint_max_intensity(exp);
    if max < 1e-10 { return; }
    for p in &mut exp.points {
        p.intensity /= max;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        assert_eq!(xyzint_point_count(&new_xyz_int_export()), 0);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_xyz_int_export();
        xyzint_add_point(&mut exp, 1.0, 0.0, 0.0, 0.8);
        assert_eq!(xyzint_point_count(&exp), 1);
    }

    #[test]
    fn test_max_intensity() {
        let mut exp = new_xyz_int_export();
        xyzint_add_point(&mut exp, 0.0, 0.0, 0.0, 0.3);
        xyzint_add_point(&mut exp, 0.0, 0.0, 0.0, 0.9);
        assert!((xyzint_max_intensity(&exp) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_to_string_contains_data() {
        let mut exp = new_xyz_int_export();
        xyzint_add_point(&mut exp, 1.5, 0.0, 0.0, 0.5);
        let s = xyzint_to_string(&exp);
        assert!(s.contains("1.5"));
        assert!(s.contains("0.5"));
    }

    #[test]
    fn test_normalize_intensity() {
        let mut exp = new_xyz_int_export();
        xyzint_add_point(&mut exp, 0.0, 0.0, 0.0, 2.0);
        xyzint_add_point(&mut exp, 0.0, 0.0, 0.0, 4.0);
        xyzint_normalize_intensity(&mut exp);
        assert!((xyzint_max_intensity(&exp) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_empty_no_crash() {
        let mut exp = new_xyz_int_export();
        xyzint_normalize_intensity(&mut exp);
        assert_eq!(xyzint_point_count(&exp), 0);
    }

    #[test]
    fn test_max_intensity_empty() {
        let exp = new_xyz_int_export();
        assert_eq!(xyzint_max_intensity(&exp), 0.0);
    }

    #[test]
    fn test_normalize_min_also_scales() {
        let mut exp = new_xyz_int_export();
        xyzint_add_point(&mut exp, 0.0, 0.0, 0.0, 2.0);
        xyzint_add_point(&mut exp, 0.0, 0.0, 0.0, 4.0);
        xyzint_normalize_intensity(&mut exp);
        assert!((exp.points[0].intensity - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_multiple_points() {
        let mut exp = new_xyz_int_export();
        for i in 0..10 {
            xyzint_add_point(&mut exp, i as f32, 0.0, 0.0, i as f32 * 0.1);
        }
        assert_eq!(xyzint_point_count(&exp), 10);
    }
}
