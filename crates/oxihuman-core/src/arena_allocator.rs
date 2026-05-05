// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bump/arena allocator with alignment support.

/// Configuration for an arena allocator.
#[derive(Debug, Clone)]
pub struct ArenaConfig {
    pub chunk_size: usize,
}

/// A bump/arena allocator backed by a `Vec<u8>`.
#[derive(Debug, Clone)]
pub struct ArenaAllocator {
    pub buf: Vec<u8>,
    pub used: usize,
    alloc_count: usize,
}

/// Statistics about the arena.
#[derive(Debug, Clone)]
pub struct ArenaStats {
    pub used: usize,
    pub capacity: usize,
    pub alloc_count: usize,
}

/// Return the default arena configuration.
pub fn default_arena_config() -> ArenaConfig {
    ArenaConfig { chunk_size: 4096 }
}

/// Create a new arena allocator.
pub fn new_arena(config: &ArenaConfig) -> ArenaAllocator {
    ArenaAllocator {
        buf: vec![0u8; config.chunk_size],
        used: 0,
        alloc_count: 0,
    }
}

/// Bump-allocate `size` bytes. Returns the byte offset into `buf`, or `None` if out of space.
pub fn arena_alloc_bytes(arena: &mut ArenaAllocator, size: usize) -> Option<usize> {
    if arena.used + size > arena.buf.len() {
        return None;
    }
    let offset = arena.used;
    arena.used += size;
    arena.alloc_count += 1;
    Some(offset)
}

/// Bump-allocate `size` bytes with the given alignment.
///
/// Returns the aligned byte offset into `buf`, or `None` if out of space.
/// The `align` argument is rounded up to at least 1.
pub fn arena_alloc_bytes_aligned(
    arena: &mut ArenaAllocator,
    size: usize,
    align: usize,
) -> Option<usize> {
    let align = align.max(1);
    let aligned_offset = (arena.used + align - 1) & !(align - 1);
    if aligned_offset + size > arena.buf.len() {
        return None;
    }
    arena.used = aligned_offset + size;
    arena.alloc_count += 1;
    Some(aligned_offset)
}

/// Reset the arena (frees all allocations at once).
pub fn arena_reset(arena: &mut ArenaAllocator) {
    arena.used = 0;
    arena.alloc_count = 0;
}

/// Return arena statistics.
pub fn arena_stats(arena: &ArenaAllocator) -> ArenaStats {
    ArenaStats {
        used: arena.used,
        capacity: arena.buf.len(),
        alloc_count: arena.alloc_count,
    }
}

/// Return the number of bytes remaining.
pub fn arena_remaining(arena: &ArenaAllocator) -> usize {
    arena.buf.len().saturating_sub(arena.used)
}

/// Return the usage ratio (0.0 = empty, 1.0 = full).
pub fn arena_usage_ratio(arena: &ArenaAllocator) -> f32 {
    if arena.buf.is_empty() {
        return 1.0;
    }
    arena.used as f32 / arena.buf.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ---------- existing unit tests ----------

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
        let off1 = arena_alloc_bytes(&mut arena, 32).expect("should succeed");
        let off2 = arena_alloc_bytes(&mut arena, 32).expect("should succeed");
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

    // ---------- new aligned-allocation tests ----------

    #[test]
    fn test_alloc_aligned() {
        // Allocate 1 byte, then 4 bytes with align=4; second offset must be 4.
        let cfg = ArenaConfig { chunk_size: 256 };
        let mut arena = new_arena(&cfg);
        let off1 = arena_alloc_bytes_aligned(&mut arena, 1, 1).expect("first alloc");
        assert_eq!(off1, 0);
        let off2 = arena_alloc_bytes_aligned(&mut arena, 4, 4).expect("second alloc");
        assert_eq!(off2, 4);
    }

    #[test]
    fn test_alloc_large_alignment() {
        let cfg = ArenaConfig { chunk_size: 256 };
        let mut arena = new_arena(&cfg);
        // Misalign by 1 byte first.
        arena_alloc_bytes(&mut arena, 1);
        // Request 8 bytes with align=16; offset must be multiple of 16.
        let off = arena_alloc_bytes_aligned(&mut arena, 8, 16).expect("aligned alloc");
        assert_eq!(off % 16, 0);
    }

    #[test]
    fn test_aligned_out_of_space() {
        let cfg = ArenaConfig { chunk_size: 8 };
        let mut arena = new_arena(&cfg);
        assert!(arena_alloc_bytes_aligned(&mut arena, 16, 1).is_none());
    }

    // ---------- proptest: alignment + no-overlap ----------

    proptest! {
        #[test]
        fn prop_aligned_alloc_invariants(
            allocs in proptest::collection::vec(
                (1usize..=64usize,
                 proptest::sample::select(vec![1usize, 2, 4, 8, 16, 32])),
                0..50
            )
        ) {
            // Use a large enough arena (50 * 64 + max padding ~32 per entry).
            let cfg = ArenaConfig { chunk_size: 8192 };
            let mut arena = new_arena(&cfg);
            let mut prev_end: usize = 0;

            for (size, align) in &allocs {
                let size = *size;
                let align = *align;
                let off = match arena_alloc_bytes_aligned(&mut arena, size, align) {
                    Some(o) => o,
                    None => break, // arena full — ok, stop checking
                };
                // Returned offset must satisfy the alignment.
                prop_assert_eq!(off % align, 0, "offset {} not aligned to {}", off, align);
                // Returned offset must not overlap the previous allocation.
                prop_assert!(off >= prev_end, "overlap: off={} prev_end={}", off, prev_end);
                prev_end = off + size;
            }
        }
    }
}
