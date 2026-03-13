// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Delta encoding for sequences of integers and floats.

#![allow(dead_code)]

/// Delta-encode a sequence of i32 values.
/// The first element is kept as-is; subsequent elements store the difference.
#[allow(dead_code)]
pub fn delta_encode_i32(data: &[i32]) -> Vec<i32> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(data.len());
    out.push(data[0]);
    for i in 1..data.len() {
        out.push(data[i].wrapping_sub(data[i - 1]));
    }
    out
}

/// Reconstruct original values from delta-encoded i32 sequence.
#[allow(dead_code)]
pub fn delta_decode_i32(deltas: &[i32]) -> Vec<i32> {
    if deltas.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(deltas.len());
    out.push(deltas[0]);
    for i in 1..deltas.len() {
        out.push(out[i - 1].wrapping_add(deltas[i]));
    }
    out
}

/// Delta-encode a sequence of f32 values by quantizing to i32 with the given scale factor.
/// Each float is multiplied by `scale` and rounded to i32, then delta-encoded.
#[allow(dead_code)]
pub fn delta_encode_f32(data: &[f32], scale: f32) -> Vec<i32> {
    if data.is_empty() {
        return Vec::new();
    }
    let quantized: Vec<i32> = data.iter().map(|&v| (v * scale).round() as i32).collect();
    delta_encode_i32(&quantized)
}

/// Reconstruct original f32 values from delta-encoded i32 sequence.
/// Values are divided by `scale` to recover the approximate original floats.
#[allow(dead_code)]
pub fn delta_decode_f32(deltas: &[i32], scale: f32) -> Vec<f32> {
    let quantized = delta_decode_i32(deltas);
    let inv_scale = if scale.abs() > f32::EPSILON { 1.0 / scale } else { 1.0 };
    quantized.iter().map(|&v| v as f32 * inv_scale).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_empty() {
        assert!(delta_encode_i32(&[]).is_empty());
    }

    #[test]
    fn decode_empty() {
        assert!(delta_decode_i32(&[]).is_empty());
    }

    #[test]
    fn encode_single() {
        assert_eq!(delta_encode_i32(&[42]), vec![42]);
    }

    #[test]
    fn roundtrip_i32_basic() {
        let data = vec![10, 12, 15, 10, 20];
        let enc = delta_encode_i32(&data);
        let dec = delta_decode_i32(&enc);
        assert_eq!(dec, data);
    }

    #[test]
    fn roundtrip_i32_negative() {
        let data = vec![-5, -3, 0, -2, 10];
        let enc = delta_encode_i32(&data);
        let dec = delta_decode_i32(&enc);
        assert_eq!(dec, data);
    }

    #[test]
    fn delta_values_correct() {
        let data = vec![1, 3, 6, 10];
        let enc = delta_encode_i32(&data);
        assert_eq!(enc, vec![1, 2, 3, 4]);
    }

    #[test]
    fn roundtrip_f32_basic() {
        let data = vec![1.0f32, 1.5, 2.0, 2.5, 3.0];
        let scale = 100.0f32;
        let enc = delta_encode_f32(&data, scale);
        let dec = delta_decode_f32(&enc, scale);
        for (a, b) in data.iter().zip(dec.iter()) {
            assert!((a - b).abs() < 0.02, "expected {a} got {b}");
        }
    }

    #[test]
    fn f32_encode_empty() {
        assert!(delta_encode_f32(&[], 100.0).is_empty());
    }

    #[test]
    fn f32_decode_empty() {
        assert!(delta_decode_f32(&[], 100.0).is_empty());
    }
}
