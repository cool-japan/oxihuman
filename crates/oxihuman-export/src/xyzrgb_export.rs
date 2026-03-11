// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! XYZRGB colored point cloud export.

#[allow(dead_code)]
pub struct XyzRgbPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[allow(dead_code)]
pub struct XyzRgbExport {
    pub points: Vec<XyzRgbPoint>,
}

#[allow(dead_code)]
pub fn new_xyzrgb_export() -> XyzRgbExport {
    XyzRgbExport { points: Vec::new() }
}

#[allow(dead_code)]
pub fn xyzrgb_add_point(exp: &mut XyzRgbExport, x: f32, y: f32, z: f32, r: u8, g: u8, b: u8) {
    exp.points.push(XyzRgbPoint { x, y, z, r, g, b });
}

#[allow(dead_code)]
pub fn xyzrgb_point_count(exp: &XyzRgbExport) -> usize {
    exp.points.len()
}

#[allow(dead_code)]
pub fn xyzrgb_to_string(exp: &XyzRgbExport) -> String {
    let mut out = String::new();
    for p in &exp.points {
        out.push_str(&format!("{} {} {} {} {} {}\n", p.x, p.y, p.z, p.r, p.g, p.b));
    }
    out
}

#[allow(dead_code)]
pub fn xyzrgb_avg_color(exp: &XyzRgbExport) -> [f32; 3] {
    let n = exp.points.len();
    if n == 0 { return [0.0; 3]; }
    let sum_r: f32 = exp.points.iter().map(|p| p.r as f32).sum();
    let sum_g: f32 = exp.points.iter().map(|p| p.g as f32).sum();
    let sum_b: f32 = exp.points.iter().map(|p| p.b as f32).sum();
    [sum_r / (n as f32 * 255.0), sum_g / (n as f32 * 255.0), sum_b / (n as f32 * 255.0)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        assert_eq!(xyzrgb_point_count(&new_xyzrgb_export()), 0);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_xyzrgb_export();
        xyzrgb_add_point(&mut exp, 1.0, 2.0, 3.0, 255, 0, 0);
        assert_eq!(xyzrgb_point_count(&exp), 1);
    }

    #[test]
    fn test_to_string_contains_numbers() {
        let mut exp = new_xyzrgb_export();
        xyzrgb_add_point(&mut exp, 1.0, 2.0, 3.0, 128, 64, 32);
        let s = xyzrgb_to_string(&exp);
        assert!(s.contains("1"));
        assert!(s.contains("128"));
    }

    #[test]
    fn test_avg_color_white() {
        let mut exp = new_xyzrgb_export();
        xyzrgb_add_point(&mut exp, 0.0, 0.0, 0.0, 255, 255, 255);
        let avg = xyzrgb_avg_color(&exp);
        assert!((avg[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_avg_color_empty() {
        let exp = new_xyzrgb_export();
        assert_eq!(xyzrgb_avg_color(&exp), [0.0; 3]);
    }

    #[test]
    fn test_multiple_points_count() {
        let mut exp = new_xyzrgb_export();
        for _ in 0..5 {
            xyzrgb_add_point(&mut exp, 0.0, 0.0, 0.0, 0, 0, 0);
        }
        assert_eq!(xyzrgb_point_count(&exp), 5);
    }

    #[test]
    fn test_to_string_empty() {
        let exp = new_xyzrgb_export();
        assert!(xyzrgb_to_string(&exp).is_empty());
    }

    #[test]
    fn test_avg_color_two_points() {
        let mut exp = new_xyzrgb_export();
        xyzrgb_add_point(&mut exp, 0.0, 0.0, 0.0, 0, 0, 0);
        xyzrgb_add_point(&mut exp, 0.0, 0.0, 0.0, 255, 0, 0);
        let avg = xyzrgb_avg_color(&exp);
        assert!((avg[0] - 0.5).abs() < 1e-4);
    }
}
