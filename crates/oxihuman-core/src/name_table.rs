// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Name table: bidirectional map between string names and u32 IDs.

use std::collections::HashMap;

/// Bidirectional name↔ID table.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NameTable {
    name_to_id: HashMap<String, u32>,
    id_to_name: HashMap<u32, String>,
    next_id: u32,
}

/// Create a new NameTable.
#[allow(dead_code)]
pub fn new_name_table() -> NameTable {
    NameTable {
        name_to_id: HashMap::new(),
        id_to_name: HashMap::new(),
        next_id: 1,
    }
}

/// Register a name; returns existing ID if already registered.
#[allow(dead_code)]
pub fn nt_register(nt: &mut NameTable, name: &str) -> u32 {
    if let Some(&id) = nt.name_to_id.get(name) {
        return id;
    }
    let id = nt.next_id;
    nt.next_id += 1;
    nt.name_to_id.insert(name.to_string(), id);
    nt.id_to_name.insert(id, name.to_string());
    id
}

/// Look up ID by name.
#[allow(dead_code)]
pub fn nt_id(nt: &NameTable, name: &str) -> Option<u32> {
    nt.name_to_id.get(name).copied()
}

/// Look up name by ID.
#[allow(dead_code)]
pub fn nt_name(nt: &NameTable, id: u32) -> Option<&str> {
    nt.id_to_name.get(&id).map(|s| s.as_str())
}

/// Whether a name is registered.
#[allow(dead_code)]
pub fn nt_has_name(nt: &NameTable, name: &str) -> bool {
    nt.name_to_id.contains_key(name)
}

/// Whether an ID is registered.
#[allow(dead_code)]
pub fn nt_has_id(nt: &NameTable, id: u32) -> bool {
    nt.id_to_name.contains_key(&id)
}

/// Unregister a name; returns the ID if it existed.
#[allow(dead_code)]
pub fn nt_unregister(nt: &mut NameTable, name: &str) -> Option<u32> {
    if let Some(id) = nt.name_to_id.remove(name) {
        nt.id_to_name.remove(&id);
        Some(id)
    } else {
        None
    }
}

/// Number of registered names.
#[allow(dead_code)]
pub fn nt_count(nt: &NameTable) -> usize {
    nt.name_to_id.len()
}

/// All registered names.
#[allow(dead_code)]
pub fn nt_names(nt: &NameTable) -> Vec<&str> {
    nt.name_to_id.keys().map(|s| s.as_str()).collect()
}

/// Clear all entries.
#[allow(dead_code)]
pub fn nt_clear(nt: &mut NameTable) {
    nt.name_to_id.clear();
    nt.id_to_name.clear();
}

/// Rename: update name for an existing ID.
#[allow(dead_code)]
pub fn nt_rename(nt: &mut NameTable, id: u32, new_name: &str) -> bool {
    if let Some(old_name) = nt.id_to_name.get(&id).cloned() {
        if nt.name_to_id.contains_key(new_name) {
            return false; // name conflict
        }
        nt.name_to_id.remove(&old_name);
        nt.name_to_id.insert(new_name.to_string(), id);
        nt.id_to_name.insert(id, new_name.to_string());
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut nt = new_name_table();
        let id = nt_register(&mut nt, "bone_arm");
        assert_eq!(nt_id(&nt, "bone_arm"), Some(id));
        assert_eq!(nt_name(&nt, id), Some("bone_arm"));
    }

    #[test]
    fn test_idempotent_register() {
        let mut nt = new_name_table();
        let a = nt_register(&mut nt, "x");
        let b = nt_register(&mut nt, "x");
        assert_eq!(a, b);
        assert_eq!(nt_count(&nt), 1);
    }

    #[test]
    fn test_has_name_and_id() {
        let mut nt = new_name_table();
        let id = nt_register(&mut nt, "mesh");
        assert!(nt_has_name(&nt, "mesh"));
        assert!(nt_has_id(&nt, id));
    }

    #[test]
    fn test_unregister() {
        let mut nt = new_name_table();
        let id = nt_register(&mut nt, "tmp");
        let removed = nt_unregister(&mut nt, "tmp");
        assert_eq!(removed, Some(id));
        assert!(!nt_has_name(&nt, "tmp"));
    }

    #[test]
    fn test_count() {
        let mut nt = new_name_table();
        nt_register(&mut nt, "a");
        nt_register(&mut nt, "b");
        assert_eq!(nt_count(&nt), 2);
    }

    #[test]
    fn test_clear() {
        let mut nt = new_name_table();
        nt_register(&mut nt, "x");
        nt_clear(&mut nt);
        assert_eq!(nt_count(&nt), 0);
    }

    #[test]
    fn test_rename() {
        let mut nt = new_name_table();
        let id = nt_register(&mut nt, "old");
        assert!(nt_rename(&mut nt, id, "new"));
        assert_eq!(nt_name(&nt, id), Some("new"));
        assert!(!nt_has_name(&nt, "old"));
    }

    #[test]
    fn test_rename_conflict() {
        let mut nt = new_name_table();
        let id = nt_register(&mut nt, "a");
        nt_register(&mut nt, "b");
        assert!(!nt_rename(&mut nt, id, "b"));
    }

    #[test]
    fn test_names_list() {
        let mut nt = new_name_table();
        nt_register(&mut nt, "alpha");
        nt_register(&mut nt, "beta");
        assert_eq!(nt_names(&nt).len(), 2);
    }
}
