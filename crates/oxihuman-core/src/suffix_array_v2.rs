// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Suffix array v2 — prefix-doubling O(n log n) construction.

/// Build a suffix array using prefix-doubling (Manber & Myers approach).
/// Returns sorted suffix starting indices.
pub fn build_sa_v2(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0];
    }

    /* initial ranking by first character */
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<i64> = s.iter().map(|&b| b as i64).collect();
    let mut tmp = vec![0i64; n];

    let mut gap = 1usize;
    while gap < n {
        let g = gap;
        let r = rank.clone();
        sa.sort_by(|&a, &b| {
            let ra = r[a];
            let rb = r[b];
            if ra != rb {
                return ra.cmp(&rb);
            }
            let ra2 = if a + g < n { r[a + g] } else { -1 };
            let rb2 = if b + g < n { r[b + g] } else { -1 };
            ra2.cmp(&rb2)
        });
        /* recompute ranks */
        tmp[sa[0]] = 0;
        for i in 1..n {
            let prev = sa[i - 1];
            let cur = sa[i];
            let same = rank[prev] == rank[cur]
                && (if prev + g < n { rank[prev + g] } else { -1 })
                    == (if cur + g < n { rank[cur + g] } else { -1 });
            tmp[cur] = tmp[prev] + if same { 0 } else { 1 };
        }
        rank = tmp.clone();
        if rank[sa[n - 1]] as usize == n - 1 {
            break; /* all ranks unique */
        }
        gap *= 2;
    }
    sa
}

/// Return the number of suffixes (= length of string).
pub fn sa2_len(sa: &[usize]) -> usize {
    sa.len()
}

/// Binary-search for all positions where `pattern` occurs in `s`.
pub fn sa2_search(s: &[u8], sa: &[usize], pattern: &[u8]) -> Vec<usize> {
    let n = s.len();
    let m = pattern.len();
    if m > n {
        return Vec::new();
    }

    /* lower bound */
    let lo = {
        let mut lo = 0usize;
        let mut hi = n;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if &s[sa[mid]..(sa[mid] + m).min(n)] < pattern {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    };

    /* upper bound */
    let hi = {
        let mut lo2 = lo;
        let mut hi2 = n;
        while lo2 < hi2 {
            let mid = (lo2 + hi2) / 2;
            let end = (sa[mid] + m).min(n);
            if &s[sa[mid]..end] <= pattern {
                lo2 = mid + 1;
            } else {
                hi2 = mid;
            }
        }
        lo2
    };

    sa[lo..hi].to_vec()
}

/// Return `true` if `pattern` occurs in `s`.
pub fn sa2_contains(s: &[u8], sa: &[usize], pattern: &[u8]) -> bool {
    !sa2_search(s, sa, pattern).is_empty()
}

/// Verify the SA is correctly sorted.
pub fn sa2_is_sorted(s: &[u8], sa: &[usize]) -> bool {
    sa.windows(2).all(|w| s[w[0]..] <= s[w[1]..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sa_v2_banana() {
        let s = b"banana";
        let sa = build_sa_v2(s);
        assert_eq!(sa.len(), 6);
        assert!(sa2_is_sorted(s, &sa));
    }

    #[test]
    fn test_build_sa_v2_sorted() {
        let s = b"mississippi";
        let sa = build_sa_v2(s);
        assert!(sa2_is_sorted(s, &sa));
    }

    #[test]
    fn test_sa2_search_found() {
        let s = b"banana";
        let sa = build_sa_v2(s);
        let pos = sa2_search(s, &sa, b"an");
        assert!(!pos.is_empty());
    }

    #[test]
    fn test_sa2_search_not_found() {
        let s = b"banana";
        let sa = build_sa_v2(s);
        let pos = sa2_search(s, &sa, b"xyz");
        assert!(pos.is_empty());
    }

    #[test]
    fn test_sa2_contains_true() {
        let s = b"hello world";
        let sa = build_sa_v2(s);
        assert!(sa2_contains(s, &sa, b"world"));
    }

    #[test]
    fn test_sa2_contains_false() {
        let s = b"hello world";
        let sa = build_sa_v2(s);
        assert!(!sa2_contains(s, &sa, b"xyz"));
    }

    #[test]
    fn test_sa2_len() {
        let s = b"abcde";
        let sa = build_sa_v2(s);
        assert_eq!(sa2_len(&sa), 5);
    }

    #[test]
    fn test_empty_string() {
        let sa = build_sa_v2(b"");
        assert!(sa.is_empty());
    }

    #[test]
    fn test_single_char() {
        let sa = build_sa_v2(b"a");
        assert_eq!(sa, vec![0]);
    }

    #[test]
    fn test_repeated_chars() {
        let s = b"aaaa";
        let sa = build_sa_v2(s);
        assert!(sa2_is_sorted(s, &sa));
    }
}
