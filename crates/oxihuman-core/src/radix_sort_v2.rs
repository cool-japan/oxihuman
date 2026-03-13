// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Radix sort for `u32` arrays using 8-bit digit passes.

/// Sort a slice of `u32` values in ascending order using LSD radix sort.
pub fn radix_sort_u32(data: &mut [u32]) {
    if data.len() < 2 {
        return;
    }
    let mut buf = vec![0u32; data.len()];
    for shift in [0u32, 8, 16, 24] {
        let mut counts = [0usize; 256];
        for &v in data.iter() {
            counts[((v >> shift) & 0xFF) as usize] += 1;
        }
        let mut offsets = [0usize; 256];
        let mut sum = 0;
        for i in 0..256 {
            offsets[i] = sum;
            sum += counts[i];
        }
        for &v in data.iter() {
            let digit = ((v >> shift) & 0xFF) as usize;
            buf[offsets[digit]] = v;
            offsets[digit] += 1;
        }
        data.copy_from_slice(&buf);
    }
}

/// Sort a slice of `u32` values in descending order.
pub fn radix_sort_u32_desc(data: &mut [u32]) {
    radix_sort_u32(data);
    data.reverse();
}

/// Return a sorted copy without modifying the input.
pub fn radix_sorted(data: &[u32]) -> Vec<u32> {
    let mut copy = data.to_vec();
    radix_sort_u32(&mut copy);
    copy
}

/// Count inversions in a u32 slice (brute-force O(n²) for small n).
pub fn count_inversions(data: &[u32]) -> u64 {
    let mut inv = 0u64;
    for i in 0..data.len() {
        for j in i + 1..data.len() {
            if data[i] > data[j] {
                inv += 1;
            }
        }
    }
    inv
}

/// Partition `data` into values less than `pivot` and greater-or-equal.
pub fn partition_by(data: &[u32], pivot: u32) -> (Vec<u32>, Vec<u32>) {
    let lo: Vec<u32> = data.iter().copied().filter(|&x| x < pivot).collect();
    let hi: Vec<u32> = data.iter().copied().filter(|&x| x >= pivot).collect();
    (lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_ascending() {
        let mut v = vec![5u32, 3, 8, 1, 9, 2];
        radix_sort_u32(&mut v);
        assert_eq!(v, vec![1, 2, 3, 5, 8, 9] /* sorted */);
    }

    #[test]
    fn test_sort_single() {
        let mut v = vec![42u32];
        radix_sort_u32(&mut v);
        assert_eq!(v, vec![42] /* unchanged */);
    }

    #[test]
    fn test_sort_empty() {
        let mut v: Vec<u32> = vec![];
        radix_sort_u32(&mut v);
        assert!(v.is_empty() /* empty */);
    }

    #[test]
    fn test_sort_desc() {
        let mut v = vec![1u32, 5, 3];
        radix_sort_u32_desc(&mut v);
        assert_eq!(v, vec![5, 3, 1] /* descending */);
    }

    #[test]
    fn test_radix_sorted_copy() {
        let v = vec![4u32, 2, 7];
        let sorted = radix_sorted(&v);
        assert_eq!(sorted, vec![2, 4, 7] /* sorted copy */);
        assert_eq!(v, vec![4, 2, 7] /* original unchanged */);
    }

    #[test]
    fn test_all_equal() {
        let mut v = vec![3u32; 10];
        radix_sort_u32(&mut v);
        assert!(v.iter().all(|&x| x == 3) /* all equal */);
    }

    #[test]
    fn test_large_values() {
        let mut v = vec![u32::MAX, 0, u32::MAX / 2];
        radix_sort_u32(&mut v);
        assert_eq!(v[0], 0 /* smallest first */);
        assert_eq!(v[2], u32::MAX);
    }

    #[test]
    fn test_count_inversions() {
        let v = vec![3u32, 1, 2];
        assert_eq!(count_inversions(&v), 2 /* two inversions */);
    }

    #[test]
    fn test_partition_by() {
        let v = vec![1u32, 5, 3, 7, 2];
        let (lo, hi) = partition_by(&v, 4);
        assert!(lo.iter().all(|&x| x < 4) /* all lo < 4 */);
        assert!(hi.iter().all(|&x| x >= 4));
    }

    #[test]
    fn test_sorted_has_zero_inversions() {
        let mut v = vec![10u32, 2, 8, 4];
        radix_sort_u32(&mut v);
        assert_eq!(count_inversions(&v), 0 /* sorted = no inversions */);
    }
}
