// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Content-addressed asset tracking using a rolling FNV-inspired hash.

// ── Hash type ─────────────────────────────────────────────────────────────────

/// A 32-byte content hash.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetHash(pub [u8; 32]);

// ── Hasher ────────────────────────────────────────────────────────────────────

/// FNV-inspired 256-bit rolling hasher (eight 32-bit lanes).
#[allow(dead_code)]
pub struct AssetHasher {
    state: [u32; 8],
}

/// FNV-1a prime (32-bit).
const FNV_PRIME: u32 = 0x0100_0193;

impl AssetHasher {
    /// Initialise with FNV offset basis split across eight lanes.
    #[allow(dead_code)]
    pub fn new() -> Self {
        // Use eight distinct constants inspired by FNV offset basis
        Self {
            state: [
                0x811c_9dc5,
                0x811c_9dc5 ^ 0x5a82_7999,
                0x811c_9dc5 ^ 0x9e37_79b9,
                0x811c_9dc5 ^ 0xf1bb_cdcb,
                0x811c_9dc5 ^ 0x27c4_acdd,
                0x811c_9dc5 ^ 0x6c62_272e,
                0x811c_9dc5 ^ 0xa09e_667f,
                0x811c_9dc5 ^ 0xcd3d_4f0d,
            ],
        }
    }

    /// Process a slice of bytes.
    #[allow(dead_code)]
    pub fn update(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let lane = i & 7;
            self.state[lane] ^= byte as u32;
            self.state[lane] = self.state[lane].wrapping_mul(FNV_PRIME);
            // Mix adjacent lanes for better diffusion
            self.state[lane] ^= self.state[(lane + 1) & 7].rotate_right(13);
        }
    }

    /// Produce the final 32-byte digest.
    #[allow(dead_code)]
    pub fn finalize(&self) -> AssetHash {
        let mut out = [0u8; 32];
        for (i, &word) in self.state.iter().enumerate() {
            let bytes = word.to_le_bytes();
            out[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }
        AssetHash(out)
    }
}

impl Default for AssetHasher {
    fn default() -> Self {
        Self::new()
    }
}

// ── Convenience helpers ───────────────────────────────────────────────────────

/// Hash a byte slice.
#[allow(dead_code)]
pub fn hash_bytes(data: &[u8]) -> AssetHash {
    let mut h = AssetHasher::new();
    h.update(data);
    h.finalize()
}

/// Hash string content.
#[allow(dead_code)]
pub fn hash_file_content(content: &str) -> AssetHash {
    hash_bytes(content.as_bytes())
}

// ── AssetHash helpers ─────────────────────────────────────────────────────────

impl AssetHash {
    /// Encode as a 64-character lowercase hex string.
    #[allow(dead_code)]
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Decode from a 64-character hex string.
    #[allow(dead_code)]
    pub fn from_hex(s: &str) -> Result<AssetHash, String> {
        if s.len() != 64 {
            return Err(format!("Expected 64 hex chars, got {}", s.len()));
        }
        let mut out = [0u8; 32];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let hi = hex_nibble(chunk[0])?;
            let lo = hex_nibble(chunk[1])?;
            out[i] = (hi << 4) | lo;
        }
        Ok(AssetHash(out))
    }
}

fn hex_nibble(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("Invalid hex character: {}", b as char)),
    }
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// A single record in the content-addressed registry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetRecord {
    pub hash: AssetHash,
    pub path: String,
    pub size_bytes: usize,
    pub kind: String,
}

/// Content-addressed asset registry.
#[allow(dead_code)]
pub struct AssetRegistry {
    records: Vec<AssetRecord>,
}

impl AssetRegistry {
    /// Create an empty registry.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Register an asset by its byte content. Returns the hash.
    #[allow(dead_code)]
    pub fn register(&mut self, path: &str, content: &[u8], kind: &str) -> AssetHash {
        let hash = hash_bytes(content);
        self.records.push(AssetRecord {
            hash: hash.clone(),
            path: path.to_string(),
            size_bytes: content.len(),
            kind: kind.to_string(),
        });
        hash
    }

    /// Find a record by its hash.
    #[allow(dead_code)]
    pub fn find_by_hash(&self, hash: &AssetHash) -> Option<&AssetRecord> {
        self.records.iter().find(|r| &r.hash == hash)
    }

    /// Find a record by path (first match).
    #[allow(dead_code)]
    pub fn find_by_path(&self, path: &str) -> Option<&AssetRecord> {
        self.records.iter().find(|r| r.path == path)
    }

    /// All hashes in registration order.
    #[allow(dead_code)]
    pub fn all_hashes(&self) -> Vec<AssetHash> {
        self.records.iter().map(|r| r.hash.clone()).collect()
    }

    /// Count of unique hashes (deduplicated).
    #[allow(dead_code)]
    pub fn dedup_count(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for r in &self.records {
            seen.insert(r.hash.clone());
        }
        seen.len()
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes_deterministic() {
        let data = b"hello oxihuman";
        let h1 = hash_bytes(data);
        let h2 = hash_bytes(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_bytes_different_inputs_differ() {
        let h1 = hash_bytes(b"abc");
        let h2 = hash_bytes(b"xyz");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_to_hex_length_64() {
        let h = hash_bytes(b"test");
        assert_eq!(h.to_hex().len(), 64);
    }

    #[test]
    fn test_to_hex_lowercase() {
        let h = hash_bytes(b"test");
        let hex = h.to_hex();
        assert!(hex.chars().all(|c| c.is_ascii_digit() || c.is_lowercase()));
    }

    #[test]
    fn test_from_hex_roundtrip() {
        let h = hash_bytes(b"roundtrip test");
        let hex = h.to_hex();
        let h2 = AssetHash::from_hex(&hex).expect("should succeed");
        assert_eq!(h, h2);
    }

    #[test]
    fn test_from_hex_bad_length_error() {
        assert!(AssetHash::from_hex("abc").is_err());
    }

    #[test]
    fn test_from_hex_invalid_char_error() {
        let bad = "z".repeat(64);
        assert!(AssetHash::from_hex(&bad).is_err());
    }

    #[test]
    fn test_register_and_find_by_hash() {
        let mut reg = AssetRegistry::new();
        let h = reg.register("mesh/body.obj", b"obj data", "mesh");
        let found = reg.find_by_hash(&h);
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").path, "mesh/body.obj");
    }

    #[test]
    fn test_find_by_path() {
        let mut reg = AssetRegistry::new();
        reg.register("tex/skin.png", b"png data", "texture");
        let found = reg.find_by_path("tex/skin.png");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").kind, "texture");
    }

    #[test]
    fn test_find_by_hash_missing_returns_none() {
        let reg = AssetRegistry::new();
        let h = hash_bytes(b"ghost");
        assert!(reg.find_by_hash(&h).is_none());
    }

    #[test]
    fn test_dedup_count_same_content() {
        let mut reg = AssetRegistry::new();
        reg.register("a.obj", b"same", "mesh");
        reg.register("b.obj", b"same", "mesh");
        assert_eq!(reg.dedup_count(), 1);
    }

    #[test]
    fn test_dedup_count_different_content() {
        let mut reg = AssetRegistry::new();
        reg.register("a.obj", b"aaa", "mesh");
        reg.register("b.obj", b"bbb", "mesh");
        assert_eq!(reg.dedup_count(), 2);
    }

    #[test]
    fn test_empty_registry_dedup_zero() {
        let reg = AssetRegistry::new();
        assert_eq!(reg.dedup_count(), 0);
    }

    #[test]
    fn test_all_hashes_length() {
        let mut reg = AssetRegistry::new();
        reg.register("a", b"1", "t");
        reg.register("b", b"2", "t");
        assert_eq!(reg.all_hashes().len(), 2);
    }

    #[test]
    fn test_hash_file_content_same_as_bytes() {
        let s = "hello world";
        assert_eq!(hash_file_content(s), hash_bytes(s.as_bytes()));
    }
}
