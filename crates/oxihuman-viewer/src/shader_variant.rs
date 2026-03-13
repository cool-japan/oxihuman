// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A key that identifies a shader variant.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariantKey {
    pub defines: Vec<String>,
}

/// A shader variant with defines and compiled state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderVariantEntry {
    pub key: VariantKey,
    pub compiled: bool,
    pub source_hash: u64,
}

/// Collection of shader variants.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ShaderVariantSet {
    pub variants: HashMap<u64, ShaderVariantEntry>,
    next_id: u64,
}

/// Create a new shader variant entry.
#[allow(dead_code)]
pub fn new_shader_variant(defines: &[&str]) -> ShaderVariantEntry {
    ShaderVariantEntry {
        key: VariantKey {
            defines: defines.iter().map(|s| s.to_string()).collect(),
        },
        compiled: false,
        source_hash: 0,
    }
}

/// Return the variant key.
#[allow(dead_code)]
pub fn variant_key(v: &ShaderVariantEntry) -> &VariantKey {
    &v.key
}

/// Return the defines as a slice.
#[allow(dead_code)]
pub fn variant_defines(v: &ShaderVariantEntry) -> &[String] {
    &v.key.defines
}

/// Return the number of defines in the variant.
#[allow(dead_code)]
pub fn variant_count(v: &ShaderVariantEntry) -> usize {
    v.key.defines.len()
}

/// Check if a variant matches a set of required defines.
#[allow(dead_code)]
pub fn variant_matches(v: &ShaderVariantEntry, required: &[&str]) -> bool {
    required
        .iter()
        .all(|r| v.key.defines.iter().any(|d| d == r))
}

/// Serialize the variant to JSON.
#[allow(dead_code)]
pub fn variant_to_json(v: &ShaderVariantEntry) -> String {
    let defs: Vec<String> = v.key.defines.iter().map(|d| format!("\"{}\"", d)).collect();
    format!(
        "{{\"defines\":[{}],\"compiled\":{}}}",
        defs.join(","),
        v.compiled
    )
}

/// Stub: compile the variant (just marks it as compiled).
#[allow(dead_code)]
pub fn compile_variant_stub(v: &mut ShaderVariantEntry) {
    v.compiled = true;
}

/// Compute a simple hash for the variant key.
#[allow(dead_code)]
pub fn variant_hash(v: &ShaderVariantEntry) -> u64 {
    let mut h: u64 = 5381;
    for d in &v.key.defines {
        for b in d.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_variant() {
        let v = new_shader_variant(&["NORMAL_MAP", "SHADOWS"]);
        assert_eq!(variant_count(&v), 2);
    }

    #[test]
    fn key_accessor() {
        let v = new_shader_variant(&["A"]);
        assert_eq!(variant_key(&v).defines.len(), 1);
    }

    #[test]
    fn defines_accessor() {
        let v = new_shader_variant(&["X", "Y"]);
        assert_eq!(variant_defines(&v).len(), 2);
    }

    #[test]
    fn matches_all() {
        let v = new_shader_variant(&["A", "B", "C"]);
        assert!(variant_matches(&v, &["A", "B"]));
    }

    #[test]
    fn no_match() {
        let v = new_shader_variant(&["A"]);
        assert!(!variant_matches(&v, &["B"]));
    }

    #[test]
    fn compile_stub() {
        let mut v = new_shader_variant(&["A"]);
        assert!(!v.compiled);
        compile_variant_stub(&mut v);
        assert!(v.compiled);
    }

    #[test]
    fn to_json() {
        let v = new_shader_variant(&["TEST"]);
        let j = variant_to_json(&v);
        assert!(j.contains("TEST"));
    }

    #[test]
    fn hash_deterministic() {
        let v = new_shader_variant(&["A", "B"]);
        let h1 = variant_hash(&v);
        let h2 = variant_hash(&v);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_different_for_different() {
        let a = new_shader_variant(&["A"]);
        let b = new_shader_variant(&["B"]);
        assert_ne!(variant_hash(&a), variant_hash(&b));
    }

    #[test]
    fn empty_variant() {
        let v = new_shader_variant(&[]);
        assert_eq!(variant_count(&v), 0);
    }
}
