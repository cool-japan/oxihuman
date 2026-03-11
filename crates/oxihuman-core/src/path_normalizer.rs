// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Path resolution and normalization utilities.

/// A normalized path wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPath(String);

impl NormalizedPath {
    pub fn new(path: &str) -> Self {
        NormalizedPath(normalize_path(path))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn join(&self, segment: &str) -> Self {
        let combined = format!(
            "{}/{}",
            self.0.trim_end_matches('/'),
            segment.trim_start_matches('/')
        );
        NormalizedPath::new(&combined)
    }

    pub fn parent(&self) -> Option<NormalizedPath> {
        let p = self.0.trim_end_matches('/');
        let idx = p.rfind('/')?;
        if idx == 0 {
            Some(NormalizedPath("/".to_string()))
        } else {
            Some(NormalizedPath(p[..idx].to_string()))
        }
    }

    pub fn file_name(&self) -> Option<&str> {
        self.0.trim_end_matches('/').rsplit('/').next()
    }
}

impl std::fmt::Display for NormalizedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Normalize a path string by resolving `.` and `..` components.
pub fn normalize_path(path: &str) -> String {
    let is_abs = path.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    let joined = parts.join("/");
    if is_abs {
        format!("/{}", joined)
    } else {
        joined
    }
}

/// Join two path segments.
pub fn join_paths(base: &str, segment: &str) -> String {
    if segment.starts_with('/') {
        normalize_path(segment)
    } else {
        normalize_path(&format!("{}/{}", base, segment))
    }
}

/// Return the file extension (without dot), if any.
pub fn file_extension(path: &str) -> Option<&str> {
    let name = path.rsplit('/').next()?;
    let dot = name.rfind('.')?;
    if dot == 0 {
        None
    } else {
        Some(&name[dot + 1..])
    }
}

/// Return true if the path is absolute.
pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

/// Make a relative path absolute by prepending `base`.
pub fn make_absolute(base: &str, path: &str) -> String {
    if is_absolute(path) {
        normalize_path(path)
    } else {
        join_paths(base, path)
    }
}

/// Strip prefix from a path.
pub fn strip_prefix<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    path.strip_prefix(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_dotdot() {
        assert_eq!(normalize_path("/a/b/../c"), "/a/c");
    }

    #[test]
    fn test_normalize_dot() {
        assert_eq!(normalize_path("/a/./b"), "/a/b");
    }

    #[test]
    fn test_normalize_double_slash() {
        assert_eq!(normalize_path("/a//b"), "/a/b");
    }

    #[test]
    fn test_join_paths_relative() {
        assert_eq!(join_paths("/a/b", "c/d"), "/a/b/c/d");
    }

    #[test]
    fn test_join_paths_abs_segment() {
        assert_eq!(join_paths("/a/b", "/c/d"), "/c/d");
    }

    #[test]
    fn test_file_extension() {
        assert_eq!(file_extension("/foo/bar.rs"), Some("rs"));
        assert_eq!(file_extension("/foo/bar"), None);
    }

    #[test]
    fn test_is_absolute() {
        assert!(is_absolute("/foo"));
        assert!(!is_absolute("foo"));
    }

    #[test]
    fn test_make_absolute() {
        assert_eq!(make_absolute("/base", "rel"), "/base/rel");
        assert_eq!(make_absolute("/base", "/abs"), "/abs");
    }

    #[test]
    fn test_normalized_path_join() {
        let p = NormalizedPath::new("/a/b");
        let q = p.join("c");
        assert_eq!(q.as_str(), "/a/b/c");
    }

    #[test]
    fn test_normalized_path_parent() {
        let p = NormalizedPath::new("/a/b/c");
        let parent = p.parent().unwrap();
        assert_eq!(parent.as_str(), "/a/b");
    }
}
