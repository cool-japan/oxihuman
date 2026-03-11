// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cosine-similarity based nearest neighbor index.

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SimilarityIndex {
    pub items: Vec<(usize, Vec<f64>)>,
}

#[allow(dead_code)]
pub fn new_similarity_index() -> SimilarityIndex {
    SimilarityIndex { items: Vec::new() }
}

#[allow(dead_code)]
pub fn insert(idx: &mut SimilarityIndex, id: usize, features: Vec<f64>) {
    idx.items.push((id, features));
}

#[allow(dead_code)]
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dot: f64 = a[..len].iter().zip(b[..len].iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a[..len].iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b[..len].iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a < f64::EPSILON || norm_b < f64::EPSILON {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

#[allow(dead_code)]
pub fn nearest(idx: &SimilarityIndex, query: &[f64]) -> Option<usize> {
    idx.items
        .iter()
        .map(|(id, feats)| (*id, cosine_similarity(query, feats)))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(id, _)| id)
}

#[allow(dead_code)]
pub fn item_count(idx: &SimilarityIndex) -> usize {
    idx.items.len()
}

#[allow(dead_code)]
pub fn clear(idx: &mut SimilarityIndex) {
    idx.items.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-9);
    }

    #[test]
    fn test_nearest_returns_correct_id() {
        let mut idx = new_similarity_index();
        insert(&mut idx, 0, vec![1.0, 0.0]);
        insert(&mut idx, 1, vec![0.0, 1.0]);
        let result = nearest(&idx, &[1.0, 0.01]);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_empty_index_nearest() {
        let idx = new_similarity_index();
        assert!(nearest(&idx, &[1.0, 0.0]).is_none());
    }

    #[test]
    fn test_item_count() {
        let mut idx = new_similarity_index();
        insert(&mut idx, 0, vec![1.0]);
        insert(&mut idx, 1, vec![2.0]);
        assert_eq!(item_count(&idx), 2);
    }

    #[test]
    fn test_clear() {
        let mut idx = new_similarity_index();
        insert(&mut idx, 0, vec![1.0]);
        clear(&mut idx);
        assert_eq!(item_count(&idx), 0);
    }

    #[test]
    fn test_cosine_empty_slices() {
        let sim = cosine_similarity(&[], &[]);
        assert!((sim).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cosine_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-9);
    }
}
