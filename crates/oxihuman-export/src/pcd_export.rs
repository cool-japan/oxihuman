// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! PCD (Point Cloud Data) format export.

/// PCD data type.
#[allow(dead_code)]
pub enum PcdDataType {
    Ascii,
    Binary,
}

/// PCD export structure.
#[allow(dead_code)]
pub struct PcdExport {
    pub points: Vec<[f32; 3]>,
    pub data_type: PcdDataType,
    pub fields: Vec<String>,
}

/// Create a new PCD export.
#[allow(dead_code)]
pub fn new_pcd_export() -> PcdExport {
    PcdExport {
        points: Vec::new(),
        data_type: PcdDataType::Ascii,
        fields: vec!["x".to_string(), "y".to_string(), "z".to_string()],
    }
}

/// Add a point.
#[allow(dead_code)]
pub fn add_pcd_point(export: &mut PcdExport, x: f32, y: f32, z: f32) {
    export.points.push([x, y, z]);
}

/// Point count.
#[allow(dead_code)]
pub fn pcd_point_count(export: &PcdExport) -> usize {
    export.points.len()
}

/// Build PCD header string.
#[allow(dead_code)]
pub fn build_pcd_header(export: &PcdExport) -> String {
    let fields = export.fields.join(" ");
    format!(
        "VERSION 0.7\nFIELDS {}\nSIZE 4 4 4\nTYPE F F F\nCOUNT 1 1 1\nWIDTH {}\nHEIGHT 1\nPOINTS {}\nDATA ascii\n",
        fields,
        export.points.len(),
        export.points.len()
    )
}

/// Export to ASCII PCD string.
#[allow(dead_code)]
pub fn export_pcd_ascii(export: &PcdExport) -> String {
    let mut s = build_pcd_header(export);
    for &p in &export.points {
        s.push_str(&format!("{} {} {}\n", p[0], p[1], p[2]));
    }
    s
}

/// Export to binary bytes stub.
#[allow(dead_code)]
pub fn export_pcd_binary(export: &PcdExport) -> Vec<u8> {
    let mut buf = Vec::new();
    for &p in &export.points {
        buf.extend_from_slice(&p[0].to_le_bytes());
        buf.extend_from_slice(&p[1].to_le_bytes());
        buf.extend_from_slice(&p[2].to_le_bytes());
    }
    buf
}

/// Validate PCD export.
#[allow(dead_code)]
pub fn validate_pcd(export: &PcdExport) -> bool {
    !export.points.is_empty() && !export.fields.is_empty()
}

/// Load from positions.
#[allow(dead_code)]
pub fn pcd_from_positions(positions: &[[f32; 3]]) -> PcdExport {
    let mut e = new_pcd_export();
    for &p in positions {
        add_pcd_point(&mut e, p[0], p[1], p[2]);
    }
    e
}

/// Compute centroid.
#[allow(dead_code)]
pub fn pcd_centroid(export: &PcdExport) -> [f32; 3] {
    if export.points.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for &p in &export.points {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    let n = export.points.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_pcd_export();
        assert_eq!(pcd_point_count(&e), 0);
    }

    #[test]
    fn add_point() {
        let mut e = new_pcd_export();
        add_pcd_point(&mut e, 1.0, 2.0, 3.0);
        assert_eq!(pcd_point_count(&e), 1);
    }

    #[test]
    fn header_contains_version() {
        let e = new_pcd_export();
        let h = build_pcd_header(&e);
        assert!(h.contains("VERSION 0.7"));
    }

    #[test]
    fn ascii_export_contains_coords() {
        let mut e = new_pcd_export();
        add_pcd_point(&mut e, 1.5, 2.5, 3.5);
        let s = export_pcd_ascii(&e);
        assert!(s.contains("1.5"));
    }

    #[test]
    fn binary_export_size() {
        let mut e = new_pcd_export();
        add_pcd_point(&mut e, 0.0, 0.0, 0.0);
        let b = export_pcd_binary(&e);
        assert_eq!(b.len(), 12);
    }

    #[test]
    fn validate_fails_empty() {
        let e = new_pcd_export();
        assert!(!validate_pcd(&e));
    }

    #[test]
    fn validate_passes() {
        let mut e = new_pcd_export();
        add_pcd_point(&mut e, 0.0, 0.0, 0.0);
        assert!(validate_pcd(&e));
    }

    #[test]
    fn from_positions() {
        let pos = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let e = pcd_from_positions(&pos);
        assert_eq!(pcd_point_count(&e), 2);
    }

    #[test]
    fn centroid_correct() {
        let pos = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let e = pcd_from_positions(&pos);
        let c = pcd_centroid(&e);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }
}
