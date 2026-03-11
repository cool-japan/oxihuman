// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Z-function string matching.

/// Compute the Z-array for string `s`.
/// `z[i]` = length of the longest substring starting at `s[i]` that is also
/// a prefix of `s`. `z[0]` is conventionally 0.
pub fn z_function(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    let mut z = vec![0usize; n];
    let (mut l, mut r) = (0usize, 0usize);
    for i in 1..n {
        if i < r {
            z[i] = (r - i).min(z[i - l]);
        }
        while i + z[i] < n && s[z[i]] == s[i + z[i]] {
            z[i] += 1;
        }
        if i + z[i] > r {
            l = i;
            r = i + z[i];
        }
    }
    z
}

/// Return all positions in `text` where `pattern` starts.
pub fn z_search(text: &[u8], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() {
        return (0..text.len()).collect();
    }
    /* s = pattern + '$' + text */
    let mut s = Vec::with_capacity(pattern.len() + 1 + text.len());
    s.extend_from_slice(pattern);
    s.push(b'$');
    s.extend_from_slice(text);

    let z = z_function(&s);
    let plen = pattern.len();
    let offset = plen + 1;

    z.iter()
        .enumerate()
        .skip(offset)
        .filter(|(_, &v)| v == plen)
        .map(|(i, _)| i - offset)
        .collect()
}

/// Return the Z-array max value (longest prefix match at any position).
pub fn z_max(z: &[usize]) -> usize {
    z.iter().copied().max().unwrap_or(0)
}

/// Return `true` if `pattern` occurs in `text`.
pub fn z_contains(text: &[u8], pattern: &[u8]) -> bool {
    !z_search(text, pattern).is_empty()
}

/// Return the count of occurrences of `pattern` in `text`.
pub fn z_count(text: &[u8], pattern: &[u8]) -> usize {
    z_search(text, pattern).len()
}

/// Return `true` if the Z-array for `s` is consistent (all `z[i]` <= n-i).
pub fn z_valid(z: &[usize], n: usize) -> bool {
    z.iter().enumerate().all(|(i, &v)| v <= n.saturating_sub(i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_function_basic() {
        let z = z_function(b"aabxaa");
        /* z[4] should be 2 ("aa" matches prefix) */
        assert_eq!(z[4], 2);
    }

    #[test]
    fn test_z_search_single_match() {
        let pos = z_search(b"hello world", b"world");
        assert_eq!(pos, vec![6]);
    }

    #[test]
    fn test_z_search_multiple_matches() {
        let pos = z_search(b"ababab", b"ab");
        assert_eq!(pos, vec![0, 2, 4]);
    }

    #[test]
    fn test_z_search_no_match() {
        let pos = z_search(b"hello", b"xyz");
        assert!(pos.is_empty());
    }

    #[test]
    fn test_z_contains_true() {
        assert!(z_contains(b"foobar", b"bar"));
    }

    #[test]
    fn test_z_contains_false() {
        assert!(!z_contains(b"foobar", b"baz"));
    }

    #[test]
    fn test_z_count() {
        assert_eq!(z_count(b"aaaa", b"aa"), 3);
    }

    #[test]
    fn test_z_all_same_chars() {
        let z = z_function(b"aaaa");
        /* z[1]=3, z[2]=2, z[3]=1 */
        assert_eq!(z[1], 3);
        assert_eq!(z[2], 2);
        assert_eq!(z[3], 1);
    }

    #[test]
    fn test_z_valid() {
        let s = b"abcabc";
        let z = z_function(s);
        assert!(z_valid(&z, s.len()));
    }
}
