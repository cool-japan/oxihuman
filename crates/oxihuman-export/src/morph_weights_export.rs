// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export morph target weight arrays in compact binary or JSON format.

#![allow(dead_code)]

/// Configuration for morph-weight export.
#[derive(Debug, Clone)]
pub struct MorphWeightsExportConfig {
    /// Names of morph targets, in index order.
    pub target_names: Vec<String>,
    /// Whether to clamp weights to [0, 1] on export.
    pub clamp_weights: bool,
    /// Float precision for JSON output.
    pub precision: usize,
}

/// A single animation frame of morph weights.
#[derive(Debug, Clone)]
pub struct MorphWeightFrame {
    /// Time in seconds.
    pub time_sec: f32,
    /// One weight per morph target.
    pub weights: Vec<f32>,
}

/// Result accumulating all frames for export.
#[derive(Debug, Clone)]
pub struct MorphWeightsExportResult {
    /// All frames.
    pub frames: Vec<MorphWeightFrame>,
    /// Last export byte count.
    pub total_bytes: usize,
}

/// Returns the default [`MorphWeightsExportConfig`].
#[allow(dead_code)]
pub fn default_morph_weights_export_config() -> MorphWeightsExportConfig {
    MorphWeightsExportConfig {
        target_names: Vec::new(),
        clamp_weights: true,
        precision: 4,
    }
}

/// Creates a new, empty [`MorphWeightsExportResult`].
#[allow(dead_code)]
pub fn new_morph_weights_export() -> MorphWeightsExportResult {
    MorphWeightsExportResult {
        frames: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds a frame to the export result.
#[allow(dead_code)]
pub fn mwe_add_frame(result: &mut MorphWeightsExportResult, frame: MorphWeightFrame) {
    result.frames.push(frame);
}

/// Serialises all frames as JSON.
#[allow(dead_code)]
pub fn mwe_export_json(
    result: &MorphWeightsExportResult,
    cfg: &MorphWeightsExportConfig,
) -> String {
    let prec = cfg.precision;
    let mut out = String::from("{\"frames\":[\n");
    for (fi, f) in result.frames.iter().enumerate() {
        let comma = if fi + 1 < result.frames.len() { "," } else { "" };
        let weights: Vec<String> = f
            .weights
            .iter()
            .map(|&w| {
                let w = if cfg.clamp_weights { w.clamp(0.0, 1.0) } else { w };
                format!("{:.prec$}", w)
            })
            .collect();
        out.push_str(&format!(
            "  {{\"time\":{:.prec$},\"weights\":[{}]}}{}",
            f.time_sec,
            weights.join(","),
            comma
        ));
        out.push('\n');
    }
    out.push_str("]}");
    out
}

/// Encodes all frames as raw binary (little-endian f32 per weight, preceded by time_sec).
#[allow(dead_code)]
pub fn mwe_export_binary(
    result: &MorphWeightsExportResult,
    cfg: &MorphWeightsExportConfig,
) -> Vec<u8> {
    let mut buf = Vec::new();
    // header: frame_count u32
    buf.extend_from_slice(&(result.frames.len() as u32).to_le_bytes());
    for f in &result.frames {
        buf.extend_from_slice(&f.time_sec.to_le_bytes());
        // weight_count u32
        buf.extend_from_slice(&(f.weights.len() as u32).to_le_bytes());
        for &w in &f.weights {
            let w = if cfg.clamp_weights { w.clamp(0.0, 1.0) } else { w };
            buf.extend_from_slice(&w.to_le_bytes());
        }
    }
    buf
}

/// Returns the number of frames.
#[allow(dead_code)]
pub fn mwe_frame_count(result: &MorphWeightsExportResult) -> usize {
    result.frames.len()
}

/// Returns the number of weights in the first frame, or 0 if empty.
#[allow(dead_code)]
pub fn mwe_weight_count(result: &MorphWeightsExportResult) -> usize {
    result.frames.first().map(|f| f.weights.len()).unwrap_or(0)
}

/// Writes the JSON export to a file (stub – returns byte count).
#[allow(dead_code)]
pub fn mwe_write_to_file(
    result: &mut MorphWeightsExportResult,
    cfg: &MorphWeightsExportConfig,
    _path: &str,
) -> usize {
    let json = mwe_export_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Returns the last export byte count.
#[allow(dead_code)]
pub fn mwe_total_bytes(result: &MorphWeightsExportResult) -> usize {
    result.total_bytes
}

/// Clears all frames.
#[allow(dead_code)]
pub fn mwe_clear(result: &mut MorphWeightsExportResult) {
    result.frames.clear();
    result.total_bytes = 0;
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn sample_frame(t: f32) -> MorphWeightFrame {
    MorphWeightFrame {
        time_sec: t,
        weights: vec![0.0, 0.5, 1.0],
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_clamps() {
        let cfg = default_morph_weights_export_config();
        assert!(cfg.clamp_weights);
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_morph_weights_export();
        assert_eq!(mwe_frame_count(&r), 0);
    }

    #[test]
    fn add_frame_increments_count() {
        let mut r = new_morph_weights_export();
        mwe_add_frame(&mut r, sample_frame(0.0));
        assert_eq!(mwe_frame_count(&r), 1);
    }

    #[test]
    fn weight_count_from_first_frame() {
        let mut r = new_morph_weights_export();
        mwe_add_frame(&mut r, sample_frame(0.0));
        assert_eq!(mwe_weight_count(&r), 3);
    }

    #[test]
    fn json_export_contains_time_and_weights() {
        let mut r = new_morph_weights_export();
        mwe_add_frame(&mut r, sample_frame(0.5));
        let cfg = default_morph_weights_export_config();
        let json = mwe_export_json(&r, &cfg);
        assert!(json.contains("\"time\""));
        assert!(json.contains("\"weights\""));
    }

    #[test]
    fn binary_export_has_correct_frame_count_header() {
        let mut r = new_morph_weights_export();
        mwe_add_frame(&mut r, sample_frame(0.0));
        mwe_add_frame(&mut r, sample_frame(1.0));
        let cfg = default_morph_weights_export_config();
        let bin = mwe_export_binary(&r, &cfg);
        let count = u32::from_le_bytes(bin[0..4].try_into().unwrap());
        assert_eq!(count, 2);
    }

    #[test]
    fn write_to_file_returns_nonzero() {
        let mut r = new_morph_weights_export();
        mwe_add_frame(&mut r, sample_frame(0.0));
        let cfg = default_morph_weights_export_config();
        let bytes = mwe_write_to_file(&mut r, &cfg, "/tmp/mwe.json");
        assert!(bytes > 0);
        assert_eq!(mwe_total_bytes(&r), bytes);
    }

    #[test]
    fn clear_resets_state() {
        let mut r = new_morph_weights_export();
        mwe_add_frame(&mut r, sample_frame(0.0));
        mwe_clear(&mut r);
        assert_eq!(mwe_frame_count(&r), 0);
        assert_eq!(mwe_total_bytes(&r), 0);
    }

    #[test]
    fn clamp_weights_limits_to_unit_range() {
        let mut r = new_morph_weights_export();
        mwe_add_frame(&mut r, MorphWeightFrame {
            time_sec: 0.0,
            weights: vec![-0.5, 1.5],
        });
        let mut cfg = default_morph_weights_export_config();
        cfg.clamp_weights = true;
        let bin = mwe_export_binary(&r, &cfg);
        // skip header (4) + time (4) + count (4) = offset 12
        let w0 = f32::from_le_bytes(bin[12..16].try_into().unwrap());
        let w1 = f32::from_le_bytes(bin[16..20].try_into().unwrap());
        assert!((w0 - 0.0).abs() < 1e-6, "clamped below 0");
        assert!((w1 - 1.0).abs() < 1e-6, "clamped above 1");
    }
}
