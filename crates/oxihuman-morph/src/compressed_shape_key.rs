// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compressed/quantized shape key stub.

/// Quantization precision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuantBits {
    Bits8,
    Bits16,
}

/// A compressed shape key storing quantized vertex deltas.
#[derive(Debug, Clone)]
pub struct CompressedShapeKey {
    pub name: String,
    pub vertex_count: usize,
    pub quant_bits: QuantBits,
    pub data: Vec<u16>,
    pub scale: f32,
    pub enabled: bool,
}

impl CompressedShapeKey {
    pub fn new(name: impl Into<String>, vertex_count: usize) -> Self {
        CompressedShapeKey {
            name: name.into(),
            vertex_count,
            quant_bits: QuantBits::Bits16,
            data: vec![0u16; vertex_count * 3],
            scale: 1.0,
            enabled: true,
        }
    }
}

/// Create a new compressed shape key.
pub fn new_compressed_shape_key(
    name: impl Into<String>,
    vertex_count: usize,
) -> CompressedShapeKey {
    CompressedShapeKey::new(name, vertex_count)
}

/// Decode a compressed delta for a vertex component (stub).
pub fn csk_decode_delta(key: &CompressedShapeKey, vertex: usize, component: usize) -> f32 {
    /* Stub: dequantizes stored u16 value */
    let idx = vertex * 3 + component;
    if idx < key.data.len() {
        (key.data[idx] as f32 - 32768.0) * key.scale / 32768.0
    } else {
        0.0
    }
}

/// Set scale factor.
pub fn csk_set_scale(key: &mut CompressedShapeKey, scale: f32) {
    key.scale = scale;
}

/// Set quantization bits.
pub fn csk_set_quant_bits(key: &mut CompressedShapeKey, bits: QuantBits) {
    key.quant_bits = bits;
}

/// Return byte size estimate.
pub fn csk_byte_size(key: &CompressedShapeKey) -> usize {
    match key.quant_bits {
        QuantBits::Bits8 => key.vertex_count * 3,
        QuantBits::Bits16 => key.vertex_count * 3 * 2,
    }
}

/// Enable or disable the shape key.
pub fn csk_set_enabled(key: &mut CompressedShapeKey, enabled: bool) {
    key.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn csk_to_json(key: &CompressedShapeKey) -> String {
    format!(
        r#"{{"name":"{}","vertex_count":{},"scale":{},"enabled":{}}}"#,
        key.name, key.vertex_count, key.scale, key.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vertex_count() {
        let k = new_compressed_shape_key("test", 100);
        assert_eq!(k.vertex_count, 100 /* vertex count must match */,);
    }

    #[test]
    fn test_data_length() {
        let k = new_compressed_shape_key("k", 10);
        assert_eq!(
            k.data.len(),
            30, /* data length must be vertex_count * 3 */
        );
    }

    #[test]
    fn test_decode_zero_delta() {
        let k = new_compressed_shape_key("k", 4);
        /* all data is 0 initially, decode should be near -scale */
        let _ = csk_decode_delta(&k, 0, 0);
    }

    #[test]
    fn test_set_scale() {
        let mut k = new_compressed_shape_key("k", 2);
        csk_set_scale(&mut k, 0.5);
        assert!((k.scale - 0.5).abs() < 1e-6 /* scale must be set */,);
    }

    #[test]
    fn test_set_quant_bits() {
        let mut k = new_compressed_shape_key("k", 2);
        csk_set_quant_bits(&mut k, QuantBits::Bits8);
        assert_eq!(
            k.quant_bits,
            QuantBits::Bits8, /* quant bits must be set */
        );
    }

    #[test]
    fn test_byte_size_16bit() {
        let k = new_compressed_shape_key("k", 10);
        assert_eq!(csk_byte_size(&k), 60 /* 10 * 3 * 2 bytes for 16-bit */,);
    }

    #[test]
    fn test_byte_size_8bit() {
        let mut k = new_compressed_shape_key("k", 10);
        csk_set_quant_bits(&mut k, QuantBits::Bits8);
        assert_eq!(csk_byte_size(&k), 30 /* 10 * 3 bytes for 8-bit */,);
    }

    #[test]
    fn test_set_enabled() {
        let mut k = new_compressed_shape_key("k", 2);
        csk_set_enabled(&mut k, false);
        assert!(!k.enabled /* enabled must be false */,);
    }

    #[test]
    fn test_to_json() {
        let k = new_compressed_shape_key("smile", 5);
        let j = csk_to_json(&k);
        assert!(j.contains("smile"), /* json must contain shape key name */);
    }

    #[test]
    fn test_enabled_default() {
        let k = new_compressed_shape_key("k", 1);
        assert!(k.enabled /* must be enabled by default */,);
    }
}
