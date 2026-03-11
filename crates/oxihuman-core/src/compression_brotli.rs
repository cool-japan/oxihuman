// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Brotli compression stub.

/// Configuration for the Brotli compressor.
#[derive(Debug, Clone)]
pub struct BrotliConfig {
    pub quality: u8,
    pub window_bits: u8,
}

impl Default for BrotliConfig {
    fn default() -> Self {
        Self {
            quality: 6,
            window_bits: 22,
        }
    }
}

/// Brotli compressor stub.
#[derive(Debug, Clone)]
pub struct BrotliCompressor {
    pub config: BrotliConfig,
}

impl BrotliCompressor {
    pub fn new(config: BrotliConfig) -> Self {
        Self { config }
    }

    pub fn default_compressor() -> Self {
        Self::new(BrotliConfig::default())
    }

    pub fn with_quality(quality: u8) -> Self {
        Self::new(BrotliConfig {
            quality: quality.min(11),
            window_bits: 22,
        })
    }
}

/// Compress bytes with Brotli stub (4-byte LE header + raw payload).
pub fn brotli_compress(data: &[u8]) -> Vec<u8> {
    /* stub: store with header */
    let mut out = Vec::with_capacity(data.len() + 4);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    out
}

/// Decompress bytes produced by [`brotli_compress`].
pub fn brotli_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    /* stub: strip 4-byte header */
    if data.len() < 4 {
        return Err("brotli: input too short".to_string());
    }
    let expected = u32::from_le_bytes(data[..4].try_into().unwrap_or_default()) as usize;
    let payload = &data[4..];
    if payload.len() < expected {
        return Err("brotli: truncated payload".to_string());
    }
    Ok(payload[..expected].to_vec())
}

/// Estimate output size for Brotli compression (stub).
pub fn brotli_max_compressed_size(input_len: usize) -> usize {
    input_len + 10
}

/// Return whether quality is in valid range [0, 11].
pub fn brotli_quality_valid(quality: u8) -> bool {
    quality <= 11
}

/// Verify round-trip integrity.
pub fn brotli_roundtrip_ok(data: &[u8]) -> bool {
    brotli_decompress(&brotli_compress(data))
        .map(|d| d == data)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_quality() {
        /* default quality level */
        assert_eq!(BrotliConfig::default().quality, 6);
    }

    #[test]
    fn test_quality_clamped() {
        /* quality above 11 is clamped */
        let c = BrotliCompressor::with_quality(20);
        assert_eq!(c.config.quality, 11);
    }

    #[test]
    fn test_quality_valid() {
        /* valid range check */
        assert!(brotli_quality_valid(0));
        assert!(brotli_quality_valid(11));
        assert!(!brotli_quality_valid(12));
    }

    #[test]
    fn test_compress_empty() {
        /* empty input gives 4-byte header */
        assert_eq!(brotli_compress(&[]).len(), 4);
    }

    #[test]
    fn test_roundtrip_hello() {
        /* hello round-trip */
        assert!(brotli_roundtrip_ok(b"Hello Brotli!"));
    }

    #[test]
    fn test_roundtrip_binary() {
        /* binary round-trip */
        let data: Vec<u8> = (0u8..=200).collect();
        assert!(brotli_roundtrip_ok(&data));
    }

    #[test]
    fn test_decompress_short() {
        /* error on short input */
        assert!(brotli_decompress(&[]).is_err());
    }

    #[test]
    fn test_max_compressed_size() {
        /* estimate is larger than input */
        assert!(brotli_max_compressed_size(50) > 50);
    }

    #[test]
    fn test_default_window_bits() {
        /* default window bits */
        assert_eq!(BrotliConfig::default().window_bits, 22);
    }
}
