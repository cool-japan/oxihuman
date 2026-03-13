// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Normalize a path: resolve '.' and '..' segments.
pub fn resolve_path_normalize(path: &str) -> String {
    let is_abs = path.starts_with('/');
    let mut stack: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if !stack.is_empty() && stack.last() != Some(&"..") {
                    stack.pop();
                } else if !is_abs {
                    stack.push("..");
                }
            }
            s => stack.push(s),
        }
    }
    let result = stack.join("/");
    if is_abs {
        format!("/{}", result)
    } else if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
}

/// Join `base` and `relative` with '/', then normalize `.` and `..` segments.
pub fn resolve_path(base: &str, relative: &str) -> String {
    let joined = if relative.starts_with('/') {
        relative.to_string()
    } else {
        format!("{}/{}", base.trim_end_matches('/'), relative)
    };
    resolve_path_normalize(&joined)
}

/// Return the filename component of the path (after the last '/').
pub fn path_basename(path: &str) -> &str {
    match path.rfind('/') {
        Some(idx) => &path[idx + 1..],
        None => path,
    }
}

/// Return everything before the last '/' in `path`.
pub fn path_dirname(path: &str) -> &str {
    match path.rfind('/') {
        Some(0) => "/",
        Some(idx) => &path[..idx],
        None => ".",
    }
}

/// Return the extension (after the last '.') if present.
pub fn path_ext(path: &str) -> Option<&str> {
    let base = path_basename(path);
    match base.rfind('.') {
        Some(idx) if idx > 0 => Some(&base[idx + 1..]),
        _ => None,
    }
}

/// True if path starts with '/'.
pub fn path_is_abs(path: &str) -> bool {
    path.starts_with('/')
}

/// Join two path segments with '/'.
pub fn path_join_parts(a: &str, b: &str) -> String {
    format!("{}/{}", a.trim_end_matches('/'), b.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basename() {
        /* basename returns filename portion */
        assert_eq!(path_basename("/foo/bar/baz.txt"), "baz.txt");
        assert_eq!(path_basename("file.rs"), "file.rs");
    }

    #[test]
    fn test_dirname() {
        /* dirname returns directory portion */
        assert_eq!(path_dirname("/foo/bar/baz.txt"), "/foo/bar");
        assert_eq!(path_dirname("file.rs"), ".");
    }

    #[test]
    fn test_extension() {
        /* extension returns text after last dot */
        assert_eq!(path_ext("foo.rs"), Some("rs"));
        assert_eq!(path_ext("foo"), None);
    }

    #[test]
    fn test_is_abs() {
        /* is_abs detects leading slash */
        assert!(path_is_abs("/foo"));
        assert!(!path_is_abs("foo"));
    }

    #[test]
    fn test_path_join_parts() {
        /* join appends with slash */
        assert_eq!(path_join_parts("/foo", "bar"), "/foo/bar");
        assert_eq!(path_join_parts("/foo/", "/bar"), "/foo/bar");
    }

    #[test]
    fn test_normalize_dotdot() {
        /* normalize resolves parent refs */
        assert_eq!(resolve_path_normalize("/foo/bar/../baz"), "/foo/baz");
    }

    #[test]
    fn test_resolve_path() {
        /* resolve_path joins and normalizes */
        assert_eq!(
            resolve_path("/home/user", "docs/../file.txt"),
            "/home/user/file.txt"
        );
    }

    #[test]
    fn test_normalize_dot() {
        /* normalize removes current-dir refs */
        assert_eq!(resolve_path_normalize("/foo/./bar"), "/foo/bar");
    }
}
