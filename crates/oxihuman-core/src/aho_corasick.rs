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
