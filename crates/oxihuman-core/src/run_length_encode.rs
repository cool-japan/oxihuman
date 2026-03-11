#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Run-length encoding for u8 and i32 sequences.

#[allow(dead_code)]
pub fn rle_encode_u8(data: &[u8]) -> Vec<(u8, u8)> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = data[0];
    let mut count: u8 = 1;
    for &byte in &data[1..] {
        if byte == current && count < u8::MAX {
            count += 1;
        } else {
            result.push((current, count));
            current = byte;
            count = 1;
        }
    }
    result.push((current, count));
    result
}

#[allow(dead_code)]
pub fn rle_decode_u8(runs: &[(u8, u8)]) -> Vec<u8> {
    let mut result = Vec::new();
    for &(val, count) in runs {
        for _ in 0..count {
            result.push(val);
        }
    }
    result
}

#[allow(dead_code)]
pub fn rle_encode_i32(data: &[i32]) -> Vec<(i32, u32)> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = data[0];
    let mut count: u32 = 1;
    for &val in &data[1..] {
        if val == current {
            count += 1;
        } else {
            result.push((current, count));
            current = val;
            count = 1;
        }
    }
    result.push((current, count));
    result
}

#[allow(dead_code)]
pub fn rle_decode_i32(runs: &[(i32, u32)]) -> Vec<i32> {
    let mut result = Vec::new();
    for &(val, count) in runs {
        for _ in 0..count {
            result.push(val);
        }
    }
    result
}

#[allow(dead_code)]
pub fn rle_compression_ratio(original_len: usize, encoded_len: usize) -> f32 {
    if encoded_len == 0 {
        return 0.0;
    }
    original_len as f32 / encoded_len as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_u8_basic() {
        let data = vec![1u8, 1, 1, 2, 2, 3];
        let encoded = rle_encode_u8(&data);
        assert_eq!(encoded, vec![(1, 3), (2, 2), (3, 1)]);
    }

    #[test]
    fn decode_u8_roundtrip() {
        let data = vec![10u8, 10, 20, 30, 30, 30];
        let encoded = rle_encode_u8(&data);
        let decoded = rle_decode_u8(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn encode_u8_empty() {
        let encoded = rle_encode_u8(&[]);
        assert!(encoded.is_empty());
    }

    #[test]
    fn encode_u8_all_same() {
        let data = vec![5u8; 50];
        let encoded = rle_encode_u8(&data);
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], (5, 50));
    }

    #[test]
    fn encode_u8_all_different() {
        let data = vec![1u8, 2, 3, 4];
        let encoded = rle_encode_u8(&data);
        assert_eq!(encoded.len(), 4);
    }

    #[test]
    fn encode_i32_basic() {
        let data = vec![-1i32, -1, 0, 0, 0, 1];
        let encoded = rle_encode_i32(&data);
        assert_eq!(encoded, vec![(-1, 2), (0, 3), (1, 1)]);
    }

    #[test]
    fn decode_i32_roundtrip() {
        let data = vec![100i32, 100, -50, -50, -50, 0];
        let encoded = rle_encode_i32(&data);
        let decoded = rle_decode_i32(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn encode_i32_empty() {
        let encoded = rle_encode_i32(&[]);
        assert!(encoded.is_empty());
    }

    #[test]
    fn compression_ratio_basic() {
        let ratio = rle_compression_ratio(100, 10);
        assert!((ratio - 10.0).abs() < 1e-5);
    }

    #[test]
    fn compression_ratio_zero_encoded() {
        let ratio = rle_compression_ratio(100, 0);
        assert_eq!(ratio, 0.0);
    }
}
