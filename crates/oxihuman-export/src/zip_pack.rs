// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Minimal hand-crafted ZIP file writer (STORE compression only, no external zip crate).
//!
//! Supports writing and reading ZIP archives with:
//! - Local file headers (signature 0x04034b50)
//! - Central directory entries
//! - End of central directory record (signature 0x06054b50)

use std::path::Path;

// ── little-endian helpers ────────────────────────────────────────────────────

fn write_u16_le(buf: &mut Vec<u8>, v: u16) {
    buf.push((v & 0xFF) as u8);
    buf.push((v >> 8) as u8);
}

fn write_u32_le(buf: &mut Vec<u8>, v: u32) {
    buf.push((v & 0xFF) as u8);
    buf.push(((v >> 8) & 0xFF) as u8);
    buf.push(((v >> 16) & 0xFF) as u8);
    buf.push((v >> 24) as u8);
}

fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    (data[offset] as u16) | ((data[offset + 1] as u16) << 8)
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    (data[offset] as u32)
        | ((data[offset + 1] as u32) << 8)
        | ((data[offset + 2] as u32) << 16)
        | ((data[offset + 3] as u32) << 24)
}

// ── CRC-32 (IEEE polynomial, table-based) ───────────────────────────────────

/// Compute standard CRC-32 (IEEE polynomial 0xEDB88320) of `data`.
pub fn crc32(data: &[u8]) -> u32 {
    // Build lookup table at call time (avoids global mutable state)
    let mut table = [0u32; 256];
    for i in 0u32..256 {
        let mut c = i;
        for _ in 0..8 {
            if c & 1 != 0 {
                c = 0xEDB88320 ^ (c >> 1);
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

// ── ZIP constants ─────────────────────────────────────────────────────────────

const LOCAL_FILE_HEADER_SIG: u32 = 0x04034B50;
const CENTRAL_DIR_SIG: u32 = 0x02014B50;
const END_OF_CENTRAL_DIR_SIG: u32 = 0x06054B50;

const VERSION_NEEDED: u16 = 20;
const VERSION_MADE_BY: u16 = 20; // MS-DOS + compatible

// ── Core ZIP builder ──────────────────────────────────────────────────────────

/// Build ZIP archive bytes entirely in memory.
///
/// Uses STORE compression (no deflate). Safe for WASM targets.
pub fn zip_bytes(entries: &[ZipEntry]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    // Track (local_header_offset, crc32, compressed_size, filename_bytes) per entry
    struct EntryMeta {
        offset: u32,
        crc: u32,
        size: u32,
        filename: Vec<u8>,
    }

    let mut metas: Vec<EntryMeta> = Vec::with_capacity(entries.len());

    // ── Local file entries ──────────────────────────────────────────────────
    for entry in entries {
        let fname_bytes = entry.filename.as_bytes();
        let data_len = entry.data.len() as u32;
        let crc = crc32(&entry.data);
        let offset = buf.len() as u32;

        // Local file header (30 bytes + filename)
        write_u32_le(&mut buf, LOCAL_FILE_HEADER_SIG); // signature
        write_u16_le(&mut buf, VERSION_NEEDED); // version needed
        write_u16_le(&mut buf, 0); // general purpose bit flag
        write_u16_le(&mut buf, 0); // compression method: STORE
        write_u16_le(&mut buf, 0); // last mod file time
        write_u16_le(&mut buf, 0); // last mod file date
        write_u32_le(&mut buf, crc); // crc-32
        write_u32_le(&mut buf, data_len); // compressed size
        write_u32_le(&mut buf, data_len); // uncompressed size
        write_u16_le(&mut buf, fname_bytes.len() as u16); // filename length
        write_u16_le(&mut buf, 0); // extra field length

        buf.extend_from_slice(fname_bytes); // filename
        buf.extend_from_slice(&entry.data); // file data

        metas.push(EntryMeta {
            offset,
            crc,
            size: data_len,
            filename: fname_bytes.to_vec(),
        });
    }

    // ── Central directory ───────────────────────────────────────────────────
    let central_dir_start = buf.len() as u32;

    for meta in &metas {
        write_u32_le(&mut buf, CENTRAL_DIR_SIG); // signature
        write_u16_le(&mut buf, VERSION_MADE_BY); // version made by
        write_u16_le(&mut buf, VERSION_NEEDED); // version needed to extract
        write_u16_le(&mut buf, 0); // general purpose bit flag
        write_u16_le(&mut buf, 0); // compression method: STORE
        write_u16_le(&mut buf, 0); // last mod file time
        write_u16_le(&mut buf, 0); // last mod file date
        write_u32_le(&mut buf, meta.crc); // crc-32
        write_u32_le(&mut buf, meta.size); // compressed size
        write_u32_le(&mut buf, meta.size); // uncompressed size
        write_u16_le(&mut buf, meta.filename.len() as u16); // filename length
        write_u16_le(&mut buf, 0); // extra field length
        write_u16_le(&mut buf, 0); // file comment length
        write_u16_le(&mut buf, 0); // disk number start
        write_u16_le(&mut buf, 0); // internal file attributes
        write_u32_le(&mut buf, 0); // external file attributes
        write_u32_le(&mut buf, meta.offset); // relative offset of local header
        buf.extend_from_slice(&meta.filename); // filename
    }

    let central_dir_size = buf.len() as u32 - central_dir_start;

    // ── End of central directory record ────────────────────────────────────
    write_u32_le(&mut buf, END_OF_CENTRAL_DIR_SIG); // signature
    write_u16_le(&mut buf, 0); // disk number
    write_u16_le(&mut buf, 0); // disk with start of central directory
    write_u16_le(&mut buf, entries.len() as u16); // entries on this disk
    write_u16_le(&mut buf, entries.len() as u16); // total entries
    write_u32_le(&mut buf, central_dir_size); // size of central directory
    write_u32_le(&mut buf, central_dir_start); // offset of central directory
    write_u16_le(&mut buf, 0); // comment length

    buf
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Write a ZIP archive to `path`.
///
/// Uses STORE compression (no deflate). Returns metadata about the written archive.
pub fn write_zip(entries: &[ZipEntry], path: &Path) -> anyhow::Result<ZipPackResult> {
    let bytes = zip_bytes(entries);
    let zip_size = bytes.len();
    let total_bytes: usize = entries.iter().map(|e| e.data.len()).sum();

    std::fs::write(path, &bytes)?;

    Ok(ZipPackResult {
        path: path.to_path_buf(),
        entry_count: entries.len(),
        total_bytes,
        zip_size_bytes: zip_size,
    })
}

/// Scan the central directory and return the list of filenames stored in the ZIP.
pub fn read_zip_entry_names(path: &Path) -> anyhow::Result<Vec<String>> {
    let data = std::fs::read(path)?;
    read_zip_entry_names_from_bytes(&data)
}

/// Inner helper that works on raw bytes (used by tests and public API).
fn read_zip_entry_names_from_bytes(data: &[u8]) -> anyhow::Result<Vec<String>> {
    // Find end-of-central-directory record by searching backwards for its signature.
    // The EOCD is at least 22 bytes; comment may follow (max 65535 bytes).
    if data.len() < 22 {
        anyhow::bail!("data too short to be a valid ZIP archive");
    }

    let eocd_offset = find_eocd(data)
        .ok_or_else(|| anyhow::anyhow!("end-of-central-directory record not found"))?;

    // Parse EOCD
    // offset +4 : disk number (2)
    // offset +6 : disk with cd start (2)
    // offset +8 : entries on this disk (2)
    // offset +10: total entries (2)
    // offset +12: cd size (4)
    // offset +16: cd offset (4)
    let cd_offset = read_u32_le(data, eocd_offset + 16) as usize;
    let total_entries = read_u16_le(data, eocd_offset + 10) as usize;

    let mut names = Vec::with_capacity(total_entries);
    let mut pos = cd_offset;

    for _ in 0..total_entries {
        if pos + 46 > data.len() {
            anyhow::bail!("central directory entry truncated at offset {pos}");
        }
        let sig = read_u32_le(data, pos);
        if sig != CENTRAL_DIR_SIG {
            anyhow::bail!("expected central directory signature at offset {pos}");
        }
        let fname_len = read_u16_le(data, pos + 28) as usize;
        let extra_len = read_u16_le(data, pos + 30) as usize;
        let comment_len = read_u16_le(data, pos + 32) as usize;

        let fname_start = pos + 46;
        if fname_start + fname_len > data.len() {
            anyhow::bail!("filename extends past end of data");
        }
        let fname_bytes = &data[fname_start..fname_start + fname_len];
        let fname = String::from_utf8_lossy(fname_bytes).into_owned();
        names.push(fname);

        pos += 46 + fname_len + extra_len + comment_len;
    }

    Ok(names)
}

/// Search backwards from the end of `data` for the EOCD signature.
fn find_eocd(data: &[u8]) -> Option<usize> {
    // EOCD signature as little-endian bytes: 50 4B 05 06
    const EOCD_SIG_BYTES: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];
    // Search from the end; the minimum comment length is 0, max 65535
    let search_start = data.len().saturating_sub(22 + 65535);
    for i in (search_start..=data.len().saturating_sub(22)).rev() {
        if data[i..i + 4] == EOCD_SIG_BYTES {
            return Some(i);
        }
    }
    None
}

/// Bundle `mesh.glb`, `params.json`, and `manifest.json` into a single ZIP archive.
pub fn pack_mesh_assets(
    mesh_glb: &[u8],
    params_json: &[u8],
    manifest_json: &[u8],
    path: &Path,
) -> anyhow::Result<ZipPackResult> {
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
    write_zip(&entries, path)
}

/// Check that the archive at `path` ends with a valid end-of-central-directory record.
pub fn validate_zip(path: &Path) -> anyhow::Result<bool> {
    let data = std::fs::read(path)?;
    Ok(find_eocd(&data).is_some())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // ── CRC-32 known values ──────────────────────────────────────────────────

    #[test]
    fn test_crc32_empty() {
        // CRC-32 of empty data is defined as 0x00000000
        assert_eq!(crc32(&[]), 0x0000_0000);
    }

    #[test]
    fn test_crc32_hello() {
        // Standard CRC-32("Hello World") = 0x4A17B156
        assert_eq!(crc32(b"Hello World"), 0x4A17B156);
    }

    #[test]
    fn test_crc32_abc() {
        // Standard CRC-32("abc") = 0x352441C2
        assert_eq!(crc32(b"abc"), 0x352441C2);
    }

    #[test]
    fn test_crc32_single_zero() {
        // CRC-32([0x00]) = 0xD202EF8D
        assert_eq!(crc32(&[0x00]), 0xD202EF8D);
    }

    #[test]
    fn test_crc32_all_ones_byte() {
        // CRC-32([0xFF]) = 0xFF000000
        assert_eq!(crc32(&[0xFF]), 0xFF000000);
    }

    #[test]
    fn test_crc32_deterministic() {
        // Same data always produces same result
        let data = b"oxihuman mesh export";
        assert_eq!(crc32(data), crc32(data));
    }

    #[test]
    fn test_crc32_different_data_different_result() {
        assert_ne!(crc32(b"alpha"), crc32(b"beta"));
    }

    // ── write_zip / read_zip_entry_names round-trip ──────────────────────────

    #[test]
    fn test_write_zip_round_trip() {
        let path = Path::new("/tmp/oxihuman_zip_pack_roundtrip.zip");
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
        let result = write_zip(&entries, path).expect("write_zip failed");
        assert_eq!(result.entry_count, 2);
        assert_eq!(result.total_bytes, 13); // 10 + 3

        let names = read_zip_entry_names(path).expect("read_zip_entry_names failed");
        assert_eq!(names, vec!["hello.txt", "world.bin"]);
    }

    #[test]
    fn test_write_zip_single_entry() {
        let path = Path::new("/tmp/oxihuman_zip_pack_single.zip");
        let entries = vec![ZipEntry {
            filename: "data.bin".to_string(),
            data: vec![42u8; 100],
        }];
        let result = write_zip(&entries, path).expect("write_zip failed");
        assert_eq!(result.entry_count, 1);
        assert_eq!(result.total_bytes, 100);
        assert!(result.zip_size_bytes > 100); // overhead from headers
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
        // Must end with EOCD signature PK\x05\x06
        let eocd_pos = bytes.len() - 22;
        assert_eq!(&bytes[eocd_pos..eocd_pos + 4], &[0x50, 0x4B, 0x05, 0x06]);
    }

    #[test]
    fn test_zip_bytes_empty_entries() {
        // An empty ZIP should still have a valid EOCD
        let bytes = zip_bytes(&[]);
        assert_eq!(bytes.len(), 22); // only EOCD
        assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x05, 0x06]);
    }

    #[test]
    fn test_zip_bytes_entry_names_roundtrip() {
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
        let bytes = zip_bytes(&entries);
        let names = read_zip_entry_names_from_bytes(&bytes).expect("parse failed");
        assert_eq!(names, vec!["mesh.glb", "params.json"]);
    }

    // ── pack_mesh_assets ─────────────────────────────────────────────────────

    #[test]
    fn test_pack_mesh_assets() {
        let path = Path::new("/tmp/oxihuman_zip_pack_mesh.zip");
        let glb = vec![0x67, 0x6C, 0x54, 0x46]; // "glTF" magic
        let params = b"{\"height\": 180}";
        let manifest = b"{\"version\": 1}";

        let result =
            pack_mesh_assets(&glb, params, manifest, path).expect("pack_mesh_assets failed");
        assert_eq!(result.entry_count, 3);
        assert_eq!(
            result.total_bytes,
            glb.len() + params.len() + manifest.len()
        );

        let names = read_zip_entry_names(path).expect("read names failed");
        assert!(names.contains(&"mesh.glb".to_string()));
        assert!(names.contains(&"params.json".to_string()));
        assert!(names.contains(&"manifest.json".to_string()));
    }

    // ── validate_zip ─────────────────────────────────────────────────────────

    #[test]
    fn test_validate_zip_valid() {
        let path = Path::new("/tmp/oxihuman_zip_pack_validate_valid.zip");
        let entries = vec![ZipEntry {
            filename: "a.txt".to_string(),
            data: b"hello".to_vec(),
        }];
        write_zip(&entries, path).expect("write_zip failed");
        let valid = validate_zip(path).expect("validate_zip failed");
        assert!(valid);
    }

    #[test]
    fn test_validate_zip_invalid() {
        let path = Path::new("/tmp/oxihuman_zip_pack_validate_invalid.zip");
        // Write garbage bytes — no EOCD signature
        std::fs::write(path, b"not a zip file at all!!!").expect("write failed");
        let valid = validate_zip(path).expect("validate_zip call failed");
        assert!(!valid);
    }

    // ── empty ZIP file ────────────────────────────────────────────────────────

    #[test]
    fn test_write_empty_zip() {
        let path = Path::new("/tmp/oxihuman_zip_pack_empty.zip");
        let result = write_zip(&[], path).expect("write_zip failed");
        assert_eq!(result.entry_count, 0);
        assert_eq!(result.total_bytes, 0);
        // An empty ZIP is 22 bytes (just the EOCD)
        assert_eq!(result.zip_size_bytes, 22);

        let names = read_zip_entry_names(path).expect("read names failed");
        assert!(names.is_empty());

        let valid = validate_zip(path).expect("validate failed");
        assert!(valid);
    }

    // ── ZipPackResult fields ──────────────────────────────────────────────────

    #[test]
    fn test_zip_pack_result_path() {
        let path = Path::new("/tmp/oxihuman_zip_pack_result_path.zip");
        let entries = vec![ZipEntry {
            filename: "x.bin".to_string(),
            data: vec![1, 2, 3, 4, 5],
        }];
        let result = write_zip(&entries, path).expect("write_zip failed");
        assert_eq!(result.path, path.to_path_buf());
        assert!(result.zip_size_bytes >= result.total_bytes);
    }

    // ── CRC stored in local header matches recomputed CRC ────────────────────

    #[test]
    fn test_local_header_crc_correct() {
        let data = b"check my crc";
        let entries = vec![ZipEntry {
            filename: "crc_test.txt".to_string(),
            data: data.to_vec(),
        }];
        let bytes = zip_bytes(&entries);
        // Local file header: 4(sig) + 2 + 2 + 2 + 2 + 2 + 4(crc) starts at offset 14
        let stored_crc = read_u32_le(&bytes, 14);
        assert_eq!(stored_crc, crc32(data));
    }
}
