// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Signed distance field (SDF) export for mesh surfaces.

/// A 3D SDF grid export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DistanceFieldExport {
    pub dims: [usize; 3],
    pub voxel_size: f32,
    pub origin: [f32; 3],
    pub values: Vec<f32>,
}

/// Create a new SDF grid, initialized to +infinity.
#[allow(dead_code)]
pub fn new_distance_field(
    dims: [usize; 3],
    voxel_size: f32,
    origin: [f32; 3],
) -> DistanceFieldExport {
    let total = dims[0] * dims[1] * dims[2];
    DistanceFieldExport {
        dims,
        voxel_size,
        origin,
        values: vec![f32::MAX; total],
    }
}

/// Linear index for a voxel (ix, iy, iz).
#[allow(dead_code)]
pub fn df_index(dims: [usize; 3], ix: usize, iy: usize, iz: usize) -> Option<usize> {
    if ix < dims[0] && iy < dims[1] && iz < dims[2] {
        Some(ix + dims[0] * (iy + dims[1] * iz))
    } else {
        None
    }
}

/// Set SDF value at voxel.
#[allow(dead_code)]
pub fn set_df_value(df: &mut DistanceFieldExport, ix: usize, iy: usize, iz: usize, val: f32) {
    if let Some(idx) = df_index(df.dims, ix, iy, iz) {
        df.values[idx] = val;
    }
}

/// Get SDF value at voxel.
#[allow(dead_code)]
pub fn get_df_value(df: &DistanceFieldExport, ix: usize, iy: usize, iz: usize) -> Option<f32> {
    df_index(df.dims, ix, iy, iz).and_then(|idx| df.values.get(idx).copied())
}

/// Total voxel count.
#[allow(dead_code)]
pub fn df_voxel_count(df: &DistanceFieldExport) -> usize {
    df.values.len()
}

/// Count voxels with negative SDF (inside surface).
#[allow(dead_code)]
pub fn count_interior_voxels(df: &DistanceFieldExport) -> usize {
    df.values.iter().filter(|&&v| v < 0.0).count()
}

/// Minimum SDF value in the grid.
#[allow(dead_code)]
pub fn df_min_value(df: &DistanceFieldExport) -> f32 {
    df.values.iter().cloned().fold(f32::MAX, f32::min)
}

/// Maximum finite SDF value.
#[allow(dead_code)]
pub fn df_max_finite_value(df: &DistanceFieldExport) -> f32 {
    df.values
        .iter()
        .cloned()
        .filter(|v| v.is_finite())
        .fold(f32::MIN, f32::max)
}

/// Clamp all values to [-max_dist, +max_dist].
#[allow(dead_code)]
pub fn clamp_df(df: &mut DistanceFieldExport, max_dist: f32) {
    for v in &mut df.values {
        *v = v.clamp(-max_dist, max_dist);
    }
}

/// Export to JSON summary.
#[allow(dead_code)]
pub fn distance_field_to_json(df: &DistanceFieldExport) -> String {
    format!(
        "{{\"dims\":[{},{},{}],\"voxel_size\":{:.6},\"voxel_count\":{}}}",
        df.dims[0],
        df.dims[1],
        df.dims[2],
        df.voxel_size,
        df_voxel_count(df)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_df() -> DistanceFieldExport {
        new_distance_field([4, 4, 4], 0.25, [0.0; 3])
    }

    #[test]
    fn test_new_distance_field() {
        let df = small_df();
        assert_eq!(df_voxel_count(&df), 64);
    }

    #[test]
    fn test_df_index_valid() {
        let idx = df_index([4, 4, 4], 1, 2, 3);
        assert!(idx.is_some());
    }

    #[test]
    fn test_df_index_oob() {
        let idx = df_index([4, 4, 4], 4, 0, 0);
        assert!(idx.is_none());
    }

    #[test]
    fn test_set_get_df_value() {
        let mut df = small_df();
        set_df_value(&mut df, 1, 1, 1, -0.5);
        let v = get_df_value(&df, 1, 1, 1).unwrap();
        assert!((v + 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_count_interior_voxels() {
        let mut df = small_df();
        set_df_value(&mut df, 0, 0, 0, -1.0);
        assert_eq!(count_interior_voxels(&df), 1);
    }

    #[test]
    fn test_clamp_df() {
        let mut df = small_df();
        set_df_value(&mut df, 0, 0, 0, 100.0);
        clamp_df(&mut df, 10.0);
        assert!(get_df_value(&df, 0, 0, 0).unwrap() <= 10.0);
    }

    #[test]
    fn test_df_min_value() {
        let mut df = small_df();
        set_df_value(&mut df, 0, 0, 0, -2.0);
        assert!((df_min_value(&df) + 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_df_max_finite_value() {
        let mut df = new_distance_field([2, 2, 2], 1.0, [0.0; 3]);
        set_df_value(&mut df, 0, 0, 0, 5.0);
        clamp_df(&mut df, 10.0);
        assert!(df_max_finite_value(&df) >= 5.0);
    }

    #[test]
    fn test_distance_field_to_json() {
        let df = small_df();
        let j = distance_field_to_json(&df);
        assert!(j.contains("\"voxel_count\":64"));
    }

    #[test]
    fn test_get_df_value_oob() {
        let df = small_df();
        assert!(get_df_value(&df, 99, 0, 0).is_none());
    }
}
