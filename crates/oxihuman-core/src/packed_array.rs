// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct PackedArray {
    pub data: Vec<u64>,
    pub bits: usize,
    pub len: usize,
}

pub fn new_packed_array(len: usize, bits: usize) -> PackedArray {
    let bits = bits.clamp(1, 64);
    let total_bits = len * bits;
    let words = total_bits.div_ceil(64);
    PackedArray {
        data: vec![0u64; words.max(1)],
        bits,
        len,
    }
}

pub fn packed_get(a: &PackedArray, i: usize) -> u64 {
    if i >= a.len {
        return 0;
    }
    let bit_idx = i * a.bits;
    let word = bit_idx / 64;
    let offset = bit_idx % 64;
    let mask = if a.bits == 64 {
        u64::MAX
    } else {
        (1u64 << a.bits) - 1
    };
    if offset + a.bits <= 64 {
        (a.data[word] >> offset) & mask
    } else {
        let lo = a.data[word] >> offset;
        let hi_bits = a.bits - (64 - offset);
        let hi = if word + 1 < a.data.len() {
            a.data[word + 1] << (64 - offset)
        } else {
            0
        };
        let hi_mask = (1u64 << hi_bits) - 1;
        lo | ((hi >> (64 - offset - hi_bits)) & hi_mask) << (64 - offset)
    }
}

pub fn packed_set(a: &mut PackedArray, i: usize, val: u64) {
    if i >= a.len {
        return;
    }
    let bit_idx = i * a.bits;
    let word = bit_idx / 64;
    let offset = bit_idx % 64;
    let mask = if a.bits == 64 {
        u64::MAX
    } else {
        (1u64 << a.bits) - 1
    };
    let val = val & mask;
    a.data[word] &= !(mask << offset);
    a.data[word] |= val << offset;
    if offset + a.bits > 64 && word + 1 < a.data.len() {
        let spill = a.bits - (64 - offset);
        let spill_mask = (1u64 << spill) - 1;
        a.data[word + 1] &= !spill_mask;
        a.data[word + 1] |= val >> (a.bits - spill);
    }
}

pub fn packed_len(a: &PackedArray) -> usize {
    a.len
}
pub fn packed_bits_per_elem(a: &PackedArray) -> usize {
    a.bits
}
pub fn packed_storage_bytes(a: &PackedArray) -> usize {
    a.data.len() * 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        /* set a value and read it back */
        let mut a = new_packed_array(10, 4);
        packed_set(&mut a, 3, 0b1010);
        assert_eq!(packed_get(&a, 3), 0b1010);
    }

    #[test]
    fn initial_zero() {
        /* all elements are zero after creation */
        let a = new_packed_array(8, 3);
        for i in 0..8 {
            assert_eq!(packed_get(&a, i), 0);
        }
    }

    #[test]
    fn multiple_elements() {
        /* multiple elements stored independently */
        let mut a = new_packed_array(4, 8);
        packed_set(&mut a, 0, 10);
        packed_set(&mut a, 1, 20);
        assert_eq!(packed_get(&a, 0), 10);
        assert_eq!(packed_get(&a, 1), 20);
    }

    #[test]
    fn len_correct() {
        /* packed_len returns the configured length */
        let a = new_packed_array(7, 5);
        assert_eq!(packed_len(&a), 7);
    }

    #[test]
    fn bits_per_elem() {
        /* bits_per_elem returns configured bit width */
        let a = new_packed_array(5, 6);
        assert_eq!(packed_bits_per_elem(&a), 6);
    }

    #[test]
    fn storage_bytes_positive() {
        /* storage is non-zero for non-empty array */
        let a = new_packed_array(10, 4);
        assert!(packed_storage_bytes(&a) > 0);
    }

    #[test]
    fn out_of_bounds_get() {
        /* out-of-bounds get returns 0 safely */
        let a = new_packed_array(4, 4);
        assert_eq!(packed_get(&a, 100), 0);
    }
}
