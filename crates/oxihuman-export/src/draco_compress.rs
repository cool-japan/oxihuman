// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Draco-like mesh compression (quantized vertex attribute encoding).

// ── Types ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct DracoConfig {
    pub position_quantization: u8,
    pub normal_quantization: u8,
    pub uv_quantization: u8,
    pub use_edgebreaker: bool,
    pub compression_level: u8,
}

#[allow(dead_code)]
pub struct CompressedMesh {
    pub data: Vec<u8>,
    pub original_vertex_count: usize,
    pub original_index_count: usize,
    pub quantization_bits: u8,
}

#[allow(dead_code)]
pub struct DracoQuantizedMesh {
    pub positions: Vec<[i32; 3]>,
    pub normals: Vec<[i32; 3]>,
    pub uvs: Vec<[i32; 2]>,
    pub indices: Vec<u32>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub position_scale: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_draco_config() -> DracoConfig {
    DracoConfig {
        position_quantization: 11,
        normal_quantization: 8,
        uv_quantization: 10,
        use_edgebreaker: true,
        compression_level: 7,
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn dequantize_uvs(quantized: &[[i32; 2]], bits: u8) -> Vec<[f32; 2]> {
    let max_val = ((1i32 << bits) - 1) as f32;
    quantized
        .iter()
        .map(|q| [q[0] as f32 / max_val, q[1] as f32 / max_val])
        .collect()
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
    let (qpos, mn, _mx, scale) = quantize_positions(positions, cfg.position_quantization);
    let qnrm = quantize_normals(normals, cfg.normal_quantization);
    let quvs = quantize_uvs(uvs, cfg.uv_quantization);
    let idx_delta = encode_indices_delta(indices);
    ((qpos, qnrm, quvs, idx_delta), mn, scale)
}

#[allow(dead_code)]
pub fn compress_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
    cfg: &DracoConfig,
) -> CompressedMesh {
    let ((qpos, qnrm, quvs, idx_delta), _mn, _scale) =
        compress_attrs(positions, normals, uvs, indices, cfg);

    // Pack everything into bytes (simple little-endian i32 streams)
    let mut data: Vec<u8> = Vec::new();

    // Header
    data.extend_from_slice(&(positions.len() as u32).to_le_bytes());
    data.extend_from_slice(&(indices.len() as u32).to_le_bytes());
    data.push(cfg.position_quantization);

    // Positions
    for p in &qpos {
        for &v in p {
            data.extend_from_slice(&v.to_le_bytes());
        }
    }
    // Normals
    for n in &qnrm {
        for &v in n {
            data.extend_from_slice(&v.to_le_bytes());
        }
    }
    // UVs
    for uv in &quvs {
        for &v in uv {
            data.extend_from_slice(&v.to_le_bytes());
        }
    }
    // Index deltas
    for &d in &idx_delta {
        data.extend_from_slice(&d.to_le_bytes());
    }

    CompressedMesh {
        data,
        original_vertex_count: positions.len(),
        original_index_count: indices.len(),
        quantization_bits: cfg.position_quantization,
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn compression_ratio(original_bytes: usize, compressed: &CompressedMesh) -> f32 {
    if compressed.data.is_empty() {
        return 1.0;
    }
    original_bytes as f32 / compressed.data.len() as f32
}

#[allow(dead_code)]
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
}
