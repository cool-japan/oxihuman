// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cryptomatte export (FNV-1a hash-based object ID matte).

/// A single Cryptomatte entry.
#[derive(Debug, Clone)]
pub struct CryptomatteEntry {
    pub name: String,
    pub hash: u32,
    pub coverage: f32,
}

/// FNV-1a 32-bit hash of a string.
pub fn cryptomatte_name_to_hash(name: &str) -> u32 {
    let mut hash: u32 = 2_166_136_261;
    for byte in name.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

/// Create a new `CryptomatteEntry` using the FNV-1a hash of `name`.
pub fn new_cryptomatte_entry(name: &str, coverage: f32) -> CryptomatteEntry {
    CryptomatteEntry {
        name: name.to_string(),
        hash: cryptomatte_name_to_hash(name),
        coverage,
    }
}

/// Serialize a single entry to JSON.
pub fn cryptomatte_to_json(e: &CryptomatteEntry) -> String {
    format!(
        "{{\"name\":\"{}\",\"hash\":{},\"coverage\":{}}}",
        e.name, e.hash, e.coverage
    )
}

/// Serialize multiple entries to a JSON array.
pub fn cryptomatte_entries_to_json(entries: &[CryptomatteEntry]) -> String {
    let inner: Vec<String> = entries.iter().map(cryptomatte_to_json).collect();
    format!("[{}]", inner.join(","))
}

/// Sum of all coverages.
pub fn cryptomatte_coverage_sum(entries: &[CryptomatteEntry]) -> f32 {
    entries.iter().map(|e| e.coverage).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a_known() {
        /* FNV-1a of empty string = 2166136261 */
        let h = cryptomatte_name_to_hash("");
        assert_eq!(h, 2_166_136_261);
    }

    #[test]
    fn test_new_cryptomatte_entry() {
        let e = new_cryptomatte_entry("Sphere", 0.8);
        assert_eq!(e.name, "Sphere");
        assert!(e.hash > 0);
    }

    #[test]
    fn test_cryptomatte_to_json() {
        let e = new_cryptomatte_entry("Cube", 0.5);
        let j = cryptomatte_to_json(&e);
        assert!(j.contains("Cube"));
    }

    #[test]
    fn test_cryptomatte_entries_to_json() {
        let entries = vec![
            new_cryptomatte_entry("A", 0.3),
            new_cryptomatte_entry("B", 0.7),
        ];
        let j = cryptomatte_entries_to_json(&entries);
        assert!(j.starts_with('['));
    }

    #[test]
    fn test_cryptomatte_coverage_sum() {
        let entries = vec![
            new_cryptomatte_entry("A", 0.3),
            new_cryptomatte_entry("B", 0.7),
        ];
        let s = cryptomatte_coverage_sum(&entries);
        assert!((s - 1.0).abs() < 1e-5);
    }
}
