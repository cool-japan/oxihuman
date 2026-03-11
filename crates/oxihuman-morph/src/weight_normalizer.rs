//! Weight normalizer — normalize morph weight vectors so they sum to a target value,
//! with clamping, pruning, and redistribution utilities.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightNormConfig {
    pub target_sum: f32,
    pub clamp_min: f32,
    pub clamp_max: f32,
    pub prune_threshold: f32,
    pub epsilon: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightNormResult {
    pub weights: Vec<f32>,
    pub original_sum: f32,
    pub final_sum: f32,
    pub pruned_count: usize,
    pub clamped_count: usize,
}

#[allow(dead_code)]
pub fn default_weight_norm_config() -> WeightNormConfig {
    WeightNormConfig {
        target_sum: 1.0,
        clamp_min: 0.0,
        clamp_max: 1.0,
        prune_threshold: 1e-5,
        epsilon: 1e-9,
    }
}

#[allow(dead_code)]
pub fn normalize_weights(weights: &[f32], cfg: &WeightNormConfig) -> WeightNormResult {
    normalize_weights_to_sum(weights, cfg.target_sum, cfg)
}

#[allow(dead_code)]
pub fn normalize_weights_to_sum(
    weights: &[f32],
    target: f32,
    cfg: &WeightNormConfig,
) -> WeightNormResult {
    let original_sum: f32 = weights.iter().copied().sum();

    // First clamp
    let mut clamped_count = 0usize;
    let mut clamped: Vec<f32> = weights
        .iter()
        .map(|&w| {
            let c = w.clamp(cfg.clamp_min, cfg.clamp_max);
            if (c - w).abs() > cfg.epsilon {
                clamped_count += 1;
            }
            c
        })
        .collect();

    let sum_after_clamp: f32 = clamped.iter().copied().sum();

    // Redistribute to target sum
    if sum_after_clamp.abs() > cfg.epsilon {
        let scale = target / sum_after_clamp;
        for w in &mut clamped {
            *w *= scale;
        }
    } else if !clamped.is_empty() {
        let uniform = target / clamped.len() as f32;
        clamped.fill(uniform);
    }

    let final_sum: f32 = clamped.iter().copied().sum();

    WeightNormResult {
        weights: clamped,
        original_sum,
        final_sum,
        pruned_count: 0,
        clamped_count,
    }
}

#[allow(dead_code)]
pub fn weight_norm_sum(weights: &[f32]) -> f32 {
    weights.iter().copied().sum()
}

#[allow(dead_code)]
pub fn weight_norm_max_weight(weights: &[f32]) -> f32 {
    weights.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn weight_norm_min_weight(weights: &[f32]) -> f32 {
    weights.iter().cloned().fold(f32::INFINITY, f32::min)
}

#[allow(dead_code)]
pub fn weight_norm_clamp(weights: &[f32], lo: f32, hi: f32) -> Vec<f32> {
    weights.iter().map(|&w| w.clamp(lo, hi)).collect()
}

#[allow(dead_code)]
pub fn weight_norm_prune(weights: &[f32], threshold: f32) -> WeightNormResult {
    let original_sum: f32 = weights.iter().copied().sum();
    let mut pruned_count = 0usize;
    let pruned: Vec<f32> = weights
        .iter()
        .map(|&w| {
            if w.abs() < threshold {
                pruned_count += 1;
                0.0
            } else {
                w
            }
        })
        .collect();
    let final_sum: f32 = pruned.iter().copied().sum();
    WeightNormResult {
        weights: pruned,
        original_sum,
        final_sum,
        pruned_count,
        clamped_count: 0,
    }
}

#[allow(dead_code)]
pub fn weight_norm_to_json(result: &WeightNormResult) -> String {
    let ws: Vec<String> = result.weights.iter().map(|w| format!("{w:.6}")).collect();
    format!(
        "{{\"weights\":[{}],\"original_sum\":{:.6},\"final_sum\":{:.6},\"pruned_count\":{},\"clamped_count\":{}}}",
        ws.join(","),
        result.original_sum,
        result.final_sum,
        result.pruned_count,
        result.clamped_count,
    )
}

#[allow(dead_code)]
pub fn weight_norm_is_normalized(weights: &[f32], target: f32, epsilon: f32) -> bool {
    let sum: f32 = weights.iter().copied().sum();
    (sum - target).abs() < epsilon
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> WeightNormConfig {
        default_weight_norm_config()
    }

    #[test]
    fn test_default_config() {
        let c = cfg();
        assert!((c.target_sum - 1.0).abs() < 1e-9);
        assert!((c.clamp_min - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_normalize_uniform() {
        let w = vec![0.25, 0.25, 0.25, 0.25];
        let r = normalize_weights(&w, &cfg());
        assert!(weight_norm_is_normalized(&r.weights, 1.0, 1e-5));
    }

    #[test]
    fn test_normalize_non_unit_sum() {
        let w = vec![2.0, 3.0, 5.0];
        let r = normalize_weights(&w, &cfg());
        assert!(weight_norm_is_normalized(&r.weights, 1.0, 1e-5));
    }

    #[test]
    fn test_normalize_to_custom_sum() {
        let w = vec![1.0, 2.0, 3.0];
        let r = normalize_weights_to_sum(&w, 6.0, &cfg());
        assert!((r.final_sum - 6.0).abs() < 1e-4);
    }

    #[test]
    fn test_weight_norm_sum() {
        let w = vec![0.1, 0.2, 0.3, 0.4];
        assert!((weight_norm_sum(&w) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_min_weight() {
        let w = vec![0.1, 0.9, 0.4];
        assert!((weight_norm_max_weight(&w) - 0.9).abs() < 1e-6);
        assert!((weight_norm_min_weight(&w) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        let w = vec![-0.5, 0.5, 1.5];
        let c = weight_norm_clamp(&w, 0.0, 1.0);
        assert_eq!(c[0], 0.0);
        assert_eq!(c[1], 0.5);
        assert_eq!(c[2], 1.0);
    }

    #[test]
    fn test_prune() {
        let w = vec![0.0001, 0.5, 0.000001, 0.3];
        let r = weight_norm_prune(&w, 1e-3);
        assert_eq!(r.pruned_count, 2);
        assert_eq!(r.weights[0], 0.0);
        assert_eq!(r.weights[2], 0.0);
    }

    #[test]
    fn test_to_json_contains_weights() {
        let r = WeightNormResult {
            weights: vec![0.5, 0.5],
            original_sum: 1.0,
            final_sum: 1.0,
            pruned_count: 0,
            clamped_count: 0,
        };
        let j = weight_norm_to_json(&r);
        assert!(j.contains("weights"));
        assert!(j.contains("original_sum"));
    }

    #[test]
    fn test_is_normalized_false() {
        let w = vec![0.3, 0.3, 0.3];
        assert!(!weight_norm_is_normalized(&w, 1.0, 1e-5));
    }

    #[test]
    fn test_normalize_all_zeros_uniform_redistribution() {
        let w = vec![0.0, 0.0, 0.0];
        let mut c = cfg();
        c.clamp_min = 0.0;
        let r = normalize_weights(&w, &c);
        // All zeros sum → uniform distribution
        assert!(weight_norm_is_normalized(&r.weights, 1.0, 1e-5));
    }
}
