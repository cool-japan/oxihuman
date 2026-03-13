// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Multiset / frequency map (String → u32).

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CountedSetConfig {
    pub max_entries: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CountedSet {
    pub counts: HashMap<String, u32>,
}

#[allow(dead_code)]
pub fn default_counted_set_config() -> CountedSetConfig {
    CountedSetConfig { max_entries: 1024 }
}

#[allow(dead_code)]
pub fn new_counted_set() -> CountedSet {
    CountedSet { counts: HashMap::new() }
}

#[allow(dead_code)]
pub fn cs_add(set: &mut CountedSet, key: &str) {
    *set.counts.entry(key.to_string()).or_insert(0) += 1;
}

#[allow(dead_code)]
pub fn cs_add_n(set: &mut CountedSet, key: &str, n: u32) {
    *set.counts.entry(key.to_string()).or_insert(0) += n;
}

#[allow(dead_code)]
pub fn cs_remove_one(set: &mut CountedSet, key: &str) {
    if let Some(v) = set.counts.get_mut(key) {
        if *v <= 1 {
            set.counts.remove(key);
        } else {
            *v -= 1;
        }
    }
}

#[allow(dead_code)]
pub fn cs_count(set: &CountedSet, key: &str) -> u32 {
    *set.counts.get(key).unwrap_or(&0)
}

#[allow(dead_code)]
pub fn cs_total(set: &CountedSet) -> u32 {
    set.counts.values().sum()
}

#[allow(dead_code)]
pub fn cs_most_common(set: &CountedSet) -> Option<String> {
    set.counts
        .iter()
        .max_by_key(|(_, v)| *v)
        .map(|(k, _)| k.clone())
}

#[allow(dead_code)]
pub fn cs_clear(set: &mut CountedSet) {
    set.counts.clear();
}

#[allow(dead_code)]
pub fn cs_unique_count(set: &CountedSet) -> usize {
    set.counts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_counted_set_config();
        assert_eq!(cfg.max_entries, 1024);
    }

    #[test]
    fn test_new_counted_set_empty() {
        let s = new_counted_set();
        assert_eq!(cs_unique_count(&s), 0);
    }

    #[test]
    fn test_cs_add() {
        let mut s = new_counted_set();
        cs_add(&mut s, "apple");
        cs_add(&mut s, "apple");
        assert_eq!(cs_count(&s, "apple"), 2);
    }

    #[test]
    fn test_cs_add_n() {
        let mut s = new_counted_set();
        cs_add_n(&mut s, "banana", 5);
        assert_eq!(cs_count(&s, "banana"), 5);
    }

    #[test]
    fn test_cs_remove_one() {
        let mut s = new_counted_set();
        cs_add_n(&mut s, "x", 3);
        cs_remove_one(&mut s, "x");
        assert_eq!(cs_count(&s, "x"), 2);
    }

    #[test]
    fn test_cs_remove_one_last() {
        let mut s = new_counted_set();
        cs_add(&mut s, "y");
        cs_remove_one(&mut s, "y");
        assert_eq!(cs_count(&s, "y"), 0);
        assert_eq!(cs_unique_count(&s), 0);
    }

    #[test]
    fn test_cs_total() {
        let mut s = new_counted_set();
        cs_add_n(&mut s, "a", 3);
        cs_add_n(&mut s, "b", 2);
        assert_eq!(cs_total(&s), 5);
    }

    #[test]
    fn test_cs_most_common() {
        let mut s = new_counted_set();
        cs_add_n(&mut s, "rare", 1);
        cs_add_n(&mut s, "common", 10);
        assert_eq!(cs_most_common(&s).expect("should succeed"), "common");
    }

    #[test]
    fn test_cs_clear() {
        let mut s = new_counted_set();
        cs_add_n(&mut s, "z", 5);
        cs_clear(&mut s);
        assert_eq!(cs_total(&s), 0);
    }
}
