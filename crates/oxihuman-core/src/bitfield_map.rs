#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dense bitfield mapping: u32 key → N bits.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitfieldMap {
    pub data: Vec<u64>,
    pub bits_per_key: u8,
    pub capacity: usize,
}

#[allow(dead_code)]
pub fn new_bitfield_map(capacity: usize, bits_per_key: u8) -> BitfieldMap {
    let bpk = bits_per_key.clamp(1, 64) as usize;
    let total_bits = capacity * bpk;
    let words = total_bits.div_ceil(64);
    BitfieldMap {
        data: vec![0u64; words],
        bits_per_key: bpk as u8,
        capacity,
    }
}

#[allow(dead_code)]
pub fn bfm_set(map: &mut BitfieldMap, key: usize, val: u64) {
    if key >= map.capacity {
        return;
    }
    let bpk = map.bits_per_key as usize;
    let mask = if bpk == 64 {
        u64::MAX
    } else {
        (1u64 << bpk) - 1
    };
    let val = val & mask;
    let bit_start = key * bpk;
    let word_idx = bit_start / 64;
    let bit_off = bit_start % 64;
    // Clear then set the bits
    if bit_off + bpk <= 64 {
        map.data[word_idx] &= !(mask << bit_off);
        map.data[word_idx] |= val << bit_off;
    } else {
        // Spans two words
        let lo_bits = 64 - bit_off;
        let hi_bits = bpk - lo_bits;
        let lo_mask = (1u64 << lo_bits) - 1;
        let hi_mask = (1u64 << hi_bits) - 1;
        map.data[word_idx] &= !(lo_mask << bit_off);
        map.data[word_idx] |= (val & lo_mask) << bit_off;
        if word_idx + 1 < map.data.len() {
            map.data[word_idx + 1] &= !hi_mask;
            map.data[word_idx + 1] |= (val >> lo_bits) & hi_mask;
        }
    }
}

#[allow(dead_code)]
pub fn bfm_get(map: &BitfieldMap, key: usize) -> u64 {
    if key >= map.capacity {
        return 0;
    }
    let bpk = map.bits_per_key as usize;
    let mask = if bpk == 64 {
        u64::MAX
    } else {
        (1u64 << bpk) - 1
    };
    let bit_start = key * bpk;
    let word_idx = bit_start / 64;
    let bit_off = bit_start % 64;
    if bit_off + bpk <= 64 {
        (map.data[word_idx] >> bit_off) & mask
    } else {
        let lo_bits = 64 - bit_off;
        let lo = (map.data[word_idx] >> bit_off) & ((1u64 << lo_bits) - 1);
        let hi = if word_idx + 1 < map.data.len() {
            let hi_bits = bpk - lo_bits;
            map.data[word_idx + 1] & ((1u64 << hi_bits) - 1)
        } else {
            0
        };
        lo | (hi << lo_bits)
    }
}

#[allow(dead_code)]
pub fn bfm_clear(map: &mut BitfieldMap, key: usize) {
    bfm_set(map, key, 0);
}

#[allow(dead_code)]
pub fn bfm_capacity(map: &BitfieldMap) -> usize {
    map.capacity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_all_zero() {
        let map = new_bitfield_map(16, 4);
        for i in 0..16 {
            assert_eq!(bfm_get(&map, i), 0);
        }
    }

    #[test]
    fn set_and_get() {
        let mut map = new_bitfield_map(8, 4);
        bfm_set(&mut map, 0, 5);
        assert_eq!(bfm_get(&map, 0), 5);
    }

    #[test]
    fn set_multiple_keys() {
        let mut map = new_bitfield_map(8, 4);
        bfm_set(&mut map, 0, 3);
        bfm_set(&mut map, 1, 7);
        assert_eq!(bfm_get(&map, 0), 3);
        assert_eq!(bfm_get(&map, 1), 7);
    }

    #[test]
    fn clear_resets_to_zero() {
        let mut map = new_bitfield_map(8, 4);
        bfm_set(&mut map, 2, 15);
        bfm_clear(&mut map, 2);
        assert_eq!(bfm_get(&map, 2), 0);
    }

    #[test]
    fn capacity_returned() {
        let map = new_bitfield_map(32, 2);
        assert_eq!(bfm_capacity(&map), 32);
    }

    #[test]
    fn out_of_bounds_get_returns_zero() {
        let map = new_bitfield_map(4, 4);
        assert_eq!(bfm_get(&map, 100), 0);
    }

    #[test]
    fn out_of_bounds_set_no_panic() {
        let mut map = new_bitfield_map(4, 4);
        bfm_set(&mut map, 100, 7); // should not panic
    }

    #[test]
    fn value_masked_to_bit_width() {
        let mut map = new_bitfield_map(8, 4);
        bfm_set(&mut map, 0, 0xFF); // should be masked to 4 bits = 0xF
        assert_eq!(bfm_get(&map, 0), 0xF);
    }

    #[test]
    fn eight_bit_values() {
        let mut map = new_bitfield_map(4, 8);
        bfm_set(&mut map, 0, 200);
        bfm_set(&mut map, 1, 100);
        assert_eq!(bfm_get(&map, 0), 200);
        assert_eq!(bfm_get(&map, 1), 100);
    }

    #[test]
    fn two_bit_values_many_keys() {
        let mut map = new_bitfield_map(32, 2);
        for i in 0..32usize {
            bfm_set(&mut map, i, (i % 4) as u64);
        }
        for i in 0..32usize {
            assert_eq!(bfm_get(&map, i), (i % 4) as u64);
        }
    }
}
