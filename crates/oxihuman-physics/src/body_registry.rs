// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Registry of physics bodies.

#![allow(dead_code)]

/// A single body entry in the registry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyEntry {
    pub id: u32,
    pub name: String,
    pub active: bool,
    pub sleeping: bool,
}

/// A registry of physics bodies indexed by id.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BodyRegistry {
    pub entries: Vec<BodyEntry>,
    pub next_id: u32,
}

/// Creates a new empty body registry.
#[allow(dead_code)]
pub fn new_body_registry() -> BodyRegistry {
    BodyRegistry {
        entries: Vec::new(),
        next_id: 1,
    }
}

/// Registers a new body and returns its assigned id.
#[allow(dead_code)]
pub fn register_body(reg: &mut BodyRegistry, name: &str) -> u32 {
    let id = reg.next_id;
    reg.next_id += 1;
    reg.entries.push(BodyEntry {
        id,
        name: name.to_string(),
        active: true,
        sleeping: false,
    });
    id
}

/// Removes a body from the registry by id.
#[allow(dead_code)]
pub fn unregister_body(reg: &mut BodyRegistry, id: u32) {
    reg.entries.retain(|e| e.id != id);
}

/// Sets the sleeping state of a body.
#[allow(dead_code)]
pub fn set_sleeping(reg: &mut BodyRegistry, id: u32, sleeping: bool) {
    if let Some(entry) = reg.entries.iter_mut().find(|e| e.id == id) {
        entry.sleeping = sleeping;
    }
}

/// Returns the count of active (non-sleeping) bodies.
#[allow(dead_code)]
pub fn active_count(reg: &BodyRegistry) -> usize {
    reg.entries.iter().filter(|e| e.active && !e.sleeping).count()
}

/// Returns the total number of registered bodies.
#[allow(dead_code)]
pub fn body_count(reg: &BodyRegistry) -> usize {
    reg.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let reg = new_body_registry();
        assert_eq!(body_count(&reg), 0);
        assert_eq!(active_count(&reg), 0);
    }

    #[test]
    fn test_register_increments_count() {
        let mut reg = new_body_registry();
        register_body(&mut reg, "body_a");
        assert_eq!(body_count(&reg), 1);
    }

    #[test]
    fn test_register_returns_unique_ids() {
        let mut reg = new_body_registry();
        let id1 = register_body(&mut reg, "a");
        let id2 = register_body(&mut reg, "b");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_unregister() {
        let mut reg = new_body_registry();
        let id = register_body(&mut reg, "tmp");
        unregister_body(&mut reg, id);
        assert_eq!(body_count(&reg), 0);
    }

    #[test]
    fn test_unregister_nonexistent() {
        let mut reg = new_body_registry();
        register_body(&mut reg, "a");
        unregister_body(&mut reg, 9999);
        assert_eq!(body_count(&reg), 1);
    }

    #[test]
    fn test_set_sleeping() {
        let mut reg = new_body_registry();
        let id = register_body(&mut reg, "body");
        set_sleeping(&mut reg, id, true);
        assert_eq!(active_count(&reg), 0);
    }

    #[test]
    fn test_wake_up() {
        let mut reg = new_body_registry();
        let id = register_body(&mut reg, "body");
        set_sleeping(&mut reg, id, true);
        set_sleeping(&mut reg, id, false);
        assert_eq!(active_count(&reg), 1);
    }

    #[test]
    fn test_active_count_mixed() {
        let mut reg = new_body_registry();
        let id1 = register_body(&mut reg, "a");
        let _id2 = register_body(&mut reg, "b");
        set_sleeping(&mut reg, id1, true);
        assert_eq!(active_count(&reg), 1);
    }

    #[test]
    fn test_body_count_after_multiple() {
        let mut reg = new_body_registry();
        for i in 0..5 {
            register_body(&mut reg, &format!("body_{i}"));
        }
        assert_eq!(body_count(&reg), 5);
    }
}
