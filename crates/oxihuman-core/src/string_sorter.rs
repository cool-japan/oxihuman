#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! String sorting utilities.

/// Sort order for string comparisons.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// A configurable string sorter.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StringSorter {
    order: SortOrder,
    case_sensitive: bool,
}

/// Create a new `StringSorter` with ascending order and case-sensitive comparisons.
#[allow(dead_code)]
pub fn new_string_sorter() -> StringSorter {
    StringSorter { order: SortOrder::Ascending, case_sensitive: true }
}

/// Sort a slice of strings in ascending order, returning a new `Vec`.
#[allow(dead_code)]
pub fn sort_strings(strings: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = strings.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

/// Sort strings in descending order.
#[allow(dead_code)]
pub fn sort_strings_desc(strings: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = strings.iter().map(|s| s.to_string()).collect();
    v.sort();
    v.reverse();
    v
}

/// Sort strings by length (shortest first), then lexicographically.
#[allow(dead_code)]
pub fn sort_by_length(strings: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = strings.iter().map(|s| s.to_string()).collect();
    v.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
    v
}

/// Stub for locale-aware sorting (returns ASCII sort for now).
#[allow(dead_code)]
pub fn sort_locale_stub(strings: &[&str], _locale: &str) -> Vec<String> {
    sort_strings(strings)
}

/// Flip the sort order.
#[allow(dead_code)]
pub fn sort_order_flip(order: &SortOrder) -> SortOrder {
    match order {
        SortOrder::Ascending => SortOrder::Descending,
        SortOrder::Descending => SortOrder::Ascending,
    }
}

/// Return true if the slice is already sorted in ascending order.
#[allow(dead_code)]
pub fn is_sorted(strings: &[&str]) -> bool {
    strings.windows(2).all(|w| w[0] <= w[1])
}

/// Sort and deduplicate strings.
#[allow(dead_code)]
pub fn sort_unique(strings: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = strings.iter().map(|s| s.to_string()).collect();
    v.sort();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_string_sorter() {
        let sorter = new_string_sorter();
        assert_eq!(sorter.order, SortOrder::Ascending);
    }

    #[test]
    fn test_sort_strings() {
        let result = sort_strings(&["banana", "apple", "cherry"]);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_sort_strings_desc() {
        let result = sort_strings_desc(&["banana", "apple", "cherry"]);
        assert_eq!(result, vec!["cherry", "banana", "apple"]);
    }

    #[test]
    fn test_sort_by_length() {
        let result = sort_by_length(&["banana", "fig", "apple"]);
        assert_eq!(result[0], "fig");
    }

    #[test]
    fn test_sort_locale_stub() {
        let result = sort_locale_stub(&["b", "a"], "en-US");
        assert_eq!(result[0], "a");
    }

    #[test]
    fn test_sort_order_flip() {
        assert_eq!(sort_order_flip(&SortOrder::Ascending), SortOrder::Descending);
        assert_eq!(sort_order_flip(&SortOrder::Descending), SortOrder::Ascending);
    }

    #[test]
    fn test_is_sorted() {
        assert!(is_sorted(&["a", "b", "c"]));
        assert!(!is_sorted(&["c", "a", "b"]));
    }

    #[test]
    fn test_sort_unique() {
        let result = sort_unique(&["b", "a", "b", "c", "a"]);
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_empty() {
        let result = sort_strings(&[]);
        assert!(result.is_empty());
    }
}
