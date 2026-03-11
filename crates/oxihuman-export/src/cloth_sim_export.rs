// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export cloth simulation state (vertex positions per frame) as a cache file.

#![allow(dead_code)]

/// Configuration for cloth simulation export.
#[derive(Debug, Clone)]
pub struct ClothSimExportConfig {
    /// Float precision in JSON.
    pub precision: usize,
    /// Whether to include vertex velocities in the output.
    pub include_velocity: bool,
}

/// A single frame of cloth simulation data.
#[derive(Debug, Clone)]
pub struct ClothFrame {
    /// Frame index.
    pub index: u32,
    /// Flat vertex positions: [x0, y0, z0, x1, y1, z1, ...].
    pub positions: Vec<f32>,
    /// Flat vertex velocities (optional): [vx0, vy0, vz0, ...].
    pub velocities: Vec<f32>,
}

/// Accumulated cloth simulation export state.
#[derive(Debug, Clone)]
pub struct ClothSimExportResult {
    /// All cloth frames.
    pub frames: Vec<ClothFrame>,
    /// Byte count from the last export.
    pub total_bytes: usize,
}

/// Returns the default [`ClothSimExportConfig`].
pub fn default_cloth_sim_export_config() -> ClothSimExportConfig {
    ClothSimExportConfig {
        precision: 6,
        include_velocity: false,
    }
}

/// Creates a new, empty [`ClothSimExportResult`].
pub fn new_cloth_sim_export() -> ClothSimExportResult {
    ClothSimExportResult {
        frames: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds a cloth frame.
pub fn cloth_sim_add_frame(result: &mut ClothSimExportResult, frame: ClothFrame) {
    result.frames.push(frame);
}

/// Returns the number of frames.
pub fn cloth_sim_frame_count(result: &ClothSimExportResult) -> usize {
    result.frames.len()
}

/// Returns vertex count per frame (from first frame, or 0).
pub fn cloth_sim_vertex_count(result: &ClothSimExportResult) -> usize {
    result
        .frames
        .first()
        .map(|f| f.positions.len() / 3)
        .unwrap_or(0)
}

/// Serialises all frames as JSON.
pub fn cloth_sim_export_to_json(
    result: &ClothSimExportResult,
    cfg: &ClothSimExportConfig,
) -> String {
    let prec = cfg.precision;
    let mut out = String::from("{\"frames\":[\n");
    for (fi, frame) in result.frames.iter().enumerate() {
        let comma = if fi + 1 < result.frames.len() { "," } else { "" };
        out.push_str(&format!("  {{\"index\":{},\"positions\":[", frame.index));
        let pos_strs: Vec<String> = frame
            .positions
            .chunks(3)
            .map(|p| format!("[{:.prec$},{:.prec$},{:.prec$}]", p[0], p[1], p[2]))
            .collect();
        out.push_str(&pos_strs.join(","));
        out.push(']');
        if cfg.include_velocity && !frame.velocities.is_empty() {
            out.push_str(",\"velocities\":[");
            let vel_strs: Vec<String> = frame
                .velocities
                .chunks(3)
                .map(|v| format!("[{:.prec$},{:.prec$},{:.prec$}]", v[0], v[1], v[2]))
                .collect();
            out.push_str(&vel_strs.join(","));
            out.push(']');
        }
        out.push('}');
        out.push_str(comma);
        out.push('\n');
    }
    out.push_str("]}");
    out
}

/// Serialises all frames as raw little-endian float32 bytes.
pub fn cloth_sim_export_to_binary(
    result: &ClothSimExportResult,
    cfg: &ClothSimExportConfig,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    for frame in &result.frames {
        for &v in &frame.positions {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        if cfg.include_velocity {
            for &v in &frame.velocities {
                buf.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    buf
}

/// Returns total byte count of the last export.
pub fn cloth_sim_total_bytes(result: &ClothSimExportResult) -> usize {
    result.total_bytes
}

/// Writes JSON to a file path (stub – returns byte count).
pub fn cloth_sim_write_to_file(
    result: &mut ClothSimExportResult,
    cfg: &ClothSimExportConfig,
    _path: &str,
) -> usize {
    let json = cloth_sim_export_to_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Clears all frames and resets state.
pub fn cloth_sim_clear(result: &mut ClothSimExportResult) {
    result.frames.clear();
    result.total_bytes = 0;
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_frame(index: u32, verts: u32) -> ClothFrame {
    let mut positions = Vec::new();
    for i in 0..verts {
        let f = i as f32;
        positions.push(f);
        positions.push(f * 0.5);
        positions.push(0.0);
    }
    ClothFrame {
        index,
        positions,
        velocities: Vec::new(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_cloth_sim_export_config();
        assert_eq!(cfg.precision, 6);
        assert!(!cfg.include_velocity);
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_cloth_sim_export();
        assert_eq!(cloth_sim_frame_count(&r), 0);
        assert_eq!(cloth_sim_vertex_count(&r), 0);
    }

    #[test]
    fn add_frame_increments_count() {
        let mut r = new_cloth_sim_export();
        cloth_sim_add_frame(&mut r, make_frame(0, 4));
        assert_eq!(cloth_sim_frame_count(&r), 1);
    }

    #[test]
    fn vertex_count_from_first_frame() {
        let mut r = new_cloth_sim_export();
        cloth_sim_add_frame(&mut r, make_frame(0, 6));
        assert_eq!(cloth_sim_vertex_count(&r), 6);
    }

    #[test]
    fn json_contains_frames_and_positions() {
        let mut r = new_cloth_sim_export();
        cloth_sim_add_frame(&mut r, make_frame(0, 3));
        let cfg = default_cloth_sim_export_config();
        let json = cloth_sim_export_to_json(&r, &cfg);
        assert!(json.contains("\"frames\""));
        assert!(json.contains("\"positions\""));
    }

    #[test]
    fn binary_size_matches_vertex_data() {
        let mut r = new_cloth_sim_export();
        cloth_sim_add_frame(&mut r, make_frame(0, 4));
        let cfg = default_cloth_sim_export_config();
        let bin = cloth_sim_export_to_binary(&r, &cfg);
        // 4 verts * 3 components * 4 bytes = 48
        assert_eq!(bin.len(), 48);
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut r = new_cloth_sim_export();
        cloth_sim_add_frame(&mut r, make_frame(0, 4));
        let cfg = default_cloth_sim_export_config();
        let n = cloth_sim_write_to_file(&mut r, &cfg, "/tmp/cloth.json");
        assert!(n > 0);
        assert_eq!(cloth_sim_total_bytes(&r), n);
    }

    #[test]
    fn clear_resets_everything() {
        let mut r = new_cloth_sim_export();
        cloth_sim_add_frame(&mut r, make_frame(0, 4));
        let cfg = default_cloth_sim_export_config();
        cloth_sim_write_to_file(&mut r, &cfg, "/tmp/cloth.json");
        cloth_sim_clear(&mut r);
        assert_eq!(cloth_sim_frame_count(&r), 0);
        assert_eq!(cloth_sim_total_bytes(&r), 0);
    }

    #[test]
    fn two_frames_in_binary() {
        let mut r = new_cloth_sim_export();
        cloth_sim_add_frame(&mut r, make_frame(0, 2));
        cloth_sim_add_frame(&mut r, make_frame(1, 2));
        let cfg = default_cloth_sim_export_config();
        let bin = cloth_sim_export_to_binary(&r, &cfg);
        // 2 frames * 2 verts * 3 * 4 bytes = 48
        assert_eq!(bin.len(), 48);
    }

    #[test]
    fn velocity_appears_in_json_when_enabled() {
        let mut r = new_cloth_sim_export();
        let mut frame = make_frame(0, 2);
        frame.velocities = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        cloth_sim_add_frame(&mut r, frame);
        let mut cfg = default_cloth_sim_export_config();
        cfg.include_velocity = true;
        let json = cloth_sim_export_to_json(&r, &cfg);
        assert!(json.contains("\"velocities\""));
    }
}
