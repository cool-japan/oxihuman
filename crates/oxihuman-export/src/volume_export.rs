// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Volumetric data export for OxiHuman.
//!
//! Supports multiple target formats (VDB, NanoVDB stub, raw binary, CSV) and
//! provides helpers for setting/getting individual voxels as well as computing
//! basic statistics.

// ── Types ─────────────────────────────────────────────────────────────────────

/// File format for a volume export.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum VolumeFormat {
    /// OpenVDB container (stub — requires external library).
    Vdb,
    /// NanoVDB container (stub — requires external library).
    Nvdb,
    /// Raw flat binary (little-endian `f32` voxels).
    Raw,
    /// Comma-separated values (one voxel per row: `x,y,z,value`).
    Csv,
}

/// Configuration for a volume export operation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeExportConfig {
    /// Target file format.
    pub format: VolumeFormat,
    /// Voxel grid dimensions `[width, height, depth]`.
    pub resolution: [u32; 3],
    /// World-space size of a single voxel.
    pub voxel_size: f32,
}

/// A dense voxel grid storing one `f32` value per voxel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeData {
    /// Flat voxel storage in `x + y*w + z*w*h` order.
    pub voxels: Vec<f32>,
    /// Grid dimensions `[width, height, depth]`.
    pub resolution: [u32; 3],
}

/// Result produced by [`export_volume_stub`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeExportResult {
    /// Configuration used for this export.
    pub config: VolumeExportConfig,
    /// Total number of voxels in the grid.
    pub total_voxels: usize,
    /// Number of voxels whose value differs from `0.0`.
    pub non_zero_voxels: usize,
    /// Rough byte estimate for the resulting file.
    pub byte_estimate: usize,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default [`VolumeExportConfig`].
#[allow(dead_code)]
pub fn default_volume_export_config() -> VolumeExportConfig {
    VolumeExportConfig {
        format: VolumeFormat::Raw,
        resolution: [64, 64, 64],
        voxel_size: 0.01,
    }
}

/// Allocate a zeroed [`VolumeData`] for the given `res` dimensions.
#[allow(dead_code)]
pub fn new_volume_data(res: [u32; 3]) -> VolumeData {
    let total = res[0] as usize * res[1] as usize * res[2] as usize;
    VolumeData {
        voxels: vec![0.0_f32; total],
        resolution: res,
    }
}

/// Set the value of a single voxel at `(x, y, z)`.
///
/// Does nothing if the coordinates are out of bounds.
#[allow(dead_code)]
pub fn volume_set_voxel(vol: &mut VolumeData, x: u32, y: u32, z: u32, val: f32) {
    let [w, h, _d] = vol.resolution;
    if x >= w || y >= h || z >= _d {
        return;
    }
    let idx = x as usize + y as usize * w as usize + z as usize * w as usize * h as usize;
    if idx < vol.voxels.len() {
        vol.voxels[idx] = val;
    }
}

/// Get the value of a single voxel at `(x, y, z)`.
///
/// Returns `0.0` if the coordinates are out of bounds.
#[allow(dead_code)]
pub fn volume_get_voxel(vol: &VolumeData, x: u32, y: u32, z: u32) -> f32 {
    let [w, h, d] = vol.resolution;
    if x >= w || y >= h || z >= d {
        return 0.0;
    }
    let idx = x as usize + y as usize * w as usize + z as usize * w as usize * h as usize;
    vol.voxels.get(idx).copied().unwrap_or(0.0)
}

/// Stub export — computes statistics and returns an estimated result.
///
/// No actual file I/O is performed.
#[allow(dead_code)]
pub fn export_volume_stub(vol: &VolumeData, cfg: &VolumeExportConfig) -> VolumeExportResult {
    let total_voxels = vol.voxels.len();
    let non_zero_voxels = count_non_zero_voxels(vol);

    let byte_estimate = match &cfg.format {
        VolumeFormat::Raw => total_voxels * 4,
        VolumeFormat::Nvdb | VolumeFormat::Vdb => total_voxels * 6, // rough overhead
        VolumeFormat::Csv => total_voxels * 24, // ~24 chars per row estimate
    };

    VolumeExportResult {
        config: cfg.clone(),
        total_voxels,
        non_zero_voxels,
        byte_estimate,
    }
}

/// Return the canonical name string for a [`VolumeFormat`].
#[allow(dead_code)]
pub fn volume_format_name(fmt: &VolumeFormat) -> &'static str {
    match fmt {
        VolumeFormat::Vdb => "vdb",
        VolumeFormat::Nvdb => "nvdb",
        VolumeFormat::Raw => "raw",
        VolumeFormat::Csv => "csv",
    }
}

/// Serialise a [`VolumeExportResult`] to a compact JSON string.
#[allow(dead_code)]
pub fn volume_result_to_json(r: &VolumeExportResult) -> String {
    let [w, h, d] = r.config.resolution;
    format!(
        r#"{{"format":"{}","resolution":[{},{},{}],"total_voxels":{},"non_zero_voxels":{},"byte_estimate":{}}}"#,
        volume_format_name(&r.config.format),
        w, h, d,
        r.total_voxels,
        r.non_zero_voxels,
        r.byte_estimate,
    )
}

/// Count the number of voxels in `vol` that are not exactly `0.0`.
#[allow(dead_code)]
pub fn count_non_zero_voxels(vol: &VolumeData) -> usize {
    vol.voxels.iter().filter(|&&v| v != 0.0).count()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_fields() {
        let cfg = default_volume_export_config();
        assert_eq!(cfg.format, VolumeFormat::Raw);
        assert_eq!(cfg.resolution, [64, 64, 64]);
        assert!((cfg.voxel_size - 0.01).abs() < f32::EPSILON);
    }

    #[test]
    fn new_volume_data_all_zeros() {
        let vol = new_volume_data([4, 4, 4]);
        assert_eq!(vol.voxels.len(), 64);
        assert!(vol.voxels.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn set_and_get_voxel() {
        let mut vol = new_volume_data([8, 8, 8]);
        volume_set_voxel(&mut vol, 1, 2, 3, 1.5);
        assert!((volume_get_voxel(&vol, 1, 2, 3) - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn get_voxel_out_of_bounds_returns_zero() {
        let vol = new_volume_data([4, 4, 4]);
        assert_eq!(volume_get_voxel(&vol, 99, 0, 0), 0.0);
    }

    #[test]
    fn set_voxel_out_of_bounds_noop() {
        let mut vol = new_volume_data([4, 4, 4]);
        volume_set_voxel(&mut vol, 99, 0, 0, 9.0); // should not panic
        assert_eq!(count_non_zero_voxels(&vol), 0);
    }

    #[test]
    fn count_non_zero_voxels_basic() {
        let mut vol = new_volume_data([2, 2, 2]);
        volume_set_voxel(&mut vol, 0, 0, 0, 1.0);
        volume_set_voxel(&mut vol, 1, 1, 1, 2.0);
        assert_eq!(count_non_zero_voxels(&vol), 2);
    }

    #[test]
    fn export_stub_raw_byte_estimate() {
        let vol = new_volume_data([4, 4, 4]);
        let cfg = default_volume_export_config(); // Raw, 64^3 but we use 4^3
        let r = export_volume_stub(&vol, &cfg);
        // 64 voxels * 4 bytes
        assert_eq!(r.byte_estimate, 64 * 4);
        assert_eq!(r.total_voxels, 64);
    }

    #[test]
    fn export_stub_csv_estimate_larger() {
        let vol = new_volume_data([4, 4, 4]);
        let cfg = VolumeExportConfig {
            format: VolumeFormat::Csv,
            resolution: [4, 4, 4],
            voxel_size: 0.01,
        };
        let r = export_volume_stub(&vol, &cfg);
        assert!(r.byte_estimate > r.total_voxels * 4);
    }

    #[test]
    fn volume_format_names() {
        assert_eq!(volume_format_name(&VolumeFormat::Vdb), "vdb");
        assert_eq!(volume_format_name(&VolumeFormat::Nvdb), "nvdb");
        assert_eq!(volume_format_name(&VolumeFormat::Raw), "raw");
        assert_eq!(volume_format_name(&VolumeFormat::Csv), "csv");
    }

    #[test]
    fn result_to_json_contains_fields() {
        let vol = new_volume_data([2, 2, 2]);
        let cfg = default_volume_export_config();
        let r = export_volume_stub(&vol, &cfg);
        let json = volume_result_to_json(&r);
        assert!(json.contains("format"));
        assert!(json.contains("raw"));
        assert!(json.contains("total_voxels"));
    }
}
