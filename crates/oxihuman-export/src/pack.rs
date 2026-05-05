// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Asset pack builder: scan a targets directory → generate a verified manifest.

use anyhow::{Context, Result};
use oxihuman_core::integrity::hash_bytes;
use oxihuman_core::parser::target::parse_target;
use oxihuman_core::policy::{Policy, PolicyProfile};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Entry for a single verified target file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetEntry {
    pub name: String,
    pub path: String,
    pub sha256: String,
    pub delta_count: usize,
    pub allowed: bool,
}

/// Statistics for the built pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackStats {
    pub total_files: usize,
    pub allowed_files: usize,
    pub blocked_files: usize,
    pub total_deltas: usize,
    pub estimated_memory_bytes: usize,
}

impl PackStats {
    fn from_entries(entries: &[TargetEntry]) -> Self {
        let allowed: Vec<_> = entries.iter().filter(|e| e.allowed).collect();
        let blocked = entries.len() - allowed.len();
        let total_deltas: usize = allowed.iter().map(|e| e.delta_count).sum();
        // Each delta is (u32 vid + f32 dx + f32 dy + f32 dz) = 16 bytes
        let estimated_memory_bytes = total_deltas * 16;
        PackStats {
            total_files: entries.len(),
            allowed_files: allowed.len(),
            blocked_files: blocked,
            total_deltas,
            estimated_memory_bytes,
        }
    }
}

/// Configuration for the pack builder.
pub struct PackBuilderConfig {
    /// Root directory to scan for .target files (recursive).
    pub targets_dir: PathBuf,
    /// Policy to apply for filtering.
    pub policy: Policy,
    /// Maximum files to process (None = all).
    pub max_files: Option<usize>,
}

impl PackBuilderConfig {
    pub fn new(targets_dir: impl Into<PathBuf>) -> Self {
        PackBuilderConfig {
            targets_dir: targets_dir.into(),
            policy: Policy::new(PolicyProfile::Standard),
            max_files: None,
        }
    }
}

/// Result of building a pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub version: String,
    pub entries: Vec<TargetEntry>,
    pub stats: PackStats,
}

impl PackManifest {
    /// Serialize the manifest to TOML.
    pub fn to_toml(&self) -> Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Write the manifest to a file.
    pub fn write_to(&self, path: &Path) -> Result<()> {
        let content = self.to_toml()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load a manifest from a TOML file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

/// Scan a directory for .target files and build a verified pack manifest.
pub fn build_pack(config: PackBuilderConfig) -> Result<PackManifest> {
    let mut entries = Vec::new();
    let max = config.max_files.unwrap_or(usize::MAX);

    scan_dir(
        &config.targets_dir,
        &config.targets_dir,
        &config.policy,
        &mut entries,
        max,
    )
    .with_context(|| format!("scanning {}", config.targets_dir.display()))?;

    let stats = PackStats::from_entries(&entries);
    Ok(PackManifest {
        version: "0.1.0".to_string(),
        entries,
        stats,
    })
}

fn scan_dir(
    base: &Path,
    dir: &Path,
    policy: &Policy,
    entries: &mut Vec<TargetEntry>,
    max: usize,
) -> Result<()> {
    if entries.len() >= max {
        return Ok(());
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    for path in paths {
        if entries.len() >= max {
            break;
        }
        if path.is_dir() {
            scan_dir(base, &path, policy, entries, max)?;
        } else if path.extension().map(|e| e == "target").unwrap_or(false) {
            if let Some(entry) = process_target(&path, base, policy) {
                entries.push(entry);
            }
        }
    }
    Ok(())
}

fn process_target(path: &Path, base: &Path, policy: &Policy) -> Option<TargetEntry> {
    let data = std::fs::read(path).ok()?;
    let src = std::str::from_utf8(&data).ok()?;
    let name = path.file_stem()?.to_str()?.to_string();
    let sha256 = hash_bytes(&data);

    let parsed = parse_target(&name, src).ok()?;
    let delta_count = parsed.deltas.len();
    let allowed = policy.is_target_allowed(&name, &[]);

    let rel_path = path
        .strip_prefix(base)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());

    Some(TargetEntry {
        name,
        path: rel_path,
        sha256,
        delta_count,
        allowed,
    })
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Result of validating a single entry in a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryValidationResult {
    pub name: String,
    pub path: String,
    pub status: EntryStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryStatus {
    /// File exists and hash matches.
    Ok,
    /// File does not exist.
    Missing,
    /// File exists but hash does not match the manifest.
    HashMismatch { actual: String },
    /// Target is not allowed by policy (should not be in manifest).
    PolicyViolation,
}

/// Full validation report for a manifest.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub total: usize,
    pub ok: usize,
    pub missing: usize,
    pub hash_mismatches: usize,
    pub policy_violations: usize,
    pub results: Vec<EntryValidationResult>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.missing == 0 && self.hash_mismatches == 0 && self.policy_violations == 0
    }

    pub fn summary(&self) -> String {
        if self.is_valid() {
            format!("OK: {}/{} entries valid", self.ok, self.total)
        } else {
            format!(
                "INVALID: {} missing, {} hash mismatches, {} policy violations (of {} total)",
                self.missing, self.hash_mismatches, self.policy_violations, self.total
            )
        }
    }
}

/// Validate all entries in a `PackManifest` against the filesystem.
///
/// For each entry: check file exists, re-compute SHA-256, compare with stored hash.
pub fn validate_manifest(
    manifest: &PackManifest,
    base_dir: &Path,
    policy: &Policy,
) -> ValidationReport {
    let mut results = Vec::new();
    let mut ok = 0;
    let mut missing = 0;
    let mut hash_mismatches = 0;
    let mut policy_violations = 0;

    for entry in &manifest.entries {
        let full_path = base_dir.join(&entry.path);
        let status = if !policy.is_target_allowed(&entry.name, &[]) {
            policy_violations += 1;
            EntryStatus::PolicyViolation
        } else if !full_path.exists() {
            missing += 1;
            EntryStatus::Missing
        } else {
            match std::fs::read(&full_path) {
                Ok(data) => {
                    let actual = hash_bytes(&data);
                    if actual == entry.sha256 {
                        ok += 1;
                        EntryStatus::Ok
                    } else {
                        hash_mismatches += 1;
                        EntryStatus::HashMismatch { actual }
                    }
                }
                Err(_) => {
                    missing += 1;
                    EntryStatus::Missing
                }
            }
        };
        results.push(EntryValidationResult {
            name: entry.name.clone(),
            path: entry.path.clone(),
            status,
        });
    }

    ValidationReport {
        total: manifest.entries.len(),
        ok,
        missing,
        hash_mismatches,
        policy_violations,
        results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn makehuman_data_dir() -> std::path::PathBuf {
        std::env::var("MAKEHUMAN_DATA_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/oxihuman_nonexistent_data"))
    }
    fn targets_dir() -> std::path::PathBuf {
        makehuman_data_dir().join("targets")
    }

    #[test]
    fn build_pack_small_sample() {
        let dir = targets_dir().join("bodyshapes");
        if !dir.exists() {
            return;
        }
        let config = PackBuilderConfig {
            targets_dir: dir,
            policy: Policy::new(PolicyProfile::Standard),
            max_files: Some(5),
        };
        let manifest = build_pack(config).expect("should succeed");
        assert!(!manifest.entries.is_empty());
        assert!(manifest.stats.total_files <= 5);
        assert!(manifest.stats.total_deltas > 0);
        // All non-explicit targets should be allowed under Standard policy
        for e in &manifest.entries {
            if e.allowed {
                assert!(!e.sha256.is_empty());
                assert!(e.delta_count > 0);
            }
        }
    }

    #[test]
    fn pack_stats_estimated_memory() {
        let entries = vec![
            TargetEntry {
                name: "height".to_string(),
                path: "height.target".to_string(),
                sha256: "abc".to_string(),
                delta_count: 100,
                allowed: true,
            },
            TargetEntry {
                name: "explicit-content".to_string(),
                path: "explicit.target".to_string(),
                sha256: "def".to_string(),
                delta_count: 50,
                allowed: false,
            },
        ];
        let stats = PackStats::from_entries(&entries);
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.allowed_files, 1);
        assert_eq!(stats.blocked_files, 1);
        assert_eq!(stats.total_deltas, 100); // only allowed
        assert_eq!(stats.estimated_memory_bytes, 100 * 16);
    }

    #[test]
    fn manifest_to_toml_round_trip() {
        let manifest = PackManifest {
            version: "0.1.0".to_string(),
            entries: vec![],
            stats: PackStats {
                total_files: 0,
                allowed_files: 0,
                blocked_files: 0,
                total_deltas: 0,
                estimated_memory_bytes: 0,
            },
        };
        let toml_str = manifest.to_toml().expect("should succeed");
        assert!(toml_str.contains("version"));
    }

    #[test]
    fn build_pack_writes_manifest() {
        let dir = targets_dir().join("armslegs");
        if !dir.exists() {
            return;
        }
        let config = PackBuilderConfig {
            targets_dir: dir,
            policy: Policy::new(PolicyProfile::Standard),
            max_files: Some(3),
        };
        let manifest = build_pack(config).expect("should succeed");
        let out = std::path::PathBuf::from("/tmp/test_pack_manifest.toml");
        manifest.write_to(&out).expect("should succeed");
        assert!(out.exists());
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn validate_empty_manifest_is_valid() {
        let manifest = PackManifest {
            version: "0.1.0".to_string(),
            entries: vec![],
            stats: PackStats {
                total_files: 0,
                allowed_files: 0,
                blocked_files: 0,
                total_deltas: 0,
                estimated_memory_bytes: 0,
            },
        };
        let policy = Policy::new(PolicyProfile::Standard);
        let report = validate_manifest(&manifest, std::path::Path::new("/tmp"), &policy);
        assert!(report.is_valid());
        assert_eq!(report.total, 0);
    }

    #[test]
    fn validate_missing_file_detected() {
        let manifest = PackManifest {
            version: "0.1.0".to_string(),
            entries: vec![TargetEntry {
                name: "height".to_string(),
                path: "nonexistent.target".to_string(),
                sha256: "abc123".to_string(),
                delta_count: 10,
                allowed: true,
            }],
            stats: PackStats {
                total_files: 1,
                allowed_files: 1,
                blocked_files: 0,
                total_deltas: 10,
                estimated_memory_bytes: 160,
            },
        };
        let policy = Policy::new(PolicyProfile::Standard);
        let report = validate_manifest(&manifest, std::path::Path::new("/tmp"), &policy);
        assert!(!report.is_valid());
        assert_eq!(report.missing, 1);
    }

    #[test]
    fn validate_hash_mismatch_detected() {
        // Write a temp file
        let dir = std::path::PathBuf::from("/tmp");
        let filename = "oxihuman_test_target.target";
        std::fs::write(dir.join(filename), b"# test\n1 0.1 0.2 0.3\n").expect("should succeed");

        let manifest = PackManifest {
            version: "0.1.0".to_string(),
            entries: vec![TargetEntry {
                name: "test".to_string(),
                path: filename.to_string(),
                sha256: "wronghash000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                delta_count: 1,
                allowed: true,
            }],
            stats: PackStats {
                total_files: 1,
                allowed_files: 1,
                blocked_files: 0,
                total_deltas: 1,
                estimated_memory_bytes: 16,
            },
        };
        let policy = Policy::new(PolicyProfile::Standard);
        let report = validate_manifest(&manifest, &dir, &policy);
        assert!(!report.is_valid());
        assert_eq!(report.hash_mismatches, 1);
        assert!(matches!(
            &report.results[0].status,
            EntryStatus::HashMismatch { .. }
        ));
        std::fs::remove_file(dir.join(filename)).ok();
    }

    #[test]
    fn validate_real_pack() {
        let dir = targets_dir().join("bodyshapes");
        if !dir.exists() {
            return;
        }
        let config = PackBuilderConfig {
            targets_dir: dir.clone(),
            policy: Policy::new(PolicyProfile::Standard),
            max_files: Some(3),
        };
        let manifest = build_pack(config).expect("should succeed");
        let policy = Policy::new(PolicyProfile::Standard);
        let report = validate_manifest(&manifest, &dir, &policy);
        // All files we just hashed should validate correctly
        assert_eq!(
            report.hash_mismatches, 0,
            "freshly built manifest should have no hash mismatches"
        );
    }
}
