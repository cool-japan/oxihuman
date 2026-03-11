//! Spatial weight map that drives morph influence by vertex position.
//!
//! A [`BlendWeightMap`] holds one or more [`WeightMapGradient`]s. Each gradient
//! defines a spatial falloff between two 3-D positions. The final weight at any
//! vertex is the sum of individual gradient contributions, optionally normalized
//! to [0, 1].

/// Configuration for a weight map.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightMapConfig {
    /// Maximum weight value allowed before normalization.
    pub max_weight: f32,
    /// Whether to clamp individual gradient outputs to [0, 1].
    pub clamp_gradients: bool,
}

/// A single spatial gradient contributing to the weight map.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightMapGradient {
    /// Origin position (full weight, value = `peak`).
    pub origin: [f32; 3],
    /// Radius over which the weight falls off to zero.
    pub radius: f32,
    /// Weight at the origin.
    pub peak: f32,
    /// Human-readable label.
    pub name: String,
}

/// A spatial blend-weight map composed of multiple gradients.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendWeightMap {
    config: WeightMapConfig,
    gradients: Vec<WeightMapGradient>,
}

/// Return a default [`WeightMapConfig`].
#[allow(dead_code)]
pub fn default_weight_map_config() -> WeightMapConfig {
    WeightMapConfig {
        max_weight: 1.0,
        clamp_gradients: true,
    }
}

/// Create a new [`BlendWeightMap`].
#[allow(dead_code)]
pub fn new_blend_weight_map(config: WeightMapConfig) -> BlendWeightMap {
    BlendWeightMap {
        config,
        gradients: Vec::new(),
    }
}

/// Append a gradient to the map.
#[allow(dead_code)]
pub fn weight_map_add_gradient(map: &mut BlendWeightMap, gradient: WeightMapGradient) {
    map.gradients.push(gradient);
}

/// Evaluate the total weight at position `pos`.
/// Each gradient contributes a spherical falloff; contributions are summed.
#[allow(dead_code)]
pub fn weight_map_evaluate(map: &BlendWeightMap, pos: [f32; 3]) -> f32 {
    let mut total = 0.0f32;
    for g in &map.gradients {
        let dx = pos[0] - g.origin[0];
        let dy = pos[1] - g.origin[1];
        let dz = pos[2] - g.origin[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if g.radius <= 0.0 {
            continue;
        }
        let t = (1.0 - (dist / g.radius)).clamp(0.0, 1.0);
        let contrib = g.peak * t;
        total += if map.config.clamp_gradients {
            contrib.clamp(0.0, map.config.max_weight)
        } else {
            contrib
        };
    }
    total
}

/// Return the number of gradients in the map.
#[allow(dead_code)]
pub fn weight_map_gradient_count(map: &BlendWeightMap) -> usize {
    map.gradients.len()
}

/// Remove all gradients.
#[allow(dead_code)]
pub fn weight_map_clear(map: &mut BlendWeightMap) {
    map.gradients.clear();
}

/// Serialize the map to a compact JSON string.
#[allow(dead_code)]
pub fn weight_map_to_json(map: &BlendWeightMap) -> String {
    let grads: Vec<String> = map
        .gradients
        .iter()
        .map(|g| {
            format!(
                r#"{{"name":"{}","peak":{:.4},"radius":{:.4}}}"#,
                g.name, g.peak, g.radius
            )
        })
        .collect();
    format!(
        r#"{{"gradient_count":{},"max_weight":{:.4},"gradients":[{}]}}"#,
        map.gradients.len(),
        map.config.max_weight,
        grads.join(",")
    )
}

/// Return the maximum weight across a set of sample positions.
#[allow(dead_code)]
pub fn weight_map_max_weight(map: &BlendWeightMap, samples: &[[f32; 3]]) -> f32 {
    samples
        .iter()
        .map(|&p| weight_map_evaluate(map, p))
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Normalize all gradient peak values so the map's maximum over `samples` equals 1.0.
/// No-op if the current maximum is zero or if `samples` is empty.
#[allow(dead_code)]
pub fn weight_map_normalize(map: &mut BlendWeightMap, samples: &[[f32; 3]]) {
    let mx = weight_map_max_weight(map, samples);
    if mx.abs() < 1e-9 || samples.is_empty() {
        return;
    }
    for g in &mut map.gradients {
        g.peak /= mx;
    }
}

/// Invert all gradient peak values: `peak = max_weight - peak`.
#[allow(dead_code)]
pub fn weight_map_invert(map: &mut BlendWeightMap) {
    let cap = map.config.max_weight;
    for g in &mut map.gradients {
        g.peak = (cap - g.peak).clamp(0.0, cap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_map() -> BlendWeightMap {
        let mut m = new_blend_weight_map(default_weight_map_config());
        weight_map_add_gradient(
            &mut m,
            WeightMapGradient {
                origin: [0.0, 0.0, 0.0],
                radius: 1.0,
                peak: 1.0,
                name: "center".into(),
            },
        );
        m
    }

    #[test]
    fn test_default_config_values() {
        let cfg = default_weight_map_config();
        assert!((cfg.max_weight - 1.0).abs() < 1e-6);
        assert!(cfg.clamp_gradients);
    }

    #[test]
    fn test_gradient_count() {
        let m = simple_map();
        assert_eq!(weight_map_gradient_count(&m), 1);
    }

    #[test]
    fn test_evaluate_at_origin() {
        let m = simple_map();
        let w = weight_map_evaluate(&m, [0.0, 0.0, 0.0]);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_at_edge() {
        let m = simple_map();
        let w = weight_map_evaluate(&m, [1.0, 0.0, 0.0]);
        assert!(w < 1e-6);
    }

    #[test]
    fn test_evaluate_halfway() {
        let m = simple_map();
        let w = weight_map_evaluate(&m, [0.5, 0.0, 0.0]);
        assert!((w - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_clear() {
        let mut m = simple_map();
        weight_map_clear(&mut m);
        assert_eq!(weight_map_gradient_count(&m), 0);
    }

    #[test]
    fn test_to_json_contains_count() {
        let m = simple_map();
        let json = weight_map_to_json(&m);
        assert!(json.contains("gradient_count"));
        assert!(json.contains("center"));
    }

    #[test]
    fn test_max_weight_over_samples() {
        let m = simple_map();
        let samples = vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mx = weight_map_max_weight(&m, &samples);
        assert!((mx - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_scales_peaks() {
        let mut m = new_blend_weight_map(default_weight_map_config());
        weight_map_add_gradient(
            &mut m,
            WeightMapGradient {
                origin: [0.0, 0.0, 0.0],
                radius: 2.0,
                peak: 2.0,
                name: "big".into(),
            },
        );
        let samples = vec![[0.0, 0.0, 0.0]];
        weight_map_normalize(&mut m, &samples);
        let w = weight_map_evaluate(&m, [0.0, 0.0, 0.0]);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_invert_changes_peaks() {
        let mut m = simple_map(); // peak = 1.0, max_weight = 1.0
        weight_map_invert(&mut m);
        // peak becomes max_weight - 1.0 = 0.0
        let w = weight_map_evaluate(&m, [0.0, 0.0, 0.0]);
        assert!(w < 1e-6);
    }
}
