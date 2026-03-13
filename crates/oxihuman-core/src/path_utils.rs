// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! File path utility functions (string-based, no std::path dependency needed).

#![allow(dead_code)]

/// Join a base path with a name component, inserting `/` if needed.
#[allow(dead_code)]
pub fn path_join(base: &str, name: &str) -> String {
    if base.is_empty() {
        return name.to_string();
    }
    if name.is_empty() {
        return base.to_string();
    }
    if base.ends_with('/') {
        format!("{base}{name}")
    } else {
        format!("{base}/{name}")
    }
}

/// Return the file extension of `path` (the part after the last `.`), or None.
#[allow(dead_code)]
pub fn path_extension(path: &str) -> Option<&str> {
    let filename = path.rsplit('/').next()?;
    let dot_pos = filename.rfind('.')?;
    if dot_pos == 0 {
        return None; // hidden file like ".gitignore"
    }
    Some(&filename[dot_pos + 1..])
}

/// Return the file stem of `path` (filename without extension), or None.
#[allow(dead_code)]
pub fn path_stem(path: &str) -> Option<&str> {
    let filename = path.rsplit('/').next()?;
    if filename.is_empty() {
        return None;
    }
    match filename.rfind('.') {
        Some(dot_pos) if dot_pos > 0 => Some(&filename[..dot_pos]),
        _ => Some(filename),
    }
}

/// Return the parent directory of `path`, or None if there is none.
#[allow(dead_code)]
pub fn path_parent(path: &str) -> Option<&str> {
    let trimmed = path.trim_end_matches('/');
    let pos = trimmed.rfind('/')?;
    if pos == 0 {
        return Some("/");
    }
    Some(&trimmed[..pos])
}

/// Return true if `path` is absolute (starts with `/`).
#[allow(dead_code)]
pub fn path_is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

/// Normalize a path by collapsing `.` and `..` components.
#[allow(dead_code)]
pub fn path_normalize(path: &str) -> String {
    let is_abs = path.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() && parts.last() != Some(&"..") {
                    parts.pop();
                } else if !is_abs {
                    parts.push("..");
                }
            }
            c => parts.push(c),
        }
    }
    let joined = parts.join("/");
    if is_abs {
        format!("/{joined}")
    } else if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_basic() {
        assert_eq!(path_join("/foo", "bar"), "/foo/bar");
    }

    #[test]
    fn join_trailing_slash() {
        assert_eq!(path_join("/foo/", "bar"), "/foo/bar");
    }

    #[test]
    fn join_empty_base() {
        assert_eq!(path_join("", "bar"), "bar");
    }

    #[test]
    fn extension_basic() {
        assert_eq!(path_extension("/foo/bar.txt"), Some("txt"));
    }

    #[test]
    fn extension_no_dot() {
        assert!(path_extension("/foo/bar").is_none());
    }

    #[test]
    fn extension_hidden_file() {
        assert!(path_extension("/foo/.gitignore").is_none());
    }

    #[test]
    fn stem_basic() {
        assert_eq!(path_stem("/foo/bar.txt"), Some("bar"));
    }

    #[test]
    fn stem_no_extension() {
        assert_eq!(path_stem("/foo/bar"), Some("bar"));
    }

    #[test]
    fn parent_basic() {
        assert_eq!(path_parent("/foo/bar/baz"), Some("/foo/bar"));
    }

    #[test]
    fn parent_root() {
        assert_eq!(path_parent("/foo"), Some("/"));
    }

    #[test]
    fn is_absolute_true() {
        assert!(path_is_absolute("/foo/bar"));
    }

    #[test]
    fn is_absolute_false() {
        assert!(!path_is_absolute("foo/bar"));
    }

    #[test]
    fn normalize_dotdot() {
        assert_eq!(path_normalize("/foo/bar/../baz"), "/foo/baz");
    }

    #[test]
    fn normalize_dot() {
        assert_eq!(path_normalize("/foo/./bar"), "/foo/bar");
    }

    #[test]
    fn normalize_relative() {
        assert_eq!(path_normalize("a/b/../c"), "a/c");
    }
}
