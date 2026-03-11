#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternId(u32);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InternPool {
    to_id: HashMap<String, InternId>,
    to_str: Vec<String>,
}

#[allow(dead_code)]
pub fn new_intern_pool() -> InternPool {
    InternPool {
        to_id: HashMap::new(),
        to_str: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn intern_string(pool: &mut InternPool, s: &str) -> InternId {
    if let Some(&id) = pool.to_id.get(s) {
        return id;
    }
    let id = InternId(pool.to_str.len() as u32);
    pool.to_str.push(s.to_string());
    pool.to_id.insert(s.to_string(), id);
    id
}

#[allow(dead_code)]
pub fn resolve_intern(pool: &InternPool, id: InternId) -> Option<&str> {
    pool.to_str.get(id.0 as usize).map(|s| s.as_str())
}

#[allow(dead_code)]
pub fn intern_count(pool: &InternPool) -> usize {
    pool.to_str.len()
}

#[allow(dead_code)]
pub fn intern_contains(pool: &InternPool, s: &str) -> bool {
    pool.to_id.contains_key(s)
}

#[allow(dead_code)]
pub fn intern_id_of(pool: &InternPool, s: &str) -> Option<InternId> {
    pool.to_id.get(s).copied()
}

#[allow(dead_code)]
pub fn intern_all(pool: &InternPool) -> Vec<String> {
    pool.to_str.clone()
}

#[allow(dead_code)]
pub fn pool_clear_ip(pool: &mut InternPool) {
    pool.to_id.clear();
    pool.to_str.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool = new_intern_pool();
        assert_eq!(intern_count(&pool), 0);
    }

    #[test]
    fn test_intern_and_resolve() {
        let mut pool = new_intern_pool();
        let id = intern_string(&mut pool, "hello");
        assert_eq!(resolve_intern(&pool, id), Some("hello"));
    }

    #[test]
    fn test_dedup() {
        let mut pool = new_intern_pool();
        let id1 = intern_string(&mut pool, "abc");
        let id2 = intern_string(&mut pool, "abc");
        assert_eq!(id1, id2);
        assert_eq!(intern_count(&pool), 1);
    }

    #[test]
    fn test_contains() {
        let mut pool = new_intern_pool();
        intern_string(&mut pool, "test");
        assert!(intern_contains(&pool, "test"));
        assert!(!intern_contains(&pool, "nope"));
    }

    #[test]
    fn test_id_of() {
        let mut pool = new_intern_pool();
        let id = intern_string(&mut pool, "xyz");
        assert_eq!(intern_id_of(&pool, "xyz"), Some(id));
        assert_eq!(intern_id_of(&pool, "zzz"), None);
    }

    #[test]
    fn test_all() {
        let mut pool = new_intern_pool();
        intern_string(&mut pool, "a");
        intern_string(&mut pool, "b");
        assert_eq!(intern_all(&pool), vec!["a", "b"]);
    }

    #[test]
    fn test_clear() {
        let mut pool = new_intern_pool();
        intern_string(&mut pool, "data");
        pool_clear_ip(&mut pool);
        assert_eq!(intern_count(&pool), 0);
    }

    #[test]
    fn test_resolve_invalid() {
        let pool = new_intern_pool();
        assert_eq!(resolve_intern(&pool, InternId(99)), None);
    }

    #[test]
    fn test_multiple_strings() {
        let mut pool = new_intern_pool();
        intern_string(&mut pool, "alpha");
        intern_string(&mut pool, "beta");
        intern_string(&mut pool, "gamma");
        assert_eq!(intern_count(&pool), 3);
    }

    #[test]
    fn test_empty_string() {
        let mut pool = new_intern_pool();
        let id = intern_string(&mut pool, "");
        assert_eq!(resolve_intern(&pool, id), Some(""));
    }
}
