// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A slab allocator that manages fixed-size slots with O(1) alloc/free using a free list.

/// Handle returned by the slab allocator.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlabKey {
    index: u32,
    generation: u32,
}

#[allow(dead_code)]
impl SlabKey {
    pub fn index(&self) -> u32 {
        self.index
    }
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum SlabEntry<T> {
    Occupied {
        value: T,
        generation: u32,
    },
    Vacant {
        next_free: Option<u32>,
        generation: u32,
    },
}

/// A slab allocator with generational keys.
#[allow(dead_code)]
#[derive(Debug)]
pub struct SlabAllocator<T> {
    entries: Vec<SlabEntry<T>>,
    free_head: Option<u32>,
    count: usize,
}

#[allow(dead_code)]
impl<T> SlabAllocator<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_head: None,
            count: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            entries: Vec::with_capacity(cap),
            free_head: None,
            count: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> SlabKey {
        self.count += 1;
        if let Some(idx) = self.free_head {
            let i = idx as usize;
            match &self.entries[i] {
                SlabEntry::Vacant {
                    next_free,
                    generation,
                } => {
                    let gen = *generation + 1;
                    let nf = *next_free;
                    self.entries[i] = SlabEntry::Occupied {
                        value,
                        generation: gen,
                    };
                    self.free_head = nf;
                    SlabKey {
                        index: idx,
                        generation: gen,
                    }
                }
                _ => unreachable!(),
            }
        } else {
            let idx = self.entries.len() as u32;
            self.entries.push(SlabEntry::Occupied {
                value,
                generation: 1,
            });
            SlabKey {
                index: idx,
                generation: 1,
            }
        }
    }

    pub fn remove(&mut self, key: SlabKey) -> Option<T> {
        let i = key.index as usize;
        if i >= self.entries.len() {
            return None;
        }
        match &self.entries[i] {
            SlabEntry::Occupied { generation, .. } if *generation == key.generation => {
                let gen = *generation;
                let old = std::mem::replace(
                    &mut self.entries[i],
                    SlabEntry::Vacant {
                        next_free: self.free_head,
                        generation: gen,
                    },
                );
                self.free_head = Some(key.index);
                self.count -= 1;
                match old {
                    SlabEntry::Occupied { value, .. } => Some(value),
                    _ => unreachable!(),
                }
            }
            _ => None,
        }
    }

    pub fn get(&self, key: SlabKey) -> Option<&T> {
        match self.entries.get(key.index as usize)? {
            SlabEntry::Occupied { value, generation } if *generation == key.generation => {
                Some(value)
            }
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: SlabKey) -> Option<&mut T> {
        match self.entries.get_mut(key.index as usize)? {
            SlabEntry::Occupied { value, generation } if *generation == key.generation => {
                Some(value)
            }
            _ => None,
        }
    }

    pub fn contains(&self, key: SlabKey) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn capacity(&self) -> usize {
        self.entries.len()
    }
}

impl<T> Default for SlabAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut slab = SlabAllocator::new();
        let k = slab.insert(42);
        assert_eq!(slab.get(k), Some(&42));
    }

    #[test]
    fn test_remove() {
        let mut slab = SlabAllocator::new();
        let k = slab.insert(10);
        assert_eq!(slab.remove(k), Some(10));
        assert_eq!(slab.get(k), None);
    }

    #[test]
    fn test_generation() {
        let mut slab = SlabAllocator::new();
        let k1 = slab.insert(1);
        slab.remove(k1);
        let k2 = slab.insert(2);
        assert_eq!(slab.get(k1), None);
        assert_eq!(slab.get(k2), Some(&2));
        assert_eq!(k1.index(), k2.index()); // reused slot
    }

    #[test]
    fn test_multiple() {
        let mut slab = SlabAllocator::new();
        let a = slab.insert("a");
        let b = slab.insert("b");
        let c = slab.insert("c");
        assert_eq!(slab.len(), 3);
        slab.remove(b);
        assert_eq!(slab.len(), 2);
        assert!(!slab.contains(b));
        assert!(slab.contains(a));
        assert!(slab.contains(c));
    }

    #[test]
    fn test_get_mut() {
        let mut slab = SlabAllocator::new();
        let k = slab.insert(100);
        *slab.get_mut(k).unwrap() = 200;
        assert_eq!(slab.get(k), Some(&200));
    }

    #[test]
    fn test_empty() {
        let slab: SlabAllocator<i32> = SlabAllocator::new();
        assert!(slab.is_empty());
    }

    #[test]
    fn test_remove_invalid() {
        let mut slab: SlabAllocator<i32> = SlabAllocator::new();
        let fake = SlabKey {
            index: 99,
            generation: 1,
        };
        assert_eq!(slab.remove(fake), None);
    }

    #[test]
    fn test_capacity() {
        let mut slab = SlabAllocator::new();
        slab.insert(1);
        slab.insert(2);
        assert_eq!(slab.capacity(), 2);
    }

    #[test]
    fn test_reuse_chain() {
        let mut slab = SlabAllocator::new();
        let a = slab.insert(1);
        let b = slab.insert(2);
        slab.remove(a);
        slab.remove(b);
        let c = slab.insert(3);
        let d = slab.insert(4);
        assert_eq!(slab.len(), 2);
        assert!(slab.contains(c));
        assert!(slab.contains(d));
    }

    #[test]
    fn test_with_capacity() {
        let slab: SlabAllocator<i32> = SlabAllocator::with_capacity(100);
        assert!(slab.is_empty());
    }
}
