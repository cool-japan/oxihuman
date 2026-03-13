// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Suffix array construction and LCP array.

/// Build a suffix array for `s` using a simple O(n log^2 n) algorithm.
/// Returns a vector of starting indices into `s`, sorted lexicographically.
#[allow(dead_code)]
pub fn build_suffix_array(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    if n == 0 {
        return vec![];
    }
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| s[a..].cmp(&s[b..]));
    sa
}

/// Build the LCP (Longest Common Prefix) array from the suffix array.
/// `lcp[i]` = length of the longest common prefix of `sa[i]` and `sa[i-1]`.
/// `lcp[0]` = 0.
#[allow(dead_code)]
pub fn build_lcp_array(s: &[u8], sa: &[usize]) -> Vec<usize> {
    let n = sa.len();
    if n == 0 {
        return vec![];
    }
    let mut lcp = vec![0usize; n];
    // Build rank array
    let mut rank = vec![0usize; n];
    for (i, &pos) in sa.iter().enumerate() {
        rank[pos] = i;
    }
    let mut h = 0usize;
    for i in 0..n {
        if rank[i] > 0 {
            let j = sa[rank[i] - 1];
            while i + h < n && j + h < n && s[i + h] == s[j + h] {
                h += 1;
            }
            lcp[rank[i]] = h;
            h = h.saturating_sub(1);
        } else {
            h = 0;
        }
    }
    lcp
}

/// Check if `pattern` occurs in `text` using binary search on the suffix array.
#[allow(dead_code)]
pub fn sa_contains(text: &[u8], sa: &[usize], pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return true;
    }
    let mut lo = 0usize;
    let mut hi = sa.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let suf = &text[sa[mid]..];
        let cmp = suf[..pattern.len().min(suf.len())].cmp(pattern);
        use std::cmp::Ordering;
        match cmp {
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid,
            Ordering::Equal => {
                if suf.len() >= pattern.len() {
                    return true;
                }
                lo = mid + 1;
            }
        }
    }
    false
}

/// Find all occurrences of `pattern` in `text` using the suffix array.
#[allow(dead_code)]
pub fn sa_find_all(text: &[u8], sa: &[usize], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() || sa.is_empty() {
        return vec![];
    }
    let plen = pattern.len();
    // Find left bound
    let left = {
        let (mut lo, mut hi) = (0usize, sa.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if &text[sa[mid]..][..plen.min(text.len() - sa[mid])] < pattern {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    };
    // Find right bound
    let right = {
        let (mut lo, mut hi) = (left, sa.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let suf = &text[sa[mid]..];
            if suf.len() >= plen && &suf[..plen] == pattern {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    };
    sa[left..right].to_vec()
}

/// Count the number of distinct suffixes (equals len(sa)).
#[allow(dead_code)]
pub fn sa_suffix_count(sa: &[usize]) -> usize {
    sa.len()
}

/// Maximum LCP value.
#[allow(dead_code)]
pub fn lcp_max(lcp: &[usize]) -> usize {
    lcp.iter().copied().max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banana() -> (&'static [u8], Vec<usize>) {
        let s = b"banana";
        let sa = build_suffix_array(s);
        (s, sa)
    }

    #[test]
    fn suffix_array_length_matches_input() {
        let (s, sa) = banana();
        assert_eq!(sa.len(), s.len());
    }

    #[test]
    fn suffix_array_sorted() {
        let (s, sa) = banana();
        for i in 1..sa.len() {
            assert!(s[sa[i - 1]..] <= s[sa[i]..]);
        }
    }

    #[test]
    fn lcp_array_length_matches() {
        let (s, sa) = banana();
        let lcp = build_lcp_array(s, &sa);
        assert_eq!(lcp.len(), sa.len());
    }

    #[test]
    fn lcp_first_is_zero() {
        let (s, sa) = banana();
        let lcp = build_lcp_array(s, &sa);
        assert_eq!(lcp[0], 0);
    }

    #[test]
    fn sa_contains_existing_pattern() {
        let (s, sa) = banana();
        assert!(sa_contains(s, &sa, b"ana"));
    }

    #[test]
    fn sa_contains_missing_pattern() {
        let (s, sa) = banana();
        assert!(!sa_contains(s, &sa, b"xyz"));
    }

    #[test]
    fn sa_find_all_finds_occurrences() {
        let (s, sa) = banana();
        let positions = sa_find_all(s, &sa, b"an");
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn empty_input() {
        let sa = build_suffix_array(b"");
        assert!(sa.is_empty());
    }

    #[test]
    fn lcp_max_correct() {
        let (s, sa) = banana();
        let lcp = build_lcp_array(s, &sa);
        let m = lcp_max(&lcp);
        assert!(m >= 3);
    }

    #[test]
    fn sa_suffix_count_is_len() {
        let (_, sa) = banana();
        assert_eq!(sa_suffix_count(&sa), 6);
    }
}
