// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LAZ compressed point cloud export stub.

#[allow(dead_code)]
pub struct LazPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub intensity: u16,
}

#[allow(dead_code)]
pub struct LazExport {
    pub points: Vec<LazPoint>,
    pub compressed: bool,
}

#[allow(dead_code)]
pub fn new_laz_export() -> LazExport {
    LazExport { points: Vec::new(), compressed: false }
}

#[allow(dead_code)]
pub fn laz_add_point(exp: &mut LazExport, x: f64, y: f64, z: f64, intensity: u16) {
    exp.points.push(LazPoint { x, y, z, intensity });
}

#[allow(dead_code)]
pub fn laz_point_count(exp: &LazExport) -> usize {
    exp.points.len()
}

#[allow(dead_code)]
pub fn laz_set_compressed(exp: &mut LazExport, v: bool) {
    exp.compressed = v;
}

#[allow(dead_code)]
pub fn laz_estimated_size(exp: &LazExport) -> usize {
    let n = exp.points.len();
    if exp.compressed { n * 8 } else { n * 20 }
}

#[allow(dead_code)]
pub fn laz_bounds(exp: &LazExport) -> Option<([f64; 3], [f64; 3])> {
    if exp.points.is_empty() { return None; }
    let mut mn = [exp.points[0].x, exp.points[0].y, exp.points[0].z];
    let mut mx = mn;
    for p in &exp.points {
        if p.x < mn[0] { mn[0] = p.x; }
        if p.x > mx[0] { mx[0] = p.x; }
        if p.y < mn[1] { mn[1] = p.y; }
        if p.y > mx[1] { mx[1] = p.y; }
        if p.z < mn[2] { mn[2] = p.z; }
        if p.z > mx[2] { mx[2] = p.z; }
    }
    Some((mn, mx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_laz_export();
        assert_eq!(laz_point_count(&exp), 0);
        assert!(!exp.compressed);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_laz_export();
        laz_add_point(&mut exp, 1.0, 2.0, 3.0, 500);
        assert_eq!(laz_point_count(&exp), 1);
    }

    #[test]
    fn test_set_compressed() {
        let mut exp = new_laz_export();
        laz_set_compressed(&mut exp, true);
        assert!(exp.compressed);
    }

    #[test]
    fn test_estimated_size_uncompressed() {
        let mut exp = new_laz_export();
        laz_add_point(&mut exp, 0.0, 0.0, 0.0, 0);
        assert_eq!(laz_estimated_size(&exp), 20);
    }

    #[test]
    fn test_estimated_size_compressed() {
        let mut exp = new_laz_export();
        laz_add_point(&mut exp, 0.0, 0.0, 0.0, 0);
        laz_set_compressed(&mut exp, true);
        assert_eq!(laz_estimated_size(&exp), 8);
    }

    #[test]
    fn test_bounds_none_empty() {
        let exp = new_laz_export();
        assert!(laz_bounds(&exp).is_none());
    }

    #[test]
    fn test_bounds_two_points() {
        let mut exp = new_laz_export();
        laz_add_point(&mut exp, -1.0, -2.0, -3.0, 0);
        laz_add_point(&mut exp, 1.0, 2.0, 3.0, 0);
        let (mn, mx) = laz_bounds(&exp).unwrap();
        assert!((mn[0] - (-1.0)).abs() < 1e-9);
        assert!((mx[2] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_compression_changes_size() {
        let mut exp = new_laz_export();
        for _ in 0..5 {
            laz_add_point(&mut exp, 0.0, 0.0, 0.0, 0);
        }
        let uncompressed = laz_estimated_size(&exp);
        laz_set_compressed(&mut exp, true);
        let compressed = laz_estimated_size(&exp);
        assert!(compressed < uncompressed);
    }
}
