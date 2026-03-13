// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! String edit-distance algorithms: Levenshtein, Hamming, Jaro.

#[allow(dead_code, clippy::needless_range_loop)]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

#[allow(dead_code)]
pub fn hamming_distance(a: &str, b: &str) -> Option<usize> {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    if ac.len() != bc.len() {
        return None;
    }
    let dist = ac.iter().zip(bc.iter()).filter(|(x, y)| x != y).count();
    Some(dist)
}

#[allow(dead_code)]
pub fn jaro_similarity(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let match_dist = ((a.len().max(b.len()) as f64 / 2.0).floor() as usize).saturating_sub(1);
    let mut a_matched = vec![false; a.len()];
    let mut b_matched = vec![false; b.len()];
    let mut matches = 0usize;
    for i in 0..a.len() {
        let lo = i.saturating_sub(match_dist);
        let hi = (i + match_dist + 1).min(b.len());
        for j in lo..hi {
            if !b_matched[j] && a[i] == b[j] {
                a_matched[i] = true;
                b_matched[j] = true;
                matches += 1;
                break;
            }
        }
    }
    if matches == 0 {
        return 0.0;
    }
    let mut transpositions = 0usize;
    let mut k = 0;
    for i in 0..a.len() {
        if a_matched[i] {
            while !b_matched[k] {
                k += 1;
            }
            if a[i] != b[k] {
                transpositions += 1;
            }
            k += 1;
        }
    }
    let m = matches as f64;
    let t = transpositions as f64 / 2.0;
    (m / a.len() as f64 + m / b.len() as f64 + (m - t) / m) / 3.0
}

#[allow(dead_code)]
pub fn is_within_distance(a: &str, b: &str, max: usize) -> bool {
    levenshtein(a, b) <= max
}

#[allow(dead_code)]
pub fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_equal() {
        assert_eq!(levenshtein("abc", "abc"), 0);
    }

    #[test]
    fn test_levenshtein_insert() {
        assert_eq!(levenshtein("abc", "abcd"), 1);
    }

    #[test]
    fn test_levenshtein_replace() {
        assert_eq!(levenshtein("abc", "axc"), 1);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn test_hamming_same_len() {
        assert_eq!(hamming_distance("abc", "axc"), Some(1));
    }

    #[test]
    fn test_hamming_diff_len() {
        assert_eq!(hamming_distance("ab", "abc"), None);
    }

    #[test]
    fn test_jaro_identical() {
        let sim = jaro_similarity("abc", "abc");
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_within_distance() {
        assert!(is_within_distance("abc", "axc", 1));
        assert!(!is_within_distance("abc", "xyz", 1));
    }

    #[test]
    fn test_common_prefix() {
        assert_eq!(common_prefix_len("hello", "help"), 3);
    }

    #[test]
    fn test_common_prefix_none() {
        assert_eq!(common_prefix_len("abc", "xyz"), 0);
    }
}
