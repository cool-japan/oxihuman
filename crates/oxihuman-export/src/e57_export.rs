// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! E57 point cloud format export stub.

pub const E57_MAGIC: &[u8; 4] = b"ASTM";

/// E57 point record.
#[allow(dead_code)]
pub struct E57Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub intensity: f32,
}

/// E57 export document stub.
#[allow(dead_code)]
pub struct E57Export {
    pub points: Vec<E57Point>,
    pub scanner_name: String,
    pub guid: String,
}

/// Create a new E57 export.
#[allow(dead_code)]
pub fn new_e57_export(scanner_name: &str) -> E57Export {
    E57Export {
        points: Vec::new(),
        scanner_name: scanner_name.to_string(),
        guid: "00000000-0000-0000-0000-000000000000".to_string(),
    }
}

/// Add a point to the E57 export.
#[allow(dead_code)]
pub fn add_e57_point(export: &mut E57Export, x: f64, y: f64, z: f64, intensity: f32) {
    export.points.push(E57Point { x, y, z, intensity });
}

/// Get point count.
#[allow(dead_code)]
pub fn e57_point_count(export: &E57Export) -> usize {
    export.points.len()
}

/// Build XML header string for E57.
#[allow(dead_code)]
pub fn e57_xml_header_v2(scanner: &str, point_count: usize) -> String {
    format!(
        "<?xml version=\"1.0\"?><e57Root><scanner>{}</scanner><points>{}</points></e57Root>",
        scanner, point_count
    )
}

/// Validate E57 export (check non-empty).
#[allow(dead_code)]
pub fn validate_e57(export: &E57Export) -> bool {
    !export.points.is_empty() && !export.scanner_name.is_empty()
}

/// Estimate binary size in bytes.
#[allow(dead_code)]
pub fn e57_size_estimate(point_count: usize) -> usize {
    point_count * 28 + 256
}

/// Export to bytes stub.
#[allow(dead_code)]
pub fn export_e57_stub(export: &E57Export) -> Vec<u8> {
    let mut buf = E57_MAGIC.to_vec();
    for _ in &export.points {
        buf.extend_from_slice(&[0u8; 28]);
    }
    buf
}

/// Load from positions (f32).
#[allow(dead_code)]
pub fn e57_from_positions(positions: &[[f32; 3]]) -> E57Export {
    let mut e = new_e57_export("unknown");
    for &p in positions {
        add_e57_point(&mut e, p[0] as f64, p[1] as f64, p[2] as f64, 1.0);
    }
    e
}

/// Compute bounding box of E57 points.
#[allow(dead_code)]
pub fn e57_bbox(export: &E57Export) -> ([f64; 3], [f64; 3]) {
    if export.points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = [export.points[0].x, export.points[0].y, export.points[0].z];
    let mut mx = mn;
    for p in &export.points {
        let arr = [p.x, p.y, p.z];
        for i in 0..3 {
            if arr[i] < mn[i] {
                mn[i] = arr[i];
            }
            if arr[i] > mx[i] {
                mx[i] = arr[i];
            }
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_bytes() {
        assert_eq!(&E57_MAGIC[..], b"ASTM");
    }

    #[test]
    fn new_export_empty() {
        let e = new_e57_export("Leica");
        assert_eq!(e57_point_count(&e), 0);
    }

    #[test]
    fn add_point_increments_count() {
        let mut e = new_e57_export("test");
        add_e57_point(&mut e, 1.0, 2.0, 3.0, 0.5);
        assert_eq!(e57_point_count(&e), 1);
    }

    #[test]
    fn xml_header_contains_scanner() {
        let xml = e57_xml_header_v2("Faro", 100);
        assert!(xml.contains("Faro"));
        assert!(xml.contains("100"));
    }

    #[test]
    fn validate_fails_empty() {
        let e = new_e57_export("test");
        assert!(!validate_e57(&e));
    }

    #[test]
    fn validate_passes_with_point() {
        let mut e = new_e57_export("test");
        add_e57_point(&mut e, 0.0, 0.0, 0.0, 1.0);
        assert!(validate_e57(&e));
    }

    #[test]
    fn size_estimate_grows_with_points() {
        assert!(e57_size_estimate(100) > e57_size_estimate(10));
    }

    #[test]
    fn export_stub_starts_with_magic() {
        let mut e = new_e57_export("test");
        add_e57_point(&mut e, 0.0, 0.0, 0.0, 1.0);
        let bytes = export_e57_stub(&e);
        assert_eq!(&bytes[..4], b"ASTM");
    }

    #[test]
    fn from_positions_count() {
        let pos = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let e = e57_from_positions(&pos);
        assert_eq!(e57_point_count(&e), 2);
    }

    #[test]
    fn bbox_single_point() {
        let mut e = new_e57_export("test");
        add_e57_point(&mut e, 1.0, 2.0, 3.0, 1.0);
        let (mn, mx) = e57_bbox(&e);
        assert!((mn[0] - 1.0).abs() < 1e-10);
        assert!((mx[0] - 1.0).abs() < 1e-10);
    }
}
