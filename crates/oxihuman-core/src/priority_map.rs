#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

/// A map that maintains entries ordered by priority.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PriorityMap {
    entries: BTreeMap<i32, Vec<(String, String)>>,
}

#[allow(dead_code)]
pub fn new_priority_map() -> PriorityMap {
    PriorityMap {
        entries: BTreeMap::new(),
    }
}

#[allow(dead_code)]
pub fn insert_priority(map: &mut PriorityMap, key: &str, value: &str, priority: i32) {
    map.entries
        .entry(priority)
        .or_default()
        .push((key.to_string(), value.to_string()));
}

#[allow(dead_code)]
pub fn get_highest(map: &PriorityMap) -> Option<(&str, &str)> {
    map.entries
        .iter()
        .next_back()
        .and_then(|(_, v)| v.first())
        .map(|(k, v)| (k.as_str(), v.as_str()))
}

#[allow(dead_code)]
pub fn remove_highest(map: &mut PriorityMap) -> Option<(String, String)> {
    if let Some((&prio, _)) = map.entries.iter().next_back() {
        let bucket = map.entries.get_mut(&prio)?;
        let item = bucket.remove(0);
        if bucket.is_empty() {
            map.entries.remove(&prio);
        }
        Some(item)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn priority_count(map: &PriorityMap) -> usize {
    map.entries.values().map(|v| v.len()).sum()
}

#[allow(dead_code)]
pub fn has_key_pm(map: &PriorityMap, key: &str) -> bool {
    map.entries
        .values()
        .any(|v| v.iter().any(|(k, _)| k == key))
}

#[allow(dead_code)]
pub fn clear_priority_map(map: &mut PriorityMap) {
    map.entries.clear();
}

#[allow(dead_code)]
pub fn priority_to_vec(map: &PriorityMap) -> Vec<(String, String, i32)> {
    let mut result = Vec::new();
    for (&prio, bucket) in &map.entries {
        for (k, v) in bucket {
            result.push((k.clone(), v.clone(), prio));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_priority_map() {
        let m = new_priority_map();
        assert_eq!(priority_count(&m), 0);
    }

    #[test]
    fn test_insert_and_count() {
        let mut m = new_priority_map();
        insert_priority(&mut m, "a", "1", 10);
        insert_priority(&mut m, "b", "2", 20);
        assert_eq!(priority_count(&m), 2);
    }

    #[test]
    fn test_get_highest() {
        let mut m = new_priority_map();
        insert_priority(&mut m, "low", "1", 1);
        insert_priority(&mut m, "high", "2", 100);
        let (k, _) = get_highest(&m).expect("should succeed");
        assert_eq!(k, "high");
    }

    #[test]
    fn test_remove_highest() {
        let mut m = new_priority_map();
        insert_priority(&mut m, "a", "1", 5);
        insert_priority(&mut m, "b", "2", 10);
        let (k, _) = remove_highest(&mut m).expect("should succeed");
        assert_eq!(k, "b");
        assert_eq!(priority_count(&m), 1);
    }

    #[test]
    fn test_has_key_pm() {
        let mut m = new_priority_map();
        insert_priority(&mut m, "x", "1", 5);
        assert!(has_key_pm(&m, "x"));
        assert!(!has_key_pm(&m, "y"));
    }

    #[test]
    fn test_clear_priority_map() {
        let mut m = new_priority_map();
        insert_priority(&mut m, "a", "1", 5);
        clear_priority_map(&mut m);
        assert_eq!(priority_count(&m), 0);
    }

    #[test]
    fn test_priority_to_vec() {
        let mut m = new_priority_map();
        insert_priority(&mut m, "a", "1", 5);
        insert_priority(&mut m, "b", "2", 10);
        let v = priority_to_vec(&m);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_get_highest_empty() {
        let m = new_priority_map();
        assert!(get_highest(&m).is_none());
    }

    #[test]
    fn test_remove_highest_empty() {
        let mut m = new_priority_map();
        assert!(remove_highest(&mut m).is_none());
    }

    #[test]
    fn test_same_priority() {
        let mut m = new_priority_map();
        insert_priority(&mut m, "a", "1", 5);
        insert_priority(&mut m, "b", "2", 5);
        assert_eq!(priority_count(&m), 2);
    }
}
