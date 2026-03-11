#![allow(dead_code)]

/// A ring (circular) allocator that manages a fixed-size buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RingAllocator {
    capacity: usize,
    head: usize,
    used: usize,
}

/// Creates a new ring allocator with the given capacity.
#[allow(dead_code)]
pub fn new_ring_allocator(capacity: usize) -> RingAllocator {
    RingAllocator {
        capacity,
        head: 0,
        used: 0,
    }
}

/// Allocates `size` units from the ring, returns the start offset or None if full.
#[allow(dead_code)]
pub fn ring_alloc(alloc: &mut RingAllocator, size: usize) -> Option<usize> {
    if alloc.used + size > alloc.capacity {
        return None;
    }
    let offset = alloc.head;
    alloc.head = (alloc.head + size) % alloc.capacity;
    alloc.used += size;
    Some(offset)
}

/// Frees `size` units from the ring (FIFO order).
#[allow(dead_code)]
pub fn ring_free(alloc: &mut RingAllocator, size: usize) {
    let freed = size.min(alloc.used);
    alloc.used -= freed;
}

/// Returns total capacity.
#[allow(dead_code)]
pub fn ring_capacity(alloc: &RingAllocator) -> usize {
    alloc.capacity
}

/// Returns currently used amount.
#[allow(dead_code)]
pub fn ring_used(alloc: &RingAllocator) -> usize {
    alloc.used
}

/// Returns available space.
#[allow(dead_code)]
pub fn ring_available(alloc: &RingAllocator) -> usize {
    alloc.capacity - alloc.used
}

/// Resets the allocator.
#[allow(dead_code)]
pub fn ring_reset(alloc: &mut RingAllocator) {
    alloc.head = 0;
    alloc.used = 0;
}

/// Returns true if the allocator is full.
#[allow(dead_code)]
pub fn ring_is_full(alloc: &RingAllocator) -> bool {
    alloc.used >= alloc.capacity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ring_allocator() {
        let alloc = new_ring_allocator(100);
        assert_eq!(ring_capacity(&alloc), 100);
        assert_eq!(ring_used(&alloc), 0);
    }

    #[test]
    fn test_ring_alloc() {
        let mut alloc = new_ring_allocator(100);
        assert_eq!(ring_alloc(&mut alloc, 30), Some(0));
        assert_eq!(ring_used(&alloc), 30);
    }

    #[test]
    fn test_ring_alloc_full() {
        let mut alloc = new_ring_allocator(50);
        ring_alloc(&mut alloc, 50);
        assert_eq!(ring_alloc(&mut alloc, 1), None);
    }

    #[test]
    fn test_ring_free() {
        let mut alloc = new_ring_allocator(100);
        ring_alloc(&mut alloc, 40);
        ring_free(&mut alloc, 20);
        assert_eq!(ring_used(&alloc), 20);
    }

    #[test]
    fn test_ring_available() {
        let mut alloc = new_ring_allocator(100);
        ring_alloc(&mut alloc, 60);
        assert_eq!(ring_available(&alloc), 40);
    }

    #[test]
    fn test_ring_reset() {
        let mut alloc = new_ring_allocator(100);
        ring_alloc(&mut alloc, 50);
        ring_reset(&mut alloc);
        assert_eq!(ring_used(&alloc), 0);
        assert_eq!(ring_available(&alloc), 100);
    }

    #[test]
    fn test_ring_is_full() {
        let mut alloc = new_ring_allocator(10);
        assert!(!ring_is_full(&alloc));
        ring_alloc(&mut alloc, 10);
        assert!(ring_is_full(&alloc));
    }

    #[test]
    fn test_ring_wrap_around() {
        let mut alloc = new_ring_allocator(10);
        ring_alloc(&mut alloc, 8);
        ring_free(&mut alloc, 8);
        let offset = ring_alloc(&mut alloc, 5);
        assert_eq!(offset, Some(8));
    }

    #[test]
    fn test_ring_free_excess() {
        let mut alloc = new_ring_allocator(10);
        ring_alloc(&mut alloc, 5);
        ring_free(&mut alloc, 100);
        assert_eq!(ring_used(&alloc), 0);
    }

    #[test]
    fn test_ring_capacity_zero() {
        let alloc = new_ring_allocator(0);
        assert!(ring_is_full(&alloc));
    }
}
