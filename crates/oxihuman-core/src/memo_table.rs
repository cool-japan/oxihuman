// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Memo table: memoizes pure function results keyed by u64 hash.

use std::collections::HashMap;

/// Single memoized entry.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemoEntry<V> {
    pub value: V,
    pub access_count: u32,
}

/// Memo table.
#[derive(Debug)]
#[allow(dead_code)]
pub struct MemoTable<V> {
    entries: HashMap<u64, MemoEntry<V>>,
    hit_count: u64,
    miss_count: u64,
}

/// Create a new MemoTable.
#[allow(dead_code)]
pub fn new_memo_table<V>() -> MemoTable<V> {
    MemoTable {
        entries: HashMap::new(),
        hit_count: 0,
        miss_count: 0,
    }
}

/// Look up a key; counts hit/miss.
#[allow(dead_code)]
pub fn memo_get<V>(table: &mut MemoTable<V>, key: u64) -> Option<&V> {
    if let Some(entry) = table.entries.get_mut(&key) {
        entry.access_count += 1;
        table.hit_count += 1;
        Some(&entry.value)
    } else {
        table.miss_count += 1;
        None
    }
}

/// Insert a computed value.
#[allow(dead_code)]
pub fn memo_set<V>(table: &mut MemoTable<V>, key: u64, value: V) {
    table.entries.insert(
        key,
        MemoEntry {
            value,
            access_count: 0,
        },
    );
}

/// Whether a key is present.
#[allow(dead_code)]
pub fn memo_contains<V>(table: &MemoTable<V>, key: u64) -> bool {
    table.entries.contains_key(&key)
}

/// Invalidate a key.
#[allow(dead_code)]
pub fn memo_invalidate<V>(table: &mut MemoTable<V>, key: u64) -> bool {
    table.entries.remove(&key).is_some()
}

/// Clear all entries.
#[allow(dead_code)]
pub fn memo_clear<V>(table: &mut MemoTable<V>) {
    table.entries.clear();
}

/// Number of stored entries.
#[allow(dead_code)]
pub fn memo_len<V>(table: &MemoTable<V>) -> usize {
    table.entries.len()
}

/// Hit rate (0.0..=1.0).
#[allow(dead_code)]
pub fn memo_hit_rate<V>(table: &MemoTable<V>) -> f32 {
    let total = table.hit_count + table.miss_count;
    if total == 0 {
        return 0.0;
    }
    table.hit_count as f32 / total as f32
}

/// Hit count.
#[allow(dead_code)]
pub fn memo_hits<V>(table: &MemoTable<V>) -> u64 {
    table.hit_count
}

/// Miss count.
#[allow(dead_code)]
pub fn memo_misses<V>(table: &MemoTable<V>) -> u64 {
    table.miss_count
}

/// Access count for a specific key.
#[allow(dead_code)]
pub fn memo_access_count<V>(table: &MemoTable<V>, key: u64) -> u32 {
    table.entries.get(&key).map(|e| e.access_count).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut t: MemoTable<u32> = new_memo_table();
        memo_set(&mut t, 1, 100);
        assert_eq!(memo_get(&mut t, 1), Some(&100));
    }

    #[test]
    fn test_miss() {
        let mut t: MemoTable<u32> = new_memo_table();
        assert_eq!(memo_get(&mut t, 99), None);
        assert_eq!(memo_misses(&t), 1);
    }

    #[test]
    fn test_hit_count() {
        let mut t: MemoTable<u32> = new_memo_table();
        memo_set(&mut t, 5, 50);
        memo_get(&mut t, 5);
        memo_get(&mut t, 5);
        assert_eq!(memo_hits(&t), 2);
    }

    #[test]
    fn test_hit_rate() {
        let mut t: MemoTable<u32> = new_memo_table();
        memo_set(&mut t, 1, 1);
        memo_get(&mut t, 1); // hit
        memo_get(&mut t, 2); // miss
        let hr = memo_hit_rate(&t);
        assert!((hr - 0.5_f32).abs() < 1e-5);
    }

    #[test]
    fn test_invalidate() {
        let mut t: MemoTable<u32> = new_memo_table();
        memo_set(&mut t, 7, 77);
        assert!(memo_invalidate(&mut t, 7));
        assert!(!memo_contains(&t, 7));
    }

    #[test]
    fn test_contains() {
        let mut t: MemoTable<u32> = new_memo_table();
        memo_set(&mut t, 3, 30);
        assert!(memo_contains(&t, 3));
        assert!(!memo_contains(&t, 4));
    }

    #[test]
    fn test_clear() {
        let mut t: MemoTable<u32> = new_memo_table();
        memo_set(&mut t, 1, 1);
        memo_clear(&mut t);
        assert_eq!(memo_len(&t), 0);
    }

    #[test]
    fn test_access_count() {
        let mut t: MemoTable<u32> = new_memo_table();
        memo_set(&mut t, 10, 100);
        memo_get(&mut t, 10);
        memo_get(&mut t, 10);
        memo_get(&mut t, 10);
        assert_eq!(memo_access_count(&t, 10), 3);
    }

    #[test]
    fn test_len() {
        let mut t: MemoTable<u32> = new_memo_table();
        memo_set(&mut t, 1, 1);
        memo_set(&mut t, 2, 2);
        assert_eq!(memo_len(&t), 2);
    }
}
