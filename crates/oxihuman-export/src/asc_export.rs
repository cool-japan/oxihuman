// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ASC ASCII point cloud export.

/// ASC export structure.
#[allow(dead_code)]
pub struct AscExport {
    pub points: Vec<[f32; 3]>,
    pub intensities: Vec<f32>,
    pub has_intensity: bool,
}

/// Create a new ASC export.
#[allow(dead_code)]
pub fn new_asc_export() -> AscExport {
    AscExport { points: Vec::new(), intensities: Vec::new(), has_intensity: false }
}

/// Add a point.
#[allow(dead_code)]
pub fn add_asc_point(export: &mut AscExport, x: f32, y: f32, z: f32) {
    export.points.push([x, y, z]);
    if export.has_intensity {
        export.intensities.push(0.0);
    }
}

/// Add a point with intensity.
#[allow(dead_code)]
pub fn add_asc_point_with_intensity(export: &mut AscExport, x: f32, y: f32, z: f32, i: f32) {
    export.points.push([x, y, z]);
    export.intensities.push(i);
    export.has_intensity = true;
}

/// Point count.
#[allow(dead_code)]
pub fn asc_point_count(export: &AscExport) -> usize {
    export.points.len()
}

/// Export to ASC string.
#[allow(dead_code)]
pub fn export_asc(export: &AscExport) -> String {
    let mut s = String::new();
    for (i, p) in export.points.iter().enumerate() {
        if export.has_intensity && i < export.intensities.len() {
            s.push_str(&format!("{} {} {} {}\n", p[0], p[1], p[2], export.intensities[i]));
        } else {
            s.push_str(&format!("{} {} {}\n", p[0], p[1], p[2]));
        }
    }
    s
}

/// Validate.
#[allow(dead_code)]
pub fn validate_asc(export: &AscExport) -> bool {
    !export.points.is_empty()
}

/// Load from positions.
#[allow(dead_code)]
pub fn asc_from_positions(positions: &[[f32; 3]]) -> AscExport {
    let mut e = new_asc_export();
    for &p in positions {
        add_asc_point(&mut e, p[0], p[1], p[2]);
    }
    e
}

/// Average intensity.
#[allow(dead_code)]
pub fn asc_avg_intensity(export: &AscExport) -> f32 {
    if export.intensities.is_empty() { return 0.0; }
    export.intensities.iter().sum::<f32>() / export.intensities.len() as f32
}

/// Estimate file size in bytes.
#[allow(dead_code)]
pub fn asc_file_size_estimate(point_count: usize) -> usize {
    point_count * 35
}

/// Compute density (points per unit volume proxy).
#[allow(dead_code)]
pub fn asc_point_density(export: &AscExport, volume: f32) -> f32 {
    if volume < 1e-10 { return 0.0; }
    export.points.len() as f32 / volume
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_export() {
        let e = new_asc_export();
        assert_eq!(asc_point_count(&e), 0);
    }

    #[test]
    fn add_point_count() {
        let mut e = new_asc_export();
        add_asc_point(&mut e, 1.0, 2.0, 3.0);
        assert_eq!(asc_point_count(&e), 1);
    }

    #[test]
    fn export_string_has_coords() {
        let mut e = new_asc_export();
        add_asc_point(&mut e, 1.5, 0.0, 0.0);
        let s = export_asc(&e);
        assert!(s.contains("1.5"));
    }

    #[test]
    fn export_with_intensity() {
        let mut e = new_asc_export();
        add_asc_point_with_intensity(&mut e, 1.0, 2.0, 3.0, 0.75);
        let s = export_asc(&e);
        assert!(s.contains("0.75"));
    }

    #[test]
    fn validate_empty_fails() {
        let e = new_asc_export();
        assert!(!validate_asc(&e));
    }

    #[test]
    fn validate_passes() {
        let e = asc_from_positions(&[[0.0f32,0.0,0.0]]);
        assert!(validate_asc(&e));
    }

    #[test]
    fn avg_intensity_zero_when_empty() {
        let e = new_asc_export();
        assert_eq!(asc_avg_intensity(&e), 0.0);
    }

    #[test]
    fn avg_intensity_correct() {
        let mut e = new_asc_export();
        add_asc_point_with_intensity(&mut e, 0.0, 0.0, 0.0, 0.5);
        add_asc_point_with_intensity(&mut e, 1.0, 0.0, 0.0, 1.0);
        let avg = asc_avg_intensity(&e);
        assert!((avg - 0.75).abs() < 1e-5);
    }

    #[test]
    fn file_size_grows() {
        assert!(asc_file_size_estimate(100) > asc_file_size_estimate(10));
    }

    #[test]
    fn density_calculation() {
        let e = asc_from_positions(&[[0.0f32,0.0,0.0],[1.0,0.0,0.0]]);
        let d = asc_point_density(&e, 10.0);
        assert!((d - 0.2).abs() < 1e-5);
    }
}
