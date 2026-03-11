// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LAS lidar point cloud format export stub.

pub const LAS_MAGIC: &[u8; 4] = b"LASF";

/// LAS point data format 0 (basic XYZ + intensity).
#[allow(dead_code)]
pub struct LasPointV2 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub intensity: u16,
    pub return_number: u8,
    pub classification: u8,
}

/// LAS 1.4 header stub.
#[allow(dead_code)]
pub struct LasHeaderV2 {
    pub version_major: u8,
    pub version_minor: u8,
    pub point_data_format: u8,
    pub point_count: u64,
    pub scale: [f64; 3],
    pub offset: [f64; 3],
}

/// LAS export bundle.
#[allow(dead_code)]
pub struct LasExport {
    pub header: LasHeaderV2,
    pub points: Vec<LasPointV2>,
}

/// Create a new LAS export.
#[allow(dead_code)]
pub fn new_las_export(scale: f64) -> LasExport {
    LasExport {
        header: LasHeaderV2 {
            version_major: 1,
            version_minor: 4,
            point_data_format: 0,
            point_count: 0,
            scale: [scale; 3],
            offset: [0.0; 3],
        },
        points: Vec::new(),
    }
}

/// Add a point (world coordinates converted to integer).
#[allow(dead_code)]
pub fn add_las_point(export: &mut LasExport, x: f64, y: f64, z: f64, intensity: u16) {
    let xi = ((x - export.header.offset[0]) / export.header.scale[0]) as i32;
    let yi = ((y - export.header.offset[1]) / export.header.scale[1]) as i32;
    let zi = ((z - export.header.offset[2]) / export.header.scale[2]) as i32;
    export.points.push(LasPointV2 {
        x: xi,
        y: yi,
        z: zi,
        intensity,
        return_number: 1,
        classification: 0,
    });
    export.header.point_count += 1;
}

/// Point count.
#[allow(dead_code)]
pub fn las_point_count_v2(export: &LasExport) -> u64 {
    export.header.point_count
}

/// Build LAS header bytes (simplified).
#[allow(dead_code)]
pub fn build_las_header_bytes(export: &LasExport) -> Vec<u8> {
    let mut buf = LAS_MAGIC.to_vec();
    buf.extend_from_slice(&[export.header.version_major, export.header.version_minor]);
    buf.extend_from_slice(&export.header.point_count.to_le_bytes());
    buf
}

/// Validate LAS export.
#[allow(dead_code)]
pub fn validate_las(export: &LasExport) -> bool {
    export.header.point_count == export.points.len() as u64
        && export.header.scale.iter().all(|&s| s > 0.0)
}

/// Estimate file size.
#[allow(dead_code)]
pub fn las_file_size_estimate_v2(point_count: u64) -> usize {
    375 + point_count as usize * 20
}

/// Load from f32 positions.
#[allow(dead_code)]
pub fn las_from_positions(positions: &[[f32; 3]], scale: f64) -> LasExport {
    let mut e = new_las_export(scale);
    for &p in positions {
        add_las_point(&mut e, p[0] as f64, p[1] as f64, p[2] as f64, 0);
    }
    e
}

/// Get world X from integer record.
#[allow(dead_code)]
pub fn las_world_x(point: &LasPointV2, header: &LasHeaderV2) -> f64 {
    point.x as f64 * header.scale[0] + header.offset[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_correct() {
        assert_eq!(&LAS_MAGIC[..], b"LASF");
    }

    #[test]
    fn new_export_zero_points() {
        let e = new_las_export(0.001);
        assert_eq!(las_point_count_v2(&e), 0);
    }

    #[test]
    fn add_point_increments_count() {
        let mut e = new_las_export(0.001);
        add_las_point(&mut e, 1.0, 2.0, 3.0, 100);
        assert_eq!(las_point_count_v2(&e), 1);
    }

    #[test]
    fn validate_passes() {
        let mut e = new_las_export(0.001);
        add_las_point(&mut e, 0.0, 0.0, 0.0, 0);
        assert!(validate_las(&e));
    }

    #[test]
    fn header_bytes_start_with_magic() {
        let e = new_las_export(0.001);
        let bytes = build_las_header_bytes(&e);
        assert_eq!(&bytes[..4], b"LASF");
    }

    #[test]
    fn file_size_grows() {
        assert!(las_file_size_estimate_v2(1000) > las_file_size_estimate_v2(100));
    }

    #[test]
    fn from_positions_count() {
        let pos = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let e = las_from_positions(&pos, 0.001);
        assert_eq!(las_point_count_v2(&e), 2);
    }

    #[test]
    fn world_x_round_trip() {
        let mut e = new_las_export(0.001);
        add_las_point(&mut e, 1.0, 0.0, 0.0, 0);
        let wx = las_world_x(&e.points[0], &e.header);
        assert!((wx - 1.0).abs() < 0.01);
    }

    #[test]
    fn version_correct() {
        let e = new_las_export(0.001);
        assert_eq!(e.header.version_major, 1);
        assert_eq!(e.header.version_minor, 4);
    }
}
