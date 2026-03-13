#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Configuration merging and diffing utilities.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigMerge {
    base: HashMap<String, String>,
    overlay: HashMap<String, String>,
    strategy: String,
}

#[allow(dead_code)]
pub fn merge_configs(
    base: &HashMap<String, String>,
    overlay: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut result = base.clone();
    for (k, v) in overlay {
        result.insert(k.clone(), v.clone());
    }
    result
}

#[allow(dead_code)]
pub fn config_diff(
    a: &HashMap<String, String>,
    b: &HashMap<String, String>,
) -> HashMap<String, (Option<String>, Option<String>)> {
    let mut diff = HashMap::new();
    for (k, v) in a {
        match b.get(k) {
            Some(bv) if bv != v => {
                diff.insert(k.clone(), (Some(v.clone()), Some(bv.clone())));
            }
            None => {
                diff.insert(k.clone(), (Some(v.clone()), None));
            }
            _ => {}
        }
    }
    for (k, v) in b {
        if !a.contains_key(k) {
            diff.insert(k.clone(), (None, Some(v.clone())));
        }
    }
    diff
}

#[allow(dead_code)]
pub fn config_patch(
    base: &mut HashMap<String, String>,
    patch: &HashMap<String, Option<String>>,
) {
    for (k, v) in patch {
        match v {
            Some(val) => {
                base.insert(k.clone(), val.clone());
            }
            None => {
                base.remove(k);
            }
        }
    }
}

#[allow(dead_code)]
pub fn diff_count(
    diff: &HashMap<String, (Option<String>, Option<String>)>,
) -> usize {
    diff.len()
}

#[allow(dead_code)]
pub fn patch_count(patch: &HashMap<String, Option<String>>) -> usize {
    patch.len()
}

#[allow(dead_code)]
pub fn merge_strategy(_cm: &ConfigMerge) -> &str {
    &_cm.strategy
}

#[allow(dead_code)]
pub fn merge_to_json(
    a: &HashMap<String, String>,
    b: &HashMap<String, String>,
) -> String {
    let merged = merge_configs(a, b);
    let entries: Vec<String> = merged
        .iter()
        .map(|(k, v)| format!(r#""{}":"{}""#, k, v))
        .collect();
    format!("{{{}}}", entries.join(","))
}

#[allow(dead_code)]
pub fn validate_merge(merged: &HashMap<String, String>) -> bool {
    !merged.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_empty() {
        let a = HashMap::new();
        let b = HashMap::new();
        assert!(merge_configs(&a, &b).is_empty());
    }

    #[test]
    fn test_merge_overlay() {
        let mut a = HashMap::new();
        a.insert("k".to_string(), "v1".to_string());
        let mut b = HashMap::new();
        b.insert("k".to_string(), "v2".to_string());
        let m = merge_configs(&a, &b);
        assert_eq!(m.get("k").expect("should succeed"), "v2");
    }

    #[test]
    fn test_merge_disjoint() {
        let mut a = HashMap::new();
        a.insert("a".to_string(), "1".to_string());
        let mut b = HashMap::new();
        b.insert("b".to_string(), "2".to_string());
        let m = merge_configs(&a, &b);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_config_diff_same() {
        let mut a = HashMap::new();
        a.insert("k".to_string(), "v".to_string());
        let d = config_diff(&a, &a);
        assert!(d.is_empty());
    }

    #[test]
    fn test_config_diff_changed() {
        let mut a = HashMap::new();
        a.insert("k".to_string(), "v1".to_string());
        let mut b = HashMap::new();
        b.insert("k".to_string(), "v2".to_string());
        let d = config_diff(&a, &b);
        assert_eq!(diff_count(&d), 1);
    }

    #[test]
    fn test_config_patch_add() {
        let mut base = HashMap::new();
        let mut patch = HashMap::new();
        patch.insert("k".to_string(), Some("v".to_string()));
        config_patch(&mut base, &patch);
        assert_eq!(base.get("k").expect("should succeed"), "v");
    }

    #[test]
    fn test_config_patch_remove() {
        let mut base = HashMap::new();
        base.insert("k".to_string(), "v".to_string());
        let mut patch = HashMap::new();
        patch.insert("k".to_string(), None);
        config_patch(&mut base, &patch);
        assert!(!base.contains_key("k"));
    }

    #[test]
    fn test_validate_merge() {
        let mut m = HashMap::new();
        m.insert("a".to_string(), "b".to_string());
        assert!(validate_merge(&m));
    }

    #[test]
    fn test_validate_merge_empty() {
        let m = HashMap::new();
        assert!(!validate_merge(&m));
    }

    #[test]
    fn test_merge_to_json() {
        let a = HashMap::new();
        let mut b = HashMap::new();
        b.insert("x".to_string(), "1".to_string());
        let json = merge_to_json(&a, &b);
        assert!(json.contains("\"x\":\"1\""));
    }
}
