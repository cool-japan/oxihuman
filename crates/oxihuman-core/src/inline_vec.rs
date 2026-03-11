#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fixed-capacity inline vector (no heap allocation up to N).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InlineVec<const N: usize> {
    data: [f64; 16],
    len: usize,
}

#[allow(dead_code)]
pub fn new_inline_vec<const N: usize>() -> InlineVec<N> {
    assert!(N <= 16, "InlineVec max capacity is 16");
    InlineVec { data: [0.0; 16], len: 0 }
}

#[allow(dead_code)]
pub fn inline_push<const N: usize>(v: &mut InlineVec<N>, val: f64) -> bool {
    if v.len < N {
        v.data[v.len] = val;
        v.len += 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn inline_pop<const N: usize>(v: &mut InlineVec<N>) -> Option<f64> {
    if v.len > 0 {
        v.len -= 1;
        Some(v.data[v.len])
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn inline_get<const N: usize>(v: &InlineVec<N>, idx: usize) -> Option<f64> {
    if idx < v.len {
        Some(v.data[idx])
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn inline_len<const N: usize>(v: &InlineVec<N>) -> usize {
    v.len
}

#[allow(dead_code)]
pub fn inline_is_full_iv<const N: usize>(v: &InlineVec<N>) -> bool {
    v.len >= N
}

#[allow(dead_code)]
pub fn inline_clear<const N: usize>(v: &mut InlineVec<N>) {
    v.len = 0;
}

#[allow(dead_code)]
pub fn inline_capacity_iv<const N: usize>(_v: &InlineVec<N>) -> usize {
    N
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_inline_vec() {
        let v = new_inline_vec::<4>();
        assert_eq!(inline_len(&v), 0);
    }

    #[test]
    fn test_push_pop() {
        let mut v = new_inline_vec::<4>();
        assert!(inline_push(&mut v, 1.0));
        assert_eq!(inline_pop(&mut v), Some(1.0));
    }

    #[test]
    fn test_push_full() {
        let mut v = new_inline_vec::<2>();
        assert!(inline_push(&mut v, 1.0));
        assert!(inline_push(&mut v, 2.0));
        assert!(!inline_push(&mut v, 3.0));
    }

    #[test]
    fn test_pop_empty() {
        let mut v = new_inline_vec::<4>();
        assert_eq!(inline_pop(&mut v), None);
    }

    #[test]
    fn test_get() {
        let mut v = new_inline_vec::<4>();
        inline_push(&mut v, 42.0);
        assert_eq!(inline_get(&v, 0), Some(42.0));
        assert_eq!(inline_get(&v, 1), None);
    }

    #[test]
    fn test_is_full() {
        let mut v = new_inline_vec::<1>();
        assert!(!inline_is_full_iv(&v));
        inline_push(&mut v, 1.0);
        assert!(inline_is_full_iv(&v));
    }

    #[test]
    fn test_clear() {
        let mut v = new_inline_vec::<4>();
        inline_push(&mut v, 1.0);
        inline_clear(&mut v);
        assert_eq!(inline_len(&v), 0);
    }

    #[test]
    fn test_capacity() {
        let v = new_inline_vec::<8>();
        assert_eq!(inline_capacity_iv(&v), 8);
    }

    #[test]
    fn test_len_tracking() {
        let mut v = new_inline_vec::<4>();
        inline_push(&mut v, 1.0);
        inline_push(&mut v, 2.0);
        assert_eq!(inline_len(&v), 2);
    }

    #[test]
    fn test_push_pop_sequence() {
        let mut v = new_inline_vec::<4>();
        inline_push(&mut v, 10.0);
        inline_push(&mut v, 20.0);
        assert_eq!(inline_pop(&mut v), Some(20.0));
        assert_eq!(inline_pop(&mut v), Some(10.0));
    }
}
