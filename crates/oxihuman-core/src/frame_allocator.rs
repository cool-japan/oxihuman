//! Arena/frame allocator that resets all allocations at frame boundary.
//!
//! Provides a fast bump-pointer allocator backed by a `Vec<u8>`. All memory
//! is reclaimed in O(1) by a single reset call, making it ideal for per-frame
//! temporary buffers in a game/simulation loop.

/// Configuration for the frame allocator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrameAllocatorConfig {
    /// Total capacity in bytes.
    pub capacity: usize,
}

#[allow(dead_code)]
impl FrameAllocatorConfig {
    fn new() -> Self {
        Self { capacity: 4096 }
    }
}

/// Returns the default frame allocator configuration.
#[allow(dead_code)]
pub fn default_frame_alloc_config() -> FrameAllocatorConfig {
    FrameAllocatorConfig::new()
}

/// Represents a single allocated block (start offset + length).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FrameAlloc {
    /// Start offset within the arena buffer.
    pub offset: usize,
    /// Length of the allocation in bytes.
    pub len: usize,
}

/// Arena frame allocator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FrameAllocator {
    buffer: Vec<u8>,
    cursor: usize,
    block_count: usize,
}

/// Creates a new `FrameAllocator` with the given configuration.
#[allow(dead_code)]
pub fn new_frame_allocator(config: FrameAllocatorConfig) -> FrameAllocator {
    FrameAllocator {
        buffer: vec![0u8; config.capacity],
        cursor: 0,
        block_count: 0,
    }
}

/// Allocates `size` bytes from the arena. Returns `Some(FrameAlloc)` on
/// success, or `None` if there is not enough space remaining.
#[allow(dead_code)]
pub fn frame_alloc_bytes(alloc: &mut FrameAllocator, size: usize) -> Option<FrameAlloc> {
    if alloc.cursor + size > alloc.buffer.len() {
        return None;
    }
    let fa = FrameAlloc {
        offset: alloc.cursor,
        len: size,
    };
    alloc.cursor += size;
    alloc.block_count += 1;
    Some(fa)
}

/// Resets the allocator, reclaiming all memory in O(1).
#[allow(dead_code)]
pub fn frame_alloc_reset(alloc: &mut FrameAllocator) {
    alloc.cursor = 0;
    alloc.block_count = 0;
}

/// Returns the number of bytes currently allocated.
#[allow(dead_code)]
pub fn frame_alloc_used(alloc: &FrameAllocator) -> usize {
    alloc.cursor
}

/// Returns the total capacity in bytes.
#[allow(dead_code)]
pub fn frame_alloc_capacity(alloc: &FrameAllocator) -> usize {
    alloc.buffer.len()
}

/// Returns `true` if the allocator has no remaining space.
#[allow(dead_code)]
pub fn frame_alloc_is_full(alloc: &FrameAllocator) -> bool {
    alloc.cursor >= alloc.buffer.len()
}

/// Serialises allocator state to a simple JSON string.
#[allow(dead_code)]
pub fn frame_alloc_to_json(alloc: &FrameAllocator) -> String {
    format!(
        "{{\"capacity\":{},\"used\":{},\"block_count\":{}}}",
        alloc.buffer.len(),
        alloc.cursor,
        alloc.block_count
    )
}

/// Returns a fragmentation estimate: `used / capacity` in [0.0, 1.0].
/// Because this is a pure bump allocator, fragmentation is always 0 until reset.
#[allow(dead_code)]
pub fn frame_alloc_fragmentation(alloc: &FrameAllocator) -> f32 {
    if alloc.buffer.is_empty() {
        return 0.0;
    }
    // Bump allocators have zero internal fragmentation by definition.
    // External fragmentation (wasted space after individual frees) does not
    // apply here since we never free individual blocks. Return 0.0.
    let _ = alloc; // keep parameter used
    0.0
}

/// Returns the number of allocations made since the last reset.
#[allow(dead_code)]
pub fn frame_alloc_block_count(alloc: &FrameAllocator) -> usize {
    alloc.block_count
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_alloc() -> FrameAllocator {
        new_frame_allocator(default_frame_alloc_config())
    }

    #[test]
    fn test_initial_state() {
        let a = make_alloc();
        assert_eq!(frame_alloc_used(&a), 0);
        assert_eq!(frame_alloc_capacity(&a), 4096);
        assert!(!frame_alloc_is_full(&a));
    }

    #[test]
    fn test_alloc_returns_correct_offset() {
        let mut a = make_alloc();
        let fa = frame_alloc_bytes(&mut a, 64).unwrap();
        assert_eq!(fa.offset, 0);
        assert_eq!(fa.len, 64);
        let fb = frame_alloc_bytes(&mut a, 32).unwrap();
        assert_eq!(fb.offset, 64);
    }

    #[test]
    fn test_used_increases() {
        let mut a = make_alloc();
        frame_alloc_bytes(&mut a, 100);
        assert_eq!(frame_alloc_used(&a), 100);
    }

    #[test]
    fn test_reset_clears_used() {
        let mut a = make_alloc();
        frame_alloc_bytes(&mut a, 200);
        frame_alloc_reset(&mut a);
        assert_eq!(frame_alloc_used(&a), 0);
        assert_eq!(frame_alloc_block_count(&a), 0);
    }

    #[test]
    fn test_overflow_returns_none() {
        let cfg = FrameAllocatorConfig { capacity: 16 };
        let mut a = new_frame_allocator(cfg);
        assert!(frame_alloc_bytes(&mut a, 17).is_none());
    }

    #[test]
    fn test_exactly_full() {
        let cfg = FrameAllocatorConfig { capacity: 8 };
        let mut a = new_frame_allocator(cfg);
        frame_alloc_bytes(&mut a, 8);
        assert!(frame_alloc_is_full(&a));
        assert!(frame_alloc_bytes(&mut a, 1).is_none());
    }

    #[test]
    fn test_block_count() {
        let mut a = make_alloc();
        frame_alloc_bytes(&mut a, 10);
        frame_alloc_bytes(&mut a, 20);
        assert_eq!(frame_alloc_block_count(&a), 2);
    }

    #[test]
    fn test_fragmentation_is_zero() {
        let mut a = make_alloc();
        frame_alloc_bytes(&mut a, 100);
        assert!((frame_alloc_fragmentation(&a)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_to_json_contains_capacity() {
        let a = make_alloc();
        let json = frame_alloc_to_json(&a);
        assert!(json.contains("capacity"));
        assert!(json.contains("4096"));
    }

    #[test]
    fn test_reuse_after_reset() {
        let mut a = make_alloc();
        frame_alloc_bytes(&mut a, 4096);
        frame_alloc_reset(&mut a);
        let fa = frame_alloc_bytes(&mut a, 128);
        assert!(fa.is_some());
        assert_eq!(fa.unwrap().offset, 0);
    }
}
