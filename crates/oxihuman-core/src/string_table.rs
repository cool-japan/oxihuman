// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Localization string table mapping (key, locale) -> string.

#![allow(dead_code)]

use std::collections::HashMap;

/// A localization string table: key -> locale -> text.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct StringTable {
    pub entries: HashMap<String, HashMap<String, String>>,
}

/// Creates a new empty `StringTable`.
#[allow(dead_code)]
pub fn new_string_table() -> StringTable {
    StringTable {
        entries: HashMap::new(),
    }
}

/// Sets a localized string for a given key and locale.
#[allow(dead_code)]
pub fn st_set(table: &mut StringTable, key: &str, locale: &str, text: &str) {
    table
        .entries
        .entry(key.to_string())
        .or_default()
        .insert(locale.to_string(), text.to_string());
}

/// Retrieves a localized string. Returns `None` if key or locale is missing.
#[allow(dead_code)]
pub fn st_get<'a>(table: &'a StringTable, key: &str, locale: &str) -> Option<&'a str> {
    table
        .entries
        .get(key)
        .and_then(|loc_map| loc_map.get(locale))
        .map(String::as_str)
}

/// Retrieves a localized string, returning `fallback` if not found.
#[allow(dead_code)]
pub fn st_get_fallback<'a>(
    table: &'a StringTable,
    key: &str,
    locale: &str,
    fallback: &'a str,
) -> &'a str {
    st_get(table, key, locale).unwrap_or(fallback)
}

/// Returns the number of unique keys in the table.
#[allow(dead_code)]
pub fn st_key_count(table: &StringTable) -> usize {
    table.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let t = new_string_table();
        assert_eq!(st_key_count(&t), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut t = new_string_table();
        st_set(&mut t, "hello", "en", "Hello");
        assert_eq!(st_get(&t, "hello", "en"), Some("Hello"));
    }

    #[test]
    fn test_get_missing_key() {
        let t = new_string_table();
        assert!(st_get(&t, "nope", "en").is_none());
    }

    #[test]
    fn test_get_missing_locale() {
        let mut t = new_string_table();
        st_set(&mut t, "hello", "en", "Hello");
        assert!(st_get(&t, "hello", "ja").is_none());
    }

    #[test]
    fn test_fallback_found() {
        let mut t = new_string_table();
        st_set(&mut t, "hi", "en", "Hi");
        assert_eq!(st_get_fallback(&t, "hi", "en", "default"), "Hi");
    }

    #[test]
    fn test_fallback_missing() {
        let t = new_string_table();
        assert_eq!(st_get_fallback(&t, "hi", "en", "default"), "default");
    }

    #[test]
    fn test_key_count() {
        let mut t = new_string_table();
        st_set(&mut t, "a", "en", "A");
        st_set(&mut t, "b", "en", "B");
        assert_eq!(st_key_count(&t), 2);
    }

    #[test]
    fn test_multiple_locales_same_key() {
        let mut t = new_string_table();
        st_set(&mut t, "hello", "en", "Hello");
        st_set(&mut t, "hello", "ja", "こんにちは");
        assert_eq!(st_key_count(&t), 1);
        assert_eq!(st_get(&t, "hello", "ja"), Some("こんにちは"));
    }

    #[test]
    fn test_overwrite() {
        let mut t = new_string_table();
        st_set(&mut t, "x", "en", "old");
        st_set(&mut t, "x", "en", "new");
        assert_eq!(st_get(&t, "x", "en"), Some("new"));
    }
}
