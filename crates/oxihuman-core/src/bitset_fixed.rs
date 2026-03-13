// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct BitsetFixed {
    pub words: Vec<u64>,
    pub len: usize,
}

pub fn new_bitset_fixed(len: usize) -> BitsetFixed {
    let words = len.div_ceil(64);
    BitsetFixed {
        words: vec![0u64; words.max(1)],
        len,
    }
}

pub fn bitset_set(b: &mut BitsetFixed, i: usize) {
    if i < b.len {
        b.words[i / 64] |= 1u64 << (i % 64);
    }
}

pub fn bitset_clear(b: &mut BitsetFixed, i: usize) {
    if i < b.len {
        b.words[i / 64] &= !(1u64 << (i % 64));
    }
}

pub fn bitset_get(b: &BitsetFixed, i: usize) -> bool {
    i < b.len && (b.words[i / 64] >> (i % 64)) & 1 == 1
}

pub fn bitset_flip(b: &mut BitsetFixed, i: usize) {
    if i < b.len {
        b.words[i / 64] ^= 1u64 << (i % 64);
    }
}

pub fn bitset_count_ones(b: &BitsetFixed) -> usize {
    b.words.iter().map(|w| w.count_ones() as usize).sum()
}

pub fn bitset_count_zeros(b: &BitsetFixed) -> usize {
    b.len - bitset_count_ones(b)
}

pub fn bitset_and(a: &BitsetFixed, b: &BitsetFixed) -> BitsetFixed {
    let len = a.len.min(b.len);
    let words = a
        .words
        .iter()
        .zip(b.words.iter())
        .map(|(x, y)| x & y)
        .collect();
    BitsetFixed { words, len }
}

pub fn bitset_or(a: &BitsetFixed, b: &BitsetFixed) -> BitsetFixed {
    let len = a.len.max(b.len);
    let aw = a.words.len();
    let bw = b.words.len();
    let max_w = aw.max(bw);
    let words = (0..max_w)
        .map(|i| {
            let av = if i < aw { a.words[i] } else { 0 };
            let bv = if i < bw { b.words[i] } else { 0 };
            av | bv
        })
        .collect();
    BitsetFixed { words, len }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        /* set a bit then read it back */
        let mut b = new_bitset_fixed(64);
        bitset_set(&mut b, 7);
        assert!(bitset_get(&b, 7));
        assert!(!bitset_get(&b, 6));
    }

    #[test]
    fn clear_bit() {
        /* clear resets a previously set bit */
        let mut b = new_bitset_fixed(64);
        bitset_set(&mut b, 3);
        bitset_clear(&mut b, 3);
        assert!(!bitset_get(&b, 3));
    }

    #[test]
    fn count_ones() {
        /* popcount matches number of set bits */
        let mut b = new_bitset_fixed(16);
        bitset_set(&mut b, 0);
        bitset_set(&mut b, 5);
        assert_eq!(bitset_count_ones(&b), 2);
    }

    #[test]
    fn count_zeros() {
        /* zero count is complement of one count */
        let mut b = new_bitset_fixed(8);
        bitset_set(&mut b, 1);
        assert_eq!(bitset_count_zeros(&b), 7);
    }

    #[test]
    fn flip() {
        /* flip toggles a bit */
        let mut b = new_bitset_fixed(8);
        bitset_flip(&mut b, 4);
        assert!(bitset_get(&b, 4));
        bitset_flip(&mut b, 4);
        assert!(!bitset_get(&b, 4));
    }

    #[test]
    fn and_operation() {
        /* AND of two bitsets */
        let mut a = new_bitset_fixed(8);
        let mut bb = new_bitset_fixed(8);
        bitset_set(&mut a, 2);
        bitset_set(&mut a, 3);
        bitset_set(&mut bb, 3);
        let c = bitset_and(&a, &bb);
        assert!(!bitset_get(&c, 2));
        assert!(bitset_get(&c, 3));
    }

    #[test]
    fn or_operation() {
        /* OR of two bitsets */
        let mut a = new_bitset_fixed(8);
        let mut bb = new_bitset_fixed(8);
        bitset_set(&mut a, 0);
        bitset_set(&mut bb, 1);
        let c = bitset_or(&a, &bb);
        assert!(bitset_get(&c, 0));
        assert!(bitset_get(&c, 1));
    }

    #[test]
    fn out_of_bounds_ignored() {
        /* operations on out-of-bounds index are safe */
        let mut b = new_bitset_fixed(4);
        bitset_set(&mut b, 100);
        assert!(!bitset_get(&b, 100));
    }
}
