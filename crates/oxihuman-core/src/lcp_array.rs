// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! LCP (Longest Common Prefix) array construction from a suffix array.

/// Build the LCP array using Kasai's O(n) algorithm.
/// `sa` is the suffix array (0-indexed positions into `s`).
/// Returns `lcp` where `lcp[i]` = LCP of suffixes at `sa[i-1]` and `sa[i]`.
/// `lcp[0]` is always 0.
pub fn build_lcp(s: &[u8], sa: &[usize]) -> Vec<usize> {
    let n = s.len();
    if n == 0 {
        return Vec::new();
    }
    /* rank[i] = position of suffix i in the suffix array */
    let mut rank = vec![0usize; n];
    for (i, &suf) in sa.iter().enumerate() {
        rank[suf] = i;
    }

    let mut lcp = vec![0usize; n];
    let mut h = 0usize;
    for i in 0..n {
        if rank[i] > 0 {
            let j = sa[rank[i] - 1];
            while i + h < n && j + h < n && s[i + h] == s[j + h] {
                h += 1;
            }
            lcp[rank[i]] = h;
            h = h.saturating_sub(1);
        }
    }
    lcp
}

/// Return the maximum LCP value.
pub fn lcp_max_val(lcp: &[usize]) -> usize {
    lcp.iter().copied().max().unwrap_or(0)
}

/// Return the average LCP value (f32).
pub fn lcp_avg(lcp: &[usize]) -> f32 {
    if lcp.is_empty() {
        return 0.0;
    }
    lcp.iter().sum::<usize>() as f32 / lcp.len() as f32
}

/// Return the number of distinct substrings in the string.
/// = n*(n+1)/2 - sum(lcp)
pub fn distinct_substrings(n: usize, lcp: &[usize]) -> usize {
    let total = n * (n + 1) / 2;
    let sum_lcp: usize = lcp.iter().sum();
    total.saturating_sub(sum_lcp)
}

/// Return LCP of the suffixes at positions `i` and `j` in `sa`.
pub fn lcp_query(lcp: &[usize], i: usize, j: usize) -> usize {
    if i == j {
        return usize::MAX; /* infinite */
    }
    let lo = i.min(j) + 1;
    let hi = i.max(j);
    if lo > hi || hi >= lcp.len() {
        return 0;
    }
    lcp[lo..=hi].iter().copied().min().unwrap_or(0)
}

/// Return `true` if the LCP array has length equal to the suffix array.
pub fn lcp_valid(lcp: &[usize], sa: &[usize]) -> bool {
    lcp.len() == sa.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /* helper: naive suffix array */
    fn naive_sa(s: &[u8]) -> Vec<usize> {
        let n = s.len();
        let mut sa: Vec<usize> = (0..n).collect();
        sa.sort_by_key(|&i| &s[i..]);
        sa
    }

    #[test]
    fn test_basic_lcp() {
        let s = b"banana";
        let sa = naive_sa(s);
        let lcp = build_lcp(s, &sa);
        /* lcp[0] must be 0 */
        assert_eq!(lcp[0], 0);
        assert_eq!(lcp.len(), s.len());
    }

    #[test]
    fn test_lcp_max() {
        let s = b"aaaa";
        let sa = naive_sa(s);
        let lcp = build_lcp(s, &sa);
        assert_eq!(lcp_max_val(&lcp), 3);
    }

    #[test]
    fn test_lcp_all_distinct() {
        /* each character is unique */
        let s = b"abcd";
        let sa = naive_sa(s);
        let lcp = build_lcp(s, &sa);
        assert_eq!(lcp_max_val(&lcp), 0);
    }

    #[test]
    fn test_distinct_substrings() {
        let s = b"aa";
        let sa = naive_sa(s);
        let lcp = build_lcp(s, &sa);
        /* "aa": sa=[1,0] lcp=[0,1]; total = 3, sum_lcp = 1, distinct = 2 */
        let d = distinct_substrings(s.len(), &lcp);
        assert_eq!(d, 2); /* "a", "aa" */
    }

    #[test]
    fn test_lcp_valid() {
        let s = b"hello";
        let sa = naive_sa(s);
        let lcp = build_lcp(s, &sa);
        assert!(lcp_valid(&lcp, &sa));
    }

    #[test]
    fn test_lcp_avg_nonnegative() {
        let s = b"abcabc";
        let sa = naive_sa(s);
        let lcp = build_lcp(s, &sa);
        assert!(lcp_avg(&lcp) >= 0.0);
    }

    #[test]
    fn test_lcp_query_same_index() {
        let s = b"abc";
        let sa = naive_sa(s);
        let lcp = build_lcp(s, &sa);
        assert_eq!(lcp_query(&lcp, 1, 1), usize::MAX);
    }

    #[test]
    fn test_empty_string() {
        let lcp = build_lcp(b"", &[]);
        assert!(lcp.is_empty());
    }

    #[test]
    fn test_single_char() {
        let s = b"a";
        let sa = naive_sa(s);
        let lcp = build_lcp(s, &sa);
        assert_eq!(lcp.len(), 1);
        assert_eq!(lcp[0], 0);
    }
}
