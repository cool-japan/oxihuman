// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vertex weight paint data export for rigging and skinning workflows.

// ── WeightPaintFormat ─────────────────────────────────────────────────────────

/// Output format for weight paint data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WeightPaintFormat {
    /// JSON text format.
    Json,
    /// Comma-separated values.
    Csv,
    /// Raw binary.
    Binary,
}

// ── WeightPaintConfig ─────────────────────────────────────────────────────────

/// Configuration controlling how vertex weight paint data is exported.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightPaintConfig {
    /// If true, per-vertex weights are normalised to sum to 1.0 before export.
    pub normalize: bool,
    /// If true, individual weight values are clamped to [0.0, 1.0].
    pub clamp_to_01: bool,
    /// Serialisation format.
    pub format: WeightPaintFormat,
}

// ── VertexWeightEntry ─────────────────────────────────────────────────────────

/// One (vertex, bone, weight) triple.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexWeightEntry {
    /// Zero-based vertex index.
    pub vertex_idx: u32,
    /// Name of the influencing bone.
    pub bone_name: String,
    /// Influence weight in [0.0, 1.0].
    pub weight: f32,
}

// ── WeightPaintData ───────────────────────────────────────────────────────────

/// Collection of vertex weight entries for a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightPaintData {
    /// All weight entries.
    pub entries: Vec<VertexWeightEntry>,
    /// Number of vertices in the mesh.
    pub vertex_count: usize,
    /// Number of distinct bones referenced.
    pub bone_count: usize,
}

// ── WeightPaintExportResult ───────────────────────────────────────────────────

/// Result produced by [`export_weight_paint`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightPaintExportResult {
    /// Serialised weight data (JSON, CSV, or hex-encoded binary depending on format).
    pub data_string: String,
    /// Total number of weight entries serialised.
    pub entry_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a sensible default [`WeightPaintConfig`].
#[allow(dead_code)]
pub fn default_weight_paint_config() -> WeightPaintConfig {
    WeightPaintConfig {
        normalize: true,
        clamp_to_01: true,
        format: WeightPaintFormat::Json,
    }
}

/// Create a new [`WeightPaintData`] with the given vertex count and no entries.
#[allow(dead_code)]
pub fn new_weight_paint_data(vertex_count: usize) -> WeightPaintData {
    WeightPaintData {
        entries: Vec::new(),
        vertex_count,
        bone_count: 0,
    }
}

/// Append a weight entry to `data`, updating `bone_count` if the bone is new.
#[allow(dead_code)]
pub fn add_weight_entry(data: &mut WeightPaintData, entry: VertexWeightEntry) {
    let is_new_bone = !data
        .entries
        .iter()
        .any(|e| e.bone_name == entry.bone_name);
    if is_new_bone {
        data.bone_count += 1;
    }
    data.entries.push(entry);
}

/// Construct a [`VertexWeightEntry`].
#[allow(dead_code)]
pub fn new_weight_entry(vertex: u32, bone: &str, weight: f32) -> VertexWeightEntry {
    VertexWeightEntry {
        vertex_idx: vertex,
        bone_name: bone.to_string(),
        weight,
    }
}

/// Export `data` according to `cfg`, returning a [`WeightPaintExportResult`].
#[allow(dead_code)]
pub fn export_weight_paint(
    data: &WeightPaintData,
    cfg: &WeightPaintConfig,
) -> WeightPaintExportResult {
    let mut working = data.clone();
    if cfg.clamp_to_01 {
        for e in &mut working.entries {
            e.weight = e.weight.clamp(0.0, 1.0);
        }
    }
    if cfg.normalize {
        normalize_vertex_weights(&mut working);
    }

    let data_string = match cfg.format {
        WeightPaintFormat::Json => {
            let entries_json: Vec<String> = working
                .entries
                .iter()
                .map(|e| {
                    format!(
                        r#"{{"vertex_idx":{},"bone_name":"{}","weight":{:.6}}}"#,
                        e.vertex_idx, e.bone_name, e.weight
                    )
                })
                .collect();
            format!(
                r#"{{"vertex_count":{},"bone_count":{},"entries":[{}]}}"#,
                working.vertex_count,
                working.bone_count,
                entries_json.join(",")
            )
        }
        WeightPaintFormat::Csv => {
            let mut lines = vec!["vertex_idx,bone_name,weight".to_string()];
            for e in &working.entries {
                lines.push(format!("{},{},{:.6}", e.vertex_idx, e.bone_name, e.weight));
            }
            lines.join("\n")
        }
        WeightPaintFormat::Binary => {
            // Encode as hex for the string output.
            let mut bytes: Vec<u8> = Vec::with_capacity(working.entries.len() * 8);
            for e in &working.entries {
                bytes.extend_from_slice(&e.vertex_idx.to_le_bytes());
                bytes.extend_from_slice(&e.weight.to_le_bytes());
            }
            bytes.iter().map(|b| format!("{:02x}", b)).collect()
        }
    };

    WeightPaintExportResult {
        entry_count: working.entries.len(),
        data_string,
    }
}

/// Normalise per-vertex weights in-place so they sum to 1.0 for each vertex.
#[allow(dead_code)]
pub fn normalize_vertex_weights(data: &mut WeightPaintData) {
    // Gather unique vertex indices.
    let mut vtx_indices: Vec<u32> = data.entries.iter().map(|e| e.vertex_idx).collect();
    vtx_indices.sort_unstable();
    vtx_indices.dedup();

    for vtx in vtx_indices {
        let sum: f32 = data
            .entries
            .iter()
            .filter(|e| e.vertex_idx == vtx)
            .map(|e| e.weight)
            .sum();
        if sum > 0.0 {
            for e in data.entries.iter_mut().filter(|e| e.vertex_idx == vtx) {
                e.weight /= sum;
            }
        }
    }
}

/// Return all weight entries that influence `vertex`.
#[allow(dead_code)]
pub fn weights_for_vertex(
    data: &WeightPaintData,
    vertex: u32,
) -> Vec<&VertexWeightEntry> {
    data.entries
        .iter()
        .filter(|e| e.vertex_idx == vertex)
        .collect()
}

/// Return a human-readable name for the format in `cfg`.
#[allow(dead_code)]
pub fn weight_format_name(cfg: &WeightPaintConfig) -> &'static str {
    match cfg.format {
        WeightPaintFormat::Json => "JSON",
        WeightPaintFormat::Csv => "CSV",
        WeightPaintFormat::Binary => "Binary",
    }
}

/// Serialise a [`WeightPaintExportResult`] as a JSON string.
#[allow(dead_code)]
pub fn weight_paint_result_to_json(r: &WeightPaintExportResult) -> String {
    format!(
        r#"{{"entry_count":{},"data_string_len":{}}}"#,
        r.entry_count,
        r.data_string.len()
    )
}

/// Return `true` if every entry has a weight in [0.0, 1.0] and vertex indices
/// are within bounds.
#[allow(dead_code)]
pub fn validate_weights(data: &WeightPaintData) -> bool {
    data.entries.iter().all(|e| {
        e.weight >= 0.0
            && e.weight <= 1.0
            && (e.vertex_idx as usize) < data.vertex_count
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_normalize_and_clamp() {
        let cfg = default_weight_paint_config();
        assert!(cfg.normalize);
        assert!(cfg.clamp_to_01);
        assert_eq!(cfg.format, WeightPaintFormat::Json);
    }

    #[test]
    fn add_weight_entry_updates_bone_count() {
        let mut d = new_weight_paint_data(4);
        add_weight_entry(&mut d, new_weight_entry(0, "hip", 0.6));
        add_weight_entry(&mut d, new_weight_entry(0, "spine", 0.4));
        add_weight_entry(&mut d, new_weight_entry(1, "hip", 0.9)); // hip already counted
        assert_eq!(d.bone_count, 2);
        assert_eq!(d.entries.len(), 3);
    }

    #[test]
    fn normalize_vertex_weights_sums_to_one() {
        let mut d = new_weight_paint_data(2);
        add_weight_entry(&mut d, new_weight_entry(0, "A", 3.0));
        add_weight_entry(&mut d, new_weight_entry(0, "B", 1.0));
        normalize_vertex_weights(&mut d);
        let sum: f32 = weights_for_vertex(&d, 0).iter().map(|e| e.weight).sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn export_json_format_contains_vertex_count() {
        let mut d = new_weight_paint_data(10);
        add_weight_entry(&mut d, new_weight_entry(0, "bone0", 1.0));
        let cfg = WeightPaintConfig {
            normalize: false,
            clamp_to_01: false,
            format: WeightPaintFormat::Json,
        };
        let result = export_weight_paint(&d, &cfg);
        assert!(result.data_string.contains("\"vertex_count\":10"));
        assert_eq!(result.entry_count, 1);
    }

    #[test]
    fn export_csv_format_has_header() {
        let d = new_weight_paint_data(5);
        let cfg = WeightPaintConfig {
            normalize: false,
            clamp_to_01: false,
            format: WeightPaintFormat::Csv,
        };
        let result = export_weight_paint(&d, &cfg);
        assert!(result.data_string.starts_with("vertex_idx,bone_name,weight"));
    }

    #[test]
    fn validate_weights_valid() {
        let mut d = new_weight_paint_data(3);
        add_weight_entry(&mut d, new_weight_entry(0, "bone", 0.5));
        add_weight_entry(&mut d, new_weight_entry(1, "bone", 1.0));
        assert!(validate_weights(&d));
    }

    #[test]
    fn validate_weights_out_of_range_vertex() {
        let mut d = new_weight_paint_data(2);
        // vertex_idx 99 is out of range for vertex_count=2
        add_weight_entry(&mut d, new_weight_entry(99, "bone", 0.5));
        assert!(!validate_weights(&d));
    }

    #[test]
    fn weight_format_name_returns_correct_strings() {
        let mut cfg = default_weight_paint_config();
        cfg.format = WeightPaintFormat::Json;
        assert_eq!(weight_format_name(&cfg), "JSON");
        cfg.format = WeightPaintFormat::Csv;
        assert_eq!(weight_format_name(&cfg), "CSV");
        cfg.format = WeightPaintFormat::Binary;
        assert_eq!(weight_format_name(&cfg), "Binary");
    }

    #[test]
    fn weight_paint_result_to_json_contains_entry_count() {
        let r = WeightPaintExportResult {
            data_string: "hello".to_string(),
            entry_count: 42,
        };
        let j = weight_paint_result_to_json(&r);
        assert!(j.contains("\"entry_count\":42"));
    }

    #[test]
    fn export_binary_format_non_empty_when_entries_present() {
        let mut d = new_weight_paint_data(1);
        add_weight_entry(&mut d, new_weight_entry(0, "root", 1.0));
        let cfg = WeightPaintConfig {
            normalize: false,
            clamp_to_01: false,
            format: WeightPaintFormat::Binary,
        };
        let result = export_weight_paint(&d, &cfg);
        assert!(!result.data_string.is_empty());
    }
}
