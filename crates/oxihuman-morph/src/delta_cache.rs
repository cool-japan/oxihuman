// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Binary delta cache for fast morph target loading.
//!
//! The cache format stores all loaded targets in a single compact binary file,
//! avoiding the overhead of text parsing on subsequent launches.
//!
//! Format (little-endian):
//!   [0..4]   Magic: b"OXDC"
//!   [4..8]   Version: u32 = 1
//!   [8..12]  Entry count: u32
//!   For each entry:
//!     [0..4]   Name length: u32
//!     [4..N]   Name bytes (UTF-8)
//!     [N..N+4] Delta count: u32
//!     For each delta: vid(u32) + dx(f32) + dy(f32) + dz(f32) = 16 bytes

use crate::target_lib::TargetLibrary;
use anyhow::{bail, Result};
use oxihuman_core::parser::target::{Delta, TargetFile};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const MAGIC: &[u8; 4] = b"OXDC";
const VERSION: u32 = 1;

/// Write all targets from a [`TargetLibrary`] to a binary cache file.
///
/// The file is written atomically via a [`BufWriter`] and will be created
/// (or truncated) at `path`.
pub fn write_cache(lib: &TargetLibrary, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut w = BufWriter::new(file);

    // Header
    w.write_all(MAGIC)?;
    w.write_all(&VERSION.to_le_bytes())?;

    // Collect entries so we can write the count first.
    // We need to count targets before iterating — collect into a vec.
    let entries: Vec<(&str, &[Delta])> = lib.iter().collect();
    let count = entries.len() as u32;
    w.write_all(&count.to_le_bytes())?;

    for (name, deltas) in entries {
        // Name
        let name_bytes = name.as_bytes();
        w.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
        w.write_all(name_bytes)?;

        // Deltas
        w.write_all(&(deltas.len() as u32).to_le_bytes())?;
        for d in deltas {
            w.write_all(&d.vid.to_le_bytes())?;
            w.write_all(&d.dx.to_le_bytes())?;
            w.write_all(&d.dy.to_le_bytes())?;
            w.write_all(&d.dz.to_le_bytes())?;
        }
    }

    w.flush()?;
    Ok(())
}

/// Read a binary cache file and return a [`Vec`] of [`TargetFile`].
///
/// Returns an error if the file cannot be read, or if the magic bytes or
/// version number do not match the expected values.
pub fn read_cache(path: &Path) -> Result<Vec<TargetFile>> {
    let file = std::fs::File::open(path)?;
    let mut r = BufReader::new(file);

    // Read and validate header
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        bail!("invalid cache magic: expected {:?}, got {:?}", MAGIC, magic);
    }

    let mut ver_buf = [0u8; 4];
    r.read_exact(&mut ver_buf)?;
    let version = u32::from_le_bytes(ver_buf);
    if version != VERSION {
        bail!(
            "unsupported cache version: expected {}, got {}",
            VERSION,
            version
        );
    }

    let mut count_buf = [0u8; 4];
    r.read_exact(&mut count_buf)?;
    let count = u32::from_le_bytes(count_buf) as usize;

    let mut targets = Vec::with_capacity(count);

    for _ in 0..count {
        // Name
        let mut nlen_buf = [0u8; 4];
        r.read_exact(&mut nlen_buf)?;
        let name_len = u32::from_le_bytes(nlen_buf) as usize;
        let mut name_bytes = vec![0u8; name_len];
        r.read_exact(&mut name_bytes)?;
        let name = String::from_utf8(name_bytes)?;

        // Deltas
        let mut dcount_buf = [0u8; 4];
        r.read_exact(&mut dcount_buf)?;
        let delta_count = u32::from_le_bytes(dcount_buf) as usize;

        let mut deltas = Vec::with_capacity(delta_count);
        for _ in 0..delta_count {
            let mut buf = [0u8; 16];
            r.read_exact(&mut buf)?;
            let vid = u32::from_le_bytes(buf[0..4].try_into().unwrap_or_default());
            let dx = f32::from_le_bytes(buf[4..8].try_into().unwrap_or_default());
            let dy = f32::from_le_bytes(buf[8..12].try_into().unwrap_or_default());
            let dz = f32::from_le_bytes(buf[12..16].try_into().unwrap_or_default());
            deltas.push(Delta { vid, dx, dy, dz });
        }

        targets.push(TargetFile { name, deltas });
    }

    Ok(targets)
}

/// Check if a cache file is valid (magic + version check).
///
/// Returns `false` if the file cannot be opened, is too short, or does not
/// contain the expected magic bytes and version number.
pub fn is_valid_cache(path: &Path) -> bool {
    (|| -> Option<bool> {
        let file = std::fs::File::open(path).ok()?;
        let mut r = BufReader::new(file);

        let mut magic = [0u8; 4];
        r.read_exact(&mut magic).ok()?;
        if &magic != MAGIC {
            return Some(false);
        }

        let mut ver_buf = [0u8; 4];
        r.read_exact(&mut ver_buf).ok()?;
        let version = u32::from_le_bytes(ver_buf);
        Some(version == VERSION)
    })()
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ParamState;
    use oxihuman_core::parser::target::{Delta, TargetFile};

    fn make_lib_with_targets(count: usize) -> TargetLibrary {
        let mut lib = TargetLibrary::new();
        for i in 0..count {
            let t = TargetFile {
                name: format!("target_{}", i),
                deltas: vec![
                    Delta {
                        vid: i as u32,
                        dx: i as f32 * 0.1,
                        dy: i as f32 * 0.2,
                        dz: i as f32 * 0.3,
                    },
                    Delta {
                        vid: i as u32 + 100,
                        dx: -(i as f32),
                        dy: 0.0,
                        dz: 1.5,
                    },
                ],
            };
            lib.add(t, Box::new(|_: &ParamState| 1.0));
        }
        lib
    }

    #[test]
    fn write_and_read_cache_roundtrip() {
        let lib = make_lib_with_targets(3);
        let path = std::path::PathBuf::from("/tmp/oxihuman_test_roundtrip.cache");

        write_cache(&lib, &path).expect("write_cache failed");
        let targets = read_cache(&path).expect("read_cache failed");

        assert_eq!(targets.len(), 3);
        for (i, target) in targets.iter().enumerate().take(3) {
            assert_eq!(target.name, format!("target_{}", i));
            assert_eq!(target.deltas.len(), 2);
        }
    }

    #[test]
    fn cache_magic_correct() {
        let lib = make_lib_with_targets(1);
        let path = std::path::PathBuf::from("/tmp/oxihuman_test_magic.cache");

        write_cache(&lib, &path).expect("write_cache failed");

        let bytes = std::fs::read(&path).expect("read file failed");
        assert_eq!(
            &bytes[0..4],
            b"OXDC",
            "first 4 bytes must be magic b\"OXDC\""
        );
    }

    #[test]
    fn cache_version_correct() {
        let lib = make_lib_with_targets(1);
        let path = std::path::PathBuf::from("/tmp/oxihuman_test_version.cache");

        write_cache(&lib, &path).expect("write_cache failed");

        let bytes = std::fs::read(&path).expect("read file failed");
        let version = u32::from_le_bytes(bytes[4..8].try_into().expect("should succeed"));
        assert_eq!(version, 1, "bytes[4..8] must encode version 1 as u32 LE");
    }

    #[test]
    fn is_valid_cache_true_for_valid_file() {
        let lib = make_lib_with_targets(2);
        let path = std::path::PathBuf::from("/tmp/oxihuman_test_valid.cache");

        write_cache(&lib, &path).expect("write_cache failed");
        assert!(
            is_valid_cache(&path),
            "is_valid_cache should return true for a correctly written cache"
        );
    }

    #[test]
    fn is_valid_cache_false_for_random_data() {
        let path = std::path::PathBuf::from("/tmp/oxihuman_test_garbage.cache");
        std::fs::write(&path, b"\xDE\xAD\xBE\xEF\x00\x00\x00\x00").expect("write garbage failed");
        assert!(
            !is_valid_cache(&path),
            "is_valid_cache should return false for garbage data"
        );
    }

    #[test]
    fn delta_values_preserved() {
        let precise_dx: f32 = 0.123_456_79;
        let precise_dy: f32 = -9.876_543;
        let precise_dz: f32 = 1.234_567_8e-5;

        let mut lib = TargetLibrary::new();
        let t = TargetFile {
            name: "precise".to_string(),
            deltas: vec![Delta {
                vid: 42,
                dx: precise_dx,
                dy: precise_dy,
                dz: precise_dz,
            }],
        };
        lib.add(t, Box::new(|_: &ParamState| 1.0));

        let path = std::path::PathBuf::from("/tmp/oxihuman_test_delta_values.cache");
        write_cache(&lib, &path).expect("write_cache failed");
        let targets = read_cache(&path).expect("read_cache failed");

        assert_eq!(targets.len(), 1);
        let d = &targets[0].deltas[0];
        assert_eq!(d.vid, 42);
        // f32 bits must be exactly preserved after binary roundtrip
        assert_eq!(
            d.dx.to_bits(),
            precise_dx.to_bits(),
            "dx not preserved exactly"
        );
        assert_eq!(
            d.dy.to_bits(),
            precise_dy.to_bits(),
            "dy not preserved exactly"
        );
        assert_eq!(
            d.dz.to_bits(),
            precise_dz.to_bits(),
            "dz not preserved exactly"
        );
    }

    #[test]
    fn empty_library_cache() {
        let lib = TargetLibrary::new();
        let path = std::path::PathBuf::from("/tmp/oxihuman_test_empty.cache");

        write_cache(&lib, &path).expect("write_cache on empty library failed");
        let targets = read_cache(&path).expect("read_cache on empty cache failed");

        assert_eq!(
            targets.len(),
            0,
            "empty library should produce 0 entries in cache"
        );

        // Verify the entry-count field in the file is 0
        let bytes = std::fs::read(&path).expect("read file failed");
        let count = u32::from_le_bytes(bytes[8..12].try_into().expect("should succeed"));
        assert_eq!(count, 0, "entry count field must be 0 for empty library");
    }
}
