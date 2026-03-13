// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! PTS point cloud text format export.

/// A single PTS point.
#[derive(Debug, Clone)]
pub struct PtsPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub intensity: i16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// PTS point cloud export.
#[derive(Debug, Clone, Default)]
pub struct PtsExport {
    pub points: Vec<PtsPoint>,
    pub has_intensity: bool,
    pub has_color: bool,
}

/// Create a new PTS export.
pub fn new_pts_export(has_intensity: bool, has_color: bool) -> PtsExport {
    PtsExport {
        points: Vec::new(),
        has_intensity,
        has_color,
    }
}

/// Add a point (x, y, z) with optional intensity and color.
#[allow(clippy::too_many_arguments)]
pub fn add_pts_point(
    export: &mut PtsExport,
    x: f64,
    y: f64,
    z: f64,
    intensity: i16,
    r: u8,
    g: u8,
    b: u8,
) {
    export.points.push(PtsPoint {
        x,
        y,
        z,
        intensity,
        r,
        g,
        b,
    });
}

/// Return the point count.
pub fn pts_point_count(export: &PtsExport) -> usize {
    export.points.len()
}

/// Render the PTS file as a text string.
pub fn export_pts_text(export: &PtsExport) -> String {
    let mut out = String::new();
    out.push_str(&format!("{}\n", export.points.len()));
    for p in &export.points {
        if export.has_intensity && export.has_color {
            out.push_str(&format!(
                "{:.6} {:.6} {:.6} {} {} {} {}\n",
                p.x, p.y, p.z, p.intensity, p.r, p.g, p.b
            ));
        } else if export.has_intensity {
            out.push_str(&format!(
                "{:.6} {:.6} {:.6} {}\n",
                p.x, p.y, p.z, p.intensity
            ));
        } else {
            out.push_str(&format!("{:.6} {:.6} {:.6}\n", p.x, p.y, p.z));
        }
    }
    out
}

/// Compute the bounding box of the point cloud.
pub fn pts_bbox(export: &PtsExport) -> Option<([f64; 3], [f64; 3])> {
    if export.points.is_empty() {
        return None;
    }
    let mut mn = [export.points[0].x, export.points[0].y, export.points[0].z];
    let mut mx = mn;
    for p in &export.points {
        mn[0] = mn[0].min(p.x);
        mn[1] = mn[1].min(p.y);
        mn[2] = mn[2].min(p.z);
        mx[0] = mx[0].max(p.x);
        mx[1] = mx[1].max(p.y);
        mx[2] = mx[2].max(p.z);
    }
    Some((mn, mx))
}

/// Compute the centroid of the point cloud.
pub fn pts_centroid(export: &PtsExport) -> [f64; 3] {
    if export.points.is_empty() {
        return [0.0; 3];
    }
    let n = export.points.len() as f64;
    let sx: f64 = export.points.iter().map(|p| p.x).sum();
    let sy: f64 = export.points.iter().map(|p| p.y).sum();
    let sz: f64 = export.points.iter().map(|p| p.z).sum();
    [sx / n, sy / n, sz / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_export() {
        let exp = new_pts_export(false, false);
        assert_eq!(pts_point_count(&exp), 0);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_pts_export(false, false);
        add_pts_point(&mut exp, 1.0, 2.0, 3.0, 0, 0, 0, 0);
        assert_eq!(pts_point_count(&exp), 1);
    }

    #[test]
    fn test_pts_text_header() {
        let mut exp = new_pts_export(false, false);
        add_pts_point(&mut exp, 1.0, 2.0, 3.0, 0, 0, 0, 0);
        let text = export_pts_text(&exp);
        assert!(text.starts_with("1\n"));
    }

    #[test]
    fn test_pts_text_with_intensity() {
        let mut exp = new_pts_export(true, false);
        add_pts_point(&mut exp, 0.0, 0.0, 0.0, 50, 0, 0, 0);
        let text = export_pts_text(&exp);
        assert!(text.contains("50"));
    }

    #[test]
    fn test_bbox_empty() {
        let exp = new_pts_export(false, false);
        assert!(pts_bbox(&exp).is_none());
    }

    #[test]
    fn test_bbox_single_point() {
        let mut exp = new_pts_export(false, false);
        add_pts_point(&mut exp, 5.0, 6.0, 7.0, 0, 0, 0, 0);
        let (mn, mx) = pts_bbox(&exp).expect("should succeed");
        assert!((mn[0] - 5.0).abs() < 1e-9);
        assert!((mx[0] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_centroid() {
        let mut exp = new_pts_export(false, false);
        add_pts_point(&mut exp, 0.0, 0.0, 0.0, 0, 0, 0, 0);
        add_pts_point(&mut exp, 2.0, 0.0, 0.0, 0, 0, 0, 0);
        let c = pts_centroid(&exp);
        assert!((c[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_centroid_empty() {
        let exp = new_pts_export(false, false);
        assert_eq!(pts_centroid(&exp), [0.0; 3]);
    }

    #[test]
    fn test_pts_text_xyz_only() {
        let mut exp = new_pts_export(false, false);
        add_pts_point(&mut exp, 1.0, 2.0, 3.0, 0, 0, 0, 0);
        let text = export_pts_text(&exp);
        assert!(text.contains("1.000000 2.000000 3.000000"));
    }
}
