// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Arena/bump allocator stub with alignment support.

#![allow(dead_code)]

/// A bump/arena allocator with alignment-aware allocation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArenaAllocStub {
    pub data: Vec<u8>,
    pub offset: usize,
    pub capacity: usize,
}

/// Create a new arena with the given capacity in bytes.
#[allow(dead_code)]
pub fn new_arena_stub(capacity: usize) -> ArenaAllocStub {
    ArenaAllocStub {
        data: vec![0u8; capacity],
        offset: 0,
        capacity,
    }
}

/// Allocate `size` bytes with alignment `align`. Returns byte offset or None if out of space.
#[allow(dead_code)]
pub fn arena_alloc_stub(arena: &mut ArenaAllocStub, size: usize, align: usize) -> Option<usize> {
    let align = align.max(1);
    // Round up offset to alignment
    let aligned_offset = (arena.offset + align - 1) & !(align - 1);
    if aligned_offset + size > arena.capacity {
        return None;
    }
    arena.offset = aligned_offset + size;
    Some(aligned_offset)
}

/// Reset the arena (free all allocations).
#[allow(dead_code)]
pub fn arena_reset_stub(arena: &mut ArenaAllocStub) {
    arena.offset = 0;
}

/// Return the number of bytes used.
#[allow(dead_code)]
pub fn arena_used_stub(arena: &ArenaAllocStub) -> usize {
    arena.offset
}

/// Return the number of bytes remaining.
#[allow(dead_code)]
pub fn arena_remaining_stub(arena: &ArenaAllocStub) -> usize {
    arena.capacity.saturating_sub(arena.offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_arena_stub() {
        let a = new_arena_stub(1024);
        assert_eq!(a.capacity, 1024);
        assert_eq!(a.offset, 0);
    }

    #[test]
    fn test_alloc_basic() {
        let mut a = new_arena_stub(256);
        let off = arena_alloc_stub(&mut a, 16, 1);
        assert_eq!(off, Some(0));
        assert_eq!(arena_used_stub(&a), 16);
    }

    #[test]
    fn test_alloc_with_alignment() {
        let mut a = new_arena_stub(256);
        arena_alloc_stub(&mut a, 1, 1); // offset is now 1
        let off = arena_alloc_stub(&mut a, 4, 4); // should align to 4
        assert_eq!(off, Some(4));
    }

    #[test]
    fn test_alloc_out_of_space() {
        let mut a = new_arena_stub(8);
        let off = arena_alloc_stub(&mut a, 16, 1);
        assert!(off.is_none());
    }

    #[test]
    fn test_reset() {
        let mut a = new_arena_stub(64);
        arena_alloc_stub(&mut a, 32, 1);
        arena_reset_stub(&mut a);
        assert_eq!(arena_used_stub(&a), 0);
    }

    #[test]
    fn test_remaining() {
        let mut a = new_arena_stub(100);
        arena_alloc_stub(&mut a, 40, 1);
        assert_eq!(arena_remaining_stub(&a), 60);
    }

    #[test]
    fn test_sequential_allocs() {
        let mut a = new_arena_stub(64);
        let o1 = arena_alloc_stub(&mut a, 8, 1).expect("should succeed");
        let o2 = arena_alloc_stub(&mut a, 8, 1).expect("should succeed");
        assert!(o2 > o1);
    }

    #[test]
    fn test_zero_capacity() {
        let mut a = new_arena_stub(0);
        assert!(arena_alloc_stub(&mut a, 1, 1).is_none());
    }

    #[test]
    fn test_used_after_alloc() {
        let mut a = new_arena_stub(128);
        arena_alloc_stub(&mut a, 50, 1);
        assert_eq!(arena_used_stub(&a), 50);
    }
}
