// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Symbolic link resolver stub.

use std::collections::HashMap;

/// A symbolic link record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symlink {
    pub path: String,
    pub target: String,
}

impl Symlink {
    pub fn new(path: &str, target: &str) -> Self {
        Symlink {
            path: path.to_string(),
            target: target.to_string(),
        }
    }
}

/// Symlink resolver — holds a virtual symlink table.
pub struct SymlinkResolver {
    table: HashMap<String, String>,
    max_depth: usize,
}

impl SymlinkResolver {
    pub fn new(max_depth: usize) -> Self {
        SymlinkResolver {
            table: HashMap::new(),
            max_depth,
        }
    }

    pub fn register(&mut self, link: Symlink) {
        self.table.insert(link.path, link.target);
    }

    pub fn resolve(&self, path: &str) -> Result<String, String> {
        let mut current = path.to_string();
        for _ in 0..self.max_depth {
            match self.table.get(&current) {
                Some(target) => current = target.clone(),
                None => return Ok(current),
            }
        }
        Err(format!("symlink loop detected at '{}'", path))
    }

    pub fn is_symlink(&self, path: &str) -> bool {
        self.table.contains_key(path)
    }

    pub fn count(&self) -> usize {
        self.table.len()
    }
}

impl Default for SymlinkResolver {
    fn default() -> Self {
        Self::new(40)
    }
}

/// Create a new resolver with default settings.
pub fn new_symlink_resolver() -> SymlinkResolver {
    SymlinkResolver::default()
}

/// Register multiple symlinks at once.
pub fn register_all(resolver: &mut SymlinkResolver, links: &[(&str, &str)]) {
    for (path, target) in links {
        resolver.register(Symlink::new(path, target));
    }
}

/// Resolve a batch of paths.
pub fn resolve_batch(resolver: &SymlinkResolver, paths: &[&str]) -> Vec<Result<String, String>> {
    paths.iter().map(|p| resolver.resolve(p)).collect()
}

/// Check for cycles: returns the paths involved if a loop is detected.
pub fn detect_cycle(resolver: &SymlinkResolver, start: &str) -> bool {
    resolver.resolve(start).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_non_symlink() {
        let r = new_symlink_resolver();
        assert_eq!(r.resolve("/real"), Ok("/real".to_string()));
    }

    #[test]
    fn test_resolve_single_hop() {
        let mut r = new_symlink_resolver();
        r.register(Symlink::new("/link", "/target"));
        assert_eq!(r.resolve("/link"), Ok("/target".to_string()));
    }

    #[test]
    fn test_resolve_chain() {
        let mut r = new_symlink_resolver();
        r.register(Symlink::new("/a", "/b"));
        r.register(Symlink::new("/b", "/c"));
        assert_eq!(r.resolve("/a"), Ok("/c".to_string()));
    }

    #[test]
    fn test_detect_cycle() {
        let mut r = SymlinkResolver::new(4);
        r.register(Symlink::new("/x", "/y"));
        r.register(Symlink::new("/y", "/x"));
        assert!(detect_cycle(&r, "/x"));
    }

    #[test]
    fn test_is_symlink() {
        let mut r = new_symlink_resolver();
        r.register(Symlink::new("/sym", "/real"));
        assert!(r.is_symlink("/sym"));
        assert!(!r.is_symlink("/real"));
    }

    #[test]
    fn test_count() {
        let mut r = new_symlink_resolver();
        register_all(&mut r, &[("/a", "/b"), ("/c", "/d")]);
        assert_eq!(r.count(), 2);
    }

    #[test]
    fn test_resolve_batch() {
        let mut r = new_symlink_resolver();
        r.register(Symlink::new("/link", "/real"));
        let results = resolve_batch(&r, &["/link", "/other"]);
        assert_eq!(results[0], Ok("/real".to_string()));
        assert_eq!(results[1], Ok("/other".to_string()));
    }

    #[test]
    fn test_register_all() {
        let mut r = new_symlink_resolver();
        register_all(&mut r, &[("/p", "/q")]);
        assert!(r.is_symlink("/p"));
    }

    #[test]
    fn test_default_max_depth() {
        let r = SymlinkResolver::default();
        assert_eq!(r.max_depth, 40);
    }

    #[test]
    fn test_resolve_ok_not_err_on_real_path() {
        let r = new_symlink_resolver();
        assert!(r.resolve("/real/path").is_ok());
    }
}
