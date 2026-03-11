#![allow(dead_code)]
//! Expression sampler: samples expression weights at a fixed rate.

/// A sampler that stores expression weight samples.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionSampler {
    rate: f32,
    samples: Vec<f32>,
}

/// Create a new sampler at the given rate (samples per second).
#[allow(dead_code)]
pub fn new_expression_sampler(rate: f32) -> ExpressionSampler {
    ExpressionSampler {
        rate: rate.max(1.0),
        samples: Vec::new(),
    }
}

/// Record a sample.
#[allow(dead_code)]
pub fn sample_expression(sampler: &mut ExpressionSampler, weight: f32) {
    sampler.samples.push(weight);
}

/// Return the sample rate.
#[allow(dead_code)]
pub fn sampler_rate(sampler: &ExpressionSampler) -> f32 {
    sampler.rate
}

/// Return the number of samples.
#[allow(dead_code)]
pub fn sampler_count(sampler: &ExpressionSampler) -> usize {
    sampler.samples.len()
}

/// Return the sample at `index`.
#[allow(dead_code)]
pub fn sampler_at(sampler: &ExpressionSampler, index: usize) -> f32 {
    sampler.samples.get(index).copied().unwrap_or(0.0)
}

/// Return the total duration based on sample count and rate.
#[allow(dead_code)]
pub fn sampler_duration_es(sampler: &ExpressionSampler) -> f32 {
    if sampler.samples.is_empty() {
        return 0.0;
    }
    (sampler.samples.len() as f32 - 1.0) / sampler.rate
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn sampler_to_json(sampler: &ExpressionSampler) -> String {
    let samples_str: Vec<String> = sampler.samples.iter().map(|s| format!("{s}")).collect();
    format!(
        "{{\"rate\":{},\"samples\":[{}]}}",
        sampler.rate,
        samples_str.join(",")
    )
}

/// Clear all samples.
#[allow(dead_code)]
pub fn sampler_clear(sampler: &mut ExpressionSampler) {
    sampler.samples.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sampler() {
        let s = new_expression_sampler(30.0);
        assert!((sampler_rate(&s) - 30.0).abs() < 1e-6);
        assert_eq!(sampler_count(&s), 0);
    }

    #[test]
    fn test_sample() {
        let mut s = new_expression_sampler(30.0);
        sample_expression(&mut s, 0.5);
        assert_eq!(sampler_count(&s), 1);
    }

    #[test]
    fn test_sampler_at() {
        let mut s = new_expression_sampler(30.0);
        sample_expression(&mut s, 0.7);
        assert!((sampler_at(&s, 0) - 0.7).abs() < 1e-6);
        assert!((sampler_at(&s, 99) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_duration_empty() {
        let s = new_expression_sampler(30.0);
        assert!((sampler_duration_es(&s) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_duration() {
        let mut s = new_expression_sampler(10.0);
        for i in 0..11 {
            sample_expression(&mut s, i as f32 * 0.1);
        }
        assert!((sampler_duration_es(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_expression_sampler(30.0);
        let json = sampler_to_json(&s);
        assert!(json.contains("\"rate\":30"));
    }

    #[test]
    fn test_clear() {
        let mut s = new_expression_sampler(30.0);
        sample_expression(&mut s, 0.5);
        sampler_clear(&mut s);
        assert_eq!(sampler_count(&s), 0);
    }

    #[test]
    fn test_min_rate() {
        let s = new_expression_sampler(0.1);
        assert!((sampler_rate(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_samples() {
        let mut s = new_expression_sampler(30.0);
        sample_expression(&mut s, 0.1);
        sample_expression(&mut s, 0.2);
        sample_expression(&mut s, 0.3);
        assert_eq!(sampler_count(&s), 3);
        assert!((sampler_at(&s, 2) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_sampler_rate_preserved() {
        let s = new_expression_sampler(60.0);
        assert!((sampler_rate(&s) - 60.0).abs() < 1e-6);
    }
}
