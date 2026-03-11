// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Convert mesh coordinates between Y-up and Z-up conventions.

/// Coordinate convention.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpAxis {
    YUp,
    ZUp,
}

/// Convert a single position from Y-up to Z-up.
/// Y-up (x, y, z) -> Z-up (x, -z, y).
#[allow(dead_code)]
pub fn yup_to_zup(p: [f32; 3]) -> [f32; 3] {
    [p[0], -p[2], p[1]]
}

/// Convert a single position from Z-up to Y-up.
/// Z-up (x, y, z) -> Y-up (x, z, -y).
#[allow(dead_code)]
pub fn zup_to_yup(p: [f32; 3]) -> [f32; 3] {
    [p[0], p[2], -p[1]]
}

/// Convert all positions in a mesh from Y-up to Z-up.
#[allow(dead_code)]
pub fn convert_yup_to_zup(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    positions.iter().map(|&p| yup_to_zup(p)).collect()
}

/// Convert all positions in a mesh from Z-up to Y-up.
#[allow(dead_code)]
pub fn convert_zup_to_yup(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    positions.iter().map(|&p| zup_to_yup(p)).collect()
}

/// Dispatch conversion based on target convention.
#[allow(dead_code)]
pub fn convert_up_axis(positions: &[[f32; 3]], from: UpAxis, to: UpAxis) -> Vec<[f32; 3]> {
    match (from, to) {
        (UpAxis::YUp, UpAxis::ZUp) => convert_yup_to_zup(positions),
        (UpAxis::ZUp, UpAxis::YUp) => convert_zup_to_yup(positions),
        _ => positions.to_vec(),
    }
}

/// Check that a round-trip conversion is lossless (within tolerance).
#[allow(dead_code)]
pub fn round_trip_error(positions: &[[f32; 3]]) -> f32 {
    let zup = convert_yup_to_zup(positions);
    let back = convert_zup_to_yup(&zup);
    positions
        .iter()
        .zip(back.iter())
        .map(|(a, b)| {
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dz = a[2] - b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Compute bounding box of positions.
#[allow(dead_code)]
pub fn bounds_zup(positions: &[[f32; 3]]) -> Option<([f32; 3], [f32; 3])> {
    if positions.is_empty() {
        return None;
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for &p in positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    Some((mn, mx))
}

/// Convert normals from Y-up to Z-up (same rotation).
#[allow(dead_code)]
pub fn normals_yup_to_zup(normals: &[[f32; 3]]) -> Vec<[f32; 3]> {
    normals.iter().map(|&n| yup_to_zup(n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yup_to_zup_x_unchanged() {
        let p = yup_to_zup([5.0, 1.0, 2.0]);
        assert!((p[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_yup_to_zup_y_becomes_z() {
        let p = yup_to_zup([0.0, 3.0, 0.0]);
        assert!((p[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_zup_to_yup_roundtrip() {
        let orig = [1.0, 2.0, 3.0];
        let err = round_trip_error(&[orig]);
        assert!(err < 1e-5);
    }

    #[test]
    fn test_convert_yup_to_zup_count() {
        let pts = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = convert_yup_to_zup(&pts);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_convert_up_axis_identity() {
        let pts = vec![[1.0, 2.0, 3.0]];
        let r = convert_up_axis(&pts, UpAxis::YUp, UpAxis::YUp);
        assert_eq!(r[0], pts[0]);
    }

    #[test]
    fn test_bounds_zup_none_empty() {
        assert!(bounds_zup(&[]).is_none());
    }

    #[test]
    fn test_bounds_zup_some() {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]];
        let (mn, mx) = bounds_zup(&pts).unwrap();
        assert!((mx[0] - 1.0).abs() < 1e-6);
        assert!((mx[2] - 3.0).abs() < 1e-6);
        let _ = mn;
    }

    #[test]
    fn test_normals_yup_to_zup_count() {
        let normals = vec![[0.0, 1.0, 0.0]];
        let r = normals_yup_to_zup(&normals);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_convert_up_axis_zup_to_yup() {
        let pts = vec![[0.0, 0.0, 1.0]];
        let r = convert_up_axis(&pts, UpAxis::ZUp, UpAxis::YUp);
        // Z-up (0,0,1) -> Y-up (0,1,0)
        assert!((r[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_round_trip_empty() {
        let err = round_trip_error(&[]);
        assert!((err - 0.0).abs() < 1e-6);
    }
}
