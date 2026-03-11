// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! PTS point cloud export (x y z intensity r g b).

/// PTS point.
#[allow(dead_code)]
pub struct PtsPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: i32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// PTS export.
#[allow(dead_code)]
pub struct PtsExport {
    pub points: Vec<PtsPoint>,
}

/// Create a new PTS export.
#[allow(dead_code)]
pub fn new_pts_export() -> PtsExport {
    PtsExport { points: Vec::new() }
}

/// Add a point.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn add_pts_point(export: &mut PtsExport, x: f32, y: f32, z: f32, intensity: i32, r: u8, g: u8, b: u8) {
    export.points.push(PtsPoint { x, y, z, intensity, r, g, b });
}

/// Point count.
#[allow(dead_code)]
pub fn pts_point_count(export: &PtsExport) -> usize {
    export.points.len()
}

/// Export to PTS string.
#[allow(dead_code)]
pub fn export_pts(export: &PtsExport) -> String {
    let mut s = format!("{}\n", export.points.len());
    for p in &export.points {
        s.push_str(&format!("{} {} {} {} {} {} {}\n", p.x, p.y, p.z, p.intensity, p.r, p.g, p.b));
    }
    s
}

/// Validate PTS.
#[allow(dead_code)]
pub fn validate_pts(export: &PtsExport) -> bool {
    !export.points.is_empty()
}

/// Load from positions (white, zero intensity).
#[allow(dead_code)]
pub fn pts_from_positions(positions: &[[f32; 3]]) -> PtsExport {
    let mut e = new_pts_export();
    for &p in positions {
        add_pts_point(&mut e, p[0], p[1], p[2], 0, 255, 255, 255);
    }
    e
}

/// Average intensity.
#[allow(dead_code)]
pub fn pts_avg_intensity(export: &PtsExport) -> f32 {
    if export.points.is_empty() { return 0.0; }
    export.points.iter().map(|p| p.intensity as f32).sum::<f32>() / export.points.len() as f32
}

/// Intensity range.
#[allow(dead_code)]
pub fn pts_intensity_range(export: &PtsExport) -> (i32, i32) {
    if export.points.is_empty() { return (0, 0); }
    let mn = export.points.iter().map(|p| p.intensity).min().unwrap_or(0);
    let mx = export.points.iter().map(|p| p.intensity).max().unwrap_or(0);
    (mn, mx)
}

/// Check all RGB values are in 0..=255.
#[allow(dead_code)]
pub fn pts_rgb_valid(_export: &PtsExport) -> bool {
    true
}

/// Estimate file size in bytes.
#[allow(dead_code)]
pub fn pts_file_size_estimate(count: usize) -> usize {
    count * 40 + 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let e = new_pts_export();
        assert_eq!(pts_point_count(&e), 0);
    }

    #[test]
    fn add_point() {
        let mut e = new_pts_export();
        add_pts_point(&mut e, 1.0, 2.0, 3.0, 100, 255, 128, 0);
        assert_eq!(pts_point_count(&e), 1);
    }

    #[test]
    fn export_starts_with_count() {
        let mut e = new_pts_export();
        add_pts_point(&mut e, 0.0, 0.0, 0.0, 0, 0, 0, 0);
        let s = export_pts(&e);
        assert!(s.starts_with('1'));
    }

    #[test]
    fn validate_empty_fails() {
        let e = new_pts_export();
        assert!(!validate_pts(&e));
    }

    #[test]
    fn validate_passes() {
        let e = pts_from_positions(&[[0.0f32,0.0,0.0]]);
        assert!(validate_pts(&e));
    }

    #[test]
    fn avg_intensity_correct() {
        let mut e = new_pts_export();
        add_pts_point(&mut e, 0.0, 0.0, 0.0, 100, 255, 255, 255);
        add_pts_point(&mut e, 0.0, 0.0, 0.0, 200, 255, 255, 255);
        let avg = pts_avg_intensity(&e);
        assert!((avg - 150.0).abs() < 1e-5);
    }

    #[test]
    fn intensity_range() {
        let mut e = new_pts_export();
        add_pts_point(&mut e, 0.0, 0.0, 0.0, 10, 0, 0, 0);
        add_pts_point(&mut e, 0.0, 0.0, 0.0, 200, 0, 0, 0);
        let (mn, mx) = pts_intensity_range(&e);
        assert_eq!(mn, 10);
        assert_eq!(mx, 200);
    }

    #[test]
    fn file_size_grows() {
        assert!(pts_file_size_estimate(100) > pts_file_size_estimate(10));
    }

    #[test]
    fn rgb_valid() {
        let e = pts_from_positions(&[[0.0f32,0.0,0.0]]);
        assert!(pts_rgb_valid(&e));
    }
}
