// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::params::ParamState;

/// A single parameter difference.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParamDifference {
    pub param_name: String,
    pub value_a: f32,
    pub value_b: f32,
    /// b - a
    pub delta: f32,
    /// |b - a|
    pub abs_delta: f32,
    /// (b - a) / max(a, b, 0.001)
    pub relative_delta: f32,
}

/// Comprehensive comparison between two body shapes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeComparison {
    pub differences: Vec<ParamDifference>,
    /// Cosine similarity [0, 1] of the parameter vectors.
    pub cosine_similarity: f32,
    /// Euclidean distance between parameter vectors.
    pub euclidean_distance: f32,
    /// Manhattan distance.
    pub manhattan_distance: f32,
    /// Weighted similarity score [0, 1] (1 = identical).
    pub similarity_score: f32,
    /// Name of the most different parameter.
    pub most_different_param: Option<String>,
}

impl ShapeComparison {
    /// Parameters sorted by absolute difference (largest first).
    #[allow(dead_code)]
    pub fn ranked_differences(&self) -> Vec<&ParamDifference> {
        let mut refs: Vec<&ParamDifference> = self.differences.iter().collect();
        refs.sort_by(|a, b| {
            b.abs_delta
                .partial_cmp(&a.abs_delta)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        refs
    }

    /// Parameters where abs_delta > threshold.
    #[allow(dead_code)]
    pub fn significant_differences(&self, threshold: f32) -> Vec<&ParamDifference> {
        self.differences
            .iter()
            .filter(|d| d.abs_delta > threshold)
            .collect()
    }

    /// Human-readable summary.
    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        let most_diff = self.most_different_param.as_deref().unwrap_or("none");
        format!(
            "similarity={:.4}, cosine={:.4}, euclidean={:.4}, manhattan={:.4}, most_different={}",
            self.similarity_score,
            self.cosine_similarity,
            self.euclidean_distance,
            self.manhattan_distance,
            most_diff,
        )
    }

    /// Whether shapes are essentially the same (all params within tolerance).
    #[allow(dead_code)]
    pub fn is_similar(&self, tolerance: f32) -> bool {
        self.differences.iter().all(|d| d.abs_delta <= tolerance)
    }
}

/// Extract the core 4-element parameter vector from a ParamState.
fn param_vec(s: &ParamState) -> [f32; 4] {
    [s.height, s.weight, s.muscle, s.age]
}

/// Compare two ParamState values.
#[allow(dead_code)]
pub fn compare_shapes(a: &ParamState, b: &ParamState) -> ShapeComparison {
    let core_names = ["height", "weight", "muscle", "age"];
    let core_a = param_vec(a);
    let core_b = param_vec(b);

    let mut differences: Vec<ParamDifference> = core_names
        .iter()
        .zip(core_a.iter())
        .zip(core_b.iter())
        .map(|((name, &va), &vb)| {
            let delta = vb - va;
            let abs_delta = delta.abs();
            let denom = va.abs().max(vb.abs()).max(0.001);
            let relative_delta = delta / denom;
            ParamDifference {
                param_name: name.to_string(),
                value_a: va,
                value_b: vb,
                delta,
                abs_delta,
                relative_delta,
            }
        })
        .collect();

    // Add shared extra keys
    for (key, &va) in &a.extra {
        if let Some(&vb) = b.extra.get(key) {
            let delta = vb - va;
            let abs_delta = delta.abs();
            let denom = va.abs().max(vb.abs()).max(0.001);
            let relative_delta = delta / denom;
            differences.push(ParamDifference {
                param_name: key.clone(),
                value_a: va,
                value_b: vb,
                delta,
                abs_delta,
                relative_delta,
            });
        }
    }

    let cos_sim = cosine_similarity(a, b);
    let euc_dist = euclidean_distance(a, b);
    let man_dist = manhattan_distance(a, b);

    // Weighted similarity: based on cosine similarity and euclidean distance
    // Normalize euclidean distance into [0, 1]: max possible for 4 params is 2.0 (each 0 to 1)
    let max_euc = (4.0f32).sqrt(); // sqrt(4 * 1^2)
    let euc_sim = 1.0 - (euc_dist / max_euc).min(1.0);
    let similarity_score = 0.5 * cos_sim + 0.5 * euc_sim;

    let most_different_param = differences
        .iter()
        .max_by(|a, b| {
            a.abs_delta
                .partial_cmp(&b.abs_delta)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .filter(|d| d.abs_delta > 0.0)
        .map(|d| d.param_name.clone());

    ShapeComparison {
        differences,
        cosine_similarity: cos_sim,
        euclidean_distance: euc_dist,
        manhattan_distance: man_dist,
        similarity_score,
        most_different_param,
    }
}

/// Compute cosine similarity between two ParamState vectors.
#[allow(dead_code)]
pub fn cosine_similarity(a: &ParamState, b: &ParamState) -> f32 {
    let va = param_vec(a);
    let vb = param_vec(b);

    let dot: f32 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = va.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = vb.iter().map(|x| x * x).sum::<f32>().sqrt();

    let denom = norm_a * norm_b;
    if denom < 1e-10 {
        0.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

/// Euclidean distance between two ParamState vectors.
#[allow(dead_code)]
pub fn euclidean_distance(a: &ParamState, b: &ParamState) -> f32 {
    let va = param_vec(a);
    let vb = param_vec(b);
    va.iter()
        .zip(vb.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Manhattan distance between two ParamState vectors.
#[allow(dead_code)]
pub fn manhattan_distance(a: &ParamState, b: &ParamState) -> f32 {
    let va = param_vec(a);
    let vb = param_vec(b);
    va.iter().zip(vb.iter()).map(|(x, y)| (x - y).abs()).sum()
}

/// Suggest how to adjust `current` params to move toward `target` by `step` amount.
/// Returns a new ParamState that is `step` closer to target.
#[allow(dead_code)]
pub fn step_toward(current: &ParamState, target: &ParamState, step: f32) -> ParamState {
    let dist = euclidean_distance(current, target);
    if dist < 1e-10 {
        return current.clone();
    }
    // t is how far we move along the direction, clamped to [0, 1]
    let t = (step / dist).min(1.0);
    interpolate_shapes(current, target, t)
}

/// Interpolate between two shapes at t in [0, 1].
#[allow(dead_code)]
pub fn interpolate_shapes(a: &ParamState, b: &ParamState, t: f32) -> ParamState {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;

    let mut extra = a.extra.clone();
    // Interpolate shared extra keys
    for (key, &va) in &a.extra {
        if let Some(&vb) = b.extra.get(key) {
            extra.insert(key.clone(), lerp(va, vb));
        }
    }
    // Keys only in b get added at weight t
    for (key, &vb) in &b.extra {
        if !a.extra.contains_key(key) {
            extra.insert(key.clone(), vb * t);
        }
    }

    ParamState {
        height: lerp(a.height, b.height),
        weight: lerp(a.weight, b.weight),
        muscle: lerp(a.muscle, b.muscle),
        age: lerp(a.age, b.age),
        extra,
    }
}

/// Find the shape that is the "average" of a collection of shapes.
#[allow(dead_code)]
pub fn average_shape(shapes: &[ParamState]) -> Option<ParamState> {
    if shapes.is_empty() {
        return None;
    }
    let n = shapes.len() as f32;
    let height = shapes.iter().map(|s| s.height).sum::<f32>() / n;
    let weight = shapes.iter().map(|s| s.weight).sum::<f32>() / n;
    let muscle = shapes.iter().map(|s| s.muscle).sum::<f32>() / n;
    let age = shapes.iter().map(|s| s.age).sum::<f32>() / n;

    // Average extra keys that appear in all shapes
    let mut extra = std::collections::HashMap::new();
    if !shapes.is_empty() {
        // Collect all extra keys
        let all_keys: std::collections::HashSet<&str> = shapes
            .iter()
            .flat_map(|s| s.extra.keys().map(|k| k.as_str()))
            .collect();
        for key in all_keys {
            let count = shapes.iter().filter(|s| s.extra.contains_key(key)).count();
            if count == shapes.len() {
                let sum: f32 = shapes.iter().filter_map(|s| s.extra.get(key)).sum();
                extra.insert(key.to_string(), sum / n);
            }
        }
    }

    Some(ParamState {
        height,
        weight,
        muscle,
        age,
        extra,
    })
}

/// Cluster shapes into N groups by similarity (simple k-means on param vectors).
/// Returns group assignments (index per shape).
#[allow(dead_code)]
pub fn cluster_shapes(shapes: &[ParamState], k: usize, iterations: usize) -> Vec<usize> {
    let n = shapes.len();
    if n == 0 || k == 0 {
        return vec![];
    }
    let k = k.min(n);

    // Initialize centroids from first k shapes
    let mut centroids: Vec<[f32; 4]> = shapes[..k].iter().map(param_vec).collect();
    let vecs: Vec<[f32; 4]> = shapes.iter().map(param_vec).collect();

    let mut assignments = vec![0usize; n];

    for _ in 0..iterations {
        // Assignment step
        for (i, v) in vecs.iter().enumerate() {
            let best = centroids
                .iter()
                .enumerate()
                .min_by(|(_, ca), (_, cb)| {
                    let da: f32 = v.iter().zip(ca.iter()).map(|(x, y)| (x - y).powi(2)).sum();
                    let db: f32 = v.iter().zip(cb.iter()).map(|(x, y)| (x - y).powi(2)).sum();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            assignments[i] = best;
        }

        // Update step
        let mut new_centroids = vec![[0.0f32; 4]; k];
        let mut counts = vec![0usize; k];
        for (i, &cluster) in assignments.iter().enumerate() {
            counts[cluster] += 1;
            let v = &vecs[i];
            let c = &mut new_centroids[cluster];
            c.iter_mut().zip(v.iter()).for_each(|(ci, vi)| *ci += vi);
        }
        for c in 0..k {
            if counts[c] > 0 {
                let cnt = counts[c] as f32;
                new_centroids[c].iter_mut().for_each(|x| *x /= cnt);
                centroids[c] = new_centroids[c];
            }
        }
    }

    assignments
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(h: f32, w: f32, m: f32, a: f32) -> ParamState {
        ParamState::new(h, w, m, a)
    }

    #[test]
    fn compare_identical_shapes_similarity_one() {
        let s = p(0.5, 0.5, 0.5, 0.5);
        let cmp = compare_shapes(&s, &s);
        assert!(
            (cmp.similarity_score - 1.0).abs() < 1e-5,
            "expected ~1.0, got {}",
            cmp.similarity_score
        );
    }

    #[test]
    fn compare_different_heights_detects_height() {
        let a = p(0.2, 0.5, 0.5, 0.5);
        let b = p(0.8, 0.5, 0.5, 0.5);
        let cmp = compare_shapes(&a, &b);
        let most_diff = cmp.most_different_param.as_deref().unwrap_or("");
        assert_eq!(most_diff, "height");
    }

    #[test]
    fn cosine_similarity_identical_is_one() {
        let s = p(0.3, 0.7, 0.4, 0.6);
        let sim = cosine_similarity(&s, &s);
        assert!((sim - 1.0).abs() < 1e-5, "expected 1.0, got {}", sim);
    }

    #[test]
    fn cosine_similarity_orthogonal_is_zero() {
        // Construct two orthogonal vectors
        // [1,0,0,0] and [0,1,0,0] are orthogonal
        let a = ParamState {
            height: 1.0,
            weight: 0.0,
            muscle: 0.0,
            age: 0.0,
            extra: Default::default(),
        };
        let b = ParamState {
            height: 0.0,
            weight: 1.0,
            muscle: 0.0,
            age: 0.0,
            extra: Default::default(),
        };
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-5, "expected ~0, got {}", sim);
    }

    #[test]
    fn euclidean_distance_zero_for_same() {
        let s = p(0.4, 0.6, 0.3, 0.7);
        let d = euclidean_distance(&s, &s);
        assert!(d.abs() < 1e-6, "expected 0, got {}", d);
    }

    #[test]
    fn manhattan_distance_zero_for_same() {
        let s = p(0.4, 0.6, 0.3, 0.7);
        let d = manhattan_distance(&s, &s);
        assert!(d.abs() < 1e-6, "expected 0, got {}", d);
    }

    #[test]
    fn ranked_differences_sorted_desc() {
        let a = p(0.1, 0.5, 0.5, 0.5);
        let b = p(0.9, 0.5, 0.5, 0.8);
        let cmp = compare_shapes(&a, &b);
        let ranked = cmp.ranked_differences();
        // Must be in descending order
        for i in 0..ranked.len().saturating_sub(1) {
            assert!(ranked[i].abs_delta >= ranked[i + 1].abs_delta);
        }
    }

    #[test]
    fn significant_differences_above_threshold() {
        let a = p(0.1, 0.5, 0.5, 0.5);
        let b = p(0.9, 0.5, 0.5, 0.5);
        let cmp = compare_shapes(&a, &b);
        let sig = cmp.significant_differences(0.5);
        assert_eq!(sig.len(), 1);
        assert_eq!(sig[0].param_name, "height");
    }

    #[test]
    fn step_toward_moves_closer() {
        let current = p(0.0, 0.5, 0.5, 0.5);
        let target = p(1.0, 0.5, 0.5, 0.5);
        let stepped = step_toward(&current, &target, 0.1);
        let d_before = euclidean_distance(&current, &target);
        let d_after = euclidean_distance(&stepped, &target);
        assert!(d_after < d_before, "should be closer after step");
    }

    #[test]
    fn interpolate_shapes_midpoint() {
        let a = p(0.0, 0.0, 0.0, 0.0);
        let b = p(1.0, 1.0, 1.0, 1.0);
        let mid = interpolate_shapes(&a, &b, 0.5);
        assert!((mid.height - 0.5).abs() < 1e-5);
        assert!((mid.weight - 0.5).abs() < 1e-5);
        assert!((mid.muscle - 0.5).abs() < 1e-5);
        assert!((mid.age - 0.5).abs() < 1e-5);
    }

    #[test]
    fn average_shape_of_two() {
        let a = p(0.0, 0.0, 0.0, 0.0);
        let b = p(1.0, 1.0, 1.0, 1.0);
        let avg = average_shape(&[a, b]).unwrap();
        assert!((avg.height - 0.5).abs() < 1e-5);
        assert!((avg.weight - 0.5).abs() < 1e-5);
    }

    #[test]
    fn cluster_shapes_returns_correct_count() {
        let shapes = vec![
            p(0.1, 0.1, 0.1, 0.1),
            p(0.9, 0.9, 0.9, 0.9),
            p(0.2, 0.1, 0.1, 0.1),
            p(0.8, 0.9, 0.9, 0.9),
        ];
        let assignments = cluster_shapes(&shapes, 2, 10);
        assert_eq!(assignments.len(), shapes.len());
    }

    #[test]
    fn cluster_shapes_k1_all_same_group() {
        let shapes = vec![
            p(0.1, 0.2, 0.3, 0.4),
            p(0.5, 0.6, 0.7, 0.8),
            p(0.9, 0.1, 0.2, 0.3),
        ];
        let assignments = cluster_shapes(&shapes, 1, 5);
        assert!(assignments.iter().all(|&a| a == 0));
    }

    #[test]
    fn is_similar_true_for_close_shapes() {
        let a = p(0.5, 0.5, 0.5, 0.5);
        let b = p(0.501, 0.499, 0.5, 0.5);
        let cmp = compare_shapes(&a, &b);
        assert!(cmp.is_similar(0.01));
    }

    #[test]
    fn summary_not_empty() {
        let a = p(0.3, 0.4, 0.5, 0.6);
        let b = p(0.7, 0.6, 0.5, 0.4);
        let cmp = compare_shapes(&a, &b);
        let s = cmp.summary();
        assert!(!s.is_empty());
    }
}
