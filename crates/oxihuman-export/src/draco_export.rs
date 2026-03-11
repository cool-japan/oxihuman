// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Draco-style mesh compression export (simplified encoding).

/// Draco export configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DracoExportConfig {
    pub quantization_bits: u32,
    pub compression_level: u32,
}

/// Draco export result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DracoExportResult {
    pub compressed_bytes: Vec<u8>,
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f32,
}

#[allow(dead_code)]
pub fn default_draco_export_config() -> DracoExportConfig {
    DracoExportConfig { quantization_bits: 14, compression_level: 7 }
}

/// Quantize a float value to N bits within a range.
#[allow(dead_code)]
pub fn quantize_value(value: f32, min: f32, max: f32, bits: u32) -> u32 {
    let range = max - min;
    if range < 1e-12 { return 0; }
    let max_val = (1u32 << bits) - 1;
    let normalized = ((value - min) / range).clamp(0.0, 1.0);
    (normalized * max_val as f32) as u32
}

/// Dequantize back to float.
#[allow(dead_code)]
pub fn dequantize_value(quantized: u32, min: f32, max: f32, bits: u32) -> f32 {
    let max_val = (1u32 << bits) - 1;
    if max_val == 0 { return min; }
    let normalized = quantized as f32 / max_val as f32;
    min + normalized * (max - min)
}

/// Quantize positions.
#[allow(dead_code)]
pub fn quantize_positions(positions: &[[f32; 3]], bits: u32) -> (Vec<[u32; 3]>, [f32; 3], [f32; 3]) {
    let (min, max) = position_bounds(positions);
    let quantized: Vec<[u32; 3]> = positions.iter().map(|p| {
        [
            quantize_value(p[0], min[0], max[0], bits),
            quantize_value(p[1], min[1], max[1], bits),
            quantize_value(p[2], min[2], max[2], bits),
        ]
    }).collect();
    (quantized, min, max)
}

/// Delta encode indices for compression.
#[allow(dead_code)]
pub fn delta_encode_indices(indices: &[u32]) -> Vec<i32> {
    if indices.is_empty() { return Vec::new(); }
    let mut result = vec![indices[0] as i32];
    for i in 1..indices.len() {
        result.push(indices[i] as i32 - indices[i - 1] as i32);
    }
    result
}

/// Simple export: quantize + delta encode.
#[allow(dead_code)]
pub fn draco_export_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &DracoExportConfig,
) -> DracoExportResult {
    let _ = config.compression_level;
    let original_size = positions.len() * 12 + indices.len() * 4;
    let (quantized, _min, _max) = quantize_positions(positions, config.quantization_bits);
    let delta_idx = delta_encode_indices(indices);
    // Pack into bytes (simplified)
    let mut bytes = Vec::new();
    for q in &quantized {
        for &v in q {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    for &d in &delta_idx {
        bytes.extend_from_slice(&d.to_le_bytes());
    }
    let compressed_size = bytes.len();
    let ratio = if original_size > 0 { compressed_size as f32 / original_size as f32 } else { 0.0 };
    DracoExportResult { compressed_bytes: bytes, original_size, compressed_size, ratio }
}

#[allow(dead_code)]
pub fn draco_compression_ratio(result: &DracoExportResult) -> f32 {
    result.ratio
}

fn position_bounds(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for p in positions {
        for k in 0..3 {
            if p[k] < min[k] { min[k] = p[k]; }
            if p[k] > max[k] { max[k] = p[k]; }
        }
    }
    if min[0].is_infinite() { min = [0.0; 3]; max = [0.0; 3]; }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_draco_export_config();
        assert_eq!(c.quantization_bits, 14);
    }

    #[test]
    fn test_quantize_value() {
        let q = quantize_value(0.5, 0.0, 1.0, 8);
        assert!(q > 100 && q < 200);
    }

    #[test]
    fn test_dequantize_roundtrip() {
        let q = quantize_value(0.75, 0.0, 1.0, 16);
        let d = dequantize_value(q, 0.0, 1.0, 16);
        assert!((d - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_quantize_positions() {
        let pos = vec![[0.0; 3], [1.0, 2.0, 3.0]];
        let (q, min, max) = quantize_positions(&pos, 10);
        assert_eq!(q.len(), 2);
        assert!((min[0] - 0.0).abs() < 1e-5);
        assert!((max[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_delta_encode() {
        let deltas = delta_encode_indices(&[0, 1, 2, 0, 2, 3]);
        assert_eq!(deltas[0], 0);
        assert_eq!(deltas[1], 1);
    }

    #[test]
    fn test_export_mesh() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let config = default_draco_export_config();
        let result = draco_export_mesh(&pos, &idx, &config);
        assert!(result.compressed_size > 0);
    }

    #[test]
    fn test_compression_ratio() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let config = default_draco_export_config();
        let result = draco_export_mesh(&pos, &[0, 1, 2], &config);
        let ratio = draco_compression_ratio(&result);
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_empty_mesh() {
        let config = default_draco_export_config();
        let result = draco_export_mesh(&[], &[], &config);
        assert_eq!(result.compressed_size, 0);
    }

    #[test]
    fn test_quantize_zero_range() {
        let q = quantize_value(5.0, 5.0, 5.0, 8);
        assert_eq!(q, 0);
    }

    #[test]
    fn test_delta_empty() {
        let deltas = delta_encode_indices(&[]);
        assert!(deltas.is_empty());
    }

}
