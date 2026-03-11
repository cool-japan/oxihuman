#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Name generation utilities for procedural/placeholder naming.

/// Style of generated names.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NameStyle {
    /// Short alphanumeric IDs.
    Short,
    /// Human-readable compound words.
    Readable,
    /// Kebab-case slugs.
    Slug,
}

/// A simple deterministic name generator.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NameGenerator {
    style: NameStyle,
    seed: u64,
    count: u64,
}

/// Prefix parts for readable names.
const PREFIXES: &[&str] = &[
    "alpha", "beta", "gamma", "delta", "echo", "foxtrot", "golf", "hotel",
];

/// Suffix parts for readable names.
const SUFFIXES: &[&str] = &[
    "node", "item", "unit", "block", "part", "entry", "slot", "cell",
];

/// Create a new `NameGenerator` with the given style.
#[allow(dead_code)]
pub fn new_name_generator(style: NameStyle) -> NameGenerator {
    NameGenerator { style, seed: 42, count: 0 }
}

/// Generate the next name from the generator.
#[allow(dead_code)]
pub fn generate_name(gen: &mut NameGenerator) -> String {
    let idx = (gen.seed.wrapping_add(gen.count)) as usize;
    gen.count += 1;
    match gen.style {
        NameStyle::Short => format!("n{:04x}", idx & 0xFFFF),
        NameStyle::Readable => {
            let p = PREFIXES[idx % PREFIXES.len()];
            let s = SUFFIXES[(idx / PREFIXES.len()) % SUFFIXES.len()];
            format!("{p}_{s}")
        }
        NameStyle::Slug => {
            let p = PREFIXES[idx % PREFIXES.len()];
            let s = SUFFIXES[(idx / PREFIXES.len()) % SUFFIXES.len()];
            format!("{p}-{s}-{}", idx % 100)
        }
    }
}

/// Append a numeric suffix to a name.
#[allow(dead_code)]
pub fn name_with_suffix(name: &str, suffix: u32) -> String {
    format!("{name}_{suffix}")
}

/// Prepend a prefix string to a name.
#[allow(dead_code)]
pub fn name_with_prefix(prefix: &str, name: &str) -> String {
    format!("{prefix}_{name}")
}

/// Set the seed of the generator.
#[allow(dead_code)]
pub fn set_seed(gen: &mut NameGenerator, seed: u64) {
    gen.seed = seed;
    gen.count = 0;
}

/// Return total names generated so far.
#[allow(dead_code)]
pub fn name_count_generated(gen: &NameGenerator) -> u64 {
    gen.count
}

/// Reset the generator count to zero.
#[allow(dead_code)]
pub fn reset_generator(gen: &mut NameGenerator) {
    gen.count = 0;
}

/// Check if a name string is valid (non-empty, ASCII alphanumeric + _ + -).
#[allow(dead_code)]
pub fn name_is_valid(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_name_generator() {
        let gen = new_name_generator(NameStyle::Short);
        assert_eq!(gen.count, 0);
    }

    #[test]
    fn test_generate_name_short() {
        let mut gen = new_name_generator(NameStyle::Short);
        let name = generate_name(&mut gen);
        assert!(name.starts_with('n'));
        assert_eq!(gen.count, 1);
    }

    #[test]
    fn test_generate_name_readable() {
        let mut gen = new_name_generator(NameStyle::Readable);
        let name = generate_name(&mut gen);
        assert!(name.contains('_'));
    }

    #[test]
    fn test_generate_name_slug() {
        let mut gen = new_name_generator(NameStyle::Slug);
        let name = generate_name(&mut gen);
        assert!(name.contains('-'));
    }

    #[test]
    fn test_name_with_suffix() {
        assert_eq!(name_with_suffix("foo", 3), "foo_3");
    }

    #[test]
    fn test_name_with_prefix() {
        assert_eq!(name_with_prefix("pre", "bar"), "pre_bar");
    }

    #[test]
    fn test_set_seed() {
        let mut gen = new_name_generator(NameStyle::Short);
        let _ = generate_name(&mut gen);
        set_seed(&mut gen, 99);
        assert_eq!(gen.count, 0);
        assert_eq!(gen.seed, 99);
    }

    #[test]
    fn test_name_count_generated() {
        let mut gen = new_name_generator(NameStyle::Short);
        assert_eq!(name_count_generated(&gen), 0);
        let _ = generate_name(&mut gen);
        assert_eq!(name_count_generated(&gen), 1);
    }

    #[test]
    fn test_reset_generator() {
        let mut gen = new_name_generator(NameStyle::Short);
        let _ = generate_name(&mut gen);
        reset_generator(&mut gen);
        assert_eq!(gen.count, 0);
    }

    #[test]
    fn test_name_is_valid() {
        assert!(name_is_valid("hello_world"));
        assert!(name_is_valid("abc-123"));
        assert!(!name_is_valid(""));
        assert!(!name_is_valid("has space"));
    }
}
