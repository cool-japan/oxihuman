// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cuckoo filter — space-efficient probabilistic membership with deletion support.

const BUCKET_SIZE: usize = 4;
const MAX_KICKS: usize = 500;
const FP_SEED: u64 = 0xbf58_476d_1ce4_e5b9;

type Fingerprint = u8;

/// Cuckoo filter with configurable number of buckets.
pub struct CuckooFilter {
    buckets: Vec<[Fingerprint; BUCKET_SIZE]>,
    num_buckets: usize,
    count: usize,
}

impl CuckooFilter {
    /// Create a new filter with at least `capacity` buckets (rounded to power of two).
    pub fn new(capacity: usize) -> Self {
        let n = capacity.next_power_of_two().max(2);
        CuckooFilter {
            buckets: vec![[0u8; BUCKET_SIZE]; n],
            num_buckets: n,
            count: 0,
        }
    }

    fn fingerprint(item: u64) -> Fingerprint {
        let h = item.wrapping_mul(FP_SEED).rotate_left(19);
        ((h & 0xff) as u8).max(1) /* 0 is reserved for empty slots */
    }

    fn index1(item: u64, n: usize) -> usize {
        let h = item.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        (h as usize) % n
    }

    fn index2(i1: usize, fp: Fingerprint, n: usize) -> usize {
        let h = (i1 as u64)
            .wrapping_add(fp as u64)
            .wrapping_mul(0x6c62_272e_07bb_0142);
        (h as usize) % n
    }

    /// Insert an item. Returns `true` on success, `false` if filter is full.
    pub fn insert(&mut self, item: u64) -> bool {
        let fp = Self::fingerprint(item);
        let i1 = Self::index1(item, self.num_buckets);
        let i2 = Self::index2(i1, fp, self.num_buckets);

        if self.try_insert_bucket(i1, fp) || self.try_insert_bucket(i2, fp) {
            self.count += 1;
            return true;
        }

        let mut cur_idx = i1;
        let mut cur_fp = fp;
        for _ in 0..MAX_KICKS {
            let slot = cur_idx % BUCKET_SIZE;
            std::mem::swap(&mut self.buckets[cur_idx][slot], &mut cur_fp);
            cur_idx = Self::index2(cur_idx, cur_fp, self.num_buckets);
            if self.try_insert_bucket(cur_idx, cur_fp) {
                self.count += 1;
                return true;
            }
        }
        false
    }

    fn try_insert_bucket(&mut self, idx: usize, fp: Fingerprint) -> bool {
        for slot in 0..BUCKET_SIZE {
            if self.buckets[idx][slot] == 0 {
                self.buckets[idx][slot] = fp;
                return true;
            }
        }
        false
    }

    /// Query membership.
    pub fn contains(&self, item: u64) -> bool {
        let fp = Self::fingerprint(item);
        let i1 = Self::index1(item, self.num_buckets);
        let i2 = Self::index2(i1, fp, self.num_buckets);
        self.buckets[i1].contains(&fp) || self.buckets[i2].contains(&fp)
    }

    /// Delete an item. Returns `true` if found and removed.
    pub fn delete(&mut self, item: u64) -> bool {
        let fp = Self::fingerprint(item);
        let i1 = Self::index1(item, self.num_buckets);
        let i2 = Self::index2(i1, fp, self.num_buckets);

        for bucket_idx in [i1, i2] {
            for slot in 0..BUCKET_SIZE {
                if self.buckets[bucket_idx][slot] == fp {
                    self.buckets[bucket_idx][slot] = 0;
                    self.count = self.count.saturating_sub(1);
                    return true;
                }
            }
        }
        false
    }

    /// Number of items in the filter.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if the filter is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut cf = CuckooFilter::new(64);
        assert!(cf.insert(123));
        assert!(cf.contains(123) /* inserted item must be found */);
    }

    #[test]
    fn test_not_inserted() {
        let cf = CuckooFilter::new(64);
        assert!(!cf.contains(999) /* empty filter has no items */);
    }

    #[test]
    fn test_delete() {
        let mut cf = CuckooFilter::new(64);
        cf.insert(77);
        assert!(cf.delete(77) /* deletion should succeed */);
    }

    #[test]
    fn test_len_tracks() {
        let mut cf = CuckooFilter::new(64);
        cf.insert(1);
        cf.insert(2);
        assert_eq!(cf.len(), 2 /* two items inserted */);
    }

    #[test]
    fn test_is_empty() {
        let cf = CuckooFilter::new(32);
        assert!(cf.is_empty() /* fresh filter is empty */);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut cf = CuckooFilter::new(256);
        for i in 0u64..50 {
            assert!(cf.insert(i) /* all inserts should succeed */);
        }
        for i in 0u64..50 {
            assert!(cf.contains(i));
        }
    }

    #[test]
    fn test_power_of_two_capacity() {
        let cf = CuckooFilter::new(7);
        assert_eq!(cf.num_buckets, 8 /* rounded to next power of two */);
    }

    #[test]
    fn test_fingerprint_non_zero() {
        let fp = CuckooFilter::fingerprint(0);
        assert!(fp >= 1 /* fingerprint is never 0 */);
    }

    #[test]
    fn test_delete_reduces_len() {
        let mut cf = CuckooFilter::new(64);
        cf.insert(55);
        cf.delete(55);
        assert!(cf.is_empty() /* after deleting the only item, len is 0 */);
    }
}
