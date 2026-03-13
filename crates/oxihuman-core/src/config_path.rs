// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Dot-separated configuration path for hierarchical config access.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigPath {
    segments: Vec<String>,
}

#[allow(dead_code)]
impl ConfigPath {
    pub fn new(path: &str) -> Self {
        Self {
            segments: path.split('.').map(|s| s.to_string()).collect(),
        }
    }

    pub fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push(&mut self, segment: &str) {
        self.segments.push(segment.to_string());
    }

    pub fn pop(&mut self) -> Option<String> {
        self.segments.pop()
    }

    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            return None;
        }
        let mut p = self.clone();
        p.segments.pop();
        Some(p)
    }

    pub fn child(&self, name: &str) -> Self {
        let mut c = self.clone();
        c.segments.push(name.to_string());
        c
    }

    pub fn to_dotted(&self) -> String {
        self.segments.join(".")
    }

    pub fn first(&self) -> Option<&str> {
        self.segments.first().map(|s| s.as_str())
    }

    pub fn last(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }

    pub fn starts_with(&self, prefix: &ConfigPath) -> bool {
        if prefix.segments.len() > self.segments.len() {
            return false;
        }
        self.segments[..prefix.segments.len()] == prefix.segments[..]
    }

    pub fn segment(&self, idx: usize) -> Option<&str> {
        self.segments.get(idx).map(|s| s.as_str())
    }
}

impl std::fmt::Display for ConfigPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_dotted())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let p = ConfigPath::new("a.b.c");
        assert_eq!(p.depth(), 3);
    }

    #[test]
    fn test_root() {
        let p = ConfigPath::root();
        assert!(p.is_root());
        assert_eq!(p.depth(), 0);
    }

    #[test]
    fn test_push_pop() {
        let mut p = ConfigPath::root();
        p.push("x");
        assert_eq!(p.depth(), 1);
        assert_eq!(p.pop(), Some("x".to_string()));
        assert!(p.is_root());
    }

    #[test]
    fn test_parent() {
        let p = ConfigPath::new("a.b.c");
        let parent = p.parent().expect("should succeed");
        assert_eq!(parent.to_dotted(), "a.b");
    }

    #[test]
    fn test_child() {
        let p = ConfigPath::new("a.b");
        let c = p.child("c");
        assert_eq!(c.to_dotted(), "a.b.c");
    }

    #[test]
    fn test_first_last() {
        let p = ConfigPath::new("x.y.z");
        assert_eq!(p.first(), Some("x"));
        assert_eq!(p.last(), Some("z"));
    }

    #[test]
    fn test_starts_with() {
        let p = ConfigPath::new("a.b.c");
        let prefix = ConfigPath::new("a.b");
        assert!(p.starts_with(&prefix));
        let other = ConfigPath::new("x.y");
        assert!(!p.starts_with(&other));
    }

    #[test]
    fn test_display() {
        let p = ConfigPath::new("foo.bar");
        assert_eq!(format!("{p}"), "foo.bar");
    }

    #[test]
    fn test_segment() {
        let p = ConfigPath::new("a.b.c");
        assert_eq!(p.segment(1), Some("b"));
        assert_eq!(p.segment(5), None);
    }

    #[test]
    fn test_root_parent_none() {
        let p = ConfigPath::root();
        assert!(p.parent().is_none());
    }
}
