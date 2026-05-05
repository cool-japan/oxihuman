// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Encode data as RLE (run-length encoding): [count, value] pairs.
pub fn compress_rle(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let val = data[i];
        let mut count = 1u8;
        while (i + count as usize) < data.len() && data[i + count as usize] == val && count < 255 {
            count += 1;
        }
        out.push(count);
        out.push(val);
        i += count as usize;
    }
    out
}

/// Decode RLE-encoded data produced by `compress_rle`.
pub fn decompress_rle(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < data.len() {
        let count = data[i] as usize;
        let val = data[i + 1];
        for _ in 0..count {
            out.push(val);
        }
        i += 2;
    }
    out
}

/// Ratio of compressed to original length (< 1.0 means compression helped).
pub fn compression_ratio(original_len: usize, compressed_len: usize) -> f32 {
    if original_len == 0 {
        return 1.0;
    }
    compressed_len as f32 / original_len as f32
}

/// Returns true if RLE compression achieves a ratio below `threshold`.
pub fn is_compressible(data: &[u8], threshold: f32) -> bool {
    let compressed = compress_rle(data);
    compression_ratio(data.len(), compressed.len()) < threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_empty() {
        /* compressing empty slice returns empty */
        assert!(compress_rle(&[]).is_empty());
    }

    #[test]
    fn test_roundtrip_single_run() {
        /* compress then decompress single run */
        let data = vec![7u8; 5];
        let compressed = compress_rle(&data);
        assert_eq!(decompress_rle(&compressed), data);
    }

    #[test]
    fn test_roundtrip_mixed() {
        /* compress then decompress mixed data */
        let data = vec![1, 1, 2, 3, 3, 3];
        let compressed = compress_rle(&data);
        assert_eq!(decompress_rle(&compressed), data);
    }

    #[test]
    fn test_compress_reduces_size() {
        /* repeated data compresses well */
        let data = vec![0u8; 100];
        let compressed = compress_rle(&data);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compression_ratio() {
        /* ratio calculation is correct */
        let ratio = compression_ratio(100, 50);
        assert!((ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_is_compressible() {
        /* highly repetitive data is compressible */
        let data = vec![42u8; 200];
        assert!(is_compressible(&data, 0.5));
    }
}
