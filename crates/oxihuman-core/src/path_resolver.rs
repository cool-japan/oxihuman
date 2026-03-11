#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Path search and resolution utility.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PathResolver {
    search_paths: Vec<String>,
}

#[allow(dead_code)]
pub fn new_path_resolver() -> PathResolver {
    PathResolver {
        search_paths: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn resolve_path(resolver: &PathResolver, name: &str) -> Option<String> {
    if let Some(sp) = resolver.search_paths.first() {
        let candidate = format!("{}/{}", sp, name);
        return Some(candidate);
    }
    None
}

#[allow(dead_code)]
pub fn add_search_path(resolver: &mut PathResolver, path: &str) {
    resolver.search_paths.push(path.to_string());
}

#[allow(dead_code)]
pub fn search_path_count(resolver: &PathResolver) -> usize {
    resolver.search_paths.len()
}

#[allow(dead_code)]
pub fn path_exists_stub(_path: &str) -> bool {
    false
}

#[allow(dead_code)]
pub fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    let result = parts.join("/");
    if path.starts_with('/') {
        format!("/{}", result)
    } else {
        result
    }
}

#[allow(dead_code)]
pub fn resolver_to_json(resolver: &PathResolver) -> String {
    format!(
        r#"{{"search_paths":{}}}"#,
        resolver.search_paths.len()
    )
}

#[allow(dead_code)]
pub fn clear_search_paths(resolver: &mut PathResolver) {
    resolver.search_paths.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_resolver() {
        let r = new_path_resolver();
        assert_eq!(search_path_count(&r), 0);
    }

    #[test]
    fn test_add_search_path() {
        let mut r = new_path_resolver();
        add_search_path(&mut r, "/usr/lib");
        assert_eq!(search_path_count(&r), 1);
    }

    #[test]
    fn test_resolve_path() {
        let mut r = new_path_resolver();
        add_search_path(&mut r, "/usr/lib");
        let resolved = resolve_path(&r, "foo.so");
        assert_eq!(resolved, Some("/usr/lib/foo.so".to_string()));
    }

    #[test]
    fn test_resolve_empty() {
        let r = new_path_resolver();
        assert_eq!(resolve_path(&r, "foo"), None);
    }

    #[test]
    fn test_normalize_simple() {
        assert_eq!(normalize_path("/a/b/c"), "/a/b/c");
    }

    #[test]
    fn test_normalize_dotdot() {
        assert_eq!(normalize_path("/a/b/../c"), "/a/c");
    }

    #[test]
    fn test_normalize_dot() {
        assert_eq!(normalize_path("/a/./b"), "/a/b");
    }

    #[test]
    fn test_path_exists_stub() {
        assert!(!path_exists_stub("/nonexistent"));
    }

    #[test]
    fn test_clear_search_paths() {
        let mut r = new_path_resolver();
        add_search_path(&mut r, "/a");
        clear_search_paths(&mut r);
        assert_eq!(search_path_count(&r), 0);
    }

    #[test]
    fn test_resolver_to_json() {
        let r = new_path_resolver();
        let json = resolver_to_json(&r);
        assert!(json.contains("\"search_paths\":0"));
    }
}
