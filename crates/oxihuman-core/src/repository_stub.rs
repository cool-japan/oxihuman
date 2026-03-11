// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct StringRepo {
    pub store: std::collections::HashMap<String, String>,
}

pub fn new_string_repo() -> StringRepo {
    StringRepo {
        store: std::collections::HashMap::new(),
    }
}

pub fn repo_save(r: &mut StringRepo, id: &str, data: &str) {
    r.store.insert(id.to_string(), data.to_string());
}

pub fn repo_find<'a>(r: &'a StringRepo, id: &str) -> Option<&'a str> {
    r.store.get(id).map(|s| s.as_str())
}

pub fn repo_delete(r: &mut StringRepo, id: &str) -> bool {
    r.store.remove(id).is_some()
}

pub fn repo_count(r: &StringRepo) -> usize {
    r.store.len()
}

pub fn repo_all_ids(r: &StringRepo) -> Vec<String> {
    let mut ids: Vec<String> = r.store.keys().cloned().collect();
    ids.sort();
    ids
}

pub fn repo_exists(r: &StringRepo, id: &str) -> bool {
    r.store.contains_key(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_find() {
        /* save and find by id */
        let mut r = new_string_repo();
        repo_save(&mut r, "1", "Alice");
        assert_eq!(repo_find(&r, "1"), Some("Alice"));
    }

    #[test]
    fn test_find_missing() {
        /* missing id returns None */
        let r = new_string_repo();
        assert_eq!(repo_find(&r, "99"), None);
    }

    #[test]
    fn test_delete() {
        /* delete removes entry */
        let mut r = new_string_repo();
        repo_save(&mut r, "1", "data");
        assert!(repo_delete(&mut r, "1"));
        assert!(!repo_exists(&r, "1"));
    }

    #[test]
    fn test_delete_missing() {
        /* delete of missing key returns false */
        let mut r = new_string_repo();
        assert!(!repo_delete(&mut r, "x"));
    }

    #[test]
    fn test_count() {
        /* count tracks number of entries */
        let mut r = new_string_repo();
        repo_save(&mut r, "a", "1");
        repo_save(&mut r, "b", "2");
        assert_eq!(repo_count(&r), 2);
    }

    #[test]
    fn test_all_ids() {
        /* all_ids returns sorted ids */
        let mut r = new_string_repo();
        repo_save(&mut r, "b", "2");
        repo_save(&mut r, "a", "1");
        let ids = repo_all_ids(&r);
        assert_eq!(ids, vec!["a", "b"]);
    }

    #[test]
    fn test_exists() {
        /* exists checks presence */
        let mut r = new_string_repo();
        repo_save(&mut r, "x", "v");
        assert!(repo_exists(&r, "x"));
        assert!(!repo_exists(&r, "y"));
    }
}
