// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Small vector with inline storage for up to 8 f32 elements, heap-fallback for larger.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmallVec8 {
    pub inline: [f32; 8],
    pub len: usize,
    pub heap: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmallVec8Config {
    pub initial_capacity: usize,
}

#[allow(dead_code)]
pub fn default_small_vec8_config() -> SmallVec8Config {
    SmallVec8Config { initial_capacity: 8 }
}

#[allow(dead_code)]
pub fn new_small_vec8() -> SmallVec8 {
    SmallVec8 { inline: [0.0; 8], len: 0, heap: Vec::new() }
}

#[allow(dead_code)]
pub fn sv8_push(v: &mut SmallVec8, val: f32) {
    if v.len < 8 {
        v.inline[v.len] = val;
    } else {
        if v.heap.is_empty() {
            // Copy inline into heap
            for i in 0..8 {
                v.heap.push(v.inline[i]);
            }
        }
        v.heap.push(val);
    }
    v.len += 1;
}

#[allow(dead_code)]
pub fn sv8_pop(v: &mut SmallVec8) -> Option<f32> {
    if v.len == 0 {
        return None;
    }
    v.len -= 1;
    if v.len >= 8 {
        v.heap.pop()
    } else {
        Some(v.inline[v.len])
    }
}

#[allow(dead_code)]
pub fn sv8_get(v: &SmallVec8, i: usize) -> Option<f32> {
    if i >= v.len {
        return None;
    }
    if i < 8 {
        Some(v.inline[i])
    } else {
        v.heap.get(i).copied()
    }
}

#[allow(dead_code)]
pub fn sv8_len(v: &SmallVec8) -> usize {
    v.len
}

#[allow(dead_code)]
pub fn sv8_is_empty(v: &SmallVec8) -> bool {
    v.len == 0
}

#[allow(dead_code)]
pub fn sv8_clear(v: &mut SmallVec8) {
    v.len = 0;
    v.heap.clear();
}

#[allow(dead_code)]
pub fn sv8_is_inline(v: &SmallVec8) -> bool {
    v.len <= 8
}

#[allow(dead_code)]
pub fn sv8_to_vec(v: &SmallVec8) -> Vec<f32> {
    let mut out = Vec::with_capacity(v.len);
    for i in 0..v.len {
        if i < 8 {
            out.push(v.inline[i]);
        } else {
            out.push(v.heap[i]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let v = new_small_vec8();
        assert!(sv8_is_empty(&v));
        assert_eq!(sv8_len(&v), 0);
    }

    #[test]
    fn test_push_inline() {
        let mut v = new_small_vec8();
        sv8_push(&mut v, 1.0);
        sv8_push(&mut v, 2.0);
        assert_eq!(sv8_len(&v), 2);
        assert!(sv8_is_inline(&v));
    }

    #[test]
    fn test_get() {
        let mut v = new_small_vec8();
        sv8_push(&mut v, 1.5);
        assert_eq!(sv8_get(&v, 0), Some(1.5));
        assert_eq!(sv8_get(&v, 1), None);
    }

    #[test]
    fn test_pop() {
        let mut v = new_small_vec8();
        sv8_push(&mut v, 7.0);
        assert_eq!(sv8_pop(&mut v), Some(7.0));
        assert!(sv8_is_empty(&v));
        assert_eq!(sv8_pop(&mut v), None);
    }

    #[test]
    fn test_heap_fallback() {
        let mut v = new_small_vec8();
        for i in 0..10 {
            sv8_push(&mut v, i as f32);
        }
        assert_eq!(sv8_len(&v), 10);
        assert!(!sv8_is_inline(&v));
        assert_eq!(sv8_get(&v, 9), Some(9.0));
    }

    #[test]
    fn test_to_vec() {
        let mut v = new_small_vec8();
        sv8_push(&mut v, 1.0);
        sv8_push(&mut v, 2.0);
        let vec = sv8_to_vec(&v);
        assert_eq!(vec, vec![1.0, 2.0]);
    }

    #[test]
    fn test_clear() {
        let mut v = new_small_vec8();
        sv8_push(&mut v, 1.0);
        sv8_clear(&mut v);
        assert!(sv8_is_empty(&v));
    }

    #[test]
    fn test_default_config() {
        let cfg = default_small_vec8_config();
        assert_eq!(cfg.initial_capacity, 8);
    }

    #[test]
    fn test_push_exactly_8_inline() {
        let mut v = new_small_vec8();
        for i in 0..8 {
            sv8_push(&mut v, i as f32);
        }
        assert!(sv8_is_inline(&v));
        assert_eq!(sv8_len(&v), 8);
    }
}
