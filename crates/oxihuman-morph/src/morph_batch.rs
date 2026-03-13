#![allow(dead_code)]
//! Batch application of multiple morph targets at once.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MorphBatch {
    names: Vec<String>,
    weights: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_morph_batch() -> MorphBatch {
    MorphBatch {
        names: Vec::new(),
        weights: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_morph_to_batch(batch: &mut MorphBatch, name: &str, weight: f32) {
    batch.names.push(name.to_string());
    batch.weights.push(weight.clamp(0.0, 1.0));
}

#[allow(dead_code)]
pub fn batch_apply(batch: &MorphBatch) -> Vec<(String, f32)> {
    batch
        .names
        .iter()
        .zip(batch.weights.iter())
        .map(|(n, &w)| (n.clone(), w))
        .collect()
}

#[allow(dead_code)]
pub fn batch_count(batch: &MorphBatch) -> usize {
    batch.names.len()
}

#[allow(dead_code)]
pub fn batch_clear(batch: &mut MorphBatch) {
    batch.names.clear();
    batch.weights.clear();
}

#[allow(dead_code)]
pub fn batch_weight_at(batch: &MorphBatch, index: usize) -> Option<f32> {
    batch.weights.get(index).copied()
}

#[allow(dead_code)]
pub fn batch_name_at(batch: &MorphBatch, index: usize) -> Option<&str> {
    batch.names.get(index).map(|s| s.as_str())
}

#[allow(dead_code)]
pub fn batch_total_weight(batch: &MorphBatch) -> f32 {
    batch.weights.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_batch() {
        let b = new_morph_batch();
        assert_eq!(batch_count(&b), 0);
    }

    #[test]
    fn test_add_morph_to_batch() {
        let mut b = new_morph_batch();
        add_morph_to_batch(&mut b, "smile", 0.5);
        assert_eq!(batch_count(&b), 1);
    }

    #[test]
    fn test_batch_apply() {
        let mut b = new_morph_batch();
        add_morph_to_batch(&mut b, "smile", 0.8);
        let applied = batch_apply(&b);
        assert_eq!(applied.len(), 1);
        assert!((applied[0].1 - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_batch_clear() {
        let mut b = new_morph_batch();
        add_morph_to_batch(&mut b, "a", 0.1);
        batch_clear(&mut b);
        assert_eq!(batch_count(&b), 0);
    }

    #[test]
    fn test_batch_weight_at() {
        let mut b = new_morph_batch();
        add_morph_to_batch(&mut b, "x", 0.3);
        assert!((batch_weight_at(&b, 0).expect("should succeed") - 0.3).abs() < 1e-6);
        assert!(batch_weight_at(&b, 5).is_none());
    }

    #[test]
    fn test_batch_name_at() {
        let mut b = new_morph_batch();
        add_morph_to_batch(&mut b, "brow", 0.2);
        assert_eq!(batch_name_at(&b, 0), Some("brow"));
        assert!(batch_name_at(&b, 1).is_none());
    }

    #[test]
    fn test_batch_total_weight() {
        let mut b = new_morph_batch();
        add_morph_to_batch(&mut b, "a", 0.3);
        add_morph_to_batch(&mut b, "b", 0.4);
        assert!((batch_total_weight(&b) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_weight() {
        let mut b = new_morph_batch();
        add_morph_to_batch(&mut b, "x", 2.0);
        assert!((batch_weight_at(&b, 0).expect("should succeed") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_entries() {
        let mut b = new_morph_batch();
        for i in 0..5 {
            add_morph_to_batch(&mut b, &format!("m{i}"), 0.1 * (i as f32 + 1.0));
        }
        assert_eq!(batch_count(&b), 5);
    }

    #[test]
    fn test_batch_apply_empty() {
        let b = new_morph_batch();
        assert!(batch_apply(&b).is_empty());
    }
}
