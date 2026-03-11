//! String interning/pooling for memory efficiency.

use std::collections::HashMap;

/// Opaque handle to an interned string.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StringId(pub u32);

/// Pool that deduplicates and interns strings.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct StringPool {
    /// Map from string content to its id.
    lookup: HashMap<String, StringId>,
    /// Reverse map from id to string content.
    strings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a new, empty string pool.
#[allow(dead_code)]
pub fn new_string_pool() -> StringPool {
    StringPool {
        lookup: HashMap::new(),
        strings: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Core operations
// ---------------------------------------------------------------------------

/// Intern a string, returning its `StringId`. If the string is already interned,
/// the existing id is returned without duplication.
#[allow(dead_code)]
pub fn intern(pool: &mut StringPool, s: &str) -> StringId {
    if let Some(&id) = pool.lookup.get(s) {
        return id;
    }
    let id = StringId(pool.strings.len() as u32);
    pool.strings.push(s.to_string());
    pool.lookup.insert(s.to_string(), id);
    id
}

/// Resolve a `StringId` back to its string content.
/// Returns `None` if the id is invalid.
#[allow(dead_code)]
pub fn resolve(pool: &StringPool, id: StringId) -> Option<&str> {
    pool.strings.get(id.0 as usize).map(|s| s.as_str())
}

/// Check whether a string is already interned.
#[allow(dead_code)]
pub fn contains(pool: &StringPool, s: &str) -> bool {
    pool.lookup.contains_key(s)
}

/// Return the number of unique strings in the pool.
#[allow(dead_code)]
pub fn pool_size(pool: &StringPool) -> usize {
    pool.strings.len()
}

/// Return the total number of bytes stored across all interned strings.
#[allow(dead_code)]
pub fn total_bytes(pool: &StringPool) -> usize {
    pool.strings.iter().map(|s| s.len()).sum()
}

/// Intern multiple strings at once, returning their ids.
#[allow(dead_code)]
pub fn intern_many(pool: &mut StringPool, strings: &[&str]) -> Vec<StringId> {
    strings.iter().map(|s| intern(pool, s)).collect()
}

/// Remove strings that are not in the `keep` set.
/// Returns the number of strings removed.
///
/// Note: This invalidates all existing `StringId` handles. The pool is
/// rebuilt with new ids.
#[allow(dead_code)]
pub fn remove_unused(pool: &mut StringPool, keep: &[StringId]) -> usize {
    let keep_set: std::collections::HashSet<u32> = keep.iter().map(|id| id.0).collect();
    let original_count = pool.strings.len();

    let retained: Vec<String> = pool
        .strings
        .iter()
        .enumerate()
        .filter(|(i, _)| keep_set.contains(&(*i as u32)))
        .map(|(_, s)| s.clone())
        .collect();

    pool.strings = retained;
    pool.lookup.clear();
    for (i, s) in pool.strings.iter().enumerate() {
        pool.lookup.insert(s.clone(), StringId(i as u32));
    }

    original_count - pool.strings.len()
}

/// Check whether a `StringId` is valid in the current pool.
#[allow(dead_code)]
pub fn string_id_valid(pool: &StringPool, id: StringId) -> bool {
    (id.0 as usize) < pool.strings.len()
}

/// Return pool statistics as a JSON string.
#[allow(dead_code)]
pub fn pool_stats_json(pool: &StringPool) -> String {
    let count = pool_size(pool);
    let bytes = total_bytes(pool);
    let avg = if count > 0 {
        bytes as f64 / count as f64
    } else {
        0.0
    };
    format!(
        "{{\"unique_strings\":{},\"total_bytes\":{},\"average_length\":{:.2}}}",
        count, bytes, avg
    )
}

/// Remove all strings from the pool.
#[allow(dead_code)]
pub fn clear_pool(pool: &mut StringPool) {
    pool.strings.clear();
    pool.lookup.clear();
}

/// Merge another pool's strings into this pool.
/// Returns the number of newly added strings.
#[allow(dead_code)]
pub fn merge_pools(dst: &mut StringPool, src: &StringPool) -> usize {
    let before = pool_size(dst);
    for s in &src.strings {
        intern(dst, s);
    }
    pool_size(dst) - before
}

/// Find all interned strings that start with the given prefix.
/// Returns their `StringId`s.
#[allow(dead_code)]
pub fn find_by_prefix(pool: &StringPool, prefix: &str) -> Vec<StringId> {
    pool.strings
        .iter()
        .enumerate()
        .filter(|(_, s)| s.starts_with(prefix))
        .map(|(i, _)| StringId(i as u32))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_string_pool_empty() {
        let pool = new_string_pool();
        assert_eq!(pool_size(&pool), 0);
        assert_eq!(total_bytes(&pool), 0);
    }

    #[test]
    fn test_intern_returns_id() {
        let mut pool = new_string_pool();
        let id = intern(&mut pool, "hello");
        assert_eq!(id.0, 0);
    }

    #[test]
    fn test_intern_deduplicates() {
        let mut pool = new_string_pool();
        let id1 = intern(&mut pool, "hello");
        let id2 = intern(&mut pool, "hello");
        assert_eq!(id1, id2);
        assert_eq!(pool_size(&pool), 1);
    }

    #[test]
    fn test_resolve_valid() {
        let mut pool = new_string_pool();
        let id = intern(&mut pool, "world");
        assert_eq!(resolve(&pool, id), Some("world"));
    }

    #[test]
    fn test_resolve_invalid() {
        let pool = new_string_pool();
        assert_eq!(resolve(&pool, StringId(999)), None);
    }

    #[test]
    fn test_contains() {
        let mut pool = new_string_pool();
        intern(&mut pool, "abc");
        assert!(contains(&pool, "abc"));
        assert!(!contains(&pool, "xyz"));
    }

    #[test]
    fn test_pool_size() {
        let mut pool = new_string_pool();
        intern(&mut pool, "a");
        intern(&mut pool, "b");
        intern(&mut pool, "c");
        assert_eq!(pool_size(&pool), 3);
    }

    #[test]
    fn test_total_bytes() {
        let mut pool = new_string_pool();
        intern(&mut pool, "ab"); // 2 bytes
        intern(&mut pool, "cde"); // 3 bytes
        assert_eq!(total_bytes(&pool), 5);
    }

    #[test]
    fn test_intern_many() {
        let mut pool = new_string_pool();
        let ids = intern_many(&mut pool, &["x", "y", "z"]);
        assert_eq!(ids.len(), 3);
        assert_eq!(pool_size(&pool), 3);
    }

    #[test]
    fn test_remove_unused() {
        let mut pool = new_string_pool();
        let id0 = intern(&mut pool, "keep");
        intern(&mut pool, "remove");
        let removed = remove_unused(&mut pool, &[id0]);
        assert_eq!(removed, 1);
        assert_eq!(pool_size(&pool), 1);
        assert!(contains(&pool, "keep"));
    }

    #[test]
    fn test_string_id_valid() {
        let mut pool = new_string_pool();
        let id = intern(&mut pool, "test");
        assert!(string_id_valid(&pool, id));
        assert!(!string_id_valid(&pool, StringId(100)));
    }

    #[test]
    fn test_pool_stats_json() {
        let mut pool = new_string_pool();
        intern(&mut pool, "abc");
        let json = pool_stats_json(&pool);
        assert!(json.contains("\"unique_strings\":1"));
        assert!(json.contains("\"total_bytes\":3"));
    }

    #[test]
    fn test_clear_pool() {
        let mut pool = new_string_pool();
        intern(&mut pool, "a");
        intern(&mut pool, "b");
        clear_pool(&mut pool);
        assert_eq!(pool_size(&pool), 0);
    }

    #[test]
    fn test_merge_pools() {
        let mut pool1 = new_string_pool();
        intern(&mut pool1, "a");
        let mut pool2 = new_string_pool();
        intern(&mut pool2, "b");
        intern(&mut pool2, "c");
        let added = merge_pools(&mut pool1, &pool2);
        assert_eq!(added, 2);
        assert_eq!(pool_size(&pool1), 3);
    }

    #[test]
    fn test_merge_pools_no_duplicates() {
        let mut pool1 = new_string_pool();
        intern(&mut pool1, "shared");
        let mut pool2 = new_string_pool();
        intern(&mut pool2, "shared");
        intern(&mut pool2, "new");
        let added = merge_pools(&mut pool1, &pool2);
        assert_eq!(added, 1); // only "new" is added
    }

    #[test]
    fn test_find_by_prefix() {
        let mut pool = new_string_pool();
        intern(&mut pool, "morph_face");
        intern(&mut pool, "morph_body");
        intern(&mut pool, "texture_skin");
        let results = find_by_prefix(&pool, "morph_");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_by_prefix_empty() {
        let mut pool = new_string_pool();
        intern(&mut pool, "abc");
        let results = find_by_prefix(&pool, "xyz");
        assert!(results.is_empty());
    }
}
