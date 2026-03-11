#![allow(dead_code)]

use std::collections::HashMap;

/// A flyweight is a shared, immutable value identified by index.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Flyweight {
    pub index: usize,
    pub value: String,
}

/// Pool of flyweight objects to share identical values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlyweightPool {
    values: Vec<String>,
    lookup: HashMap<String, usize>,
}

/// Creates a new empty flyweight pool.
#[allow(dead_code)]
pub fn new_flyweight_pool() -> FlyweightPool {
    FlyweightPool {
        values: Vec::new(),
        lookup: HashMap::new(),
    }
}

/// Gets or creates a flyweight for the given value.
#[allow(dead_code)]
pub fn get_or_create(pool: &mut FlyweightPool, value: &str) -> Flyweight {
    if let Some(&idx) = pool.lookup.get(value) {
        return Flyweight {
            index: idx,
            value: value.to_string(),
        };
    }
    let idx = pool.values.len();
    pool.values.push(value.to_string());
    pool.lookup.insert(value.to_string(), idx);
    Flyweight {
        index: idx,
        value: value.to_string(),
    }
}

/// Returns the number of unique values in the pool.
#[allow(dead_code)]
pub fn flyweight_count(pool: &FlyweightPool) -> usize {
    pool.values.len()
}

/// Returns the flyweight at the given index.
#[allow(dead_code)]
pub fn flyweight_at(pool: &FlyweightPool, index: usize) -> Option<&str> {
    pool.values.get(index).map(|s| s.as_str())
}

/// Checks if the pool contains a value.
#[allow(dead_code)]
pub fn pool_has(pool: &FlyweightPool, value: &str) -> bool {
    pool.lookup.contains_key(value)
}

/// Clears the pool.
#[allow(dead_code)]
pub fn pool_clear(pool: &mut FlyweightPool) {
    pool.values.clear();
    pool.lookup.clear();
}

/// Returns all values as a Vec.
#[allow(dead_code)]
pub fn pool_to_vec(pool: &FlyweightPool) -> Vec<String> {
    pool.values.clone()
}

/// Returns the estimated memory saved (duplicate references vs storing each copy).
#[allow(dead_code)]
pub fn pool_memory_saved(pool: &FlyweightPool, total_references: usize) -> usize {
    if total_references <= pool.values.len() {
        return 0;
    }
    let avg_len: usize = if pool.values.is_empty() {
        0
    } else {
        pool.values.iter().map(|s| s.len()).sum::<usize>() / pool.values.len()
    };
    (total_references - pool.values.len()) * avg_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool = new_flyweight_pool();
        assert_eq!(flyweight_count(&pool), 0);
    }

    #[test]
    fn test_get_or_create() {
        let mut pool = new_flyweight_pool();
        let fw = get_or_create(&mut pool, "hello");
        assert_eq!(fw.value, "hello");
        assert_eq!(fw.index, 0);
    }

    #[test]
    fn test_deduplication() {
        let mut pool = new_flyweight_pool();
        let a = get_or_create(&mut pool, "same");
        let b = get_or_create(&mut pool, "same");
        assert_eq!(a.index, b.index);
        assert_eq!(flyweight_count(&pool), 1);
    }

    #[test]
    fn test_flyweight_at() {
        let mut pool = new_flyweight_pool();
        get_or_create(&mut pool, "test");
        assert_eq!(flyweight_at(&pool, 0), Some("test"));
        assert_eq!(flyweight_at(&pool, 99), None);
    }

    #[test]
    fn test_pool_has() {
        let mut pool = new_flyweight_pool();
        get_or_create(&mut pool, "x");
        assert!(pool_has(&pool, "x"));
        assert!(!pool_has(&pool, "y"));
    }

    #[test]
    fn test_pool_clear() {
        let mut pool = new_flyweight_pool();
        get_or_create(&mut pool, "a");
        pool_clear(&mut pool);
        assert_eq!(flyweight_count(&pool), 0);
    }

    #[test]
    fn test_pool_to_vec() {
        let mut pool = new_flyweight_pool();
        get_or_create(&mut pool, "a");
        get_or_create(&mut pool, "b");
        let v = pool_to_vec(&pool);
        assert_eq!(v, vec!["a", "b"]);
    }

    #[test]
    fn test_pool_memory_saved() {
        let mut pool = new_flyweight_pool();
        get_or_create(&mut pool, "hello");
        let saved = pool_memory_saved(&pool, 10);
        assert!(saved > 0);
    }

    #[test]
    fn test_multiple_values() {
        let mut pool = new_flyweight_pool();
        get_or_create(&mut pool, "a");
        get_or_create(&mut pool, "b");
        get_or_create(&mut pool, "c");
        assert_eq!(flyweight_count(&pool), 3);
    }

    #[test]
    fn test_memory_saved_no_excess() {
        let pool = new_flyweight_pool();
        assert_eq!(pool_memory_saved(&pool, 0), 0);
    }
}
