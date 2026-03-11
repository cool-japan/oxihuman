//! Pose-space corrective shape driver.
//!
//! Activates a corrective blend shape when a joint reaches a specific angle.
//! The driver evaluates RBF-style key frames and returns the interpolated weight
//! of the corrective shape. This is commonly used to fix volume-loss at elbows
//! and shoulders.

/// Configuration for a corrective driver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveDriverConfig {
    /// Name of the joint this driver watches.
    pub joint_name: String,
    /// Name of the corrective blend shape activated by this driver.
    pub shape_name: String,
    /// If `true`, weight is smoothed via a simple hermite curve between keys.
    pub smooth_falloff: bool,
}

/// A single keyframe: joint angle (degrees) → corrective weight.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DriverKey {
    angle_deg: f32,
    weight: f32,
}

/// A pose-space corrective shape driver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveDriver {
    config: CorrectiveDriverConfig,
    keys: Vec<DriverKey>,
    /// Cached result of the last evaluation.
    last_weight: f32,
}

/// Result of a driver evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DriverEvalResult {
    /// Interpolated corrective weight in [0, 1].
    pub weight: f32,
    /// Whether the driver is currently contributing (weight > 0).
    pub active: bool,
    /// Name of the shape being driven.
    pub shape_name: String,
}

/// Return a default [`CorrectiveDriverConfig`].
#[allow(dead_code)]
pub fn default_corrective_driver_config() -> CorrectiveDriverConfig {
    CorrectiveDriverConfig {
        joint_name: "elbow_l".into(),
        shape_name: "elbow_corrective_l".into(),
        smooth_falloff: true,
    }
}

/// Create a new [`CorrectiveDriver`].
#[allow(dead_code)]
pub fn new_corrective_driver(config: CorrectiveDriverConfig) -> CorrectiveDriver {
    CorrectiveDriver {
        config,
        keys: Vec::new(),
        last_weight: 0.0,
    }
}

/// Add an angle → weight keyframe.  Keys are kept sorted by angle.
#[allow(dead_code)]
pub fn driver_add_key(driver: &mut CorrectiveDriver, angle_deg: f32, weight: f32) {
    let weight = weight.clamp(0.0, 1.0);
    driver.keys.push(DriverKey { angle_deg, weight });
    driver
        .keys
        .sort_by(|a, b| a.angle_deg.partial_cmp(&b.angle_deg).unwrap_or(std::cmp::Ordering::Equal));
}

/// Evaluate the driver at the given joint angle.  Performs linear interpolation
/// between neighbouring keys (or smooth hermite if `smooth_falloff` is enabled).
#[allow(dead_code)]
pub fn driver_evaluate(driver: &mut CorrectiveDriver, angle_deg: f32) -> DriverEvalResult {
    let w = if driver.keys.is_empty() {
        0.0f32
    } else if driver.keys.len() == 1 {
        driver.keys[0].weight
    } else {
        // Find bracketing keys
        let n = driver.keys.len();
        if angle_deg <= driver.keys[0].angle_deg {
            driver.keys[0].weight
        } else if angle_deg >= driver.keys[n - 1].angle_deg {
            driver.keys[n - 1].weight
        } else {
            let mut lo = 0usize;
            for i in 0..n - 1 {
                if driver.keys[i].angle_deg <= angle_deg
                    && angle_deg < driver.keys[i + 1].angle_deg
                {
                    lo = i;
                    break;
                }
            }
            let k0 = &driver.keys[lo];
            let k1 = &driver.keys[lo + 1];
            let range = k1.angle_deg - k0.angle_deg;
            let t = if range.abs() < 1e-9 {
                0.0
            } else {
                (angle_deg - k0.angle_deg) / range
            };
            let t = t.clamp(0.0, 1.0);
            if driver.config.smooth_falloff {
                // Hermite smooth-step
                let t2 = t * t;
                let t3 = t2 * t;
                let s = 3.0 * t2 - 2.0 * t3;
                k0.weight + s * (k1.weight - k0.weight)
            } else {
                k0.weight + t * (k1.weight - k0.weight)
            }
        }
    };

    driver.last_weight = w;
    DriverEvalResult {
        weight: w,
        active: w > 1e-6,
        shape_name: driver.config.shape_name.clone(),
    }
}

/// Return the number of keyframes registered.
#[allow(dead_code)]
pub fn driver_key_count(driver: &CorrectiveDriver) -> usize {
    driver.keys.len()
}

/// Return `true` if the last evaluated weight was greater than zero.
#[allow(dead_code)]
pub fn driver_is_active(driver: &CorrectiveDriver) -> bool {
    driver.last_weight > 1e-6
}

/// Return the maximum weight value across all keyframes.
#[allow(dead_code)]
pub fn driver_peak_weight(driver: &CorrectiveDriver) -> f32 {
    driver
        .keys
        .iter()
        .map(|k| k.weight)
        .fold(0.0f32, f32::max)
}

/// Remove all keyframes.
#[allow(dead_code)]
pub fn driver_clear_keys(driver: &mut CorrectiveDriver) {
    driver.keys.clear();
}

/// Serialize the driver to a compact JSON string.
#[allow(dead_code)]
pub fn driver_to_json(driver: &CorrectiveDriver) -> String {
    let keys_json: Vec<String> = driver
        .keys
        .iter()
        .map(|k| format!(r#"{{"angle":{:.2},"weight":{:.4}}}"#, k.angle_deg, k.weight))
        .collect();
    format!(
        r#"{{"joint":"{}","shape":"{}","key_count":{},"last_weight":{:.6},"keys":[{}]}}"#,
        driver.config.joint_name,
        driver.config.shape_name,
        driver.keys.len(),
        driver.last_weight,
        keys_json.join(",")
    )
}

/// Reset the cached last-weight to zero.
#[allow(dead_code)]
pub fn driver_reset(driver: &mut CorrectiveDriver) {
    driver.last_weight = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_driver() -> CorrectiveDriver {
        let mut d = new_corrective_driver(default_corrective_driver_config());
        driver_add_key(&mut d, 0.0, 0.0);
        driver_add_key(&mut d, 90.0, 1.0);
        driver_add_key(&mut d, 180.0, 0.0);
        d
    }

    #[test]
    fn test_default_config() {
        let cfg = default_corrective_driver_config();
        assert!(!cfg.joint_name.is_empty());
        assert!(!cfg.shape_name.is_empty());
    }

    #[test]
    fn test_key_count() {
        let d = make_driver();
        assert_eq!(driver_key_count(&d), 3);
    }

    #[test]
    fn test_evaluate_at_peak() {
        let mut d = make_driver();
        let res = driver_evaluate(&mut d, 90.0);
        assert!((res.weight - 1.0).abs() < 1e-5);
        assert!(res.active);
    }

    #[test]
    fn test_evaluate_at_zero() {
        let mut d = make_driver();
        let res = driver_evaluate(&mut d, 0.0);
        assert!(res.weight < 1e-6);
        assert!(!res.active);
    }

    #[test]
    fn test_evaluate_midpoint_linear() {
        let cfg = CorrectiveDriverConfig {
            smooth_falloff: false,
            ..default_corrective_driver_config()
        };
        let mut d = new_corrective_driver(cfg);
        driver_add_key(&mut d, 0.0, 0.0);
        driver_add_key(&mut d, 100.0, 1.0);
        let res = driver_evaluate(&mut d, 50.0);
        assert!((res.weight - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_peak_weight() {
        let d = make_driver();
        assert!((driver_peak_weight(&d) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear_keys() {
        let mut d = make_driver();
        driver_clear_keys(&mut d);
        assert_eq!(driver_key_count(&d), 0);
    }

    #[test]
    fn test_is_active_after_evaluate() {
        let mut d = make_driver();
        driver_evaluate(&mut d, 90.0);
        assert!(driver_is_active(&d));
    }

    #[test]
    fn test_reset_clears_last_weight() {
        let mut d = make_driver();
        driver_evaluate(&mut d, 90.0);
        driver_reset(&mut d);
        assert!(!driver_is_active(&d));
    }

    #[test]
    fn test_to_json() {
        let d = make_driver();
        let json = driver_to_json(&d);
        assert!(json.contains("key_count"));
        assert!(json.contains("elbow_corrective_l"));
    }

    #[test]
    fn test_keys_sorted_on_insert() {
        let mut d = new_corrective_driver(default_corrective_driver_config());
        driver_add_key(&mut d, 180.0, 0.0);
        driver_add_key(&mut d, 0.0, 0.0);
        driver_add_key(&mut d, 90.0, 1.0);
        // After sorting, key 0 should be angle 0, key 1 = 90, key 2 = 180
        assert_eq!(driver_key_count(&d), 3);
        let mut d2 = new_corrective_driver(default_corrective_driver_config());
        driver_add_key(&mut d2, 0.0, 0.0);
        driver_add_key(&mut d2, 90.0, 1.0);
        driver_add_key(&mut d2, 180.0, 0.0);
        let r1 = driver_evaluate(&mut d, 45.0);
        let r2 = driver_evaluate(&mut d2, 45.0);
        assert!((r1.weight - r2.weight).abs() < 1e-5);
    }
}
