#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simplified skip list (sorted linked structure using Vec).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkipEntry {
    pub key: i64,
    pub value: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SkipList {
    pub entries: Vec<SkipEntry>,
}

#[allow(dead_code)]
pub fn new_skip_list() -> SkipList {
    SkipList {
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn skip_insert(sl: &mut SkipList, key: i64, val: &str) {
    match sl.entries.binary_search_by_key(&key, |e| e.key) {
        Ok(pos) => sl.entries[pos].value = val.to_string(),
        Err(pos) => sl.entries.insert(
            pos,
            SkipEntry {
                key,
                value: val.to_string(),
            },
        ),
    }
}

#[allow(dead_code)]
pub fn skip_find(sl: &SkipList, key: i64) -> Option<&str> {
    sl.entries
        .binary_search_by_key(&key, |e| e.key)
        .ok()
        .map(|pos| sl.entries[pos].value.as_str())
}

#[allow(dead_code)]
pub fn skip_remove(sl: &mut SkipList, key: i64) -> bool {
    match sl.entries.binary_search_by_key(&key, |e| e.key) {
        Ok(pos) => {
            sl.entries.remove(pos);
            true
        }
        Err(_) => false,
    }
}

#[allow(dead_code)]
pub fn skip_range(sl: &SkipList, lo: i64, hi: i64) -> Vec<(&i64, &str)> {
    sl.entries
        .iter()
        .filter(|e| e.key >= lo && e.key <= hi)
        .map(|e| (&e.key, e.value.as_str()))
        .collect()
}

#[allow(dead_code)]
pub fn skip_len(sl: &SkipList) -> usize {
    sl.entries.len()
}

#[allow(dead_code)]
pub fn sl_insert(list: &mut SkipList, key: i64, value: &str) {
    skip_insert(list, key, value);
}

#[allow(dead_code)]
pub fn sl_get(list: &SkipList, key: i64) -> Option<&str> {
    skip_find(list, key)
}

#[allow(dead_code)]
pub fn sl_remove(list: &mut SkipList, key: i64) -> bool {
    skip_remove(list, key)
}

#[allow(dead_code)]
pub fn sl_len(list: &SkipList) -> usize {
    skip_len(list)
}

#[allow(dead_code)]
pub fn sl_contains(list: &SkipList, key: i64) -> bool {
    skip_find(list, key).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_list_empty() {
        let sl = new_skip_list();
        assert_eq!(skip_len(&sl), 0);
    }

    #[test]
    fn insert_and_find() {
        let mut sl = new_skip_list();
        skip_insert(&mut sl, 10, "ten");
        assert_eq!(skip_find(&sl, 10), Some("ten"));
    }

    #[test]
    fn find_missing_returns_none() {
        let sl = new_skip_list();
        assert!(skip_find(&sl, 42).is_none());
    }

    #[test]
    fn sorted_after_insert() {
        let mut sl = new_skip_list();
        skip_insert(&mut sl, 30, "c");
        skip_insert(&mut sl, 10, "a");
        skip_insert(&mut sl, 20, "b");
        assert_eq!(sl.entries[0].key, 10);
        assert_eq!(sl.entries[1].key, 20);
        assert_eq!(sl.entries[2].key, 30);
    }

    #[test]
    fn update_existing_key() {
        let mut sl = new_skip_list();
        skip_insert(&mut sl, 5, "old");
        skip_insert(&mut sl, 5, "new");
        assert_eq!(skip_find(&sl, 5), Some("new"));
        assert_eq!(skip_len(&sl), 1);
    }

    #[test]
    fn remove_existing() {
        let mut sl = new_skip_list();
        skip_insert(&mut sl, 7, "seven");
        assert!(skip_remove(&mut sl, 7));
        assert!(skip_find(&sl, 7).is_none());
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut sl = new_skip_list();
        assert!(!skip_remove(&mut sl, 99));
    }

    #[test]
    fn range_query() {
        let mut sl = new_skip_list();
        for i in 0i64..10 {
            skip_insert(&mut sl, i, &i.to_string());
        }
        let r = skip_range(&sl, 3, 6);
        assert_eq!(r.len(), 4);
        assert_eq!(*r[0].0, 3);
        assert_eq!(*r[3].0, 6);
    }

    #[test]
    fn range_empty() {
        let mut sl = new_skip_list();
        skip_insert(&mut sl, 1, "a");
        skip_insert(&mut sl, 5, "b");
        let r = skip_range(&sl, 2, 4);
        assert!(r.is_empty());
    }

    #[test]
    fn len_tracks_inserts_and_removes() {
        let mut sl = new_skip_list();
        skip_insert(&mut sl, 1, "a");
        skip_insert(&mut sl, 2, "b");
        assert_eq!(skip_len(&sl), 2);
        skip_remove(&mut sl, 1);
        assert_eq!(skip_len(&sl), 1);
    }
}
