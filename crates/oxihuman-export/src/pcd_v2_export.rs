// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! PCD v2 point cloud export (extends basic PCD with additional fields).

#[allow(dead_code)]
pub struct PcdV2Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub normal_x: f32,
    pub normal_y: f32,
    pub normal_z: f32,
    pub curvature: f32,
}

#[allow(dead_code)]
pub struct PcdV2Export {
    pub points: Vec<PcdV2Point>,
    pub version: String,
}

#[allow(dead_code)]
pub fn new_pcd_v2_export() -> PcdV2Export {
    PcdV2Export { points: Vec::new(), version: "0.7".to_string() }
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn pcd2_add_point(exp: &mut PcdV2Export, x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32, curvature: f32) {
    exp.points.push(PcdV2Point { x, y, z, normal_x: nx, normal_y: ny, normal_z: nz, curvature });
}

#[allow(dead_code)]
pub fn pcd2_point_count(exp: &PcdV2Export) -> usize {
    exp.points.len()
}

#[allow(dead_code)]
pub fn pcd2_avg_curvature(exp: &PcdV2Export) -> f32 {
    let n = exp.points.len();
    if n == 0 { return 0.0; }
    exp.points.iter().map(|p| p.curvature).sum::<f32>() / n as f32
}

#[allow(dead_code)]
pub fn pcd2_to_header(exp: &PcdV2Export) -> String {
    format!("# .PCD v{}\nVERSION {}\nPOINTS {}\n", exp.version, exp.version, exp.points.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_pcd_v2_export();
        assert_eq!(pcd2_point_count(&exp), 0);
    }

    #[test]
    fn test_version_default() {
        let exp = new_pcd_v2_export();
        assert_eq!(exp.version, "0.7");
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_pcd_v2_export();
        pcd2_add_point(&mut exp, 1.0, 2.0, 3.0, 0.0, 0.0, 1.0, 0.01);
        assert_eq!(pcd2_point_count(&exp), 1);
    }

    #[test]
    fn test_avg_curvature_empty() {
        let exp = new_pcd_v2_export();
        assert_eq!(pcd2_avg_curvature(&exp), 0.0);
    }

    #[test]
    fn test_avg_curvature() {
        let mut exp = new_pcd_v2_export();
        pcd2_add_point(&mut exp, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1);
        pcd2_add_point(&mut exp, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.3);
        assert!((pcd2_avg_curvature(&exp) - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_header_contains_version() {
        let exp = new_pcd_v2_export();
        assert!(pcd2_to_header(&exp).contains("0.7"));
    }

    #[test]
    fn test_header_contains_points() {
        let mut exp = new_pcd_v2_export();
        pcd2_add_point(&mut exp, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        assert!(pcd2_to_header(&exp).contains('1'));
    }

    #[test]
    fn test_multiple_points() {
        let mut exp = new_pcd_v2_export();
        for _ in 0..5 {
            pcd2_add_point(&mut exp, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.05);
        }
        assert_eq!(pcd2_point_count(&exp), 5);
    }
}
