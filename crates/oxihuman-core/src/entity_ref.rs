// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A generational entity reference that combines an index with a generation
//! counter to detect stale references.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityRef {
    index: u32,
    generation: u32,
}

#[allow(dead_code)]
impl EntityRef {
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub fn index(self) -> u32 {
        self.index
    }

    pub fn generation(self) -> u32 {
        self.generation
    }

    pub fn to_bits(self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }

    pub fn from_bits(bits: u64) -> Self {
        Self {
            index: bits as u32,
            generation: (bits >> 32) as u32,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Slot {
    generation: u32,
    alive: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EntityArena {
    slots: Vec<Slot>,
    free_list: Vec<u32>,
}

#[allow(dead_code)]
impl EntityArena {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> EntityRef {
        if let Some(idx) = self.free_list.pop() {
            let slot = &mut self.slots[idx as usize];
            slot.generation += 1;
            slot.alive = true;
            EntityRef::new(idx, slot.generation)
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Slot { generation: 0, alive: true });
            EntityRef::new(idx, 0)
        }
    }

    pub fn free(&mut self, entity: EntityRef) -> bool {
        let idx = entity.index() as usize;
        if idx < self.slots.len()
            && self.slots[idx].generation == entity.generation()
            && self.slots[idx].alive
        {
            self.slots[idx].alive = false;
            self.free_list.push(entity.index());
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: EntityRef) -> bool {
        let idx = entity.index() as usize;
        idx < self.slots.len()
            && self.slots[idx].generation == entity.generation()
            && self.slots[idx].alive
    }

    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.alive).count()
    }

    pub fn total_slots(&self) -> usize {
        self.slots.len()
    }
}

impl Default for EntityArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate() {
        let mut arena = EntityArena::new();
        let e = arena.allocate();
        assert_eq!(e.index(), 0);
        assert_eq!(e.generation(), 0);
    }

    #[test]
    fn test_is_alive() {
        let mut arena = EntityArena::new();
        let e = arena.allocate();
        assert!(arena.is_alive(e));
    }

    #[test]
    fn test_free_invalidates() {
        let mut arena = EntityArena::new();
        let e = arena.allocate();
        arena.free(e);
        assert!(!arena.is_alive(e));
    }

    #[test]
    fn test_reuse_slot_bumps_generation() {
        let mut arena = EntityArena::new();
        let e1 = arena.allocate();
        arena.free(e1);
        let e2 = arena.allocate();
        assert_eq!(e2.index(), e1.index());
        assert_eq!(e2.generation(), 1);
        assert!(!arena.is_alive(e1));
        assert!(arena.is_alive(e2));
    }

    #[test]
    fn test_active_count() {
        let mut arena = EntityArena::new();
        arena.allocate();
        arena.allocate();
        assert_eq!(arena.active_count(), 2);
        let e = arena.allocate();
        arena.free(e);
        assert_eq!(arena.active_count(), 2);
    }

    #[test]
    fn test_to_from_bits() {
        let e = EntityRef::new(42, 7);
        let bits = e.to_bits();
        let e2 = EntityRef::from_bits(bits);
        assert_eq!(e, e2);
    }

    #[test]
    fn test_free_stale_ref() {
        let mut arena = EntityArena::new();
        let e = arena.allocate();
        arena.free(e);
        assert!(!arena.free(e)); // double-free returns false
    }

    #[test]
    fn test_total_slots() {
        let mut arena = EntityArena::new();
        arena.allocate();
        arena.allocate();
        assert_eq!(arena.total_slots(), 2);
    }

    #[test]
    fn test_multiple_generations() {
        let mut arena = EntityArena::new();
        for _ in 0..5 {
            let e = arena.allocate();
            arena.free(e);
        }
        let e = arena.allocate();
        assert_eq!(e.generation(), 5);
    }

    #[test]
    fn test_entity_ref_eq() {
        let a = EntityRef::new(1, 2);
        let b = EntityRef::new(1, 2);
        let c = EntityRef::new(1, 3);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
