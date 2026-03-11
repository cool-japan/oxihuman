// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cesium 3D Tiles export stub.

#[allow(dead_code)]
pub struct Cesium3DTileContent {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[u8; 4]>,
}

#[allow(dead_code)]
pub struct Cesium3DTileExport {
    pub root: Cesium3DTileContent,
    pub geometric_error: f32,
}

#[allow(dead_code)]
pub fn new_cesium3dtile_export(geometric_error: f32) -> Cesium3DTileExport {
    Cesium3DTileExport {
        root: Cesium3DTileContent { positions: Vec::new(), colors: Vec::new() },
        geometric_error,
    }
}

#[allow(dead_code)]
pub fn c3d_add_point(exp: &mut Cesium3DTileExport, pos: [f32; 3], color: [u8; 4]) {
    exp.root.positions.push(pos);
    exp.root.colors.push(color);
}

#[allow(dead_code)]
pub fn c3d_point_count(exp: &Cesium3DTileExport) -> usize {
    exp.root.positions.len()
}

#[allow(dead_code)]
pub fn c3d_geometric_error(exp: &Cesium3DTileExport) -> f32 {
    exp.geometric_error
}

#[allow(dead_code)]
pub fn c3d_to_json_header(exp: &Cesium3DTileExport) -> String {
    format!(r#"{{"geometricError":{},"points":{}}}"#, exp.geometric_error, c3d_point_count(exp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_cesium3dtile_export(16.0);
        assert_eq!(c3d_point_count(&exp), 0);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_cesium3dtile_export(16.0);
        c3d_add_point(&mut exp, [1.0, 2.0, 3.0], [255, 255, 255, 255]);
        assert_eq!(c3d_point_count(&exp), 1);
    }

    #[test]
    fn test_geometric_error_getter() {
        let exp = new_cesium3dtile_export(8.5);
        assert!((c3d_geometric_error(&exp) - 8.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json_header_structure() {
        let exp = new_cesium3dtile_export(4.0);
        let json = c3d_to_json_header(&exp);
        assert!(json.contains("geometricError"));
        assert!(json.contains("points"));
    }

    #[test]
    fn test_multiple_points() {
        let mut exp = new_cesium3dtile_export(4.0);
        for i in 0..5 {
            c3d_add_point(&mut exp, [i as f32, 0.0, 0.0], [0, 0, 0, 255]);
        }
        assert_eq!(c3d_point_count(&exp), 5);
    }

    #[test]
    fn test_colors_match_positions() {
        let mut exp = new_cesium3dtile_export(1.0);
        c3d_add_point(&mut exp, [0.0; 3], [10, 20, 30, 255]);
        assert_eq!(exp.root.colors[0], [10, 20, 30, 255]);
    }

    #[test]
    fn test_json_header_contains_error_value() {
        let exp = new_cesium3dtile_export(16.0);
        let json = c3d_to_json_header(&exp);
        assert!(json.contains("16"));
    }

    #[test]
    fn test_point_count_zero_initially() {
        let exp = new_cesium3dtile_export(0.0);
        assert_eq!(c3d_point_count(&exp), 0);
    }
}
