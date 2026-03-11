// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Type alias map: maps alias names to canonical type names.

use std::collections::HashMap;

/// A registry of type aliases.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TypeAliasMap {
    aliases: HashMap<String, String>,
    reverse: HashMap<String, Vec<String>>,
}

/// Create a new `TypeAliasMap`.
#[allow(dead_code)]
pub fn new_type_alias_map() -> TypeAliasMap {
    TypeAliasMap::default()
}

/// Register an alias pointing to a canonical name.
#[allow(dead_code)]
pub fn tam_register(tam: &mut TypeAliasMap, alias: &str, canonical: &str) {
    tam.aliases.insert(alias.to_string(), canonical.to_string());
    tam.reverse
        .entry(canonical.to_string())
        .or_default()
        .push(alias.to_string());
}

/// Resolve an alias (or return the name itself if not an alias).
#[allow(dead_code)]
pub fn tam_resolve(tam: &TypeAliasMap, name: &str) -> String {
    tam.aliases
        .get(name)
        .cloned()
        .unwrap_or_else(|| name.to_string())
}

/// Whether a name is a registered alias.
#[allow(dead_code)]
pub fn tam_is_alias(tam: &TypeAliasMap, name: &str) -> bool {
    tam.aliases.contains_key(name)
}

/// All aliases pointing to a canonical name.
#[allow(dead_code)]
pub fn tam_aliases_for(tam: &TypeAliasMap, canonical: &str) -> Vec<String> {
    tam.reverse.get(canonical).cloned().unwrap_or_default()
}

/// Remove an alias.
#[allow(dead_code)]
pub fn tam_remove(tam: &mut TypeAliasMap, alias: &str) {
    if let Some(canonical) = tam.aliases.remove(alias) {
        if let Some(list) = tam.reverse.get_mut(&canonical) {
            list.retain(|a| a != alias);
        }
    }
}

/// Total number of registered aliases.
#[allow(dead_code)]
pub fn tam_count(tam: &TypeAliasMap) -> usize {
    tam.aliases.len()
}

/// Clear all aliases.
#[allow(dead_code)]
pub fn tam_clear(tam: &mut TypeAliasMap) {
    tam.aliases.clear();
    tam.reverse.clear();
}

/// All registered alias names.
#[allow(dead_code)]
pub fn tam_all_aliases(tam: &TypeAliasMap) -> Vec<String> {
    tam.aliases.keys().cloned().collect()
}

/// Recursively resolve (follow chains).
#[allow(dead_code)]
pub fn tam_resolve_chain(tam: &TypeAliasMap, name: &str) -> String {
    let mut current = name.to_string();
    for _ in 0..64 {
        match tam.aliases.get(&current) {
            Some(next) if next != &current => current = next.clone(),
            _ => break,
        }
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_resolve() {
        let mut tam = new_type_alias_map();
        tam_register(&mut tam, "uint", "u32");
        assert_eq!(tam_resolve(&tam, "uint"), "u32".to_string());
    }

    #[test]
    fn test_no_alias_passthrough() {
        let tam = new_type_alias_map();
        assert_eq!(tam_resolve(&tam, "f64"), "f64".to_string());
    }

    #[test]
    fn test_is_alias() {
        let mut tam = new_type_alias_map();
        tam_register(&mut tam, "int", "i32");
        assert!(tam_is_alias(&tam, "int"));
        assert!(!tam_is_alias(&tam, "i32"));
    }

    #[test]
    fn test_aliases_for() {
        let mut tam = new_type_alias_map();
        tam_register(&mut tam, "float", "f32");
        tam_register(&mut tam, "single", "f32");
        let aliases = tam_aliases_for(&tam, "f32");
        assert_eq!(aliases.len(), 2);
    }

    #[test]
    fn test_remove() {
        let mut tam = new_type_alias_map();
        tam_register(&mut tam, "uint", "u32");
        tam_remove(&mut tam, "uint");
        assert!(!tam_is_alias(&tam, "uint"));
    }

    #[test]
    fn test_count() {
        let mut tam = new_type_alias_map();
        tam_register(&mut tam, "a", "b");
        tam_register(&mut tam, "c", "d");
        assert_eq!(tam_count(&tam), 2);
    }

    #[test]
    fn test_clear() {
        let mut tam = new_type_alias_map();
        tam_register(&mut tam, "a", "b");
        tam_clear(&mut tam);
        assert_eq!(tam_count(&tam), 0);
    }

    #[test]
    fn test_resolve_chain() {
        let mut tam = new_type_alias_map();
        tam_register(&mut tam, "A", "B");
        tam_register(&mut tam, "B", "C");
        assert_eq!(tam_resolve_chain(&tam, "A"), "C".to_string());
    }

    #[test]
    fn test_all_aliases() {
        let mut tam = new_type_alias_map();
        tam_register(&mut tam, "x", "X");
        tam_register(&mut tam, "y", "Y");
        let all = tam_all_aliases(&tam);
        assert_eq!(all.len(), 2);
    }
}
