// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Levenshtein (edit) distance and related string similarity functions.

#[allow(clippy::needless_range_loop)]
pub fn edit_distance_lev(a: &str, b: &str) -> usize {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let m = ac.len();
    let n = bc.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if ac[i - 1] == bc[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

pub fn edit_similarity_lev(a: &str, b: &str) -> f32 {
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    let dist = edit_distance_lev(a, b);
    1.0 - dist as f32 / max_len as f32
}

pub fn edit_distance_bounded_lev(a: &str, b: &str, max_dist: usize) -> Option<usize> {
    let d = edit_distance_lev(a, b);
    if d > max_dist {
        None
    } else {
        Some(d)
    }
}

pub fn edit_is_close_lev(a: &str, b: &str, threshold: usize) -> bool {
    edit_distance_lev(a, b) <= threshold
}

#[allow(clippy::needless_range_loop)]
pub fn longest_common_subsequence_lev(a: &str, b: &str) -> usize {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let m = ac.len();
    let n = bc.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if ac[i - 1] == bc[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }
    dp[m][n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_distance_equal() {
        /* same string has distance 0 */
        assert_eq!(edit_distance_lev("hello", "hello"), 0);
    }

    #[test]
    fn test_edit_distance_insert() {
        /* one insertion */
        assert_eq!(edit_distance_lev("abc", "abcd"), 1);
    }

    #[test]
    fn test_edit_distance_delete() {
        /* one deletion */
        assert_eq!(edit_distance_lev("abcd", "abc"), 1);
    }

    #[test]
    fn test_edit_distance_replace() {
        /* one replacement */
        assert_eq!(edit_distance_lev("abc", "axc"), 1);
    }

    #[test]
    fn test_edit_similarity_identical() {
        /* identical strings have similarity 1.0 */
        let s = edit_similarity_lev("hello", "hello");
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_edit_distance_bounded_within() {
        /* bounded distance returns Some when within threshold */
        assert_eq!(edit_distance_bounded_lev("abc", "axc", 2), Some(1));
        assert!(edit_distance_bounded_lev("abc", "xyz", 1).is_none());
    }

    #[test]
    fn test_lcs() {
        /* LCS of "ABCBDAB" and "BDCAB" is 4 */
        assert_eq!(longest_common_subsequence_lev("ABCBDAB", "BDCAB"), 4);
    }
}
