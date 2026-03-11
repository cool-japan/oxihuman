// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Radix sort utilities for u32 and u64 keys.

/// Sort a slice of u32 values in ascending order using LSD radix sort.
pub fn radix_sort_u32(data: &mut [u32]) {
    let n = data.len();
    if n < 2 {
        return;
    }
    let mut buf = vec![0u32; n];
    for shift in [0u32, 8, 16, 24] {
        let mut count = [0usize; 256];
        for &v in data.iter() {
            count[((v >> shift) & 0xFF) as usize] += 1;
        }
        let mut prefix = [0usize; 256];
        for i in 1..256 {
            prefix[i] = prefix[i - 1] + count[i - 1];
        }
        for &v in data.iter() {
            let b = ((v >> shift) & 0xFF) as usize;
            buf[prefix[b]] = v;
            prefix[b] += 1;
        }
        data.clone_from_slice(&buf);
    }
}

/// Sort a slice of u64 values in ascending order using LSD radix sort.
pub fn radix_sort_u64(data: &mut [u64]) {
    let n = data.len();
    if n < 2 {
        return;
    }
    let mut buf = vec![0u64; n];
    for pass in 0..8u32 {
        let shift = pass * 8;
        let mut count = [0usize; 256];
        for &v in data.iter() {
            count[((v >> shift) & 0xFF) as usize] += 1;
        }
        let mut prefix = [0usize; 256];
        for i in 1..256 {
            prefix[i] = prefix[i - 1] + count[i - 1];
        }
        for &v in data.iter() {
            let b = ((v >> shift) & 0xFF) as usize;
            buf[prefix[b]] = v;
            prefix[b] += 1;
        }
        data.clone_from_slice(&buf);
    }
}

/// Sort (key, value) pairs by u32 key ascending.
pub fn radix_sort_pairs_u32<V: Clone>(pairs: &mut [(u32, V)]) {
    let n = pairs.len();
    if n < 2 {
        return;
    }
    let mut buf: Vec<(u32, V)> = pairs.to_owned();
    for shift in [0u32, 8, 16, 24] {
        let mut count = [0usize; 256];
        for (k, _) in pairs.iter() {
            count[((k >> shift) & 0xFF) as usize] += 1;
        }
        let mut prefix = [0usize; 256];
        for i in 1..256 {
            prefix[i] = prefix[i - 1] + count[i - 1];
        }
        for (k, v) in pairs.iter() {
            let b = ((k >> shift) & 0xFF) as usize;
            buf[prefix[b]] = (*k, v.clone());
            prefix[b] += 1;
        }
        pairs.clone_from_slice(&buf);
    }
}

/// Check if a u32 slice is sorted ascending.
pub fn is_sorted_u32(data: &[u32]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

/// Check if a u64 slice is sorted ascending.
pub fn is_sorted_u64(data: &[u64]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

/// Count distinct values in a sorted u32 slice.
pub fn count_distinct_u32(sorted: &[u32]) -> usize {
    if sorted.is_empty() {
        return 0;
    }
    let mut count = 1usize;
    for i in 1..sorted.len() {
        if sorted[i] != sorted[i - 1] {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_u32_basic() {
        let mut v = vec![5u32, 2, 8, 1, 9, 3];
        radix_sort_u32(&mut v);
        assert!(is_sorted_u32(&v));
    }

    #[test]
    fn sort_u32_single() {
        let mut v = vec![42u32];
        radix_sort_u32(&mut v);
        assert_eq!(v, vec![42]);
    }

    #[test]
    fn sort_u32_empty() {
        let mut v: Vec<u32> = vec![];
        radix_sort_u32(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn sort_u64_basic() {
        let mut v = vec![100u64, 1, 9999, 42, 0];
        radix_sort_u64(&mut v);
        assert!(is_sorted_u64(&v));
    }

    #[test]
    fn sort_u32_large_values() {
        let mut v = vec![u32::MAX, 0, u32::MAX / 2, 1];
        radix_sort_u32(&mut v);
        assert!(is_sorted_u32(&v));
    }

    #[test]
    fn sort_pairs() {
        let mut pairs = vec![(3u32, "c"), (1u32, "a"), (2u32, "b")];
        radix_sort_pairs_u32(&mut pairs);
        assert_eq!(pairs[0].0, 1);
        assert_eq!(pairs[2].0, 3);
    }

    #[test]
    fn count_distinct_sorted() {
        let v = vec![1u32, 1, 2, 3, 3, 3, 4];
        assert_eq!(count_distinct_u32(&v), 4);
    }

    #[test]
    fn already_sorted() {
        let mut v = vec![1u32, 2, 3, 4, 5];
        radix_sort_u32(&mut v);
        assert!(is_sorted_u32(&v));
    }

    #[test]
    fn sort_duplicates() {
        let mut v = vec![5u32, 5, 5, 1, 1];
        radix_sort_u32(&mut v);
        assert!(is_sorted_u32(&v));
    }
}
