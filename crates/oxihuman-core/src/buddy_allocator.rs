// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Binary buddy allocator stub — manages a power-of-two address space by
//! recursively splitting and merging "buddy" blocks to satisfy allocations.

/// Order of the allocator: manages 2^order minimum-sized units.
pub struct BuddyAllocator {
    max_order: usize,
    /// free_lists[order] holds the starting indices of free blocks at that order.
    free_lists: Vec<Vec<usize>>,
    total_units: usize,
}

impl BuddyAllocator {
    /// Create a buddy allocator spanning `2^max_order` minimum units.
    pub fn new(max_order: usize) -> Self {
        let mut free_lists = vec![Vec::new(); max_order + 1];
        free_lists[max_order].push(0);
        let total_units = 1 << max_order;
        Self {
            max_order,
            free_lists,
            total_units,
        }
    }

    /// Allocate a block of `2^order` units. Returns the block start index.
    pub fn allocate(&mut self, order: usize) -> Option<usize> {
        if order > self.max_order {
            return None;
        }
        /* find the smallest available order >= requested */
        let mut found = None;
        for o in order..=self.max_order {
            if !self.free_lists[o].is_empty() {
                found = Some(o);
                break;
            }
        }
        let found_order = found?;
        let block = self.free_lists[found_order].pop()?;
        /* split down to the requested order */
        let mut o = found_order;
        while o > order {
            o -= 1;
            let buddy = block + (1 << o);
            self.free_lists[o].push(buddy);
        }
        Some(block)
    }

    /// Free a block of `2^order` units starting at `addr`.
    pub fn free(&mut self, addr: usize, order: usize) {
        let mut current = addr;
        let mut o = order;
        while o < self.max_order {
            let buddy = current ^ (1 << o);
            /* check if buddy is free at this order */
            if let Some(pos) = self.free_lists[o].iter().position(|&b| b == buddy) {
                self.free_lists[o].remove(pos);
                current = current.min(buddy);
                o += 1;
            } else {
                break;
            }
        }
        self.free_lists[o].push(current);
    }

    /// Number of free units summed across all orders.
    pub fn free_units(&self) -> usize {
        self.free_lists
            .iter()
            .enumerate()
            .map(|(o, list)| list.len() * (1 << o))
            .sum()
    }

    /// Total capacity in minimum units.
    pub fn total_units(&self) -> usize {
        self.total_units
    }

    /// Maximum order this allocator supports.
    pub fn max_order(&self) -> usize {
        self.max_order
    }
}

/// Create a buddy allocator of 2^`max_order` minimum units.
pub fn new_buddy_allocator(max_order: usize) -> BuddyAllocator {
    BuddyAllocator::new(max_order)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_order_zero() {
        let mut b = BuddyAllocator::new(4);
        let addr = b.allocate(0);
        assert!(addr.is_some()); /* order-0 alloc succeeds */
    }

    #[test]
    fn test_alloc_max_order() {
        let mut b = BuddyAllocator::new(3);
        let addr = b.allocate(3).unwrap();
        assert_eq!(addr, 0); /* whole block starts at 0 */
        assert!(b.allocate(0).is_none()); /* pool exhausted */
    }

    #[test]
    fn test_free_and_realloc() {
        let mut b = BuddyAllocator::new(3);
        let addr = b.allocate(3).unwrap();
        b.free(addr, 3);
        assert!(b.allocate(3).is_some()); /* full block available again */
    }

    #[test]
    fn test_split_and_merge() {
        let mut b = BuddyAllocator::new(2);
        /* allocate two order-0 blocks, free both — they should merge */
        let a0 = b.allocate(0).unwrap();
        let a1 = b.allocate(0).unwrap();
        b.free(a0, 0);
        b.free(a1, 0);
        /* now order-1 buddy pair should be merged back, order-2 may coalesce */
        assert_eq!(b.free_units(), 4); /* all units free */
    }

    #[test]
    fn test_total_units() {
        let b = BuddyAllocator::new(4);
        assert_eq!(b.total_units(), 16); /* 2^4 */
    }

    #[test]
    fn test_free_units_initially_full() {
        let b = BuddyAllocator::new(3);
        assert_eq!(b.free_units(), 8); /* 2^3 units free at start */
    }

    #[test]
    fn test_max_order() {
        let b = BuddyAllocator::new(5);
        assert_eq!(b.max_order(), 5); /* accessor correct */
    }

    #[test]
    fn test_alloc_beyond_max_order() {
        let mut b = BuddyAllocator::new(2);
        assert!(b.allocate(10).is_none()); /* order too large */
    }

    #[test]
    fn test_new_helper() {
        let b = new_buddy_allocator(3);
        assert_eq!(b.total_units(), 8); /* helper works */
    }
}
