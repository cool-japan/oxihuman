// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Delta encoder v2: delta-encode and decode integer sequences with zigzag encoding.

#![allow(dead_code)]

/// Zigzag-encode a signed integer to unsigned (for compact storage).
/// Maps: 0->0, -1->1, 1->2, -2->3, 2->4, ...
#[allow(dead_code)]
pub fn zigzag_encode(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// Zigzag-decode an unsigned integer to signed.
#[allow(dead_code)]
pub fn zigzag_decode(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

/// Delta-encode a sequence of i64 values, then zigzag-encode to u64.
#[allow(dead_code)]
pub fn delta_encode(data: &[i64]) -> Vec<u64> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(data.len());
    out.push(zigzag_encode(data[0]));
    for i in 1..data.len() {
        out.push(zigzag_encode(data[i] - data[i - 1]));
    }
    out
}

/// Decode a zigzag+delta-encoded sequence back to i64.
#[allow(dead_code)]
pub fn delta_decode(encoded: &[u64]) -> Vec<i64> {
    if encoded.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(encoded.len());
    out.push(zigzag_decode(encoded[0]));
    for i in 1..encoded.len() {
        out.push(out[i - 1] + zigzag_decode(encoded[i]));
    }
    out
}

/// Compute the maximum delta magnitude in a sequence.
#[allow(dead_code)]
pub fn max_delta(data: &[i64]) -> i64 {
    if data.len() < 2 {
        return 0;
    }
    data.windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .max()
        .unwrap_or(0)
}

/// Compute the average delta magnitude.
#[allow(dead_code)]
pub fn avg_delta(data: &[i64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let sum: i64 = data.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
    sum as f64 / (data.len() - 1) as f64
}

/// Delta-encode a u32 sequence (no zigzag, values are non-decreasing).
#[allow(dead_code)]
pub fn delta_encode_u32(data: &[u32]) -> Vec<u32> {
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

/// Decode a u32 delta-encoded sequence.
#[allow(dead_code)]
pub fn delta_decode_u32(deltas: &[u32]) -> Vec<u32> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zigzag_zero() {
        assert_eq!(zigzag_encode(0), 0);
        assert_eq!(zigzag_decode(0), 0);
    }

    #[test]
    fn zigzag_positive() {
        assert_eq!(zigzag_encode(1), 2);
        assert_eq!(zigzag_decode(2), 1);
    }

    #[test]
    fn zigzag_negative() {
        assert_eq!(zigzag_encode(-1), 1);
        assert_eq!(zigzag_decode(1), -1);
    }

    #[test]
    fn zigzag_roundtrip() {
        for n in [-100i64, -1, 0, 1, 100] {
            assert_eq!(zigzag_decode(zigzag_encode(n)), n);
        }
    }

    #[test]
    fn delta_encode_empty() {
        assert!(delta_encode(&[]).is_empty());
    }

    #[test]
    fn delta_roundtrip() {
        let data = vec![10i64, 12, 15, 10, 20, -5];
        let enc = delta_encode(&data);
        let dec = delta_decode(&enc);
        assert_eq!(dec, data);
    }

    #[test]
    fn delta_encode_single() {
        let data = vec![42i64];
        let enc = delta_encode(&data);
        let dec = delta_decode(&enc);
        assert_eq!(dec, data);
    }

    #[test]
    fn max_delta_basic() {
        let data = vec![0i64, 5, 3, 10];
        assert_eq!(max_delta(&data), 7); // |10-3|=7
    }

    #[test]
    fn avg_delta_basic() {
        let data = vec![0i64, 2, 4, 6];
        let avg = avg_delta(&data);
        assert!((avg - 2.0).abs() < 1e-9);
    }

    #[test]
    fn u32_delta_roundtrip() {
        let data = vec![1u32, 3, 6, 10, 15];
        let enc = delta_encode_u32(&data);
        let dec = delta_decode_u32(&enc);
        assert_eq!(dec, data);
    }
}
