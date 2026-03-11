#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple LZ-style byte compression.

/// LZ compressor holding internal state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LzCompressor {
    window_size: usize,
    min_match: usize,
}

#[allow(dead_code)]
pub fn compress_bytes(data: &[u8]) -> Vec<u8> {
    // Simple run-length encoding as a lightweight compression stand-in.
    let mut out = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let val = data[i];
        let mut run = 1u8;
        while i + (run as usize) < data.len()
            && data[i + (run as usize)] == val
            && run < 255
        {
            run += 1;
        }
        out.push(run);
        out.push(val);
        i += run as usize;
    }
    out
}

#[allow(dead_code)]
pub fn decompress_bytes(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < data.len() {
        let run = data[i] as usize;
        let val = data[i + 1];
        for _ in 0..run {
            out.push(val);
        }
        i += 2;
    }
    out
}

#[allow(dead_code)]
pub fn compressed_size(data: &[u8]) -> usize {
    compress_bytes(data).len()
}

#[allow(dead_code)]
pub fn compression_ratio(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 1.0;
    }
    let c = compressed_size(data);
    c as f64 / data.len() as f64
}

#[allow(dead_code)]
pub fn is_compressed(data: &[u8]) -> bool {
    // Heuristic: valid compressed data has even length (pairs of run+val).
    data.len().is_multiple_of(2) && !data.is_empty()
}

#[allow(dead_code)]
pub fn compress_to_vec(data: &[u8]) -> Vec<u8> {
    compress_bytes(data)
}

#[allow(dead_code)]
pub fn decompress_to_vec(data: &[u8]) -> Vec<u8> {
    decompress_bytes(data)
}

#[allow(dead_code)]
pub fn compressor_name() -> &'static str {
    "lz-rle"
}

impl LzCompressor {
    #[allow(dead_code)]
    pub fn new(window_size: usize, min_match: usize) -> Self {
        Self { window_size, min_match }
    }

    #[allow(dead_code)]
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        let _ = self.window_size;
        let _ = self.min_match;
        compress_bytes(data)
    }

    #[allow(dead_code)]
    pub fn decompress(&self, data: &[u8]) -> Vec<u8> {
        decompress_bytes(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_simple() {
        let data = b"aaabbbccc";
        let c = compress_bytes(data);
        let d = decompress_bytes(&c);
        assert_eq!(d, data);
    }

    #[test]
    fn test_empty() {
        let c = compress_bytes(b"");
        assert!(c.is_empty());
        let d = decompress_bytes(&c);
        assert!(d.is_empty());
    }

    #[test]
    fn test_single_byte() {
        let data = b"x";
        let c = compress_bytes(data);
        let d = decompress_bytes(&c);
        assert_eq!(d, data);
    }

    #[test]
    fn test_compressed_size() {
        let data = b"aaaa";
        let s = compressed_size(data);
        assert_eq!(s, 2); // one run pair
    }

    #[test]
    fn test_compression_ratio_empty() {
        let r = compression_ratio(b"");
        assert!((r - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_is_compressed() {
        assert!(is_compressed(&[3, b'a']));
        assert!(!is_compressed(&[]));
    }

    #[test]
    fn test_compress_to_vec() {
        let data = b"bb";
        let v = compress_to_vec(data);
        assert_eq!(v, vec![2, b'b']);
    }

    #[test]
    fn test_decompress_to_vec() {
        let v = decompress_to_vec(&[2, b'c']);
        assert_eq!(v, b"cc");
    }

    #[test]
    fn test_compressor_name() {
        assert_eq!(compressor_name(), "lz-rle");
    }

    #[test]
    fn test_struct_roundtrip() {
        let lz = LzCompressor::new(256, 3);
        let data = b"hello hello";
        let c = lz.compress(data);
        let d = lz.decompress(&c);
        assert_eq!(d, data);
    }
}
