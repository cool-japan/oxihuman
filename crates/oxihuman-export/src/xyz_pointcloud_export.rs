// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XYZ point cloud text format export.

/// XYZ point cloud export — minimal format (x y z per line, optional normal).
#[derive(Debug, Clone, Default)]
pub struct XyzExport {
    pub points: Vec<[f64; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub has_normals: bool,
}

/// Create a new XYZ export.
pub fn new_xyz_export(has_normals: bool) -> XyzExport {
    XyzExport {
        points: Vec::new(),
        normals: Vec::new(),
        has_normals,
    }
}

/// Add an XYZ point.
pub fn add_xyz_point(export: &mut XyzExport, x: f64, y: f64, z: f64) {
    export.points.push([x, y, z]);
    if export.has_normals {
        export.normals.push([0.0, 1.0, 0.0]);
    }
}

/// Add an XYZ point with a normal.
pub fn add_xyz_point_normal(
    export: &mut XyzExport,
    x: f64,
    y: f64,
    z: f64,
    nx: f32,
    ny: f32,
    nz: f32,
) {
    export.points.push([x, y, z]);
    export.normals.push([nx, ny, nz]);
}

/// Return the point count.
pub fn xyz_point_count(export: &XyzExport) -> usize {
    export.points.len()
}

/// Render the XYZ file as a text string.
pub fn export_xyz_text(export: &XyzExport) -> String {
    let mut out = String::new();
    for (i, &[x, y, z]) in export.points.iter().enumerate() {
        if export.has_normals && i < export.normals.len() {
            let [nx, ny, nz] = export.normals[i];
            out.push_str(&format!(
                "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6}\n",
                x, y, z, nx, ny, nz
            ));
        } else {
            out.push_str(&format!("{:.6} {:.6} {:.6}\n", x, y, z));
        }
    }
    out
}

/// Compute the bounding box.
pub fn xyz_bbox(export: &XyzExport) -> Option<([f64; 3], [f64; 3])> {
    if export.points.is_empty() {
        return None;
    }
    let mut mn = export.points[0];
    let mut mx = export.points[0];
    for &p in &export.points {
        for k in 0..3 {
            mn[k] = mn[k].min(p[k]);
            mx[k] = mx[k].max(p[k]);
        }
    }
    Some((mn, mx))
}

/// Compute the centroid.
pub fn xyz_centroid(export: &XyzExport) -> [f64; 3] {
    if export.points.is_empty() {
        return [0.0; 3];
    }
    let n = export.points.len() as f64;
    let mut s = [0.0f64; 3];
    for &p in &export.points {
        s[0] += p[0];
        s[1] += p[1];
        s[2] += p[2];
    }
    [s[0] / n, s[1] / n, s[2] / n]
}

/// Validate that normal count matches point count when normals are enabled.
pub fn validate_xyz(export: &XyzExport) -> bool {
    if export.has_normals {
        export.normals.len() == export.points.len()
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_export() {
        let exp = new_xyz_export(false);
        assert_eq!(xyz_point_count(&exp), 0);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_xyz_export(false);
        add_xyz_point(&mut exp, 1.0, 2.0, 3.0);
        assert_eq!(xyz_point_count(&exp), 1);
    }

    #[test]
    fn test_export_text_no_normals() {
        let mut exp = new_xyz_export(false);
        add_xyz_point(&mut exp, 1.0, 2.0, 3.0);
        let text = export_xyz_text(&exp);
        assert!(text.contains("1.000000 2.000000 3.000000"));
    }

    #[test]
    fn test_export_text_with_normals() {
        let mut exp = new_xyz_export(true);
        add_xyz_point(&mut exp, 0.0, 0.0, 0.0);
        let text = export_xyz_text(&exp);
        /* should contain 6 floats per line */
        let parts: Vec<&str> = text.split_whitespace().collect();
        assert_eq!(parts.len(), 6);
    }

    #[test]
    fn test_bbox_empty() {
        let exp = new_xyz_export(false);
        assert!(xyz_bbox(&exp).is_none());
    }

    #[test]
    fn test_bbox_two_points() {
        let mut exp = new_xyz_export(false);
        add_xyz_point(&mut exp, -1.0, -2.0, -3.0);
        add_xyz_point(&mut exp, 1.0, 2.0, 3.0);
        let (mn, mx) = xyz_bbox(&exp).expect("should succeed");
        assert!((mn[0] - (-1.0)).abs() < 1e-9);
        assert!((mx[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_centroid() {
        let mut exp = new_xyz_export(false);
        add_xyz_point(&mut exp, 0.0, 0.0, 0.0);
        add_xyz_point(&mut exp, 4.0, 0.0, 0.0);
        let c = xyz_centroid(&exp);
        assert!((c[0] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_validate_with_normals() {
        let mut exp = new_xyz_export(true);
        add_xyz_point(&mut exp, 0.0, 0.0, 0.0);
        assert!(validate_xyz(&exp));
    }

    #[test]
    fn test_validate_without_normals() {
        let mut exp = new_xyz_export(false);
        add_xyz_point(&mut exp, 0.0, 0.0, 0.0);
        assert!(validate_xyz(&exp));
    }

    #[test]
    fn test_add_point_with_normal() {
        let mut exp = new_xyz_export(false);
        add_xyz_point_normal(&mut exp, 1.0, 2.0, 3.0, 0.0, 1.0, 0.0);
        assert_eq!(xyz_point_count(&exp), 1);
        assert_eq!(exp.normals.len(), 1);
    }
}
