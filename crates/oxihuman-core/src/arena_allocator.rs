// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bump/arena allocator stub.

#![allow(dead_code)]

/// Configuration for an arena allocator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArenaConfig {
    pub chunk_size: usize,
}

/// A bump/arena allocator backed by a Vec<u8>.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArenaAllocator {
    pub buf: Vec<u8>,
    pub used: usize,
    alloc_count: usize,
}

/// Statistics about the arena.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArenaStats {
    pub used: usize,
    pub capacity: usize,
    pub alloc_count: usize,
}

/// Return the default arena configuration.
#[allow(dead_code)]
pub fn default_arena_config() -> ArenaConfig {
    ArenaConfig { chunk_size: 4096 }
}

/// Create a new arena allocator.
#[allow(dead_code)]
pub fn new_arena(config: &ArenaConfig) -> ArenaAllocator {
    ArenaAllocator {
        buf: vec![0u8; config.chunk_size],
        used: 0,
        alloc_count: 0,
    }
}

/// Bump-allocate `size` bytes. Returns the byte offset into `buf`, or `None` if out of space.
#[allow(dead_code)]
pub fn arena_alloc_bytes(arena: &mut ArenaAllocator, size: usize) -> Option<usize> {
    if arena.used + size > arena.buf.len() {
        return None;
    }
    let offset = arena.used;
    arena.used += size;
    arena.alloc_count += 1;
    Some(offset)
}

/// Reset the arena (frees all allocations at once).
#[allow(dead_code)]
pub fn arena_reset(arena: &mut ArenaAllocator) {
    arena.used = 0;
    arena.alloc_count = 0;
}

/// Return arena statistics.
#[allow(dead_code)]
pub fn arena_stats(arena: &ArenaAllocator) -> ArenaStats {
    ArenaStats {
        used: arena.used,
        capacity: arena.buf.len(),
        alloc_count: arena.alloc_count,
    }
}

/// Return the number of bytes remaining.
#[allow(dead_code)]
pub fn arena_remaining(arena: &ArenaAllocator) -> usize {
    arena.buf.len().saturating_sub(arena.used)
}

/// Return the usage ratio (0.0 = empty, 1.0 = full).
#[allow(dead_code)]
pub fn arena_usage_ratio(arena: &ArenaAllocator) -> f32 {
    if arena.buf.is_empty() {
        return 1.0;
    }
    arena.used as f32 / arena.buf.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_arena_config();
        assert!(cfg.chunk_size > 0);
    }

    #[test]
    fn test_new_arena_empty() {
        let cfg = ArenaConfig { chunk_size: 1024 };
        let arena = new_arena(&cfg);
        assert_eq!(arena.used, 0);
        assert_eq!(arena.buf.len(), 1024);
    }

    #[test]
    fn test_alloc_bytes() {
        let cfg = ArenaConfig { chunk_size: 256 };
        let mut arena = new_arena(&cfg);
        let off = arena_alloc_bytes(&mut arena, 64);
        assert_eq!(off, Some(0));
        assert_eq!(arena.used, 64);
    }

    #[test]
    fn test_alloc_sequential() {
        let cfg = ArenaConfig { chunk_size: 256 };
        let mut arena = new_arena(&cfg);
        let off1 = arena_alloc_bytes(&mut arena, 32).unwrap();
        let off2 = arena_alloc_bytes(&mut arena, 32).unwrap();
        assert_eq!(off1, 0);
        assert_eq!(off2, 32);
    }

    #[test]
    fn test_alloc_out_of_space() {
        let cfg = ArenaConfig { chunk_size: 10 };
        let mut arena = new_arena(&cfg);
        assert!(arena_alloc_bytes(&mut arena, 20).is_none());
    }

    #[test]
    fn test_reset() {
        let cfg = ArenaConfig { chunk_size: 64 };
        let mut arena = new_arena(&cfg);
        arena_alloc_bytes(&mut arena, 32);
        arena_reset(&mut arena);
        assert_eq!(arena.used, 0);
        let stats = arena_stats(&arena);
        assert_eq!(stats.alloc_count, 0);
    }

    #[test]
    fn test_remaining() {
        let cfg = ArenaConfig { chunk_size: 100 };
        let mut arena = new_arena(&cfg);
        arena_alloc_bytes(&mut arena, 40);
        assert_eq!(arena_remaining(&arena), 60);
    }

    #[test]
    fn test_usage_ratio() {
        let cfg = ArenaConfig { chunk_size: 100 };
        let mut arena = new_arena(&cfg);
        arena_alloc_bytes(&mut arena, 50);
        let ratio = arena_usage_ratio(&arena);
        assert!((ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_stats() {
        let cfg = ArenaConfig { chunk_size: 128 };
        let mut arena = new_arena(&cfg);
        arena_alloc_bytes(&mut arena, 10);
        arena_alloc_bytes(&mut arena, 20);
        let stats = arena_stats(&arena);
        assert_eq!(stats.alloc_count, 2);
        assert_eq!(stats.used, 30);
        assert_eq!(stats.capacity, 128);
    }
}
