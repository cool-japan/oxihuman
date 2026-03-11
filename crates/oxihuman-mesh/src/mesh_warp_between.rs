// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Warp deform between two reference objects (from/to spaces).

/// Per-vertex warp record pairing source and destination positions.
#[derive(Debug, Clone, Default)]
pub struct WarpBetweenRecord {
    pub source_pos: [f32; 3],
    pub target_pos: [f32; 3],
    pub weight: f32,
}

impl WarpBetweenRecord {
    pub fn new(source: [f32; 3], target: [f32; 3], weight: f32) -> Self {
        Self { source_pos: source, target_pos: target, weight }
    }
}

/// Configuration for between-object warp.
#[derive(Debug, Clone)]
pub struct WarpBetweenConfig {
    pub falloff_radius: f32,
    pub strength: f32,
    pub use_object_origin: bool,
}

impl Default for WarpBetweenConfig {
    fn default() -> Self {
        Self { falloff_radius: 1.0, strength: 1.0, use_object_origin: true }
    }
}

impl WarpBetweenConfig {
    pub fn new(falloff: f32, strength: f32) -> Self {
        Self { falloff_radius: falloff, strength, use_object_origin: true }
    }
}

/// Compute the interpolated position for a vertex given warp records.
pub fn warp_between_vertex(
    pos: [f32; 3],
    records: &[WarpBetweenRecord],
    cfg: &WarpBetweenConfig,
) -> [f32; 3] {
    let mut disp = [0.0_f32; 3];
    let mut weight_sum = 0.0_f32;
    for rec in records {
        let dx = pos[0] - rec.source_pos[0];
        let dy = pos[1] - rec.source_pos[1];
        let dz = pos[2] - rec.source_pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        let t = if cfg.falloff_radius > 0.0 {
            1.0 - (dist / cfg.falloff_radius).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let w = rec.weight * t;
        let delta = [
            rec.target_pos[0] - rec.source_pos[0],
            rec.target_pos[1] - rec.source_pos[1],
            rec.target_pos[2] - rec.source_pos[2],
        ];
        disp[0] += delta[0] * w;
        disp[1] += delta[1] * w;
        disp[2] += delta[2] * w;
        weight_sum += w;
    }
    if weight_sum > 1e-8 {
        let s = cfg.strength / weight_sum;
        [pos[0] + disp[0] * s, pos[1] + disp[1] * s, pos[2] + disp[2] * s]
    } else {
        pos
    }
}

/// Apply between-object warp to a set of positions.
pub fn apply_warp_between(
    positions: &mut [[f32; 3]],
    records: &[WarpBetweenRecord],
    cfg: &WarpBetweenConfig,
) {
    for p in positions.iter_mut() {
        *p = warp_between_vertex(*p, records, cfg);
    }
}

/// Validate warp-between config.
pub fn validate_warp_between_config(cfg: &WarpBetweenConfig) -> bool {
    cfg.falloff_radius >= 0.0 && (0.0..=1.0).contains(&cfg.strength)
}

/// Compute the total displacement magnitude from source to target.
pub fn warp_record_delta_length(rec: &WarpBetweenRecord) -> f32 {
    let d = [
        rec.target_pos[0] - rec.source_pos[0],
        rec.target_pos[1] - rec.source_pos[1],
        rec.target_pos[2] - rec.source_pos[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warp_between_record_new() {
        let r = WarpBetweenRecord::new([0.0; 3], [1.0, 0.0, 0.0], 1.0);
        assert!((r.weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_warp_between_config_default() {
        let cfg = WarpBetweenConfig::default();
        assert!((cfg.strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_warp_between_vertex_no_records() {
        let pos = [1.0_f32, 2.0, 3.0];
        let cfg = WarpBetweenConfig::default();
        let out = warp_between_vertex(pos, &[], &cfg);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_warp_between_vertex_single_record_at_source() {
        let pos = [0.0_f32, 0.0, 0.0];
        let rec = vec![WarpBetweenRecord::new([0.0; 3], [1.0, 0.0, 0.0], 1.0)];
        let cfg = WarpBetweenConfig::new(0.0, 1.0);
        let out = warp_between_vertex(pos, &rec, &cfg);
        assert!((out[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_apply_warp_between_preserves_count() {
        let mut pos = vec![[0.0_f32; 3]; 5];
        let records: Vec<WarpBetweenRecord> = vec![];
        let cfg = WarpBetweenConfig::default();
        apply_warp_between(&mut pos, &records, &cfg);
        assert_eq!(pos.len(), 5);
    }

    #[test]
    fn test_validate_warp_between_config_valid() {
        let cfg = WarpBetweenConfig::default();
        assert!(validate_warp_between_config(&cfg));
    }

    #[test]
    fn test_validate_warp_between_config_invalid_strength() {
        let cfg = WarpBetweenConfig { strength: 2.0, ..WarpBetweenConfig::default() };
        assert!(!validate_warp_between_config(&cfg));
    }

    #[test]
    fn test_warp_record_delta_length() {
        let rec = WarpBetweenRecord::new([0.0; 3], [3.0, 4.0, 0.0], 1.0);
        assert!((warp_record_delta_length(&rec) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_warp_record_delta_length_zero() {
        let rec = WarpBetweenRecord::new([1.0; 3], [1.0; 3], 1.0);
        assert!(warp_record_delta_length(&rec).abs() < 1e-5);
    }
}
