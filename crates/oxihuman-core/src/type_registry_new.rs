// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Type name registry — maps type IDs to metadata.

#![allow(dead_code)]

use std::collections::HashMap;

/// Metadata for a registered type.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub id: u32,
    pub name: String,
    pub size_bytes: usize,
}

/// Registry mapping type IDs and names to TypeInfo.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TypeInfoRegistry {
    by_id: HashMap<u32, TypeInfo>,
    name_to_id: HashMap<String, u32>,
}

/// Create a new empty type registry.
#[allow(dead_code)]
pub fn new_type_info_registry() -> TypeInfoRegistry {
    TypeInfoRegistry {
        by_id: HashMap::new(),
        name_to_id: HashMap::new(),
    }
}

/// Register a type. Returns false if the id or name is already in use.
#[allow(dead_code)]
pub fn type_info_register(reg: &mut TypeInfoRegistry, info: TypeInfo) -> bool {
    if reg.by_id.contains_key(&info.id) || reg.name_to_id.contains_key(&info.name) {
        return false;
    }
    reg.name_to_id.insert(info.name.clone(), info.id);
    reg.by_id.insert(info.id, info);
    true
}

/// Get a type by id.
#[allow(dead_code)]
pub fn type_info_get_by_id(reg: &TypeInfoRegistry, id: u32) -> Option<&TypeInfo> {
    reg.by_id.get(&id)
}

/// Get a type by name.
#[allow(dead_code)]
pub fn type_info_get_by_name<'a>(reg: &'a TypeInfoRegistry, name: &str) -> Option<&'a TypeInfo> {
    let id = reg.name_to_id.get(name)?;
    reg.by_id.get(id)
}

/// Return the number of registered types.
#[allow(dead_code)]
pub fn type_info_count(reg: &TypeInfoRegistry) -> usize {
    reg.by_id.len()
}

/// Return all registered type IDs.
#[allow(dead_code)]
pub fn type_info_ids(reg: &TypeInfoRegistry) -> Vec<u32> {
    reg.by_id.keys().copied().collect()
}

/// Return all registered type names.
#[allow(dead_code)]
pub fn type_info_names(reg: &TypeInfoRegistry) -> Vec<String> {
    reg.name_to_id.keys().cloned().collect()
}

/// Remove a type by id.
#[allow(dead_code)]
pub fn type_info_remove(reg: &mut TypeInfoRegistry, id: u32) -> Option<TypeInfo> {
    let info = reg.by_id.remove(&id)?;
    reg.name_to_id.remove(&info.name);
    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_info(id: u32, name: &str, size: usize) -> TypeInfo {
        TypeInfo { id, name: name.to_string(), size_bytes: size }
    }

    #[test]
    fn test_new_registry_empty() {
        let reg = new_type_info_registry();
        assert_eq!(type_info_count(&reg), 0);
    }

    #[test]
    fn test_register_and_get_by_id() {
        let mut reg = new_type_info_registry();
        type_info_register(&mut reg, make_info(1, "Foo", 8));
        let info = type_info_get_by_id(&reg, 1).expect("should succeed");
        assert_eq!(info.name, "Foo");
        assert_eq!(info.size_bytes, 8);
    }

    #[test]
    fn test_get_by_name() {
        let mut reg = new_type_info_registry();
        type_info_register(&mut reg, make_info(2, "Bar", 4));
        let info = type_info_get_by_name(&reg, "Bar").expect("should succeed");
        assert_eq!(info.id, 2);
    }

    #[test]
    fn test_duplicate_id_rejected() {
        let mut reg = new_type_info_registry();
        assert!(type_info_register(&mut reg, make_info(1, "A", 4)));
        assert!(!type_info_register(&mut reg, make_info(1, "B", 4)));
        assert_eq!(type_info_count(&reg), 1);
    }

    #[test]
    fn test_duplicate_name_rejected() {
        let mut reg = new_type_info_registry();
        type_info_register(&mut reg, make_info(1, "Dup", 4));
        assert!(!type_info_register(&mut reg, make_info(2, "Dup", 8)));
    }

    #[test]
    fn test_ids_and_names() {
        let mut reg = new_type_info_registry();
        type_info_register(&mut reg, make_info(10, "X", 1));
        type_info_register(&mut reg, make_info(20, "Y", 2));
        assert!(type_info_ids(&reg).contains(&10));
        assert!(type_info_names(&reg).contains(&"X".to_string()));
    }

    #[test]
    fn test_remove() {
        let mut reg = new_type_info_registry();
        type_info_register(&mut reg, make_info(5, "Rem", 16));
        let removed = type_info_remove(&mut reg, 5).expect("should succeed");
        assert_eq!(removed.name, "Rem");
        assert_eq!(type_info_count(&reg), 0);
        assert!(type_info_get_by_name(&reg, "Rem").is_none());
    }

    #[test]
    fn test_get_missing_returns_none() {
        let reg = new_type_info_registry();
        assert!(type_info_get_by_id(&reg, 999).is_none());
        assert!(type_info_get_by_name(&reg, "nope").is_none());
    }

    #[test]
    fn test_count_multiple() {
        let mut reg = new_type_info_registry();
        for i in 0..5u32 {
            type_info_register(&mut reg, make_info(i, &format!("T{}", i), 4));
        }
        assert_eq!(type_info_count(&reg), 5);
    }
}
