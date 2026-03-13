// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Knuth-Morris-Pratt (KMP) pattern search.

/// Build the KMP failure (partial match) table for `pattern`.
pub fn kmp_failure(pattern: &[u8]) -> Vec<usize> {
    let m = pattern.len();
    let mut f = vec![0usize; m];
    let mut k = 0usize;
    for i in 1..m {
        while k > 0 && pattern[k] != pattern[i] {
            k = f[k - 1];
        }
        if pattern[k] == pattern[i] {
            k += 1;
        }
        f[i] = k;
    }
    f
}

/// Return all start positions in `text` where `pattern` occurs.
pub fn kmp_search(text: &[u8], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() {
        return (0..text.len()).collect();
    }
    let f = kmp_failure(pattern);
    let m = pattern.len();
    let mut q = 0usize;
    let mut positions = Vec::new();

    for (i, &c) in text.iter().enumerate() {
        while q > 0 && pattern[q] != c {
            q = f[q - 1];
        }
        if pattern[q] == c {
            q += 1;
        }
        if q == m {
            positions.push(i + 1 - m);
            q = f[q - 1];
        }
    }
    positions
}

/// Return `true` if `pattern` occurs at least once in `text`.
pub fn kmp_contains(text: &[u8], pattern: &[u8]) -> bool {
    !kmp_search(text, pattern).is_empty()
}

/// Return the count of (possibly overlapping) occurrences.
pub fn kmp_count(text: &[u8], pattern: &[u8]) -> usize {
    kmp_search(text, pattern).len()
}

/// Return the failure table length (equals pattern length).
pub fn kmp_failure_len(pattern: &[u8]) -> usize {
    kmp_failure(pattern).len()
}

/// Return the maximum value in the failure table.
pub fn kmp_max_failure(pattern: &[u8]) -> usize {
    kmp_failure(pattern).iter().copied().max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_match() {
        let pos = kmp_search(b"abcabc", b"abc");
        assert_eq!(pos, vec![0, 3]);
    }

    #[test]
    fn test_no_match() {
        let pos = kmp_search(b"hello", b"xyz");
        assert!(pos.is_empty());
    }

    #[test]
    fn test_single_char_pattern() {
        let pos = kmp_search(b"aababc", b"a");
        assert_eq!(pos, vec![0, 1, 3]);
    }

    #[test]
    fn test_overlapping() {
        /* "aa" in "aaa" starts at 0 and 1 */
        let pos = kmp_search(b"aaa", b"aa");
        assert_eq!(pos, vec![0, 1]);
    }

    #[test]
    fn test_failure_table_length() {
        assert_eq!(kmp_failure_len(b"abcabc"), 6);
    }

    #[test]
    fn test_failure_table_values() {
        /* "abcabc" -> [0,0,0,1,2,3] */
        let f = kmp_failure(b"abcabc");
        assert_eq!(f, vec![0, 0, 0, 1, 2, 3]);
    }

    #[test]
    fn test_contains_true() {
        assert!(kmp_contains(b"the quick brown fox", b"quick"));
    }

    #[test]
    fn test_contains_false() {
        assert!(!kmp_contains(b"the quick brown fox", b"slow"));
    }

    #[test]
    fn test_count_matches() {
        assert_eq!(kmp_count(b"aaaa", b"a"), 4);
    }

    #[test]
    fn test_max_failure() {
        /* "aabaabaab": max failure = 6 */
        let mf = kmp_max_failure(b"aabaabaab");
        assert_eq!(mf, 6);
    }
}
