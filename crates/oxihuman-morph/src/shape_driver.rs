//! Data-driven shape target driver.
//!
//! Maps a float parameter in [0.0, 1.0] to a morph weight through a piecewise-linear
//! lookup table (sorted `ShapeKey` entries). This lets artists author complex
//! weight curves without requiring runtime curve evaluation.

/// Configuration for the shape driver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeDriverConfig {
    /// Maximum number of shape keys.
    pub max_keys: usize,
    /// Whether the driver is enabled.
    pub enabled: bool,
}

#[allow(dead_code)]
impl ShapeDriverConfig {
    fn new() -> Self {
        Self {
            max_keys: 64,
            enabled: true,
        }
    }
}

/// Returns the default shape driver configuration.
#[allow(dead_code)]
pub fn default_shape_driver_config() -> ShapeDriverConfig {
    ShapeDriverConfig::new()
}

/// A single control point in the lookup table.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeKey {
    /// Parameter value at this key [0.0, 1.0].
    pub param: f32,
    /// Morph weight at this key [0.0, 1.0].
    pub weight: f32,
}

/// Data-driven shape target driver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeDriver {
    config: ShapeDriverConfig,
    keys: Vec<ShapeKey>,
    current_param: f32,
}

/// Creates a new `ShapeDriver` with the given configuration.
#[allow(dead_code)]
pub fn new_shape_driver(config: ShapeDriverConfig) -> ShapeDriver {
    ShapeDriver {
        config,
        keys: Vec::new(),
        current_param: 0.0,
    }
}

/// Adds a shape key (param → weight) and keeps the key list sorted by param.
/// Returns `false` if the key limit is reached.
#[allow(dead_code)]
pub fn shape_driver_add_key(driver: &mut ShapeDriver, param: f32, weight: f32) -> bool {
    if driver.keys.len() >= driver.config.max_keys {
        return false;
    }
    let p = param.clamp(0.0, 1.0);
    let w = weight.clamp(0.0, 1.0);
    // Replace if param already exists.
    for k in &mut driver.keys {
        if (k.param - p).abs() < f32::EPSILON {
            k.weight = w;
            return true;
        }
    }
    driver.keys.push(ShapeKey { param: p, weight: w });
    driver.keys.sort_by(|a, b| a.param.partial_cmp(&b.param).unwrap_or(std::cmp::Ordering::Equal));
    true
}

/// Evaluates the weight at the given parameter using piecewise-linear interpolation.
#[allow(dead_code)]
pub fn shape_driver_evaluate(driver: &ShapeDriver, param: f32) -> f32 {
    if !driver.config.enabled || driver.keys.is_empty() {
        return 0.0;
    }
    let p = param.clamp(0.0, 1.0);
    if let Some(first) = driver.keys.first() {
        if p <= first.param {
            return first.weight;
        }
    }
    if let Some(last) = driver.keys.last() {
        if p >= last.param {
            return last.weight;
        }
    }
    for i in 0..driver.keys.len() - 1 {
        let lo = &driver.keys[i];
        let hi = &driver.keys[i + 1];
        if p >= lo.param && p <= hi.param {
            let t = (p - lo.param) / (hi.param - lo.param);
            return lo.weight + t * (hi.weight - lo.weight);
        }
    }
    0.0
}

/// Returns the number of shape keys.
#[allow(dead_code)]
pub fn shape_driver_key_count(driver: &ShapeDriver) -> usize {
    driver.keys.len()
}

/// Sets the current parameter and updates the cached weight.
#[allow(dead_code)]
pub fn shape_driver_set_param(driver: &mut ShapeDriver, param: f32) {
    driver.current_param = param.clamp(0.0, 1.0);
}

/// Returns the weight evaluated at the current parameter.
#[allow(dead_code)]
pub fn shape_driver_current_weight(driver: &ShapeDriver) -> f32 {
    shape_driver_evaluate(driver, driver.current_param)
}

/// Serialises the driver to a simple JSON string.
#[allow(dead_code)]
pub fn shape_driver_to_json(driver: &ShapeDriver) -> String {
    let keys: Vec<String> = driver
        .keys
        .iter()
        .map(|k| format!("{{\"param\":{:.4},\"weight\":{:.4}}}", k.param, k.weight))
        .collect();
    format!(
        "{{\"enabled\":{},\"current_param\":{:.4},\"keys\":[{}]}}",
        driver.config.enabled,
        driver.current_param,
        keys.join(",")
    )
}

/// Removes all shape keys and resets the parameter.
#[allow(dead_code)]
pub fn shape_driver_clear(driver: &mut ShapeDriver) {
    driver.keys.clear();
    driver.current_param = 0.0;
}

/// Returns `true` if the driver is enabled and has at least one key.
#[allow(dead_code)]
pub fn shape_driver_is_active(driver: &ShapeDriver) -> bool {
    driver.config.enabled && !driver.keys.is_empty()
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_driver() -> ShapeDriver {
        let cfg = default_shape_driver_config();
        let mut d = new_shape_driver(cfg);
        shape_driver_add_key(&mut d, 0.0, 0.0);
        shape_driver_add_key(&mut d, 0.5, 0.5);
        shape_driver_add_key(&mut d, 1.0, 1.0);
        d
    }

    #[test]
    fn test_key_count() {
        let d = make_driver();
        assert_eq!(shape_driver_key_count(&d), 3);
    }

    #[test]
    fn test_evaluate_midpoint() {
        let d = make_driver();
        let w = shape_driver_evaluate(&d, 0.25);
        assert!((w - 0.25).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_at_key() {
        let d = make_driver();
        assert!((shape_driver_evaluate(&d, 0.5) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_below_range() {
        let d = make_driver();
        assert!((shape_driver_evaluate(&d, -0.5) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_above_range() {
        let d = make_driver();
        assert!((shape_driver_evaluate(&d, 1.5) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_set_param_and_current_weight() {
        let mut d = make_driver();
        shape_driver_set_param(&mut d, 0.75);
        let w = shape_driver_current_weight(&d);
        assert!((w - 0.75).abs() < 1e-4);
    }

    #[test]
    fn test_clear() {
        let mut d = make_driver();
        shape_driver_clear(&mut d);
        assert_eq!(shape_driver_key_count(&d), 0);
        assert!(!shape_driver_is_active(&d));
    }

    #[test]
    fn test_is_active() {
        let d = make_driver();
        assert!(shape_driver_is_active(&d));
    }

    #[test]
    fn test_to_json_contains_keys() {
        let d = make_driver();
        let json = shape_driver_to_json(&d);
        assert!(json.contains("keys"));
        assert!(json.contains("enabled"));
    }

    #[test]
    fn test_max_keys_limit() {
        let cfg = ShapeDriverConfig {
            max_keys: 2,
            enabled: true,
        };
        let mut d = new_shape_driver(cfg);
        assert!(shape_driver_add_key(&mut d, 0.0, 0.0));
        assert!(shape_driver_add_key(&mut d, 1.0, 1.0));
        assert!(!shape_driver_add_key(&mut d, 0.5, 0.5));
    }
}
