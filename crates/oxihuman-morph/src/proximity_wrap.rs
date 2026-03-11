// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Proximity-based surface wrap deformer stub.

/// Configuration for the proximity wrap deformer.
#[derive(Debug, Clone)]
pub struct ProximityWrapConfig {
    /// Maximum influence distance.
    pub max_distance: f32,
    /// Falloff exponent.
    pub falloff: f32,
}

impl Default for ProximityWrapConfig {
    fn default() -> Self {
        ProximityWrapConfig {
            max_distance: 0.5,
            falloff: 2.0,
        }
    }
}

/// A proximity wrap binding between a driver mesh and a target mesh.
#[derive(Debug, Clone)]
pub struct ProximityWrap {
    pub config: ProximityWrapConfig,
    /// Influence weights per vertex.
    pub weights: Vec<f32>,
}

impl ProximityWrap {
    pub fn new(vertex_count: usize) -> Self {
        ProximityWrap {
            config: ProximityWrapConfig::default(),
            weights: vec![0.0; vertex_count],
        }
    }
}

/// Create a new proximity wrap for the given vertex count.
pub fn new_proximity_wrap(vertex_count: usize) -> ProximityWrap {
    ProximityWrap::new(vertex_count)
}

/// Compute influence weight based on distance.
pub fn proximity_influence(distance: f32, config: &ProximityWrapConfig) -> f32 {
    if distance >= config.max_distance {
        return 0.0;
    }
    let t = 1.0 - distance / config.max_distance;
    t.powf(config.falloff)
}

/// Bake weights for all vertices given their distances to the driver surface.
#[allow(clippy::needless_range_loop)]
pub fn bake_proximity_weights(wrap: &mut ProximityWrap, distances: &[f32]) {
    let n = wrap.weights.len().min(distances.len());
    for i in 0..n {
        wrap.weights[i] = proximity_influence(distances[i], &wrap.config);
    }
}

/// Return the number of vertices.
pub fn proximity_vertex_count(wrap: &ProximityWrap) -> usize {
    wrap.weights.len()
}

/// Return a JSON-like string for the wrap state.
pub fn proximity_wrap_to_json(wrap: &ProximityWrap) -> String {
    format!(
        r#"{{"max_distance":{:.4},"falloff":{:.4},"vertices":{}}}"#,
        wrap.config.max_distance,
        wrap.config.falloff,
        wrap.weights.len()
    )
}

/// Return the average weight across all vertices.
pub fn proximity_average_weight(wrap: &ProximityWrap) -> f32 {
    if wrap.weights.is_empty() {
        return 0.0;
    }
    let sum: f32 = wrap.weights.iter().sum();
    sum / wrap.weights.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wrap_correct_vertex_count() {
        let w = new_proximity_wrap(10);
        assert_eq!(
            proximity_vertex_count(&w),
            10, /* vertex count must match */
        );
    }

    #[test]
    fn test_influence_at_zero_distance_is_one() {
        let cfg = ProximityWrapConfig::default();
        let inf = proximity_influence(0.0, &cfg);
        assert!((inf - 1.0).abs() < 1e-5, /* influence at zero distance should be 1 */);
    }

    #[test]
    fn test_influence_at_max_distance_is_zero() {
        let cfg = ProximityWrapConfig::default();
        let inf = proximity_influence(cfg.max_distance, &cfg);
        assert!((inf).abs() < 1e-5, /* influence at max distance should be 0 */);
    }

    #[test]
    fn test_influence_beyond_max_is_zero() {
        let cfg = ProximityWrapConfig::default();
        let inf = proximity_influence(cfg.max_distance + 1.0, &cfg);
        assert!((inf).abs() < 1e-5, /* influence beyond max distance must be 0 */);
    }

    #[test]
    fn test_bake_weights_fills_correctly() {
        let mut w = new_proximity_wrap(3);
        bake_proximity_weights(&mut w, &[0.0, 0.25, 1.0]);
        assert!(w.weights[0] > 0.0, /* near vertex should have positive weight */);
        assert!((w.weights[2]).abs() < 1e-5, /* far vertex should have zero weight */);
    }

    #[test]
    fn test_average_weight_empty() {
        let w = new_proximity_wrap(0);
        assert!((proximity_average_weight(&w)).abs() < 1e-6, /* empty gives 0 average */);
    }

    #[test]
    fn test_average_weight_all_zero() {
        let w = new_proximity_wrap(5);
        assert!((proximity_average_weight(&w)).abs() < 1e-6, /* all-zero average is 0 */);
    }

    #[test]
    fn test_to_json_contains_max_distance() {
        let w = new_proximity_wrap(4);
        let j = proximity_wrap_to_json(&w);
        assert!(j.contains("max_distance"), /* JSON should contain max_distance key */);
    }

    #[test]
    fn test_falloff_default_is_two() {
        let w = new_proximity_wrap(1);
        assert!((w.config.falloff - 2.0).abs() < 1e-5, /* default falloff is 2.0 */);
    }

    #[test]
    fn test_weights_initialized_zero() {
        let w = new_proximity_wrap(8);
        for &wt in &w.weights {
            assert!((wt).abs() < 1e-6 /* initial weights must be zero */,);
        }
    }
}
