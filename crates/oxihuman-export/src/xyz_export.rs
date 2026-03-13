// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XYZ point cloud plain text export.

/// XYZ export structure.
#[allow(dead_code)]
pub struct XyzExport {
    pub points: Vec<[f32; 3]>,
    pub has_normals: bool,
    pub normals: Vec<[f32; 3]>,
}

/// Create a new XYZ export.
#[allow(dead_code)]
pub fn new_xyz_export() -> XyzExport {
    XyzExport { points: Vec::new(), has_normals: false, normals: Vec::new() }
}

/// Add a point.
#[allow(dead_code)]
pub fn add_xyz_point(export: &mut XyzExport, x: f32, y: f32, z: f32) {
    export.points.push([x, y, z]);
}

/// Add a point with normal.
#[allow(dead_code)]
pub fn add_xyz_point_with_normal(export: &mut XyzExport, p: [f32; 3], n: [f32; 3]) {
    export.points.push(p);
    export.normals.push(n);
    export.has_normals = true;
}

/// Point count.
#[allow(dead_code)]
pub fn xyz_point_count(export: &XyzExport) -> usize {
    export.points.len()
}

/// Export to XYZ string.
#[allow(dead_code)]
pub fn export_xyz(export: &XyzExport) -> String {
    let mut s = String::new();
    if export.has_normals && export.normals.len() == export.points.len() {
        for (p, n) in export.points.iter().zip(export.normals.iter()) {
            s.push_str(&format!("{} {} {} {} {} {}\n", p[0], p[1], p[2], n[0], n[1], n[2]));
        }
    } else {
        for p in &export.points {
            s.push_str(&format!("{} {} {}\n", p[0], p[1], p[2]));
        }
    }
    s
}

/// Count lines in exported string.
#[allow(dead_code)]
pub fn xyz_line_count(text: &str) -> usize {
    text.lines().count()
}

/// Validate XYZ export.
#[allow(dead_code)]
pub fn validate_xyz(export: &XyzExport) -> bool {
    if export.has_normals {
        export.points.len() == export.normals.len()
    } else {
        !export.points.is_empty()
    }
}

/// Load XYZ from positions.
#[allow(dead_code)]
pub fn xyz_from_positions(positions: &[[f32; 3]]) -> XyzExport {
    let mut e = new_xyz_export();
    for &p in positions {
        add_xyz_point(&mut e, p[0], p[1], p[2]);
    }
    e
}

/// Compute bounding box.
#[allow(dead_code)]
pub fn xyz_bbox(export: &XyzExport) -> ([f32; 3], [f32; 3]) {
    if export.points.is_empty() { return ([0.0;3],[0.0;3]); }
    let mut mn = export.points[0];
    let mut mx = export.points[0];
    for &p in export.points.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] { mn[i] = p[i]; }
            if p[i] > mx[i] { mx[i] = p[i]; }
        }
    }
    (mn, mx)
}

/// Estimate file size in bytes.
#[allow(dead_code)]
pub fn xyz_file_size_estimate(point_count: usize, with_normals: bool) -> usize {
    let per_point = if with_normals { 60 } else { 30 };
    point_count * per_point
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let e = new_xyz_export();
        assert_eq!(xyz_point_count(&e), 0);
    }

    #[test]
    fn add_point() {
        let mut e = new_xyz_export();
        add_xyz_point(&mut e, 1.0, 2.0, 3.0);
        assert_eq!(xyz_point_count(&e), 1);
    }

    #[test]
    fn export_contains_coords() {
        let mut e = new_xyz_export();
        add_xyz_point(&mut e, 1.5, 2.5, 3.5);
        let s = export_xyz(&e);
        assert!(s.contains("1.5"));
    }

    #[test]
    fn export_with_normals() {
        let mut e = new_xyz_export();
        add_xyz_point_with_normal(&mut e, [1.0,0.0,0.0], [0.0,1.0,0.0]);
        let s = export_xyz(&e);
        assert!(s.contains("1"));
        assert!(xyz_line_count(&s) >= 1);
    }

    #[test]
    fn validate_empty_fails() {
        let e = new_xyz_export();
        assert!(!validate_xyz(&e));
    }

    #[test]
    fn validate_passes() {
        let e = xyz_from_positions(&[[0.0,0.0,0.0f32]]);
        assert!(validate_xyz(&e));
    }

    #[test]
    fn bbox_correct() {
        let e = xyz_from_positions(&[[0.0f32,0.0,0.0],[1.0,1.0,1.0]]);
        let (mn, mx) = xyz_bbox(&e);
        assert!((mx[0] - 1.0).abs() < 1e-5);
        assert!(mn[0].abs() < 1e-5);
    }

    #[test]
    fn file_size_larger_with_normals() {
        assert!(xyz_file_size_estimate(100, true) > xyz_file_size_estimate(100, false));
    }

    #[test]
    fn from_positions_count() {
        let pos = vec![[1.0f32,2.0,3.0],[4.0,5.0,6.0]];
        let e = xyz_from_positions(&pos);
        assert_eq!(xyz_point_count(&e), 2);
    }
}
