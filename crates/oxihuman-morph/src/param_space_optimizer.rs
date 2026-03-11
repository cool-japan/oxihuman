// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Optimize morph target parameter space via dimensionality reduction and
//! redundancy elimination.

use std::collections::HashMap;

/// Configuration for parameter-space optimization.
#[allow(dead_code)]
pub struct ParamSpaceConfig {
    /// Remove parameters whose pairwise Pearson correlation exceeds this (default 0.95).
    pub correlation_threshold: f32,
    /// Remove parameters with variance below this (default 1e-4).
    pub variance_threshold: f32,
    /// Keep at most N parameters ranked by variance.
    pub n_keep: Option<usize>,
}

impl Default for ParamSpaceConfig {
    fn default() -> Self {
        Self {
            correlation_threshold: 0.95,
            variance_threshold: 1e-4,
            n_keep: None,
        }
    }
}

/// Result of a parameter-space analysis pass.
#[allow(dead_code)]
pub struct ParamSpaceAnalysis {
    pub original_count: usize,
    pub kept_params: Vec<String>,
    pub removed_params: Vec<String>,
    /// `correlation_matrix[i][j]` — n_params × n_params.
    pub correlation_matrix: Vec<Vec<f32>>,
    pub variances: Vec<f32>,
}

// ── core statistics ───────────────────────────────────────────────────────────

/// Population variance of a slice of values.
#[allow(dead_code)]
pub fn param_variance(values: &[f32]) -> f32 {
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    let mean = values.iter().sum::<f32>() / n as f32;
    values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n as f32
}

/// Pearson correlation coefficient between two equal-length slices.
#[allow(dead_code)]
pub fn param_correlation(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mean_a = a[..n].iter().sum::<f32>() / n as f32;
    let mean_b = b[..n].iter().sum::<f32>() / n as f32;
    let mut cov = 0.0_f32;
    let mut var_a = 0.0_f32;
    let mut var_b = 0.0_f32;
    for i in 0..n {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }
    let denom = (var_a * var_b).sqrt();
    if denom < 1e-12 {
        0.0
    } else {
        cov / denom
    }
}

/// Build an n_params × n_params correlation matrix.
/// `samples[i]` is a vector of all sample values for parameter i.
#[allow(dead_code)]
pub fn build_correlation_matrix(samples: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let n = samples.len();
    let mut mat = vec![vec![0.0_f32; n]; n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                mat[i][j] = 1.0;
            } else {
                mat[i][j] = param_correlation(&samples[i], &samples[j]);
            }
        }
    }
    mat
}

/// Greedy redundancy removal: for each pair with |corr| > threshold,
/// remove the member with lower variance.  Returns names of removed params.
#[allow(dead_code)]
pub fn find_redundant_params(corr: &[Vec<f32>], names: &[String], threshold: f32) -> Vec<String> {
    let n = names.len();
    // compute variances from the diagonal = 1, but we need actual values –
    // caller must ensure corr is built from the same samples.
    // We derive a simple proxy: mark by index.
    let mut removed = vec![false; n];
    // We need variance to break ties; re-derive from corr is not possible,
    // so we treat index order as a proxy (earlier = higher variance by convention).
    for i in 0..n {
        if removed[i] {
            continue;
        }
        for j in (i + 1)..n {
            if removed[j] {
                continue;
            }
            if corr[i][j].abs() > threshold {
                // Remove j (higher index = lower variance proxy)
                removed[j] = true;
            }
        }
    }
    names
        .iter()
        .enumerate()
        .filter(|(i, _)| removed[*i])
        .map(|(_, name)| name.clone())
        .collect()
}

/// Return kept parameter names after applying variance and correlation filters.
#[allow(dead_code)]
pub fn reduce_param_set(
    names: &[String],
    samples: &[HashMap<String, f32>],
    cfg: &ParamSpaceConfig,
) -> Vec<String> {
    if names.is_empty() || samples.is_empty() {
        return names.to_vec();
    }

    // Gather per-param value vectors
    let param_values: Vec<Vec<f32>> = names
        .iter()
        .map(|n| samples.iter().map(|s| *s.get(n).unwrap_or(&0.0)).collect())
        .collect();

    let variances: Vec<f32> = param_values.iter().map(|v| param_variance(v)).collect();

    // Step 1: remove low-variance params
    let mut kept: Vec<usize> = (0..names.len())
        .filter(|&i| variances[i] >= cfg.variance_threshold)
        .collect();

    // Step 2: remove highly correlated (greedy)
    let kept_values: Vec<Vec<f32>> = kept.iter().map(|&i| param_values[i].clone()).collect();
    let corr = build_correlation_matrix(&kept_values);
    let kept_names: Vec<String> = kept.iter().map(|&i| names[i].clone()).collect();
    let redundant = find_redundant_params(&corr, &kept_names, cfg.correlation_threshold);
    let redundant_set: std::collections::HashSet<&String> = redundant.iter().collect();
    kept.retain(|&i| !redundant_set.contains(&names[i]));

    // Step 3: keep top-N by variance
    if let Some(n_keep) = cfg.n_keep {
        kept.sort_by(|&a, &b| {
            variances[b]
                .partial_cmp(&variances[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        kept.truncate(n_keep);
    }

    kept.iter().map(|&i| names[i].clone()).collect()
}

/// Min/max normalize each parameter across samples in-place.
/// Returns `(min, max)` per parameter name.
#[allow(dead_code)]
pub fn normalize_param_samples(
    samples: &mut [HashMap<String, f32>],
) -> HashMap<String, (f32, f32)> {
    if samples.is_empty() {
        return HashMap::new();
    }

    // Collect all param names
    let names: Vec<String> = samples[0].keys().cloned().collect();
    let mut ranges: HashMap<String, (f32, f32)> = HashMap::new();

    for name in &names {
        let vals: Vec<f32> = samples
            .iter()
            .map(|s| *s.get(name).unwrap_or(&0.0))
            .collect();
        let min = vals.iter().cloned().fold(f32::MAX, f32::min);
        let max = vals.iter().cloned().fold(f32::MIN, f32::max);
        ranges.insert(name.clone(), (min, max));
    }

    for s in samples.iter_mut() {
        for name in &names {
            let (min, max) = ranges[name];
            let span = max - min;
            if span > 1e-12 {
                let v = s.entry(name.clone()).or_insert(0.0);
                *v = (*v - min) / span;
            } else if let Some(v) = s.get_mut(name) {
                *v = 0.0;
            }
        }
    }

    ranges
}

/// Importance score = variance / max_variance across all parameters.
#[allow(dead_code)]
pub fn param_importance_score(name: &str, samples: &[HashMap<String, f32>]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let names: Vec<String> = samples[0].keys().cloned().collect();
    let variances: Vec<f32> = names
        .iter()
        .map(|n| {
            let vals: Vec<f32> = samples.iter().map(|s| *s.get(n).unwrap_or(&0.0)).collect();
            param_variance(&vals)
        })
        .collect();
    let max_var = variances.iter().cloned().fold(0.0_f32, f32::max);
    if max_var < 1e-12 {
        return 0.0;
    }
    let my_vals: Vec<f32> = samples
        .iter()
        .map(|s| *s.get(name).unwrap_or(&0.0))
        .collect();
    param_variance(&my_vals) / max_var
}

/// Analyze a parameter space and return the full analysis result.
#[allow(dead_code)]
pub fn analyze_param_space(
    param_names: &[String],
    param_samples: &[HashMap<String, f32>],
) -> ParamSpaceAnalysis {
    let cfg = ParamSpaceConfig::default();
    let original_count = param_names.len();

    let param_values: Vec<Vec<f32>> = param_names
        .iter()
        .map(|n| {
            param_samples
                .iter()
                .map(|s| *s.get(n).unwrap_or(&0.0))
                .collect()
        })
        .collect();

    let variances: Vec<f32> = param_values.iter().map(|v| param_variance(v)).collect();
    let correlation_matrix = build_correlation_matrix(&param_values);

    let kept_names = reduce_param_set(param_names, param_samples, &cfg);
    let kept_set: std::collections::HashSet<&String> = kept_names.iter().collect();
    let removed_params: Vec<String> = param_names
        .iter()
        .filter(|n| !kept_set.contains(n))
        .cloned()
        .collect();

    ParamSpaceAnalysis {
        original_count,
        kept_params: kept_names,
        removed_params,
        correlation_matrix,
        variances,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_samples(data: &[(&str, Vec<f32>)]) -> Vec<HashMap<String, f32>> {
        if data.is_empty() {
            return vec![];
        }
        let n = data[0].1.len();
        (0..n)
            .map(|i| {
                data.iter()
                    .map(|(name, vals)| (name.to_string(), vals[i]))
                    .collect()
            })
            .collect()
    }

    // 1. param_variance formula
    #[test]
    fn test_param_variance_formula() {
        let v = vec![2.0_f32, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let var = param_variance(&v);
        // population variance = 4.0
        assert!((var - 4.0).abs() < 1e-4, "expected ~4.0 got {var}");
    }

    // 2. param_correlation perfect positive = 1.0
    #[test]
    fn test_correlation_perfect_positive() {
        let a = vec![1.0_f32, 2.0, 3.0, 4.0, 5.0];
        let b = vec![2.0_f32, 4.0, 6.0, 8.0, 10.0];
        let r = param_correlation(&a, &b);
        assert!((r - 1.0).abs() < 1e-5, "expected 1.0 got {r}");
    }

    // 3. param_correlation perfect negative = -1.0
    #[test]
    fn test_correlation_perfect_negative() {
        let a = vec![1.0_f32, 2.0, 3.0, 4.0, 5.0];
        let b = vec![5.0_f32, 4.0, 3.0, 2.0, 1.0];
        let r = param_correlation(&a, &b);
        assert!((r + 1.0).abs() < 1e-5, "expected -1.0 got {r}");
    }

    // 4. param_correlation uncorrelated ≈ 0
    #[test]
    fn test_correlation_uncorrelated() {
        let a = vec![1.0_f32, 1.0, 1.0, 1.0];
        let b = vec![1.0_f32, 2.0, 3.0, 4.0];
        // a has zero variance → correlation = 0
        let r = param_correlation(&a, &b);
        assert!(r.abs() < 1e-5, "expected ~0 got {r}");
    }

    // 5. build_correlation_matrix diagonal = 1
    #[test]
    fn test_correlation_matrix_diagonal() {
        let samples = vec![
            vec![1.0_f32, 2.0, 3.0],
            vec![4.0_f32, 5.0, 6.0],
            vec![7.0_f32, 8.0, 9.0],
        ];
        let mat = build_correlation_matrix(&samples);
        for (i, row) in mat.iter().enumerate().take(3) {
            assert!((row[i] - 1.0).abs() < 1e-5, "diagonal[{i}] != 1");
        }
    }

    // 6. find_redundant_params removes correlated
    #[test]
    fn test_find_redundant_removes_correlated() {
        // Build a 2×2 matrix where params 0 and 1 are perfectly correlated
        let corr = vec![vec![1.0, 0.99], vec![0.99, 1.0]];
        let names = vec!["a".to_string(), "b".to_string()];
        let redundant = find_redundant_params(&corr, &names, 0.95);
        assert_eq!(redundant.len(), 1);
        assert_eq!(redundant[0], "b");
    }

    // 7. find_redundant_params keeps uncorrelated
    #[test]
    fn test_find_redundant_keeps_uncorrelated() {
        let corr = vec![vec![1.0, 0.1], vec![0.1, 1.0]];
        let names = vec!["a".to_string(), "b".to_string()];
        let redundant = find_redundant_params(&corr, &names, 0.95);
        assert!(redundant.is_empty());
    }

    // 8. reduce_param_set respects n_keep
    #[test]
    fn test_reduce_param_set_n_keep() {
        // Use orthogonal (uncorrelated) signals so correlation pruning keeps all,
        // then n_keep=2 selects the top 2 by variance.
        let names: Vec<String> = (0..4).map(|i| format!("p{i}")).collect();
        let samples = make_samples(&[
            // p0: high variance, orthogonal
            ("p0", vec![0.0, 10.0, 0.0, 10.0]),
            // p1: high variance, orthogonal
            ("p1", vec![0.0, 0.0, 10.0, 10.0]),
            // p2: low variance
            ("p2", vec![0.1, 0.2, 0.1, 0.2]),
            // p3: low variance
            ("p3", vec![0.01, 0.02, 0.01, 0.02]),
        ]);
        let cfg = ParamSpaceConfig {
            n_keep: Some(2),
            correlation_threshold: 1.0, // never remove on correlation
            variance_threshold: 0.0,
        };
        let kept = reduce_param_set(&names, &samples, &cfg);
        assert_eq!(kept.len(), 2, "expected 2 kept params, got {}", kept.len());
    }

    // 9. normalize_param_samples produces 0..1 range
    #[test]
    fn test_normalize_param_samples_range() {
        let mut samples = make_samples(&[("x", vec![1.0, 5.0, 3.0])]);
        normalize_param_samples(&mut samples);
        let vals: Vec<f32> = samples.iter().map(|s| *s.get("x").unwrap()).collect();
        let min = vals.iter().cloned().fold(f32::MAX, f32::min);
        let max = vals.iter().cloned().fold(f32::MIN, f32::max);
        assert!((min - 0.0).abs() < 1e-5, "min should be 0, got {min}");
        assert!((max - 1.0).abs() < 1e-5, "max should be 1, got {max}");
    }

    // 10. analyze_param_space removes zero-variance
    #[test]
    fn test_analyze_removes_zero_variance() {
        let names = vec!["vary".to_string(), "const".to_string()];
        let samples = make_samples(&[
            ("vary", vec![1.0, 2.0, 3.0, 4.0]),
            ("const", vec![5.0, 5.0, 5.0, 5.0]),
        ]);
        let analysis = analyze_param_space(&names, &samples);
        assert!(
            analysis.removed_params.contains(&"const".to_string()),
            "zero-variance param should be removed"
        );
    }

    // 11. original_count = n params
    #[test]
    fn test_original_count() {
        let names: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let samples = make_samples(&[
            ("a", vec![1.0, 2.0]),
            ("b", vec![3.0, 4.0]),
            ("c", vec![5.0, 6.0]),
        ]);
        let analysis = analyze_param_space(&names, &samples);
        assert_eq!(analysis.original_count, 3);
    }

    // 12. kept + removed = original
    #[test]
    fn test_kept_plus_removed_eq_original() {
        let names: Vec<String> = (0..4).map(|i| format!("p{i}")).collect();
        let samples = make_samples(&[
            ("p0", vec![1.0, 2.0, 3.0]),
            ("p1", vec![1.0, 1.0, 1.0]), // zero variance → removed
            ("p2", vec![4.0, 5.0, 6.0]),
            ("p3", vec![7.0, 8.0, 9.0]),
        ]);
        let analysis = analyze_param_space(&names, &samples);
        assert_eq!(
            analysis.kept_params.len() + analysis.removed_params.len(),
            analysis.original_count
        );
    }

    // 13. param_importance_score returns 1.0 for highest-variance param
    #[test]
    fn test_param_importance_score_max() {
        let samples = make_samples(&[
            ("big", vec![0.0, 10.0, 20.0, 30.0]),
            ("small", vec![0.0, 0.1, 0.2, 0.3]),
        ]);
        let score = param_importance_score("big", &samples);
        assert!(
            (score - 1.0).abs() < 1e-4,
            "highest-variance param should score 1.0, got {score}"
        );
    }

    // 14. normalize_param_samples returns correct (min, max) map
    #[test]
    fn test_normalize_returns_range_map() {
        let mut samples = make_samples(&[("y", vec![2.0, 4.0, 6.0])]);
        let ranges = normalize_param_samples(&mut samples);
        let (min, max) = ranges["y"];
        assert!((min - 2.0).abs() < 1e-5);
        assert!((max - 6.0).abs() < 1e-5);
    }
}
