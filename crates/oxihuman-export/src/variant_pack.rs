// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export multiple character mesh variants as a named pack with a JSON manifest.

#![allow(dead_code)]

use anyhow::{Context, Result};
use oxihuman_mesh::MeshBuffers;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Types ────────────────────────────────────────────────────────────────────

/// One character variant in the pack.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantEntry {
    pub id: String,
    pub name: String,
    pub glb_filename: String,
    pub params: HashMap<String, f32>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// The pack manifest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantPackManifest {
    pub version: String,
    pub pack_name: String,
    pub variant_count: usize,
    pub variants: Vec<VariantEntry>,
    pub created_at: String,
}

/// Result of writing a variant pack.
pub struct VariantPackResult {
    pub output_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub glb_paths: Vec<PathBuf>,
    pub total_bytes: usize,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Build a manifest from a list of variants.
/// `version` is fixed to "1.0"; `created_at` uses the static ISO 8601 string
/// "2026-01-01T00:00:00Z" (no external time dependency).
pub fn build_manifest(pack_name: &str, variants: Vec<VariantEntry>) -> VariantPackManifest {
    let variant_count = variants.len();
    VariantPackManifest {
        version: "1.0".to_string(),
        pack_name: pack_name.to_string(),
        variant_count,
        variants,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

/// Convenience constructor for `VariantEntry` with empty tags and metadata.
pub fn variant_entry(
    id: &str,
    name: &str,
    glb_filename: &str,
    params: HashMap<String, f32>,
) -> VariantEntry {
    VariantEntry {
        id: id.to_string(),
        name: name.to_string(),
        glb_filename: glb_filename.to_string(),
        params,
        tags: Vec::new(),
        metadata: HashMap::new(),
    }
}

/// Export each mesh variant to a GLB file and write a JSON manifest.
///
/// For each `(entry, mesh)` pair the GLB is written to
/// `output_dir/<entry.glb_filename>`.  After all GLB files are written the
/// manifest is serialised as `manifest.json` in `output_dir`.
#[allow(clippy::too_many_arguments)]
pub fn write_variant_pack(
    meshes: &[(VariantEntry, &MeshBuffers)],
    output_dir: &Path,
    pack_name: &str,
) -> Result<VariantPackResult> {
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("creating output dir {}", output_dir.display()))?;

    let mut glb_paths = Vec::new();
    let mut entries = Vec::new();
    let mut total_bytes: usize = 0;

    for (entry, mesh) in meshes {
        let glb_path = output_dir.join(&entry.glb_filename);
        crate::glb::export_glb(mesh, &glb_path)
            .with_context(|| format!("exporting GLB for variant '{}'", entry.id))?;

        let file_size = std::fs::metadata(&glb_path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        total_bytes += file_size;

        glb_paths.push(glb_path);
        entries.push(entry.clone());
    }

    let manifest = build_manifest(pack_name, entries);
    let manifest_path = output_dir.join("manifest.json");
    let manifest_json =
        serde_json::to_string_pretty(&manifest).context("serialising manifest to JSON")?;
    std::fs::write(&manifest_path, manifest_json)
        .with_context(|| format!("writing manifest to {}", manifest_path.display()))?;

    let manifest_size = std::fs::metadata(&manifest_path)
        .map(|m| m.len() as usize)
        .unwrap_or(0);
    total_bytes += manifest_size;

    Ok(VariantPackResult {
        output_dir: output_dir.to_path_buf(),
        manifest_path,
        glb_paths,
        total_bytes,
    })
}

/// Load and parse a JSON manifest from disk.
pub fn load_manifest(path: &Path) -> Result<VariantPackManifest> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading manifest at {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("parsing manifest at {}", path.display()))
}

/// Validate a pack directory against a manifest.
///
/// Returns a (possibly empty) list of error strings.  Checks:
/// - `variant_count` matches `variants.len()`
/// - each GLB file listed in the manifest actually exists in `dir`
pub fn validate_pack(dir: &Path, manifest: &VariantPackManifest) -> Vec<String> {
    let mut errors = Vec::new();

    if manifest.variant_count != manifest.variants.len() {
        errors.push(format!(
            "variant_count ({}) does not match variants array length ({})",
            manifest.variant_count,
            manifest.variants.len()
        ));
    }

    for variant in &manifest.variants {
        let glb_path = dir.join(&variant.glb_filename);
        if !glb_path.exists() {
            errors.push(format!(
                "GLB file missing for variant '{}': {}",
                variant.id,
                glb_path.display()
            ));
        }
    }

    errors
}

/// Return all variants whose `tags` list contains `tag`.
pub fn filter_variants_by_tag<'a>(
    manifest: &'a VariantPackManifest,
    tag: &str,
) -> Vec<&'a VariantEntry> {
    manifest
        .variants
        .iter()
        .filter(|v| v.tags.iter().any(|t| t == tag))
        .collect()
}

/// Find a variant by its unique `id`.
pub fn find_variant_by_id<'a>(
    manifest: &'a VariantPackManifest,
    id: &str,
) -> Option<&'a VariantEntry> {
    manifest.variants.iter().find(|v| v.id == id)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::suit::apply_suit_flag;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_suit_mesh() -> MeshBuffers {
        let raw = MB {
            positions: vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0f32, 0.0, 1.0]; 3],
            uvs: vec![[0.0f32, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        };
        let mut mesh = MeshBuffers::from_morph(raw);
        apply_suit_flag(&mut mesh);
        mesh
    }

    fn sample_entry(id: &str, name: &str, glb: &str) -> VariantEntry {
        VariantEntry {
            id: id.to_string(),
            name: name.to_string(),
            glb_filename: glb.to_string(),
            params: HashMap::new(),
            tags: vec!["default".to_string()],
            metadata: HashMap::new(),
        }
    }

    // ── build_manifest ───────────────────────────────────────────────────────

    #[test]
    fn build_manifest_version_is_1_0() {
        let m = build_manifest("TestPack", vec![]);
        assert_eq!(m.version, "1.0");
    }

    #[test]
    fn build_manifest_pack_name_stored() {
        let m = build_manifest("MyPack", vec![]);
        assert_eq!(m.pack_name, "MyPack");
    }

    #[test]
    fn build_manifest_variant_count_matches() {
        let variants = vec![sample_entry("v0", "Var 0", "v0.glb")];
        let m = build_manifest("P", variants);
        assert_eq!(m.variant_count, 1);
        assert_eq!(m.variants.len(), 1);
    }

    #[test]
    fn build_manifest_created_at_static() {
        let m = build_manifest("P", vec![]);
        assert_eq!(m.created_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn build_manifest_empty_variants() {
        let m = build_manifest("Empty", vec![]);
        assert_eq!(m.variant_count, 0);
        assert!(m.variants.is_empty());
    }

    // ── variant_entry constructor ─────────────────────────────────────────────

    #[test]
    fn variant_entry_constructor_fields() {
        let mut params = HashMap::new();
        params.insert("height".to_string(), 1.75f32);
        let e = variant_entry("id1", "Human 1", "h1.glb", params.clone());
        assert_eq!(e.id, "id1");
        assert_eq!(e.name, "Human 1");
        assert_eq!(e.glb_filename, "h1.glb");
        assert_eq!(e.params["height"], 1.75);
    }

    #[test]
    fn variant_entry_constructor_empty_tags_and_metadata() {
        let e = variant_entry("x", "X", "x.glb", HashMap::new());
        assert!(e.tags.is_empty());
        assert!(e.metadata.is_empty());
    }

    // ── filter_variants_by_tag ───────────────────────────────────────────────

    #[test]
    fn filter_by_tag_returns_matching() {
        let mut v1 = sample_entry("v1", "V1", "v1.glb");
        v1.tags = vec!["hero".to_string(), "male".to_string()];
        let mut v2 = sample_entry("v2", "V2", "v2.glb");
        v2.tags = vec!["npc".to_string()];
        let manifest = build_manifest("P", vec![v1, v2]);

        let heroes = filter_variants_by_tag(&manifest, "hero");
        assert_eq!(heroes.len(), 1);
        assert_eq!(heroes[0].id, "v1");
    }

    #[test]
    fn filter_by_tag_no_match_returns_empty() {
        let v = sample_entry("v1", "V1", "v1.glb");
        let manifest = build_manifest("P", vec![v]);
        let result = filter_variants_by_tag(&manifest, "alien");
        assert!(result.is_empty());
    }

    #[test]
    fn filter_by_tag_multiple_matches() {
        let mut v1 = sample_entry("v1", "V1", "v1.glb");
        v1.tags = vec!["shared".to_string()];
        let mut v2 = sample_entry("v2", "V2", "v2.glb");
        v2.tags = vec!["shared".to_string()];
        let manifest = build_manifest("P", vec![v1, v2]);

        let shared = filter_variants_by_tag(&manifest, "shared");
        assert_eq!(shared.len(), 2);
    }

    // ── find_variant_by_id ───────────────────────────────────────────────────

    #[test]
    fn find_variant_by_id_found() {
        let v = sample_entry("abc", "ABC", "abc.glb");
        let manifest = build_manifest("P", vec![v]);
        let found = find_variant_by_id(&manifest, "abc");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").name, "ABC");
    }

    #[test]
    fn find_variant_by_id_not_found() {
        let manifest = build_manifest("P", vec![]);
        assert!(find_variant_by_id(&manifest, "missing").is_none());
    }

    // ── validate_pack ────────────────────────────────────────────────────────

    #[test]
    fn validate_pack_count_mismatch_reported() {
        let v = sample_entry("v0", "V0", "v0.glb");
        let mut manifest = build_manifest("P", vec![v]);
        // Manually corrupt variant_count
        manifest.variant_count = 99;
        let tmp = std::path::PathBuf::from("/tmp");
        let errors = validate_pack(&tmp, &manifest);
        assert!(
            errors.iter().any(|e| e.contains("variant_count")),
            "expected count-mismatch error, got: {errors:?}"
        );
    }

    #[test]
    fn validate_pack_missing_glb_reported() {
        let v = sample_entry("v0", "V0", "nonexistent_variant_xyz.glb");
        let manifest = build_manifest("P", vec![v]);
        let tmp = std::path::PathBuf::from("/tmp");
        let errors = validate_pack(&tmp, &manifest);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("nonexistent_variant_xyz.glb")),
            "expected missing-GLB error, got: {errors:?}"
        );
    }

    // ── write_variant_pack + load_manifest roundtrip ─────────────────────────

    #[test]
    fn write_and_load_manifest_roundtrip() {
        let mesh = make_suit_mesh();
        let entry = variant_entry("rt0", "Roundtrip 0", "rt0.glb", HashMap::new());
        let out_dir = std::path::PathBuf::from("/tmp/oxihuman_variant_pack_roundtrip");

        let result = write_variant_pack(&[(entry, &mesh)], &out_dir, "RoundtripPack")
            .expect("write_variant_pack should succeed");

        assert!(result.manifest_path.exists(), "manifest.json must exist");
        assert_eq!(result.glb_paths.len(), 1);
        assert!(result.glb_paths[0].exists(), "GLB file must exist");

        let loaded = load_manifest(&result.manifest_path).expect("load_manifest should succeed");
        assert_eq!(loaded.pack_name, "RoundtripPack");
        assert_eq!(loaded.variant_count, 1);
        assert_eq!(loaded.variants[0].id, "rt0");
    }

    #[test]
    fn write_variant_pack_total_bytes_nonzero() {
        let mesh = make_suit_mesh();
        let entry = variant_entry("b0", "Bytes 0", "b0.glb", HashMap::new());
        let out_dir = std::path::PathBuf::from("/tmp/oxihuman_variant_pack_bytes");

        let result = write_variant_pack(&[(entry, &mesh)], &out_dir, "BytesPack")
            .expect("write should succeed");

        assert!(result.total_bytes > 0, "total_bytes should be non-zero");
    }

    #[test]
    fn validate_pack_valid_returns_no_errors() {
        let mesh = make_suit_mesh();
        let entry = variant_entry("vv0", "Valid 0", "vv0.glb", HashMap::new());
        let out_dir = std::path::PathBuf::from("/tmp/oxihuman_variant_pack_valid");

        let result = write_variant_pack(&[(entry, &mesh)], &out_dir, "ValidPack")
            .expect("write should succeed");

        let loaded = load_manifest(&result.manifest_path).expect("load should succeed");
        let errors = validate_pack(&out_dir, &loaded);
        assert!(
            errors.is_empty(),
            "valid pack should have no errors, got: {errors:?}"
        );
    }
}
