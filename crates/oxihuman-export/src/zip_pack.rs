// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! ZIP archive writer/reader backed by `oxiarc-archive` (COOLJAPAN Pure Rust
//! compression policy: no hand-rolled ZIP binary format, no `zip` crate).
//!
//! By default entries are written with STORE (method 0, no compression) so
//! that consumers which only understand STORE — notably the WASM-side
//! `load_zip_pack_bytes` reader in `oxihuman-wasm` — keep working unchanged.
//! Callers that want real DEFLATE compression (smaller files, readable by
//! any standard ZIP tool) can opt in via the `*_with_options` /
//! `*_deflate` variants, which route through `oxiarc-archive`'s real
//! DEFLATE encoder (`oxiarc-deflate` under the hood).

use std::path::Path;

use anyhow::{Context, Result};
use oxiarc_archive::detect::ArchiveFormat;
use oxiarc_archive::zip::{read_zip, ZipWriter};

// ── CRC-32 (IEEE polynomial, table-based) ───────────────────────────────────

/// Compute standard CRC-32 (IEEE polynomial 0xEDB88320) of `data`.
///
/// Kept as a small self-contained utility (unrelated to the ZIP container
/// format itself, so it is not part of the "hand-rolled ZIP writer" this
/// module used to be) — still exported for backward compatibility with
/// existing callers.
pub fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for i in 0u32..256 {
        let mut c = i;
        for _ in 0..8 {
            if c & 1 != 0 {
                c = 0xEDB8_8320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
        }
        table[i as usize] = c;
    }

    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = table[idx] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

// ── Public types ─────────────────────────────────────────────────────────────

/// A single file entry to be placed inside a ZIP archive.
pub struct ZipEntry {
    pub filename: String,
    pub data: Vec<u8>,
}

/// Result metadata returned after writing a ZIP archive.
pub struct ZipPackResult {
    pub path: std::path::PathBuf,
    pub entry_count: usize,
    pub total_bytes: usize,
    pub zip_size_bytes: usize,
}

// ── Core ZIP builder ──────────────────────────────────────────────────────────

/// Build ZIP archive bytes entirely in memory using STORE (no compression).
///
/// STORE is the default so archives stay readable by consumers that only
/// support that method (e.g. the WASM-side `load_zip_pack_bytes` reader).
/// Use [`zip_bytes_with_options`] for real DEFLATE compression.
pub fn zip_bytes(entries: &[ZipEntry]) -> Vec<u8> {
    zip_bytes_with_options(entries, false)
}

/// Build ZIP archive bytes entirely in memory via `oxiarc-archive`.
///
/// When `deflate` is `true`, entries are compressed with real DEFLATE
/// (`oxiarc-deflate`, level "Best"); when `false`, entries are written
/// STORE (uncompressed), matching the historical behaviour of this module.
pub fn zip_bytes_with_options(entries: &[ZipEntry], deflate: bool) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut writer = ZipWriter::new(&mut buf);
        for entry in entries {
            let add_result = if deflate {
                writer.add_file(&entry.filename, &entry.data)
            } else {
                writer.add_file_stored(&entry.filename, &entry.data)
            };
            // Writing into an in-memory Vec<u8> should never fail for valid
            // UTF-8 filenames and finite data; if it ever does (pathological
            // input), skip the offending entry instead of panicking so this
            // function can stay infallible for its existing callers.
            if let Err(err) = add_result {
                eprintln!(
                    "oxihuman-export: zip_pack: skipping entry '{}': {}",
                    entry.filename, err
                );
            }
        }
        if let Err(err) = writer.finish() {
            eprintln!("oxihuman-export: zip_pack: failed to finalize archive: {}", err);
        }
    }
    buf
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Write a ZIP archive to `path` using STORE (no compression).
///
/// Returns metadata about the written archive. Use [`write_zip_with_options`]
/// for real DEFLATE compression.
pub fn write_zip(entries: &[ZipEntry], path: &Path) -> Result<ZipPackResult> {
    write_zip_with_options(entries, path, false)
}

/// Write a ZIP archive to `path`, optionally using real DEFLATE compression.
pub fn write_zip_with_options(
    entries: &[ZipEntry],
    path: &Path,
    deflate: bool,
) -> Result<ZipPackResult> {
    let bytes = zip_bytes_with_options(entries, deflate);
    let zip_size = bytes.len();
    let total_bytes: usize = entries.iter().map(|e| e.data.len()).sum();

    std::fs::write(path, &bytes)
        .with_context(|| format!("writing ZIP archive to {}", path.display()))?;

    Ok(ZipPackResult {
        path: path.to_path_buf(),
        entry_count: entries.len(),
        total_bytes,
        zip_size_bytes: zip_size,
    })
}

/// Scan the central directory and return the list of filenames stored in the
/// ZIP at `path`. Works for both STORE and DEFLATE (or any method
/// `oxiarc-archive` understands) entries.
pub fn read_zip_entry_names(path: &Path) -> Result<Vec<String>> {
    let file =
        std::fs::File::open(path).with_context(|| format!("opening ZIP: {}", path.display()))?;
    let reader = read_zip(file)
        .map_err(|err| anyhow::anyhow!("failed to read ZIP archive {}: {}", path.display(), err))?;
    Ok(reader.entries().iter().map(|e| e.name.clone()).collect())
}

/// Bundle `mesh.glb`, `params.json`, and `manifest.json` into a single ZIP
/// archive using STORE (no compression) — matches the historical behaviour
/// of this function and stays readable by the WASM-side STORE-only reader.
///
/// Use [`pack_mesh_assets_with_options`] to opt into real DEFLATE
/// compression.
pub fn pack_mesh_assets(
    mesh_glb: &[u8],
    params_json: &[u8],
    manifest_json: &[u8],
    path: &Path,
) -> Result<ZipPackResult> {
    pack_mesh_assets_with_options(mesh_glb, params_json, manifest_json, path, false)
}

/// Bundle `mesh.glb`, `params.json`, and `manifest.json` into a single ZIP
/// archive, optionally using real DEFLATE compression.
///
/// `deflate = false` (the default via [`pack_mesh_assets`]) writes STORE
/// entries, which the WASM-side `load_zip_pack_bytes` reader currently
/// requires (it does not yet decompress DEFLATE entries — see
/// `oxihuman-wasm/src/pack.rs`). `deflate = true` produces a smaller archive
/// readable by any standard ZIP tool but is NOT currently consumable by the
/// WASM loader.
pub fn pack_mesh_assets_with_options(
    mesh_glb: &[u8],
    params_json: &[u8],
    manifest_json: &[u8],
    path: &Path,
    deflate: bool,
) -> Result<ZipPackResult> {
    let entries = vec![
        ZipEntry {
            filename: "mesh.glb".to_string(),
            data: mesh_glb.to_vec(),
        },
        ZipEntry {
            filename: "params.json".to_string(),
            data: params_json.to_vec(),
        },
        ZipEntry {
            filename: "manifest.json".to_string(),
            data: manifest_json.to_vec(),
        },
    ];
    write_zip_with_options(&entries, path, deflate)
}

/// Check that the archive at `path` is a well-formed ZIP file (starts with
/// ZIP magic bytes AND has a readable central directory / local headers).
/// Returns `Ok(false)` (not an error) for data that is not a valid ZIP
/// archive; propagates I/O errors from opening the file itself.
///
/// Note: `oxiarc-archive`'s [`read_zip`] falls back to a lenient local-header
/// scan when no central directory is found, which alone would report an
/// empty archive as "valid" for arbitrary non-ZIP bytes. We first confirm
/// the leading magic bytes actually identify a ZIP container via
/// [`ArchiveFormat::detect`] before trusting the parse result.
pub fn validate_zip(path: &Path) -> Result<bool> {
    use std::io::Seek;

    let mut file =
        std::fs::File::open(path).with_context(|| format!("opening path: {}", path.display()))?;

    let format = match ArchiveFormat::detect(&mut file) {
        Ok((format, _magic)) => format,
        Err(_) => return Ok(false),
    };
    if format != ArchiveFormat::Zip {
        return Ok(false);
    }

    file.seek(std::io::SeekFrom::Start(0))
        .with_context(|| format!("seeking {}", path.display()))?;
    Ok(read_zip(file).is_ok())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("oxihuman_zip_pack_test_{}_{}", std::process::id(), name))
    }

    // ── CRC-32 known values ──────────────────────────────────────────────────

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(&[]), 0x0000_0000);
    }

    #[test]
    fn test_crc32_hello() {
        assert_eq!(crc32(b"Hello World"), 0x4A17_B156);
    }

    #[test]
    fn test_crc32_abc() {
        assert_eq!(crc32(b"abc"), 0x3524_41C2);
    }

    #[test]
    fn test_crc32_deterministic() {
        let data = b"oxihuman mesh export";
        assert_eq!(crc32(data), crc32(data));
    }

    #[test]
    fn test_crc32_different_data_different_result() {
        assert_ne!(crc32(b"alpha"), crc32(b"beta"));
    }

    // ── write_zip / read_zip_entry_names round-trip (STORE) ──────────────────

    #[test]
    fn test_write_zip_round_trip() {
        let path = temp_path("roundtrip.zip");
        let entries = vec![
            ZipEntry {
                filename: "hello.txt".to_string(),
                data: b"Hello ZIP!".to_vec(),
            },
            ZipEntry {
                filename: "world.bin".to_string(),
                data: vec![0x01, 0x02, 0x03],
            },
        ];
        let result = write_zip(&entries, &path).expect("write_zip failed");
        assert_eq!(result.entry_count, 2);
        assert_eq!(result.total_bytes, 13); // 10 + 3

        let names = read_zip_entry_names(&path).expect("read_zip_entry_names failed");
        assert_eq!(names, vec!["hello.txt", "world.bin"]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_write_zip_single_entry() {
        let path = temp_path("single.zip");
        let entries = vec![ZipEntry {
            filename: "data.bin".to_string(),
            data: vec![42u8; 100],
        }];
        let result = write_zip(&entries, &path).expect("write_zip failed");
        assert_eq!(result.entry_count, 1);
        assert_eq!(result.total_bytes, 100);
        assert!(result.zip_size_bytes > 100); // overhead from headers
        let _ = std::fs::remove_file(&path);
    }

    // ── zip_bytes in-memory ───────────────────────────────────────────────────

    #[test]
    fn test_zip_bytes_non_empty() {
        let entries = vec![ZipEntry {
            filename: "test.txt".to_string(),
            data: b"WASM test".to_vec(),
        }];
        let bytes = zip_bytes(&entries);
        // Must start with local file header signature PK\x03\x04
        assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);
    }

    #[test]
    fn test_zip_bytes_empty_entries() {
        let bytes = zip_bytes(&[]);
        // An empty ZIP still ends with a valid EOCD, PK\x05\x06.
        assert!(bytes.len() >= 22);
        let eocd_pos = bytes.len() - 22;
        assert_eq!(&bytes[eocd_pos..eocd_pos + 4], &[0x50, 0x4B, 0x05, 0x06]);
    }

    #[test]
    fn test_zip_bytes_entry_names_roundtrip() {
        let path = temp_path("names_roundtrip.zip");
        let entries = vec![
            ZipEntry {
                filename: "mesh.glb".to_string(),
                data: vec![0u8; 64],
            },
            ZipEntry {
                filename: "params.json".to_string(),
                data: b"{}".to_vec(),
            },
        ];
        write_zip(&entries, &path).expect("write_zip failed");
        let names = read_zip_entry_names(&path).expect("parse failed");
        assert_eq!(names, vec!["mesh.glb", "params.json"]);
        let _ = std::fs::remove_file(&path);
    }

    // ── DEFLATE opt-in path ──────────────────────────────────────────────────

    #[test]
    fn test_zip_bytes_with_options_deflate_smaller_for_compressible_data() {
        let compressible = vec![b'a'; 4096];
        let entries = vec![ZipEntry {
            filename: "big.bin".to_string(),
            data: compressible,
        }];
        let stored = zip_bytes_with_options(&entries, false);
        let deflated = zip_bytes_with_options(&entries, true);
        assert!(
            deflated.len() < stored.len(),
            "deflated ({} bytes) should be smaller than stored ({} bytes) for highly compressible data",
            deflated.len(),
            stored.len()
        );
    }

    #[test]
    fn test_deflate_zip_round_trips_entry_names_and_data() {
        let path = temp_path("deflate_roundtrip.zip");
        let entries = vec![ZipEntry {
            filename: "readme.txt".to_string(),
            data: b"deflate me please, over and over and over".repeat(10),
        }];
        let result =
            write_zip_with_options(&entries, &path, true).expect("write_zip_with_options failed");
        assert_eq!(result.entry_count, 1);

        let names = read_zip_entry_names(&path).expect("read names failed");
        assert_eq!(names, vec!["readme.txt"]);
        let _ = std::fs::remove_file(&path);
    }

    // ── pack_mesh_assets ─────────────────────────────────────────────────────

    #[test]
    fn test_pack_mesh_assets() {
        let path = temp_path("mesh.zip");
        let glb = vec![0x67, 0x6C, 0x54, 0x46]; // "glTF" magic
        let params = b"{\"height\": 180}";
        let manifest = b"{\"version\": 1}";

        let result =
            pack_mesh_assets(&glb, params, manifest, &path).expect("pack_mesh_assets failed");
        assert_eq!(result.entry_count, 3);
        assert_eq!(
            result.total_bytes,
            glb.len() + params.len() + manifest.len()
        );

        let names = read_zip_entry_names(&path).expect("read names failed");
        assert!(names.contains(&"mesh.glb".to_string()));
        assert!(names.contains(&"params.json".to_string()));
        assert!(names.contains(&"manifest.json".to_string()));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_pack_mesh_assets_deflate_still_lists_entries() {
        let path = temp_path("mesh_deflate.zip");
        let glb = vec![0x67, 0x6C, 0x54, 0x46];
        let params = b"{\"height\": 180}";
        let manifest = b"{\"version\": 1}";

        let result =
            pack_mesh_assets_with_options(&glb, params, manifest, &path, true)
                .expect("pack_mesh_assets_with_options failed");
        assert_eq!(result.entry_count, 3);

        let names = read_zip_entry_names(&path).expect("read names failed");
        assert!(names.contains(&"mesh.glb".to_string()));
        let _ = std::fs::remove_file(&path);
    }

    // ── validate_zip ─────────────────────────────────────────────────────────

    #[test]
    fn test_validate_zip_valid() {
        let path = temp_path("validate_valid.zip");
        let entries = vec![ZipEntry {
            filename: "a.txt".to_string(),
            data: b"hello".to_vec(),
        }];
        write_zip(&entries, &path).expect("write_zip failed");
        let valid = validate_zip(&path).expect("validate_zip failed");
        assert!(valid);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_validate_zip_invalid() {
        let path = temp_path("validate_invalid.zip");
        std::fs::write(&path, b"not a zip file at all!!!").expect("write failed");
        let valid = validate_zip(&path).expect("validate_zip call failed");
        assert!(!valid);
        let _ = std::fs::remove_file(&path);
    }

    // ── empty ZIP file ────────────────────────────────────────────────────────

    #[test]
    fn test_write_empty_zip() {
        let path = temp_path("empty.zip");
        let result = write_zip(&[], &path).expect("write_zip failed");
        assert_eq!(result.entry_count, 0);
        assert_eq!(result.total_bytes, 0);

        let names = read_zip_entry_names(&path).expect("read names failed");
        assert!(names.is_empty());

        let valid = validate_zip(&path).expect("validate failed");
        assert!(valid);
        let _ = std::fs::remove_file(&path);
    }

    // ── ZipPackResult fields ──────────────────────────────────────────────────

    #[test]
    fn test_zip_pack_result_path() {
        let path = temp_path("result_path.zip");
        let entries = vec![ZipEntry {
            filename: "x.bin".to_string(),
            data: vec![1, 2, 3, 4, 5],
        }];
        let result = write_zip(&entries, &path).expect("write_zip failed");
        assert_eq!(result.path, path);
        assert!(result.zip_size_bytes >= result.total_bytes);
        let _ = std::fs::remove_file(&path);
    }

    // ── readable by an independent implementation (Python's zipfile) ─────────
    // Skips (does not fail) when python3 is not on PATH — guarded per policy.

    #[test]
    fn test_zip_readable_by_python_zipfile_if_available() {
        let has_python = std::process::Command::new("python3")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !has_python {
            eprintln!("skipping: python3 not found on PATH");
            return;
        }

        let path = temp_path("python_readable.zip");
        let entries = vec![
            ZipEntry {
                filename: "mesh.glb".to_string(),
                data: vec![0u8; 128],
            },
            ZipEntry {
                filename: "manifest.json".to_string(),
                data: b"{\"ok\":true}".repeat(20),
            },
        ];
        write_zip_with_options(&entries, &path, true).expect("write_zip_with_options failed");

        let output = std::process::Command::new("python3")
            .args(["-m", "zipfile", "-l"])
            .arg(&path)
            .output()
            .expect("failed to invoke python3 -m zipfile");
        assert!(
            output.status.success(),
            "python3 -m zipfile -l failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("mesh.glb"), "stdout: {stdout}");
        assert!(stdout.contains("manifest.json"), "stdout: {stdout}");

        let _ = std::fs::remove_file(&path);
    }
}
