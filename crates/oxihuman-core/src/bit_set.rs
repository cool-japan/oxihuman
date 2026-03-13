// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A compact bitset for tracking boolean flags by index.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitSet {
    words: Vec<u64>,
}

#[allow(dead_code)]
impl BitSet {
    pub fn new() -> Self {
        Self { words: Vec::new() }
    }

    pub fn with_capacity(bits: usize) -> Self {
        let words = bits.div_ceil(64);
        Self {
            words: vec![0u64; words],
        }
    }

    pub fn set(&mut self, index: usize) {
        let word = index / 64;
        let bit = index % 64;
        if word >= self.words.len() {
            self.words.resize(word + 1, 0);
        }
        self.words[word] |= 1u64 << bit;
    }

    pub fn clear_bit(&mut self, index: usize) {
        let word = index / 64;
        let bit = index % 64;
        if word < self.words.len() {
            self.words[word] &= !(1u64 << bit);
        }
    }

    pub fn test(&self, index: usize) -> bool {
        let word = index / 64;
        let bit = index % 64;
        if word >= self.words.len() {
            return false;
        }
        (self.words[word] & (1u64 << bit)) != 0
    }

    pub fn count_ones(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    pub fn clear(&mut self) {
        for w in &mut self.words {
            *w = 0;
        }
    }

    pub fn union(&self, other: &BitSet) -> BitSet {
        let len = self.words.len().max(other.words.len());
        let mut result = BitSet::with_capacity(len * 64);
        for i in 0..len {
            let a = self.words.get(i).copied().unwrap_or(0);
            let b = other.words.get(i).copied().unwrap_or(0);
            result.words[i] = a | b;
        }
        result
    }

    pub fn intersection(&self, other: &BitSet) -> BitSet {
        let len = self.words.len().min(other.words.len());
        let mut result = BitSet::with_capacity(len * 64);
        for i in 0..len {
            result.words[i] = self.words[i] & other.words[i];
        }
        result
    }

    pub fn capacity_bits(&self) -> usize {
        self.words.len() * 64
    }
}

impl Default for BitSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let bs = BitSet::new();
        assert!(bs.is_empty());
        assert_eq!(bs.count_ones(), 0);
    }

    #[test]
    fn test_set_and_test() {
        let mut bs = BitSet::new();
        bs.set(5);
        assert!(bs.test(5));
        assert!(!bs.test(4));
    }

    #[test]
    fn test_clear_bit() {
        let mut bs = BitSet::new();
        bs.set(10);
        bs.clear_bit(10);
        assert!(!bs.test(10));
    }

    #[test]
    fn test_count_ones() {
        let mut bs = BitSet::new();
        bs.set(0);
        bs.set(63);
        bs.set(64);
        assert_eq!(bs.count_ones(), 3);
    }

    #[test]
    fn test_clear() {
        let mut bs = BitSet::new();
        bs.set(1);
        bs.set(100);
        bs.clear();
        assert!(bs.is_empty());
    }

    #[test]
    fn test_union() {
        let mut a = BitSet::new();
        a.set(1);
        let mut b = BitSet::new();
        b.set(2);
        let u = a.union(&b);
        assert!(u.test(1));
        assert!(u.test(2));
    }

    #[test]
    fn test_intersection() {
        let mut a = BitSet::new();
        a.set(1);
        a.set(2);
        let mut b = BitSet::new();
        b.set(2);
        b.set(3);
        let inter = a.intersection(&b);
        assert!(!inter.test(1));
        assert!(inter.test(2));
        assert!(!inter.test(3));
    }

    #[test]
    fn test_with_capacity() {
        let bs = BitSet::with_capacity(128);
        assert_eq!(bs.capacity_bits(), 128);
    }

    #[test]
    fn test_large_index() {
        let mut bs = BitSet::new();
        bs.set(1000);
        assert!(bs.test(1000));
        assert!(!bs.test(999));
    }

    #[test]
    fn test_test_out_of_range() {
        let bs = BitSet::new();
        assert!(!bs.test(500));
    }
}
