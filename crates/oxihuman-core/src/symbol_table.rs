// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A symbol table that maps string names to integer symbol IDs and back.

use std::collections::HashMap;

/// An opaque symbol identifier.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

/// A bidirectional symbol table.
#[allow(dead_code)]
pub struct SymbolTable {
    name_to_id: HashMap<String, SymbolId>,
    id_to_name: Vec<String>,
    scope: String,
}

#[allow(dead_code)]
impl SymbolTable {
    pub fn new(scope: &str) -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
            scope: scope.to_string(),
        }
    }

    /// Interns a name, returning a new or existing [`SymbolId`].
    pub fn intern(&mut self, name: &str) -> SymbolId {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }
        let id = SymbolId(self.id_to_name.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.push(name.to_string());
        id
    }

    /// Looks up a name by id.
    pub fn lookup(&self, id: SymbolId) -> Option<&str> {
        self.id_to_name.get(id.0 as usize).map(|s| s.as_str())
    }

    /// Looks up an id by name.
    pub fn find(&self, name: &str) -> Option<SymbolId> {
        self.name_to_id.get(name).copied()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.id_to_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id_to_name.is_empty()
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }

    pub fn all_names(&self) -> &[String] {
        &self.id_to_name
    }

    pub fn clear(&mut self) {
        self.name_to_id.clear();
        self.id_to_name.clear();
    }

    /// Returns all ids that match a prefix.
    pub fn ids_with_prefix(&self, prefix: &str) -> Vec<SymbolId> {
        self.name_to_id
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(_, &v)| v)
            .collect()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new("global")
    }
}

pub fn new_symbol_table(scope: &str) -> SymbolTable {
    SymbolTable::new(scope)
}

pub fn sym_intern(table: &mut SymbolTable, name: &str) -> SymbolId {
    table.intern(name)
}

pub fn sym_lookup(table: &SymbolTable, id: SymbolId) -> Option<&str> {
    table.lookup(id)
}

pub fn sym_find(table: &SymbolTable, name: &str) -> Option<SymbolId> {
    table.find(name)
}

pub fn sym_len(table: &SymbolTable) -> usize {
    table.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let t = new_symbol_table("test");
        assert!(t.is_empty());
    }

    #[test]
    fn intern_returns_same_id() {
        let mut t = new_symbol_table("test");
        let id1 = sym_intern(&mut t, "foo");
        let id2 = sym_intern(&mut t, "foo");
        assert_eq!(id1, id2);
    }

    #[test]
    fn different_names_get_different_ids() {
        let mut t = new_symbol_table("test");
        let id1 = sym_intern(&mut t, "a");
        let id2 = sym_intern(&mut t, "b");
        assert_ne!(id1, id2);
    }

    #[test]
    fn lookup_roundtrip() {
        let mut t = new_symbol_table("test");
        let id = sym_intern(&mut t, "hello");
        assert_eq!(sym_lookup(&t, id), Some("hello"));
    }

    #[test]
    fn find_existing() {
        let mut t = new_symbol_table("test");
        sym_intern(&mut t, "x");
        assert!(sym_find(&t, "x").is_some());
    }

    #[test]
    fn find_missing_returns_none() {
        let t = new_symbol_table("test");
        assert!(sym_find(&t, "ghost").is_none());
    }

    #[test]
    fn len_grows() {
        let mut t = new_symbol_table("test");
        sym_intern(&mut t, "a");
        sym_intern(&mut t, "b");
        sym_intern(&mut t, "a"); // duplicate
        assert_eq!(sym_len(&t), 2);
    }

    #[test]
    fn clear_empties() {
        let mut t = new_symbol_table("test");
        sym_intern(&mut t, "x");
        t.clear();
        assert!(t.is_empty());
    }

    #[test]
    fn ids_with_prefix() {
        let mut t = new_symbol_table("test");
        sym_intern(&mut t, "mesh_body");
        sym_intern(&mut t, "mesh_hair");
        sym_intern(&mut t, "texture_skin");
        let ids = t.ids_with_prefix("mesh_");
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn scope_is_stored() {
        let t = new_symbol_table("physics");
        assert_eq!(t.scope(), "physics");
    }
}
