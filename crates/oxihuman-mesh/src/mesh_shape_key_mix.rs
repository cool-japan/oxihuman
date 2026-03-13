// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Shape key mixing and combining.

/// A single shape key: name, weight, and per-vertex positions.
#[derive(Debug, Clone)]
pub struct ShapeKey {
    pub name: String,
    pub weight: f32,
    pub positions: Vec<[f32; 3]>,
}

impl ShapeKey {
    /// Create a new shape key.
    pub fn new(name: &str, weight: f32, positions: Vec<[f32; 3]>) -> Self {
        Self {
            name: name.to_string(),
            weight: weight.clamp(0.0, 1.0),
            positions,
        }
    }
}

/// Mix multiple shape keys into a combined result.
/// Each key is blended by its weight. Basis positions are added.
pub fn mix_shape_keys(basis: &[[f32; 3]], keys: &[ShapeKey]) -> Vec<[f32; 3]> {
    let n = basis.len();
    let mut result = basis.to_vec();
    for key in keys {
        let w = key.weight;
        let plen = key.positions.len().min(n);
        for i in 0..plen {
            result[i][0] += (key.positions[i][0] - basis[i][0]) * w;
            result[i][1] += (key.positions[i][1] - basis[i][1]) * w;
            result[i][2] += (key.positions[i][2] - basis[i][2]) * w;
        }
    }
    result
}

/// Set weight for a named shape key (clamped to `[0,1]`).
pub fn set_key_weight(keys: &mut [ShapeKey], name: &str, weight: f32) -> bool {
    if let Some(k) = keys.iter_mut().find(|k| k.name == name) {
        k.weight = weight.clamp(0.0, 1.0);
        true
    } else {
        false
    }
}

/// Return the total influence sum across all keys.
pub fn total_influence(keys: &[ShapeKey]) -> f32 {
    keys.iter().map(|k| k.weight).sum()
}

/// Return the index of the key with the highest weight.
pub fn dominant_key(keys: &[ShapeKey]) -> Option<usize> {
    keys.iter()
        .enumerate()
        .max_by(|a, b| {
            a.1.weight
                .partial_cmp(&b.1.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

/// Reset all key weights to zero.
pub fn reset_all_weights(keys: &mut [ShapeKey]) {
    for k in keys.iter_mut() {
        k.weight = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basis() -> Vec<[f32; 3]> {
        vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0]]
    }

    fn two_keys() -> Vec<ShapeKey> {
        vec![
            ShapeKey::new("smile", 0.5, vec![[0.0, 0.5, 0.0], [1.0, 0.5, 0.0]]),
            ShapeKey::new("open", 0.0, vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0]]),
        ]
    }

    #[test]
    fn test_mix_basis_only() {
        /* with all weights zero result equals basis */
        let b = basis();
        let keys = vec![ShapeKey::new("k", 0.0, b.clone())];
        let result = mix_shape_keys(&b, &keys);
        assert!((result[0][0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_mix_full_weight() {
        /* with weight 1.0 result equals key positions */
        let b = basis();
        let target = vec![[0.0_f32, 2.0, 0.0], [1.0, 2.0, 0.0]];
        let keys = vec![ShapeKey::new("k", 1.0, target.clone())];
        let result = mix_shape_keys(&b, &keys);
        assert!((result[0][1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_mix_half_weight() {
        /* half weight blends halfway */
        let b = basis();
        let target = vec![[0.0_f32, 2.0, 0.0], [1.0, 2.0, 0.0]];
        let keys = vec![ShapeKey::new("k", 0.5, target)];
        let result = mix_shape_keys(&b, &keys);
        assert!((result[0][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_key_weight() {
        /* set_key_weight updates correctly */
        let mut keys = two_keys();
        assert!(set_key_weight(&mut keys, "smile", 0.8));
        assert!((keys[0].weight - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_key_weight_missing() {
        /* set_key_weight returns false for unknown key */
        let mut keys = two_keys();
        assert!(!set_key_weight(&mut keys, "no_such_key", 0.5));
    }

    #[test]
    fn test_total_influence() {
        /* total influence sums weights */
        let keys = two_keys();
        assert!((total_influence(&keys) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_key() {
        /* dominant key index is correct */
        let keys = two_keys();
        assert_eq!(dominant_key(&keys), Some(0));
    }

    #[test]
    fn test_reset_all_weights() {
        /* reset sets all weights to zero */
        let mut keys = two_keys();
        reset_all_weights(&mut keys);
        assert!((total_influence(&keys) - 0.0).abs() < 1e-6);
    }
}
