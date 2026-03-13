// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bitmap-based index for fast set membership and intersection queries.

/// A bitmap index storing up to `capacity` bits.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitmapIndex {
    words: Vec<u64>,
    capacity: usize,
}

#[allow(dead_code)]
impl BitmapIndex {
    pub fn new(capacity: usize) -> Self {
        let word_count = capacity.div_ceil(64);
        Self {
            words: vec![0u64; word_count],
            capacity,
        }
    }

    pub fn set(&mut self, bit: usize) {
        if bit < self.capacity {
            self.words[bit / 64] |= 1u64 << (bit % 64);
        }
    }

    pub fn clear_bit(&mut self, bit: usize) {
        if bit < self.capacity {
            self.words[bit / 64] &= !(1u64 << (bit % 64));
        }
    }

    pub fn get(&self, bit: usize) -> bool {
        if bit < self.capacity {
            (self.words[bit / 64] >> (bit % 64)) & 1 == 1
        } else {
            false
        }
    }

    pub fn count_ones(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let cap = self.capacity.min(other.capacity);
        let wc = self.words.len().min(other.words.len());
        let mut result = Self::new(cap);
        #[allow(clippy::needless_range_loop)]
        for i in 0..wc {
            result.words[i] = self.words[i] & other.words[i];
        }
        result
    }

    pub fn union(&self, other: &Self) -> Self {
        let cap = self.capacity.max(other.capacity);
        let mut result = Self::new(cap);
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.words.len().max(other.words.len()) {
            let a = if i < self.words.len() {
                self.words[i]
            } else {
                0
            };
            let b = if i < other.words.len() {
                other.words[i]
            } else {
                0
            };
            result.words[i] = a | b;
        }
        result
    }

    pub fn clear_all(&mut self) {
        for w in &mut self.words {
            *w = 0;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    pub fn first_set(&self) -> Option<usize> {
        for (i, &w) in self.words.iter().enumerate() {
            if w != 0 {
                let bit = i * 64 + w.trailing_zeros() as usize;
                if bit < self.capacity {
                    return Some(bit);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bitmap_is_empty() {
        let b = BitmapIndex::new(128);
        assert!(b.is_empty());
        assert_eq!(b.count_ones(), 0);
    }

    #[test]
    fn set_and_get() {
        let mut b = BitmapIndex::new(100);
        b.set(42);
        assert!(b.get(42));
        assert!(!b.get(43));
    }

    #[test]
    fn clear_bit() {
        let mut b = BitmapIndex::new(64);
        b.set(10);
        b.clear_bit(10);
        assert!(!b.get(10));
    }

    #[test]
    fn count_ones_accurate() {
        let mut b = BitmapIndex::new(200);
        b.set(0);
        b.set(63);
        b.set(64);
        b.set(199);
        assert_eq!(b.count_ones(), 4);
    }

    #[test]
    fn intersect_works() {
        let mut a = BitmapIndex::new(128);
        let mut b = BitmapIndex::new(128);
        a.set(5);
        a.set(10);
        b.set(10);
        b.set(20);
        let c = a.intersect(&b);
        assert!(c.get(10));
        assert!(!c.get(5));
        assert!(!c.get(20));
    }

    #[test]
    fn union_works() {
        let mut a = BitmapIndex::new(128);
        let mut b = BitmapIndex::new(128);
        a.set(3);
        b.set(7);
        let c = a.union(&b);
        assert!(c.get(3));
        assert!(c.get(7));
    }

    #[test]
    fn clear_all_resets() {
        let mut b = BitmapIndex::new(64);
        b.set(0);
        b.set(63);
        b.clear_all();
        assert!(b.is_empty());
    }

    #[test]
    fn first_set_finds_lowest() {
        let mut b = BitmapIndex::new(128);
        b.set(70);
        b.set(30);
        assert_eq!(b.first_set(), Some(30));
    }

    #[test]
    fn out_of_bounds_ignored() {
        let mut b = BitmapIndex::new(10);
        b.set(100); // no panic
        assert!(!b.get(100));
    }

    #[test]
    fn capacity_returns_max() {
        let b = BitmapIndex::new(256);
        assert_eq!(b.capacity(), 256);
    }
}
