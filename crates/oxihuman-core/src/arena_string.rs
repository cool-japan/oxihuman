#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArenaString {
    id: usize,
    offset: usize,
    len: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StringArena {
    buffer: Vec<u8>,
    index: HashMap<usize, (usize, usize)>,
    next_id: usize,
}

#[allow(dead_code)]
pub fn new_string_arena() -> StringArena {
    StringArena {
        buffer: Vec::new(),
        index: HashMap::new(),
        next_id: 0,
    }
}

#[allow(dead_code)]
pub fn arena_alloc_str(arena: &mut StringArena, s: &str) -> ArenaString {
    let offset = arena.buffer.len();
    arena.buffer.extend_from_slice(s.as_bytes());
    let id = arena.next_id;
    arena.index.insert(id, (offset, s.len()));
    arena.next_id += 1;
    ArenaString {
        id,
        offset,
        len: s.len(),
    }
}

#[allow(dead_code)]
pub fn arena_get_str<'a>(arena: &'a StringArena, handle: &ArenaString) -> Option<&'a str> {
    arena.index.get(&handle.id).and_then(|&(off, len)| {
        std::str::from_utf8(&arena.buffer[off..off + len]).ok()
    })
}

#[allow(dead_code)]
pub fn arena_str_count(arena: &StringArena) -> usize {
    arena.index.len()
}

#[allow(dead_code)]
pub fn arena_total_bytes(arena: &StringArena) -> usize {
    arena.buffer.len()
}

#[allow(dead_code)]
pub fn arena_clear(arena: &mut StringArena) {
    arena.buffer.clear();
    arena.index.clear();
    arena.next_id = 0;
}

#[allow(dead_code)]
pub fn arena_contains(arena: &StringArena, s: &str) -> bool {
    let bytes = s.as_bytes();
    arena.index.values().any(|&(off, len)| {
        len == bytes.len() && arena.buffer[off..off + len] == *bytes
    })
}

#[allow(dead_code)]
pub fn arena_dedup(arena: &StringArena) -> usize {
    let mut seen = std::collections::HashSet::new();
    let mut dupes = 0usize;
    for &(off, len) in arena.index.values() {
        let slice = &arena.buffer[off..off + len];
        if !seen.insert(slice.to_vec()) {
            dupes += 1;
        }
    }
    dupes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_arena() {
        let arena = new_string_arena();
        assert_eq!(arena_str_count(&arena), 0);
    }

    #[test]
    fn test_alloc_and_get() {
        let mut arena = new_string_arena();
        let h = arena_alloc_str(&mut arena, "hello");
        assert_eq!(arena_get_str(&arena, &h), Some("hello"));
    }

    #[test]
    fn test_multiple_allocs() {
        let mut arena = new_string_arena();
        let h1 = arena_alloc_str(&mut arena, "foo");
        let h2 = arena_alloc_str(&mut arena, "bar");
        assert_eq!(arena_get_str(&arena, &h1), Some("foo"));
        assert_eq!(arena_get_str(&arena, &h2), Some("bar"));
    }

    #[test]
    fn test_str_count() {
        let mut arena = new_string_arena();
        arena_alloc_str(&mut arena, "a");
        arena_alloc_str(&mut arena, "b");
        assert_eq!(arena_str_count(&arena), 2);
    }

    #[test]
    fn test_total_bytes() {
        let mut arena = new_string_arena();
        arena_alloc_str(&mut arena, "abc");
        arena_alloc_str(&mut arena, "de");
        assert_eq!(arena_total_bytes(&arena), 5);
    }

    #[test]
    fn test_clear() {
        let mut arena = new_string_arena();
        arena_alloc_str(&mut arena, "xyz");
        arena_clear(&mut arena);
        assert_eq!(arena_str_count(&arena), 0);
        assert_eq!(arena_total_bytes(&arena), 0);
    }

    #[test]
    fn test_contains() {
        let mut arena = new_string_arena();
        arena_alloc_str(&mut arena, "test");
        assert!(arena_contains(&arena, "test"));
        assert!(!arena_contains(&arena, "nope"));
    }

    #[test]
    fn test_dedup_none() {
        let mut arena = new_string_arena();
        arena_alloc_str(&mut arena, "a");
        arena_alloc_str(&mut arena, "b");
        assert_eq!(arena_dedup(&arena), 0);
    }

    #[test]
    fn test_dedup_some() {
        let mut arena = new_string_arena();
        arena_alloc_str(&mut arena, "a");
        arena_alloc_str(&mut arena, "a");
        assert_eq!(arena_dedup(&arena), 1);
    }

    #[test]
    fn test_empty_string() {
        let mut arena = new_string_arena();
        let h = arena_alloc_str(&mut arena, "");
        assert_eq!(arena_get_str(&arena, &h), Some(""));
    }
}
