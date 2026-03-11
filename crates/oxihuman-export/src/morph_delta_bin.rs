// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Compact binary streaming format for morph target deltas (OXMD).
//!
//! Format:
//! ```text
//! Magic:        b"OXMD"  (4 bytes)
//! Version:      u32 LE = 1
//! Vertex count: u32 LE
//! Target count: u32 LE
//! --- per target header (repeated target_count times) ---
//! Name length:  u16 LE
//! Name bytes:   [u8; name_length]  (UTF-8)
//! Delta count:  u32 LE
//! --- per delta (repeated delta_count times) ---
//! Vertex index: u32 LE
//! dx:           f32 LE
//! dy:           f32 LE
//! dz:           f32 LE
//! ```

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};
use oxihuman_core::parser::target::TargetFile;

/// Magic bytes for the OXMD format.
pub const OXMD_MAGIC: &[u8; 4] = b"OXMD";

/// Current version of the OXMD format.
pub const OXMD_VERSION: u32 = 1;

/// A single non-zero morph delta for one vertex.
#[derive(Debug, Clone, PartialEq)]
pub struct MorphDeltaEntry {
    pub vertex_index: u32,
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

/// A named morph target containing its non-zero deltas.
#[derive(Debug, Clone)]
pub struct MorphDeltaTarget {
    pub name: String,
    pub deltas: Vec<MorphDeltaEntry>,
}

/// The top-level container for an OXMD binary file.
#[derive(Debug, Clone)]
pub struct MorphDeltaBin {
    pub vertex_count: u32,
    pub targets: Vec<MorphDeltaTarget>,
}

/// Statistics about a `MorphDeltaBin`.
#[derive(Debug, Clone)]
pub struct MorphDeltaBinStats {
    pub total_deltas: usize,
    pub avg_deltas_per_target: f32,
    pub max_delta_magnitude: f32,
    pub file_size_estimate: usize,
}

// ─── write ─────────────────────────────────────────────────────────────────

/// Write a `MorphDeltaBin` to a binary file at `path`.
pub fn write_morph_delta_bin(bin: &MorphDeltaBin, path: &Path) -> Result<()> {
    let file = File::create(path).with_context(|| format!("cannot create {}", path.display()))?;
    let mut w = BufWriter::new(file);

    // header
    w.write_all(OXMD_MAGIC)?;
    w.write_all(&OXMD_VERSION.to_le_bytes())?;
    w.write_all(&bin.vertex_count.to_le_bytes())?;
    let target_count = bin.targets.len() as u32;
    w.write_all(&target_count.to_le_bytes())?;

    for target in &bin.targets {
        let name_bytes = target.name.as_bytes();
        let name_len = name_bytes.len() as u16;
        w.write_all(&name_len.to_le_bytes())?;
        w.write_all(name_bytes)?;

        let delta_count = target.deltas.len() as u32;
        w.write_all(&delta_count.to_le_bytes())?;

        for d in &target.deltas {
            w.write_all(&d.vertex_index.to_le_bytes())?;
            w.write_all(&d.dx.to_le_bytes())?;
            w.write_all(&d.dy.to_le_bytes())?;
            w.write_all(&d.dz.to_le_bytes())?;
        }
    }

    w.flush()?;
    Ok(())
}

// ─── read ──────────────────────────────────────────────────────────────────

/// Read and parse a `MorphDeltaBin` from a binary file at `path`.
pub fn read_morph_delta_bin(path: &Path) -> Result<MorphDeltaBin> {
    let file = File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    let mut r = BufReader::new(file);

    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)
        .context("failed to read magic bytes")?;
    if &magic != OXMD_MAGIC {
        bail!("invalid magic: expected OXMD, got {:?}", magic);
    }

    let version = read_u32(&mut r).context("failed to read version")?;
    if version != OXMD_VERSION {
        bail!("unsupported version: {}", version);
    }

    let vertex_count = read_u32(&mut r).context("failed to read vertex_count")?;
    let target_count = read_u32(&mut r).context("failed to read target_count")?;

    let mut targets = Vec::with_capacity(target_count as usize);
    for i in 0..target_count {
        let name_len =
            read_u16(&mut r).with_context(|| format!("target {i}: failed to read name_len"))?;
        let mut name_bytes = vec![0u8; name_len as usize];
        r.read_exact(&mut name_bytes)
            .with_context(|| format!("target {i}: failed to read name bytes"))?;
        let name = String::from_utf8(name_bytes)
            .with_context(|| format!("target {i}: name is not valid UTF-8"))?;

        let delta_count =
            read_u32(&mut r).with_context(|| format!("target {i}: failed to read delta_count"))?;

        let mut deltas = Vec::with_capacity(delta_count as usize);
        for j in 0..delta_count {
            let vertex_index =
                read_u32(&mut r).with_context(|| format!("target {i} delta {j}: vertex_index"))?;
            let dx = read_f32(&mut r).with_context(|| format!("target {i} delta {j}: dx"))?;
            let dy = read_f32(&mut r).with_context(|| format!("target {i} delta {j}: dy"))?;
            let dz = read_f32(&mut r).with_context(|| format!("target {i} delta {j}: dz"))?;
            deltas.push(MorphDeltaEntry {
                vertex_index,
                dx,
                dy,
                dz,
            });
        }

        targets.push(MorphDeltaTarget { name, deltas });
    }

    Ok(MorphDeltaBin {
        vertex_count,
        targets,
    })
}

// ─── validate ──────────────────────────────────────────────────────────────

/// Check magic bytes and version of a file without fully parsing it.
///
/// Returns `true` if the file is a valid OXMD v1 file.
pub fn validate_morph_delta_bin(path: &Path) -> Result<bool> {
    let file = File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    let mut r = BufReader::new(file);

    let mut magic = [0u8; 4];
    if r.read_exact(&mut magic).is_err() {
        return Ok(false);
    }
    if &magic != OXMD_MAGIC {
        return Ok(false);
    }

    let mut ver_bytes = [0u8; 4];
    if r.read_exact(&mut ver_bytes).is_err() {
        return Ok(false);
    }
    let version = u32::from_le_bytes(ver_bytes);
    Ok(version == OXMD_VERSION)
}

// ─── from_target_files ─────────────────────────────────────────────────────

/// Build a `MorphDeltaBin` from a slice of `(name, &TargetFile)` pairs.
pub fn from_target_files(targets: &[(String, &TargetFile)], vertex_count: u32) -> MorphDeltaBin {
    let targets_out = targets
        .iter()
        .map(|(name, tf)| {
            let deltas = tf
                .deltas
                .iter()
                .map(|d| MorphDeltaEntry {
                    vertex_index: d.vid,
                    dx: d.dx,
                    dy: d.dy,
                    dz: d.dz,
                })
                .collect();
            MorphDeltaTarget {
                name: name.clone(),
                deltas,
            }
        })
        .collect();

    MorphDeltaBin {
        vertex_count,
        targets: targets_out,
    }
}

// ─── stats ─────────────────────────────────────────────────────────────────

/// Compute statistics about a `MorphDeltaBin`.
pub fn morph_delta_stats(bin: &MorphDeltaBin) -> MorphDeltaBinStats {
    let total_deltas: usize = bin.targets.iter().map(|t| t.deltas.len()).sum();

    let avg_deltas_per_target = if bin.targets.is_empty() {
        0.0
    } else {
        total_deltas as f32 / bin.targets.len() as f32
    };

    let max_delta_magnitude = bin
        .targets
        .iter()
        .flat_map(|t| t.deltas.iter())
        .map(|d| (d.dx * d.dx + d.dy * d.dy + d.dz * d.dz).sqrt())
        .fold(0.0_f32, f32::max);

    // header: 4 + 4 + 4 + 4 = 16 bytes
    // per target: 2 + name_bytes + 4 + (4+4+4+4)*delta_count
    let file_size_estimate = 16
        + bin.targets.iter().fold(0usize, |acc, t| {
            acc + 2 + t.name.len() + 4 + t.deltas.len() * 16
        });

    MorphDeltaBinStats {
        total_deltas,
        avg_deltas_per_target,
        max_delta_magnitude,
        file_size_estimate,
    }
}

// ─── merge_bins ────────────────────────────────────────────────────────────

/// Merge two `MorphDeltaBin` instances that share the same `vertex_count`.
///
/// Returns an error if `vertex_count` values differ.
pub fn merge_bins(a: &MorphDeltaBin, b: &MorphDeltaBin) -> Result<MorphDeltaBin> {
    if a.vertex_count != b.vertex_count {
        bail!(
            "cannot merge bins with different vertex counts: {} vs {}",
            a.vertex_count,
            b.vertex_count
        );
    }
    let mut targets = a.targets.clone();
    targets.extend(b.targets.iter().cloned());
    Ok(MorphDeltaBin {
        vertex_count: a.vertex_count,
        targets,
    })
}

// ─── helpers ───────────────────────────────────────────────────────────────

#[inline]
fn read_u16(r: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

#[inline]
fn read_u32(r: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

#[inline]
fn read_f32(r: &mut impl Read) -> Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_core::parser::target::{parse_target, Delta};

    fn make_bin() -> MorphDeltaBin {
        MorphDeltaBin {
            vertex_count: 1000,
            targets: vec![
                MorphDeltaTarget {
                    name: "smile".to_string(),
                    deltas: vec![
                        MorphDeltaEntry {
                            vertex_index: 0,
                            dx: 0.1,
                            dy: 0.2,
                            dz: 0.3,
                        },
                        MorphDeltaEntry {
                            vertex_index: 5,
                            dx: -0.1,
                            dy: 0.0,
                            dz: 0.05,
                        },
                    ],
                },
                MorphDeltaTarget {
                    name: "brow_raise".to_string(),
                    deltas: vec![MorphDeltaEntry {
                        vertex_index: 100,
                        dx: 0.0,
                        dy: 0.5,
                        dz: 0.0,
                    }],
                },
            ],
        }
    }

    fn tmp_path(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("/tmp/{}", name))
    }

    // ── round-trip ──────────────────────────────────────────────────────────

    #[test]
    fn round_trip_basic() {
        let bin = make_bin();
        let path = tmp_path("oxmd_round_trip_basic.bin");
        write_morph_delta_bin(&bin, &path).unwrap();
        let loaded = read_morph_delta_bin(&path).unwrap();
        assert_eq!(loaded.vertex_count, bin.vertex_count);
        assert_eq!(loaded.targets.len(), bin.targets.len());
    }

    #[test]
    fn round_trip_names_preserved() {
        let bin = make_bin();
        let path = tmp_path("oxmd_round_trip_names.bin");
        write_morph_delta_bin(&bin, &path).unwrap();
        let loaded = read_morph_delta_bin(&path).unwrap();
        assert_eq!(loaded.targets[0].name, "smile");
        assert_eq!(loaded.targets[1].name, "brow_raise");
    }

    #[test]
    fn round_trip_deltas_preserved() {
        let bin = make_bin();
        let path = tmp_path("oxmd_round_trip_deltas.bin");
        write_morph_delta_bin(&bin, &path).unwrap();
        let loaded = read_morph_delta_bin(&path).unwrap();
        let d = &loaded.targets[0].deltas[0];
        assert_eq!(d.vertex_index, 0);
        assert!((d.dx - 0.1).abs() < 1e-6);
        assert!((d.dy - 0.2).abs() < 1e-6);
        assert!((d.dz - 0.3).abs() < 1e-6);
    }

    #[test]
    fn round_trip_vertex_count() {
        let bin = make_bin();
        let path = tmp_path("oxmd_round_trip_vc.bin");
        write_morph_delta_bin(&bin, &path).unwrap();
        let loaded = read_morph_delta_bin(&path).unwrap();
        assert_eq!(loaded.vertex_count, 1000);
    }

    #[test]
    fn round_trip_empty_targets() {
        let bin = MorphDeltaBin {
            vertex_count: 500,
            targets: vec![],
        };
        let path = tmp_path("oxmd_empty_targets.bin");
        write_morph_delta_bin(&bin, &path).unwrap();
        let loaded = read_morph_delta_bin(&path).unwrap();
        assert_eq!(loaded.vertex_count, 500);
        assert!(loaded.targets.is_empty());
    }

    #[test]
    fn round_trip_single_target_zero_deltas() {
        let bin = MorphDeltaBin {
            vertex_count: 200,
            targets: vec![MorphDeltaTarget {
                name: "empty_target".to_string(),
                deltas: vec![],
            }],
        };
        let path = tmp_path("oxmd_zero_deltas.bin");
        write_morph_delta_bin(&bin, &path).unwrap();
        let loaded = read_morph_delta_bin(&path).unwrap();
        assert_eq!(loaded.targets.len(), 1);
        assert_eq!(loaded.targets[0].name, "empty_target");
        assert!(loaded.targets[0].deltas.is_empty());
    }

    #[test]
    fn round_trip_unicode_name() {
        let bin = MorphDeltaBin {
            vertex_count: 100,
            targets: vec![MorphDeltaTarget {
                name: "スマイル_αβγ".to_string(),
                deltas: vec![MorphDeltaEntry {
                    vertex_index: 1,
                    dx: 0.1,
                    dy: 0.2,
                    dz: 0.3,
                }],
            }],
        };
        let path = tmp_path("oxmd_unicode_name.bin");
        write_morph_delta_bin(&bin, &path).unwrap();
        let loaded = read_morph_delta_bin(&path).unwrap();
        assert_eq!(loaded.targets[0].name, "スマイル_αβγ");
    }

    // ── validate ────────────────────────────────────────────────────────────

    #[test]
    fn validate_valid_file() {
        let bin = make_bin();
        let path = tmp_path("oxmd_validate_valid.bin");
        write_morph_delta_bin(&bin, &path).unwrap();
        assert!(validate_morph_delta_bin(&path).unwrap());
    }

    #[test]
    fn validate_invalid_magic() {
        let path = tmp_path("oxmd_bad_magic.bin");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"BAD!\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
            .unwrap();
        assert!(!validate_morph_delta_bin(&path).unwrap());
    }

    #[test]
    fn validate_wrong_version() {
        let path = tmp_path("oxmd_bad_version.bin");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"OXMD").unwrap();
        f.write_all(&99u32.to_le_bytes()).unwrap();
        assert!(!validate_morph_delta_bin(&path).unwrap());
    }

    // ── from_target_files ───────────────────────────────────────────────────

    #[test]
    fn from_target_files_basic() {
        let src = "0 0.1 0.2 0.3\n5 -0.1 0.0 0.05\n";
        let tf = parse_target("smile", src).unwrap();
        let pairs: Vec<(String, &TargetFile)> = vec![("smile".to_string(), &tf)];
        let bin = from_target_files(&pairs, 1000);
        assert_eq!(bin.vertex_count, 1000);
        assert_eq!(bin.targets.len(), 1);
        assert_eq!(bin.targets[0].name, "smile");
        assert_eq!(bin.targets[0].deltas.len(), 2);
    }

    #[test]
    fn from_target_files_empty_list() {
        let pairs: Vec<(String, &TargetFile)> = vec![];
        let bin = from_target_files(&pairs, 500);
        assert_eq!(bin.vertex_count, 500);
        assert!(bin.targets.is_empty());
    }

    #[test]
    fn from_target_files_delta_values() {
        let src = "42 1.0 2.0 3.0\n";
        let tf = parse_target("test", src).unwrap();
        let pairs: Vec<(String, &TargetFile)> = vec![("test".to_string(), &tf)];
        let bin = from_target_files(&pairs, 100);
        let d = &bin.targets[0].deltas[0];
        assert_eq!(d.vertex_index, 42);
        assert!((d.dx - 1.0).abs() < 1e-6);
        assert!((d.dy - 2.0).abs() < 1e-6);
        assert!((d.dz - 3.0).abs() < 1e-6);
    }

    // ── morph_delta_stats ───────────────────────────────────────────────────

    #[test]
    fn stats_total_deltas() {
        let bin = make_bin(); // 2 + 1 = 3 deltas
        let stats = morph_delta_stats(&bin);
        assert_eq!(stats.total_deltas, 3);
    }

    #[test]
    fn stats_avg_deltas() {
        let bin = make_bin();
        let stats = morph_delta_stats(&bin);
        assert!((stats.avg_deltas_per_target - 1.5).abs() < 1e-5);
    }

    #[test]
    fn stats_max_magnitude() {
        let bin = make_bin();
        let stats = morph_delta_stats(&bin);
        // target[1] delta[0]: (0,0.5,0) → magnitude = 0.5
        // target[0] delta[0]: (0.1,0.2,0.3) → magnitude ≈ 0.374
        assert!((stats.max_delta_magnitude - 0.5).abs() < 1e-5);
    }

    #[test]
    fn stats_empty_bin() {
        let bin = MorphDeltaBin {
            vertex_count: 0,
            targets: vec![],
        };
        let stats = morph_delta_stats(&bin);
        assert_eq!(stats.total_deltas, 0);
        assert!((stats.avg_deltas_per_target).abs() < 1e-6);
        assert!((stats.max_delta_magnitude).abs() < 1e-6);
    }

    #[test]
    fn stats_file_size_estimate_nonempty() {
        let bin = make_bin();
        let stats = morph_delta_stats(&bin);
        // Rough sanity: header (16) + targets
        assert!(stats.file_size_estimate > 16);
    }

    // ── merge_bins ──────────────────────────────────────────────────────────

    #[test]
    fn merge_bins_basic() {
        let a = MorphDeltaBin {
            vertex_count: 100,
            targets: vec![MorphDeltaTarget {
                name: "a".to_string(),
                deltas: vec![MorphDeltaEntry {
                    vertex_index: 0,
                    dx: 1.0,
                    dy: 0.0,
                    dz: 0.0,
                }],
            }],
        };
        let b = MorphDeltaBin {
            vertex_count: 100,
            targets: vec![MorphDeltaTarget {
                name: "b".to_string(),
                deltas: vec![],
            }],
        };
        let merged = merge_bins(&a, &b).unwrap();
        assert_eq!(merged.vertex_count, 100);
        assert_eq!(merged.targets.len(), 2);
        assert_eq!(merged.targets[0].name, "a");
        assert_eq!(merged.targets[1].name, "b");
    }

    #[test]
    fn merge_bins_mismatched_vertex_count_errors() {
        let a = MorphDeltaBin {
            vertex_count: 100,
            targets: vec![],
        };
        let b = MorphDeltaBin {
            vertex_count: 200,
            targets: vec![],
        };
        assert!(merge_bins(&a, &b).is_err());
    }

    #[test]
    fn merge_bins_round_trip() {
        let a = MorphDeltaBin {
            vertex_count: 50,
            targets: vec![MorphDeltaTarget {
                name: "x".to_string(),
                deltas: vec![MorphDeltaEntry {
                    vertex_index: 1,
                    dx: 0.5,
                    dy: 0.5,
                    dz: 0.5,
                }],
            }],
        };
        let b = MorphDeltaBin {
            vertex_count: 50,
            targets: vec![MorphDeltaTarget {
                name: "y".to_string(),
                deltas: vec![MorphDeltaEntry {
                    vertex_index: 2,
                    dx: -0.5,
                    dy: -0.5,
                    dz: -0.5,
                }],
            }],
        };
        let merged = merge_bins(&a, &b).unwrap();
        let path = tmp_path("oxmd_merge_round_trip.bin");
        write_morph_delta_bin(&merged, &path).unwrap();
        let loaded = read_morph_delta_bin(&path).unwrap();
        assert_eq!(loaded.targets.len(), 2);
        assert_eq!(loaded.targets[1].deltas[0].vertex_index, 2);
    }

    // ── extra coverage ──────────────────────────────────────────────────────

    #[test]
    fn read_invalid_magic_errors() {
        let path = tmp_path("oxmd_read_bad_magic.bin");
        let mut f = File::create(&path).unwrap();
        // Write garbage bytes
        f.write_all(&[0u8; 16]).unwrap();
        assert!(read_morph_delta_bin(&path).is_err());
    }

    #[test]
    fn from_target_files_multiple() {
        let src_a = "0 0.1 0.2 0.3\n";
        let src_b = "1 0.4 0.5 0.6\n2 0.7 0.8 0.9\n";
        let tf_a = parse_target("ta", src_a).unwrap();
        let tf_b = parse_target("tb", src_b).unwrap();
        let pairs: Vec<(String, &TargetFile)> =
            vec![("ta".to_string(), &tf_a), ("tb".to_string(), &tf_b)];
        let bin = from_target_files(&pairs, 300);
        assert_eq!(bin.targets.len(), 2);
        assert_eq!(bin.targets[0].deltas.len(), 1);
        assert_eq!(bin.targets[1].deltas.len(), 2);
    }

    // Ensure TargetFile's Delta type is used correctly (field access check)
    #[test]
    fn delta_fields_accessible() {
        let d = Delta {
            vid: 7,
            dx: 1.0,
            dy: 2.0,
            dz: 3.0,
        };
        assert_eq!(d.vid, 7);
        let entry = MorphDeltaEntry {
            vertex_index: d.vid,
            dx: d.dx,
            dy: d.dy,
            dz: d.dz,
        };
        assert_eq!(entry.vertex_index, 7);
    }
}
