// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dense index-keyed map (usize → T).

#![allow(dead_code)]

/// Configuration for an IndexMap.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndexMapConfig {
    pub initial_capacity: usize,
}

/// A dense map keyed by usize indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndexMap<T> {
    slots: Vec<Option<T>>,
    len: usize,
}

/// Create a new IndexMap.
#[allow(dead_code)]
pub fn new_index_map<T>() -> IndexMap<T> {
    IndexMap {
        slots: Vec::new(),
        len: 0,
    }
}

/// Insert a value at the given index. Returns the old value if present.
#[allow(dead_code)]
pub fn index_map_insert<T>(map: &mut IndexMap<T>, key: usize, value: T) -> Option<T> {
    if key >= map.slots.len() {
        map.slots.resize_with(key + 1, || None);
    }
    let old = map.slots[key].take();
    if old.is_none() {
        map.len += 1;
    }
    map.slots[key] = Some(value);
    old
}

/// Get a reference to the value at the given index.
#[allow(dead_code)]
pub fn index_map_get<T>(map: &IndexMap<T>, key: usize) -> Option<&T> {
    map.slots.get(key).and_then(|s| s.as_ref())
}

/// Get a mutable reference to the value at the given index.
#[allow(dead_code)]
pub fn index_map_get_mut<T>(map: &mut IndexMap<T>, key: usize) -> Option<&mut T> {
    map.slots.get_mut(key).and_then(|s| s.as_mut())
}

/// Remove the value at the given index, returning it.
#[allow(dead_code)]
pub fn index_map_remove<T>(map: &mut IndexMap<T>, key: usize) -> Option<T> {
    let val = map.slots.get_mut(key).and_then(|s| s.take());
    if val.is_some() {
        map.len -= 1;
    }
    val
}

/// Check whether the given index has a value.
#[allow(dead_code)]
pub fn index_map_contains<T>(map: &IndexMap<T>, key: usize) -> bool {
    map.slots.get(key).map(|s| s.is_some()).unwrap_or(false)
}

/// Return the number of present entries.
#[allow(dead_code)]
pub fn index_map_len<T>(map: &IndexMap<T>) -> usize {
    map.len
}

/// Check whether the map is empty.
#[allow(dead_code)]
pub fn index_map_is_empty<T>(map: &IndexMap<T>) -> bool {
    map.len == 0
}

/// Clear all entries.
#[allow(dead_code)]
pub fn index_map_clear<T>(map: &mut IndexMap<T>) {
    for s in map.slots.iter_mut() {
        *s = None;
    }
    map.len = 0;
}

/// Return the slot capacity (not the number of entries).
#[allow(dead_code)]
pub fn index_map_capacity<T>(map: &IndexMap<T>) -> usize {
    map.slots.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let map: IndexMap<i32> = new_index_map();
        assert!(index_map_is_empty(&map));
        assert_eq!(index_map_len(&map), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut map = new_index_map();
        index_map_insert(&mut map, 3, "hello");
        assert_eq!(index_map_get(&map, 3), Some(&"hello"));
    }

    #[test]
    fn test_insert_overwrites() {
        let mut map = new_index_map();
        let old = index_map_insert(&mut map, 0, 42i32);
        assert!(old.is_none());
        let old2 = index_map_insert(&mut map, 0, 99i32);
        assert_eq!(old2, Some(42));
        assert_eq!(index_map_len(&map), 1);
    }

    #[test]
    fn test_remove() {
        let mut map = new_index_map();
        index_map_insert(&mut map, 5, 100u32);
        let removed = index_map_remove(&mut map, 5);
        assert_eq!(removed, Some(100));
        assert!(!index_map_contains(&map, 5));
        assert_eq!(index_map_len(&map), 0);
    }

    #[test]
    fn test_contains() {
        let mut map = new_index_map();
        index_map_insert(&mut map, 2, "x");
        assert!(index_map_contains(&map, 2));
        assert!(!index_map_contains(&map, 3));
    }

    #[test]
    fn test_get_mut() {
        let mut map = new_index_map();
        index_map_insert(&mut map, 1, 10i32);
        if let Some(v) = index_map_get_mut(&mut map, 1) {
            *v = 20;
        }
        assert_eq!(index_map_get(&map, 1), Some(&20));
    }

    #[test]
    fn test_clear() {
        let mut map = new_index_map();
        index_map_insert(&mut map, 0, 1);
        index_map_insert(&mut map, 1, 2);
        index_map_clear(&mut map);
        assert_eq!(index_map_len(&map), 0);
        assert!(!index_map_contains(&map, 0));
    }

    #[test]
    fn test_capacity_grows() {
        let mut map: IndexMap<u8> = new_index_map();
        index_map_insert(&mut map, 100, 42);
        assert!(index_map_capacity(&map) >= 101);
    }

    #[test]
    fn test_get_missing_returns_none() {
        let map: IndexMap<f32> = new_index_map();
        assert!(index_map_get(&map, 5).is_none());
    }
}
