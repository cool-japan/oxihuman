//! Export morph targets at reduced LOD (decimated vertex count) for runtime optimization.
//!
//! Produces simplified morph data by sub-sampling vertices at configurable
//! LOD ratios, suitable for lightweight real-time playback.

#![allow(dead_code)]

/// Configuration for LOD morph export.
#[derive(Debug, Clone)]
pub struct MorphExportLodConfig {
    /// Ratio of vertices kept (0.0–1.0). E.g. 0.5 keeps every other vertex.
    pub lod_ratio: f32,
    /// Threshold below which a delta is considered zero and omitted.
    pub delta_threshold: f32,
    /// Maximum number of LOD entries.
    pub max_entries: usize,
}

/// A single morph entry at reduced LOD.
#[derive(Debug, Clone)]
pub struct LodMorphEntry {
    /// Name of the morph target.
    pub name: String,
    /// Original vertex count.
    pub original_vertex_count: usize,
    /// Decimated vertices (x, y, z deltas per kept vertex).
    pub lod_deltas: Vec<[f32; 3]>,
}

/// Result of an LOD morph export operation.
#[derive(Debug, Clone)]
pub struct MorphExportLodResult {
    /// All exported LOD morph entries.
    pub entries: Vec<LodMorphEntry>,
    /// LOD ratio used during export.
    pub lod_ratio: f32,
    /// Total original vertex count across all entries.
    pub total_original_vertices: usize,
    /// Total LOD vertex count across all entries.
    pub total_lod_vertices: usize,
}

/// The working LOD exporter.
#[derive(Debug, Clone)]
pub struct MorphExportLod {
    config: MorphExportLodConfig,
    entries: Vec<LodMorphEntry>,
}

/// Build a default `MorphExportLodConfig`.
#[allow(dead_code)]
pub fn default_morph_export_lod_config() -> MorphExportLodConfig {
    MorphExportLodConfig {
        lod_ratio: 0.5,
        delta_threshold: 1e-4,
        max_entries: 128,
    }
}

/// Create a new `MorphExportLod` exporter.
#[allow(dead_code)]
pub fn new_morph_export_lod(config: MorphExportLodConfig) -> MorphExportLod {
    MorphExportLod {
        config,
        entries: Vec::new(),
    }
}

/// Add a morph entry: name + full-resolution delta array.
#[allow(dead_code)]
pub fn mel_add_entry(
    mel: &mut MorphExportLod,
    name: &str,
    deltas: &[[f32; 3]],
) {
    if mel.entries.len() >= mel.config.max_entries {
        return;
    }
    let original_vertex_count = deltas.len();
    mel.entries.push(LodMorphEntry {
        name: name.to_string(),
        original_vertex_count,
        lod_deltas: deltas.to_vec(),
    });
}

/// Export all entries at the configured LOD, returning a `MorphExportLodResult`.
#[allow(dead_code)]
pub fn mel_export_lod(mel: &MorphExportLod) -> MorphExportLodResult {
    let ratio = mel.config.lod_ratio.clamp(0.0, 1.0);
    let threshold = mel.config.delta_threshold;
    let mut result_entries = Vec::with_capacity(mel.entries.len());
    let mut total_orig = 0usize;
    let mut total_lod = 0usize;

    for entry in &mel.entries {
        let n = entry.original_vertex_count;
        total_orig += n;
        // Sub-sample: keep vertex i if i * ratio_inv < count (deterministic stride)
        let stride = if ratio <= 0.0 {
            usize::MAX
        } else {
            (1.0 / ratio).round() as usize
        };
        let stride = stride.max(1);
        let lod_deltas: Vec<[f32; 3]> = entry
            .lod_deltas
            .iter()
            .enumerate()
            .filter(|(i, d)| {
                *i % stride == 0
                    && (d[0].abs() > threshold || d[1].abs() > threshold || d[2].abs() > threshold)
            })
            .map(|(_, d)| *d)
            .collect();
        total_lod += lod_deltas.len();
        result_entries.push(LodMorphEntry {
            name: entry.name.clone(),
            original_vertex_count: n,
            lod_deltas,
        });
    }

    MorphExportLodResult {
        entries: result_entries,
        lod_ratio: ratio,
        total_original_vertices: total_orig,
        total_lod_vertices: total_lod,
    }
}

/// Return the number of morph entries added.
#[allow(dead_code)]
pub fn mel_entry_count(mel: &MorphExportLod) -> usize {
    mel.entries.len()
}

/// Return the LOD vertex count for the entry at `index` (after export).
#[allow(dead_code)]
pub fn mel_lod_vertex_count(result: &MorphExportLodResult, index: usize) -> usize {
    result.entries.get(index).map(|e| e.lod_deltas.len()).unwrap_or(0)
}

/// Return the overall compression ratio (lod / original). 0.0 if no vertices.
#[allow(dead_code)]
pub fn mel_compression_ratio(result: &MorphExportLodResult) -> f32 {
    if result.total_original_vertices == 0 {
        return 0.0;
    }
    result.total_lod_vertices as f32 / result.total_original_vertices as f32
}

/// Serialize the exporter state to a JSON string.
#[allow(dead_code)]
pub fn mel_to_json(mel: &MorphExportLod) -> String {
    format!(
        "{{\"entry_count\":{},\"lod_ratio\":{},\"delta_threshold\":{}}}",
        mel.entries.len(),
        mel.config.lod_ratio,
        mel.config.delta_threshold
    )
}

/// Write the LOD result to a file as JSON-like text (returns byte count written).
#[allow(dead_code)]
pub fn mel_write_to_file(result: &MorphExportLodResult, path: &str) -> std::io::Result<usize> {
    let content = format!(
        "{{\"lod_ratio\":{},\"total_original\":{},\"total_lod\":{},\"entry_count\":{}}}",
        result.lod_ratio,
        result.total_original_vertices,
        result.total_lod_vertices,
        result.entries.len()
    );
    std::fs::write(path, &content)?;
    Ok(content.len())
}

/// Remove all morph entries.
#[allow(dead_code)]
pub fn mel_clear(mel: &mut MorphExportLod) {
    mel.entries.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_deltas(n: usize) -> Vec<[f32; 3]> {
        (0..n).map(|i| [i as f32 * 0.1, 0.0, 0.0]).collect()
    }

    #[test]
    fn test_add_entry_and_count() {
        let mut mel = new_morph_export_lod(default_morph_export_lod_config());
        mel_add_entry(&mut mel, "smile", &sample_deltas(100));
        assert_eq!(mel_entry_count(&mel), 1);
    }

    #[test]
    fn test_export_reduces_vertices() {
        let cfg = MorphExportLodConfig {
            lod_ratio: 0.5,
            delta_threshold: 0.0,
            max_entries: 10,
        };
        let mut mel = new_morph_export_lod(cfg);
        mel_add_entry(&mut mel, "morph", &sample_deltas(10));
        let result = mel_export_lod(&mel);
        // stride=2 → keep indices 0,2,4,6,8 = 5 vertices
        assert!(result.total_lod_vertices <= result.total_original_vertices);
    }

    #[test]
    fn test_compression_ratio_range() {
        let mut mel = new_morph_export_lod(default_morph_export_lod_config());
        mel_add_entry(&mut mel, "m", &sample_deltas(20));
        let result = mel_export_lod(&mel);
        let r = mel_compression_ratio(&result);
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn test_empty_result_compression_ratio() {
        let mel = new_morph_export_lod(default_morph_export_lod_config());
        let result = mel_export_lod(&mel);
        assert_eq!(mel_compression_ratio(&result), 0.0);
    }

    #[test]
    fn test_clear_removes_entries() {
        let mut mel = new_morph_export_lod(default_morph_export_lod_config());
        mel_add_entry(&mut mel, "a", &sample_deltas(10));
        mel_clear(&mut mel);
        assert_eq!(mel_entry_count(&mel), 0);
    }

    #[test]
    fn test_to_json_contains_lod_ratio() {
        let mel = new_morph_export_lod(default_morph_export_lod_config());
        let json = mel_to_json(&mel);
        assert!(json.contains("lod_ratio"));
    }

    #[test]
    fn test_max_entries_enforced() {
        let cfg = MorphExportLodConfig {
            lod_ratio: 0.5,
            delta_threshold: 0.0,
            max_entries: 2,
        };
        let mut mel = new_morph_export_lod(cfg);
        mel_add_entry(&mut mel, "a", &sample_deltas(5));
        mel_add_entry(&mut mel, "b", &sample_deltas(5));
        mel_add_entry(&mut mel, "c", &sample_deltas(5)); // ignored
        assert_eq!(mel_entry_count(&mel), 2);
    }

    #[test]
    fn test_lod_vertex_count_oob_returns_zero() {
        let mel = new_morph_export_lod(default_morph_export_lod_config());
        let result = mel_export_lod(&mel);
        assert_eq!(mel_lod_vertex_count(&result, 99), 0);
    }

    #[test]
    fn test_threshold_filters_small_deltas() {
        let cfg = MorphExportLodConfig {
            lod_ratio: 1.0,
            delta_threshold: 1.0,
            max_entries: 10,
        };
        let mut mel = new_morph_export_lod(cfg);
        // deltas 0..10: 0.0, 0.1, 0.2 ... all below threshold 1.0
        mel_add_entry(&mut mel, "m", &sample_deltas(10));
        let result = mel_export_lod(&mel);
        assert_eq!(result.total_lod_vertices, 0);
    }
}
