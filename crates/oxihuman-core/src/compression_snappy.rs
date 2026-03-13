// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Snappy compression stub.

/// Configuration for the Snappy compressor.
#[derive(Debug, Clone, Default)]
pub struct SnappyConfig {
    pub verify_checksum: bool,
}

/// Snappy compressor stub.
#[derive(Debug, Clone)]
pub struct SnappyCompressor {
    pub config: SnappyConfig,
}

impl SnappyCompressor {
    pub fn new(config: SnappyConfig) -> Self {
        Self { config }
    }

    pub fn default_compressor() -> Self {
        Self::new(SnappyConfig {
            verify_checksum: false,
        })
    }
}

/// Compress bytes using Snappy stub (4-byte LE length + raw payload).
pub fn snappy_compress(data: &[u8]) -> Vec<u8> {
    /* stub: store data with 4-byte length header */
    let mut out = Vec::with_capacity(data.len() + 4);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    out
}

/// Decompress bytes produced by [`snappy_compress`].
pub fn snappy_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    /* stub: read 4-byte header then extract payload */
    if data.len() < 4 {
        return Err("snappy: input too short".to_string());
    }
    let expected = u32::from_le_bytes(data[..4].try_into().unwrap_or_default()) as usize;
    let payload = &data[4..];
    if payload.len() < expected {
        return Err("snappy: truncated payload".to_string());
    }
    Ok(payload[..expected].to_vec())
}

/// Return the maximum output size for a given input length (stub).
pub fn snappy_max_compressed_length(input_len: usize) -> usize {
    32 + input_len + input_len / 6
}

/// Return whether a byte slice could be a valid Snappy stub frame.
pub fn snappy_validate_compressed_buffer(data: &[u8]) -> bool {
    data.len() >= 4
}

/// Verify round-trip integrity.
pub fn snappy_roundtrip_ok(data: &[u8]) -> bool {
    snappy_decompress(&snappy_compress(data))
        .map(|d| d == data)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        /* checksum disabled by default */
        let c = SnappyCompressor::default_compressor();
        assert!(!c.config.verify_checksum);
    }

    #[test]
    fn test_compress_empty() {
        /* empty input yields 4-byte header only */
        assert_eq!(snappy_compress(&[]).len(), 4);
    }

    #[test]
    fn test_roundtrip_ascii() {
        /* ascii round-trip */
        assert!(snappy_roundtrip_ok(b"snappy test string"));
    }

    #[test]
    fn test_roundtrip_binary() {
        /* binary data round-trip */
        let data: Vec<u8> = (0u8..=255).collect();
        assert!(snappy_roundtrip_ok(&data));
    }

    #[test]
    fn test_decompress_too_short() {
        /* error on < 4 bytes */
        assert!(snappy_decompress(&[0, 1]).is_err());
    }

    #[test]
    fn test_max_compressed_length() {
        /* max length is larger than input */
        assert!(snappy_max_compressed_length(100) > 100);
    }

    #[test]
    fn test_validate_valid() {
        /* 4-byte slice validates */
        assert!(snappy_validate_compressed_buffer(&[0, 0, 0, 0]));
    }

    #[test]
    fn test_validate_too_short() {
        /* 3-byte slice fails */
        assert!(!snappy_validate_compressed_buffer(&[0, 0, 0]));
    }

    #[test]
    fn test_new_config() {
        /* verify_checksum can be toggled */
        let c = SnappyCompressor::new(SnappyConfig {
            verify_checksum: true,
        });
        assert!(c.config.verify_checksum);
    }
}
