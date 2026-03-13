// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! KMP pattern search and Rabin-Karp rolling hash.

/// Build the KMP failure function for `pattern`.
pub fn kmp_failure_func(pattern: &[u8]) -> Vec<usize> {
    let m = pattern.len();
    let mut fail = vec![0usize; m];
    let mut k = 0usize;
    for i in 1..m {
        while k > 0 && pattern[k] != pattern[i] {
            k = fail[k - 1];
        }
        if pattern[k] == pattern[i] {
            k += 1;
        }
        fail[i] = k;
    }
    fail
}

/// Find all occurrences of `pattern` in `text` using KMP.
pub fn kmp_find_all(text: &[u8], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() {
        return Vec::new();
    }
    let fail = kmp_failure_func(pattern);
    let mut matches = Vec::new();
    let mut k = 0usize;
    for (i, &ch) in text.iter().enumerate() {
        while k > 0 && pattern[k] != ch {
            k = fail[k - 1];
        }
        if pattern[k] == ch {
            k += 1;
        }
        if k == pattern.len() {
            matches.push(i + 1 - pattern.len());
            k = fail[k - 1];
        }
    }
    matches
}

/// Check if `pattern` is a substring of `text` using KMP.
pub fn kmp_contains(text: &[u8], pattern: &[u8]) -> bool {
    !kmp_find_all(text, pattern).is_empty()
}

const RK_BASE: u64 = 257;
const RK_MOD: u64 = 1_000_000_007;

/// Rabin-Karp search: find all occurrences of `pattern` in `text`.
pub fn rabin_karp_find_all(text: &[u8], pattern: &[u8]) -> Vec<usize> {
    let m = pattern.len();
    let n = text.len();
    if m == 0 || m > n {
        return Vec::new();
    }

    /* compute pattern hash and highest power */
    let mut pat_hash = 0u64;
    let mut window_hash = 0u64;
    let mut high_power = 1u64;
    for i in 0..m {
        pat_hash = (pat_hash * RK_BASE + pattern[i] as u64) % RK_MOD;
        window_hash = (window_hash * RK_BASE + text[i] as u64) % RK_MOD;
        if i < m - 1 {
            high_power = (high_power * RK_BASE) % RK_MOD;
        }
    }

    let mut matches = Vec::new();
    let mut start = 0;
    loop {
        if window_hash == pat_hash && text[start..start + m] == *pattern {
            matches.push(start);
        }
        if start + m >= n {
            break;
        }
        window_hash = (window_hash + RK_MOD - text[start] as u64 * high_power % RK_MOD) % RK_MOD;
        window_hash = (window_hash * RK_BASE + text[start + m] as u64) % RK_MOD;
        start += 1;
    }
    matches
}

/// Count occurrences of `pattern` in `text` (KMP).
pub fn kmp_count(text: &[u8], pattern: &[u8]) -> usize {
    kmp_find_all(text, pattern).len()
}

/// String search wrapper using KMP.
pub struct StringSearcher {
    pattern: Vec<u8>,
    fail: Vec<usize>,
}

/// Construct a new StringSearcher for `pattern`.
pub fn new_string_searcher(pattern: &str) -> StringSearcher {
    let pat = pattern.as_bytes().to_vec();
    let fail = kmp_failure_func(&pat);
    StringSearcher { pattern: pat, fail }
}

impl StringSearcher {
    /// Find all occurrences in `text`.
    pub fn find_all(&self, text: &str) -> Vec<usize> {
        let t = text.as_bytes();
        let m = self.pattern.len();
        if m == 0 {
            return Vec::new();
        }
        let mut matches = Vec::new();
        let mut k = 0usize;
        for (i, &ch) in t.iter().enumerate() {
            while k > 0 && self.pattern[k] != ch {
                k = self.fail[k - 1];
            }
            if self.pattern[k] == ch {
                k += 1;
            }
            if k == m {
                matches.push(i + 1 - m);
                k = self.fail[k - 1];
            }
        }
        matches
    }

    /// Count occurrences.
    pub fn count(&self, text: &str) -> usize {
        self.find_all(text).len()
    }

    /// Check if pattern exists in text.
    pub fn contains(&self, text: &str) -> bool {
        !self.find_all(text).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmp_no_match() {
        /* KMP returns empty when pattern not found */
        let r = kmp_find_all(b"hello world", b"xyz");
        assert!(r.is_empty());
    }

    #[test]
    fn test_kmp_single_match() {
        /* KMP finds single occurrence */
        let r = kmp_find_all(b"abcabc", b"abc");
        assert_eq!(r[0], 0);
    }

    #[test]
    fn test_kmp_multiple_matches() {
        /* KMP finds overlapping matches */
        let r = kmp_find_all(b"aaa", b"aa");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_kmp_contains() {
        /* kmp_contains returns correct bool */
        assert!(kmp_contains(b"hello world", b"world"));
        assert!(!kmp_contains(b"hello", b"xyz"));
    }

    #[test]
    fn test_rabin_karp_finds_match() {
        /* Rabin-Karp finds pattern in text */
        let r = rabin_karp_find_all(b"abcabc", b"bc");
        assert!(r.contains(&1));
        assert!(r.contains(&4));
    }

    #[test]
    fn test_rabin_karp_no_match() {
        /* Rabin-Karp returns empty when no match */
        let r = rabin_karp_find_all(b"hello", b"xyz");
        assert!(r.is_empty());
    }

    #[test]
    fn test_string_searcher_find_all() {
        /* StringSearcher.find_all works correctly */
        let s = new_string_searcher("ab");
        let r = s.find_all("ababab");
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_kmp_count() {
        /* kmp_count returns correct number of occurrences */
        assert_eq!(kmp_count(b"banana", b"an"), 2);
    }

    #[test]
    fn test_searcher_contains() {
        /* StringSearcher.contains returns true when pattern exists */
        let s = new_string_searcher("world");
        assert!(s.contains("hello world"));
        assert!(!s.contains("hello earth"));
    }
}
