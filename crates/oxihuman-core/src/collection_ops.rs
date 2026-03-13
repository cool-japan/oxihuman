// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Utility functions operating on slices and vectors.
#[allow(dead_code)]
pub fn dedup_sorted<T: PartialEq>(v: &mut Vec<T>) {
    v.dedup();
}

#[allow(dead_code)]
pub fn unique_sorted(v: &mut Vec<String>) {
    v.sort();
    v.dedup();
}

#[allow(dead_code)]
pub fn partition_by<T, F: Fn(&T) -> bool>(items: &[T], pred: F) -> (Vec<&T>, Vec<&T>) {
    let mut yes = Vec::new();
    let mut no = Vec::new();
    for item in items {
        if pred(item) {
            yes.push(item);
        } else {
            no.push(item);
        }
    }
    (yes, no)
}

#[allow(dead_code)]
pub fn flatten_nested(vecs: &[Vec<f32>]) -> Vec<f32> {
    vecs.iter().flat_map(|v| v.iter().copied()).collect()
}

#[allow(dead_code)]
pub fn chunk_vec<T: Clone>(items: &[T], chunk_size: usize) -> Vec<Vec<T>> {
    if chunk_size == 0 {
        return vec![];
    }
    items.chunks(chunk_size).map(|c| c.to_vec()).collect()
}

#[allow(dead_code)]
pub fn interleave<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut ia = a.iter();
    let mut ib = b.iter();
    loop {
        match (ia.next(), ib.next()) {
            (Some(x), Some(y)) => {
                result.push(x.clone());
                result.push(y.clone());
            }
            (Some(x), None) => result.push(x.clone()),
            (None, Some(y)) => result.push(y.clone()),
            (None, None) => break,
        }
    }
    result
}

#[allow(dead_code)]
pub fn min_f32(slice: &[f32]) -> Option<f32> {
    if slice.is_empty() {
        return None;
    }
    Some(slice.iter().copied().fold(f32::INFINITY, f32::min))
}

#[allow(dead_code)]
pub fn max_f32(slice: &[f32]) -> Option<f32> {
    if slice.is_empty() {
        return None;
    }
    Some(slice.iter().copied().fold(f32::NEG_INFINITY, f32::max))
}

#[allow(dead_code)]
pub fn sum_f32(slice: &[f32]) -> f32 {
    slice.iter().sum()
}

#[allow(dead_code)]
pub fn mean_f32(slice: &[f32]) -> Option<f32> {
    if slice.is_empty() {
        return None;
    }
    Some(sum_f32(slice) / slice.len() as f32)
}

#[allow(dead_code)]
pub fn sliding_window_avg(values: &[f32], window: usize) -> Vec<f32> {
    if window == 0 || values.is_empty() {
        return vec![];
    }
    values
        .windows(window)
        .map(|w| w.iter().sum::<f32>() / w.len() as f32)
        .collect()
}

#[allow(dead_code)]
pub fn zip_with<T: Copy, U: Copy, R>(a: &[T], b: &[U], f: impl Fn(T, U) -> R) -> Vec<R> {
    a.iter().zip(b.iter()).map(|(&x, &y)| f(x, y)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_sorted() {
        let mut v = vec![1, 1, 2, 3, 3];
        dedup_sorted(&mut v);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_unique_sorted() {
        let mut v = vec!["b".to_string(), "a".to_string(), "b".to_string()];
        unique_sorted(&mut v);
        assert_eq!(v, vec!["a", "b"]);
    }

    #[test]
    fn test_partition_by() {
        let data = vec![1, 2, 3, 4, 5];
        let (evens, odds) = partition_by(&data, |x| x % 2 == 0);
        assert_eq!(evens.len(), 2);
        assert_eq!(odds.len(), 3);
    }

    #[test]
    fn test_flatten_nested() {
        let v = vec![vec![1.0, 2.0], vec![3.0]];
        assert_eq!(flatten_nested(&v), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_chunk_vec() {
        let v = vec![1, 2, 3, 4, 5];
        let chunks = chunk_vec(&v, 2);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![1, 2]);
    }

    #[test]
    fn test_interleave() {
        let a = vec![1, 3, 5];
        let b = vec![2, 4];
        assert_eq!(interleave(&a, &b), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_min_max_f32() {
        let v = vec![3.0_f32, 1.0, 2.0];
        assert_eq!(min_f32(&v), Some(1.0));
        assert_eq!(max_f32(&v), Some(3.0));
    }

    #[test]
    fn test_mean_f32() {
        let v = vec![2.0_f32, 4.0, 6.0];
        let m = mean_f32(&v).expect("should succeed");
        assert!((m - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sliding_window_avg() {
        let v = vec![1.0_f32, 2.0, 3.0, 4.0];
        let r = sliding_window_avg(&v, 2);
        assert_eq!(r.len(), 3);
        assert!((r[0] - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_zip_with() {
        let a = vec![1.0_f32, 2.0];
        let b = vec![3.0_f32, 4.0];
        let r = zip_with(&a, &b, |x, y| x + y);
        assert!((r[0] - 4.0).abs() < f32::EPSILON);
    }
}
