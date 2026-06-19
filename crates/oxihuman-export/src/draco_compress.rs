// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Draco-like mesh compression (quantized vertex attribute encoding).

#![allow(dead_code)]

use oxihuman_core::{huffman_decode, huffman_encode, HuffmanCodeTable};

// ── Types ─────────────────────────────────────────────────────────────────────

pub struct DracoConfig {
    pub position_quantization: u8,
    pub normal_quantization: u8,
    pub uv_quantization: u8,
    pub use_edgebreaker: bool,
    pub compression_level: u8,
}

pub struct CompressedMesh {
    pub data: Vec<u8>,
    pub original_vertex_count: usize,
    pub original_index_count: usize,
    pub quantization_bits: u8,
}

pub struct DracoQuantizedMesh {
    pub positions: Vec<[i32; 3]>,
    pub normals: Vec<[i32; 3]>,
    pub uvs: Vec<[i32; 2]>,
    pub indices: Vec<u32>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub position_scale: f32,
}

// ── Quantization level mapping ────────────────────────────────────────────────

/// Map compression_level (0..=10) to quantize_bits for position data.
///
/// Higher compression_level means fewer quantization bits, producing smaller
/// output at the cost of positional precision:
///
/// - level 0    : 14 bits  (most precise, raw / no entropy coding)
/// - level 1-3  : 12 bits
/// - level 4-7  : 10 bits
/// - level 8-10 :  8 bits  (highest compression)
fn quantize_bits_for_level(compression_level: u8) -> u8 {
    match compression_level {
        0 => 14,
        1..=3 => 12,
        4..=7 => 10,
        _ => 8,
    }
}

// ── Entropy coding (Huffman) ──────────────────────────────────────────────────

/// Wire format for entropy-coded sections.
///
/// Layout when flag == 0x00 (raw):
///   `[flag: u8 = 0x00][payload bytes...]`
///
/// Layout when flag == 0x01 (Huffman):
///   `[flag: u8 = 0x01]`
///   `[num_symbols: u16 LE]`    - number of distinct symbols in the table
///   For each symbol (num_symbols entries):
///     `[symbol: u8]`           - byte value
///     `[code_len: u8]`         - canonical code length (1..=15)
///   `[symbol_count: u32 LE]`   - total original symbol count (for decoder)
///   `[bit_count: u64 LE]`      - valid bit count in the packed stream
///   `[payload_len: u32 LE]`    - byte length of the packed bit stream
///   `[payload: payload_len]`   - packed Huffman bit stream
pub fn draco_entropy_encode(data: &[u8], level: u8) -> Vec<u8> {
    if level == 0 || data.is_empty() {
        // Raw pass-through: flag=0 + original bytes
        let mut out = Vec::with_capacity(1 + data.len());
        out.push(0u8);
        out.extend_from_slice(data);
        return out;
    }

    // Build Huffman code table from the data.
    let table = match HuffmanCodeTable::from_data(data) {
        Some(t) => t,
        None => {
            // Fallback to raw if we can't build a table (should not happen for
            // non-empty data, but be defensive).
            let mut out = Vec::with_capacity(1 + data.len());
            out.push(0u8);
            out.extend_from_slice(data);
            return out;
        }
    };

    let (packed, bit_count) = match huffman_encode(data, &table) {
        Ok(r) => r,
        Err(_) => {
            // Fallback to raw on encoding error.
            let mut out = Vec::with_capacity(1 + data.len());
            out.push(0u8);
            out.extend_from_slice(data);
            return out;
        }
    };

    // Collect active symbols and their code lengths for the header.
    let active_syms: Vec<(u8, u8)> = table
        .codes
        .iter()
        .enumerate()
        .filter(|(_, &(_, len))| len > 0)
        .map(|(sym, &(_, len))| (sym as u8, len))
        .collect();

    let num_symbols = active_syms.len() as u16;
    let symbol_count = data.len() as u32;

    // Total capacity estimate: flag(1) + num_sym(2) + sym_entries(2*n) +
    // symbol_count(4) + bit_count(8) + payload_len(4) + payload
    let header_len = 1 + 2 + (2 * active_syms.len()) + 4 + 8 + 4;
    let mut out = Vec::with_capacity(header_len + packed.len());

    // flag = 1: Huffman-encoded
    out.push(1u8);

    // num_symbols (u16 LE)
    out.extend_from_slice(&num_symbols.to_le_bytes());

    // Symbol table entries: (symbol: u8, code_len: u8)
    for (sym, code_len) in &active_syms {
        out.push(*sym);
        out.push(*code_len);
    }

    // symbol_count (u32 LE) — needed by decoder to know how many symbols to decode
    out.extend_from_slice(&symbol_count.to_le_bytes());

    // bit_count (u64 LE) — valid bit count in the packed stream
    out.extend_from_slice(&(bit_count as u64).to_le_bytes());

    // payload_len (u32 LE)
    out.extend_from_slice(&(packed.len() as u32).to_le_bytes());

    // packed Huffman stream
    out.extend_from_slice(&packed);

    out
}

/// Decode a buffer produced by [`draco_entropy_encode`].
///
/// Returns the original byte sequence or an error string.
pub fn draco_entropy_decode(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Err("draco_entropy_decode: empty input".to_string());
    }

    let flag = data[0];
    match flag {
        0 => {
            // Raw: everything after the flag byte is the payload.
            Ok(data[1..].to_vec())
        }
        1 => {
            // Huffman-encoded: parse header.
            let mut cursor = 1usize;

            let read_u16 = |buf: &[u8], pos: &mut usize| -> Result<u16, String> {
                if *pos + 2 > buf.len() {
                    return Err("draco_entropy_decode: truncated num_symbols".to_string());
                }
                let v = u16::from_le_bytes([buf[*pos], buf[*pos + 1]]);
                *pos += 2;
                Ok(v)
            };

            let read_u32 = |buf: &[u8], pos: &mut usize| -> Result<u32, String> {
                if *pos + 4 > buf.len() {
                    return Err("draco_entropy_decode: truncated u32 field".to_string());
                }
                let v = u32::from_le_bytes([buf[*pos], buf[*pos + 1], buf[*pos + 2], buf[*pos + 3]]);
                *pos += 4;
                Ok(v)
            };

            let read_u64 = |buf: &[u8], pos: &mut usize| -> Result<u64, String> {
                if *pos + 8 > buf.len() {
                    return Err("draco_entropy_decode: truncated u64 field".to_string());
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&buf[*pos..*pos + 8]);
                *pos += 8;
                Ok(u64::from_le_bytes(bytes))
            };

            let num_symbols = read_u16(data, &mut cursor)? as usize;

            // Read code lengths to reconstruct the canonical table.
            // We store them as a [u8; 256] lengths array.
            let mut lengths = [0u8; 256];
            for _ in 0..num_symbols {
                if cursor + 2 > data.len() {
                    return Err("draco_entropy_decode: truncated symbol entry".to_string());
                }
                let sym = data[cursor] as usize;
                let code_len = data[cursor + 1];
                cursor += 2;
                lengths[sym] = code_len;
            }

            let symbol_count = read_u32(data, &mut cursor)? as usize;
            let bit_count = read_u64(data, &mut cursor)? as usize;
            let payload_len = read_u32(data, &mut cursor)? as usize;

            if cursor + payload_len > data.len() {
                return Err("draco_entropy_decode: truncated payload".to_string());
            }
            let payload = &data[cursor..cursor + payload_len];

            // Reconstruct the canonical Huffman table from the serialised lengths.
            let table = HuffmanCodeTable::from_lengths(&lengths);

            huffman_decode(payload, bit_count, symbol_count, &table)
                .map_err(|e| format!("draco_entropy_decode: huffman error: {e}"))
        }
        other => Err(format!("draco_entropy_decode: unknown flag byte {other:#04x}")),
    }
}

// ── Functions ─────────────────────────────────────────────────────────────────

pub fn default_draco_config() -> DracoConfig {
    DracoConfig {
        position_quantization: 11,
        normal_quantization: 8,
        uv_quantization: 10,
        use_edgebreaker: true,
        compression_level: 7,
    }
}

pub fn quantize_positions(
    positions: &[[f32; 3]],
    bits: u8,
) -> (Vec<[i32; 3]>, [f32; 3], [f32; 3], f32) {
    if positions.is_empty() {
        return (Vec::new(), [0.0; 3], [0.0; 3], 1.0);
    }

    let mut mn = positions[0];
    let mut mx = positions[0];
    for p in positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }

    let range = (0..3)
        .map(|k| mx[k] - mn[k])
        .fold(0.0f32, f32::max)
        .max(1e-9);

    let max_val = ((1i32 << bits) - 1) as f32;
    let scale = range / max_val;

    let quantized = positions
        .iter()
        .map(|p| {
            [
                ((p[0] - mn[0]) / scale).round() as i32,
                ((p[1] - mn[1]) / scale).round() as i32,
                ((p[2] - mn[2]) / scale).round() as i32,
            ]
        })
        .collect();

    (quantized, mn, mx, scale)
}

pub fn dequantize_positions(quantized: &[[i32; 3]], min: [f32; 3], scale: f32) -> Vec<[f32; 3]> {
    quantized
        .iter()
        .map(|q| {
            [
                min[0] + q[0] as f32 * scale,
                min[1] + q[1] as f32 * scale,
                min[2] + q[2] as f32 * scale,
            ]
        })
        .collect()
}

pub fn quantize_normals(normals: &[[f32; 3]], bits: u8) -> Vec<[i32; 3]> {
    let max_val = ((1i32 << bits) - 1) as f32;
    let half = max_val / 2.0;
    normals
        .iter()
        .map(|n| {
            [
                ((n[0] * half) + half).round() as i32,
                ((n[1] * half) + half).round() as i32,
                ((n[2] * half) + half).round() as i32,
            ]
        })
        .collect()
}

pub fn dequantize_normals(quantized: &[[i32; 3]], bits: u8) -> Vec<[f32; 3]> {
    let max_val = ((1i32 << bits) - 1) as f32;
    let half = max_val / 2.0;
    quantized
        .iter()
        .map(|q| {
            [
                (q[0] as f32 - half) / half,
                (q[1] as f32 - half) / half,
                (q[2] as f32 - half) / half,
            ]
        })
        .collect()
}

pub fn quantize_uvs(uvs: &[[f32; 2]], bits: u8) -> Vec<[i32; 2]> {
    let max_val = ((1i32 << bits) - 1) as f32;
    uvs.iter()
        .map(|uv| {
            [
                (uv[0].clamp(0.0, 1.0) * max_val).round() as i32,
                (uv[1].clamp(0.0, 1.0) * max_val).round() as i32,
            ]
        })
        .collect()
}

pub fn dequantize_uvs(quantized: &[[i32; 2]], bits: u8) -> Vec<[f32; 2]> {
    let max_val = ((1i32 << bits) - 1) as f32;
    quantized
        .iter()
        .map(|q| [q[0] as f32 / max_val, q[1] as f32 / max_val])
        .collect()
}

pub fn encode_indices_delta(indices: &[u32]) -> Vec<i32> {
    let mut out = Vec::with_capacity(indices.len());
    let mut prev = 0i32;
    for &idx in indices {
        let val = idx as i32;
        out.push(val - prev);
        prev = val;
    }
    out
}

pub fn decode_indices_delta(deltas: &[i32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(deltas.len());
    let mut acc = 0i32;
    for &d in deltas {
        acc += d;
        out.push(acc as u32);
    }
    out
}

/// Type alias to avoid type_complexity clippy warning.
type CompressResult = (Vec<[i32; 3]>, Vec<[i32; 3]>, Vec<[i32; 2]>, Vec<i32>);

fn compress_attrs(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
    cfg: &DracoConfig,
) -> (CompressResult, [f32; 3], f32) {
    // Use the compression-level-adjusted quantization bits for positions.
    let pos_bits = quantize_bits_for_level(cfg.compression_level);
    let (qpos, mn, _mx, scale) = quantize_positions(positions, pos_bits);
    let qnrm = quantize_normals(normals, cfg.normal_quantization);
    let quvs = quantize_uvs(uvs, cfg.uv_quantization);
    let idx_delta = encode_indices_delta(indices);
    ((qpos, qnrm, quvs, idx_delta), mn, scale)
}

pub fn compress_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
    cfg: &DracoConfig,
) -> CompressedMesh {
    let pos_bits = quantize_bits_for_level(cfg.compression_level);
    let ((qpos, qnrm, quvs, idx_delta), _mn, _scale) =
        compress_attrs(positions, normals, uvs, indices, cfg);

    // Pack attribute bytes (simple little-endian i32 streams).
    let mut attr_bytes: Vec<u8> = Vec::new();

    // Positions
    for p in &qpos {
        for &v in p {
            attr_bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    // Normals
    for n in &qnrm {
        for &v in n {
            attr_bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    // UVs
    for uv in &quvs {
        for &v in uv {
            attr_bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    // Index deltas
    for &d in &idx_delta {
        attr_bytes.extend_from_slice(&d.to_le_bytes());
    }

    // Apply entropy coding gated on compression_level.
    let encoded_attrs = draco_entropy_encode(&attr_bytes, cfg.compression_level);

    // Assemble the final compressed payload.
    let mut data: Vec<u8> = Vec::with_capacity(9 + encoded_attrs.len());

    // Header: vertex_count(4) + index_count(4) + quantization_bits(1)
    data.extend_from_slice(&(positions.len() as u32).to_le_bytes());
    data.extend_from_slice(&(indices.len() as u32).to_le_bytes());
    data.push(pos_bits);

    // Entropy-coded attribute block
    data.extend_from_slice(&encoded_attrs);

    CompressedMesh {
        data,
        original_vertex_count: positions.len(),
        original_index_count: indices.len(),
        quantization_bits: pos_bits,
    }
}

/// Decompress a mesh produced by [`compress_mesh`].
///
/// Returns the raw attribute bytes (positions / normals / UVs / index deltas
/// packed as little-endian i32 streams) along with the header fields, or an
/// error string describing the failure.
pub fn decompress_mesh_bytes(
    compressed: &CompressedMesh,
) -> Result<(usize, usize, u8, Vec<u8>), String> {
    let d = &compressed.data;
    if d.len() < 9 {
        return Err("decompress_mesh_bytes: data too short for header".to_string());
    }
    let vertex_count = u32::from_le_bytes([d[0], d[1], d[2], d[3]]) as usize;
    let index_count = u32::from_le_bytes([d[4], d[5], d[6], d[7]]) as usize;
    let quantization_bits = d[8];

    let attr_bytes = draco_entropy_decode(&d[9..])?;
    Ok((vertex_count, index_count, quantization_bits, attr_bytes))
}

pub fn estimate_compressed_size(
    vertex_count: usize,
    index_count: usize,
    cfg: &DracoConfig,
) -> usize {
    let pos_bits = (cfg.position_quantization as usize) * 3 * vertex_count;
    let nrm_bits = (cfg.normal_quantization as usize) * 3 * vertex_count;
    let uv_bits = (cfg.uv_quantization as usize) * 2 * vertex_count;
    let idx_bits = 32 * index_count; // delta, worst case same as original
    (pos_bits + nrm_bits + uv_bits + idx_bits) / 8 + 16
}

pub fn compression_ratio(original_bytes: usize, compressed: &CompressedMesh) -> f32 {
    if compressed.data.is_empty() {
        return 1.0;
    }
    original_bytes as f32 / compressed.data.len() as f32
}

pub fn quantize_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
    bits: u8,
) -> DracoQuantizedMesh {
    let (qpos, mn, mx, scale) = quantize_positions(positions, bits);
    let qnrm = quantize_normals(normals, bits);
    let quvs = quantize_uvs(uvs, bits);
    DracoQuantizedMesh {
        positions: qpos,
        normals: qnrm,
        uvs: quvs,
        indices: indices.to_vec(),
        bounds_min: mn,
        bounds_max: mx,
        position_scale: scale,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
        ]
    }

    fn sample_normals() -> Vec<[f32; 3]> {
        vec![
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ]
    }

    fn sample_uvs() -> Vec<[f32; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]
    }

    fn sample_indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }

    #[test]
    fn test_quantize_positions_count() {
        let pos = sample_positions();
        let (qpos, _mn, _mx, _scale) = quantize_positions(&pos, 11);
        assert_eq!(qpos.len(), pos.len());
    }

    #[test]
    fn test_dequantize_positions_roundtrip() {
        let pos = sample_positions();
        let (qpos, mn, _mx, scale) = quantize_positions(&pos, 11);
        let restored = dequantize_positions(&qpos, mn, scale);
        for (orig, rest) in pos.iter().zip(restored.iter()) {
            for k in 0..3 {
                assert!(
                    (orig[k] - rest[k]).abs() < 0.01,
                    "pos roundtrip failed at k={}",
                    k
                );
            }
        }
    }

    #[test]
    fn test_quantize_positions_empty() {
        let (qpos, mn, mx, scale) = quantize_positions(&[], 11);
        assert!(qpos.is_empty());
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
        assert!((scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_quantize_normals_count() {
        let nrm = sample_normals();
        let qnrm = quantize_normals(&nrm, 8);
        assert_eq!(qnrm.len(), nrm.len());
    }

    #[test]
    fn test_dequantize_normals_roundtrip() {
        let nrm = vec![[0.0f32, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let q = quantize_normals(&nrm, 10);
        let r = dequantize_normals(&q, 10);
        for (orig, rest) in nrm.iter().zip(r.iter()) {
            for k in 0..3 {
                assert!((orig[k] - rest[k]).abs() < 0.02, "normal roundtrip failed");
            }
        }
    }

    #[test]
    fn test_quantize_uvs_count() {
        let uvs = sample_uvs();
        let quvs = quantize_uvs(&uvs, 10);
        assert_eq!(quvs.len(), uvs.len());
    }

    #[test]
    fn test_dequantize_uvs_roundtrip() {
        let uvs = vec![[0.0f32, 0.0], [1.0, 1.0], [0.5, 0.25]];
        let q = quantize_uvs(&uvs, 10);
        let r = dequantize_uvs(&q, 10);
        for (orig, rest) in uvs.iter().zip(r.iter()) {
            for k in 0..2 {
                assert!((orig[k] - rest[k]).abs() < 0.002, "uv roundtrip failed");
            }
        }
    }

    #[test]
    fn test_encode_decode_indices_delta() {
        let indices = sample_indices();
        let deltas = encode_indices_delta(&indices);
        let restored = decode_indices_delta(&deltas);
        assert_eq!(restored, indices);
    }

    #[test]
    fn test_index_delta_empty() {
        let d = encode_indices_delta(&[]);
        assert!(d.is_empty());
        let r = decode_indices_delta(&[]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_compress_mesh_nonempty() {
        let pos = sample_positions();
        let nrm = sample_normals();
        let uvs = sample_uvs();
        let idx = sample_indices();
        let cfg = default_draco_config();
        let compressed = compress_mesh(&pos, &nrm, &uvs, &idx, &cfg);
        assert!(!compressed.data.is_empty());
        assert_eq!(compressed.original_vertex_count, pos.len());
        assert_eq!(compressed.original_index_count, idx.len());
    }

    #[test]
    fn test_compression_ratio_gt_zero() {
        let pos = sample_positions();
        let nrm = sample_normals();
        let uvs = sample_uvs();
        let idx = sample_indices();
        let cfg = default_draco_config();
        let compressed = compress_mesh(&pos, &nrm, &uvs, &idx, &cfg);
        let original_bytes = pos.len() * 12 + nrm.len() * 12 + uvs.len() * 8 + idx.len() * 4;
        let ratio = compression_ratio(original_bytes, &compressed);
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_estimate_compressed_size() {
        let cfg = default_draco_config();
        let sz = estimate_compressed_size(100, 300, &cfg);
        assert!(sz > 0);
    }

    #[test]
    fn test_quantize_mesh_struct() {
        let pos = sample_positions();
        let nrm = sample_normals();
        let uvs = sample_uvs();
        let idx = sample_indices();
        let qm = quantize_mesh(&pos, &nrm, &uvs, &idx, 11);
        assert_eq!(qm.positions.len(), pos.len());
        assert_eq!(qm.normals.len(), nrm.len());
        assert_eq!(qm.uvs.len(), uvs.len());
        assert_eq!(qm.indices, idx);
    }

    #[test]
    fn test_default_draco_config() {
        let cfg = default_draco_config();
        assert_eq!(cfg.position_quantization, 11);
        assert_eq!(cfg.normal_quantization, 8);
        assert_eq!(cfg.uv_quantization, 10);
        assert!(cfg.use_edgebreaker);
    }

    // ── Entropy coding tests ──────────────────────────────────────────────────

    #[test]
    fn test_entropy_raw_roundtrip() {
        // level=0 → raw pass-through
        let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
        let encoded = draco_entropy_encode(&data, 0);
        assert_eq!(encoded[0], 0u8, "flag must be 0 for raw");
        let decoded = draco_entropy_decode(&encoded).expect("raw decode should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_entropy_huffman_roundtrip_level1() {
        // A realistic byte stream with varied byte values.
        let data: Vec<u8> = (0u8..=127).collect();
        let encoded = draco_entropy_encode(&data, 1);
        let decoded = draco_entropy_decode(&encoded).expect("huffman decode should succeed");
        assert_eq!(decoded, data, "round-trip must reconstruct original data");
    }

    #[test]
    fn test_entropy_huffman_roundtrip_level10() {
        // Repeated bytes — good for Huffman compression.
        let data: Vec<u8> = std::iter::repeat_n(42u8, 200).chain(
            std::iter::repeat_n(7u8, 50)
        ).collect();
        let encoded = draco_entropy_encode(&data, 10);
        assert_eq!(encoded[0], 1u8, "flag must be 1 for Huffman");
        let decoded = draco_entropy_decode(&encoded).expect("level-10 decode should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_entropy_empty_raw_fallback() {
        // Empty input always falls through as raw (even with level > 0).
        let encoded = draco_entropy_encode(&[], 5);
        assert_eq!(encoded[0], 0u8);
        let decoded = draco_entropy_decode(&encoded).expect("empty raw decode ok");
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_entropy_decode_unknown_flag_errors() {
        let bad = vec![0xFFu8, 1, 2, 3];
        assert!(draco_entropy_decode(&bad).is_err());
    }

    // ── compression_level wires quantize_bits ──────────────────────────────────

    #[test]
    fn test_quantize_bits_mapping() {
        assert_eq!(quantize_bits_for_level(0), 14);
        assert_eq!(quantize_bits_for_level(1), 12);
        assert_eq!(quantize_bits_for_level(3), 12);
        assert_eq!(quantize_bits_for_level(4), 10);
        assert_eq!(quantize_bits_for_level(7), 10);
        assert_eq!(quantize_bits_for_level(8), 8);
        assert_eq!(quantize_bits_for_level(10), 8);
    }

    #[test]
    fn test_compress_level0_uses_14bits() {
        let pos = sample_positions();
        let nrm = sample_normals();
        let uvs = sample_uvs();
        let idx = sample_indices();
        let mut cfg = default_draco_config();
        cfg.compression_level = 0;
        let compressed = compress_mesh(&pos, &nrm, &uvs, &idx, &cfg);
        // quantization_bits in output should reflect level-mapped bits.
        assert_eq!(compressed.quantization_bits, 14);
    }

    #[test]
    fn test_compress_level10_uses_8bits() {
        let pos = sample_positions();
        let nrm = sample_normals();
        let uvs = sample_uvs();
        let idx = sample_indices();
        let mut cfg = default_draco_config();
        cfg.compression_level = 10;
        let compressed = compress_mesh(&pos, &nrm, &uvs, &idx, &cfg);
        assert_eq!(compressed.quantization_bits, 8);
    }

    #[test]
    fn test_compress_higher_level_not_larger_than_lower() {
        // A large mesh gives Huffman a chance to actually compress.
        let pos: Vec<[f32; 3]> = (0..64)
            .map(|i| [i as f32 * 0.01, (i % 8) as f32 * 0.1, 0.0])
            .collect();
        let nrm: Vec<[f32; 3]> = pos.iter().map(|_| [0.0, 1.0, 0.0]).collect();
        let uvs: Vec<[f32; 2]> = pos
            .iter()
            .enumerate()
            .map(|(i, _)| [(i % 8) as f32 / 8.0, (i / 8) as f32 / 8.0])
            .collect();
        let idx: Vec<u32> = (0..60u32).collect();

        let mut cfg_low = default_draco_config();
        cfg_low.compression_level = 0;
        let mut cfg_high = default_draco_config();
        cfg_high.compression_level = 10;

        let c_low = compress_mesh(&pos, &nrm, &uvs, &idx, &cfg_low);
        let c_high = compress_mesh(&pos, &nrm, &uvs, &idx, &cfg_high);

        // Both must produce non-empty, valid output.
        assert!(!c_low.data.is_empty());
        assert!(!c_high.data.is_empty());

        // Higher compression level must produce an output that is strictly
        // smaller than the level-0 (raw + 14-bit quantization) output.
        assert!(
            c_high.data.len() < c_low.data.len(),
            "level-10 ({} bytes) should be smaller than level-0 ({} bytes)",
            c_high.data.len(),
            c_low.data.len()
        );
    }

    #[test]
    fn test_decompress_mesh_bytes_roundtrip() {
        let pos = sample_positions();
        let nrm = sample_normals();
        let uvs = sample_uvs();
        let idx = sample_indices();
        let mut cfg = default_draco_config();
        cfg.compression_level = 5;
        let compressed = compress_mesh(&pos, &nrm, &uvs, &idx, &cfg);

        let (vc, ic, qbits, attr_bytes) =
            decompress_mesh_bytes(&compressed).expect("decompress should succeed");
        assert_eq!(vc, pos.len());
        assert_eq!(ic, idx.len());
        assert_eq!(qbits, quantize_bits_for_level(5));
        // attr_bytes must be non-empty (positions + normals + uvs + index deltas).
        assert!(!attr_bytes.is_empty());
    }
}
