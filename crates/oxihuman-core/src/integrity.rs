// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Reads `path`, computes SHA-256, and checks it matches `expected_hex`.
pub fn verify_sha256(path: &Path, expected_hex: &str) -> Result<()> {
    let data = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let actual = hex::encode(result);
    if actual != expected_hex.to_lowercase() {
        bail!(
            "SHA-256 mismatch for {}: expected {}, got {}",
            path.display(),
            expected_hex,
            actual
        );
    }
    Ok(())
}

/// Compute SHA-256 of bytes and return as hex string.
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_known_value() {
        // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2ec73b00361bbef0469f492c438df9a1ef7e... (first few)
        let h = hash_bytes(b"abc");
        assert!(h.starts_with("ba7816bf"));
    }
}
