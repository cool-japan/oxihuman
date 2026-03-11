#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Narrow phase collision detection dispatcher registry.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NarrowEntry {
    pub shape_type_a: u8,
    pub shape_type_b: u8,
    pub handler_id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct NarrowPhaseRegistry {
    pub entries: Vec<NarrowEntry>,
}

#[allow(dead_code)]
pub fn new_narrow_phase_registry() -> NarrowPhaseRegistry {
    NarrowPhaseRegistry { entries: Vec::new() }
}

#[allow(dead_code)]
pub fn register_handler(reg: &mut NarrowPhaseRegistry, ta: u8, tb: u8, handler: u32) {
    // Replace if already registered for (ta, tb)
    if let Some(e) = reg.entries.iter_mut().find(|e| e.shape_type_a == ta && e.shape_type_b == tb) {
        e.handler_id = handler;
    } else {
        reg.entries.push(NarrowEntry { shape_type_a: ta, shape_type_b: tb, handler_id: handler });
    }
}

#[allow(dead_code)]
pub fn find_handler(reg: &NarrowPhaseRegistry, ta: u8, tb: u8) -> Option<u32> {
    reg.entries
        .iter()
        .find(|e| (e.shape_type_a == ta && e.shape_type_b == tb)
            || (e.shape_type_a == tb && e.shape_type_b == ta))
        .map(|e| e.handler_id)
}

#[allow(dead_code)]
pub fn handler_count(reg: &NarrowPhaseRegistry) -> usize {
    reg.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let reg = new_narrow_phase_registry();
        assert_eq!(handler_count(&reg), 0);
    }

    #[test]
    fn test_register_one() {
        let mut reg = new_narrow_phase_registry();
        register_handler(&mut reg, 0, 1, 100);
        assert_eq!(handler_count(&reg), 1);
    }

    #[test]
    fn test_find_registered() {
        let mut reg = new_narrow_phase_registry();
        register_handler(&mut reg, 0, 1, 100);
        assert_eq!(find_handler(&reg, 0, 1), Some(100));
    }

    #[test]
    fn test_find_swapped() {
        let mut reg = new_narrow_phase_registry();
        register_handler(&mut reg, 0, 1, 100);
        assert_eq!(find_handler(&reg, 1, 0), Some(100));
    }

    #[test]
    fn test_find_missing() {
        let reg = new_narrow_phase_registry();
        assert_eq!(find_handler(&reg, 0, 1), None);
    }

    #[test]
    fn test_register_multiple() {
        let mut reg = new_narrow_phase_registry();
        register_handler(&mut reg, 0, 1, 10);
        register_handler(&mut reg, 1, 2, 20);
        register_handler(&mut reg, 0, 2, 30);
        assert_eq!(handler_count(&reg), 3);
    }

    #[test]
    fn test_register_replace() {
        let mut reg = new_narrow_phase_registry();
        register_handler(&mut reg, 0, 1, 10);
        register_handler(&mut reg, 0, 1, 99);
        assert_eq!(handler_count(&reg), 1);
        assert_eq!(find_handler(&reg, 0, 1), Some(99));
    }

    #[test]
    fn test_handler_id_correct() {
        let mut reg = new_narrow_phase_registry();
        register_handler(&mut reg, 5, 6, 42);
        assert_eq!(find_handler(&reg, 5, 6), Some(42));
    }

    #[test]
    fn test_multiple_find() {
        let mut reg = new_narrow_phase_registry();
        register_handler(&mut reg, 0, 1, 10);
        register_handler(&mut reg, 2, 3, 20);
        assert_eq!(find_handler(&reg, 2, 3), Some(20));
        assert_eq!(find_handler(&reg, 0, 1), Some(10));
    }
}
