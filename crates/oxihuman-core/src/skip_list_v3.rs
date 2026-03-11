// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skip list v3 — ordered map using a probabilistic layered linked structure.
//! Uses an index-arena approach to avoid raw pointers.

const MAX_LEVEL: usize = 16;

/// A single node in the skip list (stored in a Vec arena).
struct Node<K, V> {
    key: K,
    val: V,
    /// Forward pointers: index into arena, `usize::MAX` = null.
    forward: [usize; MAX_LEVEL],
}

impl<K, V> Node<K, V> {
    fn new(key: K, val: V, level: usize) -> Self {
        let mut forward = [usize::MAX; MAX_LEVEL];
        for f in forward.iter_mut().take(level) {
            *f = usize::MAX;
        }
        Node { key, val, forward }
    }
}

/// Ordered map backed by a skip list (index-arena, no unsafe).
pub struct SkipListV3<K: Ord, V> {
    /// Head sentinel: index 0 always exists.
    arena: Vec<Node<K, V>>,
    head_fwd: [usize; MAX_LEVEL],
    level: usize,
    count: usize,
    rng_state: u64,
}

impl<K: Ord, V> SkipListV3<K, V> {
    /// Create a new empty skip list.
    pub fn new() -> Self {
        SkipListV3 {
            arena: Vec::new(),
            head_fwd: [usize::MAX; MAX_LEVEL],
            level: 1,
            count: 0,
            rng_state: 0x853c_49e6_748f_ea9b,
        }
    }

    fn rand_level(&mut self) -> usize {
        /* xorshift64 */
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        let mut lvl = 1;
        while lvl < MAX_LEVEL && (self.rng_state & 3) == 0 {
            lvl += 1;
        }
        lvl
    }

    /// Insert or update a key-value pair.
    #[allow(clippy::needless_range_loop)]
    pub fn insert(&mut self, key: K, val: V) {
        /* collect update pointers: either None (head) or Some(arena index) */
        let mut update: [Option<usize>; MAX_LEVEL] = [None; MAX_LEVEL];

        /* Walk from highest level down, tracking the last node at each level */
        let mut cur: Option<usize> = None; /* None = head */

        for lv in (0..self.level).rev() {
            loop {
                /* next node at this level from cur */
                let next = match cur {
                    None => self.head_fwd[lv],
                    Some(idx) => self.arena[idx].forward[lv],
                };
                if next == usize::MAX {
                    break;
                }
                if self.arena[next].key < key {
                    cur = Some(next);
                } else {
                    break;
                }
            }
            update[lv] = cur;
        }

        /* Check if key already exists at level 0 */
        let candidate = match update[0] {
            None => self.head_fwd[0],
            Some(idx) => self.arena[idx].forward[0],
        };
        if candidate != usize::MAX && self.arena[candidate].key == key {
            self.arena[candidate].val = val;
            return;
        }

        let new_level = self.rand_level();
        if new_level > self.level {
            self.level = new_level;
        }

        let new_idx = self.arena.len();
        self.arena.push(Node::new(key, val, new_level));

        for lv in 0..new_level {
            let prev_next = match update[lv] {
                None => self.head_fwd[lv],
                Some(idx) => self.arena[idx].forward[lv],
            };
            self.arena[new_idx].forward[lv] = prev_next;
            match update[lv] {
                None => self.head_fwd[lv] = new_idx,
                Some(idx) => self.arena[idx].forward[lv] = new_idx,
            }
        }
        self.count += 1;
    }

    /// Look up a key, returning a reference to the value if found.
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut cur: Option<usize> = None;
        for lv in (0..self.level).rev() {
            loop {
                let next = match cur {
                    None => self.head_fwd[lv],
                    Some(idx) => self.arena[idx].forward[lv],
                };
                if next == usize::MAX {
                    break;
                }
                if &self.arena[next].key < key {
                    cur = Some(next);
                } else {
                    break;
                }
            }
        }
        let candidate = match cur {
            None => self.head_fwd[0],
            Some(idx) => self.arena[idx].forward[0],
        };
        if candidate != usize::MAX && &self.arena[candidate].key == key {
            Some(&self.arena[candidate].val)
        } else {
            None
        }
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl<K: Ord, V> Default for SkipListV3<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut sl: SkipListV3<u32, u32> = SkipListV3::new();
        sl.insert(10, 100);
        assert_eq!(sl.get(&10), Some(&100) /* inserted value found */);
    }

    #[test]
    fn test_get_missing() {
        let sl: SkipListV3<u32, u32> = SkipListV3::new();
        assert_eq!(sl.get(&5), None /* key not present */);
    }

    #[test]
    fn test_update_existing() {
        let mut sl: SkipListV3<u32, u32> = SkipListV3::new();
        sl.insert(1, 10);
        sl.insert(1, 20);
        assert_eq!(sl.get(&1), Some(&20) /* updated value */);
    }

    #[test]
    fn test_len_tracks() {
        let mut sl: SkipListV3<u32, u32> = SkipListV3::new();
        sl.insert(1, 1);
        sl.insert(2, 2);
        assert_eq!(sl.len(), 2 /* two distinct keys */);
    }

    #[test]
    fn test_is_empty() {
        let sl: SkipListV3<u32, u32> = SkipListV3::new();
        assert!(sl.is_empty() /* empty on creation */);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut sl: SkipListV3<u32, u32> = SkipListV3::new();
        for i in 0u32..20 {
            sl.insert(i, i * 2);
        }
        for i in 0u32..20 {
            assert_eq!(sl.get(&i), Some(&(i * 2)) /* each value matches */);
        }
    }

    #[test]
    fn test_large_keys() {
        let mut sl: SkipListV3<u64, u64> = SkipListV3::new();
        sl.insert(u64::MAX, 42);
        assert_eq!(sl.get(&u64::MAX), Some(&42) /* max key works */);
    }

    #[test]
    fn test_default() {
        let sl: SkipListV3<u32, u32> = SkipListV3::default();
        assert!(sl.is_empty() /* default is empty */);
    }

    #[test]
    fn test_update_does_not_increase_len() {
        let mut sl: SkipListV3<u32, u32> = SkipListV3::new();
        sl.insert(5, 50);
        sl.insert(5, 99);
        assert_eq!(sl.len(), 1 /* update should not increase len */);
    }
}
