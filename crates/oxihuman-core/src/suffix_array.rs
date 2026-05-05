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

// ── SA-IS based suffix array and extended stub API ────────────────────────

/// Sentinel value used internally; must be smaller than every real symbol.
const SENTINEL: usize = 0;

/// Classify each position as S-type (true) or L-type (false).
/// By convention the last character (sentinel) is S-type.
fn classify_sl(text: &[usize]) -> Vec<bool> {
    let n = text.len();
    let mut types = vec![false; n];
    if n == 0 {
        return types;
    }
    // last position is always S-type
    types[n - 1] = true;
    for i in (0..n.saturating_sub(1)).rev() {
        types[i] = if text[i] < text[i + 1] {
            true
        } else if text[i] > text[i + 1] {
            false
        } else {
            types[i + 1]
        };
    }
    types
}

/// Check whether position `i` is a Left-Most S-type (LMS) character.
#[inline]
fn is_lms(types: &[bool], i: usize) -> bool {
    i > 0 && types[i] && !types[i - 1]
}

/// Compute bucket sizes for each symbol value in `text` with alphabet `[0, alpha)`.
fn get_buckets(text: &[usize], alpha: usize, end: bool) -> Vec<usize> {
    let mut buckets = vec![0usize; alpha];
    for &ch in text {
        if ch < alpha {
            buckets[ch] += 1;
        }
    }
    let mut sum = 0usize;
    for b in buckets.iter_mut() {
        sum += *b;
        *b = if end { sum } else { sum - *b };
    }
    buckets
}

/// Induce L-type suffixes from seeded positions.
fn induce_l(sa: &mut [usize], text: &[usize], types: &[bool], alpha: usize) {
    let n = text.len();
    let mut buckets = get_buckets(text, alpha, false);
    for i in 0..n {
        if sa[i] == usize::MAX || sa[i] == 0 {
            continue;
        }
        let j = sa[i] - 1;
        if !types[j] {
            let ch = text[j];
            sa[buckets[ch]] = j;
            buckets[ch] += 1;
        }
    }
}

/// Induce S-type suffixes from seeded positions.
fn induce_s(sa: &mut [usize], text: &[usize], types: &[bool], alpha: usize) {
    let n = text.len();
    let mut buckets = get_buckets(text, alpha, true);
    for i in (0..n).rev() {
        if sa[i] == usize::MAX || sa[i] == 0 {
            continue;
        }
        let j = sa[i] - 1;
        if types[j] {
            buckets[text[j]] -= 1;
            sa[buckets[text[j]]] = j;
        }
    }
}

/// Compare two LMS substrings for equality.
fn lms_equal(text: &[usize], types: &[bool], a: usize, b: usize) -> bool {
    let n = text.len();
    let mut i = 0;
    loop {
        let pa = a + i;
        let pb = b + i;
        if pa >= n || pb >= n {
            return pa >= n && pb >= n;
        }
        if text[pa] != text[pb] || types[pa] != types[pb] {
            return false;
        }
        if i > 0 && (is_lms(types, pa) || is_lms(types, pb)) {
            return is_lms(types, pa) && is_lms(types, pb);
        }
        i += 1;
    }
}

/// SA-IS on an integer alphabet `[0, alpha)`. `text` must end with a sentinel
/// whose value is `0` and that does not appear elsewhere.
fn sais_int(text: &[usize], alpha: usize) -> Vec<usize> {
    let n = text.len();
    if n <= 1 {
        return if n == 0 { vec![] } else { vec![0] };
    }
    if n <= 3 {
        let mut sa: Vec<usize> = (0..n).collect();
        sa.sort_by(|&a, &b| text[a..].cmp(&text[b..]));
        return sa;
    }

    let types = classify_sl(text);

    let mut lms_positions: Vec<usize> = Vec::new();
    for i in 1..n {
        if is_lms(&types, i) {
            lms_positions.push(i);
        }
    }

    let mut sa = vec![usize::MAX; n];
    {
        let mut buckets = get_buckets(text, alpha, true);
        for &pos in lms_positions.iter().rev() {
            let ch = text[pos];
            buckets[ch] -= 1;
            sa[buckets[ch]] = pos;
        }
    }

    induce_l(&mut sa, text, &types, alpha);
    induce_s(&mut sa, text, &types, alpha);

    let lms_count = lms_positions.len();
    let mut sorted_lms: Vec<usize> = Vec::with_capacity(lms_count);
    for &v in &sa {
        if v != usize::MAX && is_lms(&types, v) {
            sorted_lms.push(v);
        }
    }

    let mut name_map = vec![usize::MAX; n];
    let mut current_name = 0usize;
    name_map[sorted_lms[0]] = current_name;
    for i in 1..sorted_lms.len() {
        if !lms_equal(text, &types, sorted_lms[i - 1], sorted_lms[i]) {
            current_name += 1;
        }
        name_map[sorted_lms[i]] = current_name;
    }
    let num_names = current_name + 1;

    let reduced: Vec<usize> = lms_positions.iter().map(|&p| name_map[p]).collect();

    let sa_reduced = if num_names < lms_count {
        let mut reduced_with_sentinel = reduced;
        reduced_with_sentinel.push(SENTINEL);
        sais_int(&reduced_with_sentinel, num_names + 1)
    } else {
        let mut inv = vec![0usize; lms_count + 1];
        for (i, &r) in reduced.iter().enumerate() {
            inv[r] = i;
        }
        let mut result = vec![lms_count];
        result.extend_from_slice(&inv[..lms_count]);
        result
    };

    sa.fill(usize::MAX);
    {
        let mut buckets = get_buckets(text, alpha, true);
        for i in (1..sa_reduced.len()).rev() {
            let lms_idx = lms_positions[sa_reduced[i]];
            let ch = text[lms_idx];
            buckets[ch] -= 1;
            sa[buckets[ch]] = lms_idx;
        }
    }

    induce_l(&mut sa, text, &types, alpha);
    induce_s(&mut sa, text, &types, alpha);

    sa
}

/// Build a suffix array for the given string using the SA-IS algorithm (O(n)).
///
/// Returns a `Vec<usize>` of length `s.len()` containing the starting positions
/// of suffixes in lexicographic order.
pub fn build_suffix_array_stub(s: &str) -> Vec<usize> {
    let n = s.len();
    if n == 0 {
        return Vec::new();
    }

    let bytes = s.as_bytes();
    let mut text: Vec<usize> = Vec::with_capacity(n + 1);
    for &b in bytes {
        text.push((b as usize) + 1);
    }
    text.push(SENTINEL);

    let sa_full = sais_int(&text, 258);

    let mut result = Vec::with_capacity(n);
    for &pos in &sa_full {
        if pos < n {
            result.push(pos);
        }
    }
    result
}

/// Binary search on the suffix array to find one occurrence of `pattern` in `s`.
///
/// Returns `Some(position)` where `s[position..]` starts with `pattern`,
/// or `None` if the pattern is not found.
pub fn sa_stub_search(s: &str, sa: &[usize], pattern: &str) -> Option<usize> {
    if pattern.is_empty() || sa.is_empty() {
        return None;
    }
    let pb = pattern.as_bytes();
    let sb = s.as_bytes();
    let n = sa.len();

    let mut lo = 0usize;
    let mut hi = n;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let start = sa[mid];
        let suffix = &sb[start..];
        let cmp_len = pb.len().min(suffix.len());
        match suffix[..cmp_len].cmp(pb) {
            std::cmp::Ordering::Less => lo = mid + 1,
            _ => hi = mid,
        }
    }

    if lo < n {
        let start = sa[lo];
        let suffix = &sb[start..];
        if suffix.len() >= pb.len() && suffix[..pb.len()] == *pb {
            return Some(sa[lo]);
        }
    }
    None
}

/// Count the number of occurrences of `pattern` in `s` using the suffix array.
pub fn sa_stub_count_occurrences(s: &str, sa: &[usize], pattern: &str) -> usize {
    if pattern.is_empty() || sa.is_empty() {
        return 0;
    }
    let (lo, hi) = sa_range(s, sa, pattern);
    hi.saturating_sub(lo)
}

/// Find all positions where `pattern` occurs in `s`.
///
/// Returns a sorted `Vec<usize>` of starting positions.
pub fn sa_stub_find_all(s: &str, sa: &[usize], pattern: &str) -> Vec<usize> {
    if pattern.is_empty() || sa.is_empty() {
        return Vec::new();
    }
    let (lo, hi) = sa_range(s, sa, pattern);
    let mut positions: Vec<usize> = sa[lo..hi].to_vec();
    positions.sort_unstable();
    positions
}

/// Construct the LCP (Longest Common Prefix) array using Kasai's algorithm in O(n).
///
/// `lcp[i]` is the length of the longest common prefix between `sa[i-1]` and `sa[i]`.
/// By convention `lcp[0] = 0`.
pub fn lcp_array_stub(s: &str, sa: &[usize]) -> Vec<usize> {
    let n = sa.len();
    if n == 0 {
        return Vec::new();
    }
    let sb = s.as_bytes();

    let mut rank = vec![0usize; n];
    for (i, &pos) in sa.iter().enumerate() {
        if pos < n {
            rank[pos] = i;
        }
    }

    let mut lcp = vec![0usize; n];
    let mut h = 0usize;

    for i in 0..n {
        if rank[i] == 0 {
            h = 0;
            continue;
        }
        let j = sa[rank[i] - 1];
        while i + h < n && j + h < n && sb[i + h] == sb[j + h] {
            h += 1;
        }
        lcp[rank[i]] = h;
        h = h.saturating_sub(1);
    }
    lcp
}

/// Find the longest repeated substring in `s` using the LCP array.
///
/// Returns a tuple `(start, length)` of the first occurrence of the longest
/// substring that appears at least twice.
pub fn sa_stub_longest_repeated_substring(_s: &str, sa: &[usize], lcp: &[usize]) -> (usize, usize) {
    if sa.is_empty() || lcp.is_empty() {
        return (0, 0);
    }
    let mut best_len = 0usize;
    let mut best_pos = 0usize;
    for i in 1..lcp.len() {
        if lcp[i] > best_len {
            best_len = lcp[i];
            best_pos = sa[i];
        }
    }
    (best_pos, best_len)
}

/// Return the range `[lo, hi)` within the suffix array where all suffixes
/// share `pattern` as a prefix.
fn sa_range(s: &str, sa: &[usize], pattern: &str) -> (usize, usize) {
    let pb = pattern.as_bytes();
    let sb = s.as_bytes();
    let n = sa.len();

    let lo = {
        let mut l = 0usize;
        let mut r = n;
        while l < r {
            let mid = l + (r - l) / 2;
            let start = sa[mid];
            let suffix = &sb[start..];
            let cmp_len = pb.len().min(suffix.len());
            if suffix[..cmp_len] < *pb {
                l = mid + 1;
            } else {
                r = mid;
            }
        }
        l
    };

    let hi = {
        let mut l = lo;
        let mut r = n;
        while l < r {
            let mid = l + (r - l) / 2;
            let start = sa[mid];
            let suffix = &sb[start..];
            let cmp_len = pb.len().min(suffix.len());
            if suffix[..cmp_len] <= *pb {
                l = mid + 1;
            } else {
                r = mid;
            }
        }
        l
    };

    (lo, hi)
}

#[cfg(test)]
mod tests_stub {
    use super::*;

    #[test]
    fn test_build_suffix_array() {
        /* suffix array for "banana" is sorted */
        let sa = build_suffix_array_stub("banana");
        assert_eq!(sa.len(), 6);
        /* verify it's sorted by checking adjacent suffixes */
        let s = "banana";
        for i in 0..sa.len() - 1 {
            assert!(s[sa[i]..] <= s[sa[i + 1]..]);
        }
    }

    #[test]
    fn test_sa_search_found() {
        /* search finds existing pattern */
        let s = "banana";
        let sa = build_suffix_array_stub(s);
        let pos = sa_stub_search(s, &sa, "ana");
        assert!(pos.is_some());
        assert!(s[pos.expect("should succeed")..].starts_with("ana"));
    }

    #[test]
    fn test_sa_search_not_found() {
        /* search returns None for missing pattern */
        let s = "banana";
        let sa = build_suffix_array_stub(s);
        assert!(sa_stub_search(s, &sa, "xyz").is_none());
    }

    #[test]
    fn test_count_occurrences() {
        /* counts all occurrences of pattern */
        let s = "banana";
        let sa = build_suffix_array_stub(s);
        let n = sa_stub_count_occurrences(s, &sa, "an");
        assert_eq!(n, 2);
    }

    #[test]
    fn test_lcp_array() {
        /* LCP array has correct length */
        let s = "abcabc";
        let sa = build_suffix_array_stub(s);
        let lcp = lcp_array_stub(s, &sa);
        assert_eq!(lcp.len(), sa.len());
    }

    #[test]
    fn test_lcp_first_zero() {
        /* first LCP entry is 0 by convention */
        let s = "hello";
        let sa = build_suffix_array_stub(s);
        let lcp = lcp_array_stub(s, &sa);
        assert_eq!(lcp[0], 0);
    }

    #[test]
    fn test_banana_exact_sa() {
        let s = "banana";
        let sa = build_suffix_array_stub(s);
        assert_eq!(sa, vec![5, 3, 1, 0, 4, 2]);
    }

    #[test]
    fn test_find_all_occurrences() {
        let s = "banana";
        let sa = build_suffix_array_stub(s);
        let mut positions = sa_stub_find_all(s, &sa, "ana");
        positions.sort_unstable();
        assert_eq!(positions, vec![1, 3]);
    }

    #[test]
    fn test_find_all_no_match() {
        let s = "banana";
        let sa = build_suffix_array_stub(s);
        let positions = sa_stub_find_all(s, &sa, "xyz");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_longest_repeated_substring() {
        let s = "banana";
        let sa = build_suffix_array_stub(s);
        let lcp = lcp_array_stub(s, &sa);
        let (pos, len) = sa_stub_longest_repeated_substring(s, &sa, &lcp);
        assert_eq!(len, 3);
        assert_eq!(&s[pos..pos + len], "ana");
    }

    #[test]
    fn test_longest_repeated_substring_unique() {
        let s = "abcdefg";
        let sa = build_suffix_array_stub(s);
        let lcp = lcp_array_stub(s, &sa);
        let (_pos, len) = sa_stub_longest_repeated_substring(s, &sa, &lcp);
        assert_eq!(len, 0);
    }

    #[test]
    fn test_empty_string() {
        let sa = build_suffix_array_stub("");
        assert!(sa.is_empty());
        let lcp = lcp_array_stub("", &sa);
        assert!(lcp.is_empty());
        assert!(sa_stub_search("", &sa, "x").is_none());
        assert_eq!(sa_stub_count_occurrences("", &sa, "x"), 0);
        assert!(sa_stub_find_all("", &sa, "x").is_empty());
    }

    #[test]
    fn test_single_char() {
        let s = "a";
        let sa = build_suffix_array_stub(s);
        assert_eq!(sa, vec![0]);
        assert!(sa_stub_search(s, &sa, "a").is_some());
        assert!(sa_stub_search(s, &sa, "b").is_none());
    }

    #[test]
    fn test_all_same_chars() {
        let s = "aaaa";
        let sa = build_suffix_array_stub(s);
        assert_eq!(sa, vec![3, 2, 1, 0]);
        assert_eq!(sa_stub_count_occurrences(s, &sa, "aa"), 3);
    }

    #[test]
    fn test_kasai_lcp_values() {
        let s = "banana";
        let sa = build_suffix_array_stub(s);
        let lcp = lcp_array_stub(s, &sa);
        assert_eq!(lcp, vec![0, 1, 3, 0, 0, 2]);
    }

    #[test]
    fn test_longer_string() {
        let s = "abracadabra";
        let sa = build_suffix_array_stub(s);
        assert_eq!(sa.len(), s.len());
        for i in 0..sa.len() - 1 {
            assert!(s[sa[i]..] <= s[sa[i + 1]..]);
        }
        assert_eq!(sa_stub_count_occurrences(s, &sa, "abra"), 2);
        let mut positions = sa_stub_find_all(s, &sa, "abra");
        positions.sort_unstable();
        assert_eq!(positions, vec![0, 7]);
    }

    #[test]
    fn test_binary_string() {
        let s = "01001010010100101";
        let sa = build_suffix_array_stub(s);
        assert_eq!(sa.len(), s.len());
        for i in 0..sa.len() - 1 {
            assert!(s[sa[i]..] <= s[sa[i + 1]..]);
        }
    }

    #[test]
    fn test_count_empty_pattern() {
        let s = "hello";
        let sa = build_suffix_array_stub(s);
        assert_eq!(sa_stub_count_occurrences(s, &sa, ""), 0);
    }

    #[test]
    fn test_search_empty_pattern() {
        let s = "hello";
        let sa = build_suffix_array_stub(s);
        assert!(sa_stub_search(s, &sa, "").is_none());
    }

    #[test]
    fn test_find_all_empty_pattern() {
        let s = "hello";
        let sa = build_suffix_array_stub(s);
        assert!(sa_stub_find_all(s, &sa, "").is_empty());
    }
}
