//! Heap allocator stub — tracks allocated blocks with size/alignment metadata (no unsafe, pure bookkeeping).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AllocBlock {
    pub ptr: u64,
    pub size: usize,
    pub align: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeapAllocatorConfig {
    pub heap_size: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeapStats {
    pub used_bytes: usize,
    pub free_bytes: usize,
    pub block_count: usize,
    pub fragmentation_ratio: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeapAllocator {
    pub config: HeapAllocatorConfig,
    blocks: Vec<AllocBlock>,
    next_ptr: u64,
    used: usize,
}

#[allow(dead_code)]
pub fn default_heap_config() -> HeapAllocatorConfig {
    HeapAllocatorConfig { heap_size: 1024 * 1024 }
}

#[allow(dead_code)]
pub fn new_heap_allocator(cfg: &HeapAllocatorConfig) -> HeapAllocator {
    HeapAllocator {
        config: cfg.clone(),
        blocks: Vec::new(),
        next_ptr: 1,
        used: 0,
    }
}

/// Allocate `size` bytes with `align` alignment. Returns a simulated pointer (u64 handle) or None if out of memory.
#[allow(dead_code)]
pub fn heap_alloc(allocator: &mut HeapAllocator, size: usize, align: usize) -> Option<u64> {
    if size == 0 {
        return None;
    }
    // Align next_ptr up to `align`.
    let align_u64 = align.max(1) as u64;
    let aligned_ptr = allocator.next_ptr.div_ceil(align_u64) * align_u64;
    let end = aligned_ptr.saturating_add(size as u64);
    if allocator.used + size > allocator.config.heap_size {
        return None;
    }
    allocator.blocks.push(AllocBlock {
        ptr: aligned_ptr,
        size,
        align,
    });
    allocator.next_ptr = end;
    allocator.used += size;
    Some(aligned_ptr)
}

/// Free the block identified by `ptr`. Returns true if the block was found and removed.
#[allow(dead_code)]
pub fn heap_free(allocator: &mut HeapAllocator, ptr: u64) -> bool {
    if let Some(pos) = allocator.blocks.iter().position(|b| b.ptr == ptr) {
        let size = allocator.blocks[pos].size;
        allocator.blocks.remove(pos);
        allocator.used = allocator.used.saturating_sub(size);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn heap_stats(allocator: &HeapAllocator) -> HeapStats {
    HeapStats {
        used_bytes: heap_used_bytes(allocator),
        free_bytes: heap_free_bytes(allocator),
        block_count: heap_block_count(allocator),
        fragmentation_ratio: heap_fragmentation_ratio(allocator),
    }
}

#[allow(dead_code)]
pub fn heap_used_bytes(allocator: &HeapAllocator) -> usize {
    allocator.used
}

#[allow(dead_code)]
pub fn heap_free_bytes(allocator: &HeapAllocator) -> usize {
    allocator.config.heap_size.saturating_sub(allocator.used)
}

#[allow(dead_code)]
pub fn heap_block_count(allocator: &HeapAllocator) -> usize {
    allocator.blocks.len()
}

/// Reset the allocator, freeing all blocks.
#[allow(dead_code)]
pub fn heap_reset(allocator: &mut HeapAllocator) {
    allocator.blocks.clear();
    allocator.next_ptr = 1;
    allocator.used = 0;
}

/// Ratio of number-of-blocks to used-bytes (a simple fragmentation heuristic).
/// Returns 0.0 when no bytes are used.
#[allow(dead_code)]
pub fn heap_fragmentation_ratio(allocator: &HeapAllocator) -> f32 {
    if allocator.used == 0 {
        return 0.0;
    }
    allocator.blocks.len() as f32 / allocator.used as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_heap_config();
        assert_eq!(cfg.heap_size, 1024 * 1024);
    }

    #[test]
    fn test_new_allocator_empty() {
        let cfg = default_heap_config();
        let alloc = new_heap_allocator(&cfg);
        assert_eq!(heap_used_bytes(&alloc), 0);
        assert_eq!(heap_block_count(&alloc), 0);
    }

    #[test]
    fn test_alloc_and_free() {
        let cfg = default_heap_config();
        let mut alloc = new_heap_allocator(&cfg);
        let ptr = heap_alloc(&mut alloc, 64, 8).expect("alloc should succeed");
        assert_eq!(heap_used_bytes(&alloc), 64);
        assert_eq!(heap_block_count(&alloc), 1);
        let freed = heap_free(&mut alloc, ptr);
        assert!(freed);
        assert_eq!(heap_used_bytes(&alloc), 0);
        assert_eq!(heap_block_count(&alloc), 0);
    }

    #[test]
    fn test_alloc_out_of_memory() {
        let cfg = HeapAllocatorConfig { heap_size: 16 };
        let mut alloc = new_heap_allocator(&cfg);
        let r = heap_alloc(&mut alloc, 32, 1);
        assert!(r.is_none());
    }

    #[test]
    fn test_free_unknown_ptr_returns_false() {
        let cfg = default_heap_config();
        let mut alloc = new_heap_allocator(&cfg);
        assert!(!heap_free(&mut alloc, 9999));
    }

    #[test]
    fn test_heap_reset() {
        let cfg = default_heap_config();
        let mut alloc = new_heap_allocator(&cfg);
        heap_alloc(&mut alloc, 128, 4).unwrap();
        heap_alloc(&mut alloc, 256, 4).unwrap();
        heap_reset(&mut alloc);
        assert_eq!(heap_used_bytes(&alloc), 0);
        assert_eq!(heap_block_count(&alloc), 0);
    }

    #[test]
    fn test_heap_stats() {
        let cfg = HeapAllocatorConfig { heap_size: 1000 };
        let mut alloc = new_heap_allocator(&cfg);
        heap_alloc(&mut alloc, 200, 8).unwrap();
        let stats = heap_stats(&alloc);
        assert_eq!(stats.used_bytes, 200);
        assert_eq!(stats.free_bytes, 800);
        assert_eq!(stats.block_count, 1);
    }

    #[test]
    fn test_fragmentation_ratio_zero_when_empty() {
        let cfg = default_heap_config();
        let alloc = new_heap_allocator(&cfg);
        assert_eq!(heap_fragmentation_ratio(&alloc), 0.0);
    }

    #[test]
    fn test_alloc_zero_size_returns_none() {
        let cfg = default_heap_config();
        let mut alloc = new_heap_allocator(&cfg);
        assert!(heap_alloc(&mut alloc, 0, 4).is_none());
    }
}
