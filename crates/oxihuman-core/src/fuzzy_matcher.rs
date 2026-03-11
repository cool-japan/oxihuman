// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fuzzy string matching with score-based ranking.

pub fn fuzzy_match_score(pattern: &str, text: &str) -> f32 {
    if pattern.is_empty() {
        return 1.0;
    }
    if text.is_empty() {
        return 0.0;
    }
    let p: Vec<char> = pattern.to_lowercase().chars().collect();
    let t: Vec<char> = text.to_lowercase().chars().collect();

    // Prefix bonus: if text starts with pattern
    let prefix_bonus = if t.starts_with(&p) { 0.3f32 } else { 0.0 };

    // Exact match
    if p == t {
        return 1.0;
    }

    let mut pi = 0usize;
    let mut consecutive = 0usize;
    let mut consecutive_bonus = 0.0f32;
    let mut matched = 0usize;

    for &tc in &t {
        if pi < p.len() && tc == p[pi] {
            pi += 1;
            matched += 1;
            consecutive += 1;
            if consecutive > 1 {
                consecutive_bonus += 0.05 * consecutive as f32;
            }
        } else {
            consecutive = 0;
        }
    }

    if pi < p.len() {
        return 0.0; // not all pattern chars matched
    }

    let base_score = matched as f32 / t.len() as f32;
    (base_score + prefix_bonus + consecutive_bonus).min(1.0)
}

pub fn fuzzy_matches(pattern: &str, text: &str, threshold: f32) -> bool {
    fuzzy_match_score(pattern, text) >= threshold
}

pub fn fuzzy_best_match<'a>(pattern: &str, candidates: &[&'a str]) -> Option<&'a str> {
    candidates
        .iter()
        .map(|&c| (c, fuzzy_match_score(pattern, c)))
        .filter(|&(_, s)| s > 0.0)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(c, _)| c)
}

pub fn fuzzy_rank_candidates<'a>(pattern: &str, candidates: &[&'a str]) -> Vec<(&'a str, f32)> {
    let mut ranked: Vec<(&'a str, f32)> = candidates
        .iter()
        .map(|&c| (c, fuzzy_match_score(pattern, c)))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_exact_match() {
        /* exact match scores 1.0 */
        let s = fuzzy_match_score("hello", "hello");
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fuzzy_no_match() {
        /* pattern chars not in text scores 0 */
        let s = fuzzy_match_score("xyz", "abc");
        assert_eq!(s, 0.0);
    }

    #[test]
    fn test_fuzzy_partial_match() {
        /* partial match returns non-zero score */
        let s = fuzzy_match_score("hlo", "hello");
        assert!(s > 0.0);
    }

    #[test]
    fn test_fuzzy_matches_threshold() {
        /* threshold filtering works */
        assert!(fuzzy_matches("hello", "hello", 0.9));
        assert!(!fuzzy_matches("xyz", "abc", 0.1));
    }

    #[test]
    fn test_fuzzy_best_match() {
        /* best match selects highest scoring candidate */
        let candidates = &["hello", "world", "help"];
        let best = fuzzy_best_match("hel", candidates);
        assert!(best.is_some());
        let b = best.unwrap();
        assert!(b == "hello" || b == "help");
    }

    #[test]
    fn test_fuzzy_rank_candidates() {
        /* ranked candidates are sorted by score descending */
        let candidates = &["hello", "world", "help"];
        let ranked = fuzzy_rank_candidates("hel", candidates);
        assert_eq!(ranked.len(), 3);
        for i in 0..ranked.len() - 1 {
            assert!(ranked[i].1 >= ranked[i + 1].1);
        }
    }

    #[test]
    fn test_fuzzy_empty_pattern() {
        /* empty pattern matches everything with score 1 */
        let s = fuzzy_match_score("", "hello");
        assert!((s - 1.0).abs() < 1e-5);
    }
}
