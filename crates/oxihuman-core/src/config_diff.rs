// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Keys present in `new` but not in `old`.
pub fn config_added_keys<'a>(
    old: &'a HashMap<String, String>,
    new: &'a HashMap<String, String>,
) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = new
        .keys()
        .filter(|k| !old.contains_key(*k))
        .map(|k| k.as_str())
        .collect();
    result.sort();
    result
}

/// Keys present in `old` but not in `new`.
pub fn config_removed_keys<'a>(
    old: &'a HashMap<String, String>,
    new: &'a HashMap<String, String>,
) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = old
        .keys()
        .filter(|k| !new.contains_key(*k))
        .map(|k| k.as_str())
        .collect();
    result.sort();
    result
}

/// Keys present in both but with different values.
pub fn config_changed_keys<'a>(
    old: &'a HashMap<String, String>,
    new: &'a HashMap<String, String>,
) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = old
        .keys()
        .filter(|k| {
            new.get(*k).map(|v| v.as_str()) != old.get(*k).map(|v| v.as_str())
                && new.contains_key(*k)
        })
        .map(|k| k.as_str())
        .collect();
    result.sort();
    result
}

/// Total number of added + removed + changed keys.
pub fn config_diff_count(old: &HashMap<String, String>, new: &HashMap<String, String>) -> usize {
    config_added_keys(old, new).len()
        + config_removed_keys(old, new).len()
        + config_changed_keys(old, new).len()
}

/// Returns true if both maps are identical.
pub fn config_is_identical(old: &HashMap<String, String>, new: &HashMap<String, String>) -> bool {
    config_diff_count(old, new) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_identical_maps() {
        /* identical maps have no diff */
        let a = make_map(&[("x", "1")]);
        let b = make_map(&[("x", "1")]);
        assert!(config_is_identical(&a, &b));
    }

    #[test]
    fn test_added_keys() {
        /* new key detected as added */
        let old = make_map(&[("a", "1")]);
        let new = make_map(&[("a", "1"), ("b", "2")]);
        let added = config_added_keys(&old, &new);
        assert_eq!(added, vec!["b"]);
    }

    #[test]
    fn test_removed_keys() {
        /* removed key detected */
        let old = make_map(&[("a", "1"), ("b", "2")]);
        let new = make_map(&[("a", "1")]);
        let removed = config_removed_keys(&old, &new);
        assert_eq!(removed, vec!["b"]);
    }

    #[test]
    fn test_changed_keys() {
        /* changed value detected */
        let old = make_map(&[("x", "old")]);
        let new = make_map(&[("x", "new")]);
        let changed = config_changed_keys(&old, &new);
        assert_eq!(changed, vec!["x"]);
    }

    #[test]
    fn test_diff_count() {
        /* diff count sums all changes */
        let old = make_map(&[("a", "1"), ("b", "2")]);
        let new = make_map(&[("a", "9"), ("c", "3")]);
        assert_eq!(config_diff_count(&old, &new), 3);
    }

    #[test]
    fn test_empty_maps() {
        /* two empty maps are identical */
        let a: HashMap<String, String> = HashMap::new();
        let b: HashMap<String, String> = HashMap::new();
        assert!(config_is_identical(&a, &b));
    }
}
