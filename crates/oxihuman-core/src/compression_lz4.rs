// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LZ4-style block compression stub.

/// Configuration for the LZ4 compressor.
#[derive(Debug, Clone)]
pub struct Lz4Config {
    pub acceleration: i32,
    pub block_size: usize,
}

impl Default for Lz4Config {
    fn default() -> Self {
        Self {
            acceleration: 1,
            block_size: 65536,
        }
    }
}

/// LZ4 compressor stub.
#[derive(Debug, Clone)]
pub struct Lz4Compressor {
    pub config: Lz4Config,
}

impl Lz4Compressor {
    pub fn new(config: Lz4Config) -> Self {
        Self { config }
    }

    pub fn default_compressor() -> Self {
        Self::new(Lz4Config::default())
    }
}

/// Compress bytes using LZ4-style block encoding stub.
pub fn lz4_compress(data: &[u8]) -> Vec<u8> {
    /* stub: prefix length then raw bytes */
    let mut out = Vec::with_capacity(data.len() + 8);
    let len = data.len() as u32;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(data);
    out
}

/// Decompress bytes produced by [`lz4_compress`].
pub fn lz4_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    /* stub: strip 4-byte length prefix */
    if data.len() < 4 {
        return Err("lz4: input too short".to_string());
    }
    let expected = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let payload = &data[4..];
    if payload.len() < expected {
        return Err("lz4: truncated payload".to_string());
    }
    Ok(payload[..expected].to_vec())
}

/// Return the maximum compressed size for `input_len` bytes (stub estimate).
pub fn lz4_compress_bound(input_len: usize) -> usize {
    input_len + input_len / 255 + 16
}

/// Return whether a byte slice begins with the LZ4 stub magic.
pub fn lz4_is_compressed(data: &[u8]) -> bool {
    data.len() >= 4
}

/// Round-trip compress then decompress and verify equality.
pub fn lz4_roundtrip_ok(data: &[u8]) -> bool {
    match lz4_decompress(&lz4_compress(data)) {
        Ok(out) => out == data,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        /* acceleration should be 1 by default */
        let cfg = Lz4Config::default();
        assert_eq!(cfg.acceleration, 1);
    }

    #[test]
    fn test_compress_bound() {
        /* bound must be at least input length */
        assert!(lz4_compress_bound(100) >= 100);
    }

    #[test]
    fn test_compress_empty() {
        /* compressing empty slice yields 4-byte header */
        let out = lz4_compress(&[]);
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn test_roundtrip_hello() {
        /* basic round-trip */
        assert!(lz4_roundtrip_ok(b"Hello, World!"));
    }

    #[test]
    fn test_roundtrip_binary() {
        /* binary round-trip */
        let data: Vec<u8> = (0u8..=255).collect();
        assert!(lz4_roundtrip_ok(&data));
    }

    #[test]
    fn test_decompress_too_short() {
        /* error on tiny input */
        assert!(lz4_decompress(&[0]).is_err());
    }

    #[test]
    fn test_is_compressed_false() {
        /* empty slice is not flagged as compressed */
        assert!(!lz4_is_compressed(&[]));
    }

    #[test]
    fn test_is_compressed_true() {
        /* any 4+ byte slice passes the stub check */
        assert!(lz4_is_compressed(&[0, 0, 0, 0]));
    }

    #[test]
    fn test_compressor_new() {
        /* constructors work */
        let c = Lz4Compressor::default_compressor();
        assert_eq!(c.config.acceleration, 1);
    }
}
