// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rope data structure for efficient string concatenation and split.

/// A rope node.
#[derive(Debug, Clone)]
enum RopeNode {
    Leaf(String),
    Branch {
        left: Box<RopeNode>,
        right: Box<RopeNode>,
        weight: usize,
    },
}

impl RopeNode {
    fn len(&self) -> usize {
        match self {
            RopeNode::Leaf(s) => s.len(),
            RopeNode::Branch { weight, right, .. } => weight + right.len(),
        }
    }

    fn collect(&self, buf: &mut String) {
        match self {
            RopeNode::Leaf(s) => buf.push_str(s),
            RopeNode::Branch { left, right, .. } => {
                left.collect(buf);
                right.collect(buf);
            }
        }
    }

    fn char_at(&self, idx: usize) -> Option<char> {
        match self {
            RopeNode::Leaf(s) => s.chars().nth(idx),
            RopeNode::Branch {
                weight,
                left,
                right,
            } => {
                if idx < *weight {
                    left.char_at(idx)
                } else {
                    right.char_at(idx - weight)
                }
            }
        }
    }
}

/// A rope data structure.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Rope {
    root: Option<Box<RopeNode>>,
}

impl Rope {
    /// Create an empty rope.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a rope from a string.
    #[allow(dead_code)]
    pub fn from_string(s: &str) -> Self {
        if s.is_empty() {
            return Self::new();
        }
        Self {
            root: Some(Box::new(RopeNode::Leaf(s.to_string()))),
        }
    }

    /// Length in bytes.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.root.as_ref().map(|r| r.len()).unwrap_or(0)
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Collect into a `String`.
    #[allow(dead_code)]
    pub fn collect_string(&self) -> String {
        let mut buf = String::new();
        if let Some(root) = &self.root {
            root.collect(&mut buf);
        }
        buf
    }

    /// Concatenate another rope.
    #[allow(dead_code)]
    pub fn concat(self, other: Rope) -> Rope {
        match (self.root, other.root) {
            (None, r) => Rope { root: r },
            (l, None) => Rope { root: l },
            (Some(l), Some(r)) => {
                let weight = l.len();
                Rope {
                    root: Some(Box::new(RopeNode::Branch {
                        weight,
                        left: l,
                        right: r,
                    })),
                }
            }
        }
    }

    /// Get the character at byte index `idx`.
    #[allow(dead_code)]
    pub fn char_at(&self, idx: usize) -> Option<char> {
        self.root.as_ref()?.char_at(idx)
    }

    /// Split into two ropes at byte position `at`.
    #[allow(dead_code)]
    pub fn split_at(self, at: usize) -> (Rope, Rope) {
        let s = self.collect_string();
        let left = Rope::from_string(&s[..at.min(s.len())]);
        let right = Rope::from_string(&s[at.min(s.len())..]);
        (left, right)
    }

    /// Append a string slice.
    #[allow(dead_code)]
    pub fn append(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        let other = Rope::from_string(s);
        let old = std::mem::take(self);
        *self = old.concat(other);
    }
}

/// Rope concatenation helper.
#[allow(dead_code)]
pub fn rope_concat(a: Rope, b: Rope) -> Rope {
    a.concat(b)
}

/// Create a rope from a string slice.
#[allow(dead_code)]
pub fn rope_from(s: &str) -> Rope {
    Rope::from_string(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_rope_len_zero() {
        let r = Rope::new();
        assert_eq!(r.len(), 0);
        assert!(r.is_empty());
    }

    #[test]
    fn from_string_len_correct() {
        let r = Rope::from_string("hello");
        assert_eq!(r.len(), 5);
    }

    #[test]
    fn collect_string_matches_input() {
        let r = Rope::from_string("hello world");
        assert_eq!(r.collect_string(), "hello world");
    }

    #[test]
    fn concat_two_ropes() {
        let a = Rope::from_string("foo");
        let b = Rope::from_string("bar");
        let c = rope_concat(a, b);
        assert_eq!(c.collect_string(), "foobar");
    }

    #[test]
    fn concat_with_empty() {
        let a = Rope::from_string("foo");
        let b = Rope::new();
        let c = rope_concat(a, b);
        assert_eq!(c.collect_string(), "foo");
    }

    #[test]
    fn char_at_correct() {
        let r = Rope::from_string("abcde");
        assert_eq!(r.char_at(2), Some('c'));
    }

    #[test]
    fn char_at_out_of_bounds() {
        let r = Rope::from_string("ab");
        assert!(r.char_at(10).is_none());
    }

    #[test]
    fn split_at_correct() {
        let r = Rope::from_string("hello");
        let (a, b) = r.split_at(2);
        assert_eq!(a.collect_string(), "he");
        assert_eq!(b.collect_string(), "llo");
    }

    #[test]
    fn append_works() {
        let mut r = Rope::from_string("hello");
        r.append(" world");
        assert_eq!(r.collect_string(), "hello world");
    }

    #[test]
    fn rope_from_helper() {
        let r = rope_from("test");
        assert_eq!(r.len(), 4);
    }
}
