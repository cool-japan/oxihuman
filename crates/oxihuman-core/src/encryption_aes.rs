// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! AES-GCM encryption stub.

/// Key length variants for AES.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AesKeyLen {
    Aes128,
    Aes256,
}

/// AES-GCM configuration.
#[derive(Debug, Clone)]
pub struct AesGcmConfig {
    pub key_len: AesKeyLen,
    pub tag_len: usize,
}

impl Default for AesGcmConfig {
    fn default() -> Self {
        Self {
            key_len: AesKeyLen::Aes256,
            tag_len: 16,
        }
    }
}

/// AES-GCM cipher stub.
#[derive(Debug, Clone)]
pub struct AesGcmCipher {
    pub config: AesGcmConfig,
    key: Vec<u8>,
}

impl AesGcmCipher {
    pub fn new(key: &[u8], config: AesGcmConfig) -> Result<Self, String> {
        /* stub: accept 16 or 32-byte keys */
        if key.len() != 16 && key.len() != 32 {
            return Err("aes-gcm: key must be 16 or 32 bytes".to_string());
        }
        Ok(Self {
            config,
            key: key.to_vec(),
        })
    }

    pub fn key_len_bytes(&self) -> usize {
        self.key.len()
    }
}

/// Encrypt plaintext using AES-GCM stub.
/// Returns `nonce (12 bytes) || tag (16 bytes) || ciphertext`.
pub fn aes_gcm_encrypt(key: &[u8], nonce: &[u8; 12], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    /* stub: XOR each byte with key[i % key.len()] */
    if key.len() != 16 && key.len() != 32 {
        return Err("aes-gcm: invalid key length".to_string());
    }
    let mut out = Vec::with_capacity(12 + 16 + plaintext.len());
    out.extend_from_slice(nonce);
    /* stub tag: first 16 bytes of key XOR nonce bytes (repeated) */
    let tag: Vec<u8> = (0..16)
        .map(|i| key[i % key.len()] ^ nonce[i % 12])
        .collect();
    out.extend_from_slice(&tag);
    for (i, &b) in plaintext.iter().enumerate() {
        out.push(b ^ key[i % key.len()]);
    }
    Ok(out)
}

/// Decrypt ciphertext produced by [`aes_gcm_encrypt`].
pub fn aes_gcm_decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    /* stub: validate length then XOR-undo */
    if ciphertext.len() < 28 {
        return Err("aes-gcm: ciphertext too short".to_string());
    }
    if key.len() != 16 && key.len() != 32 {
        return Err("aes-gcm: invalid key length".to_string());
    }
    let payload = &ciphertext[28..]; /* skip 12-byte nonce + 16-byte tag */
    Ok(payload
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect())
}

/// Generate a stub 256-bit key from a passphrase (not cryptographically secure).
pub fn aes_derive_key_stub(passphrase: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    for (i, b) in passphrase.bytes().enumerate() {
        key[i % 32] ^= b;
    }
    key
}

/// Return whether a key length is valid for AES.
pub fn aes_key_len_valid(len: usize) -> bool {
    len == 16 || len == 32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_key_len() {
        /* default config uses AES-256 */
        assert_eq!(AesGcmConfig::default().key_len, AesKeyLen::Aes256);
    }

    #[test]
    fn test_key_len_valid() {
        /* only 16 and 32 are valid */
        assert!(aes_key_len_valid(16));
        assert!(aes_key_len_valid(32));
        assert!(!aes_key_len_valid(24));
    }

    #[test]
    fn test_cipher_bad_key() {
        /* wrong key length returns error */
        let result = AesGcmCipher::new(&[0u8; 24], AesGcmConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_cipher_good_key() {
        /* 32-byte key is accepted */
        let c = AesGcmCipher::new(&[0u8; 32], AesGcmConfig::default()).unwrap();
        assert_eq!(c.key_len_bytes(), 32);
    }

    #[test]
    fn test_encrypt_then_decrypt() {
        /* round-trip encryption */
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plain = b"hello aes-gcm";
        let enc = aes_gcm_encrypt(&key, &nonce, plain).unwrap();
        let dec = aes_gcm_decrypt(&key, &enc).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn test_encrypt_bad_key() {
        /* wrong key length in encrypt */
        assert!(aes_gcm_encrypt(&[0u8; 24], &[0u8; 12], b"data").is_err());
    }

    #[test]
    fn test_decrypt_short() {
        /* too-short ciphertext errors */
        assert!(aes_gcm_decrypt(&[0u8; 32], &[0u8; 5]).is_err());
    }

    #[test]
    fn test_derive_key_stub() {
        /* derive returns 32 bytes */
        let key = aes_derive_key_stub("my-passphrase");
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_encrypted_output_len() {
        /* output = 12 + 16 + plaintext */
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let enc = aes_gcm_encrypt(&key, &nonce, b"hello").unwrap();
        assert_eq!(enc.len(), 12 + 16 + 5);
    }
}
