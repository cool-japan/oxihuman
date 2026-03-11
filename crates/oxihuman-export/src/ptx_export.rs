// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! PTX (Leica scanner) point cloud format stub.

/// PTX scan header.
#[allow(dead_code)]
pub struct PtxHeader {
    pub cols: u32,
    pub rows: u32,
    pub scanner_pos: [f64; 3],
    pub scanner_axes: [[f64; 3]; 3],
}

/// PTX point.
#[allow(dead_code)]
pub struct PtxPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub intensity: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// PTX export (one scan).
#[allow(dead_code)]
pub struct PtxExport {
    pub header: PtxHeader,
    pub points: Vec<PtxPoint>,
}

/// Create a new PTX export.
#[allow(dead_code)]
pub fn new_ptx_export(cols: u32, rows: u32) -> PtxExport {
    PtxExport {
        header: PtxHeader {
            cols,
            rows,
            scanner_pos: [0.0; 3],
            scanner_axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        },
        points: Vec::new(),
    }
}

/// Add a point to PTX export.
#[allow(dead_code)]
pub fn add_ptx_point(export: &mut PtxExport, x: f64, y: f64, z: f64, intensity: f32) {
    export.points.push(PtxPoint {
        x,
        y,
        z,
        intensity,
        r: 255,
        g: 255,
        b: 255,
    });
}

/// Point count.
#[allow(dead_code)]
pub fn ptx_point_count(export: &PtxExport) -> usize {
    export.points.len()
}

/// Build PTX header string.
#[allow(dead_code)]
pub fn build_ptx_header_string(export: &PtxExport) -> String {
    format!(
        "{}\n{}\n{:.6} {:.6} {:.6}\n",
        export.header.cols,
        export.header.rows,
        export.header.scanner_pos[0],
        export.header.scanner_pos[1],
        export.header.scanner_pos[2],
    )
}

/// Validate PTX.
#[allow(dead_code)]
pub fn validate_ptx(export: &PtxExport) -> bool {
    export.header.cols > 0 && export.header.rows > 0
}

/// Export to string.
#[allow(dead_code)]
pub fn export_ptx_string(export: &PtxExport) -> String {
    let mut s = build_ptx_header_string(export);
    for p in &export.points {
        s.push_str(&format!(
            "{:.6} {:.6} {:.6} {:.3}\n",
            p.x, p.y, p.z, p.intensity
        ));
    }
    s
}

/// Load from positions.
#[allow(dead_code)]
pub fn ptx_from_positions(positions: &[[f32; 3]]) -> PtxExport {
    let n = positions.len() as u32;
    let mut e = new_ptx_export(n, 1);
    for &p in positions {
        add_ptx_point(&mut e, p[0] as f64, p[1] as f64, p[2] as f64, 1.0);
    }
    e
}

/// Estimate file size.
#[allow(dead_code)]
pub fn ptx_file_size_estimate(point_count: usize) -> usize {
    point_count * 50 + 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export() {
        let e = new_ptx_export(100, 100);
        assert_eq!(ptx_point_count(&e), 0);
    }

    #[test]
    fn add_point() {
        let mut e = new_ptx_export(100, 100);
        add_ptx_point(&mut e, 1.0, 2.0, 3.0, 0.5);
        assert_eq!(ptx_point_count(&e), 1);
    }

    #[test]
    fn header_contains_dims() {
        let e = new_ptx_export(128, 64);
        let h = build_ptx_header_string(&e);
        assert!(h.contains("128"));
        assert!(h.contains("64"));
    }

    #[test]
    fn validate_valid() {
        let e = new_ptx_export(10, 10);
        assert!(validate_ptx(&e));
    }

    #[test]
    fn validate_zero_dims_fails() {
        let e = new_ptx_export(0, 0);
        assert!(!validate_ptx(&e));
    }

    #[test]
    fn export_string_has_point() {
        let mut e = new_ptx_export(1, 1);
        add_ptx_point(&mut e, 1.0, 2.0, 3.0, 0.5);
        let s = export_ptx_string(&e);
        assert!(s.contains("1.000000"));
    }

    #[test]
    fn from_positions() {
        let pos = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let e = ptx_from_positions(&pos);
        assert_eq!(ptx_point_count(&e), 2);
    }

    #[test]
    fn file_size_estimate() {
        assert!(ptx_file_size_estimate(100) > 100);
    }

    #[test]
    fn header_rows_correct() {
        let e = new_ptx_export(100, 50);
        assert_eq!(e.header.rows, 50);
    }
}
