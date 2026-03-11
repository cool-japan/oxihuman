// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Custom data block management for mesh elements.

/// Type tag for custom data values.
#[derive(Clone, PartialEq)]
pub enum CustomDataValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Vec3([f32; 3]),
}

/// A custom data entry: name → value.
pub struct CustomDataEntry {
    pub key: String,
    pub value: CustomDataValue,
}

/// A custom data block for a single mesh element (vertex, face, etc.).
pub struct CustomDataBlock {
    pub entries: Vec<CustomDataEntry>,
}

/// Create a new empty custom data block.
pub fn new_custom_data_block() -> CustomDataBlock {
    CustomDataBlock {
        entries: Vec::new(),
    }
}

/// Set a custom data entry (insert or overwrite).
pub fn set_custom_entry(block: &mut CustomDataBlock, key: &str, value: CustomDataValue) {
    if let Some(e) = block.entries.iter_mut().find(|e| e.key == key) {
        e.value = value;
    } else {
        block.entries.push(CustomDataEntry {
            key: key.to_string(),
            value,
        });
    }
}

/// Get a reference to a custom data value by key.
pub fn get_custom_entry<'a>(block: &'a CustomDataBlock, key: &str) -> Option<&'a CustomDataValue> {
    block
        .entries
        .iter()
        .find(|e| e.key == key)
        .map(|e| &e.value)
}

/// Remove a custom data entry; returns true if removed.
pub fn remove_custom_entry(block: &mut CustomDataBlock, key: &str) -> bool {
    if let Some(pos) = block.entries.iter().position(|e| e.key == key) {
        block.entries.remove(pos);
        true
    } else {
        false
    }
}

/// Number of entries in the block.
pub fn custom_entry_count(block: &CustomDataBlock) -> usize {
    block.entries.len()
}

/// Clear all custom data.
pub fn clear_custom_data(block: &mut CustomDataBlock) {
    block.entries.clear();
}

/// List all keys in the block.
pub fn list_custom_keys(block: &CustomDataBlock) -> Vec<&str> {
    block.entries.iter().map(|e| e.key.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_block_empty() {
        let b = new_custom_data_block();
        assert_eq!(custom_entry_count(&b), 0 /* empty */);
    }

    #[test]
    fn set_and_get_float() {
        let mut b = new_custom_data_block();
        set_custom_entry(&mut b, "speed", CustomDataValue::Float(2.71));
        let v = get_custom_entry(&b, "speed");
        assert!(v.is_some() /* found */);
        if let CustomDataValue::Float(f) = v.unwrap() {
            assert!((f - 2.71).abs() < 1e-5 /* correct value */);
        }
    }

    #[test]
    fn overwrite_updates_value() {
        let mut b = new_custom_data_block();
        set_custom_entry(&mut b, "x", CustomDataValue::Int(1));
        set_custom_entry(&mut b, "x", CustomDataValue::Int(2));
        assert_eq!(custom_entry_count(&b), 1 /* no duplicate */);
        if let CustomDataValue::Int(i) = get_custom_entry(&b, "x").unwrap() {
            assert_eq!(*i, 2 /* updated */);
        }
    }

    #[test]
    fn get_missing_none() {
        let b = new_custom_data_block();
        assert!(get_custom_entry(&b, "nope").is_none() /* missing */);
    }

    #[test]
    fn remove_entry_works() {
        let mut b = new_custom_data_block();
        set_custom_entry(&mut b, "rm", CustomDataValue::Bool(true));
        let ok = remove_custom_entry(&mut b, "rm");
        assert!(ok /* removed */);
        assert_eq!(custom_entry_count(&b), 0 /* empty */);
    }

    #[test]
    fn remove_missing_false() {
        let mut b = new_custom_data_block();
        assert!(!remove_custom_entry(&mut b, "nothing") /* not found */);
    }

    #[test]
    fn list_keys_correct() {
        let mut b = new_custom_data_block();
        set_custom_entry(&mut b, "a", CustomDataValue::Bool(false));
        set_custom_entry(&mut b, "b", CustomDataValue::Float(1.0));
        let keys = list_custom_keys(&b);
        assert!(keys.contains(&"a") /* has a */);
        assert!(keys.contains(&"b") /* has b */);
    }

    #[test]
    fn clear_empties_block() {
        let mut b = new_custom_data_block();
        set_custom_entry(&mut b, "x", CustomDataValue::Int(0));
        clear_custom_data(&mut b);
        assert_eq!(custom_entry_count(&b), 0 /* cleared */);
    }

    #[test]
    fn vec3_entry_stored() {
        let mut b = new_custom_data_block();
        set_custom_entry(&mut b, "normal", CustomDataValue::Vec3([0.0, 1.0, 0.0]));
        if let CustomDataValue::Vec3(v) = get_custom_entry(&b, "normal").unwrap() {
            assert!((v[1] - 1.0).abs() < 1e-6 /* Y = 1 */);
        }
    }
}
