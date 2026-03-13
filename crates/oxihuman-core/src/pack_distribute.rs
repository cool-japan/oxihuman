// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pack distribution pipeline for OxiHuman asset packages (.oxp).
//!
//! Handles packaging, signing, verification, and distribution metadata.
//!
//! ## Package format (.oxp)
//!
//! - Magic: `b"OXP\x01"` (4 bytes)
//! - Manifest length: u32 LE
//! - Manifest JSON: manifest_length bytes
//! - File count: u32 LE
//! - For each file: path_len(u16 LE), path(path_len bytes), data_len(u32 LE), data(data_len bytes)
//! - Trailing: integrity hash (32 bytes SHA-256 of everything before it)

#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::pack_sign::double_hash_sign;

/// Magic bytes identifying an OXP package file.
const OXP_MAGIC: &[u8; 4] = b"OXP\x01";

/// Size of the trailing SHA-256 integrity hash.
const INTEGRITY_HASH_LEN: usize = 32;

// ── Public data structures ──────────────────────────────────────────────────

/// Distribution package metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub license: String,
    pub created_at: u64,
    pub targets: Vec<PackTargetEntry>,
    pub dependencies: Vec<PackDependency>,
    pub integrity: PackIntegrity,
}

/// An individual asset file entry within the package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackTargetEntry {
    pub name: String,
    pub category: String,
    pub file_path: String,
    pub size_bytes: usize,
    pub sha256: String,
}

/// A dependency on another pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackDependency {
    pub name: String,
    pub version_req: String,
}

/// Integrity metadata for the package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackIntegrity {
    pub algorithm: String,
    pub manifest_hash: String,
    pub signature: Option<String>,
}

/// An installed pack record.
#[derive(Debug, Clone)]
pub struct InstalledPack {
    pub manifest: PackManifest,
    pub install_path: String,
    pub installed_at: u64,
}

// ── PackBuilder ─────────────────────────────────────────────────────────────

/// Builder for creating `.oxp` distribution packages.
pub struct PackBuilder {
    manifest: PackManifest,
    files: Vec<(String, Vec<u8>)>,
}

impl PackBuilder {
    /// Create a new builder with required metadata.
    pub fn new(name: &str, version: &str, author: &str) -> Self {
        Self {
            manifest: PackManifest {
                name: name.to_string(),
                version: version.to_string(),
                author: author.to_string(),
                description: String::new(),
                license: String::new(),
                created_at: 0,
                targets: Vec::new(),
                dependencies: Vec::new(),
                integrity: PackIntegrity {
                    algorithm: "sha256".to_string(),
                    manifest_hash: String::new(),
                    signature: None,
                },
            },
            files: Vec::new(),
        }
    }

    /// Set the package description.
    pub fn set_description(&mut self, desc: &str) {
        self.manifest.description = desc.to_string();
    }

    /// Set the package license identifier.
    pub fn set_license(&mut self, license: &str) {
        self.manifest.license = license.to_string();
    }

    /// Set the creation timestamp (unix seconds).
    pub fn set_created_at(&mut self, ts: u64) {
        self.manifest.created_at = ts;
    }

    /// Add a target file to the package.
    pub fn add_target_file(&mut self, name: &str, category: &str, data: &[u8]) -> Result<()> {
        if name.is_empty() {
            bail!("target file name must not be empty");
        }
        let sha_hex = sha256_hex(data);
        let file_path = format!("{}/{}", category, name);

        self.manifest.targets.push(PackTargetEntry {
            name: name.to_string(),
            category: category.to_string(),
            file_path: file_path.clone(),
            size_bytes: data.len(),
            sha256: sha_hex,
        });
        self.files.push((file_path, data.to_vec()));
        Ok(())
    }

    /// Declare a dependency on another pack.
    pub fn add_dependency(&mut self, name: &str, version_req: &str) {
        self.manifest.dependencies.push(PackDependency {
            name: name.to_string(),
            version_req: version_req.to_string(),
        });
    }

    /// Build the `.oxp` package bytes (unsigned).
    pub fn build(&self) -> Result<Vec<u8>> {
        self.build_internal(None)
    }

    /// Build and sign the `.oxp` package with the given key.
    pub fn build_signed(&self, signing_key: &[u8]) -> Result<Vec<u8>> {
        self.build_internal(Some(signing_key))
    }

    /// Internal build routine shared by `build` and `build_signed`.
    fn build_internal(&self, signing_key: Option<&[u8]>) -> Result<Vec<u8>> {
        // Compute manifest hash over target entries for integrity
        let manifest_hash_hex = self.compute_manifest_hash();

        // Optionally compute signature
        let signature_hex = signing_key.map(|key| {
            let sig_bytes = double_hash_sign(key, manifest_hash_hex.as_bytes());
            hex::encode(sig_bytes)
        });

        // Finalize manifest with integrity info
        let mut manifest = self.manifest.clone();
        manifest.integrity = PackIntegrity {
            algorithm: "sha256".to_string(),
            manifest_hash: manifest_hash_hex,
            signature: signature_hex,
        };

        let manifest_json = serde_json::to_vec(&manifest)
            .with_context(|| "failed to serialize manifest to JSON")?;

        // Assemble the binary package
        let mut buf: Vec<u8> = Vec::new();

        // Magic
        buf.extend_from_slice(OXP_MAGIC);

        // Manifest length + manifest JSON
        let manifest_len = u32::try_from(manifest_json.len())
            .with_context(|| "manifest JSON too large for u32 length")?;
        buf.extend_from_slice(&manifest_len.to_le_bytes());
        buf.extend_from_slice(&manifest_json);

        // File count
        let file_count =
            u32::try_from(self.files.len()).with_context(|| "file count too large for u32")?;
        buf.extend_from_slice(&file_count.to_le_bytes());

        // Each file: path_len(u16 LE), path, data_len(u32 LE), data
        for (path, data) in &self.files {
            let path_bytes = path.as_bytes();
            let path_len = u16::try_from(path_bytes.len())
                .with_context(|| format!("file path too long: {}", path))?;
            buf.extend_from_slice(&path_len.to_le_bytes());
            buf.extend_from_slice(path_bytes);

            let data_len = u32::try_from(data.len())
                .with_context(|| format!("file data too large: {}", path))?;
            buf.extend_from_slice(&data_len.to_le_bytes());
            buf.extend_from_slice(data);
        }

        // Trailing integrity hash: SHA-256 of everything written so far
        let trailing_hash = sha256_bytes(&buf);
        buf.extend_from_slice(&trailing_hash);

        Ok(buf)
    }

    /// Compute a deterministic hash over all target entries.
    fn compute_manifest_hash(&self) -> String {
        let mut hasher = Sha256::new();
        // Sort targets by file_path for determinism
        let mut sorted_targets: Vec<&PackTargetEntry> = self.manifest.targets.iter().collect();
        sorted_targets.sort_by(|a, b| a.file_path.cmp(&b.file_path));
        for t in sorted_targets {
            let line = format!("{}:{}:{}\n", t.file_path, t.size_bytes, t.sha256);
            hasher.update(line.as_bytes());
        }
        hex::encode(hasher.finalize())
    }
}

// ── PackVerifier ────────────────────────────────────────────────────────────

/// Verifier for validating `.oxp` distribution packages.
pub struct PackVerifier;

impl PackVerifier {
    /// Verify the package trailing integrity hash and return the manifest.
    pub fn verify_integrity(package_data: &[u8]) -> Result<PackManifest> {
        let min_size = OXP_MAGIC.len() + 4 + INTEGRITY_HASH_LEN;
        if package_data.len() < min_size {
            bail!("package data too small ({} bytes)", package_data.len());
        }

        // Verify trailing hash
        let payload_len = package_data.len() - INTEGRITY_HASH_LEN;
        let payload = &package_data[..payload_len];
        let stored_hash = &package_data[payload_len..];
        let computed_hash = sha256_bytes(payload);
        if stored_hash != computed_hash.as_slice() {
            bail!("integrity hash mismatch: package data is corrupted or tampered");
        }

        Self::read_manifest(package_data)
    }

    /// Verify the package signature using a public/shared key.
    pub fn verify_signature(package_data: &[u8], public_key: &[u8]) -> Result<bool> {
        let manifest = Self::verify_integrity(package_data)?;

        let stored_signature = match &manifest.integrity.signature {
            Some(sig) => sig.clone(),
            None => bail!("package has no signature to verify"),
        };

        let expected_sig_bytes =
            double_hash_sign(public_key, manifest.integrity.manifest_hash.as_bytes());
        let expected_hex = hex::encode(expected_sig_bytes);

        Ok(stored_signature == expected_hex)
    }

    /// Extract the manifest from a package without verifying integrity.
    pub fn read_manifest(package_data: &[u8]) -> Result<PackManifest> {
        let (manifest, _offset) = parse_manifest(package_data)?;
        Ok(manifest)
    }

    /// Extract a specific file from the package by its path.
    pub fn extract_file(package_data: &[u8], file_path: &str) -> Result<Vec<u8>> {
        let files = parse_files(package_data)?;
        for (path, data) in files {
            if path == file_path {
                return Ok(data);
            }
        }
        bail!("file not found in package: {}", file_path);
    }

    /// List all files contained in the package.
    pub fn list_files(package_data: &[u8]) -> Result<Vec<PackTargetEntry>> {
        let manifest = Self::read_manifest(package_data)?;
        Ok(manifest.targets)
    }
}

// ── PackRegistry ────────────────────────────────────────────────────────────

/// Registry for tracking installed packages.
pub struct PackRegistry {
    packages: Vec<InstalledPack>,
}

impl Default for PackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PackRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
        }
    }

    /// Register an installed pack.
    pub fn register(&mut self, manifest: PackManifest, install_path: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.packages.push(InstalledPack {
            manifest,
            install_path: install_path.to_string(),
            installed_at: now,
        });
    }

    /// Remove a pack by name. Returns error if not found.
    pub fn unregister(&mut self, name: &str) -> Result<()> {
        let idx = self
            .packages
            .iter()
            .position(|p| p.manifest.name == name)
            .with_context(|| format!("package '{}' not found in registry", name))?;
        self.packages.remove(idx);
        Ok(())
    }

    /// Find an installed pack by name.
    pub fn find(&self, name: &str) -> Option<&InstalledPack> {
        self.packages.iter().find(|p| p.manifest.name == name)
    }

    /// Find all packs that contain targets in the given category.
    pub fn find_by_category(&self, category: &str) -> Vec<&InstalledPack> {
        self.packages
            .iter()
            .filter(|p| p.manifest.targets.iter().any(|t| t.category == category))
            .collect()
    }

    /// List all installed packs.
    pub fn list_all(&self) -> &[InstalledPack] {
        &self.packages
    }

    /// Check which dependencies from the given manifest are missing.
    /// Returns names of missing dependencies.
    pub fn check_dependencies(&self, manifest: &PackManifest) -> Vec<String> {
        manifest
            .dependencies
            .iter()
            .filter(|dep| {
                !self
                    .packages
                    .iter()
                    .any(|installed| installed.manifest.name == dep.name)
            })
            .map(|dep| dep.name.clone())
            .collect()
    }
}

// ── Internal helpers ────────────────────────────────────────────────────────

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().to_vec()
}

/// Parse and validate the magic bytes, then extract the manifest from the package.
/// Returns the manifest and the byte offset immediately after the manifest JSON.
fn parse_manifest(data: &[u8]) -> Result<(PackManifest, usize)> {
    if data.len() < 8 {
        bail!("package data too small to contain header");
    }
    if &data[..4] != OXP_MAGIC {
        bail!("invalid OXP magic bytes");
    }

    let manifest_len = u32::from_le_bytes(
        data[4..8]
            .try_into()
            .with_context(|| "reading manifest length")?,
    ) as usize;

    let manifest_end = 8 + manifest_len;
    if data.len() < manifest_end {
        bail!(
            "package data truncated: need {} bytes for manifest, have {}",
            manifest_end,
            data.len()
        );
    }

    let manifest: PackManifest = serde_json::from_slice(&data[8..manifest_end])
        .with_context(|| "failed to deserialize manifest JSON")?;

    Ok((manifest, manifest_end))
}

/// Parse all file entries from the package data.
fn parse_files(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    let (_manifest, mut offset) = parse_manifest(data)?;

    // Strip trailing hash for bounds checking
    let payload_end = if data.len() >= INTEGRITY_HASH_LEN {
        data.len() - INTEGRITY_HASH_LEN
    } else {
        data.len()
    };

    if offset + 4 > payload_end {
        bail!("package data truncated: cannot read file count");
    }

    let file_count = u32::from_le_bytes(
        data[offset..offset + 4]
            .try_into()
            .with_context(|| "reading file count")?,
    ) as usize;
    offset += 4;

    let mut files = Vec::with_capacity(file_count);

    for i in 0..file_count {
        // path_len: u16 LE
        if offset + 2 > payload_end {
            bail!("truncated at file {} path length", i);
        }
        let path_len = u16::from_le_bytes(
            data[offset..offset + 2]
                .try_into()
                .with_context(|| format!("reading path length for file {}", i))?,
        ) as usize;
        offset += 2;

        // path bytes
        if offset + path_len > payload_end {
            bail!("truncated at file {} path data", i);
        }
        let path = std::str::from_utf8(&data[offset..offset + path_len])
            .with_context(|| format!("file {} path is not valid UTF-8", i))?
            .to_string();
        offset += path_len;

        // data_len: u32 LE
        if offset + 4 > payload_end {
            bail!("truncated at file {} data length", i);
        }
        let data_len = u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .with_context(|| format!("reading data length for file {}", i))?,
        ) as usize;
        offset += 4;

        // data bytes
        if offset + data_len > payload_end {
            bail!("truncated at file {} data", i);
        }
        let file_data = data[offset..offset + data_len].to_vec();
        offset += data_len;

        files.push((path, file_data));
    }

    Ok(files)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_basic_builder() -> PackBuilder {
        let mut b = PackBuilder::new("test-pack", "1.0.0", "tester");
        b.set_description("A test package");
        b.set_license("MIT");
        b.set_created_at(1700000000);
        b
    }

    // 1. Build an empty package (no files)
    #[test]
    fn build_empty_package() {
        let b = make_basic_builder();
        let data = b.build().expect("should succeed");
        assert!(data.len() > OXP_MAGIC.len() + INTEGRITY_HASH_LEN);
        assert_eq!(&data[..4], OXP_MAGIC);
    }

    // 2. Build with one file and verify integrity
    #[test]
    fn build_and_verify_one_file() {
        let mut b = make_basic_builder();
        b.add_target_file("model.dat", "meshes", b"triangle-data")
            .expect("should succeed");
        let data = b.build().expect("should succeed");
        let manifest = PackVerifier::verify_integrity(&data).expect("should succeed");
        assert_eq!(manifest.name, "test-pack");
        assert_eq!(manifest.targets.len(), 1);
        assert_eq!(manifest.targets[0].name, "model.dat");
    }

    // 3. Build with multiple files
    #[test]
    fn build_multiple_files() {
        let mut b = make_basic_builder();
        b.add_target_file("a.bin", "cat_a", b"alpha")
            .expect("should succeed");
        b.add_target_file("b.bin", "cat_b", b"beta")
            .expect("should succeed");
        b.add_target_file("c.bin", "cat_a", b"gamma")
            .expect("should succeed");
        let data = b.build().expect("should succeed");
        let manifest = PackVerifier::verify_integrity(&data).expect("should succeed");
        assert_eq!(manifest.targets.len(), 3);
    }

    // 4. Integrity check fails on tampered data
    #[test]
    fn integrity_fails_on_tampered_data() {
        let mut b = make_basic_builder();
        b.add_target_file("x.bin", "cat", b"data")
            .expect("should succeed");
        let mut data = b.build().expect("should succeed");
        // Tamper a byte in the middle
        let mid = data.len() / 2;
        data[mid] ^= 0xFF;
        assert!(PackVerifier::verify_integrity(&data).is_err());
    }

    // 5. Signed build and verification
    #[test]
    fn signed_build_and_verify() {
        let key = b"my-secret-key";
        let mut b = make_basic_builder();
        b.add_target_file("asset.glb", "models", b"glb-content")
            .expect("should succeed");
        let data = b.build_signed(key).expect("should succeed");
        let ok = PackVerifier::verify_signature(&data, key).expect("should succeed");
        assert!(ok);
    }

    // 6. Wrong key fails signature verification
    #[test]
    fn wrong_key_fails_signature() {
        let mut b = make_basic_builder();
        b.add_target_file("f.bin", "cat", b"stuff")
            .expect("should succeed");
        let data = b.build_signed(b"correct-key").expect("should succeed");
        let ok = PackVerifier::verify_signature(&data, b"wrong-key").expect("should succeed");
        assert!(!ok);
    }

    // 7. Unsigned package has no signature to verify
    #[test]
    fn unsigned_package_signature_check_fails() {
        let mut b = make_basic_builder();
        b.add_target_file("f.bin", "cat", b"stuff")
            .expect("should succeed");
        let data = b.build().expect("should succeed");
        assert!(PackVerifier::verify_signature(&data, b"any-key").is_err());
    }

    // 8. Extract a specific file
    #[test]
    fn extract_file_by_path() {
        let mut b = make_basic_builder();
        b.add_target_file("mesh.obj", "models", b"obj-content")
            .expect("should succeed");
        b.add_target_file("tex.png", "textures", b"png-bytes")
            .expect("should succeed");
        let data = b.build().expect("should succeed");
        let extracted =
            PackVerifier::extract_file(&data, "textures/tex.png").expect("should succeed");
        assert_eq!(extracted, b"png-bytes");
    }

    // 9. Extract non-existent file returns error
    #[test]
    fn extract_missing_file() {
        let b = make_basic_builder();
        let data = b.build().expect("should succeed");
        assert!(PackVerifier::extract_file(&data, "no/such/file").is_err());
    }

    // 10. List files returns target entries
    #[test]
    fn list_files_returns_targets() {
        let mut b = make_basic_builder();
        b.add_target_file("a.bin", "cat_a", b"aaa")
            .expect("should succeed");
        b.add_target_file("b.bin", "cat_b", b"bbb")
            .expect("should succeed");
        let data = b.build().expect("should succeed");
        let files = PackVerifier::list_files(&data).expect("should succeed");
        assert_eq!(files.len(), 2);
    }

    // 11. Read manifest extracts metadata correctly
    #[test]
    fn read_manifest_metadata() {
        let mut b = make_basic_builder();
        b.add_dependency("base-pack", ">=1.0");
        let data = b.build().expect("should succeed");
        let manifest = PackVerifier::read_manifest(&data).expect("should succeed");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author, "tester");
        assert_eq!(manifest.license, "MIT");
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(manifest.dependencies[0].name, "base-pack");
    }

    // 12. PackRegistry basic operations
    #[test]
    fn registry_register_find_unregister() {
        let mut reg = PackRegistry::new();
        let manifest = PackManifest {
            name: "my-pack".to_string(),
            version: "0.1.0".to_string(),
            author: "author".to_string(),
            description: String::new(),
            license: "MIT".to_string(),
            created_at: 0,
            targets: vec![PackTargetEntry {
                name: "f.bin".to_string(),
                category: "meshes".to_string(),
                file_path: "meshes/f.bin".to_string(),
                size_bytes: 100,
                sha256: "abc123".to_string(),
            }],
            dependencies: Vec::new(),
            integrity: PackIntegrity {
                algorithm: "sha256".to_string(),
                manifest_hash: String::new(),
                signature: None,
            },
        };
        reg.register(manifest, "/tmp/my-pack");
        assert!(reg.find("my-pack").is_some());
        assert!(reg.find("nonexistent").is_none());
        assert_eq!(reg.list_all().len(), 1);
        reg.unregister("my-pack").expect("should succeed");
        assert!(reg.find("my-pack").is_none());
        assert_eq!(reg.list_all().len(), 0);
    }

    // 13. PackRegistry find_by_category
    #[test]
    fn registry_find_by_category() {
        let mut reg = PackRegistry::new();
        let make_manifest = |name: &str, cat: &str| PackManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            author: "a".to_string(),
            description: String::new(),
            license: String::new(),
            created_at: 0,
            targets: vec![PackTargetEntry {
                name: "f".to_string(),
                category: cat.to_string(),
                file_path: format!("{}/f", cat),
                size_bytes: 0,
                sha256: String::new(),
            }],
            dependencies: Vec::new(),
            integrity: PackIntegrity {
                algorithm: "sha256".to_string(),
                manifest_hash: String::new(),
                signature: None,
            },
        };

        reg.register(make_manifest("pack-a", "meshes"), "/a");
        reg.register(make_manifest("pack-b", "textures"), "/b");
        reg.register(make_manifest("pack-c", "meshes"), "/c");

        let meshes = reg.find_by_category("meshes");
        assert_eq!(meshes.len(), 2);
        let textures = reg.find_by_category("textures");
        assert_eq!(textures.len(), 1);
        let empty = reg.find_by_category("audio");
        assert!(empty.is_empty());
    }

    // 14. PackRegistry check_dependencies
    #[test]
    fn registry_check_dependencies() {
        let mut reg = PackRegistry::new();
        let base_manifest = PackManifest {
            name: "base-pack".to_string(),
            version: "1.0.0".to_string(),
            author: "a".to_string(),
            description: String::new(),
            license: String::new(),
            created_at: 0,
            targets: Vec::new(),
            dependencies: Vec::new(),
            integrity: PackIntegrity {
                algorithm: "sha256".to_string(),
                manifest_hash: String::new(),
                signature: None,
            },
        };
        reg.register(base_manifest, "/base");

        let dependent = PackManifest {
            name: "top-pack".to_string(),
            version: "1.0.0".to_string(),
            author: "a".to_string(),
            description: String::new(),
            license: String::new(),
            created_at: 0,
            targets: Vec::new(),
            dependencies: vec![
                PackDependency {
                    name: "base-pack".to_string(),
                    version_req: ">=1.0".to_string(),
                },
                PackDependency {
                    name: "missing-pack".to_string(),
                    version_req: ">=0.5".to_string(),
                },
            ],
            integrity: PackIntegrity {
                algorithm: "sha256".to_string(),
                manifest_hash: String::new(),
                signature: None,
            },
        };

        let missing = reg.check_dependencies(&dependent);
        assert_eq!(missing, vec!["missing-pack"]);
    }

    // 15. Empty name target file is rejected
    #[test]
    fn reject_empty_target_name() {
        let mut b = make_basic_builder();
        assert!(b.add_target_file("", "cat", b"data").is_err());
    }

    // 16. Package too small for header
    #[test]
    fn too_small_package_rejected() {
        assert!(PackVerifier::verify_integrity(b"OXP").is_err());
    }

    // 17. Wrong magic rejected
    #[test]
    fn wrong_magic_rejected() {
        let b = make_basic_builder();
        let mut data = b.build().expect("should succeed");
        data[0] = b'Z';
        // This will fail at integrity or magic check
        assert!(PackVerifier::read_manifest(&data).is_err());
    }

    // 18. Large file round-trip
    #[test]
    fn large_file_round_trip() {
        let mut b = make_basic_builder();
        let large_data = vec![0xABu8; 100_000];
        b.add_target_file("big.bin", "data", &large_data)
            .expect("should succeed");
        let pkg = b.build().expect("should succeed");
        let extracted = PackVerifier::extract_file(&pkg, "data/big.bin").expect("should succeed");
        assert_eq!(extracted.len(), 100_000);
        assert!(extracted.iter().all(|&b| b == 0xAB));
    }

    // 19. Manifest hash is deterministic
    #[test]
    fn manifest_hash_deterministic() {
        let mut b1 = make_basic_builder();
        b1.add_target_file("a.bin", "cat", b"data")
            .expect("should succeed");
        let hash1 = b1.compute_manifest_hash();

        let mut b2 = make_basic_builder();
        b2.add_target_file("a.bin", "cat", b"data")
            .expect("should succeed");
        let hash2 = b2.compute_manifest_hash();

        assert_eq!(hash1, hash2);
    }

    // 20. Unregister non-existent pack fails
    #[test]
    fn unregister_missing_pack_fails() {
        let mut reg = PackRegistry::new();
        assert!(reg.unregister("ghost").is_err());
    }

    // 21. SHA-256 in target entries is correct
    #[test]
    fn target_entry_sha256_matches() {
        let mut b = make_basic_builder();
        let file_data = b"hello oxihuman";
        b.add_target_file("hello.txt", "text", file_data)
            .expect("should succeed");
        let pkg = b.build().expect("should succeed");
        let manifest = PackVerifier::read_manifest(&pkg).expect("should succeed");
        let expected = sha256_hex(file_data);
        assert_eq!(manifest.targets[0].sha256, expected);
    }

    // 22. Default registry is empty
    #[test]
    fn default_registry_is_empty() {
        let reg = PackRegistry::default();
        assert!(reg.list_all().is_empty());
    }

    // 23. Integrity field populated in built manifest
    #[test]
    fn integrity_field_populated() {
        let mut b = make_basic_builder();
        b.add_target_file("f.bin", "cat", b"d")
            .expect("should succeed");
        let pkg = b.build().expect("should succeed");
        let manifest = PackVerifier::read_manifest(&pkg).expect("should succeed");
        assert_eq!(manifest.integrity.algorithm, "sha256");
        assert!(!manifest.integrity.manifest_hash.is_empty());
        assert!(manifest.integrity.signature.is_none());
    }

    // 24. Signed manifest has signature field
    #[test]
    fn signed_manifest_has_signature() {
        let mut b = make_basic_builder();
        b.add_target_file("f.bin", "cat", b"d")
            .expect("should succeed");
        let pkg = b.build_signed(b"key").expect("should succeed");
        let manifest = PackVerifier::read_manifest(&pkg).expect("should succeed");
        assert!(manifest.integrity.signature.is_some());
    }

    // 25. Multiple dependencies tracked
    #[test]
    fn multiple_dependencies() {
        let mut b = make_basic_builder();
        b.add_dependency("dep-a", ">=1.0");
        b.add_dependency("dep-b", ">=2.0");
        b.add_dependency("dep-c", ">=0.1");
        let pkg = b.build().expect("should succeed");
        let manifest = PackVerifier::read_manifest(&pkg).expect("should succeed");
        assert_eq!(manifest.dependencies.len(), 3);
    }
}
