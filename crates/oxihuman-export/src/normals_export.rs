// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export vertex/face normal arrays in various formats (JSON, binary float32).

#![allow(dead_code)]

/// Configuration for normals export.
#[derive(Debug, Clone)]
pub struct NormalsExportConfig {
    /// Float precision for JSON output.
    pub precision: usize,
    /// Whether to export face normals instead of vertex normals.
    pub face_normals: bool,
    /// Whether to normalise normals before export.
    pub normalise: bool,
}

/// A single frame of normal data.
#[derive(Debug, Clone)]
pub struct NormalFrame {
    /// Frame index.
    pub index: u32,
    /// Flat list of normals: [nx0, ny0, nz0, nx1, ny1, nz1, ...].
    pub normals: Vec<f32>,
}

/// Accumulated normals export state.
#[derive(Debug, Clone)]
pub struct NormalsExportResult {
    /// All normal frames.
    pub frames: Vec<NormalFrame>,
    /// Byte count of the last export operation.
    pub total_bytes: usize,
}

/// Returns the default [`NormalsExportConfig`].
pub fn default_normals_export_config() -> NormalsExportConfig {
    NormalsExportConfig {
        precision: 6,
        face_normals: false,
        normalise: true,
    }
}

/// Creates a new, empty [`NormalsExportResult`].
pub fn new_normals_export() -> NormalsExportResult {
    NormalsExportResult {
        frames: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds a frame of normals.
pub fn normals_add_frame(result: &mut NormalsExportResult, frame: NormalFrame) {
    result.frames.push(frame);
}

/// Returns the number of frames.
pub fn normals_frame_count(result: &NormalsExportResult) -> usize {
    result.frames.len()
}

/// Returns the number of normals per frame (from the first frame, or 0).
pub fn normals_per_frame(result: &NormalsExportResult) -> usize {
    result
        .frames
        .first()
        .map(|f| f.normals.len() / 3)
        .unwrap_or(0)
}

/// Serialises all frames as a JSON string.
pub fn normals_export_to_json(
    result: &NormalsExportResult,
    cfg: &NormalsExportConfig,
) -> String {
    let prec = cfg.precision;
    let mut out = String::from("{\"frames\":[\n");
    for (fi, frame) in result.frames.iter().enumerate() {
        let comma = if fi + 1 < result.frames.len() { "," } else { "" };
        out.push_str(&format!("  {{\"index\":{},\"normals\":[", frame.index));
        let chunks: Vec<String> = frame
            .normals
            .chunks(3)
            .map(|n| {
                let (nx, ny, nz) = normalise_if(n[0], n[1], n[2], cfg.normalise);
                format!("[{:.prec$},{:.prec$},{:.prec$}]", nx, ny, nz)
            })
            .collect();
        out.push_str(&chunks.join(","));
        out.push_str(&format!("]}}{}\n", comma));
    }
    out.push_str("]}");
    out
}

/// Serialises all frames as raw little-endian float32 bytes.
pub fn normals_export_to_binary(
    result: &NormalsExportResult,
    cfg: &NormalsExportConfig,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    for frame in &result.frames {
        for chunk in frame.normals.chunks(3) {
            let (nx, ny, nz) = normalise_if(chunk[0], chunk[1], chunk[2], cfg.normalise);
            buf.extend_from_slice(&nx.to_le_bytes());
            buf.extend_from_slice(&ny.to_le_bytes());
            buf.extend_from_slice(&nz.to_le_bytes());
        }
    }
    buf
}

/// Returns total byte size of the last export.
pub fn normals_total_bytes(result: &NormalsExportResult) -> usize {
    result.total_bytes
}

/// Writes JSON to a file path (stub – returns byte count).
pub fn normals_write_to_file(
    result: &mut NormalsExportResult,
    cfg: &NormalsExportConfig,
    _path: &str,
) -> usize {
    let json = normals_export_to_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Clears all frames and resets byte count.
pub fn normals_export_clear(result: &mut NormalsExportResult) {
    result.frames.clear();
    result.total_bytes = 0;
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn normalise_if(x: f32, y: f32, z: f32, do_it: bool) -> (f32, f32, f32) {
    if !do_it {
        return (x, y, z);
    }
    let len = (x * x + y * y + z * z).sqrt();
    if len < 1e-10 {
        (0.0, 1.0, 0.0)
    } else {
        (x / len, y / len, z / len)
    }
}

fn make_frame(index: u32) -> NormalFrame {
    NormalFrame {
        index,
        normals: vec![0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_normals_export_config();
        assert_eq!(cfg.precision, 6);
        assert!(!cfg.face_normals);
        assert!(cfg.normalise);
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_normals_export();
        assert_eq!(normals_frame_count(&r), 0);
        assert_eq!(normals_per_frame(&r), 0);
    }

    #[test]
    fn add_frame_increases_count() {
        let mut r = new_normals_export();
        normals_add_frame(&mut r, make_frame(0));
        assert_eq!(normals_frame_count(&r), 1);
    }

    #[test]
    fn normals_per_frame_computed_correctly() {
        let mut r = new_normals_export();
        normals_add_frame(&mut r, make_frame(0));
        // 9 floats / 3 = 3 normals
        assert_eq!(normals_per_frame(&r), 3);
    }

    #[test]
    fn json_contains_frames_key() {
        let mut r = new_normals_export();
        normals_add_frame(&mut r, make_frame(0));
        let cfg = default_normals_export_config();
        let json = normals_export_to_json(&r, &cfg);
        assert!(json.contains("\"frames\""));
        assert!(json.contains("\"normals\""));
    }

    #[test]
    fn binary_length_matches_frame_count() {
        let mut r = new_normals_export();
        normals_add_frame(&mut r, make_frame(0));
        let cfg = default_normals_export_config();
        let bin = normals_export_to_binary(&r, &cfg);
        // 3 normals * 3 components * 4 bytes = 36
        assert_eq!(bin.len(), 36);
    }

    #[test]
    fn write_to_file_updates_total_bytes() {
        let mut r = new_normals_export();
        normals_add_frame(&mut r, make_frame(0));
        let cfg = default_normals_export_config();
        let n = normals_write_to_file(&mut r, &cfg, "/tmp/normals.json");
        assert!(n > 0);
        assert_eq!(normals_total_bytes(&r), n);
    }

    #[test]
    fn clear_resets_everything() {
        let mut r = new_normals_export();
        normals_add_frame(&mut r, make_frame(0));
        let cfg = default_normals_export_config();
        normals_write_to_file(&mut r, &cfg, "/tmp/normals.json");
        normals_export_clear(&mut r);
        assert_eq!(normals_frame_count(&r), 0);
        assert_eq!(normals_total_bytes(&r), 0);
    }

    #[test]
    fn normalise_if_unit_vector_unchanged() {
        let (x, y, z) = normalise_if(0.0, 1.0, 0.0, true);
        assert!((x - 0.0).abs() < 1e-6);
        assert!((y - 1.0).abs() < 1e-6);
        assert!((z - 0.0).abs() < 1e-6);
    }

    #[test]
    fn normalise_if_scales_vector() {
        let (x, y, z) = normalise_if(2.0, 0.0, 0.0, true);
        assert!((x - 1.0).abs() < 1e-6);
        assert!((y - 0.0).abs() < 1e-6);
        assert!((z - 0.0).abs() < 1e-6);
    }

    #[test]
    fn multiple_frames_binary_size() {
        let mut r = new_normals_export();
        normals_add_frame(&mut r, make_frame(0));
        normals_add_frame(&mut r, make_frame(1));
        let cfg = default_normals_export_config();
        let bin = normals_export_to_binary(&r, &cfg);
        // 2 frames * 3 normals * 3 components * 4 bytes = 72
        assert_eq!(bin.len(), 72);
    }
}
