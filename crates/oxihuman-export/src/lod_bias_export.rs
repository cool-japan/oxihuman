// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// LOD bias entry for a mesh.
#[allow(dead_code)]
#[derive(Clone)]
pub struct LodBiasEntry {
    pub mesh_name: String,
    pub bias: f32,
    pub min_lod: usize,
    pub max_lod: usize,
}

/// LOD bias export bundle.
#[allow(dead_code)]
#[derive(Default)]
pub struct LodBiasExport {
    pub entries: Vec<LodBiasEntry>,
}

/// Create a new LOD bias export.
#[allow(dead_code)]
pub fn new_lod_bias_export() -> LodBiasExport {
    LodBiasExport::default()
}

/// Add a LOD bias entry.
#[allow(dead_code)]
pub fn add_lod_bias(
    export: &mut LodBiasExport,
    mesh: &str,
    bias: f32,
    min_lod: usize,
    max_lod: usize,
) {
    export.entries.push(LodBiasEntry {
        mesh_name: mesh.to_string(),
        bias,
        min_lod,
        max_lod,
    });
}

/// Count entries.
#[allow(dead_code)]
pub fn lod_bias_count(export: &LodBiasExport) -> usize {
    export.entries.len()
}

/// Average bias value.
#[allow(dead_code)]
pub fn avg_lod_bias(export: &LodBiasExport) -> f32 {
    if export.entries.is_empty() {
        return 0.0;
    }
    export.entries.iter().map(|e| e.bias).sum::<f32>() / export.entries.len() as f32
}

/// Find entry by mesh name.
#[allow(dead_code)]
pub fn find_lod_bias<'a>(export: &'a LodBiasExport, name: &str) -> Option<&'a LodBiasEntry> {
    export.entries.iter().find(|e| e.mesh_name == name)
}

/// Validate entries (min_lod <= max_lod).
#[allow(dead_code)]
pub fn validate_lod_bias(export: &LodBiasExport) -> bool {
    export.entries.iter().all(|e| e.min_lod <= e.max_lod)
}

/// Maximum LOD level used.
#[allow(dead_code)]
pub fn max_lod_level(export: &LodBiasExport) -> usize {
    export.entries.iter().map(|e| e.max_lod).max().unwrap_or(0)
}

/// Entries with bias above threshold.
#[allow(dead_code)]
pub fn high_bias_entries(export: &LodBiasExport, threshold: f32) -> Vec<&LodBiasEntry> {
    export
        .entries
        .iter()
        .filter(|e| e.bias > threshold)
        .collect()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn lod_bias_to_json(export: &LodBiasExport) -> String {
    format!(
        r#"{{"lod_bias_entries":{},"avg_bias":{:.4}}}"#,
        export.entries.len(),
        avg_lod_bias(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut e = new_lod_bias_export();
        add_lod_bias(&mut e, "body", 0.5, 0, 4);
        assert_eq!(lod_bias_count(&e), 1);
    }

    #[test]
    fn avg_bias() {
        let mut e = new_lod_bias_export();
        add_lod_bias(&mut e, "a", 0.2, 0, 2);
        add_lod_bias(&mut e, "b", 0.4, 0, 2);
        assert!((avg_lod_bias(&e) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn find_entry() {
        let mut e = new_lod_bias_export();
        add_lod_bias(&mut e, "head", 0.3, 0, 3);
        assert!(find_lod_bias(&e, "head").is_some());
    }

    #[test]
    fn find_missing() {
        let e = new_lod_bias_export();
        assert!(find_lod_bias(&e, "x").is_none());
    }

    #[test]
    fn validate_valid() {
        let mut e = new_lod_bias_export();
        add_lod_bias(&mut e, "x", 0.0, 0, 3);
        assert!(validate_lod_bias(&e));
    }

    #[test]
    fn validate_invalid() {
        let mut e = new_lod_bias_export();
        add_lod_bias(&mut e, "x", 0.0, 3, 1);
        assert!(!validate_lod_bias(&e));
    }

    #[test]
    fn max_lod() {
        let mut e = new_lod_bias_export();
        add_lod_bias(&mut e, "a", 0.0, 0, 2);
        add_lod_bias(&mut e, "b", 0.0, 0, 5);
        assert_eq!(max_lod_level(&e), 5);
    }

    #[test]
    fn high_bias_filter() {
        let mut e = new_lod_bias_export();
        add_lod_bias(&mut e, "a", 0.1, 0, 2);
        add_lod_bias(&mut e, "b", 0.9, 0, 2);
        assert_eq!(high_bias_entries(&e, 0.5).len(), 1);
    }

    #[test]
    fn json_has_entries() {
        let e = new_lod_bias_export();
        let j = lod_bias_to_json(&e);
        assert!(j.contains("\"lod_bias_entries\":0"));
    }

    #[test]
    fn empty_max_lod() {
        let e = new_lod_bias_export();
        assert_eq!(max_lod_level(&e), 0);
    }
}
