// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Zstandard compression stub.

/// Configuration for the Zstd compressor.
#[derive(Debug, Clone)]
pub struct ZstdConfig {
    pub level: i32,
    pub checksum: bool,
}

impl Default for ZstdConfig {
    fn default() -> Self {
        Self {
            level: 3,
            checksum: true,
        }
    }
}

/// Zstd compressor stub.
#[derive(Debug, Clone)]
pub struct ZstdCompressor {
    pub config: ZstdConfig,
}

impl ZstdCompressor {
    pub fn new(config: ZstdConfig) -> Self {
        Self { config }
    }

    pub fn default_compressor() -> Self {
        Self::new(ZstdConfig::default())
    }

    pub fn with_level(level: i32) -> Self {
        Self::new(ZstdConfig {
            level,
            checksum: true,
        })
    }
}

/// Compress bytes using Zstd stub (stores raw with 4-byte header).
pub fn zstd_compress(data: &[u8]) -> Vec<u8> {
    /* stub: 4-byte LE length prefix followed by raw bytes */
    let mut out = Vec::with_capacity(data.len() + 4);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    out
}

/// Decompress bytes produced by [`zstd_compress`].
pub fn zstd_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    /* stub: verify and strip header */
    if data.len() < 4 {
        return Err("zstd: input too short".to_string());
    }
    let expected = u32::from_le_bytes(data[..4].try_into().unwrap_or_default()) as usize;
    let payload = &data[4..];
    if payload.len() < expected {
        return Err("zstd: truncated data".to_string());
    }
    Ok(payload[..expected].to_vec())
}

/// Estimate the frame size for a given input length (stub).
pub fn zstd_frame_size_estimate(input_len: usize) -> usize {
    input_len + 12
}

/// Return true if the data length is sufficient for a valid Zstd stub frame.
pub fn zstd_frame_valid(data: &[u8]) -> bool {
    data.len() >= 4
}

/// Verify round-trip integrity.
pub fn zstd_roundtrip_ok(data: &[u8]) -> bool {
    zstd_decompress(&zstd_compress(data))
        .map(|d| d == data)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_level() {
        /* default compression level */
        assert_eq!(ZstdConfig::default().level, 3);
    }

    #[test]
    fn test_with_level() {
        /* custom level constructor */
        let c = ZstdCompressor::with_level(9);
        assert_eq!(c.config.level, 9);
    }

    #[test]
    fn test_compress_empty() {
        /* empty input gives 4-byte header */
        assert_eq!(zstd_compress(&[]).len(), 4);
    }

    #[test]
    fn test_roundtrip_hello() {
        /* hello round-trip */
        assert!(zstd_roundtrip_ok(b"hello zstd"));
    }

    #[test]
    fn test_roundtrip_binary() {
        /* binary round-trip */
        let data: Vec<u8> = (0u8..128).collect();
        assert!(zstd_roundtrip_ok(&data));
    }

    #[test]
    fn test_decompress_short() {
        /* error on short input */
        assert!(zstd_decompress(&[1, 2]).is_err());
    }

    #[test]
    fn test_frame_size_estimate() {
        /* estimate is larger than input */
        assert!(zstd_frame_size_estimate(100) > 100);
    }

    #[test]
    fn test_frame_valid() {
        /* 4+ bytes is valid stub frame */
        assert!(zstd_frame_valid(&[0, 0, 0, 0]));
        assert!(!zstd_frame_valid(&[0]));
    }

    #[test]
    fn test_checksum_default() {
        /* checksum enabled by default */
        assert!(ZstdConfig::default().checksum);
    }
}
