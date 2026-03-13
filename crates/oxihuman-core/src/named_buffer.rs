// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Named byte buffer registry — store and retrieve byte buffers by name.

use std::collections::HashMap;

/// Registry of named byte buffers.
#[derive(Debug, Default, Clone)]
pub struct NamedBufferRegistry {
    buffers: HashMap<String, Vec<u8>>,
}

impl NamedBufferRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        NamedBufferRegistry { buffers: HashMap::new() }
    }

    /// Store or replace a buffer under `name`.
    pub fn store(&mut self, name: impl Into<String>, data: Vec<u8>) {
        self.buffers.insert(name.into(), data);
    }

    /// Retrieve a reference to the buffer named `name`.
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.buffers.get(name).map(|v| v.as_slice())
    }

    /// Retrieve a mutable reference to the buffer named `name`.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Vec<u8>> {
        self.buffers.get_mut(name)
    }

    /// Remove and return the buffer named `name`.
    pub fn remove(&mut self, name: &str) -> Option<Vec<u8>> {
        self.buffers.remove(name)
    }

    /// Return the number of registered buffers.
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// True if no buffers are registered.
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Returns an iterator over buffer names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.buffers.keys().map(|s| s.as_str())
    }

    /// Total byte size of all buffers combined.
    pub fn total_bytes(&self) -> usize {
        self.buffers.values().map(|v| v.len()).sum()
    }
}

/// Create a new named buffer registry.
pub fn new_named_buffer_registry() -> NamedBufferRegistry {
    NamedBufferRegistry::new()
}

/// Store a buffer in the registry.
pub fn nb_store(reg: &mut NamedBufferRegistry, name: &str, data: Vec<u8>) {
    reg.store(name, data);
}

/// Retrieve a buffer slice from the registry.
pub fn nb_get<'a>(reg: &'a NamedBufferRegistry, name: &str) -> Option<&'a [u8]> {
    reg.get(name)
}

/// Remove a buffer from the registry.
pub fn nb_remove(reg: &mut NamedBufferRegistry, name: &str) -> Option<Vec<u8>> {
    reg.remove(name)
}

/// Total bytes stored across all buffers.
pub fn nb_total_bytes(reg: &NamedBufferRegistry) -> usize {
    reg.total_bytes()
}

/// Length (number of buffers).
pub fn nb_len(reg: &NamedBufferRegistry) -> usize {
    reg.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_get() {
        let mut r = new_named_buffer_registry();
        nb_store(&mut r, "buf1", vec![1, 2, 3]);
        assert_eq!(nb_get(&r, "buf1"), Some([1u8, 2, 3].as_slice()) /* stored */);
    }

    #[test]
    fn test_get_missing() {
        let r = new_named_buffer_registry();
        assert_eq!(nb_get(&r, "nope"), None /* not found */);
    }

    #[test]
    fn test_remove() {
        let mut r = new_named_buffer_registry();
        nb_store(&mut r, "x", vec![42]);
        let removed = nb_remove(&mut r, "x").expect("should succeed");
        assert_eq!(removed, vec![42u8] /* removed data */);
        assert!(r.is_empty());
    }

    #[test]
    fn test_len() {
        let mut r = new_named_buffer_registry();
        nb_store(&mut r, "a", vec![]);
        nb_store(&mut r, "b", vec![]);
        assert_eq!(nb_len(&r), 2 /* two buffers */);
    }

    #[test]
    fn test_total_bytes() {
        let mut r = new_named_buffer_registry();
        nb_store(&mut r, "a", vec![1, 2, 3]);
        nb_store(&mut r, "b", vec![4, 5]);
        assert_eq!(nb_total_bytes(&r), 5 /* 3 + 2 */);
    }

    #[test]
    fn test_overwrite() {
        let mut r = new_named_buffer_registry();
        nb_store(&mut r, "k", vec![0]);
        nb_store(&mut r, "k", vec![7, 8]);
        assert_eq!(nb_get(&r, "k"), Some([7u8, 8].as_slice()) /* overwritten */);
        assert_eq!(nb_len(&r), 1);
    }

    #[test]
    fn test_names() {
        let mut r = new_named_buffer_registry();
        nb_store(&mut r, "alpha", vec![]);
        let names: Vec<&str> = r.names().collect();
        assert!(names.contains(&"alpha") /* name present */);
    }

    #[test]
    fn test_get_mut() {
        let mut r = new_named_buffer_registry();
        nb_store(&mut r, "m", vec![1, 2]);
        if let Some(buf) = r.get_mut("m") {
            buf.push(3);
        }
        assert_eq!(nb_get(&r, "m"), Some([1u8, 2, 3].as_slice()) /* mutated */);
    }

    #[test]
    fn test_empty_registry() {
        let r = new_named_buffer_registry();
        assert!(r.is_empty() /* starts empty */);
        assert_eq!(nb_total_bytes(&r), 0);
    }

    #[test]
    fn test_remove_missing() {
        let mut r = new_named_buffer_registry();
        assert_eq!(nb_remove(&mut r, "ghost"), None /* nothing to remove */);
    }
}
