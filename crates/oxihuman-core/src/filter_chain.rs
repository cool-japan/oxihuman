#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Filter chain for composing multiple predicates.

/// A filter predicate: a name and a boxed function.
#[allow(dead_code)]
pub struct FilterPredicate {
    pub name: String,
    /// Returns true if the value passes this filter.
    pub test: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

impl std::fmt::Debug for FilterPredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterPredicate").field("name", &self.name).finish()
    }
}

/// A chain of filter predicates applied in sequence.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct FilterChain {
    filters: Vec<FilterPredicate>,
}

/// Create a new empty `FilterChain`.
#[allow(dead_code)]
pub fn new_filter_chain() -> FilterChain {
    FilterChain { filters: Vec::new() }
}

/// Add a filter to the chain.
#[allow(dead_code)]
pub fn add_filter(chain: &mut FilterChain, name: &str, predicate: impl Fn(&str) -> bool + Send + Sync + 'static) {
    chain.filters.push(FilterPredicate { name: name.to_string(), test: Box::new(predicate) });
}

/// Remove a filter by name. Returns true if removed.
#[allow(dead_code)]
pub fn remove_filter(chain: &mut FilterChain, name: &str) -> bool {
    let len_before = chain.filters.len();
    chain.filters.retain(|f| f.name != name);
    chain.filters.len() < len_before
}

/// Apply all filters to a value; returns only values that pass all filters.
#[allow(dead_code)]
pub fn apply_chain<'a>(chain: &FilterChain, values: &[&'a str]) -> Vec<&'a str> {
    values.iter().copied().filter(|v| chain.filters.iter().all(|f| (f.test)(v))).collect()
}

/// Return the number of filters in the chain.
#[allow(dead_code)]
pub fn filter_count(chain: &FilterChain) -> usize {
    chain.filters.len()
}

/// Check whether a single value passes all filters.
#[allow(dead_code)]
pub fn chain_accepts(chain: &FilterChain, value: &str) -> bool {
    chain.filters.iter().all(|f| (f.test)(value))
}

/// Clear all filters from the chain.
#[allow(dead_code)]
pub fn clear_filters(chain: &mut FilterChain) {
    chain.filters.clear();
}

/// Return a string listing all filter names.
#[allow(dead_code)]
pub fn filter_chain_to_string(chain: &FilterChain) -> String {
    chain.filters.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_filter_chain() {
        let chain = new_filter_chain();
        assert_eq!(filter_count(&chain), 0);
    }

    #[test]
    fn test_add_filter() {
        let mut chain = new_filter_chain();
        add_filter(&mut chain, "non_empty", |v| !v.is_empty());
        assert_eq!(filter_count(&chain), 1);
    }

    #[test]
    fn test_apply_chain() {
        let mut chain = new_filter_chain();
        add_filter(&mut chain, "long", |v| v.len() > 3);
        let result = apply_chain(&chain, &["hi", "hello", "ab", "world"]);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_chain_accepts() {
        let mut chain = new_filter_chain();
        add_filter(&mut chain, "starts_a", |v| v.starts_with('a'));
        assert!(chain_accepts(&chain, "apple"));
        assert!(!chain_accepts(&chain, "banana"));
    }

    #[test]
    fn test_remove_filter() {
        let mut chain = new_filter_chain();
        add_filter(&mut chain, "f1", |_| true);
        let removed = remove_filter(&mut chain, "f1");
        assert!(removed);
        assert_eq!(filter_count(&chain), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut chain = new_filter_chain();
        let removed = remove_filter(&mut chain, "nope");
        assert!(!removed);
    }

    #[test]
    fn test_clear_filters() {
        let mut chain = new_filter_chain();
        add_filter(&mut chain, "f1", |_| true);
        add_filter(&mut chain, "f2", |_| false);
        clear_filters(&mut chain);
        assert_eq!(filter_count(&chain), 0);
    }

    #[test]
    fn test_filter_chain_to_string() {
        let mut chain = new_filter_chain();
        add_filter(&mut chain, "alpha", |_| true);
        add_filter(&mut chain, "beta", |_| true);
        let s = filter_chain_to_string(&chain);
        assert!(s.contains("alpha"));
        assert!(s.contains("beta"));
    }

    #[test]
    fn test_empty_chain_accepts_all() {
        let chain = new_filter_chain();
        assert!(chain_accepts(&chain, "anything"));
    }
}
