// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export a scene manifest listing all assets, their types, and file paths.

#![allow(dead_code)]

/// Configuration for scene manifest export.
#[derive(Debug, Clone)]
pub struct SceneManifestConfig {
    /// Whether to validate that asset paths are non-empty.
    pub validate_paths: bool,
    /// Pretty-print JSON.
    pub pretty: bool,
}

/// A single asset entry in the manifest.
#[derive(Debug, Clone)]
pub struct ManifestEntry {
    /// Unique asset identifier.
    pub id: String,
    /// Asset type tag (e.g. "mesh", "texture", "audio").
    pub asset_type: String,
    /// File path (relative or absolute).
    pub path: String,
    /// File size in bytes (0 if unknown).
    pub size_bytes: u64,
}

/// The full scene manifest.
#[derive(Debug, Clone)]
pub struct SceneManifest {
    /// Scene name.
    pub scene_name: String,
    /// All asset entries.
    pub entries: Vec<ManifestEntry>,
    /// Total byte size reported by the last export.
    pub total_bytes: usize,
}

/// Returns the default [`SceneManifestConfig`].
pub fn default_scene_manifest_config() -> SceneManifestConfig {
    SceneManifestConfig {
        validate_paths: true,
        pretty: true,
    }
}

/// Creates a new, empty [`SceneManifest`].
pub fn new_scene_manifest(scene_name: &str) -> SceneManifest {
    SceneManifest {
        scene_name: scene_name.to_string(),
        entries: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds an entry to the manifest.
pub fn manifest_add_entry(manifest: &mut SceneManifest, entry: ManifestEntry) {
    manifest.entries.push(entry);
}

/// Returns the number of entries.
pub fn manifest_entry_count(manifest: &SceneManifest) -> usize {
    manifest.entries.len()
}

/// Returns all entries whose `asset_type` matches `type_tag`.
pub fn manifest_find_by_type<'a>(
    manifest: &'a SceneManifest,
    type_tag: &str,
) -> Vec<&'a ManifestEntry> {
    manifest
        .entries
        .iter()
        .filter(|e| e.asset_type == type_tag)
        .collect()
}

/// Validates the manifest: checks all paths are non-empty when configured.
/// Returns a list of validation error strings (empty = valid).
pub fn manifest_validate(manifest: &SceneManifest, cfg: &SceneManifestConfig) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    if manifest.scene_name.is_empty() {
        errors.push("scene_name is empty".to_string());
    }
    if cfg.validate_paths {
        for entry in &manifest.entries {
            if entry.path.is_empty() {
                errors.push(format!("entry '{}' has empty path", entry.id));
            }
            if entry.id.is_empty() {
                errors.push("entry with empty id".to_string());
            }
        }
    }
    errors
}

/// Returns the total sum of `size_bytes` across all entries.
pub fn manifest_total_size(manifest: &SceneManifest) -> u64 {
    manifest.entries.iter().map(|e| e.size_bytes).sum()
}

/// Serialises the manifest as JSON.
pub fn manifest_to_json(manifest: &SceneManifest, cfg: &SceneManifestConfig) -> String {
    let sep = if cfg.pretty { "\n  " } else { "" };
    let indent = if cfg.pretty { "  " } else { "" };
    let mut out = format!("{{\"scene\":\"{}\",\"entries\":[", manifest.scene_name);
    if cfg.pretty {
        out.push('\n');
    }
    for (i, entry) in manifest.entries.iter().enumerate() {
        let comma = if i + 1 < manifest.entries.len() { "," } else { "" };
        out.push_str(indent);
        out.push_str(&format!(
            "{{\"id\":\"{}\",\"type\":\"{}\",\"path\":\"{}\",\"size\":{}}}{}{}",
            entry.id, entry.asset_type, entry.path, entry.size_bytes, comma, sep
        ));
    }
    if cfg.pretty && !manifest.entries.is_empty() {
        // trim the trailing sep
    }
    out.push_str("]}");
    out
}

/// Writes JSON to a file path (stub – returns byte count).
pub fn manifest_write_to_file(
    manifest: &mut SceneManifest,
    cfg: &SceneManifestConfig,
    _path: &str,
) -> usize {
    let json = manifest_to_json(manifest, cfg);
    manifest.total_bytes = json.len();
    manifest.total_bytes
}

/// Clears all entries and resets state.
pub fn manifest_clear(manifest: &mut SceneManifest) {
    manifest.entries.clear();
    manifest.total_bytes = 0;
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_entry(id: &str, asset_type: &str, path: &str, size: u64) -> ManifestEntry {
    ManifestEntry {
        id: id.to_string(),
        asset_type: asset_type.to_string(),
        path: path.to_string(),
        size_bytes: size,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_scene_manifest_config();
        assert!(cfg.validate_paths);
        assert!(cfg.pretty);
    }

    #[test]
    fn new_manifest_is_empty() {
        let m = new_scene_manifest("TestScene");
        assert_eq!(manifest_entry_count(&m), 0);
        assert_eq!(m.scene_name, "TestScene");
    }

    #[test]
    fn add_entry_increments_count() {
        let mut m = new_scene_manifest("Scene");
        manifest_add_entry(&mut m, make_entry("mesh_01", "mesh", "assets/body.obj", 1024));
        assert_eq!(manifest_entry_count(&m), 1);
    }

    #[test]
    fn find_by_type_filters_correctly() {
        let mut m = new_scene_manifest("Scene");
        manifest_add_entry(&mut m, make_entry("mesh_01", "mesh", "body.obj", 512));
        manifest_add_entry(&mut m, make_entry("tex_01", "texture", "skin.png", 4096));
        manifest_add_entry(&mut m, make_entry("mesh_02", "mesh", "hair.obj", 256));
        let meshes = manifest_find_by_type(&m, "mesh");
        assert_eq!(meshes.len(), 2);
        let textures = manifest_find_by_type(&m, "texture");
        assert_eq!(textures.len(), 1);
    }

    #[test]
    fn total_size_sums_entries() {
        let mut m = new_scene_manifest("Scene");
        manifest_add_entry(&mut m, make_entry("a", "mesh", "a.obj", 100));
        manifest_add_entry(&mut m, make_entry("b", "mesh", "b.obj", 200));
        assert_eq!(manifest_total_size(&m), 300);
    }

    #[test]
    fn validate_catches_empty_path() {
        let mut m = new_scene_manifest("Scene");
        manifest_add_entry(&mut m, make_entry("mesh_01", "mesh", "", 0));
        let cfg = default_scene_manifest_config();
        let errs = manifest_validate(&m, &cfg);
        assert!(!errs.is_empty());
    }

    #[test]
    fn validate_ok_when_all_valid() {
        let mut m = new_scene_manifest("Scene");
        manifest_add_entry(&mut m, make_entry("mesh_01", "mesh", "body.obj", 512));
        let cfg = default_scene_manifest_config();
        let errs = manifest_validate(&m, &cfg);
        assert!(errs.is_empty());
    }

    #[test]
    fn json_contains_scene_and_type() {
        let mut m = new_scene_manifest("TestScene");
        manifest_add_entry(&mut m, make_entry("mesh_01", "mesh", "body.obj", 512));
        let cfg = default_scene_manifest_config();
        let json = manifest_to_json(&m, &cfg);
        assert!(json.contains("\"scene\""));
        assert!(json.contains("\"type\""));
        assert!(json.contains("mesh"));
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut m = new_scene_manifest("Scene");
        manifest_add_entry(&mut m, make_entry("mesh_01", "mesh", "body.obj", 512));
        let cfg = default_scene_manifest_config();
        let n = manifest_write_to_file(&mut m, &cfg, "/tmp/manifest.json");
        assert!(n > 0);
        assert_eq!(m.total_bytes, n);
    }

    #[test]
    fn clear_resets_state() {
        let mut m = new_scene_manifest("Scene");
        manifest_add_entry(&mut m, make_entry("mesh_01", "mesh", "body.obj", 512));
        let cfg = default_scene_manifest_config();
        manifest_write_to_file(&mut m, &cfg, "/tmp/manifest.json");
        manifest_clear(&mut m);
        assert_eq!(manifest_entry_count(&m), 0);
        assert_eq!(m.total_bytes, 0);
    }
}
