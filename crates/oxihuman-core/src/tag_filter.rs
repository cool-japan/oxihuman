// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tag-based filtering: match items that have all required tags and none of the excluded tags.

use std::collections::HashSet;

/// A filter specification: include-all + exclude-any.
#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct TagFilter {
    pub required: HashSet<String>,
    pub excluded: HashSet<String>,
    pub apply_count: u64,
}

#[allow(dead_code)]
impl TagFilter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Require that matched items have this tag.
    pub fn require(&mut self, tag: &str) {
        self.required.insert(tag.to_string());
    }

    /// Exclude items that have this tag.
    pub fn exclude(&mut self, tag: &str) {
        self.excluded.insert(tag.to_string());
    }

    /// Remove a requirement.
    pub fn remove_required(&mut self, tag: &str) -> bool {
        self.required.remove(tag)
    }

    /// Remove an exclusion.
    pub fn remove_excluded(&mut self, tag: &str) -> bool {
        self.excluded.remove(tag)
    }

    /// Returns `true` if the given tag set passes this filter.
    pub fn matches(&mut self, tags: &HashSet<String>) -> bool {
        self.apply_count += 1;
        let has_all = self.required.iter().all(|r| tags.contains(r));
        let has_none_excluded = !self.excluded.iter().any(|e| tags.contains(e));
        has_all && has_none_excluded
    }

    /// Returns `true` if the given tag slice passes this filter.
    pub fn matches_slice(&mut self, tags: &[&str]) -> bool {
        let set: HashSet<String> = tags.iter().map(|s| s.to_string()).collect();
        self.matches(&set)
    }

    /// Filter a list of (id, tags) pairs, returning ids that pass.
    pub fn filter(&mut self, items: &[(u32, Vec<String>)]) -> Vec<u32> {
        items
            .iter()
            .filter(|(_, tags)| {
                let set: HashSet<String> = tags.iter().cloned().collect();
                self.apply_count += 1;
                let has_all = self.required.iter().all(|r| set.contains(r));
                let has_none = !self.excluded.iter().any(|e| set.contains(e));
                has_all && has_none
            })
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn required_count(&self) -> usize {
        self.required.len()
    }

    pub fn excluded_count(&self) -> usize {
        self.excluded.len()
    }

    pub fn is_trivial(&self) -> bool {
        self.required.is_empty() && self.excluded.is_empty()
    }

    pub fn clear(&mut self) {
        self.required.clear();
        self.excluded.clear();
    }
}

pub fn new_tag_filter() -> TagFilter {
    TagFilter::new()
}

pub fn tf_require(f: &mut TagFilter, tag: &str) {
    f.require(tag);
}

pub fn tf_exclude(f: &mut TagFilter, tag: &str) {
    f.exclude(tag);
}

pub fn tf_matches(f: &mut TagFilter, tags: &[&str]) -> bool {
    f.matches_slice(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial_filter_matches_all() {
        let mut f = new_tag_filter();
        assert!(tf_matches(&mut f, &["a", "b"]));
        assert!(tf_matches(&mut f, &[]));
    }

    #[test]
    fn required_tag_must_be_present() {
        let mut f = new_tag_filter();
        tf_require(&mut f, "visible");
        assert!(tf_matches(&mut f, &["visible", "active"]));
        assert!(!tf_matches(&mut f, &["active"]));
    }

    #[test]
    fn excluded_tag_blocks_match() {
        let mut f = new_tag_filter();
        tf_exclude(&mut f, "hidden");
        assert!(tf_matches(&mut f, &["visible"]));
        assert!(!tf_matches(&mut f, &["visible", "hidden"]));
    }

    #[test]
    fn require_and_exclude_combined() {
        let mut f = new_tag_filter();
        tf_require(&mut f, "active");
        tf_exclude(&mut f, "debug");
        assert!(tf_matches(&mut f, &["active"]));
        assert!(!tf_matches(&mut f, &["active", "debug"]));
        assert!(!tf_matches(&mut f, &["inactive"]));
    }

    #[test]
    fn filter_list() {
        let mut f = new_tag_filter();
        tf_require(&mut f, "mesh");
        let items = vec![
            (1u32, vec!["mesh".to_string(), "body".to_string()]),
            (2u32, vec!["texture".to_string()]),
            (3u32, vec!["mesh".to_string(), "hair".to_string()]),
        ];
        let ids = f.filter(&items);
        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn remove_required_relaxes_filter() {
        let mut f = new_tag_filter();
        tf_require(&mut f, "x");
        f.remove_required("x");
        assert!(tf_matches(&mut f, &[]));
    }

    #[test]
    fn is_trivial_when_empty() {
        let f = new_tag_filter();
        assert!(f.is_trivial());
    }

    #[test]
    fn not_trivial_when_has_rules() {
        let mut f = new_tag_filter();
        tf_require(&mut f, "a");
        assert!(!f.is_trivial());
    }

    #[test]
    fn clear_makes_trivial() {
        let mut f = new_tag_filter();
        tf_require(&mut f, "a");
        tf_exclude(&mut f, "b");
        f.clear();
        assert!(f.is_trivial());
    }

    #[test]
    fn required_and_excluded_counts() {
        let mut f = new_tag_filter();
        tf_require(&mut f, "a");
        tf_require(&mut f, "b");
        tf_exclude(&mut f, "c");
        assert_eq!(f.required_count(), 2);
        assert_eq!(f.excluded_count(), 1);
    }
}
