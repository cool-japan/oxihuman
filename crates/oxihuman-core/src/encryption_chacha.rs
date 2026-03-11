// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ChaCha20-Poly1305 encryption stub.

/// ChaCha20-Poly1305 configuration.
#[derive(Debug, Clone)]
pub struct ChaChaConfig {
    pub rounds: u8,
    pub tag_len: usize,
}

impl Default for ChaChaConfig {
    fn default() -> Self {
        Self {
            rounds: 20,
            tag_len: 16,
        }
    }
}

/// ChaCha20-Poly1305 cipher stub.
#[derive(Debug, Clone)]
pub struct ChaChaCipher {
    pub config: ChaChaConfig,
    key: [u8; 32],
}

impl ChaChaCipher {
    pub fn new(key: [u8; 32], config: ChaChaConfig) -> Self {
        Self { config, key }
    }

    pub fn default_cipher(key: [u8; 32]) -> Self {
        Self::new(key, ChaChaConfig::default())
    }
}

/// Encrypt plaintext with ChaCha20-Poly1305 stub.
/// Returns `nonce (12 bytes) || tag (16 bytes) || ciphertext`.
pub fn chacha_encrypt(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> Vec<u8> {
    /* stub: XOR with key stream derived from key */
    let mut out = Vec::with_capacity(28 + plaintext.len());
    out.extend_from_slice(nonce);
    let tag: Vec<u8> = (0..16).map(|i| key[i] ^ nonce[i % 12]).collect();
    out.extend_from_slice(&tag);
    for (i, &b) in plaintext.iter().enumerate() {
        out.push(b ^ key[i % 32]);
    }
    out
}

/// Decrypt ciphertext produced by [`chacha_encrypt`].
pub fn chacha_decrypt(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    /* stub: validate header and XOR-undo */
    if ciphertext.len() < 28 {
        return Err("chacha: ciphertext too short".to_string());
    }
    let payload = &ciphertext[28..];
    Ok(payload
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % 32])
        .collect())
}

/// Generate a stub 256-bit key from a seed value.
pub fn chacha_key_from_seed(seed: u64) -> [u8; 32] {
    let mut key = [0u8; 32];
    let bytes = seed.to_le_bytes();
    for i in 0..32 {
        key[i] = bytes[i % 8].wrapping_add(i as u8);
    }
    key
}

/// Return the nonce length used by this stub.
pub fn chacha_nonce_len() -> usize {
    12
}

/// Verify round-trip integrity.
pub fn chacha_roundtrip_ok(key: &[u8; 32], data: &[u8]) -> bool {
    let nonce = [0u8; 12];
    let enc = chacha_encrypt(key, &nonce, data);
    chacha_decrypt(key, &enc)
        .map(|d| d == data)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rounds() {
        /* default is 20 rounds */
        assert_eq!(ChaChaConfig::default().rounds, 20);
    }

    #[test]
    fn test_nonce_len() {
        /* nonce is 12 bytes */
        assert_eq!(chacha_nonce_len(), 12);
    }

    #[test]
    fn test_encrypt_output_len() {
        /* output = 12 + 16 + plaintext */
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let enc = chacha_encrypt(&key, &nonce, b"hello");
        assert_eq!(enc.len(), 28 + 5);
    }

    #[test]
    fn test_roundtrip_hello() {
        /* basic round-trip */
        let key = [0xABu8; 32];
        assert!(chacha_roundtrip_ok(&key, b"Hello ChaCha20!"));
    }

    #[test]
    fn test_roundtrip_binary() {
        /* binary data round-trip */
        let key = chacha_key_from_seed(12345);
        let data: Vec<u8> = (0u8..=255).collect();
        assert!(chacha_roundtrip_ok(&key, &data));
    }

    #[test]
    fn test_decrypt_short() {
        /* error on short ciphertext */
        let key = [0u8; 32];
        assert!(chacha_decrypt(&key, &[0u8; 10]).is_err());
    }

    #[test]
    fn test_key_from_seed() {
        /* key from seed is 32 bytes */
        let k = chacha_key_from_seed(0);
        assert_eq!(k.len(), 32);
    }

    #[test]
    fn test_cipher_new() {
        /* cipher constructor works */
        let key = [0u8; 32];
        let c = ChaChaCipher::default_cipher(key);
        assert_eq!(c.config.rounds, 20);
    }

    #[test]
    fn test_different_keys_give_different_ciphertext() {
        /* different keys produce different output */
        let nonce = [0u8; 12];
        let k1 = [0u8; 32];
        let k2 = [1u8; 32];
        let e1 = chacha_encrypt(&k1, &nonce, b"data");
        let e2 = chacha_encrypt(&k2, &nonce, b"data");
        assert_ne!(e1[28..], e2[28..]);
    }
}
