// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Compressed morph target loading — pure Rust, no C/Fortran dependencies.
//!
//! Morph targets (blend shapes) are typically very sparse: only a small
//! fraction of vertices move.  This module exploits that sparsity with a
//! layered encoding:
//!
//! 1. **Delta coding** — store the difference between consecutive
//!    `[f64; 3]` values, which is near-zero for most entries.
//! 2. **Quantisation** — each f64 delta is mapped to a variable-length
//!    signed integer (multiply by a scale factor, round).
//! 3. **Run-length encoding** — consecutive zero triples are collapsed to a
//!    single (zero-marker, count) pair.
//! 4. **Varint encoding** — all resulting integers are written with a
//!    variable-width encoding (LEB128-style) for compact storage.
//!
//! The combination yields excellent ratios on typical morph data
//! (often 5–20× depending on sparsity).

use anyhow::{bail, ensure, Context, Result};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Scale factor for quantising f64 → i64 before varint encoding.
/// 1e7 preserves ~7 decimal digits which is more than enough for morph deltas
/// that are usually in the range ±0.01.
const QUANT_SCALE: f64 = 1e7;
const INV_QUANT_SCALE: f64 = 1.0 / QUANT_SCALE;

/// Sentinel value that signals the start of a zero-run in the encoded stream.
/// Chosen to be outside the plausible quantised range.
const ZERO_RUN_MARKER: i64 = i64::MIN;

/// Magic bytes at the start of a serialised [`LitePack`].
const LITE_PACK_MAGIC: &[u8; 4] = b"LMPK";

/// Current format version for [`LitePack`] serialisation.
const LITE_PACK_VERSION: u8 = 1;

// ---------------------------------------------------------------------------
// Varint helpers (LEB128-style, signed)
// ---------------------------------------------------------------------------

/// Encode a signed 64-bit integer using zig-zag + LEB128.
fn encode_varint(buf: &mut Vec<u8>, value: i64) {
    // Zig-zag: map negatives to odd positives.
    let mut v: u64 = ((value << 1) ^ (value >> 63)) as u64;
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            buf.push(byte);
            break;
        }
        buf.push(byte | 0x80);
    }
}

/// Decode a single zig-zag + LEB128 varint from the byte slice, returning
/// `(value, bytes_consumed)`.
fn decode_varint(data: &[u8]) -> Result<(i64, usize)> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    for (i, &byte) in data.iter().enumerate() {
        ensure!(
            shift < 64,
            "varint overflow (more than 9 continuation bytes)"
        );
        result |= ((byte & 0x7F) as u64) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            // Zig-zag decode.
            let signed = ((result >> 1) as i64) ^ (-((result & 1) as i64));
            return Ok((signed, i + 1));
        }
    }
    bail!("truncated varint at end of input");
}

// ---------------------------------------------------------------------------
// CompressedTarget
// ---------------------------------------------------------------------------

/// A morph-target delta array compressed with delta + RLE + varint encoding.
#[derive(Debug, Clone)]
pub struct CompressedTarget {
    compressed_data: Vec<u8>,
    vertex_count: usize,
    original_size: usize,
}

impl CompressedTarget {
    /// Compress an array of per-vertex deltas.
    ///
    /// Each entry is `[dx, dy, dz]` for the corresponding vertex.  Vertices
    /// with no displacement should be `[0.0, 0.0, 0.0]`.
    pub fn compress(deltas: &[[f64; 3]]) -> Result<Self> {
        let vertex_count = deltas.len();
        let original_size = vertex_count * 3 * 8; // f64 × 3 per vertex

        // --- Step 1: delta-code + quantise --------------------------------
        let mut prev = [0i64; 3];
        let mut quant: Vec<[i64; 3]> = Vec::with_capacity(vertex_count);
        for d in deltas {
            let q = [quantise(d[0]), quantise(d[1]), quantise(d[2])];
            let delta_q = [q[0] - prev[0], q[1] - prev[1], q[2] - prev[2]];
            prev = q;
            quant.push(delta_q);
        }

        // --- Step 2: RLE for zero-triples + varint encode -----------------
        let mut encoded = Vec::with_capacity(original_size / 4); // rough guess
        let mut i = 0;
        while i < quant.len() {
            if quant[i] == [0, 0, 0] {
                // Count consecutive zero triples.
                let start = i;
                while i < quant.len() && quant[i] == [0, 0, 0] {
                    i += 1;
                }
                let run_len = (i - start) as i64;
                encode_varint(&mut encoded, ZERO_RUN_MARKER);
                encode_varint(&mut encoded, run_len);
            } else {
                encode_varint(&mut encoded, quant[i][0]);
                encode_varint(&mut encoded, quant[i][1]);
                encode_varint(&mut encoded, quant[i][2]);
                i += 1;
            }
        }

        Ok(Self {
            compressed_data: encoded,
            vertex_count,
            original_size,
        })
    }

    /// Decompress back to the original `[f64; 3]` per-vertex deltas.
    pub fn decompress(&self) -> Result<Vec<[f64; 3]>> {
        let mut out: Vec<[f64; 3]> = Vec::with_capacity(self.vertex_count);
        let data = &self.compressed_data;
        let mut pos = 0usize;
        let mut prev = [0i64; 3];

        while out.len() < self.vertex_count {
            ensure!(pos < data.len(), "compressed stream ended prematurely");

            let (val, consumed) =
                decode_varint(&data[pos..]).context("decoding first varint of triple")?;
            pos += consumed;

            if val == ZERO_RUN_MARKER {
                // Zero-run.
                let (run_len, consumed2) =
                    decode_varint(&data[pos..]).context("decoding zero-run length")?;
                pos += consumed2;
                ensure!(run_len > 0, "zero-run with non-positive length");
                let run_len = run_len as usize;
                for _ in 0..run_len {
                    // Delta is zero, so reconstructed == prev.
                    out.push(dequantise_triple(prev));
                }
            } else {
                // Non-zero triple; `val` is the first component.
                let (vy, c2) = decode_varint(&data[pos..]).context("decoding Y component")?;
                pos += c2;
                let (vz, c3) = decode_varint(&data[pos..]).context("decoding Z component")?;
                pos += c3;

                prev = [prev[0] + val, prev[1] + vy, prev[2] + vz];
                out.push(dequantise_triple(prev));
            }
        }

        Ok(out)
    }

    /// Ratio of compressed size to original size (lower is better).
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 1.0;
        }
        self.compressed_data.len() as f64 / self.original_size as f64
    }

    /// Number of vertices in this target.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Size of the compressed representation in bytes.
    #[inline]
    pub fn compressed_size(&self) -> usize {
        self.compressed_data.len()
    }
}

// ---------------------------------------------------------------------------
// Quantisation helpers
// ---------------------------------------------------------------------------

#[inline]
fn quantise(v: f64) -> i64 {
    (v * QUANT_SCALE).round() as i64
}

#[inline]
fn dequantise(v: i64) -> f64 {
    v as f64 * INV_QUANT_SCALE
}

#[inline]
fn dequantise_triple(q: [i64; 3]) -> [f64; 3] {
    [dequantise(q[0]), dequantise(q[1]), dequantise(q[2])]
}

// ---------------------------------------------------------------------------
// LitePack — lightweight bundle of compressed targets for browser delivery
// ---------------------------------------------------------------------------

/// A lightweight collection of named compressed morph targets, suitable for
/// streaming to the browser.
///
/// Serialisation layout (all integers little-endian):
///
/// ```text
/// [magic: 4 bytes "LMPK"]
/// [version: u8]
/// [n_meta: u32]
///   for each meta entry:
///     [key_len: u32][key bytes][val_len: u32][val bytes]
/// [n_targets: u32]
///   for each target:
///     [name_len: u32][name bytes]
///     [vertex_count: u64]
///     [original_size: u64]
///     [data_len: u32][compressed bytes]
/// ```
#[derive(Debug, Clone)]
pub struct LitePack {
    targets: Vec<(String, CompressedTarget)>,
    metadata: HashMap<String, String>,
}

impl Default for LitePack {
    fn default() -> Self {
        Self::new()
    }
}

impl LitePack {
    /// Create an empty pack.
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a named morph target from raw deltas.  The deltas are compressed
    /// immediately.
    pub fn add_target(&mut self, name: String, deltas: &[[f64; 3]]) -> Result<()> {
        let ct = CompressedTarget::compress(deltas)
            .with_context(|| format!("compressing target '{name}'"))?;
        self.targets.push((name, ct));
        Ok(())
    }

    /// Set a metadata key/value pair.
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Serialise the entire pack to bytes.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Magic + version.
        buf.extend_from_slice(LITE_PACK_MAGIC);
        buf.push(LITE_PACK_VERSION);

        // Metadata.
        write_u32_le(&mut buf, self.metadata.len() as u32);
        for (k, v) in &self.metadata {
            write_bytes_with_len(&mut buf, k.as_bytes());
            write_bytes_with_len(&mut buf, v.as_bytes());
        }

        // Targets.
        write_u32_le(&mut buf, self.targets.len() as u32);
        for (name, ct) in &self.targets {
            write_bytes_with_len(&mut buf, name.as_bytes());
            buf.extend_from_slice(&(ct.vertex_count as u64).to_le_bytes());
            buf.extend_from_slice(&(ct.original_size as u64).to_le_bytes());
            write_bytes_with_len(&mut buf, &ct.compressed_data);
        }

        Ok(buf)
    }

    /// Deserialise from bytes produced by [`serialize`](Self::serialize).
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        ensure!(data.len() >= 5, "data too short for LitePack header");
        ensure!(&data[0..4] == LITE_PACK_MAGIC, "bad LitePack magic");
        ensure!(data[4] == LITE_PACK_VERSION, "unsupported LitePack version");

        let mut pos = 5usize;

        // Metadata.
        let n_meta = read_u32_le(data, &mut pos)? as usize;
        let mut metadata = HashMap::with_capacity(n_meta);
        for _ in 0..n_meta {
            let key = read_string(data, &mut pos)?;
            let val = read_string(data, &mut pos)?;
            metadata.insert(key, val);
        }

        // Targets.
        let n_targets = read_u32_le(data, &mut pos)? as usize;
        let mut targets = Vec::with_capacity(n_targets);
        for _ in 0..n_targets {
            let name = read_string(data, &mut pos)?;
            let vertex_count = read_u64_le(data, &mut pos)? as usize;
            let original_size = read_u64_le(data, &mut pos)? as usize;
            let compressed_data = read_bytes(data, &mut pos)?;
            targets.push((
                name,
                CompressedTarget {
                    compressed_data,
                    vertex_count,
                    original_size,
                },
            ));
        }

        Ok(Self { targets, metadata })
    }

    /// List all target names in insertion order.
    pub fn target_names(&self) -> Vec<&str> {
        self.targets.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// Decompress a single target by name.
    pub fn get_target(&self, name: &str) -> Result<Vec<[f64; 3]>> {
        let (_, ct) = self
            .targets
            .iter()
            .find(|(n, _)| n == name)
            .with_context(|| format!("target '{name}' not found in LitePack"))?;
        ct.decompress()
    }

    /// Number of targets stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    /// Whether this pack contains zero targets.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Access the metadata map.
    #[inline]
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

// ---------------------------------------------------------------------------
// Serialisation helpers (little-endian, length-prefixed)
// ---------------------------------------------------------------------------

#[inline]
fn write_u32_le(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_bytes_with_len(buf: &mut Vec<u8>, bytes: &[u8]) {
    write_u32_le(buf, bytes.len() as u32);
    buf.extend_from_slice(bytes);
}

fn read_u32_le(data: &[u8], pos: &mut usize) -> Result<u32> {
    ensure!(
        *pos + 4 <= data.len(),
        "unexpected EOF reading u32 at offset {pos}",
    );
    let v = u32::from_le_bytes(
        data[*pos..*pos + 4]
            .try_into()
            .context("u32 slice conversion")?,
    );
    *pos += 4;
    Ok(v)
}

fn read_u64_le(data: &[u8], pos: &mut usize) -> Result<u64> {
    ensure!(
        *pos + 8 <= data.len(),
        "unexpected EOF reading u64 at offset {pos}",
    );
    let v = u64::from_le_bytes(
        data[*pos..*pos + 8]
            .try_into()
            .context("u64 slice conversion")?,
    );
    *pos += 8;
    Ok(v)
}

fn read_bytes(data: &[u8], pos: &mut usize) -> Result<Vec<u8>> {
    let len = read_u32_le(data, pos)? as usize;
    ensure!(
        *pos + len <= data.len(),
        "unexpected EOF reading {len} bytes at offset {pos}",
    );
    let v = data[*pos..*pos + len].to_vec();
    *pos += len;
    Ok(v)
}

fn read_string(data: &[u8], pos: &mut usize) -> Result<String> {
    let bytes = read_bytes(data, pos)?;
    String::from_utf8(bytes).context("invalid UTF-8 in LitePack string")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: assert two f64-triples are close enough after round-tripping
    /// through quantisation.
    fn assert_close(a: [f64; 3], b: [f64; 3], eps: f64) {
        for i in 0..3 {
            assert!(
                (a[i] - b[i]).abs() < eps,
                "component {i}: {:.10} vs {:.10} (diff {:.2e})",
                a[i],
                b[i],
                (a[i] - b[i]).abs()
            );
        }
    }

    #[test]
    fn varint_round_trip() {
        let values = [
            0i64,
            1,
            -1,
            127,
            -128,
            1000000,
            -999999,
            i64::MAX,
            i64::MIN + 1,
        ];
        for &v in &values {
            let mut buf = Vec::new();
            encode_varint(&mut buf, v);
            let (decoded, consumed) = decode_varint(&buf).expect("should succeed");
            assert_eq!(v, decoded, "varint round-trip failed for {v}");
            assert_eq!(consumed, buf.len());
        }
    }

    #[test]
    fn compress_all_zeros() {
        let deltas = vec![[0.0; 3]; 1000];
        let ct = CompressedTarget::compress(&deltas).expect("should succeed");
        // Should compress very well.
        assert!(
            ct.compression_ratio() < 0.01,
            "ratio = {}",
            ct.compression_ratio()
        );
        let out = ct.decompress().expect("should succeed");
        assert_eq!(out.len(), 1000);
        for d in &out {
            assert_close(*d, [0.0; 3], 1e-6);
        }
    }

    #[test]
    fn compress_sparse() {
        // Only vertex 5 and 50 have non-zero deltas (typical morph target).
        let mut deltas = vec![[0.0; 3]; 100];
        deltas[5] = [0.001, -0.002, 0.003];
        deltas[50] = [-0.005, 0.004, 0.001];

        let ct = CompressedTarget::compress(&deltas).expect("should succeed");
        assert!(ct.compression_ratio() < 0.1);
        let out = ct.decompress().expect("should succeed");
        assert_eq!(out.len(), 100);
        assert_close(out[5], deltas[5], 1e-6);
        assert_close(out[50], deltas[50], 1e-6);
        // Zeros should be exact-ish.
        assert_close(out[0], [0.0; 3], 1e-6);
        assert_close(out[99], [0.0; 3], 1e-6);
    }

    #[test]
    fn lite_pack_round_trip() {
        let mut pack = LitePack::new();
        pack.set_metadata("author".into(), "test".into());

        let deltas_a = vec![[0.0; 3]; 50];
        let mut deltas_b = vec![[0.0; 3]; 50];
        deltas_b[10] = [0.01, 0.02, 0.03];

        pack.add_target("smile".into(), &deltas_a)
            .expect("should succeed");
        pack.add_target("blink".into(), &deltas_b)
            .expect("should succeed");

        let bytes = pack.serialize().expect("should succeed");
        let pack2 = LitePack::deserialize(&bytes).expect("should succeed");

        assert_eq!(pack2.target_names(), vec!["smile", "blink"]);
        assert_eq!(
            pack2.metadata().get("author").map(String::as_str),
            Some("test")
        );

        let out_b = pack2.get_target("blink").expect("should succeed");
        assert_close(out_b[10], [0.01, 0.02, 0.03], 1e-6);
    }

    #[test]
    fn empty_deltas() {
        let ct = CompressedTarget::compress(&[]).expect("should succeed");
        assert_eq!(ct.vertex_count(), 0);
        let out = ct.decompress().expect("should succeed");
        assert!(out.is_empty());
    }

    #[test]
    fn bad_magic_rejected() {
        let data = b"BADMxxxxx";
        assert!(LitePack::deserialize(data).is_err());
    }

    #[test]
    fn lite_pack_empty() {
        let pack = LitePack::new();
        let bytes = pack.serialize().expect("should succeed");
        let pack2 = LitePack::deserialize(&bytes).expect("should succeed");
        assert!(pack2.is_empty());
    }
}
