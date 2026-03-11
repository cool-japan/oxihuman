#![allow(dead_code)]
//! Statistics computation for morph weight sets.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MorphStats {
    pub active: usize,
    pub total: usize,
    pub max: f32,
    pub min: f32,
    pub avg: f32,
}

#[allow(dead_code)]
pub fn compute_morph_stats(weights: &[f32]) -> MorphStats {
    if weights.is_empty() {
        return MorphStats {
            active: 0,
            total: 0,
            max: 0.0,
            min: 0.0,
            avg: 0.0,
        };
    }
    let active = weights.iter().filter(|w| w.abs() > 1e-6).count();
    let max = weights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let min = weights.iter().copied().fold(f32::INFINITY, f32::min);
    let avg = weights.iter().sum::<f32>() / weights.len() as f32;
    MorphStats {
        active,
        total: weights.len(),
        max,
        min,
        avg,
    }
}

#[allow(dead_code)]
pub fn active_morph_count(stats: &MorphStats) -> usize {
    stats.active
}

#[allow(dead_code)]
pub fn total_morph_count(stats: &MorphStats) -> usize {
    stats.total
}

#[allow(dead_code)]
pub fn max_weight(stats: &MorphStats) -> f32 {
    stats.max
}

#[allow(dead_code)]
pub fn min_weight(stats: &MorphStats) -> f32 {
    stats.min
}

#[allow(dead_code)]
pub fn average_weight(stats: &MorphStats) -> f32 {
    stats.avg
}

#[allow(dead_code)]
pub fn stats_to_json(stats: &MorphStats) -> String {
    format!(
        "{{\"active\":{},\"total\":{},\"max\":{},\"min\":{},\"avg\":{}}}",
        stats.active, stats.total, stats.max, stats.min, stats.avg
    )
}

#[allow(dead_code)]
pub fn stats_summary(stats: &MorphStats) -> String {
    format!(
        "{}/{} active, range [{}, {}], avg {}",
        stats.active, stats.total, stats.min, stats.max, stats.avg
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_morph_stats_empty() {
        let s = compute_morph_stats(&[]);
        assert_eq!(s.total, 0);
    }

    #[test]
    fn test_compute_morph_stats() {
        let s = compute_morph_stats(&[0.0, 0.5, 1.0]);
        assert_eq!(s.total, 3);
        assert_eq!(s.active, 2);
    }

    #[test]
    fn test_active_morph_count() {
        let s = compute_morph_stats(&[0.0, 0.1, 0.0, 0.9]);
        assert_eq!(active_morph_count(&s), 2);
    }

    #[test]
    fn test_total_morph_count() {
        let s = compute_morph_stats(&[0.1, 0.2, 0.3]);
        assert_eq!(total_morph_count(&s), 3);
    }

    #[test]
    fn test_max_weight() {
        let s = compute_morph_stats(&[0.1, 0.9, 0.5]);
        assert!((max_weight(&s) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_min_weight() {
        let s = compute_morph_stats(&[0.3, 0.1, 0.5]);
        assert!((min_weight(&s) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_average_weight() {
        let s = compute_morph_stats(&[0.0, 0.5, 1.0]);
        assert!((average_weight(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_stats_to_json() {
        let s = compute_morph_stats(&[0.5]);
        let json = stats_to_json(&s);
        assert!(json.contains("\"active\":1"));
    }

    #[test]
    fn test_stats_summary() {
        let s = compute_morph_stats(&[0.5]);
        let summary = stats_summary(&s);
        assert!(summary.contains("active"));
    }

    #[test]
    fn test_all_zero() {
        let s = compute_morph_stats(&[0.0, 0.0, 0.0]);
        assert_eq!(active_morph_count(&s), 0);
    }
}
