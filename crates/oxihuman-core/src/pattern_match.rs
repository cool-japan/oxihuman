// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Simple glob-style pattern matching (supports `*` and `?`).
#[allow(dead_code)]
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    glob_match_inner(&p, &t)
}

fn glob_match_inner(p: &[char], t: &[char]) -> bool {
    match (p.first(), t.first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(&'*'), _) => {
            // Try skipping the '*' or consuming one char from text
            glob_match_inner(&p[1..], t) || (!t.is_empty() && glob_match_inner(p, &t[1..]))
        }
        (Some(&'?'), Some(_)) => glob_match_inner(&p[1..], &t[1..]),
        (Some(pc), Some(tc)) if pc == tc => glob_match_inner(&p[1..], &t[1..]),
        _ => false,
    }
}

/// Case-insensitive glob match.
#[allow(dead_code)]
pub fn glob_match_ci(pattern: &str, text: &str) -> bool {
    glob_match(&pattern.to_lowercase(), &text.to_lowercase())
}

/// Check if text has prefix.
#[allow(dead_code)]
pub fn has_prefix(text: &str, prefix: &str) -> bool {
    text.starts_with(prefix)
}

/// Check if text has suffix.
#[allow(dead_code)]
pub fn has_suffix(text: &str, suffix: &str) -> bool {
    text.ends_with(suffix)
}

/// Count matches of a fixed literal in text.
#[allow(dead_code)]
pub fn count_occurrences(text: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut start = 0;
    while let Some(pos) = text[start..].find(needle) {
        count += 1;
        start += pos + needle.len();
    }
    count
}

/// Extract the portion of text matching between two delimiters.
#[allow(dead_code)]
pub fn extract_between<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = text.find(open).map(|i| i + open.len())?;
    let end = text[start..].find(close).map(|i| i + start)?;
    Some(&text[start..end])
}

/// Simple tokenizer splitting on whitespace.
#[allow(dead_code)]
pub fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

/// Replace all occurrences of `from` with `to`.
#[allow(dead_code)]
pub fn replace_all(text: &str, from: &str, to: &str) -> String {
    text.replace(from, to)
}

/// Filter lines matching a glob pattern.
#[allow(dead_code)]
pub fn grep_lines<'a>(lines: &[&'a str], pattern: &str) -> Vec<&'a str> {
    lines
        .iter()
        .copied()
        .filter(|l| glob_match(pattern, l))
        .collect()
}

#[allow(dead_code)]
pub struct PatternMatcher {
    patterns: Vec<String>,
}

#[allow(dead_code)]
impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }
    pub fn add(&mut self, pat: &str) {
        self.patterns.push(pat.to_string());
    }
    pub fn matches_any(&self, text: &str) -> bool {
        self.patterns.iter().any(|p| glob_match(p, text))
    }
    pub fn matches_all(&self, text: &str) -> bool {
        self.patterns.iter().all(|p| glob_match(p, text))
    }
    pub fn count(&self) -> usize {
        self.patterns.len()
    }
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
    pub fn clear(&mut self) {
        self.patterns.clear();
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_star_matches_any() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "main.txt"));
    }
    #[test]
    fn test_question_mark() {
        assert!(glob_match("te?t", "test"));
        assert!(!glob_match("te?t", "teat2"));
    }
    #[test]
    fn test_exact_match() {
        assert!(glob_match("hello", "hello"));
        assert!(!glob_match("hello", "world"));
    }
    #[test]
    fn test_glob_ci() {
        assert!(glob_match_ci("*.RS", "main.rs"));
    }
    #[test]
    fn test_count_occurrences() {
        assert_eq!(count_occurrences("abcabcabc", "abc"), 3);
        assert_eq!(count_occurrences("aaa", "b"), 0);
    }
    #[test]
    fn test_extract_between() {
        assert_eq!(extract_between("foo(bar)baz", "(", ")"), Some("bar"));
        assert_eq!(extract_between("no delimiters", "(", ")"), None);
    }
    #[test]
    fn test_tokenize() {
        let tokens = tokenize("  hello   world  ");
        assert_eq!(tokens, vec!["hello", "world"]);
    }
    #[test]
    fn test_replace_all() {
        assert_eq!(replace_all("aXbXc", "X", "-"), "a-b-c");
    }
    #[test]
    fn test_pattern_matcher_any() {
        let mut m = PatternMatcher::new();
        m.add("*.rs");
        m.add("*.toml");
        assert!(m.matches_any("Cargo.toml"));
        assert!(!m.matches_any("image.png"));
    }
    #[test]
    fn test_has_prefix_suffix() {
        assert!(has_prefix("hello_world", "hello"));
        assert!(has_suffix("hello_world", "world"));
    }
}
