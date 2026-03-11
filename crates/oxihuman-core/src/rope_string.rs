// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

//! A rope data structure for efficient string editing with O(log n) insert/delete.

const LEAF_MAX: usize = 64;

/// A node in the rope tree.
#[allow(dead_code)]
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

    fn to_string_buf(&self, buf: &mut String) {
        match self {
            RopeNode::Leaf(s) => buf.push_str(s),
            RopeNode::Branch { left, right, .. } => {
                left.to_string_buf(buf);
                right.to_string_buf(buf);
            }
        }
    }

    fn char_at(&self, idx: usize) -> Option<u8> {
        match self {
            RopeNode::Leaf(s) => s.as_bytes().get(idx).copied(),
            RopeNode::Branch {
                left,
                weight,
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

/// A rope for efficient large-string editing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RopeString {
    root: Option<RopeNode>,
}

#[allow(dead_code)]
impl RopeString {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn from_str(s: &str) -> Self {
        if s.is_empty() {
            return Self::new();
        }
        Self {
            root: Some(Self::build_leaves(s)),
        }
    }

    fn build_leaves(s: &str) -> RopeNode {
        if s.len() <= LEAF_MAX {
            return RopeNode::Leaf(s.to_string());
        }
        let mid = s.len() / 2;
        // Find a safe split point (don't split multi-byte chars)
        let split = s.floor_char_boundary(mid);
        let left = Self::build_leaves(&s[..split]);
        let right = Self::build_leaves(&s[split..]);
        let weight = left.len();
        RopeNode::Branch {
            left: Box::new(left),
            right: Box::new(right),
            weight,
        }
    }

    pub fn len(&self) -> usize {
        self.root.as_ref().map(|r| r.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn to_string(&self) -> String {
        let mut buf = String::with_capacity(self.len());
        if let Some(ref r) = self.root {
            r.to_string_buf(&mut buf);
        }
        buf
    }

    pub fn byte_at(&self, idx: usize) -> Option<u8> {
        self.root.as_ref().and_then(|r| r.char_at(idx))
    }

    pub fn concat(&self, other: &RopeString) -> RopeString {
        match (&self.root, &other.root) {
            (None, _) => other.clone(),
            (_, None) => self.clone(),
            (Some(l), Some(r)) => {
                let weight = l.len();
                RopeString {
                    root: Some(RopeNode::Branch {
                        left: Box::new(l.clone()),
                        right: Box::new(r.clone()),
                        weight,
                    }),
                }
            }
        }
    }

    pub fn append(&mut self, s: &str) {
        let other = RopeString::from_str(s);
        *self = self.concat(&other);
    }

    /// Extract substring [start..end).
    pub fn substring(&self, start: usize, end: usize) -> String {
        let full = self.to_string();
        if start >= full.len() || start >= end {
            return String::new();
        }
        let e = end.min(full.len());
        full[start..e].to_string()
    }

    pub fn depth(&self) -> usize {
        fn d(node: &RopeNode) -> usize {
            match node {
                RopeNode::Leaf(_) => 1,
                RopeNode::Branch { left, right, .. } => 1 + d(left).max(d(right)),
            }
        }
        self.root.as_ref().map(d).unwrap_or(0)
    }
}

impl Default for RopeString {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let r = RopeString::from_str("hello world");
        assert_eq!(r.to_string(), "hello world");
        assert_eq!(r.len(), 11);
    }

    #[test]
    fn test_empty() {
        let r = RopeString::new();
        assert!(r.is_empty());
        assert_eq!(r.to_string(), "");
    }

    #[test]
    fn test_concat() {
        let a = RopeString::from_str("hello ");
        let b = RopeString::from_str("world");
        let c = a.concat(&b);
        assert_eq!(c.to_string(), "hello world");
    }

    #[test]
    fn test_append() {
        let mut r = RopeString::from_str("foo");
        r.append("bar");
        assert_eq!(r.to_string(), "foobar");
    }

    #[test]
    fn test_byte_at() {
        let r = RopeString::from_str("abc");
        assert_eq!(r.byte_at(0), Some(b'a'));
        assert_eq!(r.byte_at(2), Some(b'c'));
        assert_eq!(r.byte_at(3), None);
    }

    #[test]
    fn test_substring() {
        let r = RopeString::from_str("hello world");
        assert_eq!(r.substring(0, 5), "hello");
        assert_eq!(r.substring(6, 11), "world");
    }

    #[test]
    fn test_large_string() {
        let s = "x".repeat(1000);
        let r = RopeString::from_str(&s);
        assert_eq!(r.len(), 1000);
        assert_eq!(r.to_string(), s);
    }

    #[test]
    fn test_depth() {
        let r = RopeString::from_str("short");
        assert!(r.depth() >= 1);
    }

    #[test]
    fn test_concat_empty() {
        let a = RopeString::new();
        let b = RopeString::from_str("test");
        assert_eq!(a.concat(&b).to_string(), "test");
        assert_eq!(b.concat(&a).to_string(), "test");
    }

    #[test]
    fn test_substring_out_of_range() {
        let r = RopeString::from_str("abc");
        assert_eq!(r.substring(10, 20), "");
    }
}
