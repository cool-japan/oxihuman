// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Aho-Corasick multi-pattern string search stub.

use std::collections::{HashMap, VecDeque};

/// A node in the Aho-Corasick automaton.
#[derive(Debug, Default, Clone)]
pub struct AcNode {
    pub children: HashMap<u8, usize>,
    pub fail: usize,
    /// Output: indices into the pattern list.
    pub output: Vec<usize>,
}

/// The Aho-Corasick automaton.
pub struct AhoCorasick {
    pub nodes: Vec<AcNode>,
    pub patterns: Vec<Vec<u8>>,
}

impl AhoCorasick {
    fn new() -> Self {
        AhoCorasick {
            nodes: vec![AcNode::default()],
            patterns: Vec::new(),
        }
    }
}

/// Create a new (empty) Aho-Corasick automaton.
pub fn new_aho_corasick() -> AhoCorasick {
    AhoCorasick::new()
}

/// Add a pattern to the automaton (must call `ac_build` before searching).
pub fn ac_add_pattern(ac: &mut AhoCorasick, pattern: &[u8]) {
    let id = ac.patterns.len();
    ac.patterns.push(pattern.to_vec());
    let mut cur = 0;
    for &b in pattern {
        if !ac.nodes[cur].children.contains_key(&b) {
            let next = ac.nodes.len();
            ac.nodes.push(AcNode::default());
            ac.nodes[cur].children.insert(b, next);
        }
        cur = ac.nodes[cur].children[&b];
    }
    ac.nodes[cur].output.push(id);
}

/// Build failure links (BFS over the trie).
pub fn ac_build(ac: &mut AhoCorasick) {
    let mut queue = VecDeque::new();
    /* root's children have fail = 0 */
    let root_children: Vec<(u8, usize)> =
        ac.nodes[0].children.iter().map(|(&b, &v)| (b, v)).collect();
    for (_, child) in &root_children {
        ac.nodes[*child].fail = 0;
        queue.push_back(*child);
    }
    while let Some(u) = queue.pop_front() {
        let children: Vec<(u8, usize)> =
            ac.nodes[u].children.iter().map(|(&b, &v)| (b, v)).collect();
        for (b, v) in children {
            let mut f = ac.nodes[u].fail;
            loop {
                if ac.nodes[f].children.contains_key(&b) {
                    f = ac.nodes[f].children[&b];
                    break;
                }
                if f == 0 {
                    break;
                }
                f = ac.nodes[f].fail;
            }
            /* avoid self-loop at root */
            if f == v {
                f = 0;
            }
            ac.nodes[v].fail = f;
            let fail_out = ac.nodes[f].output.clone();
            ac.nodes[v].output.extend(fail_out);
            queue.push_back(v);
        }
    }
}

/// A search match: (start_pos, pattern_index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcMatch {
    pub start: usize,
    pub pattern_id: usize,
}

/// Search `text` for all pattern occurrences. Returns matches in order.
pub fn ac_search(ac: &AhoCorasick, text: &[u8]) -> Vec<AcMatch> {
    let mut cur = 0;
    let mut results = Vec::new();

    for (i, &b) in text.iter().enumerate() {
        /* follow fail links until a transition exists or we reach root */
        loop {
            if ac.nodes[cur].children.contains_key(&b) {
                cur = ac.nodes[cur].children[&b];
                break;
            }
            if cur == 0 {
                break;
            }
            cur = ac.nodes[cur].fail;
        }
        for &pid in &ac.nodes[cur].output {
            let pat_len = ac.patterns[pid].len();
            let start = i + 1 - pat_len;
            results.push(AcMatch {
                start,
                pattern_id: pid,
            });
        }
    }
    results
}

/// Return the number of patterns.
pub fn ac_pattern_count(ac: &AhoCorasick) -> usize {
    ac.patterns.len()
}

/// Return `true` if any pattern appears in `text`.
pub fn ac_contains(ac: &AhoCorasick, text: &[u8]) -> bool {
    !ac_search(ac, text).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(patterns: &[&str]) -> AhoCorasick {
        let mut ac = new_aho_corasick();
        for p in patterns {
            ac_add_pattern(&mut ac, p.as_bytes());
        }
        ac_build(&mut ac);
        ac
    }

    #[test]
    fn test_single_pattern_found() {
        let ac = build(&["he"]);
        let matches = ac_search(&ac, b"ahebcd");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].start, 1);
    }

    #[test]
    fn test_multiple_patterns() {
        let ac = build(&["he", "she", "his", "hers"]);
        let matches = ac_search(&ac, b"ushers");
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_no_match() {
        let ac = build(&["xyz"]);
        let matches = ac_search(&ac, b"hello world");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_pattern_count() {
        let ac = build(&["a", "b", "c"]);
        assert_eq!(ac_pattern_count(&ac), 3);
    }

    #[test]
    fn test_contains_true() {
        let ac = build(&["world"]);
        assert!(ac_contains(&ac, b"hello world"));
    }

    #[test]
    fn test_contains_false() {
        let ac = build(&["moon"]);
        assert!(!ac_contains(&ac, b"hello world"));
    }

    #[test]
    fn test_overlapping_patterns() {
        /* "a" appears inside "aa" */
        let ac = build(&["a", "aa"]);
        let m = ac_search(&ac, b"aaa");
        assert!(m.len() >= 3); /* at least three "a" matches */
    }

    #[test]
    fn test_empty_text() {
        let ac = build(&["abc"]);
        assert!(ac_search(&ac, b"").is_empty());
    }

    #[test]
    fn test_pattern_at_end() {
        let ac = build(&["end"]);
        let m = ac_search(&ac, b"the end");
        assert!(!m.is_empty());
        assert_eq!(m[0].start, 4);
    }
}

// ── AcStubAutomaton: full Aho-Corasick with dict-suffix links ─────────────

/// A single match result from the Aho-Corasick automaton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcStubMatch {
    /// Index of the matched pattern in the original pattern list.
    pub pattern_id: usize,
    /// Start byte position in the text (inclusive).
    pub start: usize,
    /// End byte position in the text (exclusive).
    pub end: usize,
}

/// A node in the Aho-Corasick trie/automaton.
#[derive(Debug, Clone)]
pub struct AcStubNode {
    /// Mapping from byte value to child node index.
    pub children: HashMap<u8, usize>,
    /// Failure link: index of the longest proper suffix node that is also a
    /// prefix of some pattern. Root (index 0) points to itself.
    pub failure: usize,
    /// Pattern indices that end at this node (direct outputs).
    pub output: Vec<usize>,
    /// Depth of this node in the trie (= length of the string from root to here).
    pub depth: usize,
    /// Dictionary suffix link: points to the nearest ancestor (via failure chain)
    /// that is a complete pattern output node. `None` if no such ancestor exists.
    pub dict_suffix: Option<usize>,
}

impl AcStubNode {
    fn new_stub(depth: usize) -> Self {
        Self {
            children: HashMap::new(),
            failure: 0,
            output: Vec::new(),
            depth,
            dict_suffix: None,
        }
    }
}

/// Configuration for how the Aho-Corasick automaton performs matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcStubMatchKind {
    /// Report all overlapping matches.
    Overlapping,
    /// Report only non-overlapping matches (leftmost, earliest pattern id wins on tie).
    NonOverlapping,
}

/// The Aho-Corasick automaton (stub module variant).
#[derive(Debug, Clone)]
pub struct AcStubAutomaton {
    /// All nodes of the automaton. Index 0 is always the root.
    pub nodes: Vec<AcStubNode>,
    /// Number of patterns inserted.
    pub pattern_count: usize,
    /// Stored pattern lengths, indexed by pattern id.
    pattern_lengths: Vec<usize>,
    /// Whether matching is case-insensitive.
    case_insensitive: bool,
}

impl AcStubAutomaton {
    /// Build a new Aho-Corasick automaton from the given patterns.
    ///
    /// Empty patterns are silently skipped (they will never match).
    pub fn new(patterns: &[&str]) -> Self {
        Self::builder(patterns, false)
    }

    /// Build with case-insensitive matching.
    pub fn new_case_insensitive(patterns: &[&str]) -> Self {
        Self::builder(patterns, true)
    }

    fn builder(patterns: &[&str], case_insensitive: bool) -> Self {
        let mut ac = Self {
            nodes: vec![AcStubNode::new_stub(0)],
            pattern_count: patterns.len(),
            pattern_lengths: Vec::with_capacity(patterns.len()),
            case_insensitive,
        };

        for (pid, &pat) in patterns.iter().enumerate() {
            ac.pattern_lengths.push(pat.len());
            if pat.is_empty() {
                continue;
            }
            ac.insert_pattern(pat.as_bytes(), pid);
        }

        ac.compute_failure_and_dict_links();
        ac
    }

    fn insert_pattern(&mut self, pattern: &[u8], pattern_id: usize) {
        let mut cur = 0usize;
        for (i, &byte) in pattern.iter().enumerate() {
            let b = if self.case_insensitive {
                byte.to_ascii_lowercase()
            } else {
                byte
            };
            let next = if let Some(&child) = self.nodes[cur].children.get(&b) {
                child
            } else {
                let idx = self.nodes.len();
                self.nodes.push(AcStubNode::new_stub(i + 1));
                self.nodes[cur].children.insert(b, idx);
                idx
            };
            cur = next;
        }
        self.nodes[cur].output.push(pattern_id);
    }

    fn compute_failure_and_dict_links(&mut self) {
        let mut queue = VecDeque::new();

        let root_children: Vec<(u8, usize)> = self.nodes[0]
            .children
            .iter()
            .map(|(&b, &c)| (b, c))
            .collect();

        for &(_byte, child_idx) in &root_children {
            queue.push_back(child_idx);
        }

        while let Some(u) = queue.pop_front() {
            let children_of_u: Vec<(u8, usize)> = self.nodes[u]
                .children
                .iter()
                .map(|(&b, &c)| (b, c))
                .collect();

            for (byte, child_idx) in children_of_u {
                let mut f = self.nodes[u].failure;
                loop {
                    if self.nodes[f].children.contains_key(&byte) {
                        let target = self.nodes[f].children[&byte];
                        self.nodes[child_idx].failure = target;
                        break;
                    }
                    if f == 0 {
                        self.nodes[child_idx].failure = 0;
                        break;
                    }
                    f = self.nodes[f].failure;
                }

                let fail_of_child = self.nodes[child_idx].failure;
                if !self.nodes[fail_of_child].output.is_empty() {
                    self.nodes[child_idx].dict_suffix = Some(fail_of_child);
                } else {
                    self.nodes[child_idx].dict_suffix = self.nodes[fail_of_child].dict_suffix;
                }

                queue.push_back(child_idx);
            }
        }
    }

    /// Find all overlapping matches in `text`, returned sorted by start position
    /// (ties broken by pattern id).
    pub fn find_overlapping(&self, text: &str) -> Vec<AcStubMatch> {
        let bytes = text.as_bytes();
        let mut results = Vec::new();
        let mut state = 0usize;

        for (i, &byte) in bytes.iter().enumerate() {
            let b = if self.case_insensitive {
                byte.to_ascii_lowercase()
            } else {
                byte
            };
            state = self.goto(state, b);
            self.collect_outputs(state, i, &mut results);
        }

        results.sort_by(|a, b| a.start.cmp(&b.start).then(a.pattern_id.cmp(&b.pattern_id)));
        results
    }

    /// Find non-overlapping matches (leftmost-first, earliest pattern wins on tie).
    pub fn find_non_overlapping(&self, text: &str) -> Vec<AcStubMatch> {
        let bytes = text.as_bytes();
        let mut results = Vec::new();
        let mut state = 0usize;
        let mut last_end = 0usize;

        for (i, &byte) in bytes.iter().enumerate() {
            let b = if self.case_insensitive {
                byte.to_ascii_lowercase()
            } else {
                byte
            };
            state = self.goto(state, b);

            let mut candidates = Vec::new();
            self.collect_outputs(state, i, &mut candidates);

            candidates.sort_by(|a, b| a.start.cmp(&b.start).then(a.pattern_id.cmp(&b.pattern_id)));

            for c in candidates {
                if c.start >= last_end {
                    last_end = c.end;
                    results.push(c);
                    break;
                }
            }
        }

        results
    }

    /// Generic search returning `(pattern_id, start_position)` tuples, sorted by
    /// start position. This is the overlapping mode (compatible with the old stub API).
    pub fn search(&self, text: &str) -> Vec<(usize, usize)> {
        self.find_overlapping(text)
            .into_iter()
            .map(|m| (m.pattern_id, m.start))
            .collect()
    }

    /// Count all overlapping matches.
    pub fn count_matches(&self, text: &str) -> usize {
        self.find_overlapping(text).len()
    }

    /// Check if any pattern matches anywhere in `text`.
    pub fn any_match(&self, text: &str) -> bool {
        let bytes = text.as_bytes();
        let mut state = 0usize;
        for &byte in bytes {
            let b = if self.case_insensitive {
                byte.to_ascii_lowercase()
            } else {
                byte
            };
            state = self.goto(state, b);
            if !self.nodes[state].output.is_empty() {
                return true;
            }
            if self.nodes[state].dict_suffix.is_some() {
                return true;
            }
        }
        false
    }

    /// Return the first (leftmost) match, if any.
    pub fn first_match(&self, text: &str) -> Option<AcStubMatch> {
        let bytes = text.as_bytes();
        let mut state = 0usize;
        let mut best: Option<AcStubMatch> = None;

        for (i, &byte) in bytes.iter().enumerate() {
            let b = if self.case_insensitive {
                byte.to_ascii_lowercase()
            } else {
                byte
            };
            state = self.goto(state, b);

            let mut tmp = Vec::new();
            self.collect_outputs(state, i, &mut tmp);

            for m in tmp {
                let dominated = if let Some(ref current) = best {
                    m.start > current.start
                        || (m.start == current.start && m.pattern_id >= current.pattern_id)
                } else {
                    false
                };
                if !dominated {
                    best = Some(m);
                }
            }

            if let Some(ref b_match) = best {
                if b_match.end <= i + 1 {
                    return best;
                }
            }
        }

        best
    }

    fn goto(&self, mut state: usize, b: u8) -> usize {
        loop {
            if let Some(&next) = self.nodes[state].children.get(&b) {
                return next;
            }
            if state == 0 {
                return 0;
            }
            state = self.nodes[state].failure;
        }
    }

    fn collect_outputs(&self, state: usize, pos: usize, results: &mut Vec<AcStubMatch>) {
        for &pid in &self.nodes[state].output {
            let plen = self.pattern_lengths.get(pid).copied().unwrap_or(0);
            if plen == 0 {
                continue;
            }
            let start = pos + 1 - plen;
            results.push(AcStubMatch {
                pattern_id: pid,
                start,
                end: pos + 1,
            });
        }

        let mut link = self.nodes[state].dict_suffix;
        while let Some(node_idx) = link {
            for &pid in &self.nodes[node_idx].output {
                let plen = self.pattern_lengths.get(pid).copied().unwrap_or(0);
                if plen == 0 {
                    continue;
                }
                let start = pos + 1 - plen;
                results.push(AcStubMatch {
                    pattern_id: pid,
                    start,
                    end: pos + 1,
                });
            }
            link = self.nodes[node_idx].dict_suffix;
        }
    }
}

// ── Legacy public API (compatible with the old stub) ─────────────────

pub fn ac_stub_search(text: &str, patterns: &[&str]) -> Vec<(usize, usize)> {
    let ac = AcStubAutomaton::new(patterns);
    ac.search(text)
}

pub fn ac_stub_count_matches(text: &str, patterns: &[&str]) -> usize {
    let ac = AcStubAutomaton::new(patterns);
    ac.count_matches(text)
}

pub fn ac_stub_any_match(text: &str, patterns: &[&str]) -> bool {
    let ac = AcStubAutomaton::new(patterns);
    ac.any_match(text)
}

pub fn ac_stub_first_match(text: &str, patterns: &[&str]) -> Option<(usize, usize)> {
    let ac = AcStubAutomaton::new(patterns);
    ac.first_match(text).map(|m| (m.pattern_id, m.start))
}

#[cfg(test)]
mod tests_stub {
    use super::*;

    #[test]
    fn test_ac_stub_search_single() {
        /* finds single pattern */
        let matches = ac_stub_search("hello world", &["world"]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (0, 6));
    }

    #[test]
    fn test_ac_stub_search_multiple() {
        /* finds multiple patterns */
        let matches = ac_stub_search("abcabc", &["abc", "bc"]);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_ac_stub_count_matches() {
        /* counts total matches */
        let n = ac_stub_count_matches("abababab", &["ab"]);
        assert_eq!(n, 4);
    }

    #[test]
    fn test_ac_stub_any_match_true() {
        /* any_match returns true when pattern exists */
        assert!(ac_stub_any_match("hello world", &["world", "xyz"]));
    }

    #[test]
    fn test_ac_stub_any_match_false() {
        /* any_match returns false when no pattern matches */
        assert!(!ac_stub_any_match("hello", &["xyz", "foo"]));
    }

    #[test]
    fn test_ac_stub_first_match() {
        /* first match returns earliest occurrence */
        let m = ac_stub_first_match("abcdef", &["def", "abc"]);
        assert!(m.is_some());
        assert_eq!(m.expect("should succeed").1, 0); // "abc" at position 0 comes first
    }

    #[test]
    fn test_empty_patterns() {
        let ac = AcStubAutomaton::new(&[]);
        assert_eq!(ac.count_matches("anything"), 0);
        assert!(!ac.any_match("anything"));
    }

    #[test]
    fn test_empty_text() {
        let ac = AcStubAutomaton::new(&["foo", "bar"]);
        assert_eq!(ac.count_matches(""), 0);
        assert!(!ac.any_match(""));
    }

    #[test]
    fn test_empty_pattern_in_list() {
        let ac = AcStubAutomaton::new(&["", "abc", ""]);
        let hits = ac.find_overlapping("xyzabcxyz");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].pattern_id, 1);
        assert_eq!(hits[0].start, 3);
        assert_eq!(hits[0].end, 6);
    }

    #[test]
    fn test_overlapping_matches() {
        let ac = AcStubAutomaton::new(&["abc", "bc", "c"]);
        let hits = ac.find_overlapping("abc");
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].start, 0);
        assert_eq!(hits[1].start, 1);
        assert_eq!(hits[2].start, 2);
    }

    #[test]
    fn test_non_overlapping_matches() {
        let ac = AcStubAutomaton::new(&["ab", "bc"]);
        let hits = ac.find_non_overlapping("abcabc");
        assert!(hits.len() >= 2);
        assert_eq!(hits[0].start, 0);
        assert_eq!(hits[0].pattern_id, 0);
    }

    #[test]
    fn test_case_insensitive() {
        let ac = AcStubAutomaton::new_case_insensitive(&["hello"]);
        assert!(ac.any_match("HELLO WORLD"));
        assert!(ac.any_match("Hello World"));
        assert!(ac.any_match("hElLo WoRlD"));

        let hits = ac.find_overlapping("HeLLo");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].start, 0);
        assert_eq!(hits[0].end, 5);
    }

    #[test]
    fn test_case_insensitive_multiple() {
        let ac = AcStubAutomaton::new_case_insensitive(&["cat", "dog"]);
        let hits = ac.find_overlapping("The CAT chased the DOG");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_first_match_returns_leftmost() {
        let ac = AcStubAutomaton::new(&["xyz", "abc"]);
        let m = ac.first_match("___abc___xyz");
        assert!(m.is_some());
        let m = m.expect("should succeed");
        assert_eq!(m.pattern_id, 1);
        assert_eq!(m.start, 3);
    }

    #[test]
    fn test_first_match_none() {
        let ac = AcStubAutomaton::new(&["xyz"]);
        assert!(ac.first_match("abcdef").is_none());
    }

    #[test]
    fn test_repeated_pattern() {
        let ac = AcStubAutomaton::new(&["aa"]);
        let hits = ac.find_overlapping("aaaa");
        assert_eq!(hits.len(), 3);
    }

    #[test]
    fn test_classic_aho_corasick_example() {
        let ac = AcStubAutomaton::new(&["he", "she", "his", "hers"]);
        let hits = ac.find_overlapping("ushers");
        assert_eq!(hits.len(), 3);

        let ids: Vec<usize> = hits.iter().map(|m| m.pattern_id).collect();
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
    }

    #[test]
    fn test_acmatch_struct() {
        let m = AcStubMatch {
            pattern_id: 2,
            start: 5,
            end: 10,
        };
        let m2 = m.clone();
        assert_eq!(m, m2);
        let _ = format!("{:?}", m);
    }

    #[test]
    fn test_single_byte_patterns() {
        let ac = AcStubAutomaton::new(&["a", "b", "c"]);
        let hits = ac.find_overlapping("abc");
        assert_eq!(hits.len(), 3);
    }

    #[test]
    fn test_no_match() {
        let ac = AcStubAutomaton::new(&["xyz", "123"]);
        assert!(!ac.any_match("abcdef"));
        assert_eq!(ac.count_matches("abcdef"), 0);
    }

    #[test]
    fn test_pattern_at_boundaries() {
        let ac = AcStubAutomaton::new(&["abc"]);
        let hits = ac.find_overlapping("abcxyz");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].start, 0);

        let hits = ac.find_overlapping("xyzabc");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].start, 3);
    }

    #[test]
    fn test_duplicate_patterns() {
        let ac = AcStubAutomaton::new(&["ab", "ab"]);
        let hits = ac.find_overlapping("ab");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_failure_link_chain() {
        let ac = AcStubAutomaton::new(&["abcabd", "abd"]);
        let hits = ac.find_overlapping("abcabd");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_long_text_many_patterns() {
        let patterns: Vec<&str> = vec!["the", "he", "she", "his", "hers", "her", "er"];
        let ac = AcStubAutomaton::new(&patterns);
        let text = "the quick brown fox said she hers his her and the other";
        let hits = ac.find_overlapping(text);
        assert!(hits.len() > 5);
    }

    #[test]
    fn test_match_kind_enum() {
        let o = AcStubMatchKind::Overlapping;
        let n = AcStubMatchKind::NonOverlapping;
        assert_ne!(o, n);
    }
}
