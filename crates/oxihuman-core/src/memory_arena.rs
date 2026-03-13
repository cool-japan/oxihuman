#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bump-allocator style memory arena.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MemoryArena {
    capacity: usize,
    used: usize,
    alloc_count: usize,
}

#[allow(dead_code)]
pub fn new_memory_arena(capacity: usize) -> MemoryArena {
    MemoryArena {
        capacity,
        used: 0,
        alloc_count: 0,
    }
}

#[allow(dead_code)]
pub fn arena_alloc_bytes(arena: &mut MemoryArena, bytes: usize) -> Option<usize> {
    if arena.used + bytes <= arena.capacity {
        let offset = arena.used;
        arena.used += bytes;
        arena.alloc_count += 1;
        Some(offset)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn arena_used(arena: &MemoryArena) -> usize {
    arena.used
}

#[allow(dead_code)]
pub fn arena_capacity_ma(arena: &MemoryArena) -> usize {
    arena.capacity
}

#[allow(dead_code)]
pub fn arena_reset(arena: &mut MemoryArena) {
    arena.used = 0;
    arena.alloc_count = 0;
}

#[allow(dead_code)]
pub fn arena_alloc_count(arena: &MemoryArena) -> usize {
    arena.alloc_count
}

#[allow(dead_code)]
pub fn arena_to_json(arena: &MemoryArena) -> String {
    format!(
        r#"{{"capacity":{},"used":{},"allocs":{}}}"#,
        arena.capacity, arena.used, arena.alloc_count
    )
}

#[allow(dead_code)]
pub fn arena_is_empty(arena: &MemoryArena) -> bool {
    arena.used == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_arena() {
        let a = new_memory_arena(1024);
        assert_eq!(arena_capacity_ma(&a), 1024);
        assert!(arena_is_empty(&a));
    }

    #[test]
    fn test_alloc() {
        let mut a = new_memory_arena(1024);
        let off = arena_alloc_bytes(&mut a, 64);
        assert_eq!(off, Some(0));
        assert_eq!(arena_used(&a), 64);
    }

    #[test]
    fn test_alloc_full() {
        let mut a = new_memory_arena(32);
        assert!(arena_alloc_bytes(&mut a, 32).is_some());
        assert!(arena_alloc_bytes(&mut a, 1).is_none());
    }

    #[test]
    fn test_alloc_count() {
        let mut a = new_memory_arena(1024);
        arena_alloc_bytes(&mut a, 10);
        arena_alloc_bytes(&mut a, 20);
        assert_eq!(arena_alloc_count(&a), 2);
    }

    #[test]
    fn test_reset() {
        let mut a = new_memory_arena(1024);
        arena_alloc_bytes(&mut a, 100);
        arena_reset(&mut a);
        assert!(arena_is_empty(&a));
        assert_eq!(arena_alloc_count(&a), 0);
    }

    #[test]
    fn test_arena_to_json() {
        let a = new_memory_arena(512);
        let json = arena_to_json(&a);
        assert!(json.contains("\"capacity\":512"));
    }

    #[test]
    fn test_sequential_alloc() {
        let mut a = new_memory_arena(1024);
        let o1 = arena_alloc_bytes(&mut a, 10).expect("should succeed");
        let o2 = arena_alloc_bytes(&mut a, 20).expect("should succeed");
        assert_eq!(o1, 0);
        assert_eq!(o2, 10);
    }

    #[test]
    fn test_is_empty_after_alloc() {
        let mut a = new_memory_arena(1024);
        arena_alloc_bytes(&mut a, 1);
        assert!(!arena_is_empty(&a));
    }

    #[test]
    fn test_zero_capacity() {
        let mut a = new_memory_arena(0);
        assert!(arena_alloc_bytes(&mut a, 1).is_none());
    }

    #[test]
    fn test_exact_fit() {
        let mut a = new_memory_arena(100);
        assert!(arena_alloc_bytes(&mut a, 100).is_some());
        assert_eq!(arena_used(&a), 100);
    }
}
