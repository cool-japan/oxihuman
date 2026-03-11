// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Map from left vertex indices to their right-side mirror indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FacialSymmetryMap {
    pub pairs: Vec<(usize, usize)>,
}

/// Create a new empty symmetry map.
#[allow(dead_code)]
pub fn new_symmetry_map() -> FacialSymmetryMap {
    FacialSymmetryMap { pairs: Vec::new() }
}

/// Add a (left, right) vertex index pair to the map.
#[allow(dead_code)]
pub fn add_pair(map: &mut FacialSymmetryMap, left: usize, right: usize) {
    map.pairs.push((left, right));
}

/// Enforce symmetry on a weight slice by averaging left/right pairs.
/// `strength` in [0, 1] controls blend toward perfect symmetry.
#[allow(dead_code)]
pub fn enforce_symmetry(weights: &mut [f32], map: &FacialSymmetryMap, strength: f32) {
    let s = strength.clamp(0.0, 1.0);
    for &(l, r) in &map.pairs {
        if l < weights.len() && r < weights.len() {
            let avg = (weights[l] + weights[r]) * 0.5;
            weights[l] = weights[l] + (avg - weights[l]) * s;
            weights[r] = weights[r] + (avg - weights[r]) * s;
        }
    }
}

/// Compute the total symmetry error (sum of absolute differences for all pairs).
#[allow(dead_code)]
pub fn symmetry_error(weights: &[f32], map: &FacialSymmetryMap) -> f32 {
    map.pairs
        .iter()
        .filter_map(|&(l, r)| {
            if l < weights.len() && r < weights.len() {
                Some((weights[l] - weights[r]).abs())
            } else {
                None
            }
        })
        .sum()
}

/// Return the number of pairs in the map.
#[allow(dead_code)]
pub fn pair_count(map: &FacialSymmetryMap) -> usize {
    map.pairs.len()
}

// ── New types required by task ────────────────────────────────────────────────

/// Per-vertex asymmetry weights and midline offset for the face.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FacialSymmetry {
    pub asymmetry_weight: f32,
    pub midline_offset: f32,
    pub symmetry_blend: f32,
}

/// Compute the mean absolute difference between left- and right-side params.
#[allow(dead_code)]
pub fn compute_symmetry_error(left: &[f32], right: &[f32]) -> f32 {
    let n = left.len().min(right.len());
    if n == 0 {
        return 0.0;
    }
    let sum: f32 = (0..n).map(|i| (left[i] - right[i]).abs()).sum();
    sum / n as f32
}

/// Mirror right params onto left (make perfectly symmetric).
#[allow(dead_code)]
pub fn symmetrize_params(left: &mut [f32], right: &[f32]) {
    let n = left.len().min(right.len());
    left[..n].copy_from_slice(&right[..n]);
}

/// Return the asymmetry weight for the symmetry record.
#[allow(dead_code)]
pub fn asymmetry_weight(fs: &FacialSymmetry) -> f32 {
    fs.asymmetry_weight
}

/// Return the facial midline offset.
#[allow(dead_code)]
pub fn facial_midline_offset(fs: &FacialSymmetry) -> f32 {
    fs.midline_offset
}

/// Apply a symmetry correction by blending left toward right at `strength`.
#[allow(dead_code)]
pub fn apply_symmetry_correction(left: &mut [f32], right: &[f32], strength: f32) {
    let n = left.len().min(right.len());
    let s = strength.clamp(0.0, 1.0);
    for i in 0..n {
        left[i] = left[i] + (right[i] - left[i]) * s;
    }
}

/// Score in [0, 1] where 1.0 is perfectly symmetric.
#[allow(dead_code)]
pub fn symmetry_score(left: &[f32], right: &[f32]) -> f32 {
    let err = compute_symmetry_error(left, right);
    1.0 / (1.0 + err)
}

/// Return the signed difference (left[i] - right[i]) for each index.
#[allow(dead_code)]
pub fn left_right_delta(left: &[f32], right: &[f32]) -> Vec<f32> {
    let n = left.len().min(right.len());
    (0..n).map(|i| left[i] - right[i]).collect()
}

/// Blend left and right toward their average by `t`.
#[allow(dead_code)]
pub fn symmetry_blend(left: &mut [f32], right: &mut [f32], t: f32) {
    let n = left.len().min(right.len());
    let t = t.clamp(0.0, 1.0);
    for i in 0..n {
        let avg = (left[i] + right[i]) * 0.5;
        left[i] = left[i] + (avg - left[i]) * t;
        right[i] = right[i] + (avg - right[i]) * t;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_symmetry_map_empty() {
        let m = new_symmetry_map();
        assert_eq!(pair_count(&m), 0);
    }

    #[test]
    fn add_pair_increments_count() {
        let mut m = new_symmetry_map();
        add_pair(&mut m, 0, 1);
        assert_eq!(pair_count(&m), 1);
    }

    #[test]
    fn enforce_symmetry_full_strength() {
        let mut m = new_symmetry_map();
        add_pair(&mut m, 0, 1);
        let mut weights = vec![0.0, 1.0, 0.5];
        enforce_symmetry(&mut weights, &m, 1.0);
        assert!((weights[0] - 0.5).abs() < 1e-5);
        assert!((weights[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn enforce_symmetry_zero_strength_no_change() {
        let mut m = new_symmetry_map();
        add_pair(&mut m, 0, 1);
        let mut weights = vec![0.0, 1.0];
        enforce_symmetry(&mut weights, &m, 0.0);
        assert!((weights[0] - 0.0).abs() < 1e-5);
        assert!((weights[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn symmetry_error_symmetric_weights() {
        let mut m = new_symmetry_map();
        add_pair(&mut m, 0, 1);
        let weights = vec![0.5, 0.5];
        assert!((symmetry_error(&weights, &m)).abs() < 1e-6);
    }

    #[test]
    fn symmetry_error_asymmetric() {
        let mut m = new_symmetry_map();
        add_pair(&mut m, 0, 1);
        let weights = vec![0.0, 1.0];
        assert!((symmetry_error(&weights, &m) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn out_of_range_pairs_skipped() {
        let mut m = new_symmetry_map();
        add_pair(&mut m, 0, 100); // index 100 out of range
        let mut weights = vec![0.5];
        enforce_symmetry(&mut weights, &m, 1.0); // should not panic
        assert!((weights[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn multiple_pairs() {
        let mut m = new_symmetry_map();
        add_pair(&mut m, 0, 3);
        add_pair(&mut m, 1, 4);
        add_pair(&mut m, 2, 5);
        assert_eq!(pair_count(&m), 3);
    }

    #[test]
    fn enforce_symmetry_partial_strength() {
        let mut m = new_symmetry_map();
        add_pair(&mut m, 0, 1);
        let mut weights = vec![0.0, 1.0];
        enforce_symmetry(&mut weights, &m, 0.5);
        // avg = 0.5; each moves half way to 0.5
        assert!((weights[0] - 0.25).abs() < 1e-5);
        assert!((weights[1] - 0.75).abs() < 1e-5);
    }

    #[test]
    fn symmetry_error_empty_map() {
        let m = new_symmetry_map();
        let weights = vec![0.1, 0.9];
        assert!((symmetry_error(&weights, &m)).abs() < 1e-9);
    }

    #[test]
    fn test_compute_symmetry_error_symmetric() {
        let left = vec![0.5, 0.5];
        let right = vec![0.5, 0.5];
        assert!((compute_symmetry_error(&left, &right)).abs() < 1e-6);
    }

    #[test]
    fn test_compute_symmetry_error_asymmetric() {
        let left = vec![0.0, 0.0];
        let right = vec![1.0, 1.0];
        let e = compute_symmetry_error(&left, &right);
        assert!((e - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_symmetrize_params() {
        let mut left = vec![0.0, 0.0];
        let right = vec![1.0, 0.5];
        symmetrize_params(&mut left, &right);
        assert!((left[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_asymmetry_weight_default() {
        let fs = FacialSymmetry::default();
        assert_eq!(asymmetry_weight(&fs), 0.0);
    }

    #[test]
    fn test_symmetry_score_perfect() {
        let v = vec![0.5, 0.5];
        let score = symmetry_score(&v, &v);
        assert!((score - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_left_right_delta() {
        let left = vec![1.0, 0.5];
        let right = vec![0.5, 0.5];
        let d = left_right_delta(&left, &right);
        assert!((d[0] - 0.5).abs() < 1e-6);
        assert!(d[1].abs() < 1e-6);
    }

    #[test]
    fn test_symmetry_blend_midpoint() {
        let mut l = vec![0.0, 0.0];
        let mut r = vec![1.0, 1.0];
        symmetry_blend(&mut l, &mut r, 1.0);
        assert!((l[0] - 0.5).abs() < 1e-5);
        assert!((r[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_apply_symmetry_correction_full() {
        let mut left = vec![0.0, 0.0];
        let right = vec![1.0, 0.5];
        apply_symmetry_correction(&mut left, &right, 1.0);
        assert!((left[0] - 1.0).abs() < 1e-6);
    }
}
