// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fuzzy string matching using character overlap ratio.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub query: String,
    pub threshold: f64,
}

#[allow(dead_code)]
pub fn new_fuzzy_match(query: &str, threshold: f64) -> FuzzyMatch {
    FuzzyMatch {
        query: query.to_string(),
        threshold,
    }
}

/// Score: Jaccard on char sets, penalized by length difference ratio.
#[allow(dead_code)]
pub fn score(fm: &FuzzyMatch, candidate: &str) -> f64 {
    if fm.query.is_empty() && candidate.is_empty() {
        return 1.0;
    }
    if fm.query.is_empty() || candidate.is_empty() {
        return 0.0;
    }
    let q_chars: std::collections::HashSet<char> = fm.query.chars().collect();
    let c_chars: std::collections::HashSet<char> = candidate.chars().collect();
    let intersection = q_chars.intersection(&c_chars).count();
    let union = q_chars.union(&c_chars).count();
    let jaccard = if union == 0 { 0.0 } else { intersection as f64 / union as f64 };
    let q_len = fm.query.len();
    let c_len = candidate.len();
    let len_sim = 1.0 - (q_len as f64 - c_len as f64).abs() / (q_len.max(c_len) as f64);
    (jaccard + len_sim) / 2.0
}

#[allow(dead_code)]
pub fn matches(fm: &FuzzyMatch, candidate: &str) -> bool {
    score(fm, candidate) >= fm.threshold
}

#[allow(dead_code)]
pub fn best_match<'a>(fm: &FuzzyMatch, candidates: &'a [&str]) -> Option<&'a str> {
    candidates
        .iter()
        .max_by(|a, b| {
            let sa = score(fm, a);
            let sb = score(fm, b);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

#[allow(dead_code)]
pub fn set_threshold(fm: &mut FuzzyMatch, t: f64) {
    fm.threshold = t;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_score() {
        let fm = new_fuzzy_match("hello", 0.5);
        let s = score(&fm, "hello");
        assert!((s - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_empty_query_empty_candidate() {
        let fm = new_fuzzy_match("", 0.5);
        assert!((score(&fm, "") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_empty_query_nonempty_candidate() {
        let fm = new_fuzzy_match("", 0.5);
        assert!((score(&fm, "abc")).abs() < 1e-9);
    }

    #[test]
    fn test_no_candidates() {
        let fm = new_fuzzy_match("hello", 0.5);
        assert!(best_match(&fm, &[]).is_none());
    }

    #[test]
    fn test_threshold_filtering() {
        let fm = new_fuzzy_match("hello", 0.99);
        assert!(!matches(&fm, "world"));
        assert!(matches(&fm, "hello"));
    }

    #[test]
    fn test_best_match_exact_wins() {
        /* "hello" vs "world" — exact match should win */
        let fm = new_fuzzy_match("hello", 0.0);
        let candidates = &["world", "hello"];
        let best = best_match(&fm, candidates);
        assert_eq!(best, Some("hello"));
    }

    #[test]
    fn test_set_threshold() {
        let mut fm = new_fuzzy_match("hi", 0.9);
        set_threshold(&mut fm, 0.1);
        assert!((fm.threshold - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_score_range() {
        let fm = new_fuzzy_match("abc", 0.0);
        let s = score(&fm, "xyz");
        assert!((0.0..=1.0).contains(&s));
    }
}
