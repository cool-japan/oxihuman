// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Arena allocator for objects — bump allocates into a typed Vec; reset
//! reclaims all memory at once without per-object destruction overhead.

/// Generation tag to invalidate stale handles after an arena reset.
pub type Generation = u32;

/// Handle to an arena-allocated object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArenaHandle {
    pub index: usize,
    pub generation: Generation,
}

/// Arena allocator that bump-allocates objects of type `T`.
pub struct ObjectArena<T> {
    data: Vec<T>,
    generation: Generation,
}

impl<T> ObjectArena<T> {
    /// Create an empty arena with a pre-allocated hint.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
            generation: 0,
        }
    }

    /// Allocate an object and return its handle.
    pub fn alloc(&mut self, value: T) -> ArenaHandle {
        let index = self.data.len();
        self.data.push(value);
        ArenaHandle {
            index,
            generation: self.generation,
        }
    }

    /// Get a reference, validating the generation.
    pub fn get(&self, handle: ArenaHandle) -> Option<&T> {
        if handle.generation == self.generation {
            self.data.get(handle.index)
        } else {
            None
        }
    }

    /// Number of live objects in the arena.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Reset the arena, invalidating all existing handles.
    pub fn reset(&mut self) {
        self.data.clear();
        self.generation = self.generation.wrapping_add(1);
    }

    /// Current generation counter.
    pub fn generation(&self) -> Generation {
        self.generation
    }
}

/// Create an arena with the given capacity hint.
pub fn new_object_arena<T>(cap: usize) -> ObjectArena<T> {
    ObjectArena::with_capacity(cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_and_get() {
        let mut arena: ObjectArena<i32> = ObjectArena::with_capacity(8);
        let h = arena.alloc(42);
        assert_eq!(arena.get(h), Some(&42)); /* round-trip ok */
    }

    #[test]
    fn test_reset_invalidates_handles() {
        let mut arena: ObjectArena<i32> = ObjectArena::with_capacity(8);
        let h = arena.alloc(1);
        arena.reset();
        assert_eq!(arena.get(h), None); /* stale handle rejected */
    }

    #[test]
    fn test_len_after_alloc() {
        let mut arena: ObjectArena<u8> = ObjectArena::with_capacity(4);
        arena.alloc(1);
        arena.alloc(2);
        assert_eq!(arena.len(), 2); /* two objects stored */
    }

    #[test]
    fn test_is_empty_initial() {
        let arena: ObjectArena<i32> = ObjectArena::with_capacity(4);
        assert!(arena.is_empty()); /* fresh arena is empty */
    }

    #[test]
    fn test_is_empty_after_reset() {
        let mut arena: ObjectArena<i32> = ObjectArena::with_capacity(4);
        arena.alloc(5);
        arena.reset();
        assert!(arena.is_empty()); /* reset clears data */
    }

    #[test]
    fn test_generation_increments() {
        let mut arena: ObjectArena<i32> = ObjectArena::with_capacity(4);
        let g0 = arena.generation();
        arena.reset();
        assert_eq!(arena.generation(), g0 + 1); /* generation advances */
    }

    #[test]
    fn test_multiple_allocs() {
        let mut arena: ObjectArena<f32> = ObjectArena::with_capacity(16);
        let handles: Vec<_> = (0..8).map(|i| arena.alloc(i as f32)).collect();
        for (i, h) in handles.iter().enumerate() {
            assert_eq!(arena.get(*h), Some(&(i as f32))); /* each value correct */
        }
    }

    #[test]
    fn test_new_helper() {
        let arena = new_object_arena::<u64>(32);
        assert!(arena.is_empty()); /* helper creates empty arena */
    }

    #[test]
    fn test_handle_index() {
        let mut arena: ObjectArena<i32> = ObjectArena::with_capacity(4);
        let h = arena.alloc(7);
        assert_eq!(h.index, 0); /* first alloc at index 0 */
    }
}
