// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use std::path::Path;

use crate::integrity::hash_bytes;
use crate::manifest::AssetManifest;

/// A checksum record for a single file inside a pack.
pub struct FileRecord {
    pub relative_path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

/// The result of verifying a pack directory against a set of `FileRecord`s.
pub struct PackVerifyReport {
    pub total_files: usize,
    pub ok_files: usize,
    /// Files that existed but had a wrong hash or size.
    pub failed_files: Vec<String>,
    /// Files listed in records but not found on disk.
    pub missing_files: Vec<String>,
    pub is_valid: bool,
}

impl PackVerifyReport {
    /// Human-readable one-liner summary.
    pub fn summary(&self) -> String {
        if self.is_valid {
            format!(
                "Pack OK: {}/{} files verified",
                self.ok_files, self.total_files
            )
        } else {
            format!(
                "Pack INVALID: {}/{} ok, {} failed, {} missing",
                self.ok_files,
                self.total_files,
                self.failed_files.len(),
                self.missing_files.len()
            )
        }
    }
}

/// Scan all files in `pack_dir` recursively and build a `Vec<FileRecord>`.
pub fn scan_pack(pack_dir: &Path) -> Result<Vec<FileRecord>> {
    let mut records = Vec::new();
    collect_files(pack_dir, pack_dir, &mut records)?;
    Ok(records)
}

fn collect_files(root: &Path, current: &Path, records: &mut Vec<FileRecord>) -> Result<()> {
    for entry in std::fs::read_dir(current)
        .with_context(|| format!("reading directory {}", current.display()))?
    {
        let entry = entry.with_context(|| format!("dir entry in {}", current.display()))?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(root, &path, records)?;
        } else {
            let data =
                std::fs::read(&path).with_context(|| format!("reading file {}", path.display()))?;
            let sha256 = hash_bytes(&data);
            let size_bytes = data.len() as u64;

            // Relative path uses forward slashes.
            let relative_path = path
                .strip_prefix(root)
                .with_context(|| "stripping root prefix")?
                .to_string_lossy()
                .replace('\\', "/");

            records.push(FileRecord {
                relative_path,
                sha256,
                size_bytes,
            });
        }
    }
    Ok(())
}

/// Verify all files in `pack_dir` against the expected `records`.
pub fn verify_pack(pack_dir: &Path, records: &[FileRecord]) -> PackVerifyReport {
    let total_files = records.len();
    let mut ok_files = 0usize;
    let mut failed_files = Vec::new();
    let mut missing_files = Vec::new();

    for rec in records {
        let full_path = pack_dir.join(&rec.relative_path);
        match std::fs::read(&full_path) {
            Err(_) => {
                missing_files.push(rec.relative_path.clone());
            }
            Ok(data) => {
                let actual_hash = hash_bytes(&data);
                let actual_size = data.len() as u64;
                if actual_hash == rec.sha256 && actual_size == rec.size_bytes {
                    ok_files += 1;
                } else {
                    failed_files.push(rec.relative_path.clone());
                }
            }
        }
    }

    let is_valid = failed_files.is_empty() && missing_files.is_empty();

    PackVerifyReport {
        total_files,
        ok_files,
        failed_files,
        missing_files,
        is_valid,
    }
}

/// Check that `pack_dir` contains `oxihuman_assets.toml` and that it is a
/// valid `AssetManifest`.
pub fn verify_manifest_present(pack_dir: &Path) -> Result<()> {
    let manifest_path = pack_dir.join("oxihuman_assets.toml");
    if !manifest_path.exists() {
        anyhow::bail!("manifest not found: {}", manifest_path.display());
    }
    AssetManifest::load(&manifest_path)
        .with_context(|| format!("parsing manifest {}", manifest_path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn tempdir() -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should succeed")
            .subsec_nanos();
        let path = PathBuf::from(format!("/tmp/oxihuman_pack_verify_test_{}", nanos));
        fs::create_dir_all(&path).expect("should succeed");
        path
    }

    fn write_file(path: &Path, content: &[u8]) {
        let mut f = fs::File::create(path).expect("should succeed");
        f.write_all(content).expect("should succeed");
    }

    // ------------------------------------------------------------------
    // scan_pack
    // ------------------------------------------------------------------

    #[test]
    fn scan_pack_finds_three_files() {
        let tmp = tempdir();
        write_file(&tmp.join("a.bin"), b"hello");
        write_file(&tmp.join("b.bin"), b"world");
        write_file(&tmp.join("c.bin"), b"rust");

        let records = scan_pack(&tmp).expect("should succeed");
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn scan_pack_records_correct_sha256() {
        let tmp = tempdir();
        let content = b"oxihuman test data";
        write_file(&tmp.join("data.bin"), content);

        let records = scan_pack(&tmp).expect("should succeed");
        assert_eq!(records.len(), 1);
        let expected = hash_bytes(content);
        assert_eq!(records[0].sha256, expected);
    }

    #[test]
    fn scan_pack_records_correct_size() {
        let tmp = tempdir();
        let content = b"1234567890"; // 10 bytes
        write_file(&tmp.join("size_test.bin"), content);

        let records = scan_pack(&tmp).expect("should succeed");
        assert_eq!(records[0].size_bytes, 10);
    }

    // ------------------------------------------------------------------
    // verify_pack — happy path
    // ------------------------------------------------------------------

    #[test]
    fn verify_pack_all_ok() {
        let tmp = tempdir();
        write_file(&tmp.join("a.bin"), b"alpha");
        write_file(&tmp.join("b.bin"), b"beta");
        write_file(&tmp.join("c.bin"), b"gamma");

        let records = scan_pack(&tmp).expect("should succeed");
        let report = verify_pack(&tmp, &records);

        assert!(report.is_valid);
        assert_eq!(report.ok_files, 3);
        assert!(report.failed_files.is_empty());
        assert!(report.missing_files.is_empty());
    }

    // ------------------------------------------------------------------
    // verify_pack — modified file
    // ------------------------------------------------------------------

    #[test]
    fn verify_pack_modified_file_appears_in_failed() {
        let tmp = tempdir();
        write_file(&tmp.join("good.bin"), b"good content");
        write_file(&tmp.join("bad.bin"), b"original");

        let records = scan_pack(&tmp).expect("should succeed");

        // Tamper with the file after scanning.
        write_file(&tmp.join("bad.bin"), b"tampered!");

        let report = verify_pack(&tmp, &records);

        assert!(!report.is_valid);
        assert_eq!(report.failed_files.len(), 1);
        assert!(report.failed_files[0].contains("bad.bin"));
        assert!(report.missing_files.is_empty());
    }

    // ------------------------------------------------------------------
    // verify_pack — missing file
    // ------------------------------------------------------------------

    #[test]
    fn verify_pack_missing_file_appears_in_missing() {
        let tmp = tempdir();
        write_file(&tmp.join("present.bin"), b"here");
        write_file(&tmp.join("gone.bin"), b"temporary");

        let records = scan_pack(&tmp).expect("should succeed");

        // Remove the file after scanning.
        fs::remove_file(tmp.join("gone.bin")).expect("should succeed");

        let report = verify_pack(&tmp, &records);

        assert!(!report.is_valid);
        assert_eq!(report.missing_files.len(), 1);
        assert!(report.missing_files[0].contains("gone.bin"));
        assert!(report.failed_files.is_empty());
    }

    // ------------------------------------------------------------------
    // verify_pack — empty records
    // ------------------------------------------------------------------

    #[test]
    fn verify_pack_empty_records_trivially_valid() {
        let tmp = tempdir();
        let report = verify_pack(&tmp, &[]);
        assert!(report.is_valid);
        assert_eq!(report.ok_files, 0);
        assert_eq!(report.total_files, 0);
    }

    // ------------------------------------------------------------------
    // summary
    // ------------------------------------------------------------------

    #[test]
    fn summary_is_non_empty_when_valid() {
        let tmp = tempdir();
        write_file(&tmp.join("x.bin"), b"data");
        let records = scan_pack(&tmp).expect("should succeed");
        let report = verify_pack(&tmp, &records);
        assert!(!report.summary().is_empty());
        assert!(report.summary().contains("OK"));
    }

    #[test]
    fn summary_is_non_empty_when_invalid() {
        let tmp = tempdir();
        write_file(&tmp.join("x.bin"), b"original");
        let records = scan_pack(&tmp).expect("should succeed");
        write_file(&tmp.join("x.bin"), b"changed");
        let report = verify_pack(&tmp, &records);
        assert!(!report.summary().is_empty());
        assert!(report.summary().contains("INVALID"));
    }

    // ------------------------------------------------------------------
    // verify_manifest_present
    // ------------------------------------------------------------------

    #[test]
    fn verify_manifest_missing_returns_err() {
        let tmp = tempdir();
        let result = verify_manifest_present(&tmp);
        assert!(result.is_err());
    }

    #[test]
    fn verify_manifest_invalid_toml_returns_err() {
        let tmp = tempdir();
        write_file(&tmp.join("oxihuman_assets.toml"), b"not valid toml ][");
        let result = verify_manifest_present(&tmp);
        assert!(result.is_err());
    }

    #[test]
    fn verify_manifest_valid_toml_returns_ok() {
        let tmp = tempdir();
        let toml = r#"
version = "0.1.0"
base_mesh_path = "data/3dobjs/base.obj"
allowed_targets = ["height-up", "height-down"]
policy_profile = "Standard"
"#;
        write_file(&tmp.join("oxihuman_assets.toml"), toml.as_bytes());
        let result = verify_manifest_present(&tmp);
        assert!(result.is_ok());
    }
}
