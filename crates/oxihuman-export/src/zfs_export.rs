// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ZFS (Zoller+Fröhlich) format stub export.

pub const ZFS_MAGIC: &[u8; 8] = b"ZFS_SCAN";

/// ZFS scan header.
#[allow(dead_code)]
pub struct ZfsHeader {
    pub version: u32,
    pub point_count: u64,
    pub scanner_model: String,
}

/// ZFS export stub.
#[allow(dead_code)]
pub struct ZfsExport {
    pub header: ZfsHeader,
    pub points: Vec<[f32; 4]>,
}

/// Create a new ZFS export.
#[allow(dead_code)]
pub fn new_zfs_export(scanner_model: &str) -> ZfsExport {
    ZfsExport {
        header: ZfsHeader {
            version: 1,
            point_count: 0,
            scanner_model: scanner_model.to_string(),
        },
        points: Vec::new(),
    }
}

/// Add a point (x,y,z,intensity).
#[allow(dead_code)]
pub fn add_zfs_point(export: &mut ZfsExport, x: f32, y: f32, z: f32, intensity: f32) {
    export.points.push([x, y, z, intensity]);
    export.header.point_count += 1;
}

/// Point count.
#[allow(dead_code)]
pub fn zfs_point_count(export: &ZfsExport) -> u64 {
    export.header.point_count
}

/// Validate.
#[allow(dead_code)]
pub fn validate_zfs(export: &ZfsExport) -> bool {
    export.header.point_count == export.points.len() as u64
        && !export.header.scanner_model.is_empty()
}

/// Build binary header stub.
#[allow(dead_code)]
pub fn build_zfs_header_bytes(export: &ZfsExport) -> Vec<u8> {
    let mut buf = ZFS_MAGIC.to_vec();
    buf.extend_from_slice(&export.header.version.to_le_bytes());
    buf.extend_from_slice(&export.header.point_count.to_le_bytes());
    buf
}

/// Export stub.
#[allow(dead_code)]
pub fn export_zfs_stub(export: &ZfsExport) -> Vec<u8> {
    let mut buf = build_zfs_header_bytes(export);
    for &p in &export.points {
        for v in &p {
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }
    buf
}

/// Load from positions.
#[allow(dead_code)]
pub fn zfs_from_positions(positions: &[[f32; 3]]) -> ZfsExport {
    let mut e = new_zfs_export("IMAGER5010");
    for &p in positions {
        add_zfs_point(&mut e, p[0], p[1], p[2], 1.0);
    }
    e
}

/// Max intensity.
#[allow(dead_code)]
pub fn zfs_max_intensity(export: &ZfsExport) -> f32 {
    export.points.iter().map(|p| p[3]).fold(f32::NEG_INFINITY, f32::max)
}

/// Estimate file size.
#[allow(dead_code)]
pub fn zfs_file_size_estimate(point_count: u64) -> usize {
    20 + point_count as usize * 16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_length() {
        assert_eq!(ZFS_MAGIC.len(), 8);
    }

    #[test]
    fn new_empty() {
        let e = new_zfs_export("IMAGER");
        assert_eq!(zfs_point_count(&e), 0);
    }

    #[test]
    fn add_point() {
        let mut e = new_zfs_export("IMAGER");
        add_zfs_point(&mut e, 1.0, 2.0, 3.0, 0.8);
        assert_eq!(zfs_point_count(&e), 1);
    }

    #[test]
    fn validate_passes() {
        let e = zfs_from_positions(&[[0.0f32,0.0,0.0]]);
        assert!(validate_zfs(&e));
    }

    #[test]
    fn header_bytes_start_with_magic() {
        let e = new_zfs_export("test");
        let b = build_zfs_header_bytes(&e);
        assert_eq!(&b[..8], b"ZFS_SCAN");
    }

    #[test]
    fn export_stub_grows_with_points() {
        let mut e = new_zfs_export("test");
        add_zfs_point(&mut e, 0.0, 0.0, 0.0, 1.0);
        let b1 = export_zfs_stub(&e);
        add_zfs_point(&mut e, 1.0, 0.0, 0.0, 1.0);
        let b2 = export_zfs_stub(&e);
        assert!(b2.len() > b1.len());
    }

    #[test]
    fn max_intensity() {
        let mut e = new_zfs_export("test");
        add_zfs_point(&mut e, 0.0, 0.0, 0.0, 0.5);
        add_zfs_point(&mut e, 1.0, 0.0, 0.0, 0.9);
        let mx = zfs_max_intensity(&e);
        assert!((mx - 0.9).abs() < 1e-5);
    }

    #[test]
    fn file_size_estimate() {
        assert!(zfs_file_size_estimate(100) > zfs_file_size_estimate(10));
    }

    #[test]
    fn from_positions_count() {
        let pos = vec![[1.0f32,2.0,3.0],[4.0,5.0,6.0]];
        let e = zfs_from_positions(&pos);
        assert_eq!(zfs_point_count(&e), 2);
    }
}
