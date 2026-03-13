// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! JSON Pointer (RFC 6901) resolver stub.

/// Error type for JSON Pointer operations.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonPointerError {
    InvalidSyntax(String),
    KeyNotFound(String),
    IndexOutOfRange(usize),
    InvalidIndex(String),
}

impl std::fmt::Display for JsonPointerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSyntax(s) => write!(f, "invalid JSON pointer syntax: {s}"),
            Self::KeyNotFound(k) => write!(f, "key not found: {k}"),
            Self::IndexOutOfRange(i) => write!(f, "index out of range: {i}"),
            Self::InvalidIndex(s) => write!(f, "invalid array index: {s}"),
        }
    }
}

/// A parsed JSON Pointer (RFC 6901).
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPointer {
    tokens: Vec<String>,
}

impl JsonPointer {
    /// Parse a JSON Pointer string (e.g. `"/foo/bar/0"`).
    pub fn parse(s: &str) -> Result<Self, JsonPointerError> {
        if s.is_empty() {
            return Ok(JsonPointer { tokens: vec![] });
        }
        if !s.starts_with('/') {
            return Err(JsonPointerError::InvalidSyntax(s.to_string()));
        }
        let tokens = s[1..]
            .split('/')
            .map(|tok| tok.replace("~1", "/").replace("~0", "~"))
            .collect();
        Ok(JsonPointer { tokens })
    }

    /// Return the reference tokens of this pointer.
    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }

    /// Return whether this pointer is the root (empty).
    pub fn is_root(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Return the depth (number of tokens).
    pub fn depth(&self) -> usize {
        self.tokens.len()
    }

    /// Serialize back to a JSON Pointer string.
    pub fn to_string_repr(&self) -> String {
        if self.tokens.is_empty() {
            return String::new();
        }
        let mut out = String::new();
        for tok in &self.tokens {
            out.push('/');
            out.push_str(&tok.replace('~', "~0").replace('/', "~1"));
        }
        out
    }
}

/// Escape a single reference token per RFC 6901.
pub fn escape_token(tok: &str) -> String {
    tok.replace('~', "~0").replace('/', "~1")
}

/// Unescape a single reference token per RFC 6901.
pub fn unescape_token(tok: &str) -> String {
    tok.replace("~1", "/").replace("~0", "~")
}

/// Return the last token of a pointer, or `None` if root.
pub fn pointer_leaf(ptr: &JsonPointer) -> Option<&str> {
    ptr.tokens().last().map(|s| s.as_str())
}

/// Return a new pointer that is the parent of `ptr`, or `None` if root.
pub fn pointer_parent(ptr: &JsonPointer) -> Option<JsonPointer> {
    if ptr.is_root() {
        return None;
    }
    let tokens = ptr.tokens[..ptr.tokens.len().saturating_sub(1)].to_vec();
    Some(JsonPointer { tokens })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_root() {
        /* root pointer is empty string */
        let p = JsonPointer::parse("").expect("should succeed");
        assert!(p.is_root());
    }

    #[test]
    fn test_parse_simple() {
        /* parse /foo/bar */
        let p = JsonPointer::parse("/foo/bar").expect("should succeed");
        assert_eq!(p.tokens(), &["foo", "bar"]);
    }

    #[test]
    fn test_escape_tilde() {
        /* ~ must be escaped as ~0 */
        let p = JsonPointer::parse("/a~0b").expect("should succeed");
        assert_eq!(p.tokens()[0], "a~b");
    }

    #[test]
    fn test_escape_slash() {
        /* / in token must be escaped as ~1 */
        let p = JsonPointer::parse("/a~1b").expect("should succeed");
        assert_eq!(p.tokens()[0], "a/b");
    }

    #[test]
    fn test_invalid_no_leading_slash() {
        /* must start with / */
        assert!(JsonPointer::parse("foo/bar").is_err());
    }

    #[test]
    fn test_depth() {
        /* depth counts tokens */
        let p = JsonPointer::parse("/a/b/c").expect("should succeed");
        assert_eq!(p.depth(), 3);
    }

    #[test]
    fn test_roundtrip() {
        /* serialization roundtrip */
        let s = "/foo~0bar/baz~1qux";
        let p = JsonPointer::parse(s).expect("should succeed");
        assert_eq!(p.to_string_repr(), s);
    }

    #[test]
    fn test_leaf() {
        /* leaf returns last token */
        let p = JsonPointer::parse("/x/y/z").expect("should succeed");
        assert_eq!(pointer_leaf(&p), Some("z"));
    }

    #[test]
    fn test_parent() {
        /* parent drops last token */
        let p = JsonPointer::parse("/a/b").expect("should succeed");
        let parent = pointer_parent(&p).expect("should succeed");
        assert_eq!(parent.tokens(), &["a"]);
    }

    #[test]
    fn test_root_parent_none() {
        /* root has no parent */
        let p = JsonPointer::parse("").expect("should succeed");
        assert!(pointer_parent(&p).is_none());
    }
}
