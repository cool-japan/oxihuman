// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Manifest JSON generation for OxiHuman exports.
//!
//! A manifest describes all exported assets — their paths, sizes, checksums,
//! and metadata — for use by downstream tools (game engines, web loaders, etc.).

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

// ── ManifestEntry ─────────────────────────────────────────────────────────────

/// A single asset entry in the export manifest.
pub struct ManifestEntry {
    pub name: String,
    /// Relative path of the asset.
    pub path: String,
    /// Format identifier: "glb", "obj", "png", etc.
    pub format: String,
    pub size_bytes: u64,
    /// SHA-256 digest as a lowercase hex string, if computed.
    pub sha256: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl ManifestEntry {
    /// Create a new entry with the given name, relative path, and format.
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        format: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            format: format.into(),
            size_bytes: 0,
            sha256: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the file size in bytes (builder-style).
    pub fn with_size(mut self, size: u64) -> Self {
        self.size_bytes = size;
        self
    }

    /// Attach a SHA-256 hex digest (builder-style).
    pub fn with_sha256(mut self, hash: impl Into<String>) -> Self {
        self.sha256 = Some(hash.into());
        self
    }

    /// Append a tag (builder-style).
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Insert a key-value metadata pair (builder-style).
    pub fn with_meta(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), val.into());
        self
    }

    /// Serialize this entry to a [`serde_json::Value`] object.
    pub fn to_json_object(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({
            "name": self.name,
            "path": self.path,
            "format": self.format,
            "size_bytes": self.size_bytes,
            "tags": self.tags,
            "metadata": self.metadata,
        });

        if let Some(ref hash) = self.sha256 {
            obj["sha256"] = serde_json::Value::String(hash.clone());
        }

        obj
    }
}

// ── ExportManifest ────────────────────────────────────────────────────────────

/// Complete export manifest containing all asset entries and global metadata.
pub struct ExportManifest {
    pub version: String,
    pub timestamp: String,
    pub generator: String,
    pub entries: Vec<ManifestEntry>,
    pub global_metadata: HashMap<String, String>,
}

impl ExportManifest {
    /// Create a new, empty manifest with sensible defaults.
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            timestamp: chrono_timestamp(),
            generator: "oxihuman-export/1.0".to_string(),
            entries: Vec::new(),
            global_metadata: HashMap::new(),
        }
    }

    /// Append an entry to the manifest.
    pub fn add_entry(&mut self, entry: ManifestEntry) {
        self.entries.push(entry);
    }

    /// Return the number of entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Return references to all entries that carry the given tag.
    pub fn entries_with_tag(&self, tag: &str) -> Vec<&ManifestEntry> {
        self.entries
            .iter()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Return references to all entries whose format matches.
    pub fn entries_by_format(&self, format: &str) -> Vec<&ManifestEntry> {
        self.entries.iter().filter(|e| e.format == format).collect()
    }

    /// Sum of `size_bytes` across all entries.
    pub fn total_size_bytes(&self) -> u64 {
        self.entries.iter().map(|e| e.size_bytes).sum()
    }

    /// Serialize the manifest to a [`serde_json::Value`].
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "timestamp": self.timestamp,
            "generator": self.generator,
            "entry_count": self.entry_count(),
            "total_size_bytes": self.total_size_bytes(),
            "global_metadata": self.global_metadata,
            "entries": self.entries.iter().map(|e| e.to_json_object()).collect::<Vec<_>>(),
        })
    }

    /// Serialize to a compact JSON string.
    pub fn to_json_string(&self) -> String {
        self.to_json().to_string()
    }

    /// Serialize to a pretty-printed JSON string.
    pub fn to_json_string_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.to_json()).unwrap_or_else(|_| self.to_json_string())
    }
}

impl Default for ExportManifest {
    fn default() -> Self {
        Self::new()
    }
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Write `manifest` as a JSON file at `path`.
pub fn export_manifest(manifest: &ExportManifest, path: &Path) -> Result<()> {
    let json = manifest.to_json_string_pretty();
    let mut file = fs::File::create(path)
        .with_context(|| format!("failed to create manifest file: {}", path.display()))?;
    file.write_all(json.as_bytes())
        .with_context(|| format!("failed to write manifest file: {}", path.display()))?;
    Ok(())
}

/// Load a manifest from a JSON file at `path`.
///
/// Only the fields present in the JSON are restored; missing optional fields
/// fall back to their defaults.
pub fn load_manifest(path: &Path) -> Result<ExportManifest> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read manifest file: {}", path.display()))?;
    let val: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse manifest JSON: {}", path.display()))?;

    let mut manifest = ExportManifest::new();

    if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
        manifest.version = v.to_string();
    }
    if let Some(v) = val.get("timestamp").and_then(|v| v.as_str()) {
        manifest.timestamp = v.to_string();
    }
    if let Some(v) = val.get("generator").and_then(|v| v.as_str()) {
        manifest.generator = v.to_string();
    }
    if let Some(obj) = val.get("global_metadata").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                manifest.global_metadata.insert(k.clone(), s.to_string());
            }
        }
    }
    if let Some(arr) = val.get("entries").and_then(|v| v.as_array()) {
        for item in arr {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let path_str = item
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let format = item
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let size_bytes = item.get("size_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
            let sha256 = item
                .get("sha256")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let tags: Vec<String> = item
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let mut metadata: HashMap<String, String> = HashMap::new();
            if let Some(obj) = item.get("metadata").and_then(|v| v.as_object()) {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        metadata.insert(k.clone(), s.to_string());
                    }
                }
            }

            let mut entry = ManifestEntry::new(name, path_str, format).with_size(size_bytes);
            if let Some(h) = sha256 {
                entry = entry.with_sha256(h);
            }
            for tag in tags {
                entry = entry.with_tag(tag);
            }
            for (k, v) in metadata {
                entry = entry.with_meta(k, v);
            }
            manifest.add_entry(entry);
        }
    }

    Ok(manifest)
}

/// Scan `dir` and auto-generate a manifest from every file found there.
///
/// Sub-directories are skipped. Each file's size and SHA-256 are computed
/// automatically.
pub fn manifest_from_dir(dir: &Path, generator: &str) -> Result<ExportManifest> {
    let mut manifest = ExportManifest::new();
    manifest.generator = generator.to_string();

    let read = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?;

    for entry in read {
        let entry =
            entry.with_context(|| format!("failed to read dir entry in {}", dir.display()))?;
        let file_path = entry.path();

        if !file_path.is_file() {
            continue;
        }

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let size = file_path.metadata().map(|m| m.len()).unwrap_or(0);
        let format = detect_format(&file_path);
        let sha = file_sha256(&file_path).unwrap_or_default();

        let rel_path = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let entry = ManifestEntry::new(file_name, rel_path, format)
            .with_size(size)
            .with_sha256(sha);

        manifest.add_entry(entry);
    }

    Ok(manifest)
}

/// Compute the SHA-256 digest of the file at `path` and return it as a
/// lowercase hex string.
pub fn file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read file for sha256: {}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    Ok(hex::encode(digest))
}

/// Detect a format identifier from the file extension of `path`.
pub fn detect_format(path: &Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "glb" => "glb",
        "gltf" => "gltf",
        "obj" => "obj",
        "stl" => "stl",
        "ply" => "ply",
        "png" => "png",
        "svg" => "svg",
        "json" => "json",
        "csv" => "csv",
        "oxb" => "oxb",
        "opc" => "opc",
        _ => "unknown",
    }
    .to_string()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Return a timestamp string. We use a fixed format without pulling in `chrono`
/// to keep dependencies minimal.
fn chrono_timestamp() -> String {
    // Use UNIX epoch seconds via std::time.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{}", secs)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_entry() -> ManifestEntry {
        ManifestEntry::new("mesh.glb", "assets/mesh.glb", "glb")
            .with_size(1024)
            .with_sha256("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")
            .with_tag("mesh")
            .with_tag("3d")
            .with_meta("author", "test")
    }

    #[test]
    fn test_manifest_entry_new() {
        let e = ManifestEntry::new("foo.obj", "exports/foo.obj", "obj");
        assert_eq!(e.name, "foo.obj");
        assert_eq!(e.path, "exports/foo.obj");
        assert_eq!(e.format, "obj");
        assert_eq!(e.size_bytes, 0);
        assert!(e.sha256.is_none());
        assert!(e.tags.is_empty());
        assert!(e.metadata.is_empty());
    }

    #[test]
    fn test_manifest_entry_builder() {
        let e = sample_entry();
        assert_eq!(e.size_bytes, 1024);
        assert_eq!(
            e.sha256.as_deref(),
            Some("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")
        );
        assert_eq!(e.tags, vec!["mesh", "3d"]);
        assert_eq!(e.metadata.get("author").map(|s| s.as_str()), Some("test"));
    }

    #[test]
    fn test_manifest_entry_to_json() {
        let e = sample_entry();
        let v = e.to_json_object();
        assert_eq!(v["name"], "mesh.glb");
        assert_eq!(v["path"], "assets/mesh.glb");
        assert_eq!(v["format"], "glb");
        assert_eq!(v["size_bytes"], 1024u64);
        assert_eq!(
            v["sha256"],
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        );
        assert_eq!(v["tags"][0], "mesh");

        // Entry without sha256 should not have the key
        let e2 = ManifestEntry::new("a.png", "a.png", "png");
        let v2 = e2.to_json_object();
        assert!(v2.get("sha256").is_none());
    }

    #[test]
    fn test_export_manifest_new() {
        let m = ExportManifest::new();
        assert_eq!(m.version, "1.0");
        assert_eq!(m.generator, "oxihuman-export/1.0");
        assert_eq!(m.entry_count(), 0);
        assert!(m.timestamp.starts_with("unix:"));
    }

    #[test]
    fn test_add_entry() {
        let mut m = ExportManifest::new();
        m.add_entry(sample_entry());
        m.add_entry(ManifestEntry::new("body.stl", "body.stl", "stl").with_size(512));
        assert_eq!(m.entry_count(), 2);
    }

    #[test]
    fn test_entries_with_tag() {
        let mut m = ExportManifest::new();
        m.add_entry(sample_entry()); // tags: mesh, 3d
        m.add_entry(
            ManifestEntry::new("tex.png", "tex.png", "png")
                .with_tag("texture")
                .with_tag("2d"),
        );
        let mesh_entries = m.entries_with_tag("mesh");
        assert_eq!(mesh_entries.len(), 1);
        assert_eq!(mesh_entries[0].name, "mesh.glb");

        let d3_entries = m.entries_with_tag("3d");
        assert_eq!(d3_entries.len(), 1);

        let none_entries = m.entries_with_tag("nonexistent");
        assert!(none_entries.is_empty());
    }

    #[test]
    fn test_entries_by_format() {
        let mut m = ExportManifest::new();
        m.add_entry(sample_entry()); // glb
        m.add_entry(ManifestEntry::new("body.glb", "body.glb", "glb"));
        m.add_entry(ManifestEntry::new("body.obj", "body.obj", "obj"));

        let glb = m.entries_by_format("glb");
        assert_eq!(glb.len(), 2);

        let obj = m.entries_by_format("obj");
        assert_eq!(obj.len(), 1);

        let stl = m.entries_by_format("stl");
        assert!(stl.is_empty());
    }

    #[test]
    fn test_total_size_bytes() {
        let mut m = ExportManifest::new();
        m.add_entry(ManifestEntry::new("a", "a", "glb").with_size(100));
        m.add_entry(ManifestEntry::new("b", "b", "obj").with_size(200));
        m.add_entry(ManifestEntry::new("c", "c", "png").with_size(50));
        assert_eq!(m.total_size_bytes(), 350);
    }

    #[test]
    fn test_to_json_string() {
        let mut m = ExportManifest::new();
        m.add_entry(sample_entry());
        let s = m.to_json_string();
        assert!(s.contains("\"version\""));
        assert!(s.contains("\"entries\""));
        assert!(s.contains("mesh.glb"));

        let pretty = m.to_json_string_pretty();
        // Pretty-printed output contains newlines
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("mesh.glb"));
    }

    #[test]
    fn test_export_and_load_manifest() {
        let mut m = ExportManifest::new();
        m.version = "2.0".to_string();
        m.generator = "test-gen".to_string();
        m.global_metadata
            .insert("project".to_string(), "oxihuman".to_string());
        m.add_entry(sample_entry());
        m.add_entry(ManifestEntry::new("body.obj", "body.obj", "obj").with_size(999));

        let path = PathBuf::from("/tmp/oxihuman_test_manifest.json");
        export_manifest(&m, &path).expect("export failed");

        let loaded = load_manifest(&path).expect("load failed");
        assert_eq!(loaded.version, "2.0");
        assert_eq!(loaded.generator, "test-gen");
        assert_eq!(loaded.entry_count(), 2);
        assert_eq!(
            loaded.global_metadata.get("project").map(|s| s.as_str()),
            Some("oxihuman")
        );
        let e = &loaded.entries[0];
        assert_eq!(e.name, "mesh.glb");
        assert_eq!(e.format, "glb");
        assert_eq!(e.size_bytes, 1024);
        assert_eq!(e.tags, vec!["mesh", "3d"]);
        assert_eq!(e.metadata.get("author").map(|s| s.as_str()), Some("test"));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_detect_format() {
        let cases: &[(&str, &str)] = &[
            ("model.glb", "glb"),
            ("model.gltf", "gltf"),
            ("model.obj", "obj"),
            ("model.stl", "stl"),
            ("cloud.ply", "ply"),
            ("tex.png", "png"),
            ("icon.svg", "svg"),
            ("data.json", "json"),
            ("table.csv", "csv"),
            ("bundle.oxb", "oxb"),
            ("cache.opc", "opc"),
            ("archive.zip", "unknown"),
            ("no_extension", "unknown"),
        ];
        for (file, expected) in cases {
            let p = PathBuf::from(file);
            assert_eq!(detect_format(&p), *expected, "file: {}", file);
        }
    }

    #[test]
    fn test_file_sha256() {
        let path = PathBuf::from("/tmp/oxihuman_sha256_test.bin");
        fs::write(&path, b"hello world").expect("write failed");
        let hash = file_sha256(&path).expect("sha256 failed");
        // Known SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_manifest_from_dir() {
        let dir = PathBuf::from("/tmp/oxihuman_manifest_dir_test");
        fs::create_dir_all(&dir).expect("mkdir failed");

        // Create some test files
        fs::write(dir.join("model.glb"), b"glb data here").expect("write failed");
        fs::write(dir.join("texture.png"), b"png data here").expect("write failed");
        fs::write(dir.join("data.csv"), b"a,b,c\n1,2,3").expect("write failed");

        let manifest = manifest_from_dir(&dir, "test-scanner").expect("manifest_from_dir failed");
        assert_eq!(manifest.generator, "test-scanner");
        assert_eq!(manifest.entry_count(), 3);

        // All entries should have non-empty sha256
        for e in &manifest.entries {
            assert!(e.sha256.is_some());
            assert!(!e.sha256.as_ref().unwrap().is_empty());
        }

        // Find glb entry
        let glb_entries = manifest.entries_by_format("glb");
        assert_eq!(glb_entries.len(), 1);
        assert_eq!(glb_entries[0].name, "model.glb");
        assert_eq!(glb_entries[0].size_bytes, 13); // "glb data here".len()

        // Cleanup
        fs::remove_dir_all(&dir).ok();
    }
}
