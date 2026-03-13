// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Free-list slot allocator for dense arrays with stable indices.

/// Slot state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum Slot<T> {
    Occupied(T),
    Free(Option<usize>), // next free index
}

/// Free-list allocator.
#[derive(Debug)]
#[allow(dead_code)]
pub struct FreeSlot<T> {
    slots: Vec<Slot<T>>,
    free_head: Option<usize>,
    count: usize,
}

/// Create a new FreeSlot allocator.
#[allow(dead_code)]
pub fn new_free_slot<T>() -> FreeSlot<T> {
    FreeSlot {
        slots: Vec::new(),
        free_head: None,
        count: 0,
    }
}

/// Allocate a slot and insert value; returns index.
#[allow(dead_code)]
pub fn alloc<T>(fs: &mut FreeSlot<T>, val: T) -> usize {
    if let Some(idx) = fs.free_head {
        if let Slot::Free(next) = &fs.slots[idx] {
            fs.free_head = *next;
        }
        fs.slots[idx] = Slot::Occupied(val);
        fs.count += 1;
        idx
    } else {
        let idx = fs.slots.len();
        fs.slots.push(Slot::Occupied(val));
        fs.count += 1;
        idx
    }
}

/// Free a slot by index; returns the value if occupied.
#[allow(dead_code)]
pub fn free<T>(fs: &mut FreeSlot<T>, idx: usize) -> Option<T> {
    if idx >= fs.slots.len() {
        return None;
    }
    let old = std::mem::replace(&mut fs.slots[idx], Slot::Free(fs.free_head));
    if let Slot::Occupied(val) = old {
        fs.free_head = Some(idx);
        fs.count -= 1;
        Some(val)
    } else {
        // restore
        fs.slots[idx] = old;
        None
    }
}

/// Get a reference to an occupied slot.
#[allow(dead_code)]
pub fn get<T>(fs: &FreeSlot<T>, idx: usize) -> Option<&T> {
    fs.slots.get(idx).and_then(|s| {
        if let Slot::Occupied(ref v) = s {
            Some(v)
        } else {
            None
        }
    })
}

/// Get a mutable reference to an occupied slot.
#[allow(dead_code)]
pub fn get_mut<T>(fs: &mut FreeSlot<T>, idx: usize) -> Option<&mut T> {
    fs.slots.get_mut(idx).and_then(|s| {
        if let Slot::Occupied(ref mut v) = s {
            Some(v)
        } else {
            None
        }
    })
}

/// Number of occupied slots.
#[allow(dead_code)]
pub fn slot_count<T>(fs: &FreeSlot<T>) -> usize {
    fs.count
}

/// Total capacity including free slots.
#[allow(dead_code)]
pub fn slot_capacity<T>(fs: &FreeSlot<T>) -> usize {
    fs.slots.len()
}

/// Whether a slot index is occupied.
#[allow(dead_code)]
pub fn is_occupied<T>(fs: &FreeSlot<T>, idx: usize) -> bool {
    get(fs, idx).is_some()
}

/// Iterate over occupied (index, value) pairs.
#[allow(dead_code)]
pub fn iter_occupied<T>(fs: &FreeSlot<T>) -> Vec<(usize, &T)> {
    fs.slots
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if let Slot::Occupied(ref v) = s {
                Some((i, v))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_and_get() {
        let mut fs = new_free_slot();
        let idx = alloc(&mut fs, 42u32);
        assert_eq!(get(&fs, idx), Some(&42));
    }

    #[test]
    fn test_free_and_reuse() {
        let mut fs = new_free_slot();
        let a = alloc(&mut fs, 1u32);
        let b = alloc(&mut fs, 2u32);
        free(&mut fs, a);
        let c = alloc(&mut fs, 99u32);
        assert_eq!(c, a);
        assert_eq!(get(&fs, b), Some(&2));
    }

    #[test]
    fn test_count() {
        let mut fs = new_free_slot();
        alloc(&mut fs, 1u32);
        alloc(&mut fs, 2u32);
        assert_eq!(slot_count(&fs), 2);
        free(&mut fs, 0);
        assert_eq!(slot_count(&fs), 1);
    }

    #[test]
    fn test_is_occupied() {
        let mut fs = new_free_slot();
        let idx = alloc(&mut fs, 7u32);
        assert!(is_occupied(&fs, idx));
        free(&mut fs, idx);
        assert!(!is_occupied(&fs, idx));
    }

    #[test]
    fn test_get_out_of_bounds() {
        let fs: FreeSlot<u32> = new_free_slot();
        assert_eq!(get(&fs, 99), None);
    }

    #[test]
    fn test_iter_occupied() {
        let mut fs = new_free_slot();
        alloc(&mut fs, 10u32);
        alloc(&mut fs, 20u32);
        free(&mut fs, 0);
        let pairs = iter_occupied(&fs);
        assert_eq!(pairs.len(), 1);
        assert_eq!(*pairs[0].1, 20);
    }

    #[test]
    fn test_get_mut() {
        let mut fs = new_free_slot();
        let idx = alloc(&mut fs, 5u32);
        if let Some(v) = get_mut(&mut fs, idx) {
            *v = 50;
        }
        assert_eq!(get(&fs, idx), Some(&50));
    }

    #[test]
    fn test_free_unoccupied_returns_none() {
        let mut fs: FreeSlot<u32> = new_free_slot();
        let idx = alloc(&mut fs, 1);
        free(&mut fs, idx);
        assert_eq!(free(&mut fs, idx), None);
    }

    #[test]
    fn test_capacity_grows() {
        let mut fs = new_free_slot();
        for i in 0..8u32 {
            alloc(&mut fs, i);
        }
        assert_eq!(slot_capacity(&fs), 8);
    }
}
