// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::{Read, Write};

use oxihuman_core::parser::target::{Delta, TargetFile};

/// Compression settings for morph deltas.
#[derive(Debug, Clone)]
pub struct CompressConfig {
    /// Deltas with magnitude < threshold are dropped (near-zero filter).
    pub threshold: f32,
    /// Quantization scale: delta → round(delta * scale) as i16.
    /// Higher scale = more precision but more i16 range needed.
    pub quantize_scale: f32,
}

impl CompressConfig {
    /// Lossless-ish: threshold 1e-6, scale 10000.0
    pub fn high_quality() -> Self {
        Self {
            threshold: 1e-6,
            quantize_scale: 10000.0,
        }
    }

    /// Balanced: threshold 1e-4, scale 1000.0
    pub fn balanced() -> Self {
        Self {
            threshold: 1e-4,
            quantize_scale: 1000.0,
        }
    }

    /// Aggressive: threshold 1e-3, scale 100.0
    pub fn aggressive() -> Self {
        Self {
            threshold: 1e-3,
            quantize_scale: 100.0,
        }
    }
}

impl Default for CompressConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

/// A quantized delta: vertex index + i16 components.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct QuantizedDelta {
    pub vid: u32,
    pub dx: i16,
    pub dy: i16,
    pub dz: i16,
}

/// Compress a TargetFile's deltas using quantization and near-zero filtering.
pub fn compress_target(target: &TargetFile, config: &CompressConfig) -> Vec<QuantizedDelta> {
    let scale = config.quantize_scale;
    let threshold_sq = config.threshold * config.threshold;
    target
        .deltas
        .iter()
        .filter(|d| d.dx * d.dx + d.dy * d.dy + d.dz * d.dz >= threshold_sq)
        .filter_map(|d| {
            let qx = (d.dx * scale).round() as i32;
            let qy = (d.dy * scale).round() as i32;
            let qz = (d.dz * scale).round() as i32;
            // Skip if all components are zero after quantization
            if qx == 0 && qy == 0 && qz == 0 {
                return None;
            }
            // Clamp to i16 range
            Some(QuantizedDelta {
                vid: d.vid,
                dx: qx.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
                dy: qy.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
                dz: qz.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            })
        })
        .collect()
}

/// Decompress quantized deltas back to f32 deltas (inverse of compress_target).
pub fn decompress_target(
    name: &str,
    deltas: &[QuantizedDelta],
    config: &CompressConfig,
) -> TargetFile {
    let scale = config.quantize_scale;
    let reconstructed = deltas
        .iter()
        .map(|q| Delta {
            vid: q.vid,
            dx: q.dx as f32 / scale,
            dy: q.dy as f32 / scale,
            dz: q.dz as f32 / scale,
        })
        .collect();
    TargetFile {
        name: name.to_string(),
        deltas: reconstructed,
    }
}

/// Compute compression ratio: compressed_count / original_count.
pub fn compression_ratio(original: &TargetFile, config: &CompressConfig) -> f32 {
    let orig_count = original.deltas.len();
    if orig_count == 0 {
        return 1.0;
    }
    let compressed = compress_target(original, config);
    compressed.len() as f32 / orig_count as f32
}

/// Compute maximum reconstruction error after quantize→dequantize roundtrip.
pub fn max_reconstruction_error(target: &TargetFile, config: &CompressConfig) -> f32 {
    let compressed = compress_target(target, config);
    let reconstructed = decompress_target(&target.name, &compressed, config);

    // Build a lookup from vid to reconstructed delta
    let mut recon_map = std::collections::HashMap::new();
    for d in &reconstructed.deltas {
        recon_map.insert(d.vid, d);
    }

    let mut max_err = 0.0_f32;
    for orig in &target.deltas {
        if let Some(recon) = recon_map.get(&orig.vid) {
            let ex = (orig.dx - recon.dx).abs();
            let ey = (orig.dy - recon.dy).abs();
            let ez = (orig.dz - recon.dz).abs();
            max_err = max_err.max(ex).max(ey).max(ez);
        } else {
            // Delta was filtered out; its original magnitude is the error
            let mag = orig.dx.abs().max(orig.dy.abs()).max(orig.dz.abs());
            max_err = max_err.max(mag);
        }
    }
    max_err
}

const MAGIC: &[u8; 4] = b"OXDQ";
const VERSION: u32 = 1;

/// Write a compressed target library to a binary file (OXDC-Q format).
/// Format:
///   magic: b"OXDQ" (4 bytes)
///   version: 1u32 (4 bytes)
///   scale: f32 (4 bytes, the quantize_scale used)
///   entry_count: u32 (4 bytes)
///   for each entry:
///     name_len: u32 + name bytes
///     delta_count: u32
///     for each delta: vid(u32) + dx(i16) + dy(i16) + dz(i16) = 10 bytes
pub fn write_compressed_cache(
    targets: &[(&str, Vec<QuantizedDelta>)],
    scale: f32,
    path: &std::path::Path,
) -> anyhow::Result<()> {
    let mut buf = Vec::new();

    buf.write_all(MAGIC)?;
    buf.write_all(&VERSION.to_le_bytes())?;
    buf.write_all(&scale.to_le_bytes())?;
    buf.write_all(&(targets.len() as u32).to_le_bytes())?;

    for (name, deltas) in targets {
        let name_bytes = name.as_bytes();
        buf.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
        buf.write_all(name_bytes)?;
        buf.write_all(&(deltas.len() as u32).to_le_bytes())?;
        for d in deltas {
            buf.write_all(&d.vid.to_le_bytes())?;
            buf.write_all(&d.dx.to_le_bytes())?;
            buf.write_all(&d.dy.to_le_bytes())?;
            buf.write_all(&d.dz.to_le_bytes())?;
        }
    }

    std::fs::write(path, &buf)?;
    Ok(())
}

/// Read a compressed target library from a binary file.
#[allow(clippy::type_complexity)]
pub fn read_compressed_cache(
    path: &std::path::Path,
) -> anyhow::Result<(Vec<(String, Vec<QuantizedDelta>)>, f32)> {
    let data = std::fs::read(path)?;
    let mut cursor = std::io::Cursor::new(data);

    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    anyhow::ensure!(&magic == MAGIC, "invalid OXDQ magic");

    let mut u32_buf = [0u8; 4];
    cursor.read_exact(&mut u32_buf)?;
    let version = u32::from_le_bytes(u32_buf);
    anyhow::ensure!(version == VERSION, "unsupported OXDQ version {version}");

    cursor.read_exact(&mut u32_buf)?;
    let scale = f32::from_le_bytes(u32_buf);

    cursor.read_exact(&mut u32_buf)?;
    let entry_count = u32::from_le_bytes(u32_buf) as usize;

    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        cursor.read_exact(&mut u32_buf)?;
        let name_len = u32::from_le_bytes(u32_buf) as usize;
        let mut name_bytes = vec![0u8; name_len];
        cursor.read_exact(&mut name_bytes)?;
        let name = String::from_utf8(name_bytes)?;

        cursor.read_exact(&mut u32_buf)?;
        let delta_count = u32::from_le_bytes(u32_buf) as usize;

        let mut deltas = Vec::with_capacity(delta_count);
        for _ in 0..delta_count {
            let mut vid_buf = [0u8; 4];
            cursor.read_exact(&mut vid_buf)?;
            let vid = u32::from_le_bytes(vid_buf);

            let mut i16_buf = [0u8; 2];
            cursor.read_exact(&mut i16_buf)?;
            let dx = i16::from_le_bytes(i16_buf);
            cursor.read_exact(&mut i16_buf)?;
            let dy = i16::from_le_bytes(i16_buf);
            cursor.read_exact(&mut i16_buf)?;
            let dz = i16::from_le_bytes(i16_buf);

            deltas.push(QuantizedDelta { vid, dx, dy, dz });
        }
        entries.push((name, deltas));
    }

    Ok((entries, scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_target() -> TargetFile {
        TargetFile {
            name: "test".into(),
            deltas: vec![
                Delta {
                    vid: 0,
                    dx: 0.1,
                    dy: -0.05,
                    dz: 0.0,
                },
                Delta {
                    vid: 1,
                    dx: 0.0001,
                    dy: 0.0,
                    dz: 0.0,
                }, // tiny, should be filtered
                Delta {
                    vid: 2,
                    dx: -0.3,
                    dy: 0.2,
                    dz: 0.1,
                },
            ],
        }
    }

    #[test]
    fn compress_filters_near_zero() {
        let target = sample_target();
        let config = CompressConfig::balanced();
        let compressed = compress_target(&target, &config);
        // vid=1 (0.0001, 0.0, 0.0) is below threshold 1e-4 magnitude, should be filtered
        assert!(
            !compressed.iter().any(|q| q.vid == 1),
            "near-zero delta (vid=1) should be filtered out"
        );
        // vid=0 and vid=2 should remain
        assert!(compressed.iter().any(|q| q.vid == 0));
        assert!(compressed.iter().any(|q| q.vid == 2));
    }

    #[test]
    fn decompress_roundtrip_accuracy() {
        let target = sample_target();
        let config = CompressConfig::high_quality();
        let compressed = compress_target(&target, &config);
        let reconstructed = decompress_target(&target.name, &compressed, &config);

        // Build recon map
        let recon_map: std::collections::HashMap<u32, &Delta> =
            reconstructed.deltas.iter().map(|d| (d.vid, d)).collect();

        let mut max_err = 0.0f32;
        for orig in &target.deltas {
            if let Some(recon) = recon_map.get(&orig.vid) {
                max_err = max_err
                    .max((orig.dx - recon.dx).abs())
                    .max((orig.dy - recon.dy).abs())
                    .max((orig.dz - recon.dz).abs());
            }
        }
        assert!(
            max_err < 0.002,
            "roundtrip error {max_err} exceeds 0.002 for high_quality config"
        );
    }

    #[test]
    fn compression_ratio_less_than_one() {
        let target = sample_target();
        let config = CompressConfig::balanced();
        let ratio = compression_ratio(&target, &config);
        assert!(ratio <= 1.0, "compression ratio {ratio} should be <= 1.0");
    }

    #[test]
    fn max_error_high_quality_small() {
        let target = sample_target();
        let config = CompressConfig::high_quality();
        let err = max_reconstruction_error(&target, &config);
        assert!(
            err < 0.001,
            "max reconstruction error {err} should be < 0.001 for high_quality"
        );
    }

    #[test]
    fn quantized_delta_size() {
        assert_eq!(
            std::mem::size_of::<QuantizedDelta>(),
            10,
            "QuantizedDelta must be exactly 10 bytes (vid=4, dx+dy+dz=6)"
        );
    }

    #[test]
    fn write_read_cache_roundtrip() {
        let target = sample_target();
        let config = CompressConfig::balanced();
        let compressed = compress_target(&target, &config);
        let scale = config.quantize_scale;

        let path = std::path::PathBuf::from("/tmp/oxihuman_test_cache.oxdq");
        write_compressed_cache(&[("test", compressed.clone())], scale, &path)
            .expect("write should succeed");

        let (entries, read_scale) = read_compressed_cache(&path).expect("read should succeed");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "test");
        assert_eq!(entries[0].1.len(), compressed.len());
        assert!((read_scale - scale).abs() < f32::EPSILON);

        // Check all vids match
        for (orig, read) in compressed.iter().zip(entries[0].1.iter()) {
            let (o_vid, r_vid) = (orig.vid, read.vid);
            assert_eq!(o_vid, r_vid);
            let (o_dx, r_dx) = (orig.dx, read.dx);
            assert_eq!(o_dx, r_dx);
            let (o_dy, r_dy) = (orig.dy, read.dy);
            assert_eq!(o_dy, r_dy);
            let (o_dz, r_dz) = (orig.dz, read.dz);
            assert_eq!(o_dz, r_dz);
        }
    }

    #[test]
    fn aggressive_compresses_more() {
        let target = sample_target();
        let balanced = compress_target(&target, &CompressConfig::balanced());
        let aggressive = compress_target(&target, &CompressConfig::aggressive());
        assert!(
            aggressive.len() <= balanced.len(),
            "aggressive ({}) should remove at least as many deltas as balanced ({})",
            aggressive.len(),
            balanced.len()
        );
    }
}
