// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LAS (LiDAR) format export stub.

#[allow(dead_code)]
pub struct LasPointV2 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub intensity: u16,
    pub classification: u8,
}

#[allow(dead_code)]
pub struct LasExportV2 {
    pub points: Vec<LasPointV2>,
    pub scale: [f64; 3],
    pub offset: [f64; 3],
}

#[allow(dead_code)]
pub fn new_las_export_v2(scale: f64, offset: [f64; 3]) -> LasExportV2 {
    LasExportV2 { points: Vec::new(), scale: [scale; 3], offset }
}

#[allow(dead_code)]
pub fn las_add_point_v2(exp: &mut LasExportV2, x: f64, y: f64, z: f64, intensity: u16, classification: u8) {
    exp.points.push(LasPointV2 { x, y, z, intensity, classification });
}

#[allow(dead_code)]
pub fn las_point_count_v2(exp: &LasExportV2) -> usize {
    exp.points.len()
}

#[allow(dead_code)]
pub fn las_avg_intensity_v2(exp: &LasExportV2) -> f64 {
    if exp.points.is_empty() { return 0.0; }
    let sum: f64 = exp.points.iter().map(|p| p.intensity as f64).sum();
    sum / exp.points.len() as f64
}

#[allow(dead_code)]
pub fn las_class_count_v2(exp: &LasExportV2, class: u8) -> usize {
    exp.points.iter().filter(|p| p.classification == class).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_las_export_v2(0.001, [0.0; 3]);
        assert_eq!(las_point_count_v2(&exp), 0);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_las_export_v2(0.001, [0.0; 3]);
        las_add_point_v2(&mut exp, 1.0, 2.0, 3.0, 500, 2);
        assert_eq!(las_point_count_v2(&exp), 1);
    }

    #[test]
    fn test_avg_intensity_empty() {
        let exp = new_las_export_v2(0.001, [0.0; 3]);
        assert_eq!(las_avg_intensity_v2(&exp), 0.0);
    }

    #[test]
    fn test_avg_intensity() {
        let mut exp = new_las_export_v2(0.001, [0.0; 3]);
        las_add_point_v2(&mut exp, 0.0, 0.0, 0.0, 100, 0);
        las_add_point_v2(&mut exp, 0.0, 0.0, 0.0, 300, 0);
        assert!((las_avg_intensity_v2(&exp) - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_class_count() {
        let mut exp = new_las_export_v2(0.001, [0.0; 3]);
        las_add_point_v2(&mut exp, 0.0, 0.0, 0.0, 100, 2);
        las_add_point_v2(&mut exp, 0.0, 0.0, 0.0, 100, 5);
        las_add_point_v2(&mut exp, 0.0, 0.0, 0.0, 100, 2);
        assert_eq!(las_class_count_v2(&exp, 2), 2);
        assert_eq!(las_class_count_v2(&exp, 5), 1);
    }

    #[test]
    fn test_scale_stored() {
        let exp = new_las_export_v2(0.01, [0.0; 3]);
        assert!((exp.scale[0] - 0.01).abs() < 1e-9);
    }

    #[test]
    fn test_offset_stored() {
        let exp = new_las_export_v2(0.001, [1.0, 2.0, 3.0]);
        assert!((exp.offset[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_class_count_zero_missing_class() {
        let exp = new_las_export_v2(0.001, [0.0; 3]);
        assert_eq!(las_class_count_v2(&exp, 99), 0);
    }
}
