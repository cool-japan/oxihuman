// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Space-efficient packed 3D vector storage using u16 components.

#![allow(dead_code)]

/// A packed 3D vector with u16 components for compact storage.
/// Values are encoded in the range [min, max] mapped to [0, u16::MAX].
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedVec3 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

/// Configuration for encoding/decoding packed vectors.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PackedVec3Config {
    pub min: f32,
    pub max: f32,
}

impl Default for PackedVec3Config {
    fn default() -> Self {
        PackedVec3Config {
            min: -1.0,
            max: 1.0,
        }
    }
}

/// Create a default config with range [-1, 1].
#[allow(dead_code)]
pub fn default_packed_config() -> PackedVec3Config {
    PackedVec3Config::default()
}

/// Create a config with custom range.
#[allow(dead_code)]
pub fn packed_config(min: f32, max: f32) -> PackedVec3Config {
    PackedVec3Config { min, max }
}

/// Encode a float value to u16 given a range config.
#[allow(dead_code)]
pub fn encode_f32_to_u16(val: f32, cfg: &PackedVec3Config) -> u16 {
    let range = cfg.max - cfg.min;
    if range < 1e-10 {
        return 0;
    }
    let normalized = ((val - cfg.min) / range).clamp(0.0, 1.0);
    (normalized * u16::MAX as f32).round() as u16
}

/// Decode a u16 value to f32 given a range config.
#[allow(dead_code)]
pub fn decode_u16_to_f32(val: u16, cfg: &PackedVec3Config) -> f32 {
    let normalized = val as f32 / u16::MAX as f32;
    cfg.min + normalized * (cfg.max - cfg.min)
}

/// Pack a [f32; 3] vector into a PackedVec3.
#[allow(dead_code)]
pub fn pack_vec3(v: [f32; 3], cfg: &PackedVec3Config) -> PackedVec3 {
    PackedVec3 {
        x: encode_f32_to_u16(v[0], cfg),
        y: encode_f32_to_u16(v[1], cfg),
        z: encode_f32_to_u16(v[2], cfg),
    }
}

/// Unpack a PackedVec3 into a [f32; 3] vector.
#[allow(dead_code)]
pub fn unpack_vec3(pv: &PackedVec3, cfg: &PackedVec3Config) -> [f32; 3] {
    [
        decode_u16_to_f32(pv.x, cfg),
        decode_u16_to_f32(pv.y, cfg),
        decode_u16_to_f32(pv.z, cfg),
    ]
}

/// A storage buffer for packed vec3 values.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PackedVec3Buffer {
    data: Vec<PackedVec3>,
    config: PackedVec3Config,
}

/// Create a new buffer with the given config.
#[allow(dead_code)]
pub fn new_packed_buffer(cfg: PackedVec3Config) -> PackedVec3Buffer {
    PackedVec3Buffer {
        data: Vec::new(),
        config: cfg,
    }
}

/// Push a float vector to the buffer.
#[allow(dead_code)]
pub fn pvbuf_push(buf: &mut PackedVec3Buffer, v: [f32; 3]) {
    buf.data.push(pack_vec3(v, &buf.config));
}

/// Get a vector from the buffer, decoded.
#[allow(dead_code)]
pub fn pvbuf_get(buf: &PackedVec3Buffer, idx: usize) -> Option<[f32; 3]> {
    buf.data.get(idx).map(|pv| unpack_vec3(pv, &buf.config))
}

/// Number of elements in the buffer.
#[allow(dead_code)]
pub fn pvbuf_len(buf: &PackedVec3Buffer) -> usize {
    buf.data.len()
}

/// Check if buffer is empty.
#[allow(dead_code)]
pub fn pvbuf_is_empty(buf: &PackedVec3Buffer) -> bool {
    buf.data.is_empty()
}

/// Clear the buffer.
#[allow(dead_code)]
pub fn pvbuf_clear(buf: &mut PackedVec3Buffer) {
    buf.data.clear();
}

/// Bytes used by the buffer (3 * 2 bytes per element).
#[allow(dead_code)]
pub fn pvbuf_bytes(buf: &PackedVec3Buffer) -> usize {
    buf.data.len() * 6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let cfg = default_packed_config();
        let val = 0.5f32;
        let encoded = encode_f32_to_u16(val, &cfg);
        let decoded = decode_u16_to_f32(encoded, &cfg);
        assert!((decoded - val).abs() < 0.0001);
    }

    #[test]
    fn test_pack_unpack_zero() {
        let cfg = default_packed_config();
        let v = [0.0f32, 0.0, 0.0];
        let packed = pack_vec3(v, &cfg);
        let unpacked = unpack_vec3(&packed, &cfg);
        assert!((unpacked[0]).abs() < 0.0001);
    }

    #[test]
    fn test_pack_unpack_max() {
        let cfg = default_packed_config();
        let v = [1.0f32, 1.0, 1.0];
        let packed = pack_vec3(v, &cfg);
        let unpacked = unpack_vec3(&packed, &cfg);
        assert!((unpacked[0] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_pack_unpack_min() {
        let cfg = default_packed_config();
        let v = [-1.0f32, -1.0, -1.0];
        let packed = pack_vec3(v, &cfg);
        let unpacked = unpack_vec3(&packed, &cfg);
        assert!((unpacked[0] + 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_buffer_push_and_get() {
        let cfg = default_packed_config();
        let mut buf = new_packed_buffer(cfg);
        pvbuf_push(&mut buf, [0.5, -0.5, 0.0]);
        assert_eq!(pvbuf_len(&buf), 1);
        let got = pvbuf_get(&buf, 0).expect("should succeed");
        assert!((got[0] - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_buffer_is_empty() {
        let cfg = default_packed_config();
        let buf = new_packed_buffer(cfg);
        assert!(pvbuf_is_empty(&buf));
    }

    #[test]
    fn test_buffer_clear() {
        let cfg = default_packed_config();
        let mut buf = new_packed_buffer(cfg);
        pvbuf_push(&mut buf, [1.0, 0.0, 0.0]);
        pvbuf_clear(&mut buf);
        assert!(pvbuf_is_empty(&buf));
    }

    #[test]
    fn test_buffer_bytes() {
        let cfg = default_packed_config();
        let mut buf = new_packed_buffer(cfg);
        pvbuf_push(&mut buf, [0.0, 0.0, 0.0]);
        assert_eq!(pvbuf_bytes(&buf), 6);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let cfg = default_packed_config();
        let buf = new_packed_buffer(cfg);
        assert!(pvbuf_get(&buf, 0).is_none());
    }

    #[test]
    fn test_custom_range() {
        let cfg = packed_config(0.0, 100.0);
        let val = 50.0f32;
        let encoded = encode_f32_to_u16(val, &cfg);
        let decoded = decode_u16_to_f32(encoded, &cfg);
        assert!((decoded - val).abs() < 0.01);
    }
}
