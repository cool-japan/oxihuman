// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! FLS (Faro scan) format stub export.

pub const FLS_VERSION: u32 = 1;

/// FLS scan metadata.
#[allow(dead_code)]
pub struct FlsMetadata {
    pub scanner_serial: String,
    pub scan_date: String,
    pub point_count: u64,
}

/// FLS export stub.
#[allow(dead_code)]
pub struct FlsExport {
    pub metadata: FlsMetadata,
    pub points: Vec<[f32; 3]>,
    pub reflectances: Vec<f32>,
}

/// Create a new FLS export.
#[allow(dead_code)]
pub fn new_fls_export(scanner_serial: &str) -> FlsExport {
    FlsExport {
        metadata: FlsMetadata {
            scanner_serial: scanner_serial.to_string(),
            scan_date: "2026-01-01".to_string(),
            point_count: 0,
        },
        points: Vec::new(),
        reflectances: Vec::new(),
    }
}

/// Add a point.
#[allow(dead_code)]
pub fn add_fls_point(export: &mut FlsExport, x: f32, y: f32, z: f32, reflectance: f32) {
    export.points.push([x, y, z]);
    export.reflectances.push(reflectance);
    export.metadata.point_count += 1;
}

/// Point count.
#[allow(dead_code)]
pub fn fls_point_count(export: &FlsExport) -> u64 {
    export.metadata.point_count
}

/// Validate FLS.
#[allow(dead_code)]
pub fn validate_fls(export: &FlsExport) -> bool {
    export.metadata.point_count == export.points.len() as u64
        && !export.metadata.scanner_serial.is_empty()
}

/// Build FLS binary header stub (just version + count).
#[allow(dead_code)]
pub fn build_fls_header_bytes(export: &FlsExport) -> Vec<u8> {
    let mut buf = FLS_VERSION.to_le_bytes().to_vec();
    buf.extend_from_slice(&export.metadata.point_count.to_le_bytes());
    buf
}

/// Export to bytes stub.
#[allow(dead_code)]
pub fn export_fls_stub(export: &FlsExport) -> Vec<u8> {
    let mut buf = build_fls_header_bytes(export);
    for &p in &export.points {
        buf.extend_from_slice(&p[0].to_le_bytes());
        buf.extend_from_slice(&p[1].to_le_bytes());
        buf.extend_from_slice(&p[2].to_le_bytes());
    }
    buf
}

/// Load from positions.
#[allow(dead_code)]
pub fn fls_from_positions(positions: &[[f32; 3]]) -> FlsExport {
    let mut e = new_fls_export("FARO-X130");
    for &p in positions {
        add_fls_point(&mut e, p[0], p[1], p[2], 0.5);
    }
    e
}

/// Average reflectance.
#[allow(dead_code)]
pub fn fls_avg_reflectance(export: &FlsExport) -> f32 {
    if export.reflectances.is_empty() { return 0.0; }
    export.reflectances.iter().sum::<f32>() / export.reflectances.len() as f32
}

/// Estimate file size.
#[allow(dead_code)]
pub fn fls_file_size_estimate(point_count: u64) -> usize {
    12 + point_count as usize * 16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constant() {
        assert_eq!(FLS_VERSION, 1);
    }

    #[test]
    fn new_empty() {
        let e = new_fls_export("FARO");
        assert_eq!(fls_point_count(&e), 0);
    }

    #[test]
    fn add_point() {
        let mut e = new_fls_export("FARO");
        add_fls_point(&mut e, 1.0, 2.0, 3.0, 0.8);
        assert_eq!(fls_point_count(&e), 1);
    }

    #[test]
    fn validate_passes() {
        let e = fls_from_positions(&[[0.0f32,0.0,0.0]]);
        assert!(validate_fls(&e));
    }

    #[test]
    fn validate_fails_empty_serial() {
        let mut e = new_fls_export("");
        add_fls_point(&mut e, 0.0, 0.0, 0.0, 1.0);
        assert!(!validate_fls(&e));
    }

    #[test]
    fn header_bytes_correct_size() {
        let e = new_fls_export("FARO");
        let b = build_fls_header_bytes(&e);
        assert_eq!(b.len(), 12);
    }

    #[test]
    fn avg_reflectance() {
        let mut e = new_fls_export("FARO");
        add_fls_point(&mut e, 0.0, 0.0, 0.0, 0.5);
        add_fls_point(&mut e, 1.0, 0.0, 0.0, 1.0);
        let avg = fls_avg_reflectance(&e);
        assert!((avg - 0.75).abs() < 1e-5);
    }

    #[test]
    fn file_size_grows() {
        assert!(fls_file_size_estimate(100) > fls_file_size_estimate(10));
    }

    #[test]
    fn from_positions() {
        let pos = vec![[1.0f32,2.0,3.0],[4.0,5.0,6.0]];
        let e = fls_from_positions(&pos);
        assert_eq!(fls_point_count(&e), 2);
    }
}
