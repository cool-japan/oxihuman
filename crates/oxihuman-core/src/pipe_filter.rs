// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A composable filter pipeline operating on string records.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FilterOp {
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    MinLen(usize),
    MaxLen(usize),
    Not(Box<FilterOp>),
}

impl FilterOp {
    pub fn apply(&self, s: &str) -> bool {
        match self {
            FilterOp::Contains(sub) => s.contains(sub.as_str()),
            FilterOp::StartsWith(pre) => s.starts_with(pre.as_str()),
            FilterOp::EndsWith(suf) => s.ends_with(suf.as_str()),
            FilterOp::MinLen(n) => s.len() >= *n,
            FilterOp::MaxLen(n) => s.len() <= *n,
            FilterOp::Not(inner) => !inner.apply(s),
        }
    }
}

#[allow(dead_code)]
pub struct PipeFilter {
    filters: Vec<FilterOp>,
    passed: u64,
    rejected: u64,
}

#[allow(dead_code)]
impl PipeFilter {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            passed: 0,
            rejected: 0,
        }
    }
    pub fn add(&mut self, op: FilterOp) {
        self.filters.push(op);
    }
    pub fn apply<'a>(&mut self, items: &[&'a str]) -> Vec<&'a str> {
        let mut out = Vec::new();
        for &item in items {
            if self.filters.iter().all(|f| f.apply(item)) {
                self.passed += 1;
                out.push(item);
            } else {
                self.rejected += 1;
            }
        }
        out
    }
    pub fn passes(&self, s: &str) -> bool {
        self.filters.iter().all(|f| f.apply(s))
    }
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
    pub fn passed(&self) -> u64 {
        self.passed
    }
    pub fn rejected(&self) -> u64 {
        self.rejected
    }
    pub fn clear(&mut self) {
        self.filters.clear();
    }
    pub fn reset_stats(&mut self) {
        self.passed = 0;
        self.rejected = 0;
    }
}

impl Default for PipeFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_pipe_filter() -> PipeFilter {
    PipeFilter::new()
}
#[allow(dead_code)]
pub fn pf_add(f: &mut PipeFilter, op: FilterOp) {
    f.add(op);
}
#[allow(dead_code)]
pub fn pf_apply<'a>(f: &mut PipeFilter, items: &[&'a str]) -> Vec<&'a str> {
    f.apply(items)
}
#[allow(dead_code)]
pub fn pf_passes(f: &PipeFilter, s: &str) -> bool {
    f.passes(s)
}
#[allow(dead_code)]
pub fn pf_count(f: &PipeFilter) -> usize {
    f.filter_count()
}
#[allow(dead_code)]
pub fn pf_is_empty(f: &PipeFilter) -> bool {
    f.is_empty()
}
#[allow(dead_code)]
pub fn pf_passed(f: &PipeFilter) -> u64 {
    f.passed()
}
#[allow(dead_code)]
pub fn pf_rejected(f: &PipeFilter) -> u64 {
    f.rejected()
}
#[allow(dead_code)]
pub fn pf_clear(f: &mut PipeFilter) {
    f.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_contains_filter() {
        let mut f = new_pipe_filter();
        pf_add(&mut f, FilterOp::Contains("foo".to_string()));
        let result = pf_apply(&mut f, &["foobar", "baz", "foo"]);
        assert_eq!(result, vec!["foobar", "foo"]);
    }
    #[test]
    fn test_starts_with() {
        let mut f = new_pipe_filter();
        pf_add(&mut f, FilterOp::StartsWith("pre".to_string()));
        let result = pf_apply(&mut f, &["prefix", "no", "prefix2"]);
        assert_eq!(result.len(), 2);
    }
    #[test]
    fn test_ends_with() {
        let mut f = new_pipe_filter();
        pf_add(&mut f, FilterOp::EndsWith(".rs".to_string()));
        let result = pf_apply(&mut f, &["main.rs", "lib.rs", "readme.md"]);
        assert_eq!(result.len(), 2);
    }
    #[test]
    fn test_min_len() {
        let mut f = new_pipe_filter();
        pf_add(&mut f, FilterOp::MinLen(4));
        let result = pf_apply(&mut f, &["ab", "abcd", "abcde"]);
        assert_eq!(result.len(), 2);
    }
    #[test]
    fn test_not_filter() {
        let mut f = new_pipe_filter();
        pf_add(
            &mut f,
            FilterOp::Not(Box::new(FilterOp::Contains("bad".to_string()))),
        );
        let result = pf_apply(&mut f, &["good", "bad_item", "ok"]);
        assert_eq!(result.len(), 2);
    }
    #[test]
    fn test_passes() {
        let mut f = new_pipe_filter();
        pf_add(&mut f, FilterOp::MinLen(3));
        assert!(pf_passes(&f, "abc"));
        assert!(!pf_passes(&f, "ab"));
    }
    #[test]
    fn test_stats() {
        let mut f = new_pipe_filter();
        pf_add(&mut f, FilterOp::MinLen(3));
        pf_apply(&mut f, &["ab", "abc"]);
        assert_eq!(pf_passed(&f), 1);
        assert_eq!(pf_rejected(&f), 1);
    }
    #[test]
    fn test_clear() {
        let mut f = new_pipe_filter();
        pf_add(&mut f, FilterOp::MinLen(1));
        pf_clear(&mut f);
        assert!(pf_is_empty(&f));
    }
    #[test]
    fn test_no_filters_passes_all() {
        let mut f = new_pipe_filter();
        let result = pf_apply(&mut f, &["a", "b", "c"]);
        assert_eq!(result.len(), 3);
    }
    #[test]
    fn test_max_len() {
        let mut f = new_pipe_filter();
        pf_add(&mut f, FilterOp::MaxLen(3));
        let result = pf_apply(&mut f, &["ab", "abc", "abcd"]);
        assert_eq!(result.len(), 2);
    }
}
